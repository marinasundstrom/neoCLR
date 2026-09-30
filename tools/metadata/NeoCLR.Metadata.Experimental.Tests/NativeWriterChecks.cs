using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;
using System.Text.Json;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeWriterChecks
{
    private static (AssemblyBuilder App, AssemblyBuilder Library) Build()
    {
        var core = new AssemblyIdentity("neoCLR.Core", new Version(1, 0, 0, 0));
        var library = new AssemblyBuilder(new("NativeFunctions", new Version(1, 0, 0, 0)), core);
        var twice = library.AddFunction("Twice", 1);
        twice.LoadArgument(0); twice.LoadConstant(2); twice.Multiply(); twice.Return();
        var app = new AssemblyBuilder(new("NativeConsumer", new Version(1, 0, 0, 0)), core);
        var noResult = app.AddFunction("NoResult", returnsValue: false); noResult.Return();
        var plusTwo = app.AddType("Example", "Math").AddMethod("PlusTwo", 1);
        plusTwo.LoadArgument(0); plusTwo.LoadConstant(2); plusTwo.Add(); plusTwo.Return();
        var main = app.AddFunction("Main");
        main.Call(noResult); main.LoadConstant(21); main.Call(twice); main.Call(plusTwo);
        main.LoadConstant(2); main.Subtract(); main.Return();
        app.EntryPoint = main;
        return (app, library);
    }
    internal static void Run()
    {
        var (app, library) = Build();
        Check(app.Functions.Count == 2 && app.EntryPoint!.DeclaringType is null && ReferenceEquals(app.EntryPoint.Assembly, app), "top-level ownership");
        using var document = JsonDocument.Parse(app.WriteNativeAssembly());
        var root = document.RootElement;
        Check(root.GetProperty("format").GetInt32() == 5, "native format");
        Check(root.GetProperty("types").GetArrayLength() == 1, "no synthetic global type");
        Check(root.GetProperty("functions")[0].GetProperty("owner").ValueKind == JsonValueKind.Null, "no type owner");
        Check(root.GetProperty("functions")[0].GetProperty("no_result").GetBoolean(), "absent result");
        Check(app.WriteNativeAssembly().SequenceEqual(app.WriteNativeAssembly()), "stable native output");
        using var pe = new PEReader(new MemoryStream(library.Write()));
        var metadata = pe.GetMetadataReader();
        var module = metadata.GetTypeDefinition(System.Reflection.Metadata.Ecma335.MetadataTokens.TypeDefinitionHandle(1));
        Check(module.GetMethods().Count == 1 && metadata.TypeDefinitions.Count == 1, "CLI global method without user type");
        Reject(() => app.Write()); // A cross-assembly top-level call has no supported PE mapping yet.
        app.EntryPoint!.ClearBody(); app.EntryPoint.Return(); Reject(() => app.WriteNativeAssembly());
        app.EntryPoint.ClearBody(); app.EntryPoint.LoadConstant(7); app.EntryPoint.Return();
        using var changed = JsonDocument.Parse(app.WriteNativeAssembly());
        Check(changed.RootElement.GetProperty("references").GetArrayLength() == 0, "removed call removes dependency");
        try { app.AddFunction("Main"); throw new Exception("duplicate accepted"); } catch (ArgumentException) { }
        using (var localPe = new PEReader(new MemoryStream(app.Write())))
        {
            var localMetadata = localPe.GetMetadataReader();
            var global = localMetadata.GetTypeDefinition(System.Reflection.Metadata.Ecma335.MetadataTokens.TypeDefinitionHandle(1));
            var owned = localMetadata.GetTypeDefinition(System.Reflection.Metadata.Ecma335.MetadataTokens.TypeDefinitionHandle(2));
            Check(global.GetMethods().Count == 2 && owned.GetMethods().Count == 1, "global and type method ranges");
            Check(localPe.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress == 0x06000002, "global entry token");
        }
        var badNames = new AssemblyBuilder(new("BadNames", new Version(1, 0, 0, 0)), app.CoreLibrary);
        var bad = badNames.AddFunction("\ud800", returnsValue: false); bad.Return();
        Reject(() => badNames.WriteNativeAssembly());
        var foreign = new AssemblyBuilder(app.Identity, app.CoreLibrary).AddFunction("Foreign");
        foreign.LoadConstant(1); foreign.Return();
        app.EntryPoint.ClearBody(); app.EntryPoint.Call(foreign); app.EntryPoint.Return(); Reject(() => app.WriteNativeAssembly());
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        runtime = Path.GetFullPath(runtime); output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("native-test output must be a fresh directory");
        Directory.CreateDirectory(output);
        var apiHash = Hash(typeof(AssemblyBuilder).Assembly.Location);
        var (app, library) = Build();
        var appPath = Path.Combine(output, "Application.neo.json");
        var libraryPath = Path.Combine(output, "Functions.neo.json");
        File.WriteAllBytes(appPath, app.WriteNativeAssembly());
        File.WriteAllBytes(libraryPath, library.WriteNativeAssembly());
        await RuntimeIntegration.Command(runtime, 0, "verify", appPath, "--module", libraryPath);
        var result = await RuntimeIntegration.Command(runtime, 42, "run", appPath, "--module", libraryPath, "--show-result");
        Check(result.Contains("=> Int32(42)", StringComparison.Ordinal), "direct runtime result");
        var missing = await RuntimeIntegration.Command(runtime, 1, "verify", appPath);
        Check(missing.Contains("missing referenced module", StringComparison.Ordinal), "missing dependency diagnostic");
        var changed = System.Text.Json.Nodes.JsonNode.Parse(File.ReadAllBytes(libraryPath))!;
        changed["revision"] = "2.0.0.0";
        var wrong = Path.Combine(output, "WrongRevision.neo.json"); File.WriteAllText(wrong, changed.ToJsonString());
        var mismatch = await RuntimeIntegration.Command(runtime, 1, "verify", appPath, "--module", wrong);
        Check(mismatch.Contains("module revision mismatch", StringComparison.Ordinal), "wrong revision diagnostic");
        File.WriteAllText(Path.Combine(output, "native-validation.json"), JsonSerializer.Serialize(new {
            date = "2026-09-30", scope = "Public .NET model -> native format-5 bytes -> neoCLR loader/verifier/VM; no CLI bridge",
            result = 42, directNativeLoading = true, directPeLoading = false,
            topLevelFunctions = true, typeOwnedMethods = true, crossAssemblyCall = true, noResult = true,
            missingDependencyRejected = true, wrongRevisionRejected = true,
            runtimeSha256 = Hash(runtime), libraryApiSha256 = apiHash, appSha256 = Hash(appPath), dependencySha256 = Hash(libraryPath)
        }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
        Console.WriteLine("PASS directly loaded API-produced native assemblies; top-level cross-assembly function result: 42");
    }
    private static string Hash(string path) => Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path))).ToLowerInvariant();
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid writer graph accepted");
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
}
