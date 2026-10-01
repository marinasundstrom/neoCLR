using System.Diagnostics;
using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericClassChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = GenericOwnerChecks.Create();
        var box = graph.AddGenericClass("Example", "Box", ["Element"]);
        var t = SignatureType.TypeParameter(0);
        var value = box.AddField("value", t);
        var ctor = box.AddConstructor(new MethodSignature(PrimitiveType.Void, [t]));
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(value); ctor.Return();
        var get = box.AddInstanceMethod("Get", new MethodSignature(t, []));
        get.LoadArgument(0); get.LoadField(value); get.Return();
        var set = box.AddInstanceMethod("Set", new MethodSignature(PrimitiveType.Void, [t]));
        set.LoadArgument(0); set.LoadArgument(1); set.StoreField(value); set.Return();
        var echo = box.AddInstanceMethod("Echo", new MethodSignature(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["Other"]));
        echo.LoadArgument(1); echo.Return();
        var main = graph.EntryPoint!; main.ClearBody();
        var local = main.DeclareLocal(box.MakeGenericInstance(PrimitiveType.Int32));
        main.LoadConstant(1); main.NewObject(ctor.MakeConstructedReference([PrimitiveType.Int32])); main.StoreLocal(local);
        main.LoadLocal(local); main.LoadConstant(42); main.Call(set.MakeConstructedReference([PrimitiveType.Int32]));
        main.LoadLocal(local); main.Emit(OpCode.Ldc_I8, 5000000000L); main.Call(echo.MakeConstructedReference([PrimitiveType.Int32], [PrimitiveType.Int64])); main.Emit(OpCode.Pop);
        main.LoadLocal(local); main.Call(get.MakeConstructedReference([PrimitiveType.Int32])); main.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("generic class execution");
        var box = loaded.GetType("Example.Box`1")!.MakeGenericType(typeof(string));
        var instance = Activator.CreateInstance(box, "before");
        box.GetMethod("Set")!.Invoke(instance, ["after"]);
        if (!Equals(box.GetMethod("Get")!.Invoke(instance, null), "after")) throw new Exception("generic reference storage");
        var projected = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        var direct = AssemblyDefinition.ReadAssembly(graph.Write(), false);
        var field = projected.MainModule.Types.Single(t => t.Name == "Box`1").Fields.Single();
        var original = direct.MainModule.Types.Single(t => t.Name == "Box`1").Fields.Single();
        if (!field.GetSignature().SequenceEqual(original.GetSignature())) throw new Exception("generic field projection");
        SignatureType first = graph.Types.Last().MakeGenericInstance(PrimitiveType.Int32);
        SignatureType second = graph.Types.Last().MakeGenericInstance(PrimitiveType.Int32);
        if (first != second || first.GetHashCode() != second.GetHashCode()) throw new Exception("constructed type identity");
        void Reject(Action action)
        {
            try { action(); } catch (ArgumentException) { return; }
            throw new Exception("invalid generic class contract accepted");
        }
        var definition = graph.Types.Last();
        Reject(() => definition.MakeGenericInstance());
        Reject(() => definition.MakeGenericInstance(PrimitiveType.Void));
        Reject(() => { SignatureType bare = definition; });
        Reject(() => definition.AddField("bad", SignatureType.MethodParameter(0)));
        Reject(() => definition.AddField("bad", SignatureType.TypeParameter(1)));
        Reject(() => graph.EntryPoint!.LoadField(definition.Fields[0]));
        Reject(() => graph.EntryPoint!.DeclareLocal(definition.MakeGenericInstance(SignatureType.TypeParameter(0))));
        Reject(() => graph.EntryPoint!.Call(definition.Methods[0].MakeConstructedReference([PrimitiveType.Int32])));
        var foreign = Create();
        Reject(() => graph.EntryPoint!.DeclareLocal(foreign.Types.Last().MakeGenericInstance(PrimitiveType.Int32)));
        SignatureType nested = PrimitiveType.Int32;
        for (int i = 0; i < 16; i++) nested = definition.MakeGenericInstance(nested);
        Reject(() => definition.MakeGenericInstance(nested));
        var wrong = Create(); var body = wrong.EntryPoint!; body.ClearBody();
        body.LoadConstant(42); body.NewObject(wrong.Types.Last().Methods[0].MakeConstructedReference([PrimitiveType.Int32]));
        body.Call(wrong.Types.Last().Methods[1].MakeConstructedReference([PrimitiveType.String])); body.Emit(OpCode.Pop); body.LoadConstant(42); body.Return();
        try { wrong.Write(); throw new Exception("wrong constructed receiver accepted"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var graph = Create();
        var path = Path.Combine(output, "GenericClasses.dll");
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
        Console.WriteLine("PASS generic class binary: constructors, fields and methods, verify/run 42");
    }
}
