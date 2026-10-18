using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class MemoryViewChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("Views", new Version(1, 0, 0, 0)), core);
        foreach (var kind in Enum.GetValues<MemoryViewKind>())
            graph.AddGenericValueType("System", kind.ToString(), ["T"]).SetMemoryView(kind);
        var bytes = graph.WriteNativeAssembly();
        var read = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph));
        foreach (var kind in Enum.GetValues<MemoryViewKind>())
        {
            var type = read.MainModule.Types.Single(t => t.Name == kind + "`1");
            if (type.MemoryView != kind || !type.IsValueType || type.Fields.Count != 0)
                throw new Exception("memory-view metadata lost");
        }
        var consumer = new AssemblyBuilder(new("ViewConsumer", new Version(1, 0, 0, 0)), core);
        var imported = consumer.ImportReference(read.MainModule.Types.Single(t => t.Name == "Slice`1"), core);
        var consumerBytes = consumer.WriteNativeAssembly();
        var consumerJson = JsonNode.Parse(consumerBytes)!;
        if (consumerJson["assemblies"]![0]!["value_type_references"]![0]!.GetValue<string>() != "System.Slice")
            throw new Exception("imported view lost canonical runtime identity");
        _ = NativeAssemblyDefinition.ReadAssembly(consumerBytes);
        var malformed = JsonNode.Parse(bytes)!;
        malformed["types"]![0]!["memory_view"] = "read_only_slice";
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())));
        Reject<NotSupportedException>(() => NativeAssemblyDefinition.ReadAssembly(bytes).CreateReferenceAssembly(core));
        var bad = new AssemblyBuilder(new("Bad", new Version(1, 0, 0, 0)), core);
        var noncanonical = bad.AddGenericValueType("Example", "Slice", ["T"]);
        Reject<InvalidOperationException>(() => noncanonical.SetMemoryView(MemoryViewKind.Slice));
        var stored = bad.AddGenericValueType("System", "Slice", ["U"]);
        stored.AddField("Data", PrimitiveType.Int32);
        Reject<InvalidOperationException>(() => stored.SetMemoryView(MemoryViewKind.Slice));
        Console.WriteLine("PASS memory-view metadata, identity, shape and projection guards");
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}
