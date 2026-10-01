using System.Diagnostics;
using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class LocalObjectChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = DefaultValueChecks.Create();
        var t = SignatureType.MethodParameter(0);
        var copy = graph.AddFunction("CopyThroughAddress", new MethodSignature(t, [t], ["T"]));
        var slot = copy.DeclareLocal(t);
        copy.LoadLocalAddress(slot);
        copy.LoadArgument(0);
        copy.Emit(OpCode.Stobj, t);
        copy.LoadLocalAddress(slot);
        copy.Emit(OpCode.Ldobj, t);
        copy.Return();
        var main = graph.EntryPoint!;
        main.ClearBody();
        main.Emit(OpCode.Ldstr, "managed reference");
        main.Call(copy.MakeGenericInstance(PrimitiveType.String));
        main.Emit(OpCode.Pop);
        main.LoadDefault(SignatureType.ArrayOf(PrimitiveType.Int32));
        main.Call(copy.MakeGenericInstance(SignatureType.ArrayOf(PrimitiveType.Int32)));
        main.Emit(OpCode.Pop);
        var local = main.DeclareLocal(PrimitiveType.Int32);
        var alternate = main.DefineLabel();
        var done = main.DefineLabel();
        main.Emit(OpCode.Ldc_Bool, false);
        main.Emit(OpCode.Brtrue, alternate);
        main.LoadLocalAddress(local);
        main.LoadConstant(40);
        main.StoreObject(PrimitiveType.Int32);
        main.Emit(OpCode.Br, done);
        main.MarkLabel(alternate);
        main.LoadLocalAddress(local);
        main.LoadConstant(41);
        main.StoreObject(PrimitiveType.Int32);
        main.MarkLabel(done);
        main.LoadLocalAddress(local);
        main.Duplicate();
        main.LoadObject(PrimitiveType.Int32);
        main.LoadConstant(2);
        main.Emit(OpCode.Add);
        main.StoreObject(PrimitiveType.Int32);
        main.LoadLocal(local);
        main.Call(copy.MakeGenericInstance(PrimitiveType.Int32));
        main.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("local address execution");
        var copy = loaded.ManifestModule.GetMethods().Single(m => m.Name == "CopyThroughAddress");
        foreach (var value in new object[] { 42, 123L, true, "managed reference" })
            if (!Equals(copy.MakeGenericMethod(value.GetType()).Invoke(null, [value]), value))
                throw new Exception("generic indirect copy");
        _ = graph.WriteNativeAssembly();
        var main = graph.EntryPoint!;
        var local = main.DeclareLocal(PrimitiveType.Int32);
        main.ClearBody();
        main.LoadLocalAddress(local); main.LoadObject(PrimitiveType.Int32); main.Return();
        Reject(graph, "uninitialized indirect read");
        main.ClearBody();
        main.LoadLocalAddress(local); main.LoadConstant(42); main.StoreObject(PrimitiveType.Int64);
        main.LoadLocal(local); main.Return();
        Reject(graph, "wrong stored value");
        main.ClearBody();
        main.LoadLocalAddress(local); main.Emit(OpCode.Ldc_I8, 42L); main.StoreObject(PrimitiveType.Int64);
        main.LoadLocal(local); main.Return();
        Reject(graph, "wrong addressed type");
        main.ClearBody();
        main.LoadConstant(0); main.LoadObject(PrimitiveType.Int32); main.Return();
        Reject(graph, "non-address load");
        main.ClearBody();
        var skip = main.DefineLabel();
        main.Emit(OpCode.Ldc_Bool, true); main.Emit(OpCode.Brtrue, skip);
        main.LoadLocalAddress(local); main.LoadConstant(42); main.StoreObject(PrimitiveType.Int32);
        main.MarkLabel(skip); main.LoadLocalAddress(local); main.LoadObject(PrimitiveType.Int32); main.Return();
        Reject(graph, "store must dominate indirect read");
        foreach (var code in new[] { OpCode.Ldobj, OpCode.Stobj })
        {
            try { main.Emit(code, (SignatureType)PrimitiveType.Void); throw new Exception("Void accepted"); }
            catch (ArgumentException) { }
        }
    }

    private static void Reject(AssemblyBuilder graph, string detail)
    {
        try { graph.Write(); throw new Exception(detail); } catch (InvalidDataException) { }
        try { graph.WriteNativeAssembly(); throw new Exception(detail); } catch (InvalidDataException) { }
    }

    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var graph = Create();
        var path = Path.Combine(output, "LocalObjects.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync();
            var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync();
            var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("PASS typed local object binary: CLR 42, neoCLR verify/run 42");
    }
}
