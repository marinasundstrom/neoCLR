using System.Diagnostics;
using System.Runtime.Loader;
using System.Text.Json;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ImportedValueChecks
{
    private static (AssemblyBuilder Library, AssemblyBuilder App) Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("ValueLibrary", new Version(1, 0, 0, 0)), core);
        var number = library.AddValueType("Example", "Number");
        var field = number.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var ops = library.AddType("Example", "Operations");
        var create = ops.AddMethod("Create", new MethodSignature(number, []));
        var local = create.DeclareLocal(number); create.LoadDefault(number); create.StoreLocal(local);
        create.LoadLocalAddress(local); create.LoadConstant(42); create.StoreField(field); create.LoadLocal(local); create.Return();
        var read = ops.AddMethod("Read", new MethodSignature(PrimitiveType.Int32, [number]));
        read.LoadArgument(0); read.LoadField(field); read.Return();
        var wrapper = library.AddGenericValueType("Example", "Wrapper", ["T"]);
        wrapper.AddField("Value", SignatureType.TypeParameter(0), FieldVisibility.Public);
        var t = SignatureType.MethodParameter(0);
        var constructed = wrapper.MakeGenericInstance(t);
        var forward = ops.AddMethod("Forward", new MethodSignature(constructed, [constructed], ["T"]));
        forward.LoadArgument(0); forward.Return();
        var snapshot = AssemblyDefinition.ReadAssembly(library.Write(), false);
        var app = new AssemblyBuilder(new("ValueConsumer", new Version(1, 0, 0, 0)), core);
        ImportedMethodReference Import(string name) => app.ImportReference(snapshot.MainModule.Methods.Single(m => m.Name == name), core);
        var factory = Import("Create"); var reader = Import("Read");
        if (factory.Signature.ReturnType.ImportedType?.IsValueType != true || factory.Signature.ReturnType != reader.Signature.ParameterTypes[0])
            throw new Exception("imported value identity/category");
        var generic = Import("Forward").MakeGenericInstance(PrimitiveType.Int32);
        if (generic.Signature.ReturnType.ImportedType?.IsValueType != true) throw new Exception("constructed value category");
        var echo = app.AddFunction("Echo", new MethodSignature(generic.Signature.ReturnType, [generic.Signature.ReturnType]));
        echo.LoadArgument(0); echo.Return();
        var entry = app.AddFunction("Main"); app.EntryPoint = entry;
        entry.LoadDefault(generic.Signature.ReturnType); entry.Call(generic); entry.Emit(OpCode.Pop);
        entry.Call(factory); entry.Call(reader); entry.Return();
        return (library, app);
    }
    internal static void Run()
    {
        var (library, app) = Create();
        var context = new AssemblyLoadContext("imported-values", isCollectible: true);
        try
        {
            context.LoadFromStream(new MemoryStream(library.Write()));
            if ((int)context.LoadFromStream(new MemoryStream(app.Write())).EntryPoint!.Invoke(null, null)! != 42) throw new Exception("CLR imported value call");
        }
        finally { context.Unload(); }
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
        var echoSignature = projection.MainModule.Functions.Single(m => m.Name == "Echo").GetSignature();
        if (echoSignature[2] != 0x15 || echoSignature[3] != 0x11) throw new Exception("projected imported GENERICINST VALUETYPE category");
        // Re-emitting the native reference image must preserve value signatures in all metadata,
        // including the generic default local's TypeSpec. CLR loading detects category mismatches.
        var projectedLibrary = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), library.CoreLibrary));
        var consumer = new AssemblyBuilder(new("ProjectedValues", new Version(1, 0, 0, 0)), app.CoreLibrary);
        if (!consumer.ImportReference(projectedLibrary.MainModule.Types.Single(t => t.Name == "Number"), app.CoreLibrary).IsValueType)
            throw new Exception("projected value declaration category");
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output); var (library, app) = Create();
        var libraryPath = Path.Combine(output, "ValueLibrary.dll"); var appPath = Path.Combine(output, "ValueConsumer.dll");
        File.WriteAllBytes(libraryPath, RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), library.CoreLibrary));
        File.WriteAllBytes(appPath, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { command, appPath, "--module", libraryPath }) start.ArgumentList.Add(argument);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var message = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + message);
        }
        var badLibrary = JsonNode.Parse(library.WriteNativeAssembly())!;
        badLibrary["types"]![0]!["is_reference_type"] = true;
        var badPath = Path.Combine(output, "WrongCategory.json"); File.WriteAllText(badPath, badLibrary.ToJsonString());
        var invalidStart = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
        foreach (var argument in new[] { "verify", appPath, "--module", badPath }) invalidStart.ArgumentList.Add(argument);
        using (var process = Process.Start(invalidStart)!)
        {
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var diagnostic = await stdout + await stderr;
            if (process.ExitCode == 0 || !diagnostic.Contains("value type category mismatch")) throw new Exception("runtime must reject imported category mismatch: " + diagnostic);
        }
        File.WriteAllText(Path.Combine(output, "validation.json"), JsonSerializer.Serialize(new { verified = true, result = 42, scope = "separate value library/consumer; nominal and generic imported value signatures, factory/read and default/forward calls" }) + "\n");
    }
}
