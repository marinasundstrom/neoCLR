using System.Diagnostics;
using System.Reflection;
using System.Text.Json;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;
using MethodBuilder = NeoCLR.Metadata.Experimental.Model.MethodBuilder;

internal static class FunctionVisibilityChecks
{
    private static (AssemblyBuilder Graph, MethodBuilder Hidden) Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("FunctionVisibility", new Version(1, 0, 0, 0)), core);
        var hidden = graph.AddFunction("Hidden", new(PrimitiveType.Int32, []), MethodVisibility.Internal);
        hidden.LoadConstant(42); hidden.Return();
        var main = graph.AddFunction("Main"); main.Call(hidden); main.Return(); graph.EntryPoint = main;
        return (graph, hidden);
    }

    internal static void Run()
    {
        var (graph, hidden) = Create();
        if (hidden.DeclaringType is not null || hidden.Visibility != MethodVisibility.Internal || graph.EntryPoint!.Visibility != MethodVisibility.Public)
            throw new Exception("assembly function ownership/access");
        var cli = graph.Write();
        if (!Equals(Assembly.Load(cli).EntryPoint!.Invoke(null, null), 42)) throw new Exception("same-assembly internal function");
        foreach (var image in new[] { cli, NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary) })
        {
            var method = AssemblyDefinition.ReadAssembly(image, false).MainModule.Methods.Single(m => m.Name == "Hidden");
            if (((MethodAttributes)method.Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.Assembly)
                throw new Exception("global function access projection");
        }
        foreach (var invalid in new[] { MethodVisibility.Private, (MethodVisibility)999 })
        {
            try { graph.AddFunction("Bad", new(PrimitiveType.Int32, []), invalid); throw new Exception("invalid function visibility accepted"); }
            catch (ArgumentOutOfRangeException) { }
        }
        if (graph.Functions.Count != 2) throw new Exception("rejection mutated graph");
        var json = JsonNode.Parse(graph.WriteNativeAssembly())!;
        json["functions"]![0]!["visibility"] = "private";
        json["functions"]![0]!["origin"]!["member_access"] = "Private";
        try { NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(json.ToJsonString())); throw new Exception("private global accepted"); }
        catch (InvalidDataException) { }
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        if (Directory.Exists(directory)) throw new IOException("output directory must be fresh");
        Directory.CreateDirectory(directory);
        var (graph, hidden) = Create();
        var valid = Save("Valid", graph);
        await Command(0, "verify", valid); await Command(42, "run", valid);
        graph.EntryPoint = hidden;
        var internalEntry = Save("InternalEntry", graph);
        await Command(0, "verify", internalEntry); await Command(42, "run", internalEntry);
        graph.EntryPoint = null;
        var library = Save("Library", graph);
        var foreign = new AssemblyBuilder(new("ForeignFunction", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var main = foreign.AddFunction("Main"); main.Call(hidden); main.Return(); foreign.EntryPoint = main;
        if (!(await Command(1, "verify", Save("Denied", foreign), "--module", library)).Contains("method access denied"))
            throw new Exception("external internal function accepted");
        main.ClearBody(); main.Call(graph.Functions.Single(m => m.Name == "Main")); main.Return();
        var external = Save("External", foreign);
        await Command(0, "verify", external, "--module", library); await Command(42, "run", external, "--module", library);
        File.WriteAllText(Path.Combine(directory, "validation.json"), JsonSerializer.Serialize(new {
            date = "2026-10-01", producer = "independent metadata API", internalFunctionResult = 42,
            internalEntryAccepted = true, externalInternalRejected = true, publicFacadeAccepted = true,
            runtimeSha256 = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(runtime))).ToLowerInvariant()
        }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
        Console.WriteLine("PASS API function visibility -> binary runtime load and access enforcement");
        string Save(string name, AssemblyBuilder assembly)
        {
            var path = Path.Combine(directory, name + ".dll");
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(assembly.WriteNativeAssembly(), assembly.CoreLibrary)); return path;
        }
        async Task<string> Command(int expected, params string[] arguments)
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
            foreach (var argument in arguments) start.ArgumentList.Add(argument);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != expected) throw new Exception(string.Join(" ", arguments) + ": " + text);
            return text;
        }
    }
}
