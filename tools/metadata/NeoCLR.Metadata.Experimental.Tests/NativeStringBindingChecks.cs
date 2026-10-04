using System.Diagnostics;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeStringBindingChecks
{
    internal static async Task Run(string runtime, string seedPath, string corePath, string directory)
    {
        var reference = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(corePath), expectedExtended: false);
        var seed = NativeLibraryDefinition.ReadAssembly(File.ReadAllBytes(seedPath));
        var owner = reference.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == "String");
        var concat = owner.Methods.Single(m => m.Name == "Concat" && m.GetSignature().AsSpan().SequenceEqual(new byte[] { 0, 2, 0x0e, 0x0e, 0x0e }));
        foreach (var external in new[] { false, true })
        {
            var graphemeProjection = new AssemblyBuilder(new("GraphemeProjection", new Version(1, 0, 0, 0)), reference.Identity);
            graphemeProjection.BindNativeLibrary(reference, seed, reference.Identity);
            SignatureType character;
            if (external)
            {
                var externalChar = graphemeProjection.CreateValueTypeReference(new("Text", new Version(1, 0, 0, 0)), reference.Identity,
                    new string('a', 64), "System", "Char");
                graphemeProjection.SetNativeGrapheme(externalChar);
                character = externalChar;
            }
            else
            {
                var local = graphemeProjection.AddValueType("System", "Char");
                local.SetNativeGrapheme();
                character = local;
            }
            var indexer = graphemeProjection.ImportReference(owner.Methods.Single(m => m.Name == "get_Item"), reference.Identity);
            if (indexer.Signature.ReturnType != character)
                throw new Exception("explicit bootstrap signature did not select native grapheme owner");
        }
        AssemblyBuilder Create(NativeLibraryDefinition implementation)
        {
            var graph = new AssemblyBuilder(new("StaticStringConsumer", new Version(1, 0, 0, 0)), reference.Identity);
            graph.BindNativeLibrary(reference, implementation, reference.Identity);
            var method = graph.ImportReference(concat, reference.Identity);
            var main = graph.AddFunction("Main");
            var il = main.GetILGenerator();
            var character = graph.ImportReference(reference.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == "Char"), reference.Identity);
            var indexer = graph.ImportReference(owner.Methods.Single(m => m.Name == "get_Item"), reference.Identity);
            var text = graph.ImportReference(reference.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == "Char").Methods.Single(m => m.Name == "ToString"), reference.Identity);
            if (indexer.Signature.ReturnType != (SignatureType)character || !text.RequiresManagedReceiver)
                throw new Exception("character import contract");
            var echo = graph.AddFunction("EchoChar", new MethodSignature(character, [character]));
            echo.GetILGenerator().LoadArgument(0); echo.GetILGenerator().Return();
            var slot = il.DeclareLocal(character);
            il.Emit(OpCode.Ldstr, "é👩‍👩‍👧‍👦"); il.LoadConstant(1); il.Call(indexer); il.Call(echo); il.StoreLocal(slot);
            il.LoadLocalAddress(slot); il.Call(text);
            il.Emit(OpCode.Ldstr, "👩‍👩‍👧‍👦");
            il.Call(graph.ImportReference(owner.Methods.Single(m => m.Name == "Equals"), reference.Identity));
            var correct = il.DefineLabel(); il.Emit(OpCode.Brtrue, correct); il.Fail("grapheme text changed"); il.MarkLabel(correct);
            var valid = il.DefineLabel();
            il.LoadConstant(42); il.Box(PrimitiveType.Int32); il.IsInstance(character); il.IsNull(); il.Emit(OpCode.Brtrue, valid);
            il.Fail("boxed Int32 must not satisfy the intrinsic Char test");
            il.MarkLabel(valid);
            il.Emit(OpCode.Ldstr, "union "); il.Emit(OpCode.Ldstr, "payload"); il.Call(method); il.WriteConsoleLine();
            il.LoadConstant(42); il.Return(); graph.EntryPoint = main;
            // The static mapping must not admit intrinsic String as an ordinary class.
            Reject(() => graph.ImportReference(owner, reference.Identity));
            return graph;
        }
        var graph = Create(seed);
        Directory.CreateDirectory(directory);
        var path = Path.Combine(directory, "Consumer.dll");
        var image = RuntimeAssemblyContainer.WriteBinary(graph);
        File.WriteAllBytes(path, image);
        var projection = RuntimeAssemblyContainer.ReadCliProjection(image);
        if (!projection.MainModule.Functions.Single(m => m.Name == "EchoChar").GetSignature().SequenceEqual(new byte[] { 0, 1, 3, 3 }))
            throw new Exception("native Char projection is not canonical CLI Char");
        Reject(() => NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(new("WrongCore", new Version(1, 0, 0, 0))));
        foreach (var invalid in new[] { "missing", "namespace", "category" })
        {
            var json = JsonNode.Parse(graph.WriteNativeAssembly())!;
            var aliases = json["assemblies"]![0]!["native_type_bindings"]!.AsArray();
            var characterAlias = aliases.Single(a => a!["native_name"]!.GetValue<string>() == "System.Char")!;
            if (invalid == "missing") aliases.Remove(characterAlias);
            if (invalid == "namespace") characterAlias["namespace"] = "Wrong";
            if (invalid == "category") characterAlias["value_type"] = false;
            Reject(() => NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(json.ToJsonString())));
        }
        var native = AssemblyDefinition.ReadNativeAssembly(image);
        var consumer = new AssemblyBuilder(new("CharReimport", new Version(1, 0, 0, 0)), reference.Identity);
        consumer.BindNativeLibrary(reference, seed, reference.Identity);
        var imported = consumer.ImportReference(native.MainModule.Functions.Single(m => m.Name == "EchoChar"), reference.Identity, new CoreResolver(reference));
        if (imported.Signature.ReturnType.ImportedType?.Name != "Char") throw new Exception("native character round trip");

        foreach (var change in new[] { "owner", "result", "missing", "module" })
        {
            var json = JsonNode.Parse(seed.Declarations.GetRawText())!;
            var method = json["functions"]!.AsArray().Single(f => f!["name"]!.GetValue<string>() == "System.String.Concat" && f["parameters"]!.ToJsonString() == "[\"String\",\"String\"]");
            if (change == "owner") method!["owner"] = new JsonObject { ["Named"] = "System.String" };
            if (change == "result") method!["returns"] = "Int32";
            if (change == "missing") method!["name"] = "System.String.Other";
            if (change == "module") json["name"] = "OtherCore";
            var changed = NativeLibraryDefinition.ReadAssembly(NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(json.ToJsonString())));
            Reject(() => Create(changed));
        }
        foreach (var (name, key, value) in new[] {
            ("System.Char.ToString", "receiver_byref", "false"),
            ("System.Char.ToString", "owner", "{\"Named\":\"System.Char\"}"),
            ("System.Char.ToString", "is_virtual", "true"),
            ("System.String.get_Item", "returns", "\"Int32\"") })
        {
            var json = JsonNode.Parse(seed.Declarations.GetRawText())!;
            json["functions"]!.AsArray().Single(f => f!["name"]!.GetValue<string>() == name)![key] = JsonNode.Parse(value);
            Reject(() => Create(NativeLibraryDefinition.ReadAssembly(NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(json.ToJsonString())))));
        }
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { command, path, "--system", seedPath }) start.ArgumentList.Add(argument);
            using var process = Process.Start(start)!;
            var output = process.StandardOutput.ReadToEndAsync(); var error = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(60));
            try { await process.WaitForExitAsync(timeout.Token); } catch { process.Kill(true); throw; }
            var stdout = await output; var stderr = await error;
            if (process.ExitCode != (command == "run" ? 42 : 0) || stderr.Length != 0 || command == "run" && stdout.Replace("\r\n", "\n") != "union payload\n")
                throw new Exception(stdout + stderr);
        }
        Console.WriteLine("PASS exact core static String binding, canonical primitive owner and runtime concatenation");
    }
    private sealed class CoreResolver(AssemblyDefinition core) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => core.Identity.Equals(identity) ? core : null;
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("expected invalid binding rejection");
    }
}
