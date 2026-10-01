using System.Reflection;
using System.Diagnostics;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class DefaultValueChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = InstanceObjectChecks.Create();
        var t = SignatureType.MethodParameter(0);
        var empty = graph.AddFunction("Empty", new MethodSignature(t, [], ["T"]));
        empty.LoadDefault(t); empty.Return();
        var vectorType = SignatureType.ArrayOf(PrimitiveType.Int32);
        var vector = graph.AddFunction("EmptyVector", new MethodSignature(vectorType, []));
        vector.LoadDefault(vectorType); vector.Return();
        var main = graph.EntryPoint!; main.ClearBody();
        var skip = main.DefineLabel();
        main.Emit(OpCode.Ldc_Bool, true); main.Emit(OpCode.Brtrue, skip);
        main.LoadDefault(PrimitiveType.Int32); main.Emit(OpCode.Pop); main.MarkLabel(skip);
        main.Call(vector); main.Emit(OpCode.Pop);
        main.Call(empty.MakeGenericInstance(PrimitiveType.String)); main.Emit(OpCode.Pop);
        main.Call(empty.MakeGenericInstance(graph.Types[0])); main.Emit(OpCode.Pop);
        main.Call(empty.MakeGenericInstance(SignatureType.ArrayOf(PrimitiveType.Int32))); main.Emit(OpCode.Pop);
        main.Call(empty.MakeGenericInstance(PrimitiveType.Int64)); main.Emit(OpCode.Conv_I4);
        main.Call(empty.MakeGenericInstance(PrimitiveType.Int32)); main.Emit(OpCode.Add);
        main.LoadConstant(42); main.Emit(OpCode.Add); main.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("default generic execution");
        if (loaded.ManifestModule.GetMethods().Single(m => m.Name == "EmptyVector").Invoke(null, null) is not null) throw new Exception("vector default");
        var empty = loaded.ManifestModule.GetMethods().Single(m => m.Name == "Empty");
        foreach (var type in new[] { typeof(int), typeof(long), typeof(bool), typeof(string), typeof(int[]), loaded.GetType("Example.Order")! })
        {
            var expected = type.IsValueType ? Activator.CreateInstance(type) : null;
            if (!Equals(empty.MakeGenericMethod(type).Invoke(null, null), expected)) throw new Exception("default mismatch: " + type);
        }
        var main = graph.EntryPoint!; main.ClearBody();
        var local = main.DeclareLocal(PrimitiveType.Int32);
        main.Emit(OpCode.Ldloca, local); main.Emit(OpCode.Initobj, (SignatureType)PrimitiveType.Int32);
        main.LoadLocal(local); main.Return();
        if (!Equals(Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null), 0)) throw new Exception("raw local initialization");
        main.ClearBody(); main.LoadLocalAddress(local); main.InitializeObject(PrimitiveType.Int64); main.LoadLocal(local); main.Return();
        Reject(graph, "wrong init type");
        main.ClearBody(); main.LoadLocalAddress(local); main.Return(); Reject(graph, "address escape");
        main.ClearBody(); var skip = main.DefineLabel();
        main.Emit(OpCode.Ldc_Bool, true); main.Emit(OpCode.Brtrue, skip);
        main.LoadLocalAddress(local); main.InitializeObject(PrimitiveType.Int32); main.MarkLabel(skip); main.LoadLocal(local); main.Return();
        Reject(graph, "initialization must dominate load");
        main.ClearBody(); var count = main.Locals.Count;
        try { main.LoadDefault(PrimitiveType.Void); throw new Exception("Void default accepted"); } catch (ArgumentException) { }
        try { main.LoadDefault(SignatureType.MethodParameter(0)); throw new Exception("out of scope default accepted"); } catch (ArgumentException) { }
        if (main.Locals.Count != count) throw new Exception("failed default mutated locals");
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
        var path = Path.Combine(output, "DefaultValues.dll");
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
        Console.WriteLine("PASS default value binary: generic primitive/reference/vector initialization, verify/run 42");
    }

}
