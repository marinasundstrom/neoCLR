using System.Diagnostics;
using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ByReferenceChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = DefaultValueChecks.Create();
        var t = SignatureType.MethodParameter(0);
        var copy = graph.AddFunction("Replace", new MethodSignature(PrimitiveType.Void, [SignatureType.ByReference(t), t], ["T"]));
        copy.LoadArgument(0); copy.LoadArgument(1); copy.StoreObject(t); copy.Return();
        var increment = graph.AddType("Example", "RefOperations").AddMethod("Increment", new MethodSignature(PrimitiveType.Void, [SignatureType.ByReference(PrimitiveType.Int32)]));
        increment.LoadArgument(0); increment.Duplicate(); increment.LoadObject(PrimitiveType.Int32);
        increment.LoadConstant(2); increment.Emit(OpCode.Add); increment.StoreObject(PrimitiveType.Int32); increment.Return();
        var forward = graph.AddFunction("Forward", increment.Signature);
        forward.LoadArgument(0); forward.Call(increment); forward.Return();
        var main = graph.EntryPoint!; main.ClearBody();
        var local = main.DeclareLocal(PrimitiveType.Int32);
        main.LoadConstant(0); main.StoreLocal(local);
        main.LoadLocalAddress(local); main.LoadConstant(40); main.Call(copy.MakeGenericInstance(PrimitiveType.Int32));
        main.LoadLocalAddress(local); main.Call(forward);
        main.LoadLocal(local); main.Return();
        return graph;
    }

    private static AssemblyBuilder Consumer(AssemblyBuilder library, AssemblyDefinition snapshot)
    {
        var consumer = new AssemblyBuilder(new("RefConsumer", new Version(1, 0, 0, 0)), library.CoreLibrary);
        var imported = consumer.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "RefOperations").Methods.Single(), library.CoreLibrary);
        if (imported.Signature.ParameterTypes[0].ByReferenceElement?.Primitive != PrimitiveType.Int32)
            throw new Exception("imported ref signature lost");
        var main = consumer.AddFunction("Main"); consumer.EntryPoint = main;
        var local = main.DeclareLocal(PrimitiveType.Int32);
        main.LoadConstant(40); main.StoreLocal(local);
        main.LoadLocalAddress(local); main.Call(imported); main.LoadLocal(local); main.Return();
        return consumer;
    }

    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("ref call execution");
        var replace = loaded.ManifestModule.GetMethods().Single(m => m.Name == "Replace");
        if (!replace.GetParameters()[0].ParameterType.IsByRef) throw new Exception("missing CLI byref signature");
        var projected = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        var snapshot = projected;
        if (!snapshot.MainModule.Functions.Single(m => m.Name == "Replace").TryGetStaticGenericValueSignature(out var decoded)
            || decoded!.ParameterTypes[0].ByReferenceElement?.MethodParameterIndex != 0)
            throw new Exception("projected generic ref signature lost");
        foreach (var source in new[] { snapshot, AssemblyDefinition.ReadAssembly(graph.Write(), expectedExtended: false) })
        {
            var consumer = Consumer(graph, source);
            var context = new System.Runtime.Loader.AssemblyLoadContext("byref-consumer", true);
            try
            {
                context.LoadFromStream(new MemoryStream(graph.Write()));
                var app = context.LoadFromStream(new MemoryStream(consumer.Write()));
                if (!Equals(app.EntryPoint!.Invoke(null, null), 42)) throw new Exception("imported byref call");
            }
            finally { context.Unload(); }
        }
        if (snapshot.MainModule.Functions.All(m => m.Name != "Replace")) throw new Exception("missing projected byref function");
        var main = graph.EntryPoint!;
        var local = main.DeclareLocal(PrimitiveType.Int32);
        var increment = graph.Types.Single(t => t.Name == "RefOperations").Methods.Single();
        main.ClearBody(); main.LoadLocalAddress(local); main.Call(increment); main.LoadConstant(0); main.Return();
        Reject(graph, "uninitialized ref call");
        main.ClearBody(); main.LoadConstant(0); main.Call(increment); main.LoadConstant(0); main.Return();
        Reject(graph, "value passed to ref call");
        var wrong = main.DeclareLocal(PrimitiveType.Int64);
        main.ClearBody(); main.LoadLocalAddress(wrong); main.InitializeObject(PrimitiveType.Int64);
        main.LoadLocalAddress(wrong); main.Call(increment); main.LoadConstant(0); main.Return();
        Reject(graph, "wrong ref target");
        var byref = SignatureType.ByReference(PrimitiveType.Int32);
        void Invalid(Action action) { try { action(); } catch (ArgumentException) { return; } throw new Exception("invalid byref shape admitted"); }
        Invalid(() => SignatureType.ByReference(PrimitiveType.Void));
        Invalid(() => SignatureType.ByReference(byref));
        Invalid(() => SignatureType.ArrayOf(byref));
        Invalid(() => main.DeclareLocal(byref));
        Invalid(() => new MethodSignature(byref, []));
        Invalid(() => graph.Functions.Single(m => m.Name == "Replace").MakeGenericInstance(byref));
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
        var path = Path.Combine(output, "ByReference.dll");
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
        var consumer = Consumer(graph, RuntimeAssemblyContainer.ReadCliProjection(File.ReadAllBytes(path)));
        graph.EntryPoint = null;
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        var consumerPath = Path.Combine(output, "Consumer.dll");
        File.WriteAllBytes(consumerPath, RuntimeAssemblyContainer.WriteBinary(consumer.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var arg in new[] { command, consumerPath, "--module", path }) start.ArgumentList.Add(arg);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("PASS managed-reference calls: CLR and native library/consumer verify/run 42");
    }
}
