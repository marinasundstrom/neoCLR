using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class UnaryIntegerChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("UnaryIntegers", new Version(1, 0, 0, 0)), core);
        var type = graph.AddType("", "Operations");
        foreach (var width in new[] { PrimitiveType.Int32, PrimitiveType.Int64 })
            foreach (var op in new[] { OpCode.Neg, OpCode.Not })
            {
                var method = type.AddMethod(op + width.ToString(), new(width, [width]));
                method.LoadArgument(0); method.Emit(op); method.Return();
            }
        var loaded = System.Reflection.Assembly.Load(graph.Write()).GetType("Operations")!;
        foreach (var value in new[] { int.MinValue, -1, 0, 42, int.MaxValue })
        {
            if (!Equals(loaded.GetMethod("NegInt32")!.Invoke(null, [value]), unchecked(-value))) throw new Exception("Int32 negation");
            if (!Equals(loaded.GetMethod("NotInt32")!.Invoke(null, [value]), ~value)) throw new Exception("Int32 complement");
        }
        foreach (var value in new[] { long.MinValue, -1L, 0L, 42L, long.MaxValue })
        {
            if (!Equals(loaded.GetMethod("NegInt64")!.Invoke(null, [value]), unchecked(-value))) throw new Exception("Int64 negation");
            if (!Equals(loaded.GetMethod("NotInt64")!.Invoke(null, [value]), ~value)) throw new Exception("Int64 complement");
        }
        _ = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), core));
        var invalid = graph.AddFunction("Invalid");
        foreach (var op in new[] { OpCode.Neg, OpCode.Not })
        {
            invalid.ClearBody(); invalid.Emit(op); invalid.Return();
            Reject(() => graph.Write());
            invalid.ClearBody(); invalid.Emit(OpCode.Ldc_Bool, true); invalid.Emit(op); invalid.Return();
            Reject(() => graph.WriteNativeAssembly());
        }
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid unary operand accepted");
    }
}
