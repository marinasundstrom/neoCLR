using System.Diagnostics;
using System.Text;
using System.Text.Json.Nodes;
using System.Text.Json;
using System.Security.Cryptography;
using NeoCLR.Metadata.Experimental;

internal static class NativeModuleChecks
{
    internal static void Run()
    {
        byte[] json = """{"format":5,"name":"Existing","types":[{"name":"Box","fields":[{"name":"Value","ty":"Int32"}]}],"functions":[],"revision":"unchanged"}"""u8.ToArray();
        var image = NativeModuleContainer.WriteBinary(json);
        Check(JsonNode.DeepEquals(JsonNode.Parse(json), JsonNode.Parse(NativeModuleContainer.Read(image))), "native values preserved");
        foreach (var invalid in new[] { "{}", "[]", "{\"format\":\"5\"}", "{\"format\":4,\"name\":\"X\",\"functions\":[]}", "{\"format\":5,\"name\":\"\",\"functions\":[]}", "{\"format\":5,\"name\":\"X\",\"functions\":{}}" })
            Reject(() => NativeModuleContainer.WriteBinary(Encoding.UTF8.GetBytes(invalid)));
        for (int length = 0; length < image.Length; length++)
        { int size = length; Reject(() => NativeModuleContainer.Read(image.AsSpan(0, size))); }
        var optional = MetadataEnvelope.Write([new MetadataSection(256, 2, false, [0])], new Dictionary<ushort, ushort> { [256] = 2 });
        Reject(() => NativeModuleContainer.Read(optional));
    }

    // Run from the neoCLR repository root. Exercise the actual class library, not a reduced fixture.
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Directory.CreateDirectory(directory);
        var systemJson = Path.Combine(directory, "System.neo.json");
        await Invoke("assemble", "runtime/System.neoil", systemJson);
        var system = Translate(systemJson);
        await Invoke("verify", system);
        foreach (var sample in new[] { "examples/hello.neoil", "examples/hello_functions.neoil" })
        {
            var original = await Invoke("run", sample, "--system", systemJson);
            Check(original.Trim() == "Hello, world!", sample + " expected output");
            Check(await Invoke("run", sample, "--system", system) == original, sample + " output unchanged");
            await Invoke("verify", sample, "--system", system);
        }
        var paths = new List<string>();
        foreach (var module in new[] { "models", "operations", "app" })
        {
            var json = Path.Combine(directory, module + ".neo.json");
            var arguments = new List<string> { "assemble", "examples/modules/" + module + ".neoil", json, "--system", system };
            foreach (var dependency in paths) { arguments.Add("--module"); arguments.Add(dependency); }
            await Invoke(arguments.ToArray());
            paths.Add(Translate(json));
        }
        var invocation = new[] { "run", paths[2], "--module", paths[1], "--module", paths[0], "--system", system };
        Check((await Invoke(invocation)).Trim() == "42", "binary application, generic dependencies and System execute");
        invocation[0] = "verify"; await Invoke(invocation);
        var invalidJson = JsonNode.Parse(File.ReadAllBytes(systemJson))!;
        invalidJson["functions"]![0]!["body"] = new JsonArray(new JsonObject { ["op"] = "future.op" });
        var invalid = Path.Combine(directory, "invalid.neox");
        File.WriteAllBytes(invalid, NativeModuleContainer.WriteBinary(Encoding.UTF8.GetBytes(invalidJson.ToJsonString())));
        await InvokeExpected(1, "verify", invalid);
        var systemBytes = File.ReadAllBytes(systemJson);
        var model = JsonNode.Parse(systemBytes)!;
        File.WriteAllText(Path.Combine(directory, "validation.json"), JsonSerializer.Serialize(new
        {
            producer = "current runtime/System.neoil manifest and existing generated IL",
            directRavenSourceEmission = false,
            cliReferenceProjection = false,
            systemTypes = model["types"]!.AsArray().Count,
            systemFunctions = model["functions"]!.AsArray().Count,
            jsonBytes = systemBytes.Length,
            binaryBytes = new FileInfo(system).Length,
            jsonSha256 = Convert.ToHexString(SHA256.HashData(systemBytes)).ToLowerInvariant(),
            binarySha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(system))).ToLowerInvariant(),
            completeJsonValuesPreserved = true,
            systemVerification = true,
            helloWorld = true,
            helloFunctionCall = true,
            genericBinaryDependencyChain = true,
            unknownBodyRejected = true
        }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
        Console.WriteLine("PASS translated System, both Hello cases, generic module chain and invalid-body rejection");

        string Translate(string path)
        {
            var json = File.ReadAllBytes(path);
            var encoded = NativeModuleContainer.WriteBinary(json);
            Check(JsonNode.DeepEquals(JsonNode.Parse(json), JsonNode.Parse(NativeModuleContainer.Read(encoded))), path + " complete value preservation");
            var output = Path.ChangeExtension(path, ".neox"); File.WriteAllBytes(output, encoded);
            Console.WriteLine($"Translated {path}: {json.Length} JSON bytes -> {encoded.Length} binary bytes");
            return output;
        }
        Task<string> Invoke(params string[] arguments) => InvokeExpected(0, arguments);
        async Task<string> InvokeExpected(int expected, params string[] arguments)
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in arguments) start.ArgumentList.Add(argument);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
            try { await process.WaitForExitAsync(timeout.Token); }
            catch (OperationCanceledException) { process.Kill(true); throw; }
            var output = await stdout; var error = await stderr;
            Check(process.ExitCode == expected, string.Join(' ', arguments) + ": " + output + error);
            return output.Replace("\r\n", "\n");
        }
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid native module container accepted");
    }
}
