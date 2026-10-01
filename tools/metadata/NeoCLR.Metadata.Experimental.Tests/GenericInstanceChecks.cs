using System.Reflection;
using System.Diagnostics;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericInstanceChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = InstanceObjectChecks.Create();
        var owner = graph.Types[0];
        var t = SignatureType.MethodParameter(0);
        var remember = owner.AddInstanceMethod("Remember", new MethodSignature(t, [t, PrimitiveType.Int32], ["T"]));
        remember.LoadArgument(0); remember.LoadArgument(2); remember.StoreField(owner.Fields[0]);
        var copy = remember.DeclareLocal(t);
        remember.LoadArgument(1); remember.StoreLocal(copy); remember.LoadLocal(copy); remember.Return();
        var forward = owner.AddInstanceMethod("Forward", new MethodSignature(t, [t, PrimitiveType.Int32], ["U"]));
        forward.LoadArgument(0); forward.LoadArgument(1); forward.LoadArgument(2);
        forward.Emit(OpCode.Call, remember.MakeGenericInstance(t)); forward.Return();
        var main = graph.EntryPoint!; main.ClearBody();
        var local = main.DeclareLocal(owner);
        main.LoadConstant(1); main.Emit(OpCode.Ldc_Bool, true); main.NewObject(owner.Methods[0]); main.StoreLocal(local);
        main.LoadLocal(local); main.LoadLocal(local); main.LoadConstant(42);
        main.Call(forward.MakeGenericInstance(owner)); main.Call(owner.Methods.Single(m => m.Name == "GetNumber")); main.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("generic instance execution");
        var projection = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        var method = projection.MainModule.Types.Single(t => t.Name == "Order").Methods.Single(m => m.Name == "Forward");
        if (method.IsStatic || method.GenericArity != 1) throw new Exception("generic receiver projection");
        var main = graph.EntryPoint!; main.ClearBody();
        main.LoadConstant(1); main.LoadConstant(2); main.Call(graph.Types[0].Methods.Single(m => m.Name == "Remember").MakeGenericInstance(PrimitiveType.Int32)); main.Return();
        try { graph.Write(); throw new Exception("missing receiver accepted"); } catch (InvalidDataException) { }
        try { graph.WriteNativeAssembly(); throw new Exception("missing native receiver accepted"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var graph = Create();
        var path = Path.Combine(output, "GenericInstances.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("PASS generic instance binary: receiver mutation, forwarding and identity, verify/run 42");
    }

}
