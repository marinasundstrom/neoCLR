namespace System.Runtime.CompilerServices;

// Executable target adapters; no source-library declarations are duplicated here.
public static class CheckedStorage
{
    // CLR arrays are zero initialized. Source collections track their initialized extent.
    public static T[] Reserve<T>(int length) => new T[length];
}

public static class RuntimeFailure
{
    [System.Diagnostics.CodeAnalysis.DoesNotReturn]
    public static void Terminate(string message)
    {
        Console.Error.WriteLine(message);
        Environment.Exit(1);
    }
}

/// <summary>Empty CLR value used for the source library's inhabited unit.</summary>
public readonly struct UnitValue { }
