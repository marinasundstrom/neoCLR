using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class LocalChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var legacy = new AssemblyBuilder(new("LegacyLocals", new Version(1, 0, 0, 0)), core);
        var legacyMain = legacy.AddFunction("Main"); legacyMain.LoadConstant(42); legacyMain.Return(); legacy.EntryPoint = legacyMain;
        var legacyJson = System.Text.Json.Nodes.JsonNode.Parse(legacy.WriteNativeAssembly())!;
        foreach (var function in legacyJson["functions"]!.AsArray()) function!.AsObject().Remove("locals");
        var legacyBytes = System.Text.Encoding.UTF8.GetBytes(legacyJson.ToJsonString());
        _ = NativeAssemblyDefinition.ReadAssembly(legacyBytes);
        _ = RuntimeAssemblyContainer.Read(RuntimeAssemblyContainer.WriteBinary(legacyBytes, core));
        var assembly = new AssemblyBuilder(new("Locals", new Version(1, 0, 0, 0)), core);
        var main = assembly.AddFunction("Main"); assembly.EntryPoint = main;
        var local = main.DeclareInt32Local();
        main.LoadConstant(40); main.StoreLocal(local); main.LoadLocal(local); main.LoadConstant(2); main.Add(); main.Return();
        var pe = assembly.Write();
        if (!Equals(System.Reflection.Assembly.Load(pe).EntryPoint!.Invoke(null, null), 42)) throw new Exception("local CLI execution");
        var native = assembly.WriteNativeAssembly();
        _ = NativeAssemblyDefinition.ReadAssembly(native);
        _ = RuntimeAssemblyContainer.Read(RuntimeAssemblyContainer.WriteBinary(native, core));
        main.ClearBody();
        main.Emit(OpCode.Ldc_I4, 40); main.Emit(OpCode.Stloc, local); main.Emit(OpCode.Ldloc, local.Index);
        main.Emit(OpCode.Ldc_I4, 2); main.Emit(OpCode.Add); main.Emit(OpCode.Ret);
        if (!assembly.Write().SequenceEqual(pe) || !assembly.WriteNativeAssembly().SequenceEqual(native)) throw new Exception("local overload equivalence");
        var other = assembly.AddFunction("Other"); other.LoadConstant(0); other.Return();
        Reject<ArgumentException>(() => other.LoadLocal(local));
        Reject<ArgumentException>(() => main.Emit(OpCode.Call, local));
        Reject<ArgumentNullException>(() => main.LoadLocal(null!));
        main.ClearBody(); main.LoadLocal(local); main.Return();
        Reject<InvalidDataException>(() => assembly.Write());
        Reject<InvalidDataException>(() => assembly.WriteNativeAssembly());
        main.ClearBody(); main.LoadConstant(1); main.Emit(OpCode.Stloc, 256); main.LoadConstant(1); main.Return();
        Reject<InvalidDataException>(() => assembly.Write());
        main.ClearBody(); main.StoreLocal(local); main.LoadLocal(local); main.Return();
        Reject<InvalidDataException>(() => assembly.WriteNativeAssembly());
        for (int i = 1; i < 256; i++) main.DeclareInt32Local();
        Reject<InvalidDataException>(() => main.DeclareInt32Local());
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("invalid local contract accepted");
    }
}
