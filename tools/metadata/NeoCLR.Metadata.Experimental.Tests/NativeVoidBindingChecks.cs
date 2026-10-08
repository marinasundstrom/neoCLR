using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeVoidBindingChecks
{
    internal static void Run(string seedPath, string corePath)
    {
        var core = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(corePath), false);
        var seed = NativeLibraryDefinition.ReadAssembly(File.ReadAllBytes(seedPath));
        var declaration = core.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == "Void");
        var graph = new AssemblyBuilder(new("VoidRoundTrip", new Version(1, 0, 0, 0)), core.Identity);
        graph.BindNativeLibrary(core, seed, core.Identity);
        var value = graph.ImportReference(declaration, core.Identity);
        var echo = graph.AddFunction("Echo", new MethodSignature(value, [value]));
        echo.GetILGenerator().LoadArgument(0);
        echo.GetILGenerator().Return();
        var method = JsonNode.Parse(graph.WriteNativeAssembly())!["functions"]!.AsArray().Single()!;
        if (method["returns"]!.GetValue<string>() != "Void" || method["parameters"]![0]!.GetValue<string>() != "Void" ||
            method["no_result"]?.GetValue<bool>() == true)
            throw new Exception("inhabited Void lost its value result");
        var native = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
        var consumer = new AssemblyBuilder(new("VoidReimport", new Version(1, 0, 0, 0)), core.Identity);
        consumer.BindNativeLibrary(core, seed, core.Identity);
        var imported = consumer.ImportReference(native.MainModule.Functions.Single(m => m.Name == "Echo"), core.Identity, new Resolver(core));
        if (imported.Signature.ReturnType.ImportedType is not { Name: "Void", Namespace: "System", IsValueType: true } owner ||
            !owner.AssemblyIdentity.Equals(core.Identity) || imported.Signature.ParameterTypes[0] != imported.Signature.ReturnType)
            throw new Exception("inhabited Void lost exact core identity on reimport");
        var changed = JsonNode.Parse(seed.Declarations.GetRawText())!;
        changed["name"] = "WrongCore";
        var wrongSeed = NativeLibraryDefinition.ReadAssembly(NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(changed.ToJsonString())));
        var invalid = new AssemblyBuilder(new("WrongVoid", new Version(1, 0, 0, 0)), core.Identity);
        invalid.BindNativeLibrary(core, wrongSeed, core.Identity);
        try { invalid.ImportReference(declaration, core.Identity); }
        catch (InvalidDataException) { Console.WriteLine("PASS inhabited Void import, value result, native reimport and core binding rejection"); return; }
        throw new Exception("non-System binding admitted intrinsic Void");
    }

    private sealed class Resolver(AssemblyDefinition core) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => core.Identity.Equals(identity) ? core : null;
    }
}
