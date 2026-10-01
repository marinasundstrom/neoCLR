using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class PrimitiveSignatureChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("PrimitiveSignatures", new Version(1, 0, 0, 0)), core);
        var types = new[] { PrimitiveType.Boolean };
        var signature = new PrimitiveMethodSignature(PrimitiveType.Boolean, types);
        types[0] = PrimitiveType.Int32;
        var owner = graph.AddType("", "Predicates");
        var identity = owner.AddMethod("Identity", signature);
        identity.LoadArgument(0); identity.Return();
        var integer = owner.AddMethod("Identity", 1);
        integer.LoadArgument(0); integer.Return();
        var choose = owner.AddMethod("Choose", new(PrimitiveType.Int32, [PrimitiveType.Int32, PrimitiveType.Boolean]));
        var otherwise = choose.DefineLabel();
        choose.LoadArgument(1); choose.Emit(OpCode.Brfalse, otherwise);
        choose.LoadArgument(0); choose.Return();
        choose.MarkLabel(otherwise); choose.LoadConstant(0); choose.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        main.Emit(OpCode.Ldc_Bool, false); main.Call(identity); main.Emit(OpCode.Pop);
        main.LoadConstant(7); main.Call(integer); main.Emit(OpCode.Pop);
        main.LoadConstant(42); main.Emit(OpCode.Ldc_Bool, true); main.Call(identity); main.Call(choose); main.Return();
        if (!Equals(System.Reflection.Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null), 42)) throw new Exception("primitive CLI execution");
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), core));
        var methods = projection.MainModule.Types.Single(t => t.Name == "Predicates").Methods;
        var boolean = methods.Single(m => m.Name == "Identity" && m.TryGetStaticPrimitiveSignature(out var s) && s!.ReturnType == PrimitiveType.Boolean);
        if (boolean.TryGetStaticInt32Signature(out _, out _) || !boolean.TryGetStaticPrimitiveSignature(out var decoded) ||
            !decoded!.ParameterTypes.SequenceEqual([PrimitiveType.Boolean])) throw new Exception("signature recognition");
        var consumer = new AssemblyBuilder(new("PrimitiveConsumer", new Version(1, 0, 0, 0)), core);
        var imported = consumer.ImportReference(boolean, core);
        var entry = consumer.AddFunction("Main"); consumer.EntryPoint = entry;
        var yes = entry.DefineLabel();
        entry.Emit(OpCode.Ldc_Bool, true); entry.Call(imported); entry.Emit(OpCode.Brtrue, yes);
        entry.LoadConstant(0); entry.Return(); entry.MarkLabel(yes); entry.LoadConstant(42); entry.Return();
        var importedSnapshot = AssemblyDefinition.ReadAssembly(consumer.Write(), false);
        if (!ReferenceEquals(importedSnapshot.MainModule.MemberReferences.Single().ResolveMethod(new Resolver(projection)), boolean))
            throw new Exception("Boolean MemberRef resolved wrong overload");
        _ = consumer.WriteNativeAssembly();
        Reject<ArgumentException>(() => owner.AddMethod("Identity", new(PrimitiveType.Void, [PrimitiveType.Boolean])));
        Reject<ArgumentException>(() => new PrimitiveMethodSignature(PrimitiveType.Int32, [PrimitiveType.Void]));
        Reject<ArgumentException>(() => new PrimitiveMethodSignature((PrimitiveType)99, []));
        Reject<ArgumentException>(() => new PrimitiveMethodSignature(PrimitiveType.Void, Enumerable.Repeat(PrimitiveType.Int32, 257)));
        entry.ClearBody(); entry.LoadConstant(1); entry.Call(imported); entry.Return();
        Reject<InvalidDataException>(() => consumer.Write());
        entry.ClearBody(); entry.Emit(OpCode.Pop); entry.LoadConstant(42); entry.Return();
        Reject<InvalidDataException>(() => consumer.Write());
        Reject<InvalidDataException>(() => consumer.WriteNativeAssembly());
        var finish = consumer.AddFunction("Finish", returnsValue: false); finish.Return();
        entry.ClearBody(); entry.Call(finish); entry.Emit(OpCode.Pop); entry.LoadConstant(42); entry.Return();
        Reject<InvalidDataException>(() => consumer.WriteNativeAssembly());
        Reject<ArgumentException>(() => entry.Emit(OpCode.Pop, 0));
        graph.EntryPoint = graph.AddFunction("BooleanEntry", new(PrimitiveType.Boolean, []));
        graph.EntryPoint.Emit(OpCode.Ldc_Bool, true); graph.EntryPoint.Return();
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
    }

    private sealed class Resolver(AssemblyDefinition assembly) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => identity.Equals(assembly.Identity) ? assembly : null;
    }

    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("invalid primitive contract accepted");
    }
}
