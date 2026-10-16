using System.Reflection;
using System.Text.Json;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class CheckedArithmeticChecks
{
    private sealed record Case(string Name, PrimitiveType Type, OpCode Op, long Left, long Right, long? Expected);

    private static IEnumerable<Case> Cases()
    {
        foreach (var type in new[] { PrimitiveType.Int32, PrimitiveType.UInt32, PrimitiveType.Int64,
                     PrimitiveType.UInt64, PrimitiveType.IntPtr, PrimitiveType.UIntPtr })
        {
            var narrow = type is PrimitiveType.Int32 or PrimitiveType.UInt32 ||
                IntPtr.Size == 4 && type is PrimitiveType.IntPtr or PrimitiveType.UIntPtr;
            var max = narrow ? int.MaxValue : long.MaxValue;
            var min = narrow ? int.MinValue : long.MinValue;
            foreach (var op in new[] { OpCode.Add_Ovf, OpCode.Sub_Ovf, OpCode.Mul_Ovf,
                         OpCode.Add_Ovf_Un, OpCode.Sub_Ovf_Un, OpCode.Mul_Ovf_Un })
            {
                var name = type + "-" + op;
                (long Left, long Right, long OverflowLeft, long OverflowRight, long BitsLeft, long BitsRight, long BitsExpected) values = op switch
                {
                    OpCode.Add_Ovf => (40L, 2L, max, 1L, -1L, 1L, 0L),
                    OpCode.Add_Ovf_Un => (40L, 2L, -1L, 1L, max, 1L, min),
                    OpCode.Sub_Ovf => (44L, 2L, min, 1L, min + 1, 1L, min),
                    OpCode.Sub_Ovf_Un => (44L, 2L, 0L, 1L, -1L, 1L, -2L),
                    OpCode.Mul_Ovf => (6L, 7L, max, 2L, -1L, 2L, -2L),
                    OpCode.Mul_Ovf_Un => (6L, 7L, -1L, 2L, min, 1L, min),
                    _ => throw new InvalidOperationException()
                };
                yield return new(name + "-valid", type, op, values.Left, values.Right, 42);
                yield return new(name + "-overflow", type, op, values.OverflowLeft, values.OverflowRight, null);
                // Signedness is selected by the opcode, not the parameter's type name.
                yield return new(name + "-bits", type, op, values.BitsLeft, values.BitsRight, values.BitsExpected);
            }
        }
    }

    private static AssemblyBuilder Create(Case test)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("Checked" + test.Name, new(1, 0, 0, 0)), core);
        var owner = graph.AddType("", "Arithmetic");
        var operation = owner.AddMethod("Evaluate", new(test.Type, [test.Type, test.Type]));
        operation.LoadArgument(0); operation.LoadArgument(1); operation.Emit(test.Op); operation.Return();
        var main = owner.AddMethod("Main", new(PrimitiveType.Int32, []));
        Constant(main, test.Type, test.Left); Constant(main, test.Type, test.Right);
        main.Call(operation);
        if (test.Expected is { } expected)
        {
            if (test.Type is PrimitiveType.IntPtr or PrimitiveType.UIntPtr) main.Emit(OpCode.Conv_I8);
            Constant(main, test.Type, expected);
            if (test.Type is PrimitiveType.IntPtr or PrimitiveType.UIntPtr) main.Emit(OpCode.Conv_I8);
            main.Emit(OpCode.Ceq);
            var failure = main.DefineLabel(); main.Emit(OpCode.Brfalse, failure);
            main.LoadConstant(42); main.Return();
            main.MarkLabel(failure); main.LoadConstant(1); main.Return();
        }
        else
        {
            main.Emit(OpCode.Pop); main.LoadConstant(1); main.Return();
        }
        graph.EntryPoint = main;
        return graph;
    }

    private static void Constant(MethodBuilder method, PrimitiveType type, long value)
    {
        if (type is PrimitiveType.Int32 or PrimitiveType.UInt32) method.LoadConstant(unchecked((int)value));
        else method.Emit(OpCode.Ldc_I8, value);
        if (type == PrimitiveType.IntPtr) method.Emit(OpCode.Conv_I);
        if (type == PrimitiveType.UIntPtr) method.Emit(OpCode.Conv_U);
    }

    internal static void Run(string? output = null)
    {
        if (output is not null)
        {
            if (Directory.Exists(output)) throw new IOException("output must be fresh");
            Directory.CreateDirectory(output);
        }
        var manifest = new List<object>();
        foreach (var test in Cases())
        {
            var graph = Create(test);
            var image = graph.Write();
            var loaded = Assembly.Load(image);
            try
            {
                var value = loaded.EntryPoint!.Invoke(null, null);
                if (test.Expected is null || !Equals(value, 42)) throw new Exception(test.Name + ": expected overflow or 42, got " + value);
            }
            catch (TargetInvocationException error) when (test.Expected is null && error.InnerException is OverflowException) { }
            var native = RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary);
            if (output is not null) File.WriteAllBytes(Path.Combine(output, test.Name + ".dll"), native);
            manifest.Add(new { name = test.Name, overflow = test.Expected is null });
        }
        foreach (var op in new[] { OpCode.Add_Ovf, OpCode.Sub_Ovf, OpCode.Mul_Ovf,
                     OpCode.Add_Ovf_Un, OpCode.Sub_Ovf_Un, OpCode.Mul_Ovf_Un })
        {
            var graph = Create(new("Invalid", PrimitiveType.Int32, op, 1, 2, 3));
            var method = graph.EntryPoint!;
            foreach (var invalid in new[] { "empty", "one", "mixed", "float", "bool" })
            {
                method.ClearBody();
                if (invalid is "one" or "mixed") method.LoadConstant(1);
                if (invalid == "mixed") method.Emit(OpCode.Ldc_I8, 2L);
                if (invalid == "float") { method.GetILGenerator().Emit(OpCode.Ldc_R8, 1.0); method.GetILGenerator().Emit(OpCode.Ldc_R8, 2.0); }
                if (invalid == "bool") { method.Emit(OpCode.Ldc_Bool, true); method.Emit(OpCode.Ldc_Bool, false); }
                method.Emit(op); method.Emit(OpCode.Pop); method.LoadConstant(0); method.Return();
                foreach (var write in new Func<byte[]>[] { graph.Write, graph.WriteNativeAssembly })
                {
                    try { write(); throw new Exception(op + ": accepted " + invalid); }
                    catch (InvalidDataException) { }
                }
            }
        }
        if (output is not null) File.WriteAllText(Path.Combine(output, "cases.json"), JsonSerializer.Serialize(manifest));
        Console.WriteLine($"PASS checked arithmetic: {manifest.Count} CLI executions and native images; invalid stack categories rejected");
    }
}
