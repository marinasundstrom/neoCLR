using System.Diagnostics;
using System.Reflection;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ImportedInterfaceChecks
{
    internal static (AssemblyBuilder Library, AssemblyBuilder App) Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var library = new AssemblyBuilder(new("ImportedInterfaceLibrary", new Version(1, 0, 0, 0)), core);
        var contract = library.AddGenericInterface("Example", "Value", ["T"]);
        var echo = contract.AddInterfaceMethod("Echo", new MethodSignature(SignatureType.TypeParameter(0), [SignatureType.TypeParameter(0)]));
        var implementation = library.AddClass("Example", "Concrete");
        implementation.AddInterfaceImplementation(contract.MakeGenericInstance(PrimitiveType.Int32));
        var ctor = implementation.AddConstructor(Array.Empty<PrimitiveType>()); ctor.Return();
        var method = implementation.AddInstanceMethod("Echo", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        method.LoadArgument(1); method.Return();
        var factory = library.AddType("Example", "Factory");
        var create = factory.AddMethod("Create", new MethodSignature(contract.MakeGenericInstance(PrimitiveType.Int32), []));
        create.NewObject(ctor); create.Return();
        var snapshot = AssemblyDefinition.ReadAssembly(library.Write(), expectedExtended: false);
        var app = new AssemblyBuilder(new("ImportedInterfaceApp", new Version(1, 0, 0, 0)), core);
        var imported = app.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Value`1").Methods.Single(), core);
        var bound = imported.MakeConstructedReference([PrimitiveType.Int32]);
        if (!imported.IsInterfaceMethod || imported.IsStatic || bound.Signature.ReturnType.Primitive != PrimitiveType.Int32) throw new Exception("imported interface contract");
        var make = app.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Factory").Methods.Single(), core);
        var main = app.AddFunction("Main"); main.Call(make); main.LoadConstant(42); main.CallVirtual(bound); main.Return(); app.EntryPoint = main;
        void Reject(Action action) { try { action(); } catch (ArgumentException) { return; } throw new Exception("invalid import accepted"); }
        Reject(() => imported.MakeConstructedReference([]));
        Reject(() => main.Call(bound));
        var foreign = new AssemblyBuilder(new("Foreign", new Version(1, 0, 0, 0)), core);
        Reject(() => foreign.AddFunction("Bad").CallVirtual(bound));
        Reject(() => imported.MakeConstructedReference([foreign.AddClass("Example", "WrongScope")]));
        return (library, app);
    }
    internal static void Run()
    {
        var (library, app) = Create();
        var context = new AssemblyLoadContext("imported-interface", true);
        try {
            context.LoadFromStream(new MemoryStream(library.Write()));
            var loaded = context.LoadFromStream(new MemoryStream(app.Write()));
            if (!Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("CLR imported generic dispatch");
        } finally { context.Unload(); }
        var projection = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(library.WriteNativeAssembly()).CreateReferenceAssembly(library.CoreLibrary), expectedExtended: false);
        var projectedConsumer = new AssemblyBuilder(new("ProjectedInterfaceConsumer", new Version(1, 0, 0, 0)), library.CoreLibrary);
        _ = projectedConsumer.ImportReference(projection.MainModule.Types.Single(t => t.Name == "Value`1").Methods.Single(), library.CoreLibrary).MakeConstructedReference([PrimitiveType.Int32]);
        var missing = library.AddClass("Example", "MissingImplementation");
        missing.AddInterfaceImplementation(library.Types[0].MakeGenericInstance(PrimitiveType.Int32));
        if (missing.Definition.Interfaces.Single().TypeArguments.Single().Primitive != PrimitiveType.Int32) throw new Exception("definition relationship lost arguments");
        try { library.Write(); throw new Exception("missing implementation accepted"); }
        catch (InvalidDataException error) when (error.Message.Contains("missing public interface implementation")) { }
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        if (Directory.Exists(output)) throw new IOException("output must be fresh"); Directory.CreateDirectory(output);
        var (library, app) = Create();
        var libraryPath = Path.Combine(output, "Library.dll"); var appPath = Path.Combine(output, "App.dll");
        File.WriteAllBytes(libraryPath, RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), library.CoreLibrary));
        File.WriteAllBytes(appPath, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
        foreach (var command in new[] { "verify", "run" }) {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { command, appPath, "--module", libraryPath }) start.ArgumentList.Add(argument);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        var nullApp = new AssemblyBuilder(new("NullInterfaceApp", new Version(1, 0, 0, 0)), app.CoreLibrary);
        var snapshot = AssemblyDefinition.ReadAssembly(library.Write(), expectedExtended: false);
        var contract = nullApp.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Value`1").Methods.Single(), library.CoreLibrary).MakeConstructedReference([PrimitiveType.Int32]);
        var main = nullApp.AddFunction("Main"); nullApp.EntryPoint = main;
        main.LoadDefault(contract.DeclaringType); main.LoadConstant(42); main.CallVirtual(contract); main.Return();
        var nullPath = Path.Combine(output, "Null.dll"); File.WriteAllBytes(nullPath, RuntimeAssemblyContainer.WriteBinary(nullApp.WriteNativeAssembly(), app.CoreLibrary));
        var nullStart = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
        foreach (var argument in new[] { "run", nullPath, "--module", libraryPath }) nullStart.ArgumentList.Add(argument);
        using var failed = Process.Start(nullStart)!; var error = failed.StandardError.ReadToEndAsync(); var ignored = failed.StandardOutput.ReadToEndAsync();
        await failed.WaitForExitAsync(); var fault = await error + await ignored;
        if (failed.ExitCode == 0 || !fault.Contains("null", StringComparison.OrdinalIgnoreCase)) throw new Exception("expected null dispatch fault: " + fault);
        Console.WriteLine("PASS native imported constructed interface dispatch: 42; null receiver faults");
    }
}
