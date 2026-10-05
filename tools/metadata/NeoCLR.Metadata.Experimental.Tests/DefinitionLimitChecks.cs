using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class DefinitionLimitChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("LargeLibrary", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var module = graph.Definition.MainModule;
        var objectType = module.ImportReference(graph.CoreLibrary, "System", "Object");
        for (int i = 0; i < 4095; i++)
        {
            if (i % 2 == 0) graph.AddClass("Example", "Type" + i);
            else module.Types.Add(new TypeDefinition("Example", "Type" + i, 1, objectType));
        }
        Reject<ArgumentException>(() => graph.AddClass("Example", "Overflow"));
        Reject<ArgumentException>(() => module.Types.Add(new TypeDefinition("Example", "ManualOverflow", 1, objectType)));
        if (module.Types.Count != 4095) throw new Exception("failed attachment changed type count");
        var cli = AssemblyDefinition.ReadAssembly(graph.Write(), expectedExtended: false);
        if (cli.MainModule.Types.Count != 4096 || cli.MainModule.Types.Last().Name != "Type4094")
            throw new Exception($"CLI type rows did not survive the boundary: {cli.MainModule.Types.Count}, {cli.MainModule.Types.Last().Name}");
        var native = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph));
        var context = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([native]);
        if (context.Resolve(native.Identity).GetTypes().Count() != 4095 || native.MainModule.Types.Last().Name != "Type4094")
            throw new Exception("native type rows did not survive the boundary");
        var malformed = JsonNode.Parse(graph.WriteNativeAssembly())!;
        var rows = malformed["types"]!.AsArray();
        rows.Add(rows[0]!.DeepClone());
        try { _ = NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())); }
        catch (InvalidDataException e) when (e.Message == "invalid or excessive types") { return; }
        throw new Exception("native reader did not reject excessive type rows before materialization");
    }

    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
