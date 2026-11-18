// Bounded loopback fixture/capture for LoginStage feasibility, never authentication.
using System;
using System.IO;
using System.Net;
using System.Net.Sockets;
using System.Threading;
using System.Collections.Generic;

public sealed class M8LoginCapture : IDisposable {
    private readonly List<TcpListener> listeners = new List<TcpListener>();
    private readonly List<TcpClient> clients = new List<TcpClient>();
    private readonly List<Thread> threads = new List<Thread>();
    private readonly object gate = new object();
    private readonly StreamWriter output;
    private readonly byte[] hello, wcp;
    private volatile bool stopped;
    private int connections;
    public M8LoginCapture(string path, byte[] helloBytes, byte[] wcpBytes) {
        if (helloBytes.Length != 15 || wcpBytes.Length < 1 || wcpBytes.Length > 65536)
            throw new ArgumentException("Pinned fixture size");
        hello = helloBytes; wcp = wcpBytes;
        output = new StreamWriter(new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.Read));
        output.AutoFlush = true;
        for (int port = 3808; port < 4000; port += 10) {
            TcpListener listener = new TcpListener(IPAddress.Loopback, port);
            listener.Start(4); listeners.Add(listener);
            Thread thread = new Thread(delegate() { Accept(listener); });
            thread.IsBackground = true; threads.Add(thread); thread.Start();
        }
        Log("{\"event\":\"ready\",\"scope\":\"loopback_only_no_authentication\"}");
    }
    private void Log(string value) { lock(gate) { output.WriteLine(value); } }
    private void Accept(TcpListener listener) {
        while (!stopped) {
            TcpClient client;
            try { client = listener.AcceptTcpClient(); } catch (SocketException) { break; }
            int id = Interlocked.Increment(ref connections);
            if (id > 32) { client.Close(); continue; }
            lock(gate) { clients.Add(client); }
            Thread thread = new Thread(delegate() { Serve(client, id); });
            thread.IsBackground = true;
            lock(gate) { threads.Add(thread); }
            thread.Start();
        }
    }
    private void Serve(TcpClient client, int id) {
        int port = ((IPEndPoint)client.Client.LocalEndPoint).Port;
        Log("{\"event\":\"accepted\",\"connection\":"+id+",\"port\":"+port+"}");
        try {
            client.ReceiveTimeout = 1000; client.SendTimeout = 2000;
            NetworkStream stream = client.GetStream();
            stream.Write(hello, 0, hello.Length);
            byte[] buffer = new byte[4096]; byte[] first = new byte[9]; int filled = 0, total = 0;
            bool decided = false;
            DateTime deadline = DateTime.UtcNow.AddSeconds(240);
            while (!stopped && total < 65536 && DateTime.UtcNow < deadline) {
                int count;
                try { count = stream.Read(buffer, 0, Math.Min(buffer.Length, 65536-total)); }
                catch (IOException error) {
                    SocketException socketError = error.InnerException as SocketException;
                    if (socketError != null && socketError.SocketErrorCode == SocketError.TimedOut) continue;
                    throw;
                }
                if (count == 0) break;
                total += count;
                Log("{\"event\":\"received\",\"connection\":"+id+",\"port\":"+port+
                    ",\"utc\":\""+DateTime.UtcNow.ToString("o")+"\",\"hex\":\""+
                    BitConverter.ToString(buffer,0,count).Replace("-","").ToLowerInvariant()+"\"}");
                int copied = Math.Min(9-filled,count); Array.Copy(buffer,0,first,filled,copied); filled += copied;
                if (!decided && filled == 9) {
                    decided = true;
                    if (BitConverter.ToUInt32(first,5) == 0) {
                        stream.Write(wcp,0,wcp.Length);
                        Log("{\"event\":\"wcp_fixture_sent\",\"connection\":"+id+"}");
                    }
                }
            }
        } catch (Exception error) {
            Log("{\"event\":\"connection_error\",\"connection\":"+id+",\"type\":\""+error.GetType().Name+"\"}");
        } finally { client.Close(); Log("{\"event\":\"closed\",\"connection\":"+id+"}"); }
    }
    public void Dispose() {
        stopped = true;
        foreach (TcpListener listener in listeners) listener.Stop();
        TcpClient[] live; Thread[] running;
        lock(gate) { live = clients.ToArray(); running = threads.ToArray(); }
        foreach (TcpClient client in live) client.Close();
        foreach (Thread thread in running) if (thread != Thread.CurrentThread) thread.Join(3000);
        Log("{\"event\":\"stopped\",\"connections\":"+connections+"}"); output.Dispose();
    }
}
