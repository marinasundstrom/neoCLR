using System.Numerics;
using System.Reflection;
using System.Text.Json;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class CheckedConversionChecks
{
    private sealed record Case(string Name, PrimitiveType Source, OpCode Op, object Value, long? Expected);

    private static IEnumerable<Case> Cases()
    {
        foreach (var (suffix, width, signed) in new[]
        {
            ("I1", 8, true), ("U1", 8, false), ("I2", 16, true), ("U2", 16, false),
            ("I4", 32, true), ("U4", 32, false), ("I8", 64, true), ("U8", 64, false),
            ("I", IntPtr.Size * 8, true), ("U", IntPtr.Size * 8, false)
        })
        {
            var lower = signed ? -(BigInteger.One << (width - 1)) : BigInteger.Zero;
            var upper = (BigInteger.One << (width - (signed ? 1 : 0))) - 1;
            foreach (var unsigned in new[] { false, true })
            {
                var op = Enum.Parse<OpCode>("Conv_Ovf_" + suffix + (unsigned ? "_Un" : ""));
                foreach (var source in new[] { PrimitiveType.Int32, PrimitiveType.Int64, PrimitiveType.IntPtr, PrimitiveType.UIntPtr })
                {
                    var sourceBits = source == PrimitiveType.Int32 ? 32 :
                        source == PrimitiveType.Int64 ? 64 : IntPtr.Size * 8;
                    var min = -(BigInteger.One << (sourceBits - 1));
                    var max = (BigInteger.One << (sourceBits - 1)) - 1;
                    var inputs = new[] { BigInteger.Zero, new BigInteger(42), -BigInteger.One,
                        min, max, lower, upper, lower - 1, upper + 1 };
                    var index = 0;
                    foreach (var raw in inputs.Distinct().Where(n => n >= min && n <= max))
                    {
                        var value = unsigned && raw < 0 ? raw + (BigInteger.One << sourceBits) : raw;
                        long? expected = value < lower || value > upper ? null :
                            unchecked((long)(ulong)(value & ulong.MaxValue));
                        yield return new($"{op}-{source}-{index++}", source, op, (long)raw, expected);
                    }
                }
                foreach (var source in new[] { PrimitiveType.Single, PrimitiveType.Double })
                {
                    var index = 0;
                    foreach (var input in new[] { 42.75, -0.75, -1.75, double.NaN,
                        double.PositiveInfinity, double.NegativeInfinity,
                        (double)lower, (double)(upper + 1) })
                    {
                        var value = source == PrimitiveType.Single ? (double)(float)input : input;
                        var truncated = Math.Truncate(value);
                        long? expected = !double.IsFinite(truncated) || truncated < (double)lower ||
                            truncated >= (double)(upper + 1) ? null :
                            unchecked((long)(ulong)(new BigInteger(truncated) & ulong.MaxValue));
                        yield return new($"{op}-{source}-{index++}", source, op, value, expected);
                    }
                }
            }
        }
    }

    private static AssemblyBuilder Graph()
    {
        var host = typeof(object).Assembly.GetName();
        return new(new("CheckedConversions", new(1, 0, 0, 0)),
            new(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? [])));
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
            var graph = Graph();
            var owner = graph.AddType("", "Conversions");
            var main = owner.AddMethod("Main", new(PrimitiveType.Int32, []));
            if (test.Source == PrimitiveType.Int32) main.LoadConstant((int)(long)test.Value);
            else if (test.Source == PrimitiveType.Single) main.GetILGenerator().Emit(OpCode.Ldc_R4, (float)(double)test.Value);
            else if (test.Source == PrimitiveType.Double) main.GetILGenerator().Emit(OpCode.Ldc_R8, (double)test.Value);
            else main.Emit(OpCode.Ldc_I8, (long)test.Value);
            if (test.Source == PrimitiveType.IntPtr) main.Emit(OpCode.Conv_I);
            if (test.Source == PrimitiveType.UIntPtr) main.Emit(OpCode.Conv_U);
            main.Emit(test.Op);
            // Compare raw result bits, preserving UInt32 zero extension before widening.
            var name = test.Op.ToString();
            main.Emit(name.StartsWith("Conv_Ovf_U4") ? OpCode.Conv_U8 : OpCode.Conv_I8);
            if (test.Expected is { } expected)
            {
                main.Emit(OpCode.Ldc_I8, expected);
                main.Emit(OpCode.Ceq);
                var bad = main.DefineLabel();
                main.Emit(OpCode.Brfalse, bad);
                main.LoadConstant(42); main.Return();
                main.MarkLabel(bad);
            }
            else main.Emit(OpCode.Pop);
            main.LoadConstant(1); main.Return();
            graph.EntryPoint = main;
            try
            {
                var actual = Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null);
                if (test.Expected is null || !Equals(actual, 42))
                    throw new Exception(test.Name + ": expected overflow or 42, got " + actual);
            }
            catch (TargetInvocationException e) when (test.Expected is null && e.InnerException is OverflowException) { }
            var image = RuntimeAssemblyContainer.WriteBinary(graph);
            if (output is not null) File.WriteAllBytes(Path.Combine(output, test.Name + ".dll"), image);
            manifest.Add(new { name = test.Name, overflow = test.Expected is null });
        }
        foreach (var op in Enum.GetValues<OpCode>().Where(op => op.ToString().StartsWith("Conv_Ovf_")))
        {
            foreach (var invalid in new[] { "empty", "bool", "string" })
            {
                var graph = Graph();
                var main = graph.AddType("", "Invalid").AddMethod("Main", new(PrimitiveType.Int32, []));
                if (invalid == "bool") main.Emit(OpCode.Ldc_Bool, true);
                if (invalid == "string") main.Emit(OpCode.Ldstr, "invalid");
                main.Emit(op); main.Emit(OpCode.Pop); main.LoadConstant(0); main.Return();
                graph.EntryPoint = main;
                foreach (var write in new Func<byte[]>[] { graph.Write, graph.WriteNativeAssembly })
                {
                    try { write(); throw new Exception(op + ": accepted " + invalid); }
                    catch (InvalidDataException) { }
                }
            }
        }
        if (output is not null) File.WriteAllText(Path.Combine(output, "cases.json"), JsonSerializer.Serialize(manifest));
        Console.WriteLine($"PASS checked conversions: {manifest.Count} CLI executions and native images; invalid stacks rejected");
    }
}
