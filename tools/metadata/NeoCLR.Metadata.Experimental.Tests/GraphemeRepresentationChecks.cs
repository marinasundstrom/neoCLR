using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class GraphemeRepresentationChecks
{
    internal static void Run()
    {
        foreach (var manual in new[] { false, true })
        {
            var graph = new AssemblyBuilder(new("GraphemeOwner", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
            TypeBuilder character;
            if (manual)
            {
                var definition = new TypeDefinition("System", "Char", 0x109,
                    graph.Definition.MainModule.ImportReference(graph.CoreLibrary, "System", "ValueType"));
                definition.SetNativeGrapheme();
                graph.Definition.MainModule.Types.Add(definition);
                character = graph.Types.Single();
            }
            else
            {
                character = graph.AddValueType("System", "Char");
                character.SetNativeGrapheme();
            }
            var echo = character.AddInstanceMethod("Echo", new(character, []));
            var il = echo.GetILGenerator(); il.LoadArgument(0); il.LoadObject(character); il.Return();
            var image = RuntimeAssemblyContainer.WriteBinary(graph);
            var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
            var loaded = snapshot.MainModule.Types.Single(t => t.Name == "Char");
            if (!loaded.NativeGrapheme || !loaded.IsValueType || loaded.NativePrimitive is not null)
                throw new Exception("grapheme declaration became numeric or reference storage");
            var view = new MetadataLoadContext([snapshot]).Resolve(snapshot.Identity).GetTypes().Single();
            if (!ReferenceEquals(view.GetMethods().Single().ReturnType, view)) throw new Exception("grapheme signature lost declaring identity");
            if (!view.NativeGrapheme) throw new Exception("introspection lost grapheme fact");
            Reject(() => loaded.SetNativeGrapheme());
            var consumer = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
            Reject(() => consumer.ImportReference(loaded, graph.CoreLibrary));
            Reject(() => consumer.ImportReference(loaded.Methods.Single(), graph.CoreLibrary));
            Reject(() => graph.Write());
            if (!manual && Environment.GetEnvironmentVariable("NEOCLR_GRAPHEME_ARTIFACT") is { } path)
                File.WriteAllBytes(path, image);
            character.AddField("bad", PrimitiveType.Int32);
            Reject(() => graph.WriteNativeAssembly());
        }
        var invalid = new AssemblyBuilder(new("Invalid", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
        Reject(() => invalid.AddClass("System", "Char").SetNativeGrapheme());
        Reject(() => invalid.AddValueType("Other", "Char").SetNativeGrapheme());
    }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception error) when (error is ArgumentException or InvalidOperationException or InvalidDataException or NotSupportedException) { return; }
        throw new Exception("invalid grapheme contract accepted");
    }
}
