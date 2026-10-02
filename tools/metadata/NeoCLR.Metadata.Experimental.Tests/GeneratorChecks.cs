using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental.Model;

internal static class GeneratorChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var app = new AssemblyBuilder(new("GeneratorApp", new Version(1, 0, 0, 0)), core);
        var twice = app.AddFunction("Twice", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        IILGenerator helper = twice.GetILGenerator();
        helper.LoadArgument(0); helper.LoadConstant(2); helper.Multiply(); helper.Return();
        var entry = app.AddFunction("Main"); app.EntryPoint = entry;
        IILGenerator il = entry.GetILGenerator();
        if (!ReferenceEquals(il, entry.Definition.GetILGenerator())) throw new Exception("canonical generator");
        var local = il.DeclareInt32Local();
        il.LoadConstant(7); il.Return(); il.ClearBody();
        if (il.Locals.Count != 1 || !ReferenceEquals(local, il.Locals[0])) throw new Exception("clear preserves locals");
        var end = il.DefineLabel();
        il.Emit(OpCode.Ldc_I4, 21); il.Call(twice); il.StoreLocal(local);
        il.Emit(OpCode.Br, end); il.MarkLabel(end); il.LoadLocal(local); il.Emit(OpCode.Ret);
        Reject<ArgumentException>(() => helper.LoadLocal(local));
        Reject<ArgumentException>(() => helper.MarkLabel(end));
        Reject<ArgumentException>(() => il.Emit(OpCode.Call, 4));
        var context = new AssemblyLoadContext("generator-check", true);
        try
        {
            var loaded = context.LoadFromStream(new MemoryStream(app.Write()));
            if (!Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("generator CLR execution");
        }
        finally { context.Unload(); }
        var detached = new MethodDefinition("Detached", new MethodSignature(PrimitiveType.Void, []));
        Reject<InvalidOperationException>(() => detached.GetILGenerator());
        var read = AssemblyDefinition.ReadAssembly(app.Write(), expectedExtended: false);
        Reject<InvalidOperationException>(() => read.MainModule.Functions.First().GetILGenerator());
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
