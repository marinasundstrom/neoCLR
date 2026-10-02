using System.Diagnostics;
using System.Runtime.Loader;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeBindingChecks
{
    static (AssemblyBuilder Library, AssemblyBuilder App, byte[] Native) Create(bool wrongResult = false, bool unversioned = false)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("BoundLibrary", new Version(1, 0, 0, 0)), core);
        var type = library.AddClass("Example", "Box");
        var ctor = type.AddConstructor(Array.Empty<PrimitiveType>()); ctor.Return();
        var read = type.AddInstanceMethod("Read", new MethodSignature(PrimitiveType.Int32, [])); read.LoadConstant(42); read.Return();
        var touch = type.AddMethod("Touch", new MethodSignature(PrimitiveType.Void, [])); touch.Return();
        var native = JsonNode.Parse(library.WriteNativeAssembly())!.AsObject();
        var replacements = new Dictionary<string, string> { [native["name"]!.GetValue<string>()] = "BoundRuntime" };
        replacements[native["types"]![0]!["name"]!.GetValue<string>()] = "Example.Box";
        foreach (var function in native["functions"]!.AsArray())
            replacements[function!["name"]!.GetValue<string>()] = "Example.Box." + function["origin"]!["name"]!.GetValue<string>();
        JsonNode? Rewrite(JsonNode? node) => node switch
        {
            JsonValue value when value.TryGetValue<string>(out var text) && replacements.TryGetValue(text, out var replacement) => JsonValue.Create(replacement),
            JsonObject map => new JsonObject(map.Select(p => new KeyValuePair<string, JsonNode?>(p.Key, Rewrite(p.Value)))),
            JsonArray array => new JsonArray(array.Select(Rewrite).ToArray()),
            _ => node?.DeepClone()
        };
        native = Rewrite(native)!.AsObject();
        if (unversioned) native.Remove("revision");
        var nativeTouch = native["functions"]!.AsArray().Single(f => f!["name"]!.GetValue<string>() == "Example.Box.Touch")!;
        nativeTouch["no_result"] = false; nativeTouch["body"] = JsonNode.Parse("[{\"op\":\"ldvoid\"},{\"op\":\"ret\"}]");
        if (wrongResult) native["functions"]!.AsArray().Single(f => f!["name"]!.GetValue<string>() == "Example.Box.Read")!["returns"] = "Int64";
        var binary = NativeModuleContainer.WriteBinary(System.Text.Encoding.UTF8.GetBytes(native.ToJsonString()));
        var reference = AssemblyDefinition.ReadAssembly(library.Write(), expectedExtended: false);
        var app = new AssemblyBuilder(new("BoundApp", new Version(1, 0, 0, 0)), core);
        app.BindNativeLibrary(reference, NativeLibraryDefinition.ReadAssembly(binary), core);
        var definition = reference.MainModule.Types.Single(t => t.Name == "Box");
        var importedCtor = app.ImportReference(definition.Methods.Single(m => m.Name == ".ctor"), core);
        var importedRead = app.ImportReference(definition.Methods.Single(m => m.Name == "Read"), core);
        var importedTouch = app.ImportReference(definition.Methods.Single(m => m.Name == "Touch"), core);
        var importedType = app.ImportReference(definition, core);
        var identity = app.AddFunction("Identity", new MethodSignature(importedType, [importedType]));
        identity.LoadArgument(0); identity.Return();
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        main.Call(importedTouch);
        var success = main.DefineLabel(); main.Emit(OpCode.Ldc_Bool, true); main.Emit(OpCode.Brtrue, success);
        main.LoadConstant(-1); main.Return(); main.MarkLabel(success);
        main.NewObject(importedCtor); main.Call(importedRead); main.Return();
        return (library, app, binary);
    }
    internal static void Run()
    {
        var (library, app, _) = Create();
        var context = new AssemblyLoadContext("native-binding-cli", true);
        try { context.LoadFromStream(new MemoryStream(library.Write())); if (!Equals(42, context.LoadFromStream(new MemoryStream(app.Write())).EntryPoint!.Invoke(null, null))) throw new Exception("CLI binding changed"); }
        finally { context.Unload(); }
        var image = RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary);
        var projection = RuntimeAssemblyContainer.ReadCliProjection(image);
        if (projection.MainModule.AssemblyReferences.All(r => r.Identity.Name != library.Identity.Name)) throw new Exception("original reference scope lost");
        var (_, nameOnly, _) = Create(unversioned: true);
        _ = RuntimeAssemblyContainer.WriteBinary(nameOnly.WriteNativeAssembly(), nameOnly.CoreLibrary);
        foreach (var corruption in new[] { "scope", "duplicate", "category", "cycle" })
        {
            var json = JsonNode.Parse(app.WriteNativeAssembly())!;
            var manifest = json["assemblies"]![0]!;
            var aliases = manifest["native_type_bindings"]!.AsArray();
            if (corruption == "scope") manifest["native_module_bindings"] = new JsonArray();
            if (corruption == "duplicate") aliases.Add(aliases[0]!.DeepClone());
            if (corruption == "category") aliases[0]!["value_type"] = true;
            if (corruption == "cycle") aliases[0]!["declaring"] = aliases[0]!["native_name"]!.DeepClone();
            try { NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(json.ToJsonString())); throw new Exception("invalid native alias admitted: " + corruption); }
            catch (InvalidDataException) { }
        }
        try { Create(wrongResult: true); throw new Exception("incompatible native result admitted"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory); var (_, app, native) = Create(unversioned: true);
        var dependency = Path.Combine(directory, "Library.neox"); var path = Path.Combine(directory, "App.dll");
        File.WriteAllBytes(dependency, native); File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var arg in new[] { command, path, "--module", dependency }) start.ArgumentList.Add(arg);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("Explicit native linkage preserves CLI scope and native calls/result adaptation: 42");
    }
}
