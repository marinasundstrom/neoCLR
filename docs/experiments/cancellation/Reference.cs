using System;
using System.Threading;

static class Program
{
    static void Check(bool value)
    {
        if (!value) throw new Exception("Cancellation baseline failed");
    }

    static void Main()
    {
        Check(!CancellationToken.None.CanBeCanceled);
        using var source = new CancellationTokenSource();
        var token = source.Token;
        var copy = token;
        var order = 0;
        using var first = token.Register(() => {
            Check(token.IsCancellationRequested);
            order = order * 10 + 1;
        });
        using var removed = token.Register(() => throw new Exception("Removed callback ran"));
        using var second = token.Register(() => {
            source.Cancel();
            order = order * 10 + 2;
            removed.Dispose();
            using var inline = token.Register(() => order = order * 10 + 3);
        });
        source.Cancel();
        source.Cancel();
        Check(order == 231 && copy.IsCancellationRequested);
        var disposed = new CancellationTokenSource();
        var old = disposed.Token;
        using var unused = old.Register(() => throw new Exception("Dispose cancelled"));
        disposed.Dispose();
        Check(!old.IsCancellationRequested);
        Console.WriteLine(".NET 10 shared cancellation behavior passed");
    }
}
