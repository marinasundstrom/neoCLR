using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class StringChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("TextValues", new Version(1, 0, 0, 0)), core);
        var type = graph.AddType("", "Text");
        var echo = type.AddMethod("Echo", new(PrimitiveType.String, [PrimitiveType.String]));
        var local = echo.DeclareLocal(PrimitiveType.String);
        echo.LoadArgument(0); echo.StoreLocal(local); echo.LoadLocal(local); echo.Return();
        var literal = type.AddMethod("Literal", new(PrimitiveType.String, []));
        const string text = "Hej 🌍\0 café";
        literal.Emit(OpCode.Ldstr, text); literal.Return();
        var empty = type.AddMethod("Empty", new(PrimitiveType.String, []));
        empty.Emit(OpCode.Ldstr, ""); empty.Return();
        var loaded = System.Reflection.Assembly.Load(graph.Write()).GetType("Text")!;
        foreach (var value in new[] { "", text })
            Check(Equals(loaded.GetMethod("Echo")!.Invoke(null, [value]), value), "CLI String argument/local/result");
        Check(Equals(loaded.GetMethod("Literal")!.Invoke(null, null), text), "CLI Unicode literal");
        Check(Equals(loaded.GetMethod("Empty")!.Invoke(null, null), ""), "CLI empty literal");
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), core));
        var definition = projection.MainModule.Types.Single(t => t.Name == "Text").Methods.Single(m => m.Name == "Echo");
        Check(!definition.TryGetStaticInt32Signature(out _, out _) && definition.TryGetStaticPrimitiveSignature(out var signature) &&
            signature!.ReturnType == PrimitiveType.String && signature.ParameterTypes.SequenceEqual(new[] { PrimitiveType.String }), "String projection signature");
        var consumer = new AssemblyBuilder(new("TextConsumer", new Version(1, 0, 0, 0)), core);
        var main = consumer.AddFunction("Main"); consumer.EntryPoint = main;
        main.Emit(OpCode.Ldstr, text); main.Call(consumer.ImportReference(definition, core)); main.Emit(OpCode.Pop); main.LoadConstant(42); main.Return();
        _ = consumer.Write(); _ = consumer.WriteNativeAssembly();
        graph.EntryPoint = literal; Reject(() => graph.Write(), typeof(InvalidDataException)); graph.EntryPoint = null;
        // Operand rejection must leave the previous valid body unchanged.
        Reject(() => literal.Emit(OpCode.Ldstr, (string)null!), typeof(ArgumentNullException));
        Reject(() => literal.Emit(OpCode.Add, "text"), typeof(ArgumentException));
        Reject(() => literal.Emit(OpCode.Ldstr, "\ud800"), typeof(ArgumentException));
        Reject(() => literal.Emit(OpCode.Ldstr, new string('é', 32769)), typeof(ArgumentException));
        _ = graph.Write();
        literal.ClearBody(); literal.Emit(OpCode.Ldstr, new string('é', 32768)); literal.Return(); _ = graph.WriteNativeAssembly();
        var equality = type.AddMethod("Equality", new(PrimitiveType.Boolean, []));
        equality.Emit(OpCode.Ldstr, "a"); equality.Emit(OpCode.Ldstr, "b"); equality.Emit(OpCode.Ceq); equality.Return();
        Reject(() => graph.Write(), typeof(InvalidDataException));
        equality.ClearBody(); equality.Emit(OpCode.Ldc_Bool, true); equality.Return();
        foreach (var op in new[] { OpCode.Add })
        {
            literal.ClearBody(); literal.Emit(OpCode.Ldstr, "a"); literal.Emit(OpCode.Ldstr, "b"); literal.Emit(op); literal.Return();
            Reject(() => graph.Write(), typeof(InvalidDataException));
        }
        literal.ClearBody(); literal.LoadConstant(1); literal.StoreLocal(literal.DeclareLocal(PrimitiveType.String)); literal.Return();
        Reject(() => graph.Write(), typeof(InvalidDataException));
        literal.ClearBody(); literal.Emit(OpCode.Ldstr, "printed"); literal.WriteConsoleLine(); literal.Emit(OpCode.Ldstr, "returned"); literal.Return();
        _ = graph.WriteNativeAssembly(); Reject(() => graph.Write(), typeof(InvalidDataException));
        literal.ClearBody(); literal.LoadConstant(1); literal.WriteConsoleLine(); literal.Emit(OpCode.Ldstr, ""); literal.Return();
        Reject(() => graph.WriteNativeAssembly(), typeof(InvalidDataException));
    }

    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject(Action action, Type expected)
    {
        try { action(); } catch (Exception error) when (error.GetType() == expected) { return; }
        throw new Exception("invalid String contract accepted");
    }
}
