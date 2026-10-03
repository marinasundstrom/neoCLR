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
        AssemblyBuilder Create(NativeLibraryDefinition implementation)
        {
            var graph = new AssemblyBuilder(new("StaticStringConsumer", new Version(1, 0, 0, 0)), reference.Identity);
            graph.BindNativeLibrary(reference, implementation, reference.Identity);
            var method = graph.ImportReference(concat, reference.Identity);
            var main = graph.AddFunction("Main");
            var il = main.GetILGenerator();
            var character = graph.ImportReference(reference.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == "Char"), reference.Identity);
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
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph));
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
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("expected invalid binding rejection");
    }
}
