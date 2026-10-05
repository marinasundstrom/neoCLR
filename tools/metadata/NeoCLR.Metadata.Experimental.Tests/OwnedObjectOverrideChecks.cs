using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;
using MethodDefinition = NeoCLR.Metadata.Experimental.Model.MethodDefinition;

internal static class OwnedObjectOverrideChecks
{
    internal static AssemblyBuilder Create(bool manual = false)
    {
        var graph = ObjectSlotChecks.Create();
        var root = graph.Types[0];
        var rootConstructor = root.AddConstructor(new MethodSignature(PrimitiveType.Void, []), MethodVisibility.Protected);
        rootConstructor.GetILGenerator().Return();
        var derived = graph.AddClass("Example", "Derived", root);
        var constructor = derived.AddConstructor(new MethodSignature(PrimitiveType.Void, []));
        var il = constructor.GetILGenerator();
        il.LoadArgument(0); il.Call(rootConstructor); il.Return();
        foreach (var (name, signature) in new[] {
            ("ToString", new MethodSignature(PrimitiveType.String, [])),
            ("GetHashCode", new MethodSignature(PrimitiveType.Int32, [])),
            ("Equals", new MethodSignature(PrimitiveType.Boolean, [graph.ObjectType])) })
        {
            MethodBuilder method;
            if (manual)
            {
                var definition = new MethodDefinition(name, 0x46, signature);
                derived.Definition.Methods.Add(definition);
                method = MethodBuilder.ForDefinition(definition);
            }
            else method = derived.AddOverride(name, signature);
            il = method.GetILGenerator();
            if (name == "ToString") il.Emit(OpCode.Ldstr, "derived override");
            else if (name == "GetHashCode") il.LoadConstant(93);
            else il.Emit(OpCode.Ldc_Bool, true);
            il.Return();
            var call = graph.AddFunction("Override" + name, new MethodSignature(signature.ReturnType, []));
            il = call.GetILGenerator();
            il.NewObject(constructor);
            if (name == "Equals") il.NewObject(constructor);
            il.CallVirtual(root.Methods.Single(m => m.Name == name));
            il.Return();
        }
        return graph;
    }

    internal static void Run()
    {
        var lateRoot = new AssemblyBuilder(new("LateRoot", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var owner = lateRoot.AddClass("Example", "BeforeRoot");
        var equals = owner.AddOverride("Equals", new(PrimitiveType.Boolean, [lateRoot.CoreObjectType]));
        equals.GetILGenerator().Emit(OpCode.Ldc_Bool, false); equals.GetILGenerator().Return();
        lateRoot.AddNativeObjectRoot();
        Reject<InvalidDataException>(() => lateRoot.WriteNativeAssembly());
        foreach (bool manual in new[] { false, true })
        {
            var graph = Create(manual);
            var image = RuntimeAssemblyContainer.WriteBinary(graph);
            var loaded = AssemblyDefinition.ReadNativeAssembly(image);
            var views = new MetadataLoadContext([loaded]).Resolve(loaded.Identity).GetTypes();
            var root = views.Single(t => t.Name == "Object");
            var derived = views.Single(t => t.Name == "Derived");
            if (derived.GetMethods().Any(m => !m.IsVirtual || m.IsNewSlot || m.IsAbstract))
                throw new Exception("native overrides do not reuse root slots");
            if (!ReferenceEquals(root, derived.GetMethods().Single(m => m.Name == "Equals").GetParameters()[0].ParameterType))
                throw new Exception("override Equals lost the owned root identity");
            using var pe = new PEReader(new MemoryStream(image));
            var reader = pe.GetMetadataReader();
            var row = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(t => reader.GetString(t.Name) == "Derived");
            if (row.BaseType.Kind != HandleKind.TypeDefinition) throw new Exception("CLI derived base is not local");
            foreach (var method in row.GetMethods().Select(reader.GetMethodDefinition).Where(m => reader.GetString(m.Name) != ".ctor"))
                if (((int)method.Attributes & 0x540) != 0x40) throw new Exception("CLI override flags changed");
            // Native snapshot projection of inherited classes remains explicitly unsupported.
            Reject<NotSupportedException>(() => NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary));
            Reject<InvalidOperationException>(() => graph.Types[1].AddOverride("Equals", new(PrimitiveType.Boolean, [graph.CoreObjectType])));
            Reject<InvalidOperationException>(() => graph.Types[0].AddOverride("ToString", new(PrimitiveType.String, [])));
            Reject<InvalidOperationException>(() => graph.Types[1].AddOverride("Equals", new(PrimitiveType.Boolean, [ObjectSlotChecks.Create().ObjectType])));
            var malformed = JsonNode.Parse(graph.WriteNativeAssembly())!;
            malformed["functions"]![0]!["is_virtual"] = true;
            malformed["functions"]![0]!["is_override"] = true;
            Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())));
        }
    }

    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
