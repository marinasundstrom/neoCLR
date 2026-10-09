using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ErasedValueChecks
{
    internal static void Run(string directory)
    {
        Directory.CreateDirectory(directory);
        foreach (var mismatch in new[] { false, true })
        {
            var identity = new AssemblyIdentity("ValueOps", new(1, 0, 0, 0));
            var graph = new AssemblyBuilder(identity, new("Core", new(1, 0, 0, 0)));
            var value = graph.AddValueType("System", "Value"); value.SetNativePrimitive(PrimitiveType.Value);
            var slice = graph.AddFunction("neoCLR.Runtime", "StringSliceUtf8", new(value, [PrimitiveType.String, PrimitiveType.Int32, PrimitiveType.Int32])); slice.SetInternalCall();
            var count = graph.AddFunction("neoCLR.Runtime", "StringByteCount", new(PrimitiveType.Int32, [PrimitiveType.String])); count.SetInternalCall();
            var unpack = graph.AddFunction("Unpack", new(SignatureType.MethodParameter(0), [value], ["T"]));
            unpack.LoadArgument(0); unpack.Emit(OpCode.ValueUnpack, SignatureType.MethodParameter(0)); unpack.Return();
            var main = graph.AddFunction("Main"); graph.EntryPoint = main;
            main.Emit(OpCode.Ldstr, "é"); main.LoadConstant(mismatch ? -1 : 0); main.LoadConstant(2); main.Call(slice);
            var payload = main.DeclareLocal(value); main.StoreLocal(payload);
            main.LoadLocal(payload); main.Emit(OpCode.ValueIs, PrimitiveType.String);
            main.Emit(OpCode.Ldc_Bool, !mismatch); main.Emit(OpCode.Ceq);
            var validTag = main.DefineLabel(); main.Emit(OpCode.Brtrue, validTag);
            main.LoadConstant(0); main.Return(); main.MarkLabel(validTag);
            main.LoadLocal(payload); main.Call(unpack.MakeGenericInstance(PrimitiveType.String)); main.Call(count); main.LoadConstant(40); main.Add(); main.Return();
            var image = RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary);
            _ = AssemblyDefinition.ReadNativeAssembly(image);
            File.WriteAllBytes(Path.Combine(directory, mismatch ? "Mismatch.dll" : "Matching.dll"), image);
            Reject(() => graph.Write());
            Reject(() => main.Emit(OpCode.ValueIs, PrimitiveType.Void));
            var bad = new AssemblyBuilder(new("Bad", new(1, 0, 0, 0)), identity);
            var body = bad.AddFunction("Main"); bad.EntryPoint = body;
            body.LoadConstant(1); body.Emit(OpCode.ValueUnpack, PrimitiveType.Int32); body.Return();
            Reject(() => bad.WriteNativeAssembly());
        }
        Console.WriteLine("PASS erased Value metadata round-trip, native-only admission and input/operand validation");
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (Exception error) when (error is ArgumentException or InvalidDataException) { return; }
        throw new Exception("Invalid erased Value operation admitted");
    }
}
