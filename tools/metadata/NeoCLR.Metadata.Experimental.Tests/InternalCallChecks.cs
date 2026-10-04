using System.Diagnostics;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class InternalCallChecks
{
    private static AssemblyBuilder Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("InternalCalls", new(1, 0, 0, 0)), core);
        var count = graph.AddFunction("neoCLR.Runtime", "TypeArgumentCount", new(PrimitiveType.Int32, [PrimitiveType.RuntimeTypeHandle]));
        count.SetInternalCall();
        var same = new MethodDefinition("TypeEquals", new(PrimitiveType.Boolean, [PrimitiveType.RuntimeTypeHandle, PrimitiveType.RuntimeTypeHandle]), @namespace: "neoCLR.Runtime");
        same.SetInternalCall();
        graph.Definition.MainModule.Functions.Add(same);
        var main = graph.AddFunction("Main");
        graph.EntryPoint = main;
        var il = main.GetILGenerator();
        il.LoadTypeToken(graph.AddClass("Example", "Item"));
        il.Call(count);
        il.LoadConstant(42);
        il.Add();
        il.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        foreach (var image in new[] { AssemblyDefinition.ReadAssembly(graph.Write(), false),
            AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary)) })
        {
            var services = image.MainModule.Methods.Where(m => m.ImplementationAttributes == 0x1000).ToArray();
            if (services.Length != 2 || services.Any(m => !m.IsStatic || (m.Attributes & 0x400) != 0))
                throw new Exception("internal call flags/definition-builder parity lost");
            try { services[0].SetInternalCall(); throw new Exception("loaded declaration mutated"); }
            catch (InvalidOperationException) { }
        }
        foreach (var invalidFlags in new[] { 1, 0x1001 })
        {
            var invalid = JsonNode.Parse(graph.WriteNativeAssembly())!;
            invalid["functions"]![0]!["impl_flags"] = invalidFlags;
            RejectWrite(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(invalid.ToJsonString())));
        }
        var withBody = JsonNode.Parse(graph.WriteNativeAssembly())!;
        withBody["functions"]![0]!["body"] = JsonNode.Parse("[{\"op\":\"ret\"}]");
        RejectWrite(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(withBody.ToJsonString())));
        var projected = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        if (projected.MainModule.Methods.Count(m => m.ImplementationAttributes == 0x1000) != 2)
            throw new Exception("reference projection lost internal-call flags");
        var localContract = Create();
        var descriptor = localContract.AddClass("Example", "Descriptor");
        localContract.AddFunction("runtime", "Describe", new MethodSignature(descriptor, [PrimitiveType.RuntimeTypeHandle])).SetInternalCall();
        var localImage = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(localContract.WriteNativeAssembly(), localContract.CoreLibrary));
        var describe = localImage.MainModule.Methods.Single(m => m.Name == "Describe");
        if (describe.ImplementationAttributes != 0x1000 || !describe.TryGetSignature(out var localSignature) || localSignature!.ReturnType.Primitive is not null)
            throw new Exception("source-owned nominal service signature lost");
        var generic = graph.AddFunction("Generic", new(PrimitiveType.Void, [], ["T"]));
        Reject(() => generic.SetInternalCall());
        var owned = graph.AddClass("Example", "Owner").AddMethod("Service", new(PrimitiveType.Void, []));
        Reject(() => owned.SetInternalCall());
        var nonempty = graph.AddFunction("Nonempty"); nonempty.LoadConstant(0);
        Reject(() => nonempty.SetInternalCall());
        var modified = Create();
        modified.Functions[0].GetILGenerator().LoadConstant(1);
        RejectWrite(() => modified.WriteNativeAssembly());
        RejectWrite(() => modified.Write());
        var entry = Create();
        var bodyless = entry.AddFunction("runtime", "Entry", new(PrimitiveType.Int32, []));
        bodyless.SetInternalCall();
        var invalidEntry = JsonNode.Parse(entry.WriteNativeAssembly())!;
        invalidEntry["entry"] = "runtime.Entry";
        RejectWrite(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(invalidEntry.ToJsonString())));
        entry.EntryPoint = bodyless;
        RejectWrite(() => entry.WriteNativeAssembly());
        RejectWrite(() => entry.Write());
    }

    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidOperationException) { return; }
        throw new Exception("invalid internal-call authoring accepted");
    }
    private static void RejectWrite(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid internal-call graph published");
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run();
        Directory.CreateDirectory(directory);
        var graph = Create();
        var path = Path.Combine(directory, "InternalCalls.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        var seed = Path.Combine(directory, "EmptySystem.neoil");
        File.WriteAllText(seed, ".module System\n");
        graph.EntryPoint = null;
        var libraryPath = Path.Combine(directory, "Library.dll");
        File.WriteAllBytes(libraryPath, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        var imported = AssemblyDefinition.ReadNativeAssembly(File.ReadAllBytes(libraryPath));
        var consumer = new AssemblyBuilder(new("InternalCallConsumer", new(1, 0, 0, 0)), graph.CoreLibrary);
        var count = consumer.ImportReference(imported.MainModule.Methods.Single(m => m.Name == "TypeArgumentCount"), graph.CoreLibrary);
        var entry = consumer.AddFunction("Main"); consumer.EntryPoint = entry;
        entry.GetILGenerator().LoadTypeToken(consumer.AddClass("Consumer", "Local"));
        entry.GetILGenerator().Call(count); entry.LoadConstant(42); entry.Add(); entry.Return();
        var consumerPath = Path.Combine(directory, "Consumer.dll");
        File.WriteAllBytes(consumerPath, RuntimeAssemblyContainer.WriteBinary(consumer.WriteNativeAssembly(), consumer.CoreLibrary));
        var unknown = Create();
        unknown.AddFunction("neoCLR.Runtime", "MissingService", new(PrimitiveType.Int32, [])).SetInternalCall();
        var unknownPath = Path.Combine(directory, "Unknown.dll");
        File.WriteAllBytes(unknownPath, RuntimeAssemblyContainer.WriteBinary(unknown.WriteNativeAssembly(), unknown.CoreLibrary));
        foreach (var target in new[] { path, consumerPath, unknownPath })
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(target);
            if (target == consumerPath) { start.ArgumentList.Add("--module"); start.ArgumentList.Add(libraryPath); }
            start.ArgumentList.Add("--system"); start.ArgumentList.Add(seed);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync();
            var output = await stdout + await stderr;
            var expected = target == unknownPath ? 1 : command == "verify" ? 0 : 42;
            if (process.ExitCode != expected || target == unknownPath && !output.Contains("no runtime binding"))
                throw new Exception(command + ": " + output);
        }
    }
}
