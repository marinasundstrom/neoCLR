using System.Diagnostics;
using System.Reflection;
using System.Text.Json;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;
using MethodBuilder = NeoCLR.Metadata.Experimental.Model.MethodBuilder;

internal static class MethodVisibilityChecks
{
    private static (AssemblyBuilder Graph, MethodBuilder Hidden, MethodBuilder Internal) Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("MethodVisibility", new Version(1, 0, 0, 0)), core);
        var secrets = graph.AddType("", "Secrets");
        var hidden = secrets.AddMethod("Hidden", new(PrimitiveType.Int32, []), MethodVisibility.Private);
        hidden.LoadConstant(20); hidden.Return();
        var internalMethod = secrets.AddMethod("Internal", new(PrimitiveType.Int32, []), MethodVisibility.Internal);
        internalMethod.Call(hidden); internalMethod.LoadConstant(1); internalMethod.Add(); internalMethod.Return();
        var main = graph.AddType("", "Caller").AddMethod("Main");
        main.Call(internalMethod); main.LoadConstant(2); main.Multiply(); main.Return(); graph.EntryPoint = main;
        return (graph, hidden, internalMethod);
    }

    internal static void Run()
    {
        var (graph, hidden, internalMethod) = Create();
        if (hidden.Visibility != MethodVisibility.Private || internalMethod.Visibility != MethodVisibility.Internal || graph.EntryPoint!.Visibility != MethodVisibility.Public)
            throw new Exception("method builder visibility");
        var cli = graph.Write();
        if (!Equals(Assembly.Load(cli).EntryPoint!.Invoke(null, null), 42)) throw new Exception("same-assembly private/internal call");
        Check(cli);
        var native = graph.WriteNativeAssembly();
        Check(NativeAssemblyDefinition.ReadAssembly(native).CreateReferenceAssembly(graph.CoreLibrary));
        var json = JsonNode.Parse(native)!;
        json["functions"]![0]!["origin"]!["member_access"] = "Public"; Reject(json);
        json["functions"]![0]!["origin"]!["member_access"] = "Private";
        json["functions"]![0]!["visibility"] = "protected"; Reject(json);
        json["functions"]![0]!.AsObject().Remove("visibility"); Reject(json);
        try { graph.Types[0].AddMethod("Bad", new(PrimitiveType.Int32, []), (MethodVisibility)999); throw new Exception("invalid visibility accepted"); }
        catch (ArgumentOutOfRangeException) { }
    }

    private static void Check(byte[] bytes)
    {
        var snapshot = AssemblyDefinition.ReadAssembly(bytes, false);
        foreach (var (name, access) in new[] { ("Hidden", MethodAttributes.Private), ("Internal", MethodAttributes.Assembly), ("Main", MethodAttributes.Public) })
            if (((MethodAttributes)snapshot.MainModule.Methods.Single(m => m.Name == name).Attributes & MethodAttributes.MemberAccessMask) != access)
                throw new Exception("method visibility projection");
    }

    private static void Reject(JsonNode json)
    {
        try { NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(json.ToJsonString())); throw new Exception("inconsistent method visibility accepted"); }
        catch (InvalidDataException) { }
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        if (Directory.Exists(directory)) throw new IOException("output directory must be fresh");
        Directory.CreateDirectory(directory);
        var (graph, hidden, internalMethod) = Create();
        var valid = Save("Valid", graph);
        await Command(0, "verify", valid); await Command(42, "run", valid);
        graph.EntryPoint = null;
        var library = Save("Library", graph);
        foreach (var target in new[] { hidden, internalMethod })
        {
            var foreign = new AssemblyBuilder(new("Foreign" + target.Name, new Version(1, 0, 0, 0)), graph.CoreLibrary);
            var main = foreign.AddFunction("Main"); main.Call(target); main.Return(); foreign.EntryPoint = main;
            var path = Save("Foreign" + target.Name, foreign);
            if (!(await Command(1, "verify", path, "--module", library)).Contains("method access denied")) throw new Exception("external access not rejected");
        }
        var bad = graph.AddType("", "Other").AddMethod("Bad"); bad.Call(hidden); bad.Return(); graph.EntryPoint = bad;
        if (!(await Command(1, "verify", Save("WrongOwner", graph))).Contains("method access denied")) throw new Exception("private owner access not rejected");
        File.WriteAllText(Path.Combine(directory, "validation.json"), JsonSerializer.Serialize(new {
            date = "2026-10-01", producer = "independent metadata API", sameAssemblyInternalAndSameTypePrivateResult = 42,
            externalInternalAndPrivateRejected = true, privateWrongOwnerRejected = true,
            runtimeSha256 = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(runtime))).ToLowerInvariant()
        }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
        Console.WriteLine("PASS API method visibility -> binary runtime load and access enforcement");

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
