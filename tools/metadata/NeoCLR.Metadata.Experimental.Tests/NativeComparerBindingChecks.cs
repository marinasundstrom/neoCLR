using System.Diagnostics;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeComparerBindingChecks
{
    internal static async Task Run(string runtime, string seedPath, string corePath, string directory)
    {
        var reference = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(corePath), expectedExtended: false);
        var seed = NativeLibraryDefinition.ReadAssembly(File.ReadAllBytes(seedPath));
        MethodDefinition Method(string type, string name) => reference.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == type).Methods.Single(m => m.Name == name);
        AssemblyBuilder Create(NativeLibraryDefinition implementation)
        {
            var graph = new AssemblyBuilder(new("ComparerBinding", new Version(1, 0, 0, 0)), reference.Identity);
            graph.BindNativeLibrary(reference, implementation, reference.Identity);
            var equals = graph.ImportReference(Method("String", "Equals"), reference.Identity);
            var compare = graph.ImportReference(Method("Int32", "CompareTo"), reference.Identity);
            var hash = graph.ImportReference(Method("Object", "GetHashCode"), reference.Identity);
            if (equals.RequiresManagedReceiver || !compare.RequiresManagedReceiver || !hash.RequiresVirtualDispatch)
                throw new Exception("primitive or Object receiver contract lost");
            var main = graph.AddFunction("Main"); graph.EntryPoint = main;
            var il = main.GetILGenerator();
            var failed = il.DefineLabel();
            il.Emit(OpCode.Ldstr, "café"); il.Emit(OpCode.Ldstr, "café"); il.Call(equals); il.Emit(OpCode.Brfalse, failed);
            var value = il.DeclareLocal(PrimitiveType.Int32);
            il.LoadConstant(int.MinValue); il.StoreLocal(value); il.LoadLocalAddress(value);
            il.LoadConstant(int.MaxValue); il.Call(compare); il.LoadConstant(-1); il.Emit(OpCode.Ceq); il.Emit(OpCode.Brfalse, failed);
            il.Emit(OpCode.Ldstr, "café"); il.CastReference(graph.CoreObjectType); il.CallVirtual(hash);
            il.Emit(OpCode.Ldstr, "café"); il.CastReference(graph.CoreObjectType); il.CallVirtual(hash); il.Emit(OpCode.Ceq); il.Emit(OpCode.Brfalse, failed);
            il.LoadConstant(42); il.Return(); il.MarkLabel(failed); il.LoadConstant(1); il.Return();
            Reject(() => graph.ImportReference(reference.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == "String"), reference.Identity));
            return graph;
        }
        Directory.CreateDirectory(directory);
        var path = Path.Combine(directory, "ComparerBinding.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(Create(seed)));
        foreach (var (name, key, value) in new[] {
            ("System.String.Equals", "receiver_byref", "true"),
            ("System.String.Equals", "is_virtual", "true"),
            ("System.Int32.CompareTo", "receiver_byref", "false"),
            ("System.Object.GetHashCode", "is_virtual", "false"),
            ("System.Object.GetHashCode", "returns", "\"Int64\"") })
        {
            var json = JsonNode.Parse(seed.Declarations.GetRawText())!;
            json["functions"]!.AsArray().Single(f => f!["name"]!.GetValue<string>() == name)![key] = JsonNode.Parse(value);
            Reject(() => Create(NativeLibraryDefinition.ReadAssembly(NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(json.ToJsonString())))));
        }
        var unbound = new AssemblyBuilder(new("UnboundHash", new Version(1, 0, 0, 0)), reference.Identity);
        var unboundHash = unbound.ImportReference(Method("Object", "GetHashCode"), reference.Identity);
        var body = unbound.AddFunction("Hash", new(PrimitiveType.Int32, [unbound.CoreObjectType])).GetILGenerator();
        body.LoadArgument(0); body.CallVirtual(unboundHash); body.Return();
        Reject(() => unbound.WriteNativeAssembly());
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { command, path, "--system", seedPath }) start.ArgumentList.Add(argument);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(60));
            try { await process.WaitForExitAsync(timeout.Token); } catch { process.Kill(true); throw; }
            if (process.ExitCode != (command == "run" ? 42 : 0)) throw new Exception(await stdout + await stderr);
        }
        Console.WriteLine("PASS primitive receiver and Object hash bootstrap contracts and execution");
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("expected invalid binding rejection");
    }
}
