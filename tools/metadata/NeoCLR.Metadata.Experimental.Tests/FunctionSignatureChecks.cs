using System.Diagnostics;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class FunctionSignatureChecks
{
    static (AssemblyBuilder Library, AssemblyBuilder App) Create()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var shape = SignatureType.Function(new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        var library = new AssemblyBuilder(new("FunctionLibrary", new Version(1, 0, 0, 0)), core);
        var type = library.AddType("Example", "Functions");
        var apply = type.AddMethod("Apply", new MethodSignature(PrimitiveType.Int32, [shape, PrimitiveType.Int32]));
        apply.LoadArgument(0); apply.LoadArgument(1); apply.InvokeFunction(shape); apply.Return();
        var parameter = SignatureType.MethodParameter(0);
        var genericShape = SignatureType.Function(new MethodSignature(parameter, [parameter]));
        var applyGeneric = type.AddMethod("ApplyGeneric", new MethodSignature(parameter, [genericShape, parameter], ["T"]));
        applyGeneric.LoadArgument(0); applyGeneric.LoadArgument(1); applyGeneric.InvokeFunction(genericShape); applyGeneric.Return();
        var noResult = SignatureType.Function(new MethodSignature(PrimitiveType.Void, []));
        var call = type.AddMethod("Run", new MethodSignature(PrimitiveType.Void, [noResult]));
        call.LoadArgument(0); call.InvokeFunction(noResult); call.Return();
        var app = new AssemblyBuilder(new("FunctionApp", new Version(1, 0, 0, 0)), core);
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core));
        var definition = projection.MainModule.Types.Single(t => t.Name == "Functions");
        var imported = app.ImportReference(definition.Methods.Single(m => m.Name == "Apply"), core);
        var genericImport = app.ImportReference(definition.Methods.Single(m => m.Name == "ApplyGeneric"), core).MakeGenericInstance(PrimitiveType.Int32);
        var importedRun = app.ImportReference(definition.Methods.Single(m => m.Name == "Run"), core);
        if (imported.Signature.ParameterTypes[0] != shape || importedRun.Signature.ParameterTypes[0] != noResult) throw new Exception("Function carrier import lost shape");
        var increment = app.AddFunction("Increment", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        increment.LoadArgument(0); increment.LoadConstant(2); increment.Emit(OpCode.Add); increment.Return();
        var marker = app.AddFunction("Marker", new MethodSignature(PrimitiveType.Void, [])); marker.Return();
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        var local = main.DeclareLocal(shape);
        main.BindFunction(noResult, marker); main.Call(importedRun);
        main.Emit(OpCode.BindFunction, new FunctionBinding(shape, increment)); main.StoreLocal(local);
        main.LoadLocal(local); main.LoadConstant(38); main.Call(imported); var intermediate = main.DeclareLocal(PrimitiveType.Int32); main.StoreLocal(intermediate);
        main.LoadLocal(local); main.LoadLocal(intermediate); main.Call(genericImport); main.Return();
        return (library, app);
    }
    internal static void Run()
    {
        var (library, app) = Create();
        var context = new AssemblyLoadContext("structural-functions", true);
        try { context.LoadFromStream(new MemoryStream(library.Write())); var loaded = context.LoadFromStream(new MemoryStream(app.Write())); if (!Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("CLR function transport"); }
        finally { context.Unload(); }
        var first = SignatureType.Function(new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        var second = SignatureType.Function(new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        if (first != second || first.GetHashCode() != second.GetHashCode()) throw new Exception("Function structural identity");
        var target = app.Functions.Single(m => m.Name == "Increment");
        try { new FunctionBinding(SignatureType.Function(new MethodSignature(PrimitiveType.Boolean, [PrimitiveType.Int32])), target); throw new Exception("wrong Function result admitted"); } catch (ArgumentException) { }
        var caller = app.AddFunction("Bad"); caller.LoadConstant(42); caller.LoadConstant(1); caller.InvokeFunction(first); caller.Return();
        try { app.Write(); throw new Exception("non-Function receiver admitted"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory); var (library, app) = Create();
        var dependency = Path.Combine(directory, "Library.dll"); var path = Path.Combine(directory, "App.dll");
        File.WriteAllBytes(dependency, RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), library.CoreLibrary));
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
        foreach (var command in new[] { "verify", "run" }) {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var arg in new[] { command, path, "--module", dependency }) start.ArgumentList.Add(arg);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("Structural Function metadata executes across assemblies on CLR and neoCLR: 42");
    }
}
