using System.Reflection;
using System.Diagnostics;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericOwnerChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = GenericSignatureChecks.Create();
        var owner = graph.AddGenericType("Example", "Helpers", ["Element"]);
        var t = SignatureType.TypeParameter(0); var u = SignatureType.MethodParameter(0);
        var first = owner.AddMethod("First", new MethodSignature(t, [SignatureType.ArrayOf(t)]));
        first.LoadArgument(0); first.LoadConstant(0); first.LoadArrayElement(t); first.Return();
        var select = owner.AddMethod("Select", new MethodSignature(u, [t, u], ["Result"]));
        var local = select.DeclareLocal(u); select.LoadArgument(1); select.StoreLocal(local); select.LoadLocal(local); select.Return();
        var forward = owner.AddMethod("Forward", new MethodSignature(u, [t, u], ["Other"]));
        forward.LoadArgument(0); forward.LoadArgument(1); forward.Call(select.MakeConstructedReference([t], [u])); forward.Return();
        var identity = owner.AddMethod("Identity", new MethodSignature(t, [t]));
        identity.LoadArgument(0); identity.Call(graph.Functions.Single(m => m.Name == "Identity").MakeGenericInstance(t)); identity.Return();
        var main = graph.EntryPoint!; main.ClearBody();
        main.LoadConstant(2); main.Emit(OpCode.Ldc_I8, 5000000000L);
        main.Call(forward.MakeConstructedReference([PrimitiveType.Int32], [PrimitiveType.Int64])); main.Emit(OpCode.Pop);
        main.LoadConstant(42); main.Call(identity.MakeConstructedReference([PrimitiveType.Int32])); main.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create(); var image = graph.Write(); var loaded = Assembly.Load(image);
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("constructed-owner execution");
        var type = loaded.GetType("Example.Helpers`1")!.MakeGenericType(typeof(string));
        if (!Equals(type.GetMethod("First")!.Invoke(null, [new[] { "value" }]), "value")) throw new Exception("VAR array operand");
        if (!Equals(type.GetMethod("Select")!.MakeGenericMethod(typeof(int)).Invoke(null, ["ignored", 42]), 42)) throw new Exception("VAR/MVAR separation");
        var projected = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        var direct = AssemblyDefinition.ReadAssembly(image, false);
        foreach (var declaration in direct.MainModule.Types)
        {
            var copy = projected.MainModule.Types.Single(t => t.Name == declaration.Name);
            if (declaration.GenericArity != copy.GenericArity) throw new Exception("type arity projection");
            if (!declaration.Methods.Zip(copy.Methods).All(p => p.First.GetSignature().SequenceEqual(p.Second.GetSignature()))) throw new Exception("owner signature projection");
        }
        // Reject malformed declaration scopes before producing a CLI reference image.
        void RejectNative(Action<JsonNode> mutate)
        {
            var document = JsonNode.Parse(graph.WriteNativeAssembly())!;
            mutate(document);
            try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(document.ToJsonString())); }
            catch (InvalidDataException) { return; }
            throw new Exception("invalid native owner contract accepted");
        }
        JsonNode First(JsonNode document) => document["functions"]!.AsArray().Single(m => m!["origin"]!["name"]!.GetValue<string>() == "First" && m["owner"]?["Constructed"] is not null)!;
        RejectNative(document => First(document)["returns"] = new JsonObject { ["TypeParameter"] = 1 });
        RejectNative(document => First(document)["owner"]!["Constructed"]!["arguments"]![0] = new JsonObject { ["TypeParameter"] = 1 });
        RejectNative(document => First(document)["owner"]!["Constructed"]!["arguments"]![0] = "I32");
        RejectNative(document => document["types"]!.AsArray().Last()!["generic_parameters"] = new JsonArray("T", "U"));
        var method = graph.Types.Last().Methods[0];
        void Reject(Action action) { try { action(); } catch (ArgumentException) { return; } throw new Exception("invalid owner contract accepted"); }
        Reject(() => method.MakeConstructedReference([]));
        Reject(() => method.MakeConstructedReference([PrimitiveType.Void]));
        if (method.MakeConstructedReference([SignatureType.ArrayOf(PrimitiveType.Int32)]).Signature.ParameterTypes[0].ArrayElement?.ArrayElement?.Primitive != PrimitiveType.Int32) throw new Exception("nested owner substitution");
        Reject(() => graph.EntryPoint!.Call(method));
        Reject(() => graph.EntryPoint!.Call(method.MakeConstructedReference([SignatureType.TypeParameter(0)])));
        Reject(() => graph.AddFunction("BadOwner", new MethodSignature(SignatureType.TypeParameter(0), [])));
        Reject(() => graph.Types.Last().AddMethod("BadOwner", new MethodSignature(SignatureType.TypeParameter(1), [])));
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var graph = Create();
        var path = Path.Combine(output, "GenericOwners.dll");
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
        Console.WriteLine("PASS generic owner binary: VAR/MVAR, constructed calls and forwarding, verify/run 42");
    }

}
