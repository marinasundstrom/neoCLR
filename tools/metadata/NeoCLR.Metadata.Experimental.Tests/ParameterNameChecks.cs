using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class ParameterNameChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("ParameterNames", new Version(1, 0, 0, 0)), new("Core", new Version(1, 0, 0, 0)));
        var owner = graph.AddGenericClass("Example", "Box", ["T"]);
        var method = owner.AddMethod("Echo", new(SignatureType.TypeParameter(0), [SignatureType.TypeParameter(0), PrimitiveType.Int32]));
        method.GetILGenerator().LoadArgument(0); method.GetILGenerator().Return();
        method.SetParameterName(0, "payload");
        method.Definition.SetParameterName(1, "ignored");
        method.SetParameterName(1, null);
        foreach (var native in new[] { false, true })
        {
            var image = native ? RuntimeAssemblyContainer.WriteBinary(graph) : graph.Write();
            var snapshot = native ? AssemblyDefinition.ReadNativeAssembly(image) : AssemblyDefinition.ReadAssembly(image, false);
            var loaded = snapshot.MainModule.Types.Single(t => t.Name == "Box`1").Methods.Single();
            Check(loaded.ParameterNames.Count == 1 && loaded.ParameterNames[0] == "payload", "name snapshot");
            if (native)
            {
                var context = new MetadataLoadContext([snapshot]);
                var type = context.Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Box`1");
                var parameters = type.GetMethods().Single().GetParameters();
                Check(parameters[0].Name == "payload" && parameters[1].Name is null, "facade names and missing name");
                var constructed = type.MakeGenericType(context.ResolveSignature(PrimitiveType.Int32)).GetMethods().Single().GetParameters();
                Check(constructed[0].Name == "payload" && ReferenceEquals(constructed[0].ParameterType, context.ResolveSignature(PrimitiveType.Int32)), "constructed parameter name and substituted type");
            }
            try { loaded.SetParameterName(0, "changed"); throw new Exception("mutable snapshot"); } catch (InvalidOperationException) { }
            if (native)
                Check(RuntimeAssemblyContainer.ReadCliProjection(image).MainModule.Types.Single(t => t.Name == "Box`1").Methods.Single().ParameterNames[0] == "payload", "projection name");
        }
        foreach (var index in new[] { -1, 2 })
            try { method.SetParameterName(index, "invalid"); throw new Exception("invalid position admitted"); } catch (ArgumentOutOfRangeException) { }
        foreach (var name in new[] { "", "bad\nname", new string('x', 1025) })
            try { method.SetParameterName(0, name); throw new Exception("invalid name admitted"); } catch (ArgumentException) { }
        var corrupt = JsonNode.Parse(graph.WriteNativeAssembly())!;
        corrupt["functions"]![0]!["parameter_names"] = new JsonArray("short");
        try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(corrupt.ToJsonString())); throw new Exception("misaligned names admitted"); } catch (InvalidDataException) { }
    }
    private static void Check(bool condition, string detail) { if (!condition) throw new Exception(detail); }
}
