using System.Diagnostics;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class SourceModuleInfoChecks
{
    private static AssemblyBuilder Create(bool wrongLayout = false)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("SourceModuleInfo", new(1, 0, 0, 0)), core);
        var contract = graph.AddInterface("System.Introspection", "ModuleInfo");
        var name = contract.AddInterfaceMethod("get_Name", new(PrimitiveType.String, []));
        contract.AddProperty("Name", PrimitiveType.String, name, null);
        var provider = graph.AddClass("System.Introspection", "RuntimeModuleInfo", TypeVisibility.Internal);
        provider.AddInterfaceImplementation(contract);
        provider.AddField(wrongLayout ? "WrongIdentity" : "StoredIdentity", PrimitiveType.String, FieldVisibility.Private);
        var storedName = provider.AddField("StoredName", PrimitiveType.String, FieldVisibility.Private);
        var getter = provider.AddInstanceMethod("get_Name", new(PrimitiveType.String, []));
        getter.GetILGenerator().LoadArgument(0);
        getter.GetILGenerator().LoadField(storedName);
        getter.Return();
        provider.AddProperty("Name", PrimitiveType.String, getter, null);
        var service = graph.AddFunction("neoCLR.Runtime", "TypeModule", new MethodSignature(contract, [PrimitiveType.RuntimeTypeHandle]), MethodVisibility.Internal);
        service.SetInternalCall();
        var compare = graph.AddFunction("neoCLR.Runtime", "StringCompareOrdinal", new(PrimitiveType.Int32, [PrimitiveType.String, PrimitiveType.String]), MethodVisibility.Internal);
        compare.SetInternalCall();
        var local = graph.AddClass("Example", "Item");
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var il = main.GetILGenerator();
        il.LoadTypeToken(local);
        il.Call(service);
        il.CallVirtual(name);
        il.Emit(OpCode.Ldstr, "SourceModuleInfo.dll");
        il.Call(compare);
        il.LoadConstant(0); il.Emit(OpCode.Ceq);
        var failure = il.DefineLabel();
        il.Emit(OpCode.Brfalse, failure);
        il.LoadConstant(42); il.Return();
        il.MarkLabel(failure); il.LoadConstant(1); il.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var image = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
        if (image.MainModule.Methods.Single(m => m.Name == "TypeModule").ImplementationAttributes != 0x1000)
            throw new Exception("source descriptor service flags lost");
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run();
        Directory.CreateDirectory(directory);
        var seed = Path.Combine(directory, "System.neoil");
        File.WriteAllText(seed, ".module System\n");
        foreach (var wrongLayout in new[] { false, true })
        {
            var path = Path.Combine(directory, wrongLayout ? "Invalid.dll" : "ModuleInfo.dll");
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(Create(wrongLayout)));
            foreach (var command in new[] { "verify", "run" })
            {
                var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
                foreach (var arg in new[] { command, path, "--system", seed }) start.ArgumentList.Add(arg);
                using var process = Process.Start(start)!;
                var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
                await process.WaitForExitAsync();
                var output = await stdout + await stderr;
                var expected = wrongLayout ? 1 : command == "verify" ? 0 : 42;
                if (process.ExitCode != expected || wrongLayout && !output.Contains("provider layout mismatch"))
                    throw new Exception(command + ": " + output);
            }
        }
    }
}
