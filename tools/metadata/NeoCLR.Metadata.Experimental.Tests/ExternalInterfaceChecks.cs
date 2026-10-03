using System.Diagnostics;
using System.Security.Cryptography;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class ExternalInterfaceChecks
{
    private static (AssemblyBuilder Contracts, AssemblyBuilder Implementation, byte[] ContractsImage) Create(bool complete = true, bool implement = true, bool manual = false, bool wrongSignature = false)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var contracts = new AssemblyBuilder(new("ExternalContracts", new Version(1, 0, 0, 0)), core);
        var root = contracts.AddGenericInterface("Example", "Root", ["T"]);
        root.AddInterfaceMethod("Get", new(SignatureType.TypeParameter(0), []));
        var left = contracts.AddGenericInterface("Example", "Left", ["T"]);
        var right = contracts.AddGenericInterface("Example", "Right", ["T"]);
        left.AddBaseInterface(root.MakeGenericInstance(SignatureType.TypeParameter(0)));
        right.AddBaseInterface(root.MakeGenericInstance(SignatureType.TypeParameter(0)));
        var image = RuntimeAssemblyContainer.WriteBinary(contracts);
        var app = new AssemblyBuilder(new("ExternalImplementation", new Version(1, 0, 0, 0)), core);
        ImportedTypeReference Import(string name) => app.CreateInterfaceReference(contracts.Identity, core, Convert.ToHexString(SHA256.HashData(image)), "Example", name + "`1", 1);
        var importedRoot = Import("Root");
        var importedLeft = Import("Left");
        var importedRight = Import("Right");
        var get = app.CreateMethodReference(importedRoot, "Get", new(SignatureType.TypeParameter(0), []));
        app.AddInterfaceConversion(importedLeft, importedRoot.MakeGenericInstance(SignatureType.TypeParameter(0)));
        app.AddInterfaceConversion(importedRight, importedRoot.MakeGenericInstance(SignatureType.TypeParameter(0)));
        if (complete)
        {
            foreach (var reference in new[] { importedRoot, importedLeft, importedRight }) app.CompleteInterfaceReference(reference);
            app.CompleteInterfaceReference(importedRoot);
            Check(ReferenceEquals(get, app.CreateMethodReference(importedRoot, "Get", new(SignatureType.TypeParameter(0), []))), "completed method interning");
            Reject<InvalidOperationException>(() => app.CreateMethodReference(importedRoot, "Missing", new(PrimitiveType.Void, [])));
            Reject<InvalidOperationException>(() => app.AddInterfaceConversion(importedLeft, importedRight.MakeGenericInstance(SignatureType.TypeParameter(0))));
        }
        var diamond = app.AddGenericInterface("Example", "Diamond", ["T"]);
        diamond.AddBaseInterface(importedLeft.MakeGenericInstance(SignatureType.TypeParameter(0)));
        diamond.AddBaseInterface(importedRight.MakeGenericInstance(SignatureType.TypeParameter(0)));
        var box = app.AddClass("Example", "Box");
        if (manual)
            box.Definition.Interfaces.Add(new InterfaceImplementation(app.Definition.MainModule.ImportReference(contracts.Identity, "Example", "Root`1"), [PrimitiveType.Int32]));
        else box.AddInterfaceImplementation(importedRoot.MakeGenericInstance(PrimitiveType.Int32));
        box.AddInterfaceImplementation(diamond.MakeGenericInstance(PrimitiveType.Int32));
        var constructor = box.AddConstructor(Array.Empty<PrimitiveType>()); constructor.Return();
        if (implement)
        {
            var method = box.AddInstanceMethod("Get", new(PrimitiveType.Int32, wrongSignature ? [PrimitiveType.Int32] : [])); method.LoadConstant(42); method.Return();
        }
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        main.NewObject(constructor); main.CallVirtual(get.MakeConstructedReference([PrimitiveType.Int32])); main.Return();
        Reject<ArgumentException>(() => box.AddInterfaceImplementation(importedRoot.MakeGenericInstance(PrimitiveType.Int32)));
        Reject<ArgumentException>(() => app.CompleteInterfaceReference(importedRoot.MakeGenericInstance(PrimitiveType.Int32)));
        return (contracts, app, image);
    }

    internal static void Run()
    {
        foreach (var manual in new[] { false, true })
        {
            var (contracts, app, image) = Create(manual: manual);
            var load = new AssemblyLoadContext("external-interface-test", isCollectible: true);
            try
            {
                var dependency = load.LoadFromStream(new MemoryStream(contracts.Write()));
                load.Resolving += (_, name) => name.Name == dependency.GetName().Name ? dependency : null;
                var executable = load.LoadFromStream(new MemoryStream(app.Write()));
                Check(Equals(executable.EntryPoint!.Invoke(null, null), 42), "CLR external interface dispatch");
            }
            finally { load.Unload(); }
            var unrelated = app.Types.Single(t => t.Name == "Box").AddInstanceMethod("Unrelated", new(PrimitiveType.Int32, []));
            unrelated.LoadConstant(7); unrelated.Return();
            var appImage = RuntimeAssemblyContainer.WriteBinary(app);
            var context = new MetadataLoadContext([AssemblyDefinition.ReadNativeAssembly(image), AssemblyDefinition.ReadNativeAssembly(appImage)]);
            var view = context.Resolve(app.Identity).GetTypes().Single(t => t.Name == "Box");
            Check(view.GetInterfaces().Count == 4, "external generic diamond closure");
            var cli = RuntimeAssemblyContainer.ReadCliProjection(appImage);
            var method = cli.MainModule.Types.Single(t => t.Name == "Box").Methods.Single(m => m.Name == "Get");
            Check((method.Attributes & 0x160) == 0x160, "CLI implementation flags");
            Check((cli.MainModule.Types.Single(t => t.Name == "Box").Methods.Single(m => m.Name == "Unrelated").Attributes & 0x40) == 0, "unrelated method is not virtual");
            var missing = new MetadataLoadContext([AssemblyDefinition.ReadNativeAssembly(appImage)]);
            Reject<InvalidDataException>(() => missing.Resolve(app.Identity).GetTypes().Single(t => t.Name == "Box").GetInterfaces());
        }
        Reject<InvalidDataException>(() => Create(complete: false).Implementation.WriteNativeAssembly());
        Reject<InvalidDataException>(() => Create(implement: false).Implementation.WriteNativeAssembly());
        Reject<InvalidDataException>(() => Create(wrongSignature: true).Implementation.WriteNativeAssembly());
        var foreign = Create();
        var external = foreign.Implementation.CreateInterfaceReference(foreign.Contracts.Identity, foreign.Contracts.CoreLibrary,
            Convert.ToHexString(SHA256.HashData(foreign.ContractsImage)), "Example", "Empty");
        Reject<ArgumentException>(() => foreign.Contracts.CompleteInterfaceReference(external));
        foreign.Implementation.CompleteInterfaceReference(external);
        foreign.Implementation.AddClass("Example", "EmptyImplementation").AddInterfaceImplementation(external);
        _ = foreign.Implementation.WriteNativeAssembly();
    }

    internal static async Task RunRuntime(string runtime, string output)
    {
        Run(); Directory.CreateDirectory(output);
        var (_, app, contractsImage) = Create();
        var contractPath = Path.Combine(output, "Contracts.dll");
        var appPath = Path.Combine(output, "Implementation.dll");
        File.WriteAllBytes(contractPath, contractsImage);
        File.WriteAllBytes(appPath, RuntimeAssemblyContainer.WriteBinary(app));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { command, appPath, "--module", contractPath }) start.ArgumentList.Add(argument);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "run" ? 42 : 0)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("External generic diamond interface dispatch: native 42");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
