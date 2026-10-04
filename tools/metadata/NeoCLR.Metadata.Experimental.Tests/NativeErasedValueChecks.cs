using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeErasedValueChecks
{
    internal static void Run(string seedPath, string corePath)
    {
        var core = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(corePath), false);
        var seed = NativeLibraryDefinition.ReadAssembly(File.ReadAllBytes(seedPath));
        var graph = new AssemblyBuilder(new("ErasedRoundTrip", new Version(1, 0, 0, 0)), core.Identity);
        graph.BindNativeLibrary(core, seed, core.Identity);
        var value = graph.ImportReference(core.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == "Value"), core.Identity);
        var echo = graph.AddFunction("Echo", new MethodSignature(value, [value]));
        echo.GetILGenerator().LoadArgument(0);
        echo.GetILGenerator().Return();
        var nativeBytes = graph.WriteNativeAssembly();
        var json = JsonNode.Parse(nativeBytes)!;
        var method = json["functions"]!.AsArray().Single()!;
        if (method["returns"]!.GetValue<string>() != "Value" || method["parameters"]![0]!.GetValue<string>() != "Value")
            throw new Exception("erased Value must not be encoded as an ordinary struct or Object");
        var image = RuntimeAssemblyContainer.WriteBinary(graph);
        var native = AssemblyDefinition.ReadNativeAssembly(image);
        var reader = native.MainModule.Functions.Single(m => m.Name == "Echo");
        var consumer = new AssemblyBuilder(new("ErasedReimport", new Version(1, 0, 0, 0)), core.Identity);
        consumer.BindNativeLibrary(core, seed, core.Identity);
        var imported = consumer.ImportReference(reader, core.Identity, new Resolver(core));
        if (imported.Signature.ReturnType.ImportedType is not { Namespace: "System", Name: "Value", IsValueType: true } owner ||
            !owner.AssemblyIdentity.Equals(core.Identity) || imported.Signature.ParameterTypes[0] != imported.Signature.ReturnType)
            throw new Exception("erased Value lost exact core identity on reimport");
        Reject(() => NativeAssemblyDefinition.ReadAssembly(nativeBytes).CreateReferenceAssembly(new("WrongCore", new Version(1, 0, 0, 0))));
        foreach (var invalid in new[] { "missing", "namespace", "category" })
        {
            var changed = JsonNode.Parse(nativeBytes)!;
            var aliases = changed["assemblies"]![0]!["native_type_bindings"]!.AsArray();
            var alias = aliases.Single(a => a!["native_name"]!.GetValue<string>() == "System.Value")!;
            if (invalid == "missing") aliases.Remove(alias);
            if (invalid == "namespace") alias["namespace"] = "Wrong";
            if (invalid == "category") alias["value_type"] = false;
            Reject(() => NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(changed.ToJsonString())));
        }
        Console.WriteLine("PASS erased Value signatures, native reimport and exact core alias validation");
    }

    private sealed class Resolver(AssemblyDefinition core) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => core.Identity.Equals(identity) ? core : null;
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("expected erased Value identity rejection");
    }
}
