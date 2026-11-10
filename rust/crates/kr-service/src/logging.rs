//! Bounded JSONL writer. A failed evidence sink must fail the service visibly.
use serde_json::Value;
use std::{io, time::Duration};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    sync::mpsc,
    task::JoinHandle,
    time::timeout,
};

pub(crate) async fn write_events<W: AsyncWrite + Unpin>(
    mut sink: W,
    mut events: mpsc::Receiver<Value>,
    deadline: Duration,
) -> io::Result<()> {
    let mut ordinal = 0u64;
    while let Some(value) = events.recv().await {
        ordinal += 1;
        let mut data = serde_json::to_vec(&value).map_err(io::Error::other)?;
        data.push(b'\n');
        let context =
            |phase, error| io::Error::other(format!("log {phase} at event {ordinal}: {error}"));
        timeout(deadline, sink.write_all(&data))
            .await
            .map_err(|e| context("write timeout", e.to_string()))?
            .map_err(|e| context("write", e.to_string()))?;
        timeout(deadline, sink.flush())
            .await
            .map_err(|e| context("flush timeout", e.to_string()))?
            .map_err(|e| context("flush", e.to_string()))?;
    }
    Ok(())
}

pub(crate) fn joined(result: Result<io::Result<()>, tokio::task::JoinError>) -> io::Result<()> {
    result.map_err(|e| io::Error::other(format!("log writer task: {e}")))?
}

/// Always join even when the final event cannot be queued. Prefer the sink's
/// original error to the secondary closed-channel error from emit(stopped).
pub(crate) async fn finish(
    mut writer: JoinHandle<io::Result<()>>,
    observed: Option<io::Result<()>>,
    final_event: io::Result<()>,
) -> io::Result<()> {
    let result = if let Some(result) = observed {
        result
    } else {
        match timeout(Duration::from_secs(3), &mut writer).await {
            Ok(result) => joined(result),
            Err(_) => {
                writer.abort();
                let _ = writer.await;
                Err(io::Error::other("logger shutdown timeout"))
            }
        }
    };
    result?;
    final_event
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    enum Fault {
        Write,
        Flush,
        PendingWrite,
        PendingFlush,
    }
    struct Sink(Fault);
    impl AsyncWrite for Sink {
        fn poll_write(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            match self.0 {
                Fault::Write => Poll::Ready(Err(io::Error::other("injected ENOSPC"))),
                Fault::PendingWrite => Poll::Pending,
                _ => Poll::Ready(Ok(buf.len())),
            }
        }
        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            match self.0 {
                Fault::Flush => Poll::Ready(Err(io::Error::other("injected flush failure"))),
                Fault::PendingFlush => Poll::Pending,
                _ => Poll::Ready(Ok(())),
            }
        }
        fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn sink_failure_survives_closed_channel_and_cleanup() {
        for (fault, expected) in [
            (Fault::Write, "log write at event 1: injected ENOSPC"),
            (Fault::Flush, "log flush at event 1: injected flush failure"),
            (Fault::PendingWrite, "log write timeout at event 1"),
            (Fault::PendingFlush, "log flush timeout at event 1"),
        ] {
            let (tx, rx) = mpsc::channel(2);
            let writer = tokio::spawn(write_events(Sink(fault), rx, Duration::from_millis(10)));
            tx.send(json!({"event":"boot"})).await.unwrap();
            // Observe sink closure without waiting for another network request.
            timeout(Duration::from_secs(1), tx.closed()).await.unwrap();
            let stopped = tx
                .send(json!({"event":"stopped"}))
                .await
                .map_err(io::Error::other);
            assert!(stopped.is_err());
            drop(tx);
            let error = finish(writer, None, stopped).await.unwrap_err().to_string();
            assert!(error.starts_with(expected), "{error}");
        }
    }

    #[tokio::test]
    async fn supervision_preserves_error_and_clean_shutdown_drains_all_events() {
        let (tx, rx) = mpsc::channel(2);
        let mut writer = tokio::spawn(write_events(Sink(Fault::Flush), rx, Duration::from_secs(2)));
        tx.send(json!({"event":"ready"})).await.unwrap();
        let observed = timeout(Duration::from_secs(1), &mut writer).await.unwrap();
        drop(tx);
        let error = finish(
            writer,
            Some(joined(observed)),
            Err(io::Error::other("unavailable")),
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("injected flush failure"));

        let (tx, rx) = mpsc::channel(2);
        let (sink, mut reader) = tokio::io::duplex(1024);
        let writer = tokio::spawn(write_events(sink, rx, Duration::from_secs(2)));
        tx.send(json!({"event":"boot"})).await.unwrap();
        tx.send(json!({"event":"stopped"})).await.unwrap();
        drop(tx);
        finish(writer, None, Ok(())).await.unwrap();
        use tokio::io::AsyncReadExt;
        let mut bytes = String::new();
        reader.read_to_string(&mut bytes).await.unwrap();
        assert_eq!(bytes, "{\"event\":\"boot\"}\n{\"event\":\"stopped\"}\n");
    }
}
