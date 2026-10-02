using System.Diagnostics;
using System.Reflection;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ValueReceiverChecks
{
    internal static (AssemblyBuilder Library, AssemblyBuilder App) Create() => Create(out _);
    private static (AssemblyBuilder Library, AssemblyBuilder App) Create(out ImportedMethodReference change)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var library = new AssemblyBuilder(new("ValueReceiverLibrary", new Version(1, 0, 0, 0)), core);
        var number = library.AddValueType("Example", "Number");
        var field = number.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var increment = number.AddInstanceMethod("Increment", new MethodSignature(PrimitiveType.Void, [SignatureType.ByReference(PrimitiveType.Int32)], outParameters: [0]));
        increment.LoadArgument(0); increment.LoadObject(number); increment.Emit(OpCode.Pop);
        increment.LoadArgument(0); increment.LoadArgument(0); increment.LoadField(field);
        increment.LoadConstant(42); increment.Emit(OpCode.Add); increment.StoreField(field);
        increment.LoadArgument(1); increment.LoadArgument(0); increment.LoadField(field); increment.StoreObject(PrimitiveType.Int32); increment.Return();
        var box = library.AddGenericValueType("Example", "Box", ["T"]);
        var t = SignatureType.TypeParameter(0);
        var value = box.AddField("Value", t, FieldVisibility.Public);
        var set = box.AddInstanceMethod("Set", new MethodSignature(PrimitiveType.Void, [t]));
        set.LoadArgument(0); set.LoadArgument(1); set.StoreField(value); set.Return();
        var read = box.AddInstanceMethod("TryGet", new MethodSignature(PrimitiveType.Boolean, [SignatureType.ByReference(t)], outParameters: [0]));
        read.LoadArgument(1); read.LoadArgument(0); read.LoadField(value); read.StoreObject(t); read.Emit(OpCode.Ldc_Bool, true); read.Return();
        var snapshot = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core));
        var app = new AssemblyBuilder(new("ValueReceiverApp", new Version(1, 0, 0, 0)), core);
        var importedNumber = app.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Number"), core);
        change = app.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Number").Methods.Single(), core);
        var importedBox = snapshot.MainModule.Types.Single(t => t.Name == "Box`1");
        var write = app.ImportReference(importedBox.Methods.Single(m => m.Name == "Set"), core).MakeConstructedReference([PrimitiveType.Int32]);
        var get = app.ImportReference(importedBox.Methods.Single(m => m.Name == "TryGet"), core).MakeConstructedReference([PrimitiveType.Int32]);
        if (!change.RequiresManagedReceiver || change.RequiresVirtualDispatch || !get.Definition.RequiresManagedReceiver) throw new Exception("wrong receiver contract");
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        var local = main.DeclareLocal(importedNumber); var result = main.DeclareLocal(PrimitiveType.Int32);
        main.LoadLocalAddress(local); main.InitializeObject(importedNumber);
        main.LoadLocalAddress(local); main.LoadLocalAddress(result); main.Call(change);
        var boxLocal = main.DeclareLocal(write.DeclaringType);
        main.LoadLocalAddress(boxLocal); main.InitializeObject(write.DeclaringType);
        main.LoadLocalAddress(boxLocal); main.LoadLocal(result); main.Call(write);
        main.LoadLocalAddress(boxLocal); main.LoadLocalAddress(result); main.Call(get); main.Emit(OpCode.Pop);
        main.LoadLocal(result); main.Return();
        return (library, app);
    }
    internal static void Run()
    {
        var (library, app) = Create(out var change);
        var context = new AssemblyLoadContext("value-receiver", true);
        try {
            context.LoadFromStream(new MemoryStream(library.Write()));
            var loaded = context.LoadFromStream(new MemoryStream(app.Write()));
            if (!Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("CLR value receiver calls");
        } finally { context.Unload(); }
        var projection = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(library.WriteNativeAssembly()).CreateReferenceAssembly(library.CoreLibrary), expectedExtended: false);
        var projectedConsumer = new AssemblyBuilder(new("ProjectedValueConsumer", new Version(1, 0, 0, 0)), library.CoreLibrary);
        var get = projectedConsumer.ImportReference(projection.MainModule.Types.Single(t => t.Name == "Box`1").Methods.Single(m => m.Name == "TryGet"), library.CoreLibrary).MakeConstructedReference([PrimitiveType.Int32]);
        if (!get.Definition.RequiresManagedReceiver || !get.Signature.OutParameters.SequenceEqual([0])) throw new Exception("projection lost receiver/output contract");
        var main = app.EntryPoint!;
        var number = main.Locals[0]; var output = main.Locals[1];
        main.ClearBody(); main.LoadLocalAddress(number); main.LoadLocalAddress(output); main.Call(change); main.LoadLocal(output); main.Return();
        Reject(app, "uninitialized receiver");
        main.ClearBody(); main.LoadDefault(number.SignatureType); main.LoadLocalAddress(output); main.Call(change); main.LoadLocal(output); main.Return();
        Reject(app, "value instead of address receiver");
        try { main.CallVirtual(change); throw new Exception("value callvirt admitted without constrained semantics"); } catch (ArgumentException) { }
        var alias = new AssemblyBuilder(new("AliasReceiver", new Version(1, 0, 0, 0)), library.CoreLibrary);
        var type = alias.AddValueType("Example", "Value");
        var fill = type.AddInstanceMethod("Fill", new MethodSignature(PrimitiveType.Void, [SignatureType.ByReference(type)], outParameters: [0]));
        fill.LoadArgument(1); fill.LoadDefault(type); fill.StoreObject(type); fill.Return();
        var entry = alias.AddFunction("Main"); alias.EntryPoint = entry;
        var slot = entry.DeclareLocal(type); entry.LoadLocalAddress(slot); entry.LoadLocalAddress(slot); entry.Call(fill); entry.LoadConstant(42); entry.Return();
        Reject(alias, "out alias must not initialize receiver precondition");
    }
    private static void Reject(AssemblyBuilder graph, string message)
    {
        try { graph.Write(); throw new Exception(message); } catch (InvalidDataException) { }
        try { graph.WriteNativeAssembly(); throw new Exception(message); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
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
        Console.WriteLine("PASS imported value receiver mutation and generic out call: CLR/native 42");
    }
}
