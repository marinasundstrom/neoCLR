using System.Net;
using System.Net.Sockets;

static void Check(bool condition, string message)
{
    if (!condition) throw new Exception(message);
}

using var watchdog = new CancellationTokenSource(TimeSpan.FromSeconds(5));
var token = watchdog.Token;
using var listener = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
listener.Bind(new IPEndPoint(IPAddress.Loopback, 0));
listener.Listen(4);
using var client = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
await client.ConnectAsync(listener.LocalEndPoint!, token);
using var server = await listener.AcceptAsync(token);
Check(Equals(client.RemoteEndPoint, server.LocalEndPoint), "endpoint identity");

var bytes = new byte[4];
using (var cancellation = CancellationTokenSource.CreateLinkedTokenSource(token))
{
    var pending = server.ReceiveAsync(bytes.AsMemory(), SocketFlags.None, cancellation.Token);
    Check(!pending.IsCompleted, "receive should wait for data");
    cancellation.Cancel();
    try
    {
        await pending;
        throw new Exception("expected cancellation");
    }
    catch (OperationCanceledException) { }
}

var payload = new byte[] { 1, 2, 3 };
var sent = 0;
while (sent < payload.Length)
{
    var count = await client.SendAsync(payload.AsMemory(sent), SocketFlags.None, token);
    Check(count > 0, "send progress");
    sent += count;
}
client.Shutdown(SocketShutdown.Send);
var received = new List<byte>();
while (true)
{
    var count = await server.ReceiveAsync(bytes.AsMemory(0, 2), SocketFlags.None, token);
    if (count == 0) break;
    received.AddRange(bytes.Take(count));
}
Check(received.SequenceEqual(payload), "payload before EOF");
Check(await server.SendAsync(new byte[] { 9 }.AsMemory(), SocketFlags.None, token) == 1, "reverse send");
Check(await client.ReceiveAsync(bytes.AsMemory(), SocketFlags.None, token) == 1 && bytes[0] == 9, "reverse receive");
Console.WriteLine("PASS: endpoints, pending receive cancellation, short reads, EOF, half-close");
