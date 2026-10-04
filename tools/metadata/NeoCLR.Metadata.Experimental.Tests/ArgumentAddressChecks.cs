using System.Diagnostics;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ArgumentAddressChecks
{
    private static AssemblyBuilder Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("ArgumentAddresses", new Version(1, 0, 0, 0)), core);
        var update = graph.AddFunction("Update", new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        var body = update.GetILGenerator();
        body.LoadArgumentAddress(0); body.LoadConstant(40); body.Emit(OpCode.Stobj, (SignatureType)PrimitiveType.Int32);
        body.LoadArgument(0); body.Return();
        var type = graph.AddClass("Example", "Counter");
        var constructor = type.AddConstructor(Array.Empty<PrimitiveType>());
        constructor.GetILGenerator().Return();
        var instance = type.AddInstanceMethod("Update", new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        var instanceBody = instance.GetILGenerator();
        instanceBody.Emit(OpCode.Ldarga, 1); instanceBody.LoadConstant(2); instanceBody.Emit(OpCode.Stobj, (SignatureType)PrimitiveType.Int32);
        instanceBody.LoadArgument(1); instanceBody.Return();
        var parameter = SignatureType.MethodParameter(0);
        var identity = graph.AddFunction("Identity", new(parameter, [parameter], ["T"]));
        var identityBody = identity.GetILGenerator();
        identityBody.LoadArgumentAddress(0); identityBody.Emit(OpCode.Ldobj, parameter); identityBody.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var il = main.GetILGenerator();
        il.LoadConstant(7); il.Call(update);
        il.NewObject(constructor); il.LoadConstant(9); il.Call(instance); il.Add(); il.Call(identity.MakeGenericInstance(PrimitiveType.Int32)); il.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var context = new AssemblyLoadContext("argument-address-checks", isCollectible: true);
        try
        {
            using var stream = new MemoryStream(graph.Write());
            var assembly = context.LoadFromStream(stream);
            if (!Equals(42, assembly.EntryPoint!.Invoke(null, null))) throw new Exception("argument mutation did not persist");
        }
        finally { context.Unload(); }
        foreach (var kind in new[] { "negative", "outside", "receiver", "byref", "wrong-type" })
        {
            var invalid = Create();
            var method = kind == "receiver"
                ? invalid.AddClass("Example", "Bad").AddInstanceMethod("Bad", new(PrimitiveType.Void, []))
                : invalid.AddFunction("Bad", new(PrimitiveType.Void, [kind == "byref" ? SignatureType.ByReference(PrimitiveType.Int32) : PrimitiveType.Int32]));
            var il = method.GetILGenerator();
            if (kind != "wrong-type") il.Return(); // Validate invalid operands even when unreachable.
            il.Emit(OpCode.Ldarga, kind == "negative" ? -1 : kind == "outside" ? 1 : 0);
            if (kind == "wrong-type") { il.Emit(OpCode.Ldc_I8, 42L); il.Emit(OpCode.Stobj, (SignatureType)PrimitiveType.Int64); il.Return(); }
            Reject(() => invalid.Write()); Reject(() => invalid.WriteNativeAssembly());
        }
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        var path = Path.Combine(directory, "ArgumentAddresses.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(Create()));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var arg in new[] { command, path }) start.ArgumentList.Add(arg);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(60));
            try { await process.WaitForExitAsync(timeout.Token); } catch { process.Kill(true); throw; }
            if (process.ExitCode != (command == "run" ? 42 : 0)) throw new Exception(await stdout + await stderr);
        }
        Console.WriteLine("PASS CLI/native by-value argument address mutation and receiver offsets");
    }

    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("expected argument-address rejection");
    }
}
