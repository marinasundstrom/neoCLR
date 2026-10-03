using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental.Model;

internal static class ArrayBackingChecks
{
    static AssemblyBuilder Create()
    {
        var host = typeof(object).Assembly.GetName();
        var graph = new AssemblyBuilder(new("ArrayBacking", new Version(1, 0, 0, 0)),
            new(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!)));
        var shape = graph.AddGenericClass("Example", "Vector", ["T"]);
        shape.AddField("storage", SignatureType.ArrayOf(SignatureType.TypeParameter(0)));
        var main = graph.AddFunction("Main"); main.LoadConstant(42); main.Return(); graph.EntryPoint = main;
        graph.SetArrayBacking(shape);
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var shape = graph.Types[0];
        graph.SetArrayBacking(shape);
        var bytes = graph.WriteNativeAssembly();
        _ = NativeAssemblyDefinition.ReadAssembly(bytes);
        var root = JsonNode.Parse(bytes)!;
        var selection = root["assemblies"]![0]!["array_backing"]!;
        if (selection["index"]!.GetValue<int>() != 0) throw new Exception("wrong backing identity");
        void Reject(Action action)
        {
            try { action(); } catch (Exception e) when (e is ArgumentException or InvalidOperationException or InvalidDataException) { return; }
            throw new Exception("invalid array backing accepted");
        }
        selection["revision"] = "wrong";
        Reject(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(root.ToJsonString())));
        root = JsonNode.Parse(bytes)!;
        root["types"]![0]!["fields"]![0]!["visibility"] = "public";
        Reject(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(root.ToJsonString())));
        var foreign = new AssemblyBuilder(new("Foreign", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        Reject(() => foreign.SetArrayBacking(shape));
        var other = graph.AddGenericClass("Example", "Other", ["T"]);
        other.AddField("storage", SignatureType.ArrayOf(SignatureType.TypeParameter(0)));
        Reject(() => graph.SetArrayBacking(other));
        var invalid = graph.AddGenericClass("Example", "Invalid", ["T"]);
        invalid.AddField("storage", PrimitiveType.Int32);
        Reject(() => graph.SetArrayBacking(invalid));
        shape.AddField("extra", PrimitiveType.Int32);
        Reject(() => graph.WriteNativeAssembly());
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run(); Directory.CreateDirectory(output);
        foreach (var mode in new[] { "valid", "missing", "storage", "duplicate" })
        {
            var graph = Create();
            var root = JsonNode.Parse(graph.WriteNativeAssembly())!;
            if (mode == "missing") root["assemblies"]![0]!["array_backing"]!["index"] = 999;
            if (mode == "storage") root["types"]![0]!["fields"]![0]!["ty"] = "Int32";
            if (mode == "duplicate") root["assemblies"]!.AsArray().Add(root["assemblies"]![0]!.DeepClone());
            var path = Path.Combine(output, mode + ".json");
            File.WriteAllText(path, root.ToJsonString());
            var start = new System.Diagnostics.ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add("verify"); start.ArgumentList.Add(path);
            using var process = System.Diagnostics.Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (mode == "valid" ? process.ExitCode != 0 : process.ExitCode == 0 || !text.Contains("nominal array backing"))
                throw new Exception(mode + ": " + text);
        }
        Console.WriteLine("PASS nominal backing validation: valid, missing, storage, duplicate");
    }

}
