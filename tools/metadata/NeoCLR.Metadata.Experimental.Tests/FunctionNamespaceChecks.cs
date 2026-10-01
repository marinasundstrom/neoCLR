using System.Diagnostics;
using System.Reflection;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class FunctionNamespaceChecks
{
    private static AssemblyBuilder Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("FunctionNamespaces", new Version(1, 0, 0, 0)), core);
        var first = graph.AddFunction("Library.First", "Value", new(PrimitiveType.Int32, []));
        first.LoadConstant(20); first.Return();
        var second = graph.AddFunction("Library.Second", "Value", new(PrimitiveType.Int32, []));
        second.LoadConstant(22); second.Return();
        var entry = graph.AddFunction("Main"); entry.Call(first); entry.Call(second); entry.Add(); entry.Return(); graph.EntryPoint = entry;
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        if (graph.Functions[0].Namespace != "Library.First" || graph.Functions[0].Name != "Value" || graph.Functions[0].DeclaringType is not null)
            throw new Exception("namespace/simple name/ownership lost");
        if (!Equals(Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null), 42)) throw new Exception("CLI execution of namespaced functions");
        var native = graph.WriteNativeAssembly();
        var projection = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(native).CreateReferenceAssembly(graph.CoreLibrary), false);
        if (projection.MainModule.Functions.Count != 3) throw new Exception("namespace projection added owner types");
        var foreign = new AssemblyBuilder(new("NamespaceConsumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        foreach (var definition in projection.MainModule.Functions.Where(f => f.Name != "Main"))
        {
            var imported = foreign.ImportReference(definition, graph.CoreLibrary);
            if (imported.Name != "Value" || imported.Namespace is not ("Library.First" or "Library.Second") || imported.DeclaringTypeName is not null)
                throw new Exception("namespace import lost");
            if (!ReferenceEquals(imported, foreign.ImportReference(definition, graph.CoreLibrary))) throw new Exception("import cache");
        }
        foreach (var invalid in new[] { "A..B", ".A", "A.", "A.\nB" })
        {
            try { graph.AddFunction(invalid, "Bad", new(PrimitiveType.Int32, [])); throw new Exception("invalid namespace accepted"); }
            catch (ArgumentException) { }
        }
        try { graph.AddFunction("Library.First", "Value", new(PrimitiveType.Int32, [])); throw new Exception("duplicate accepted"); }
        catch (ArgumentException) { }
        if (graph.Functions.Count != 3) throw new Exception("failed declaration mutated graph");
        var changed = JsonNode.Parse(native)!; changed["functions"]![0]!["namespace"] = "Other";
        try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(changed.ToJsonString())); throw new Exception("namespace/name mismatch accepted"); }
        catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var library = Create();
        var path = Save("Local", library);
        await Command(0, "verify", path); await Command(42, "run", path);
        library.EntryPoint = null;
        var dependency = Save("Library", library);
        var snapshot = RuntimeAssemblyContainer.ReadCliProjection(File.ReadAllBytes(dependency));
        var consumer = new AssemblyBuilder(new("NamespaceConsumer", new Version(1, 0, 0, 0)), library.CoreLibrary);
        var main = consumer.AddFunction("Main");
        foreach (var definition in snapshot.MainModule.Functions.Where(f => f.Name != "Main")) main.Call(consumer.ImportReference(definition, library.CoreLibrary));
        main.Add(); main.Return(); consumer.EntryPoint = main;
        var application = Save("Consumer", consumer);
        await Command(0, "verify", application, "--module", dependency); await Command(42, "run", application, "--module", dependency);
        Console.WriteLine("PASS namespaced ownerless functions: CLI, binary native, imported calls and collision isolation");
        string Save(string name, AssemblyBuilder graph)
        {
            var destination = Path.Combine(output, name + ".dll");
            File.WriteAllBytes(destination, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary)); return destination;
        }
        async Task Command(int expected, params string[] arguments)
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var arg in arguments) start.ArgumentList.Add(arg);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync();
            var text = await stdout + await stderr;
            if (process.ExitCode != expected) throw new Exception(text);
        }
    }
}
