using System.Diagnostics;
using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ConstructedInheritanceChecks
{
    static AssemblyBuilder Create(bool missing = false)
    {
        var host = typeof(object).Assembly.GetName();
        var graph = new AssemblyBuilder(new("ConstructedInheritance", new Version(1, 0, 0, 0)), new(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!)));
        var leaf = graph.AddInterface("Example", "Leaf");
        var middle = graph.AddGenericInterface("Example", "Middle", ["T"]);
        var root = graph.AddGenericInterface("Example", "Root", ["Unused", "T"]);
        var get = root.AddInterfaceMethod("Get", new MethodSignature(SignatureType.TypeParameter(1), []));
        leaf.AddBaseInterface(middle.MakeGenericInstance(PrimitiveType.Int32));
        middle.AddBaseInterface(root.MakeGenericInstance(PrimitiveType.Boolean, SignatureType.TypeParameter(0)));
        var implementation = graph.AddClass("Example", "Implementation"); implementation.AddInterfaceImplementation(leaf);
        var ctor = implementation.AddConstructor(Array.Empty<PrimitiveType>()); ctor.Return();
        if (!missing) { var read = implementation.AddInstanceMethod("Get", new(PrimitiveType.Int32, [])); read.LoadConstant(42); read.Return(); }
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        main.NewObject(ctor); main.CallVirtual(get.MakeConstructedReference([PrimitiveType.Boolean, PrimitiveType.Int32])); main.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("inherited generic CLR dispatch");
        if (loaded.GetType("Example.Leaf")!.GetInterfaces().Length != 2) throw new Exception("missing transitive interfaces");
        _ = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        void Reject(Action action) { try { action(); } catch (Exception e) when (e is ArgumentException or InvalidDataException) { return; } throw new Exception("invalid inherited contract admitted"); }
        Reject(() => Create(missing: true).Write());
        Reject(() => graph.Types[2].AddBaseInterface(graph.Types[1].MakeGenericInstance(SignatureType.TypeParameter(0))));
        Reject(() => graph.Types[0].AddBaseInterface(graph.Types[1].MakeGenericInstance(PrimitiveType.Int32)));
        Reject(() => graph.Types[0].AddBaseInterface(graph.Types[2].MakeGenericInstance(PrimitiveType.Boolean, SignatureType.TypeParameter(0))));
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run(); Directory.CreateDirectory(output); var graph = Create(); var path = Path.Combine(output, "Inheritance.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("Constructed inherited interface dispatch: CLR/native 42");
    }
}
