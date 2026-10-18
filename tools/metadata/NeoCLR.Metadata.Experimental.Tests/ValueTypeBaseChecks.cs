using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ValueTypeBaseChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", new Version(10, 0, 0, 0));
        var library = new AssemblyBuilder(new("ValueFoundation", new Version(1, 0, 0, 0)), core);
        var foundation = library.AddClass("System", "ValueType");
        foundation.SetAbstractClass();
        var snapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(library));
        var app = new AssemblyBuilder(new("ValueConsumer", new Version(1, 0, 0, 0)), core);
        var parent = app.ImportReference(snapshot.MainModule.Types.Single(), core);
        var value = app.AddValueType("Example", "Value");
        Reject<ArgumentException>(() => value.SetValueTypeBase(parent));
        app.CompleteClassBaseReference(parent, [], 0);
        value.SetValueTypeBase(parent);
        Reject<InvalidOperationException>(() => value.SetValueTypeBase(parent));
        value.AddConstructor([]).Return();
        var read = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(app));
        var definition = read.MainModule.Types.Single();
        if (!definition.IsValueType || definition.BaseType?.Name != "ValueType")
            throw new Exception("native value category or ancestry lost");
        Reject<InvalidOperationException>(() => app.AddClass("Example", "Reference").SetValueTypeBase(parent));
        var other = new AssemblyBuilder(new("Other", new Version(1, 0, 0, 0)), core);
        var stored = other.ImportReference(snapshot.MainModule.Types.Single(), core);
        other.CompleteClassBaseReference(stored, [], 1);
        Reject<ArgumentException>(() => other.AddValueType("Example", "Stored").SetValueTypeBase(stored));
        Reject<ArgumentException>(() => other.AddValueType("Example", "Foreign").SetValueTypeBase(parent));
        Console.WriteLine("PASS fieldless native ValueType base contracts");
    }

    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
