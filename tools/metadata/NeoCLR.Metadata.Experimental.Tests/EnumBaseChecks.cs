using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class EnumBaseChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", new Version(10, 0, 0, 0));
        var library = new AssemblyBuilder(new("EnumFoundation", new Version(1, 0, 0, 0)), core);
        library.AddClass("System", "Enum").SetAbstractClass();
        var snapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(library));
        var app = new AssemblyBuilder(new("EnumConsumer", new Version(1, 0, 0, 0)), core);
        var parent = app.ImportReference(snapshot.MainModule.Types.Single(), core);
        var value = app.AddEnum("Example", "Color");
        value.AddEnumMember("Red", 1);
        Reject<ArgumentException>(() => value.SetEnumBase(parent));
        app.CompleteClassBaseReference(parent, [], 0);
        value.SetEnumBase(parent);
        Reject<InvalidOperationException>(() => value.SetEnumBase(parent));
        Reject<InvalidOperationException>(() => app.AddValueType("Example", "Value").SetEnumBase(parent));
        var read = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(app));
        var definition = read.MainModule.Types.Single(t => t.Name == "Color");
        if (!definition.IsEnum || !definition.IsValueType || definition.BaseType?.Name != "Enum")
            throw new Exception("native enum category or ancestry lost");
        var other = new AssemblyBuilder(new("Other", new Version(1, 0, 0, 0)), core);
        Reject<ArgumentException>(() => other.AddEnum("Example", "Foreign").SetEnumBase(parent));
        Console.WriteLine("PASS native Enum base contracts");
    }

    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
