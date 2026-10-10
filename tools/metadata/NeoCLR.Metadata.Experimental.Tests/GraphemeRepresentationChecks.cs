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
            foreach (var authored in new[] { false, true })
            {
                var consumer = new AssemblyBuilder(new(authored ? "AuthoredGraphemeConsumer" : "ImportedGraphemeConsumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
                var reference = authored
                    ? consumer.CreateValueTypeReference(snapshot.Identity, graph.CoreLibrary,
                        Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)), "System", "Char")
                    : consumer.ImportReference(loaded, graph.CoreLibrary);
                consumer.SetNativeGrapheme(reference);
                Reject(() => consumer.CreateMethodReference(reference, ".ctor", new(PrimitiveType.Void, [])));
                Reject(() => consumer.CreateMethodReference(reference, "Wrong", new(reference, []), isOverride: true));
                var method = authored
                    ? consumer.CreateMethodReference(reference, "Echo", new(reference, []))
                    : consumer.ImportReference(loaded.Methods.Single(), graph.CoreLibrary);
                var forward = consumer.AddFunction("Forward", new(reference, [reference]));
                var body = forward.GetILGenerator();
                var local = body.DeclareLocal(reference);
                body.LoadArgument(0); body.StoreLocal(local); body.LoadLocalAddress(local);
                body.Call(method); body.Return();
                var consumerImage = RuntimeAssemblyContainer.WriteBinary(consumer);
                var consumerSnapshot = AssemblyDefinition.ReadNativeAssembly(consumerImage);
                var context = new MetadataLoadContext([snapshot, consumerSnapshot]);
                var target = context.Resolve(snapshot.Identity).GetTypes().Single();
                var function = context.Resolve(consumerSnapshot.Identity).GetModules().SelectMany(module => module.GetFunctions()).Single();
                if (!ReferenceEquals(function.ReturnType, target)) throw new Exception("external grapheme identity lost");
                Reject(() => consumer.Write());
                var missing = new MetadataLoadContext([consumerSnapshot]);
                Reject(() => _ = missing.Resolve(consumerSnapshot.Identity).GetModules().SelectMany(module => module.GetFunctions()).Single().ReturnType);
                Reject(() => graph.SetNativeGrapheme(reference));
                Reject(() => consumer.SetNativeGrapheme(consumer.CreateValueTypeReference(
                    new("WrongOwner", new Version(1, 0, 0, 0)), graph.CoreLibrary, new string('a', 64), "System", "Char")));
                if (!manual && Environment.GetEnvironmentVariable("NEOCLR_GRAPHEME_ARTIFACT") is { } consumerPath)
                {
                    File.WriteAllBytes(consumerPath + (authored ? ".authored" : ".imported"), consumerImage);
                    File.WriteAllBytes(consumerPath + (authored ? ".authored.json" : ".imported.json"), consumer.WriteNativeAssembly());
                }
            }
            Reject(() => graph.Write());
            if (!manual && Environment.GetEnvironmentVariable("NEOCLR_GRAPHEME_ARTIFACT") is { } path)
                File.WriteAllBytes(path, image);
            character.AddField("bad", PrimitiveType.Int32);
            Reject(() => graph.WriteNativeAssembly());
        }
        foreach (var externalFirst in new[] { false, true })
        {
            var duplicate = new AssemblyBuilder(new("Duplicate", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
            var external = duplicate.CreateValueTypeReference(new("External", new Version(1, 0, 0, 0)), duplicate.CoreLibrary,
                new string('d', 64), "System", "Char");
            if (externalFirst) duplicate.SetNativeGrapheme(external);
            duplicate.AddValueType("System", "Char").SetNativeGrapheme();
            if (externalFirst) Reject(() => duplicate.WriteNativeAssembly());
            else Reject(() => duplicate.SetNativeGrapheme(external));
        }
        var invalid = new AssemblyBuilder(new("Invalid", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
        Reject(() => invalid.SetNativeGrapheme(null!));
        var late = invalid.CreateValueTypeReference(new("Late", new Version(1, 0, 0, 0)), invalid.CoreLibrary,
            new string('b', 64), "System", "Char");
        invalid.CreateMethodReference(late, "Ordinary", new(PrimitiveType.Int32, []));
        Reject(() => invalid.SetNativeGrapheme(late));
        var wrong = invalid.CreateValueTypeReference(new("Wrong", new Version(1, 0, 0, 0)), invalid.CoreLibrary,
            new string('c', 64), "Other", "Char");
        Reject(() => invalid.SetNativeGrapheme(wrong));
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
