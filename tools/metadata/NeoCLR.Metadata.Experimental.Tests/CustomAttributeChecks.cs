using System.Runtime.Loader;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class CustomAttributeChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("AttributeCli", new Version(1, 0, 0, 0)), core);
        var type = graph.AddClass("Example", "Annotated");
        var reference = graph.Definition.MainModule.ImportReference(core, "System", "ObsoleteAttribute");
        var attribute = new CustomAttributeDefinition(reference, [new(PrimitiveType.String, "union case ☃"), new(PrimitiveType.Boolean, true)]);
        type.Definition.CustomAttributes.Add(attribute);
        var bytes = graph.Write();
        var snapshot = AssemblyDefinition.ReadAssembly(bytes, false);
        var read = snapshot.MainModule.Types.Single(t => t.Name == "Annotated").CustomAttributes.Single();
        Check(read.AttributeType.Name == "ObsoleteAttribute" && read.GetValue().SequenceEqual(attribute.GetValue()), "CLI attribute roundtrip");
        Check((string)read.GetArguments()[0].Value! == "union case ☃", "UTF8 argument");
        var context = new AssemblyLoadContext("attribute-checks", isCollectible: true);
        try
        {
            var loaded = context.LoadFromStream(new MemoryStream(bytes));
            var data = loaded.GetType("Example.Annotated")!.GetCustomAttributesData().Single(a => a.AttributeType == typeof(ObsoleteAttribute));
            Check(Equals(data.ConstructorArguments[0].Value, "union case ☃") && Equals(data.ConstructorArguments[1].Value, true), "CLR interprets standard custom attribute blob");
        }
        finally { context.Unload(); }
        var foreign = new AssemblyBuilder(new("ForeignAttribute", new Version(1, 0, 0, 0)), core);
        Reject<ArgumentException>(() => foreign.AddClass("Example", "Other").AddCustomAttribute(attribute));
        Reject<NotSupportedException>(() => snapshot.MainModule.Types.Single(t => t.Name == "Annotated").CustomAttributes.Add(attribute));
        Reject<ArgumentException>(() => new CustomAttributeArgument(PrimitiveType.Int64, 1L));
        Reject<ArgumentException>(() => new CustomAttributeArgument(PrimitiveType.Int32, null));
        var blob = read.GetValue(); blob[0] = 0;
        Check(read.GetValue()[0] == 1, "snapshot blob ownership");
        var view = new MetadataLoadContext([snapshot]).Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Annotated").GetCustomAttributes().Single();
        Check(view.Name == "ObsoleteAttribute", "attribute facade name");
        Reject<InvalidDataException>(() => view.GetAttributeType());
        Check(Equals(view.GetArguments()[1].Value, true), "argument inspection does not load dependencies");
        NativeRoundtrip();
    }
    internal static (byte[] Library, byte[] App) NativeRoundtrip()
    {
        // Native metadata permits constructor-bearing record types. This fixture tests
        // constructor binding/data preservation, not CLI Attribute inheritance generation.
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var library = new AssemblyBuilder(new("AttributeContracts", new Version(1, 0, 0, 0)), core);
        var attributeType = library.AddClass("Contracts", "CaseAttribute");
        var ctor = attributeType.AddConstructor(new MethodSignature(PrimitiveType.Void, [PrimitiveType.String, PrimitiveType.String, PrimitiveType.Int32]));
        ctor.GetILGenerator().Fail("attribute constructor must not execute during metadata loading");
        var localArguments = new CustomAttributeArgument[] { new(PrimitiveType.String, "Self"), new(PrimitiveType.String, "marker"), new(PrimitiveType.Int32, -1) };
        attributeType.AddCustomAttribute(new(ctor.Definition, localArguments));
        Reject<ArgumentException>(() => new CustomAttributeDefinition(ctor.Definition, [new(PrimitiveType.Int32, 0)]));
        var libraryImage = RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core);
        var app = new AssemblyBuilder(new("AttributeConsumer", new Version(1, 0, 0, 0)), core);
        var owner = app.AddGenericValueType("Example", "Choice", ["T"]);
        var reference = app.Definition.MainModule.ImportReference(library.Identity, "Contracts", "CaseAttribute");
        var args = new CustomAttributeArgument[] { new(PrimitiveType.String, "Example.Choice+Some`1"), new(PrimitiveType.String, "Some"), new(PrimitiveType.Int32, 0) };
        var importedConstructor = app.ImportReference(AssemblyDefinition.ReadNativeAssembly(libraryImage).MainModule.Types.Single(t => t.Name == "CaseAttribute").Methods.Single(), core);
        owner.AddCustomAttribute(new(importedConstructor, args));
        Reject<ArgumentException>(() => new CustomAttributeDefinition(importedConstructor, [new(PrimitiveType.Int32, 0)]));
        owner.Definition.CustomAttributes.Add(new(reference, [new(PrimitiveType.String, null), new(PrimitiveType.String, "None"), new(PrimitiveType.Int32, 1)]));
        var entry = app.AddFunction("Main"); entry.GetILGenerator().LoadConstant(42); entry.GetILGenerator().Return(); app.EntryPoint = entry;
        var json = app.WriteNativeAssembly();
        var image = RuntimeAssemblyContainer.WriteBinary(json, core);
        var loaded = AssemblyDefinition.ReadNativeAssembly(image);
        var read = loaded.MainModule.Types.Single(t => t.Name == "Choice`1").CustomAttributes;
        Check(read.Count == 2 && Equals(read[0].GetArguments()[2].Value, 0) && read[1].GetArguments()[0].Value is null, "native attribute data and ordering");
        var projection = RuntimeAssemblyContainer.ReadCliProjection(image);
        Check(projection.MainModule.Types.Single(t => t.Name == "Choice`1").CustomAttributes[0].GetValue().SequenceEqual(read[0].GetValue()), "projection retains standard attribute bytes");
        var catalog = new MetadataLoadContext([loaded, AssemblyDefinition.ReadNativeAssembly(libraryImage)]);
        var view = catalog.Resolve(loaded.Identity).GetTypes().Single(t => t.Name == "Choice`1");
        var resolved = view.GetCustomAttributes()[0].GetAttributeType();
        Check(resolved.Name == "CaseAttribute" && ReferenceEquals(resolved, view.GetCustomAttributes()[1].GetAttributeType()), "canonical explicit dependency resolution");
        foreach (var corruption in new[] { "count", "type", "name", "owner" })
        {
            var changed = JsonNode.Parse(json)!;
            var a = changed["types"]![0]!["custom_attributes"]![0]!;
            if (corruption == "count") a["arguments"]!.AsArray().RemoveAt(0);
            if (corruption == "type") a["constructor"]!["parameters"]![2] = "Int64";
            if (corruption == "name") a["constructor"]!["name"] = "Missing";
            if (corruption == "owner") a["constructor"]!["owner"] = "Int32";
            Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(changed.ToJsonString())));
        }
        return (libraryImage, image);
    }
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); var (library, app) = NativeRoundtrip(); Directory.CreateDirectory(directory);
        var libraryPath = Path.Combine(directory, "Library.dll"); var appPath = Path.Combine(directory, "Consumer.dll");
        File.WriteAllBytes(libraryPath, library); File.WriteAllBytes(appPath, app);
        await Execute("verify", [appPath], 1, "missing referenced module");
        foreach (var command in new[] { "verify", "run" })
            await Execute(command, [appPath, "--module", libraryPath], command == "run" ? 42 : 0);

        async Task Execute(string command, string[] arguments, int expectedExit, string? expectedError = null)
        {
            var start = new System.Diagnostics.ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command);
            foreach (var arg in arguments) start.ArgumentList.Add(arg);
            using var process = System.Diagnostics.Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync();
            var stderr = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(60));
            try { await process.WaitForExitAsync(timeout.Token); }
            catch { process.Kill(true); throw; }
            var output = await stdout;
            var error = await stderr;
            Check(process.ExitCode == expectedExit, output + error);
            if (expectedError is not null) Check(error.Contains(expectedError), error);
            else Check((command != "run" || output.Length == 0) && error.Length == 0, output + error);
        }
        Console.WriteLine("PASS custom attribute native load/verify/run (42), CLI decoding and native/projection/introspection roundtrips");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
