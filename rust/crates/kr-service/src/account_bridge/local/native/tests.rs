use super::*;
use crate::account_bridge::local::tests::{active, channel, login, trial};
use crate::account_bridge::tests::{Temp, rt, worker};
use crate::accounts::TEST_SERIAL;
use tokio::io::DuplexStream;

fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn templates() -> Templates {
    Templates {
        lobby: Some(LobbyTemplates {
            ready: vec![0x5a],
            channel_list: vec![0x5a],
        }),
        account: Arc::new(
            AccountTemplate::parse(
                &hex(fixtures::AUTH),
                &hex(fixtures::OWN),
                &hex(fixtures::PLAYER),
            )
            .unwrap(),
        ),
        header: [0; HEADER_LEN],
        netinfo_header: [0; HEADER_LEN],
        flags_header: [0; HEADER_LEN],
        redirect_header: [0; HEADER_LEN],
        channel_header: [0; HEADER_LEN],
        netinfo: vec![0x5a],
        redirect: vec![0x5b],
        channel: vec![0x5a],
    }
}
fn input(opcode: u32, body: Vec<u8>) -> Input {
    Input::new(opcode, Zeroizing::new(body)).unwrap()
}
async fn receive(stream: &mut DuplexStream, sequence: &mut u32) -> Input {
    read(stream, sequence).await.unwrap()
}

