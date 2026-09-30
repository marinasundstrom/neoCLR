using NeoCLR.Metadata.Experimental.Model;

internal static class FlowChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var assembly = new AssemblyBuilder(new("Flow", new Version(1, 0, 0, 0)), core);
        var main = assembly.AddFunction("Main"); assembly.EntryPoint = main;
        var value = main.DeclareInt32Local();
        var loop = main.DefineLabel(); var done = main.DefineLabel();
        main.LoadConstant(0); main.StoreLocal(value);
        main.MarkLabel(loop); main.LoadLocal(value); main.LoadConstant(42); main.Emit(OpCode.Clt); main.Emit(OpCode.Brfalse, done);
        main.LoadLocal(value); main.LoadConstant(1); main.Add(); main.StoreLocal(value); main.Emit(OpCode.Br, loop);
        main.MarkLabel(done); main.LoadLocal(value); main.Return();
        if (!Equals(System.Reflection.Assembly.Load(assembly.Write()).EntryPoint!.Invoke(null, null), 42)) throw new Exception("CLI loop execution");
        _ = assembly.WriteNativeAssembly();
        Reject<ArgumentException>(() => main.MarkLabel(done));
        main.ClearBody();
        main.Emit(OpCode.Ldc_Bool, false); main.Emit(OpCode.Ldc_Bool, false); main.Emit(OpCode.Ceq);
        main.Emit(OpCode.Brtrue, done); main.LoadConstant(0); main.Return();
        main.MarkLabel(done); main.LoadConstant(42); main.Return();
        if (!Equals(System.Reflection.Assembly.Load(assembly.Write()).EntryPoint!.Invoke(null, null), 42)) throw new Exception("Boolean equality execution");
        _ = assembly.WriteNativeAssembly();
        var other = assembly.AddFunction("Other"); other.LoadConstant(0); other.Return();
        Reject<ArgumentException>(() => other.Emit(OpCode.Br, done));
        main.ClearBody(); main.Emit(OpCode.Br, done); main.Return();
        Reject<InvalidDataException>(() => assembly.Write());
        main.ClearBody(); main.LoadConstant(1); main.Emit(OpCode.Brtrue, done); main.MarkLabel(done); main.LoadConstant(0); main.Return();
        Reject<InvalidDataException>(() => assembly.WriteNativeAssembly());
        main.ClearBody(); main.Emit(OpCode.Ldc_Bool, true); main.Emit(OpCode.Brtrue, done);
        main.LoadConstant(42); main.StoreLocal(value); main.MarkLabel(done); main.LoadLocal(value); main.Return();
        Reject<InvalidDataException>(() => assembly.Write());
        main.ClearBody(); main.Emit(OpCode.Ldc_Bool, true); main.Emit(OpCode.Brtrue, done);
        main.LoadConstant(42); main.MarkLabel(done); main.Return();
        Reject<InvalidDataException>(() => assembly.WriteNativeAssembly());
        main.ClearBody(); main.Emit(OpCode.Ldc_Bool, true); main.Return();
        Reject<InvalidDataException>(() => assembly.Write());
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("invalid control flow accepted");
    }
}
