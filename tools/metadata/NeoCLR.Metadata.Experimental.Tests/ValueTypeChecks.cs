using System.Diagnostics;
using System.Runtime.Loader;
using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ValueTypeChecks
{
    private static AssemblyBuilder Build()
    {
        var name = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(name.Name!, name.Version!, "", Convert.ToHexString(name.GetPublicKeyToken()!));
        var app = new AssemblyBuilder(new("ValueTypes", new Version(1, 0, 0, 0)), core);
        var number = app.AddValueType("Example", "Number");
        var field = number.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var box = app.AddGenericValueType("Example", "Box", ["T"]);
        var boxField = box.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var boxInt = box.MakeGenericInstance(PrimitiveType.Int32);
        var operations = app.AddType("Example", "Operations");
        var identity = operations.AddMethod("Identity", new MethodSignature(number, [number]));
        identity.LoadArgument(0); identity.Return();
        var forward = operations.AddMethod("Forward", new MethodSignature(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["T"]));
        forward.LoadArgument(0); forward.Return();
        var entry = operations.AddMethod("Main"); app.EntryPoint = entry;
        entry.LoadDefault(number); entry.Call(identity); entry.LoadField(field);
        entry.LoadDefault(boxInt); entry.Call(forward.MakeGenericInstance(boxInt)); entry.LoadField(boxField.MakeConstructedReference(PrimitiveType.Int32)); entry.Add();
        entry.LoadConstant(1); entry.NewArray(number); entry.Duplicate(); entry.LoadConstant(0);
        entry.LoadDefault(number); entry.StoreArrayElement(number);
        entry.LoadConstant(0); entry.LoadArrayElement(number); entry.LoadField(field); entry.Add();
        entry.LoadConstant(42); entry.Add(); entry.Return();
        var constrained = app.AddGenericClass("Example", "ValueOnly", ["T"]);
        constrained.SetSpecialConstraints(0, TypeParameterConstraints.ValueType | TypeParameterConstraints.DefaultConstructor);
        _ = constrained.MakeGenericInstance(number); _ = constrained.MakeGenericInstance(boxInt);
        var references = app.AddGenericClass("Example", "ReferenceOnly", ["T"]);
        references.SetSpecialConstraints(0, TypeParameterConstraints.ReferenceType);
        Reject<ArgumentException>(() => references.MakeGenericInstance(number));
        Reject<ArgumentException>(() => references.MakeGenericInstance(boxInt));
        Reject<ArgumentException>(() => references.AddBaseTypeConstraint(0, number));
        Reject<ArgumentException>(() => number.AddField("Recursive", number));
        Reject<ArgumentException>(() => box.AddField("Payload", SignatureType.TypeParameter(0)));
        Reject<InvalidOperationException>(() => number.AddConstructor([]));
        Reject<InvalidOperationException>(() => number.AddInstanceMethod("Bad", new MethodSignature(PrimitiveType.Void, [])));
        Reject<ArgumentException>(() => entry.StoreField(field));
        Reject<ArgumentException>(() => entry.StoreField(boxField.MakeConstructedReference(PrimitiveType.Int32)));
        return app;
    }

    internal static void Run()
    {
        var app = Build();
        var image = app.Write();
        var context = new AssemblyLoadContext("value-types", isCollectible: true);
        try
        {
            var loaded = context.LoadFromStream(new MemoryStream(image));
            if ((int)loaded.EntryPoint!.Invoke(null, null)! != 42) throw new Exception("CLR default value flow");
            if (!loaded.GetType("Example.Number")!.IsValueType || !loaded.GetType("Example.Box`1")!.IsValueType)
                throw new Exception("CLR value categories");
        }
        finally { context.Unload(); }
        var direct = AssemblyDefinition.ReadAssembly(image, false);
        var projection = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
        foreach (var snapshot in new[] { direct, projection })
        {
            if (!snapshot.MainModule.Types.Single(t => t.Name == "Number").IsValueType ||
                !snapshot.MainModule.Types.Single(t => t.Name == "Box`1").IsValueType ||
                snapshot.MainModule.Types.Single(t => t.Name == "Operations").IsValueType)
                throw new Exception("snapshot value categories");
            var importer = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), app.CoreLibrary);
            Reject<InvalidDataException>(() => importer.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Number"), app.CoreLibrary));
        }
        var invalid = JsonNode.Parse(app.WriteNativeAssembly())!;
        invalid["types"]![0]!["is_sealed"] = false;
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(invalid.ToJsonString())));
        invalid = JsonNode.Parse(app.WriteNativeAssembly())!;
        invalid["types"]![0]!["fields"]![0]!["ty"] = new JsonObject { ["Named"] = invalid["types"]![0]!["name"]!.GetValue<string>() };
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(invalid.ToJsonString())));
    }

    internal static async Task RunRuntime(string runtime, string output)
    {
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var app = Build();
        var path = Path.Combine(output, "ValueTypes.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
        foreach (var (command, expected) in new[] { ("verify", 0), ("run", 42) })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync();
            var text = await stdout; var error = await stderr;
            if (process.ExitCode != expected) throw new Exception(command + ": " + process.ExitCode + " " + text + error);
        }
        File.WriteAllText(Path.Combine(output, "validation.json"), JsonSerializer.Serialize(new
        {
            verified = true, result = 42,
            runtimeSha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(runtime))),
            assemblySha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path))),
            scope = "owned value-type categories, primitive fields, defaults, static forwarding, generic construction/arguments and arrays; no imported value types, generic payload fields, instance methods or addressed mutation"
        }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
    }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