#[test]
fn one_or_two_db_principals_follow_native_reply_sequence_into_the_same_owned_channel() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, a, b) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        for guest in [false, true] {
            for names in [&["Alice"][..], &["Alice", "Bob"][..]] {
                let session = LocalSession::new(worker.client(), 1).unwrap();
                let mut driver = Driver::new(session, templates()).unwrap();
                let mut login_lane = Lane::login(1).unwrap();
                let (mut writer, mut peer) = tokio::io::duplex(4096);
                let mut sequence = 43;
                driver
                    .packet(
                        &mut login_lane,
                        if guest {
                            input(62, trial(names.len() as u8))
                        } else {
                            input(1, login(names))
                        },
                        &mut writer,
                        60,
                        None,
                    )
                    .await
                    .unwrap();
                let auth = receive(&mut peer, &mut sequence).await;
                assert_eq!(auth.opcode, 4);
                assert_eq!(
                    kr_wire::read_uint(&auth.payload[4..5], Width::U8).unwrap(),
                    names.len() as u64
                );
                let rows = {
                    let state = driver.session.shared.lock().unwrap();
                    state
                        .grants
                        .iter()
                        .zip(&state.echoes)
                        .map(|(g, e)| (g.player.clone(), *e))
                        .collect::<Vec<_>>()
                };
                if guest {
                    assert!(
                        rows.iter()
                            .all(|r| r.0.account_id != a.account_id
                                && r.0.account_id != b.account_id)
                    );
                } else {
                    assert_eq!(
                        rows.iter().map(|r| &r.0).collect::<Vec<_>>(),
                        vec![&a, &b][..names.len()]
                    );
                }
                assert_eq!(active(&temp), names.len() as i64);
                driver
                    .packet(&mut login_lane, input(9, vec![0; 8]), &mut writer, 60, None)
                    .await
                    .unwrap();
                assert_eq!(receive(&mut peer, &mut sequence).await.opcode, 15);
                for slot in 0..names.len() as u64 {
                    let flags = receive(&mut peer, &mut sequence).await;
                    assert_eq!(flags.opcode, 62);
                    assert_eq!(
                        kr_wire::read_uint(&flags.payload[1..2], Width::U8).unwrap(),
                        slot
                    );
                    assert_eq!(
                        kr_wire::read_uint(&flags.payload[2..3], Width::U8).unwrap(),
                        2
                    );
                }
                driver
                    .packet(&mut login_lane, input(5, vec![0; 9]), &mut writer, 60, None)
                    .await
                    .unwrap();
                assert_eq!(receive(&mut peer, &mut sequence).await.opcode, 7);
                assert_eq!(sequence, 43 + 3 * (3 + names.len() as u32));
                assert!(!driver.disconnected(&login_lane).await.unwrap());
                let mut channel_lane = Lane::channel(2).unwrap();
                let (mut out, mut remote) = tokio::io::duplex(4096);
                driver
                    .packet(
                        &mut channel_lane,
                        input(6, channel(&rows)),
                        &mut out,
                        60,
                        None,
                    )
                    .await
                    .unwrap();
                let mut fresh_sequence = 43;
                assert_eq!(receive(&mut remote, &mut fresh_sequence).await.opcode, 17);
                assert!(
                    driver
                        .authorities()
                        .iter()
                        .all(|h| h.current() && h.connection() == 2)
                );
                let handles = driver.authorities().to_vec();
                let request = input(67, write_uint(0, Width::U16).unwrap());
                assert!(request.lobby_ready());
                assert!(!input(67, vec![0; 2]).lobby_ready());
                assert!(!input(67, write_uint(1, Width::U16).unwrap()).lobby_ready());
                driver
                    .enter_lobby(&mut channel_lane, &mut out, &[(91, vec![0x5a])])
                    .await
                    .unwrap();
                for opcode in [241, 91, 14] {
                    assert_eq!(
                        receive(&mut remote, &mut fresh_sequence).await.opcode,
                        opcode
                    );
                }
                let own = receive(&mut remote, &mut fresh_sequence).await;
                assert_eq!(own.opcode, 291);
                let names: Vec<_> = rows
                    .iter()
                    .map(|(player, _)| {
                        kr_wire::kr00967::text949x::encode_name(&player.display_name).unwrap()
                    })
                    .collect();
                let refs: Vec<_> = names.iter().map(Vec::as_slice).collect();
                assert_eq!(
                    own.payload.as_slice(),
                    kr_wire::kr00967::local_own291(&refs).unwrap()
                );
                driver
                    .packet(
                        &mut channel_lane,
                        input(87, write_uint(0, Width::U16).unwrap()),
                        &mut out,
                        60,
                        None,
                    )
                    .await
                    .unwrap();
                driver
                    .lobby_reply(&mut channel_lane, &mut out, 238, &[0x5a])
                    .await
                    .unwrap();
                assert_eq!(receive(&mut remote, &mut fresh_sequence).await.opcode, 238);
                assert!(driver.disconnected(&channel_lane).await.unwrap());
                assert!(handles.iter().all(|h| !h.current()));
                assert_eq!(active(&temp), 0);
                driver.finish().await.unwrap();
            }
        }
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn wrong_password_and_unknown_second_principal_share_one_denial_and_no_partial_grant() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, _, _) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        let mut replies = Vec::new();
        let unknown = login(&["Alice", "Missing"]);
        let mut wrong = login(&["Alice", "Bob"]);
        let index = wrong.len() - 26;
        wrong[index] ^= 1;
        for body in [unknown, wrong] {
            let mut driver =
                Driver::new(LocalSession::new(worker.client(), 1).unwrap(), templates()).unwrap();
            let mut lane = Lane::login(1).unwrap();
            let (mut writer, mut peer) = tokio::io::duplex(4096);
            assert!(
                driver
                    .packet(&mut lane, input(1, body), &mut writer, 60, None)
                    .await
                    .is_err()
            );
            let mut sequence = 43;
            let reply = receive(&mut peer, &mut sequence).await;
            assert_eq!(reply.opcode, 4);
            replies.push(reply.payload.to_vec());
            assert_eq!(active(&temp), 0);
            assert!(driver.authorities().is_empty());
            driver.finish().await.unwrap();
        }
        assert_eq!(replies, vec![vec![0x5b], vec![0x5b]]);
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn cancelled_channel_reply_retires_already_projected_handles() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, _, _) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        let mut session = LocalSession::new(worker.client(), 1).unwrap();
        let rows = session
            .login(&login(&["Alice", "Bob"]), 60, None)
            .await
            .unwrap();
        session.redirected(1).unwrap();
        let mut driver = Driver::new(session, templates()).unwrap();
        let mut lane = Lane::channel(2).unwrap();
        let (mut writer, _peer) = tokio::io::duplex(1);
        // A one-byte output capacity holds delivery after roster validation.
        // Observe that state, then cancel the future without a write error.
        let state = driver.session.shared.clone();
        let mut pending =
            Box::pin(driver.packet(&mut lane, input(6, channel(&rows)), &mut writer, 60, None));
        tokio::select! {
            result = &mut pending => panic!("unexpected completed delivery: {result:?}"),
            _ = async {
                while state.lock().unwrap().phase != super::super::Phase::Bound {
                    tokio::task::yield_now().await;
                }
            } => {}
        }
        drop(pending);
        assert_eq!(driver.authorities().len(), 2);
        assert!(
            driver
                .authorities()
                .iter()
                .all(|handle| !handle.current() && *handle.stopped().borrow())
        );
        driver.finish().await.unwrap();
        assert_eq!(active(&temp), 0);
        worker.shutdown().await.unwrap();
    });
}

