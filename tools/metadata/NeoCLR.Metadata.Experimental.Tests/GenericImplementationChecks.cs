using System.Diagnostics;
using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericImplementationChecks
{
    static AssemblyBuilder Create(bool wrongSignature = false, bool wrongReceiver = false)
    {
        var host = typeof(object).Assembly.GetName();
        var graph = new AssemblyBuilder(new("GenericImplementation", new Version(1, 0, 0, 0)), new(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!)));
        var root = graph.AddGenericInterface("Example", "Root", ["T"]);
        var echo = root.AddInterfaceMethod("Echo", new(SignatureType.TypeParameter(0), [SignatureType.TypeParameter(0)]));
        var middle = graph.AddGenericInterface("Example", "Middle", ["T"]);
        middle.AddBaseInterface(root.MakeGenericInstance(SignatureType.TypeParameter(0)));
        var implementation = graph.AddGenericClass("Example", "Implementation", ["Unused", "T"]);
        implementation.AddInterfaceImplementation(middle.MakeGenericInstance(SignatureType.TypeParameter(1)));
        var ctor = implementation.AddConstructor(Array.Empty<PrimitiveType>()); ctor.Return();
        var type = SignatureType.TypeParameter(wrongSignature ? 0 : 1);
        var body = implementation.AddInstanceMethod("Echo", new(type, [type])); body.LoadArgument(1); body.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        main.NewObject(ctor.MakeConstructedReference([PrimitiveType.Boolean, wrongReceiver ? PrimitiveType.Boolean : PrimitiveType.Int32]));
        main.LoadConstant(42); main.CallVirtual(echo.MakeConstructedReference([PrimitiveType.Int32])); main.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("generic implementation CLR dispatch");
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        if (!projection.MainModule.Types.Any(t => t.Name == "Implementation`2")) throw new Exception("generic implementation projection");
        void Reject(Action action) { try { action(); } catch (Exception e) when (e is ArgumentException or InvalidDataException) { return; } throw new Exception("invalid generic implementation admitted"); }
        Reject(() => Create(true).Write());
        Reject(() => Create(wrongReceiver: true).Write());
        Reject(() => graph.Types[2].AddInterfaceImplementation(graph.Types[0].MakeGenericInstance(SignatureType.TypeParameter(2))));
        Reject(() => graph.Types[2].AddInterfaceImplementation(graph.Types[0].MakeGenericInstance(SignatureType.MethodParameter(0))));
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        Directory.CreateDirectory(output);
        var graph = Create(); var path = Path.Combine(output, "GenericImplementation.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync();
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + await stdout + await stderr);
        }
        File.WriteAllText(Path.Combine(output, "validation.json"), System.Text.Json.JsonSerializer.Serialize(new {
            scenario = "Implementation<Unused,T> implements Middle<T> inheriting Root<T>; Echo dispatch through Root<int>",
            cliResult = 42, nativeResult = 42,
            runtimeSha256 = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(runtime))),
            binarySha256 = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(path))),
            rejectionChecks = new[] { "wrong owner argument in implementation signature", "incompatible constructed receiver", "out-of-scope owner argument", "out-of-scope method argument" }
        }, new System.Text.Json.JsonSerializerOptions { WriteIndented = true }) + "\n");
        Console.WriteLine("Generic class inherited interface dispatch: CLR/native 42");
    }
}
