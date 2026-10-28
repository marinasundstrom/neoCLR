using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;
using System.Text;
using System.Text.Json.Nodes;

internal static class GenericAttributeChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("GenericAttributes", new(1, 0, 0, 0)), core);
        var attribute = graph.Definition.MainModule.ImportReference(core, "Tests", "MarkerAttribute");
        CustomAttributeDefinition Marker(string value) => new(attribute, [new(PrimitiveType.String, value)]);
        var owner = graph.AddGenericClass("Tests", "Holder", ["T"]);
        owner.Definition.GetGenericParameterCustomAttributes(0).Add(Marker("type"));
        var method = owner.AddMethod("Read", new(PrimitiveType.Int32, [], ["U"]));
        method.Definition.GetGenericParameterCustomAttributes(0).Add(Marker("method"));
        method.LoadConstant(42); method.Return();
        var binary = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        foreach (var assembly in new[] { AssemblyDefinition.ReadAssembly(graph.Write(), false), AssemblyDefinition.ReadNativeAssembly(binary), RuntimeAssemblyContainer.ReadCliProjection(binary) })
        {
            var definition = assembly.MainModule.Types.Single(t => t.Name == "Holder`1");
            Check(definition.GetGenericParameterCustomAttributes(0).Single().GetArguments().Single().Value is "type", "type annotation lost");
            Check(definition.Methods.Single().GetGenericParameterCustomAttributes(0).Single().GetArguments().Single().Value is "method", "method annotation lost");
            try { definition.GetGenericParameterCustomAttributes(0).Add(Marker("bad")); throw new Exception("loaded generic attributes mutable"); } catch (NotSupportedException) { }
            var view = new MetadataLoadContext([assembly]).Resolve(assembly.Identity).GetTypes().Single(t => t.Name == "Holder`1");
            Check(((GenericParameterTypeInfo)view.GetGenericArguments().Single()).GetCustomAttributes().Single().GetArguments().Single().Value is "type", "type view lost annotation");
            Check(((MethodGenericParameterTypeInfo)view.GetMethods().Single().GetGenericArguments().Single()).GetCustomAttributes().Single().GetArguments().Single().Value is "method", "method view lost annotation");
        }
        foreach (var corruption in new[] { "zero", "table", "duplicate", "count" })
        {
            var bad = JsonNode.Parse(graph.WriteNativeAssembly())!;
            var typeTokens = bad["types"]![0]!["origin"]!["generic_parameter_tokens"]!.AsArray();
            var methodTokens = bad["functions"]![0]!["origin"]!["generic_parameter_tokens"]!.AsArray();
            if (corruption == "zero") typeTokens[0] = 0;
            if (corruption == "table") typeTokens[0] = 0x08000001;
            if (corruption == "duplicate") methodTokens[0] = typeTokens[0]!.DeepClone();
            if (corruption == "count") typeTokens.Clear();
            try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(bad.ToJsonString())); throw new Exception("invalid generic identity accepted: " + corruption); } catch (InvalidDataException) { }
        }
        Console.WriteLine("PASS generic parameter attributes: type/method native/CLI roundtrip, projection, introspection and malformed identities");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
}
