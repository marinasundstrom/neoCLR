using System.Diagnostics;
using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class InterfaceDispatchChecks
{
    internal static AssemblyBuilder Create()
    {
        var host = typeof(object).Assembly.GetName();
        var graph = new AssemblyBuilder(new("InterfaceDispatch", new Version(1, 0, 0, 0)),
            new(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? [])));
        var contract = graph.AddInterface("Example", "Value");
        var get = contract.AddInterfaceMethod("Get", new(PrimitiveType.Int32, []));
        var derived = graph.AddInterface("Example", "Derived"); derived.AddBaseInterface(contract);
        var constructors = new List<MethodBuilder>();
        foreach (var (name, value) in new[] { ("First", 19), ("Second", 23) })
        {
            var type = graph.AddClass("Example", name); type.AddInterfaceImplementation(derived);
            var ctor = type.AddConstructor(Array.Empty<PrimitiveType>()); ctor.Return(); constructors.Add(ctor);
            var method = type.AddInstanceMethod("Get", new(PrimitiveType.Int32, [])); method.LoadConstant(value); method.Return();
        }
        var invoke = graph.AddFunction("Invoke", new MethodSignature(PrimitiveType.Int32, [contract]));
        invoke.LoadArgument(0); invoke.CallVirtual(get); invoke.Return();
        var main = graph.AddFunction("Main");
        foreach (var ctor in constructors) { main.NewObject(ctor); main.Call(invoke); }
        main.Emit(OpCode.Add); main.Return(); graph.EntryPoint = main;
        return graph;
    }
    internal static void Run()
    {
        var graph = Create(); var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("CLI interface dispatch");
        var implementation = loaded.GetType("Example.First")!.GetInterfaceMap(loaded.GetType("Example.Value")!);
        if (implementation.TargetMethods.Single().Name != "Get") throw new Exception("interface map");
        _ = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary);
        void Reject(Action action) { try { action(); } catch (Exception e) when (e is ArgumentException or InvalidOperationException or InvalidDataException) { return; } throw new Exception("invalid dispatch accepted"); }
        Reject(() => graph.EntryPoint!.CallVirtual(graph.Functions[0]));
        var missing = graph.AddClass("Example", "Missing"); missing.AddInterfaceImplementation(graph.Types[0]);
        Reject(() => graph.Write()); Reject(() => graph.WriteNativeAssembly());
        var wrongReceiver = Create(); wrongReceiver.EntryPoint!.ClearBody();
        var unrelated = wrongReceiver.AddClass("Example", "Unrelated"); var unrelatedCtor = unrelated.AddConstructor(Array.Empty<PrimitiveType>()); unrelatedCtor.Return();
        wrongReceiver.EntryPoint.NewObject(unrelatedCtor); wrongReceiver.EntryPoint.CallVirtual(wrongReceiver.Types[0].Methods[0]); wrongReceiver.EntryPoint.Return();
        Reject(() => wrongReceiver.Write());
        var nullReceiver = Create(); nullReceiver.EntryPoint!.ClearBody(); nullReceiver.EntryPoint.LoadDefault(nullReceiver.Types[0]);
        nullReceiver.EntryPoint.Emit(OpCode.Callvirt, nullReceiver.Types[0].Methods[0]); nullReceiver.EntryPoint.Return();
        try { Assembly.Load(nullReceiver.Write()).EntryPoint!.Invoke(null, null); throw new Exception("null callvirt returned"); }
        catch (TargetInvocationException e) when (e.InnerException is NullReferenceException) { }
        var direct = Create(); direct.Functions[0].ClearBody(); direct.Functions[0].LoadArgument(0); direct.Functions[0].Call(direct.Types[0].Methods[0]); direct.Functions[0].Return();
        Reject(() => direct.Write());
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run(); if (Directory.Exists(output)) throw new IOException("output must be fresh"); Directory.CreateDirectory(output);
        var graph = Create(); var path = Path.Combine(output, "Dispatch.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        graph.EntryPoint!.ClearBody(); graph.EntryPoint.LoadDefault(graph.Types[0]); graph.EntryPoint.CallVirtual(graph.Types[0].Methods[0]); graph.EntryPoint.Return();
        var nullPath = Path.Combine(output, "NullDispatch.dll"); File.WriteAllBytes(nullPath, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        var nullStart = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
        nullStart.ArgumentList.Add("run"); nullStart.ArgumentList.Add(nullPath);
        using var nullProcess = Process.Start(nullStart)!; var nullOut = nullProcess.StandardOutput.ReadToEndAsync(); var nullErr = nullProcess.StandardError.ReadToEndAsync();
        await nullProcess.WaitForExitAsync(); var fault = await nullOut + await nullErr;
        if (nullProcess.ExitCode == 0 || !fault.Contains("null", StringComparison.OrdinalIgnoreCase)) throw new Exception("expected null interface receiver fault: " + fault);
        Console.WriteLine("PASS CLI/native interface dispatch through inherited contract to two implementations: 42");
    }
}
