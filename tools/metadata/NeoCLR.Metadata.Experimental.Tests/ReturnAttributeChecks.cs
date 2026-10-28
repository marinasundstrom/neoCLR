using System.Runtime.Loader;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class ReturnAttributeChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("ReturnAttributes", new(1, 0, 0, 0)), core);
        var marker = new CustomAttributeDefinition(graph.Definition.MainModule.ImportReference(core, "System", "ObsoleteAttribute"), [new(PrimitiveType.String, "return contract")]);
        var method = graph.AddClass("Tests", "Factory").AddMethod("Read", new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        method.LoadArgument(0); method.Return();
        method.Definition.ReturnCustomAttributes.Add(marker);
        method.Definition.GetParameterCustomAttributes(0).Add(new(marker.AttributeType, [new(PrimitiveType.String, "parameter contract")]));
        var cli = graph.Write();
        var context = new AssemblyLoadContext("return-attributes", isCollectible: true);
        try
        {
            var reflected = context.LoadFromStream(new MemoryStream(cli)).GetType("Tests.Factory")!.GetMethod("Read")!;
            Check(reflected.ReturnParameter.GetCustomAttributesData().Single().ConstructorArguments.Single().Value is "return contract", "CLR return target changed");
            Check(reflected.GetParameters().Single().GetCustomAttributesData().Single().ConstructorArguments.Single().Value is "parameter contract", "CLR parameter target changed");
            Check((int)reflected.Invoke(null, [42])! == 42, "method behavior changed");
        }
        finally { context.Unload(); }
        // Native dependencies deliberately use unsigned identities; CLI execution
        // above uses the real signed host core.
        var nativeCore = new AssemblyIdentity("Core", new(1, 0, 0, 0));
        graph = new AssemblyBuilder(new("NativeReturnAttributes", new(1, 0, 0, 0)), nativeCore);
        marker = new CustomAttributeDefinition(graph.Definition.MainModule.ImportReference(nativeCore, "System", "ObsoleteAttribute"), [new(PrimitiveType.String, "return contract")]);
        method = graph.AddClass("Tests", "Factory").AddMethod("Read", new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        method.LoadArgument(0); method.Return();
        method.Definition.ReturnCustomAttributes.Add(marker);
        method.Definition.GetParameterCustomAttributes(0).Add(new(marker.AttributeType, [new(PrimitiveType.String, "parameter contract")]));
        var json = graph.WriteNativeAssembly();
        var binary = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        foreach (var assembly in new[] { AssemblyDefinition.ReadAssembly(cli, false), AssemblyDefinition.ReadNativeAssembly(binary), RuntimeAssemblyContainer.ReadCliProjection(binary) })
        {
            var loaded = assembly.MainModule.Types.Single(t => t.Name == "Factory").Methods.Single();
            Check(loaded.ReturnCustomAttributes.Single().GetArguments().Single().Value is "return contract", "return annotation lost");
            Check(loaded.GetParameterCustomAttributes(0).Single().GetArguments().Single().Value is "parameter contract", "parameter annotation lost");
            Check(loaded.CustomAttributes.Count == 0, "return annotation leaked to method");
            try { loaded.ReturnCustomAttributes.Add(marker); throw new Exception("loaded return attributes mutable"); }
            catch (NotSupportedException) { }
            var view = new MetadataLoadContext([assembly]).Resolve(assembly.Identity).GetTypes().Single(t => t.Name == "Factory").GetMethods().Single();
            Check(view.GetReturnCustomAttributes().Single().GetArguments().Single().Value is "return contract", "introspection return annotation lost");
        }
        foreach (var corruption in new[] { "zero", "table", "duplicate", "missing" })
        {
            var bad = JsonNode.Parse(json)!;
            var function = bad["functions"]!.AsArray().Single(f => f!["origin"]!["name"]!.GetValue<string>() == "Read")!;
            var origin = function["origin"]!;
            if (corruption == "zero") origin["return_token"] = 0;
            if (corruption == "table") origin["return_token"] = 0x06000001;
            if (corruption == "duplicate") origin["return_token"] = origin["parameter_tokens"]![0]!.DeepClone();
            if (corruption == "missing") origin.AsObject().Remove("return_token");
            try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(bad.ToJsonString())); throw new Exception("invalid return token accepted: " + corruption); }
            catch (InvalidDataException) { }
        }
        Console.WriteLine("PASS return attributes: CLI reflection, native roundtrip, projection, introspection and malformed targets");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
}
