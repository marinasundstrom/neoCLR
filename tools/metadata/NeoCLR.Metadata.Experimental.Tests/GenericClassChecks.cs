using System.Diagnostics;
using System.Reflection;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericClassChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = GenericOwnerChecks.Create();
        var helpers = graph.Types.Last();
        var empty = helpers.AddMethod("get_Empty", new MethodSignature(SignatureType.TypeParameter(0), []));
        empty.LoadDefault(SignatureType.TypeParameter(0)); empty.Return();
        helpers.AddProperty("Empty", SignatureType.TypeParameter(0), empty);
        var box = graph.AddGenericClass("Example", "Box", ["Element"]);
        var t = SignatureType.TypeParameter(0);
        var value = box.AddField("value", t);
        var ctor = box.AddConstructor(new MethodSignature(PrimitiveType.Void, [t]));
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(value); ctor.Return();
        var get = box.AddInstanceMethod("Get", new MethodSignature(t, []));
        get.LoadArgument(0); get.LoadField(value); get.Return();
        var set = box.AddInstanceMethod("Set", new MethodSignature(PrimitiveType.Void, [t]));
        set.LoadArgument(0); set.LoadArgument(1); set.StoreField(value); set.Return();
        box.AddProperty("Value", t, get, set);
        var getIndex = box.AddInstanceMethod("get_Item", new MethodSignature(t, [PrimitiveType.Int32]));
        getIndex.LoadArgument(0); getIndex.LoadField(value); getIndex.Return();
        var setIndex = box.AddInstanceMethod("set_Item", new MethodSignature(PrimitiveType.Void, [PrimitiveType.Int32, t]));
        setIndex.LoadArgument(0); setIndex.LoadArgument(2); setIndex.StoreField(value); setIndex.Return();
        box.AddProperty("Item", t, getIndex, setIndex);
        var byValue = box.AddInstanceMethod("get_ByValue", new MethodSignature(t, [t]));
        byValue.LoadArgument(0); byValue.LoadField(value); byValue.Return();
        box.AddProperty("ByValue", t, byValue);
        var echo = box.AddInstanceMethod("Echo", new MethodSignature(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["Other"]));
        echo.LoadArgument(1); echo.Return();
        var main = graph.EntryPoint!; main.ClearBody();
        var local = main.DeclareLocal(box.MakeGenericInstance(PrimitiveType.Int32));
        main.LoadConstant(1); main.NewObject(ctor.MakeConstructedReference([PrimitiveType.Int32])); main.StoreLocal(local);
        main.LoadLocal(local); main.LoadConstant(42); main.Call(set.MakeConstructedReference([PrimitiveType.Int32]));
        main.LoadLocal(local); main.Emit(OpCode.Ldc_I8, 5000000000L); main.Call(echo.MakeConstructedReference([PrimitiveType.Int32], [PrimitiveType.Int64])); main.Emit(OpCode.Pop);
        main.LoadLocal(local); main.LoadConstant(0); main.Call(getIndex.MakeConstructedReference([PrimitiveType.Int32])); main.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("generic class execution");
        if (!Equals(loaded.GetType("Example.Helpers`1")!.MakeGenericType(typeof(int)).GetProperty("Empty")!.GetValue(null), 0))
            throw new Exception("static generic property");
        var box = loaded.GetType("Example.Box`1")!.MakeGenericType(typeof(string));
        var instance = Activator.CreateInstance(box, "before");
        box.GetMethod("Set")!.Invoke(instance, ["after"]);
        if (!Equals(box.GetMethod("Get")!.Invoke(instance, null), "after")) throw new Exception("generic reference storage");
        var projected = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        var direct = AssemblyDefinition.ReadAssembly(graph.Write(), false);
        var field = projected.MainModule.Types.Single(t => t.Name == "Box`1").Fields.Single();
        var original = direct.MainModule.Types.Single(t => t.Name == "Box`1").Fields.Single();
        if (!field.GetSignature().SequenceEqual(original.GetSignature())) throw new Exception("generic field projection");
        var property = box.GetProperty("Value")!;
        property.SetValue(instance, "property");
        if (!Equals(property.GetValue(instance), "property")) throw new Exception("generic property execution");
        var indexed = box.GetProperty("Item")!;
        indexed.SetValue(instance, "indexed", [0]);
        if (!Equals(indexed.GetValue(instance, [0]), "indexed")) throw new Exception("generic indexer execution");
        var projectedOwner = projected.MainModule.Types.Single(t => t.Name == "Box`1");
        var directOwner = direct.MainModule.Types.Single(t => t.Name == "Box`1");
        if (projectedOwner.Properties.Count != 3 || !projectedOwner.Properties.Zip(directOwner.Properties).All(p => p.First.GetSignature().SequenceEqual(p.Second.GetSignature())))
            throw new Exception("generic property projection");
        if (!Equals(box.GetProperty("ByValue")!.GetValue(instance, ["key"]), "indexed")) throw new Exception("generic index parameter");
        void RejectProperty(Action<JsonNode> mutate)
        {
            var document = JsonNode.Parse(graph.WriteNativeAssembly())!;
            var propertyRow = document["types"]!.AsArray().Last()!["properties"]![0]!;
            mutate(propertyRow);
            try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(document.ToJsonString())); }
            catch (InvalidDataException) { return; }
            throw new Exception("invalid generic property metadata accepted");
        }
        RejectProperty(p => p["ty"] = new JsonObject { ["TypeParameter"] = 1 });
        RejectProperty(p => p["getter"]!["owner"]!["Constructed"]!["arguments"]![0] = new JsonObject { ["TypeParameter"] = 1 });
        RejectProperty(p => p["getter"]!["owner"]!["Constructed"]!["arguments"]![0] = "Int32");
        RejectProperty(p => p["getter"]!["owner"]!["Constructed"]!["arguments"] = new JsonArray());
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
        Reject(() => definition.AddProperty("bad", SignatureType.MethodParameter(0), definition.Methods[1]));
        Reject(() => definition.AddProperty("bad", SignatureType.TypeParameter(1), definition.Methods[1]));
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
