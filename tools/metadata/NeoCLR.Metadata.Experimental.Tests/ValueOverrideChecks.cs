using System.Runtime.Loader;
using System.Diagnostics;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ValueOverrideChecks
{
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        var input = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(typeof(ValueOverrideFixture).Assembly.Location), expectedExtended: false);
        var core = input.MainModule.AssemblyReferences.Single(r => r.Identity.Name == "System.Runtime").Identity;
        var library = new AssemblyBuilder(input.Identity, core);
        var value = library.AddValueType("", nameof(ValueOverrideFixture));
        var implementation = value.AddInstanceMethod("ToString", new MethodSignature(PrimitiveType.String, []));
        implementation.Emit(OpCode.Ldstr, "value override"); implementation.Return();
        var app = new AssemblyBuilder(new("ValueOverrideNative", new Version(1, 0, 0, 0)), core);
        var definition = input.MainModule.Types.Single(t => t.Name == nameof(ValueOverrideFixture));
        var type = app.ImportReference(definition, core);
        var method = app.ImportReference(definition.Methods.Single(m => m.Name == "ToString"), core);
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        var local = main.DeclareLocal(type); main.LoadLocalAddress(local); main.InitializeObject(type);
        main.LoadLocalAddress(local); main.Call(method); main.WriteConsoleLine(); main.LoadConstant(42); main.Return();
        var dependency = Path.Combine(directory, "Library.dll"); var path = Path.Combine(directory, "App.dll");
        File.WriteAllBytes(dependency, RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core));
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), core));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var arg in new[] { command, path, "--module", dependency }) start.ArgumentList.Add(arg);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var output = await stdout; var error = await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42) || command == "run" && output.Trim() != "value override") throw new Exception(command + ": " + output + error);
        }
        Console.WriteLine("Imported value override executes on CLR and native concrete implementation: value override / 42");
    }
    internal static void Run()
    {
        var input = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(typeof(ValueOverrideFixture).Assembly.Location), expectedExtended: false);
        var core = input.MainModule.AssemblyReferences.Single(r => r.Identity.Name == "System.Runtime").Identity;
        var app = new AssemblyBuilder(new("ValueOverrideConsumer", new Version(1, 0, 0, 0)), core);
        var type = input.MainModule.Types.Single(t => t.Name == nameof(ValueOverrideFixture));
        var reference = app.ImportReference(type, core);
        var method = app.ImportReference(type.Methods.Single(m => m.Name == "ToString"), core);
        if (!method.RequiresManagedReceiver || method.RequiresVirtualDispatch) throw new Exception("value override must use a managed concrete receiver");
        var read = app.AddType("Example", "Consumer").AddMethod("Read", new MethodSignature(PrimitiveType.String, []));
        var local = read.DeclareLocal(reference); read.LoadLocalAddress(local); read.InitializeObject(reference);
        read.LoadLocalAddress(local); read.Call(method); read.Return();
        var context = new AssemblyLoadContext("value-override", true);
        try
        {
            context.LoadFromAssemblyPath(typeof(ValueOverrideFixture).Assembly.Location);
            var loaded = context.LoadFromStream(new MemoryStream(app.Write()));
            if (!Equals("value override", loaded.GetType("Example.Consumer")!.GetMethod("Read")!.Invoke(null, null))) throw new Exception("concrete value override result");
        }
        finally { context.Unload(); }
        var classType = input.MainModule.Types.Single(t => t.Name == nameof(ReferenceOverrideFixture));
        try { app.ImportReference(classType.Methods.Single(m => m.Name == "ToString"), core); throw new Exception("nonfinal reference override admitted"); }
        catch (InvalidDataException) { }
    }
}

public struct ValueOverrideFixture
{
    public override string ToString() => "value override";
}
public class ReferenceOverrideFixture
{
    public override string ToString() => "reference override";
}
