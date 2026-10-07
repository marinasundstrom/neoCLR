using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class NativeIntegerChecks
{
    internal static void WriteInputs(string corePath, string output)
    {
        var core = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(corePath), false);
        var graph = new AssemblyBuilder(new("NativeWidthInputs", new(1, 0, 0, 0)), core.Identity);
        var owner = graph.AddType("", "NativeWidthInputs");
        foreach (var (name, kind, opcode, value) in new[] {
            ("Negative", PrimitiveType.IntPtr, OpCode.Conv_I, -42L),
            ("Maximum", PrimitiveType.UIntPtr, OpCode.Conv_U, -1L),
            ("Six", PrimitiveType.UIntPtr, OpCode.Conv_U, 6L),
            ("Seven", PrimitiveType.UIntPtr, OpCode.Conv_U, 7L) })
        {
            var method = owner.AddMethod(name, new(kind, []));
            var il = method.GetILGenerator(); il.Emit(OpCode.Ldc_I8, value); il.Emit(opcode); il.Return();
        }
        File.WriteAllBytes(output, RuntimeAssemblyContainer.WriteLibraryBinary(graph));
    }

    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("NativeIntegers", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var owner = graph.AddType("Example", "Numbers");
        var storage = graph.AddValueType("Example", "Storage");
        var main = owner.AddMethod("Main", new(PrimitiveType.Int32, []));
        var il = main.GetILGenerator();
        var failed = il.DefineLabel();
        foreach (var (kind, opcode, value) in new[] {
            (PrimitiveType.IntPtr, OpCode.Conv_I, -42L),
            (PrimitiveType.UIntPtr, OpCode.Conv_U, IntPtr.Size == 8 ? 4294967296L : 42L) })
        {
            // Definitions and builder helpers share the same signature encoding.
            storage.Definition.Fields.Add(new FieldDefinition(kind.ToString(), 6, kind));
            var identity = owner.AddMethod(kind.ToString(), new(kind, [kind]));
            var body = identity.GetILGenerator();
            var local = body.DeclareLocal(kind);
            body.LoadArgument(0); body.StoreLocal(local); body.LoadLocal(local); body.Return();
            var getter = owner.AddMethod("get_" + kind, new(kind, []));
            body = getter.GetILGenerator(); body.Emit(OpCode.Ldc_I8, value); body.Emit(opcode); body.Return();
            owner.AddProperty(kind.ToString(), kind, getter, null);
            il.Call(getter); il.Call(identity); il.Emit(OpCode.Conv_I8); il.Emit(OpCode.Ldc_I8, value);
            il.Emit(OpCode.Ceq); il.Emit(OpCode.Brfalse, failed);
        }
        il.LoadConstant(42); il.Return();
        il.MarkLabel(failed); il.LoadConstant(1); il.Return();
        graph.EntryPoint = main;
        var cli = graph.Write();
        var reflected = Assembly.Load(cli).GetType("Example.Numbers")!;
        if ((int)reflected.GetMethod("Main")!.Invoke(null, null)! != 42 ||
            reflected.GetMethod("IntPtr")!.ReturnType != typeof(nint) ||
            reflected.GetMethod("UIntPtr")!.GetParameters()[0].ParameterType != typeof(nuint))
            throw new Exception("CLI native integer execution or signature");
        var native = RuntimeAssemblyContainer.WriteBinary(graph);
        foreach (var snapshot in new[] { AssemblyDefinition.ReadAssembly(cli, false), AssemblyDefinition.ReadNativeAssembly(native) })
        {
            var numbers = snapshot.MainModule.Types.Single(t => t.Name == "Numbers");
            var fields = snapshot.MainModule.Types.Single(t => t.Name == "Storage");
            var views = new MetadataLoadContext([snapshot]).Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Numbers");
            foreach (var kind in new[] { PrimitiveType.IntPtr, PrimitiveType.UIntPtr })
            {
                if (!numbers.Methods.Single(m => m.Name == kind.ToString()).TryGetStaticPrimitiveSignature(out var signature) ||
                    signature!.ReturnType != kind || signature.ParameterTypes.Single() != kind ||
                    !fields.Fields.Single(f => f.Name == kind.ToString()).TryGetPrimitiveType(out var field) || field != kind ||
                    !numbers.Properties.Single(p => p.Name == kind.ToString()).TryGetPrimitiveSignature(out var property, out _) || property != kind ||
                    views.GetMethods().Single(m => m.Name == kind.ToString()).ReturnType is not PrimitiveTypeInfo { Kind: var result } || result != kind)
                    throw new Exception("native integer signature round trip: " + kind);
            }
        }
        foreach (var opcode in new[] { OpCode.Conv_I, OpCode.Conv_U })
        {
            var invalid = new AssemblyBuilder(new("Bad" + opcode, new(1, 0, 0, 0)), graph.CoreLibrary);
            var method = invalid.AddFunction("Invalid", new(PrimitiveType.IntPtr, []));
            var body = method.GetILGenerator(); body.Emit(OpCode.Ldstr, "wrong"); body.Emit(opcode); body.Return();
            try { invalid.Write(); throw new Exception("nonnumeric native conversion accepted"); }
            catch (InvalidDataException) { }
        }
        var owners = new AssemblyBuilder(new("NativeIntegerOwners", new(1, 0, 0, 0)), graph.CoreLibrary);
        foreach (var kind in new[] { PrimitiveType.IntPtr, PrimitiveType.UIntPtr })
        {
            var type = owners.AddValueType("System", kind.ToString());
            type.SetNativePrimitive(kind);
        }
        var owned = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(owners));
        if (!owned.MainModule.Types.Select(t => t.NativePrimitive).SequenceEqual(new PrimitiveType?[] { PrimitiveType.IntPtr, PrimitiveType.UIntPtr }))
            throw new Exception("native integer ownership lost");
        if (Environment.GetEnvironmentVariable("NEOCLR_NATIVE_INTEGER_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
    }
}
