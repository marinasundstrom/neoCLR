using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class Int64Checks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("WideValues", new Version(1, 0, 0, 0)), core);
        var type = graph.AddType("", "Wide");
        var widen = type.AddMethod("Widen", new(PrimitiveType.Int64, [PrimitiveType.Int32]));
        widen.LoadArgument(0); widen.Emit(OpCode.Conv_I8); widen.Return();
        var narrow = type.AddMethod("Narrow", new(PrimitiveType.Int32, [PrimitiveType.Int64]));
        var local = narrow.DeclareLocal(PrimitiveType.Int64);
        narrow.LoadArgument(0); narrow.StoreLocal(local); narrow.LoadLocal(local); narrow.Emit(OpCode.Conv_I4); narrow.Return();
        var maximum = type.AddMethod("Maximum", new(PrimitiveType.Int64, []));
        maximum.Emit(OpCode.Ldc_I8, long.MaxValue); maximum.Return();
        var minimum = type.AddMethod("Minimum", new(PrimitiveType.Int64, []));
        minimum.Emit(OpCode.Ldc_I8, long.MinValue); minimum.Return();
        var loaded = System.Reflection.Assembly.Load(graph.Write()).GetType("Wide")!;
        foreach (var value in new[] { int.MinValue, -1, 0, int.MaxValue })
            if (!Equals(loaded.GetMethod("Widen")!.Invoke(null, [value]), (long)value)) throw new Exception("signed widening");
        foreach (var value in new[] { long.MinValue, -1L, 4294967338L, long.MaxValue })
            if (!Equals(loaded.GetMethod("Narrow")!.Invoke(null, [value]), unchecked((int)value))) throw new Exception("truncating narrowing");
        if (!Equals(loaded.GetMethod("Maximum")!.Invoke(null, null), long.MaxValue) ||
            !Equals(loaded.GetMethod("Minimum")!.Invoke(null, null), long.MinValue)) throw new Exception("Int64 constants");
        var native = graph.WriteNativeAssembly();
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(native, core));
        var definition = projection.MainModule.Types.Single(t => t.Name == "Wide").Methods.Single(m => m.Name == "Maximum");
        if (definition.TryGetStaticInt32Signature(out _, out _) || !definition.TryGetStaticPrimitiveSignature(out var signature) ||
            signature!.ReturnType != PrimitiveType.Int64) throw new Exception("Int64 recognition");
        var consumer = new AssemblyBuilder(new("WideConsumer", new Version(1, 0, 0, 0)), core);
        var main = consumer.AddFunction("Main"); consumer.EntryPoint = main;
        main.Call(consumer.ImportReference(definition, core)); main.Emit(OpCode.Conv_I4); main.Return();
        _ = consumer.Write(); _ = consumer.WriteNativeAssembly();
        graph.EntryPoint = maximum;
        Reject(() => graph.WriteNativeAssembly()); graph.EntryPoint = null;
        maximum.ClearBody(); maximum.Emit(OpCode.Ldc_Bool, true); maximum.Emit(OpCode.Conv_I8); maximum.Return();
        Reject(() => graph.Write());
        maximum.ClearBody(); maximum.Emit(OpCode.Ldc_I8, 1L); maximum.LoadConstant(2); maximum.Add(); maximum.Return();
        Reject(() => graph.WriteNativeAssembly());
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid Int64 contract accepted");
    }
}