#[test]
fn delivery_failure_retires_the_roster_and_crc_failure_never_advances_secret_reader() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let (temp, _, _) = Temp::new();
    rt().block_on(async {
        let worker = worker(&temp).await;
        let mut driver =
            Driver::new(LocalSession::new(worker.client(), 1).unwrap(), templates()).unwrap();
        let mut lane = Lane::login(1).unwrap();
        let (mut writer, peer) = tokio::io::duplex(1);
        drop(peer);
        assert!(
            driver
                .packet(
                    &mut lane,
                    input(1, login(&["Alice", "Bob"])),
                    &mut writer,
                    60,
                    None
                )
                .await
                .is_err()
        );
        assert!(driver.session.shared.lock().unwrap().phase == super::super::Phase::Ended);
        driver.finish().await.unwrap();
        assert_eq!(active(&temp), 0);
        worker.shutdown().await.unwrap();
        let frame = kr_wire::encode(&[0; HEADER_LEN], 1, &login(&["Alice"]), 40).unwrap();
        let (mut writer, mut peer) = tokio::io::duplex(1024);
        let mut bad = frame.clone();
        *bad.last_mut().unwrap() ^= 1;
        writer.write_all(&bad).await.unwrap();
        let mut sequence = 40;
        assert!(read(&mut peer, &mut sequence).await.is_err());
        assert_eq!(sequence, 40);
        let (mut writer, mut peer) = tokio::io::duplex(1024);
        let sender = tokio::spawn(async move {
            writer.write_all(&frame[..17]).await.unwrap();
            tokio::task::yield_now().await;
            writer.write_all(&frame[17..]).await.unwrap();
        });
        assert_eq!(read(&mut peer, &mut sequence).await.unwrap().opcode, 1);
        assert_eq!(sequence, 43);
        sender.await.unwrap();
    });
}

#[test]
fn idle_socket_uses_owner_lifetime_but_partial_frame_keeps_its_read_deadline() {
    rt().block_on(async {
        let frame = kr_wire::encode(&[0; HEADER_LEN], 67, &[0x5a, 0x5a], 40).unwrap();
        let (mut writer, mut peer) = tokio::io::duplex(1024);
        let sender = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(2200)).await;
            writer.write_all(&frame).await.unwrap();
        });
        let mut sequence = 40;
        assert_eq!(read(&mut peer, &mut sequence).await.unwrap().opcode, 67);
        assert_eq!(sequence, 43);
        sender.await.unwrap();

        let (mut writer, mut peer) = tokio::io::duplex(1024);
        writer.write_all(&[0]).await.unwrap();
        let mut sequence = 40;
        let failure = tokio::time::timeout(Duration::from_secs(3), read(&mut peer, &mut sequence))
            .await
            .unwrap()
            .err()
            .unwrap();
        assert_eq!(failure.to_string(), "native read deadline");
        assert_eq!(sequence, 40);
    });
}
