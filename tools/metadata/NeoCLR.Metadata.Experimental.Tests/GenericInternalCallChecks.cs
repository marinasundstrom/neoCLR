using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class GenericInternalCallChecks
{
    internal static void Write(string bundle, string directory)
    {
        Directory.CreateDirectory(directory);
        var runtime = AssemblyDefinition.ReadNativeAssembly(File.ReadAllBytes(Path.Combine(bundle, "System.Runtime.dll")));
        var core = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(Path.Combine(bundle, "Core.dll")), false).Identity;
        var graph = new AssemblyBuilder(new("NativeQueueServices", new(1, 0, 0, 0)), core);
        foreach (var name in new[] { "GetDefaultTaskQueue", "GetCurrentTaskQueue" })
            graph.AddFunction("neoCLR.Runtime", name, new(SignatureType.MethodParameter(0), [], ["T"])).SetInternalCall();
        var bytes = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        File.WriteAllBytes(Path.Combine(directory, "NativeQueueServices.dll"), bytes);
        var loaded = AssemblyDefinition.ReadNativeAssembly(bytes);
        foreach (var symbolic in new[] { false, true })
        {
            var name = symbolic ? "SymbolicQueueConsumer" : "ImportedQueueConsumer";
            var app = new AssemblyBuilder(new(name, new(1, 0, 0, 0)), core);
            var queue = app.ImportReference(runtime.MainModule.Types.Single(t => t.Namespace == "System.Concurrency.Tasks" && t.Name == "TaskQueue"), core);
            var entry = app.AddFunction("Main"); app.EntryPoint = entry;
            foreach (var definition in loaded.MainModule.Methods)
            {
                var signature = new MethodSignature(SignatureType.MethodParameter(0), [], ["T"]);
                var digest = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(bytes));
                var reference = symbolic ? app.CreateFunctionReference(loaded.Identity, core, digest,
                    "neoCLR.Runtime", definition.Name, signature, isInternalCall: true) : app.ImportReference(definition, core);
                if (symbolic)
                {
                    try
                    {
                        app.CreateFunctionReference(loaded.Identity, core, digest, "neoCLR.Runtime", definition.Name, signature);
                        throw new Exception("conflicting internal-call contract accepted");
                    }
                    catch (InvalidDataException) { }
                }
                entry.Call(reference.MakeGenericInstance(queue)); entry.Emit(OpCode.Pop);
            }
            entry.LoadConstant(42); entry.Return();
            File.WriteAllBytes(Path.Combine(directory, name + ".dll"), RuntimeAssemblyContainer.WriteLibraryBinary(app));
        }
        Console.WriteLine("PASS generic native internal-call authoring/import and symbol-only references; executable consumers written");
    }
}
