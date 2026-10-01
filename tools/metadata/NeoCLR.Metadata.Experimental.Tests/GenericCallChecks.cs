using System.Reflection;
using System.Diagnostics;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericCallChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = GenericSignatureChecks.Create();
        var t = SignatureType.MethodParameter(0);
        var identity = graph.Functions.Single(m => m.Name == "Identity");
        var forward = graph.AddFunction("Forward", new MethodSignature(t, [t], ["U"]));
        forward.LoadArgument(0); forward.Emit(OpCode.Call, identity.MakeGenericInstance(t)); forward.Return();
        var single = graph.AddFunction("Single", new MethodSignature(SignatureType.ArrayOf(t), [t], ["T"]));
        var values = single.DeclareLocal(SignatureType.ArrayOf(t));
        single.LoadConstant(1); single.NewArray(t); single.StoreLocal(values);
        single.LoadLocal(values); single.LoadConstant(0); single.LoadArgument(0); single.StoreArrayElement(t);
        single.LoadLocal(values); single.Return();
        var first = graph.Types.Single(x => x.Name == "Generic").Methods.Single(m => m.Name == "First");
        var entry = graph.EntryPoint!; entry.ClearBody();
        entry.Emit(OpCode.Ldc_I8, 5000000000L); entry.Call(forward.MakeGenericInstance(PrimitiveType.Int64)); entry.Emit(OpCode.Pop);
        entry.Emit(OpCode.Ldc_Bool, true); entry.Call(forward.MakeGenericInstance(PrimitiveType.Boolean)); entry.Emit(OpCode.Pop);
        var order = graph.Types[0];
        entry.LoadConstant(42); entry.Emit(OpCode.Ldc_Bool, true); entry.NewObject(order.Methods[0]);
        entry.Call(forward.MakeGenericInstance(order)); entry.Call(single.MakeGenericInstance(order));
        entry.Call(first.MakeGenericInstance(order)); entry.Call(order.Methods.Single(m => m.Name == "GetNumber"));
        entry.Call(forward.MakeGenericInstance(PrimitiveType.Int32)); entry.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        if (!Equals(Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null), 42)) throw new Exception("generic execution");
        NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary);
        var identity = graph.Functions.Single(m => m.Name == "Identity");
        void Reject(Action action) { try { action(); } catch (ArgumentException) { return; } throw new Exception("invalid instantiation accepted"); }
        Reject(() => identity.MakeGenericInstance());
        Reject(() => identity.MakeGenericInstance(PrimitiveType.Void));
        Reject(() => identity.MakeGenericInstance(GenericSignatureChecks.Create().Types[0]));
        Reject(() => graph.EntryPoint!.Call(identity.MakeGenericInstance(SignatureType.MethodParameter(0))));
        Reject(() => graph.EntryPoint!.Emit(OpCode.Add, identity.MakeGenericInstance(PrimitiveType.Int32)));
        SignatureType[] arguments = [PrimitiveType.Int32];
        var immutable = identity.MakeGenericInstance(arguments);
        arguments[0] = PrimitiveType.Boolean;
        if (immutable.TypeArguments[0].Primitive != PrimitiveType.Int32 || immutable.Signature.ReturnType.Primitive != PrimitiveType.Int32)
            throw new Exception("instantiation retained mutable argument array");
        var single = graph.Functions.Single(m => m.Name == "Single");
        Reject(() => single.MakeGenericInstance(SignatureType.ArrayOf(PrimitiveType.Int32)));
        if (!Equals(Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null), 42)) throw new Exception("rejection mutated body");
        var entry = graph.EntryPoint!; entry.ClearBody();
        entry.LoadConstant(1); entry.Call(identity.MakeGenericInstance(PrimitiveType.Boolean)); entry.Emit(OpCode.Pop); entry.LoadConstant(42); entry.Return();
        try { graph.Write(); throw new Exception("wrong generic stack argument accepted"); } catch (InvalidDataException) { }
        try { graph.WriteNativeAssembly(); throw new Exception("wrong native generic stack argument accepted"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var graph = Create();
        var path = Path.Combine(output, "GenericCalls.dll");
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
        Console.WriteLine("PASS generic binary: instantiated and forwarded functions, array factory/access and nominal identity, verify/run 42");
    }

}
