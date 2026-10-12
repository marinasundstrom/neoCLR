using System.Reflection;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class LocalBudgetChecks
{
    internal static void Run(string output)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("LocalBudget", new Version(1, 0, 0, 0)), core);
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var il = main.GetILGenerator();
        for (var i = 0; i < 1024; i++) il.DeclareLocal(PrimitiveType.Int32);
        il.LoadConstant(42); il.Emit(OpCode.Stloc, 1023); il.Emit(OpCode.Ldloc, 1023); il.Return();
        var context = new AssemblyLoadContext("local-budget", true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(graph.Write()));
            if (!Equals(42, assembly.EntryPoint!.Invoke(null, null))) throw new Exception("high local slot lost");
        }
        finally { context.Unload(); }
        var json = graph.WriteNativeAssembly();
        var binary = RuntimeAssemblyContainer.WriteBinary(json, core);
        _ = AssemblyDefinition.ReadNativeAssembly(binary);
        File.WriteAllBytes(output, binary);
        var image = System.Text.Json.Nodes.JsonNode.Parse(json)!;
        var locals = image["functions"]![0]!["locals"]!.AsArray();
        locals.Add(locals[0]!.DeepClone());
        try {
            _ = NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(image.ToJsonString()));
            throw new Exception("reader accepted an oversized local table");
        } catch (InvalidDataException) { }
        var owner = graph.AddClass("Example", "Owner");
        foreach (var declare in new Action[] {
            () => il.DeclareLocal(PrimitiveType.Int32),
            () => il.DeclareLocal((SignatureType)PrimitiveType.Int32),
            () => il.DeclareLocal(owner) })
        {
            try { declare(); throw new Exception("local quota not enforced"); }
            catch (InvalidDataException) { }
            if (il.Locals.Count != 1024) throw new Exception("failed local declaration mutated body");
        }
        Console.WriteLine("PASS 1024 locals: CLI execution, native roundtrip and quota rejection");
    }
}
