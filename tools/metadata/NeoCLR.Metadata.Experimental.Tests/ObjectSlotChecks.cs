using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;
using MethodDefinition = NeoCLR.Metadata.Experimental.Model.MethodDefinition;

internal static class ObjectSlotChecks
{
    internal static AssemblyBuilder Create(bool manual = false)
    {
        var graph = new AssemblyBuilder(new("OwnedRootSlots", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var root = graph.AddNativeObjectRoot();
        foreach (var (name, signature) in new[] {
            ("ToString", new MethodSignature(PrimitiveType.String, [])),
            ("GetHashCode", new MethodSignature(PrimitiveType.Int32, [])),
            ("Equals", new MethodSignature(PrimitiveType.Boolean, [root])) })
        {
            MethodBuilder method;
            if (manual)
            {
                var definition = new MethodDefinition(name, 0x146, signature);
                root.Definition.Methods.Add(definition);
                method = MethodBuilder.ForDefinition(definition);
            }
            else method = root.AddNativeObjectSlot(name, signature);
            var il = method.GetILGenerator();
            if (name == "ToString") il.Emit(OpCode.Ldstr, "authored root");
            else if (name == "GetHashCode") il.LoadConstant(73);
            else il.Emit(OpCode.Ldc_Bool, false);
            il.Return();
        }
        var boxed = graph.AddFunction("BoxedDisplay", new MethodSignature(PrimitiveType.String, []));
        var body = boxed.GetILGenerator();
        body.LoadConstant(42);
        body.Box(PrimitiveType.Int32);
        body.Emit(OpCode.Isinst, (SignatureType)PrimitiveType.Int32);
        body.CallVirtual(root.Methods.Single(m => m.Name == "ToString"));
        body.Return();
        return graph;
    }

    internal static void Run()
    {
        var incomplete = new AssemblyBuilder(new("IncompleteRoot", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        if (incomplete.ObjectType.ImportedType != incomplete.CoreObjectType)
            throw new Exception("default Object identity changed");
        var incompleteRoot = incomplete.AddNativeObjectRoot();
        if (incomplete.ObjectType.ClassType != incompleteRoot)
            throw new Exception("authored root was not selected for Object signatures");
        var box = incomplete.AddFunction("Box", new MethodSignature(incomplete.ObjectType, []));
        box.GetILGenerator().LoadConstant(1); box.GetILGenerator().Box(PrimitiveType.Int32); box.GetILGenerator().Return();
        Reject<InvalidDataException>(() => incomplete.WriteNativeAssembly());
        var mixed = Create();
        var wrongBox = mixed.AddFunction("WrongBox", new MethodSignature(mixed.CoreObjectType, []));
        wrongBox.GetILGenerator().LoadConstant(1); wrongBox.GetILGenerator().Box(PrimitiveType.Int32); wrongBox.GetILGenerator().Return();
        Reject<InvalidDataException>(() => mixed.WriteNativeAssembly());
        var calls = Create();
        var root = calls.Types[0];
        var display = calls.AddFunction("Display", new MethodSignature(PrimitiveType.String, [root]));
        display.GetILGenerator().LoadArgument(0);
        display.GetILGenerator().CallVirtual(root.Methods.Single(m => m.Name == "ToString"));
        display.GetILGenerator().Return();
        _ = RuntimeAssemblyContainer.WriteBinary(calls);
        foreach (bool manual in new[] { false, true })
        {
            var graph = Create(manual);
            var image = RuntimeAssemblyContainer.WriteBinary(graph);
            var loaded = AssemblyDefinition.ReadNativeAssembly(image);
            var view = new MetadataLoadContext([loaded]).Resolve(loaded.Identity).GetTypes().Single();
            if (view.GetMethods().Any(m => !m.IsVirtual || !m.IsNewSlot || m.IsAbstract || m.IsStatic))
                throw new Exception("introspection lost concrete root slot facts");
            if (!ReferenceEquals(view, view.GetMethods().Single(m => m.Name == "Equals").GetParameters()[0].ParameterType))
                throw new Exception("Equals parameter resolved to a different root");
            foreach (var method in loaded.MainModule.Types.Single().Methods)
                if ((method.Attributes & 0x540) != 0x140) throw new Exception("native root slot lost Virtual/NewSlot flags");
            using var pe = new PEReader(new MemoryStream(image));
            var reader = pe.GetMetadataReader();
            foreach (var handle in reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(t => reader.GetString(t.Name) == "Object").GetMethods())
                if (((int)reader.GetMethodDefinition(handle).Attributes & 0x540) != 0x140)
                    throw new Exception("CLI projection changed root slots into overrides or abstract methods");
            // Exercise the native-reader-to-projection path as well as graph emission.
            _ = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary);
            var wrong = graph.AddClass("Example", "Wrong");
            Reject<InvalidOperationException>(() => wrong.AddNativeObjectSlot("ToString", new(PrimitiveType.String, [])));
            Reject<ArgumentException>(() => graph.Types[0].AddNativeObjectSlot("Equals", new(PrimitiveType.Boolean, [graph.CoreObjectType])));
            Reject<ArgumentException>(() => graph.Types[0].AddNativeObjectSlot("Other", new(PrimitiveType.Int32, [])));
            var foreign = Create().Types[0];
            Reject<InvalidOperationException>(() => graph.Types[0].AddNativeObjectSlot("Equals", new(PrimitiveType.Boolean, [foreign])));
            var malformed = JsonNode.Parse(graph.WriteNativeAssembly())!;
            malformed["functions"]![1]!["is_abstract"] = true;
            Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())));
        }
    }

    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
