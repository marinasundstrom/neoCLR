using System.Collections.Concurrent;
using System.Diagnostics;
using System.Net;
using System.Net.Sockets;
using System.Text.Json;

sealed class OwnerContext : SynchronizationContext
{
    readonly BlockingCollection<(SendOrPostCallback, object?)> queue = new();
    public override void Post(SendOrPostCallback callback, object? state) => queue.Add((callback, state));
    public void Run(Func<Task> start)
    {
        SetSynchronizationContext(this);
        Task task = start();
        task.ContinueWith(_ => queue.CompleteAdding(), CancellationToken.None,
            TaskContinuationOptions.ExecuteSynchronously, TaskScheduler.Default);
        foreach (var (callback, state) in queue.GetConsumingEnumerable()) callback(state);
        task.GetAwaiter().GetResult();
    }
}

static class Program
{
    static ulong Compute(int iterations)
    {
        ulong x = 1;
        for (int n = 0; n < iterations; n++) { x ^= x << 13; x ^= x >> 7; x ^= x << 17; }
        return x;
    }
    static void Main(string[] args)
    {
        string mode = args[0]; int rounds = int.Parse(args[1]), iterations = int.Parse(args[2]);
        if (mode is not ("idle" or "inline" or "offload") || rounds < 1 || iterations < 1) throw new ArgumentException();
        // One owner plus at most one in-flight compute task. Pool/IO/runtime helper
        // threads are not constrained to two OS threads by this application bound.
        new OwnerContext().Run(async () =>
        {
            int owner = Environment.CurrentManagedThreadId;
            using var listener = new TcpListener(IPAddress.Loopback, 0);
            listener.Start(1);
            Console.WriteLine(((IPEndPoint)listener.LocalEndpoint).Port);
            using var socket = await listener.AcceptSocketAsync();
            socket.NoDelay = true;
            byte[] received = new byte[1], ready = [(byte)'R'], ack = [(byte)'A'];
            var cpuStart = Process.GetCurrentProcess().TotalProcessorTime;
            var watch = Stopwatch.StartNew();
            ulong checksum = 0;
            for (int round = 0; round < rounds; round++)
            {
                Task receive = Receive();
                if (socket.Send(ready) != 1) throw new IOException();
                Task<ulong>? work = mode == "offload" ? Task.Run(() => Compute(iterations)) : null;
                if (mode == "inline") checksum = Compute(iterations);
                await receive;
                if (work != null) checksum = await work;
            }
            Console.WriteLine(JsonSerializer.Serialize(new {
                mode, rounds, iterations, checksum = checksum.ToString(),
                seconds = watch.Elapsed.TotalSeconds,
                cpuSeconds = (Process.GetCurrentProcess().TotalProcessorTime - cpuStart).TotalSeconds,
                runtime = Environment.Version.ToString()
            }));
            async Task Receive()
            {
                if (await socket.ReceiveAsync(received, SocketFlags.None) != 1 || received[0] != (byte)'x') throw new IOException();
                if (Environment.CurrentManagedThreadId != owner) throw new InvalidOperationException("owner affinity lost");
                if (socket.Send(ack) != 1) throw new IOException();
            }
        });
    }
}
