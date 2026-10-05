using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;
using TypeDefinition = NeoCLR.Metadata.Experimental.Model.TypeDefinition;

internal static class ObjectRootChecks
{
    internal static void Run()
    {
        foreach (bool manual in new[] { false, true })
        {
            var graph = Create();
            TypeDefinition root;
            if (manual)
            {
                root = new("System", "Object", 0x81, null);
                root.SetNativeObjectRoot();
                graph.Definition.MainModule.Types.Add(root);
            }
            else root = graph.AddNativeObjectRoot().Definition;
            if (!root.IsNativeObjectRoot || root.BaseType is not null) throw new Exception("root authoring lost designation or retained base");
            var builder = graph.Types.Single();
            builder.SetNativeObjectRoot();
            var receiver = builder.AddInstanceMethod("Identity", new MethodSignature(builder, []));
            receiver.GetILGenerator().LoadArgument(0); receiver.GetILGenerator().Return();
            var echo = graph.AddFunction("Echo", new MethodSignature(builder, [builder]));
            echo.GetILGenerator().LoadArgument(0); echo.GetILGenerator().Return();
            var image = RuntimeAssemblyContainer.WriteBinary(graph);
            var loaded = AssemblyDefinition.ReadNativeAssembly(image).MainModule.Types.Single();
            if (loaded.Namespace != "System" || loaded.Name != "Object" || loaded.BaseType is not null ||
                (loaded.Attributes & 0x180) != 0x80 || loaded.IsNativeObjectRoot)
                throw new Exception("root shape or host-only admission changed during roundtrip");
            Reject<InvalidOperationException>(loaded.SetNativeObjectRoot);
            using var pe = new PEReader(new MemoryStream(image));
            var reader = pe.GetMetadataReader();
            var row = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(t => reader.GetString(t.Name) == "Object");
            if (!row.BaseType.IsNil) throw new Exception("CLI root cannot extend bootstrap Object");
            var echoRow = reader.MethodDefinitions.Select(reader.GetMethodDefinition).Single(m => reader.GetString(m.Name) == "Echo");
            if (!reader.GetBlobBytes(echoRow.Signature).SequenceEqual(new byte[] { 0, 1, 0x1c, 0x1c }))
                throw new Exception("root signatures must use CLI ELEMENT_TYPE_OBJECT");

            Reject<InvalidDataException>(() => graph.Write());
            Reject<ArgumentException>(() => graph.AddNativeObjectRoot());
            root.Fields.Add(new("payload", 1, PrimitiveType.Int32));
            Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        }
        foreach (var declaration in new[] {
            new TypeDefinition("Other", "Object", 0x81, null),
            new TypeDefinition("System", "Object", 1, null),
            new TypeDefinition("System", "Object", 0x181, null),
            new TypeDefinition("System", "Object", 0x80, null) })
            Reject<ArgumentException>(declaration.SetNativeObjectRoot);
        var ordinary = Create();
        ordinary.AddClass("System", "Object");
        var plain = JsonNode.Parse(ordinary.WriteNativeAssembly())!;
        if (plain["types"]![0]!["name"]!.GetValue<string>() == "System.Object")
            throw new Exception("ordinary lookalike silently claimed canonical naming");
        var malformed = Create(); malformed.AddNativeObjectRoot();
        var json = JsonNode.Parse(malformed.WriteNativeAssembly())!;
        var types = json["types"]!.AsArray();
        types[0]!["is_sealed"] = true;
        Reject<InvalidDataException>(() => AssemblyDefinition.ReadNativeAssembly(System.Text.Encoding.UTF8.GetBytes(json.ToJsonString())));
    }

    private static AssemblyBuilder Create() => new(new("OwnedRoot", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
