using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;
using MethodDefinition = NeoCLR.Metadata.Experimental.Model.MethodDefinition;
using TypeDefinition = NeoCLR.Metadata.Experimental.Model.TypeDefinition;

internal static class ObjectRootChecks
{
    internal static void Run()
    {
        ExternalRoot();
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

    internal static void WriteExternalConsumer(string libraryPath, string corePath, string output)
    {
        var image = File.ReadAllBytes(libraryPath);
        var library = AssemblyDefinition.ReadNativeAssembly(image);
        var core = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(corePath), false);
        var graph = new AssemblyBuilder(new("ExternalObjectConsumer", new Version(1, 0, 0, 0)), core.Identity);
        var root = graph.CreateTypeReference(library.Identity, core.Identity,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)), "System", "Object", 0);
        graph.SetNativeObjectRoot(root);
        var owner = graph.AddClass("Example", "Item");
        var ctor = owner.AddConstructor([]); ctor.GetILGenerator().Return();
        var equals = owner.AddOverride("Equals", new(PrimitiveType.Boolean, [root]));
        equals.GetILGenerator().Emit(OpCode.Ldc_Bool, true); equals.GetILGenerator().Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var il = main.GetILGenerator(); var failed = il.DefineLabel();
        il.NewObject(ctor); il.LoadDefault(root); il.CallVirtual(equals);
        il.Emit(OpCode.Brfalse, failed); il.LoadConstant(42); il.Return();
        il.MarkLabel(failed); il.LoadConstant(1); il.Return();
        File.WriteAllBytes(output, RuntimeAssemblyContainer.WriteBinary(graph));
    }

    private static void ExternalRoot()
    {
        foreach (bool manual in new[] { false, true })
        {
            var graph = Create();
            var identity = new AssemblyIdentity("ExternalRoot", new Version(1, 0, 0, 0));
            var root = graph.CreateTypeReference(identity, graph.CoreLibrary, new string('A', 64), "System", "Object", 0);
            var owner = graph.AddClass("Example", "Item");
            var signature = new MethodSignature(PrimitiveType.Boolean, [root]);
            Reject<InvalidOperationException>(() => owner.AddOverride("Equals", signature));
            graph.SetNativeObjectRoot(root);
            graph.SetNativeObjectRoot(root);
            var externalClass = graph.CreateTypeReference(identity, graph.CoreLibrary, new string('A', 64), "Example", "External", 0);
            _ = graph.CreateMethodReference(externalClass, "Equals", signature, isOverride: true);
            Reject<ArgumentException>(() => graph.CreateMethodReference(externalClass, "Equals",
                new(PrimitiveType.Boolean, [graph.CoreObjectType]), isOverride: true));
            if (!Equals(graph.ObjectType.ImportedType, root)) throw new Exception("external Object selection lost");
            MethodBuilder method;
            if (manual)
            {
                owner.Definition.Methods.Add(new MethodDefinition("Equals", 0x46, signature));
                method = owner.Methods.Single();
            }
            else method = owner.AddOverride("Equals", signature);
            method.GetILGenerator().Emit(OpCode.Ldc_Bool, true);
            method.GetILGenerator().Return();
            var native = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly());
            _ = native.CreateReferenceAssembly(graph.CoreLibrary);
            var loaded = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph));
            if (loaded.MainModule.Types.Single().Methods.Single().Name != "Equals")
                throw new Exception("external Object override lost on roundtrip");
            Reject<InvalidDataException>(() => graph.Write());
            Reject<InvalidOperationException>(() => graph.AddNativeObjectRoot());
            Reject<InvalidOperationException>(() => graph.SetNativeObjectRoot(graph.CoreObjectType));
            Reject<InvalidOperationException>(() => owner.AddOverride("Equals", new(PrimitiveType.Boolean, [graph.CoreObjectType])));
            Reject<ArgumentException>(() => graph.SetNativeObjectRoot(Create().CoreObjectType));
        }
    }

    private static AssemblyBuilder Create() => new(new("OwnedRoot", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
