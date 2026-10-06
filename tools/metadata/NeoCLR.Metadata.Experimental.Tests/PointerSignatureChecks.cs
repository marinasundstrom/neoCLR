using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class PointerSignatureChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("PointerSignatures", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var owner = graph.AddType("Example", "Pointers");
        var pointer = SignatureType.PointerTo(PrimitiveType.Void);
        foreach (var (name, type) in new[] { ("Identity", pointer), ("Nested", SignatureType.PointerTo(pointer)), ("Byte", SignatureType.PointerTo(PrimitiveType.Byte)) })
        {
            var method = owner.AddMethod(name, new(type, [type]));
            var il = method.GetILGenerator();
            var local = il.DeclareLocal(type);
            il.LoadArgument(0); il.StoreLocal(local); il.LoadLocal(local); il.Return();
        }
        var deepest = pointer;
        for (var i = 1; i < 16; i++) deepest = SignatureType.PointerTo(deepest);
        var deep = owner.AddMethod("Deep", new(deepest, [deepest]));
        deep.GetILGenerator().LoadArgument(0); deep.GetILGenerator().Return();
        var manual = new MethodDefinition("Manual", (ushort)0x16, new(pointer, [pointer]));
        owner.Definition.Methods.Add(manual);
        var manualIl = MethodBuilder.ForDefinition(manual).GetILGenerator();
        manualIl.LoadArgument(0); manualIl.Return();
        var generic = graph.AddFunction("Generic", new(PrimitiveType.Void, [], ["T"]));
        generic.GetILGenerator().Return();
        Reject(() => generic.MakeGenericInstance(pointer));
        var caller = new AssemblyBuilder(new("PointerFunctions", new(1, 0, 0, 0)), graph.CoreLibrary);
        var external = caller.CreateFunctionReference(graph.Identity, graph.CoreLibrary, new string('a', 64), "Example", "Identity", new(pointer, [pointer]));
        var wrapper = caller.AddFunction("Echo", new(pointer, [pointer]));
        wrapper.GetILGenerator().LoadArgument(0); wrapper.GetILGenerator().Call(external); wrapper.GetILGenerator().Return();
        _ = RuntimeAssemblyContainer.WriteLibraryBinary(caller);
        var cli = graph.Write();
        if (Assembly.Load(cli).GetType("Example.Pointers")!.GetMethod("Identity")!.ReturnType != typeof(void).MakePointerType())
            throw new Exception("CLI PTR encoding lost");
        foreach (var snapshot in new[] { AssemblyDefinition.ReadAssembly(cli, false), AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph)) })
        {
            var definition = snapshot.MainModule.Types.Single(t => t.Name == "Pointers");
            var importer = new AssemblyBuilder(new("PointerConsumer", new(1, 0, 0, 0)), graph.CoreLibrary);
            var view = new MetadataLoadContext([snapshot]).Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Pointers");
            foreach (var method in definition.Methods)
            {
                if (!method.TryGetStaticValueSignature(out var signature) || signature!.ReturnType.PointerElement is null || signature.ParameterTypes.Single() != signature.ReturnType)
                    throw new Exception("pointer signature roundtrip");
                if (importer.ImportReference(method, graph.CoreLibrary).Signature.ReturnType != signature.ReturnType)
                    throw new Exception("pointer import identity");
                var projected = view.GetMethods().Single(m => m.Name == method.Name);
                if (projected.ReturnType is not PointerTypeInfo || projected.ReturnType.DisplayName != signature.ReturnType.ToString() ||
                    !ReferenceEquals(projected.ReturnType, projected.GetParameters().Single().ParameterType))
                    throw new Exception("pointer introspection");
            }
        }
        Reject(() => SignatureType.PointerTo(PrimitiveType.String));
        Reject(() => SignatureType.PointerTo(SignatureType.TypeParameter(0)));
        Reject(() => SignatureType.ArrayOf(pointer));
        Reject(() => SignatureType.Function(new(pointer, [])));
        var nested = pointer;
        for (var i = 1; i < 16; i++) nested = SignatureType.PointerTo(nested);
        Reject(() => SignatureType.PointerTo(nested));
        var bad = new AssemblyBuilder(new("BadPointer", new(1, 0, 0, 0)), graph.CoreLibrary);
        var mismatch = bad.AddFunction("Mismatch", new(pointer, [SignatureType.PointerTo(PrimitiveType.Byte)]));
        mismatch.GetILGenerator().LoadArgument(0); mismatch.GetILGenerator().Return();
        try { bad.Write(); throw new Exception("different pointer targets accepted"); }
        catch (InvalidDataException) { }
    }

    internal static void WriteRuntime(string path)
    {
        var graph = new AssemblyBuilder(new("PointerAllocation", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var pointer = SignatureType.PointerTo(PrimitiveType.Void);
        var allocate = graph.AddFunction("neoCLR.Runtime", "NativeAllocate", new(pointer, [PrimitiveType.UIntPtr]));
        var free = graph.AddFunction("neoCLR.Runtime", "NativeFree", new(PrimitiveType.Void, [pointer]));
        var multiply = graph.AddFunction("neoCLR.Runtime", "NativeMultiplyChecked", new(PrimitiveType.UIntPtr, [PrimitiveType.UIntPtr, PrimitiveType.UIntPtr]));
        allocate.SetInternalCall(); free.SetInternalCall(); multiply.SetInternalCall();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var il = main.GetILGenerator();
        var local = il.DeclareLocal(pointer);
        il.LoadConstant(6); il.Emit(OpCode.Conv_U);
        il.LoadConstant(7); il.Emit(OpCode.Conv_U);
        il.Call(multiply); il.Call(allocate); il.StoreLocal(local);
        il.LoadLocal(local); il.Call(free);
        il.LoadConstant(42); il.Return();
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph));
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (ArgumentException) { return; }
        throw new Exception("unsupported pointer shape accepted");
    }
}
