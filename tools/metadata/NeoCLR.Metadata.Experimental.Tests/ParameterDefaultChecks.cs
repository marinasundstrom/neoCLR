using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class ParameterDefaultChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("Defaults", new Version(1, 0, 0, 0)), new("Core", new Version(1, 0, 0, 0)));
        var token = graph.AddValueType("Example", "Token");
        token.AddField("Requested", PrimitiveType.Boolean);
        var method = graph.AddClass("Example", "Defaults").AddMethod("Read",
            new(PrimitiveType.Int32, [PrimitiveType.Int32, PrimitiveType.Boolean, PrimitiveType.String, PrimitiveType.String, PrimitiveType.Double, token]));
        method.GetILGenerator().LoadArgument(0); method.GetILGenerator().Return();
        method.SetParameterDefault(0, new(42));
        method.SetParameterDefault(1, new(false));
        method.SetParameterDefault(2, new("héllo"));
        method.SetParameterDefault(3, new(null));
        method.SetParameterDefault(4, new(-0.0d));
        method.SetParameterDefault(5, new(null));
        foreach (var snapshot in new[] {
            AssemblyDefinition.ReadAssembly(graph.Write(), false),
            AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph)),
            AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false) })
        {
            var loaded = snapshot.MainModule.Types.Single(t => t.Name == "Defaults").Methods.Single();
            Check(loaded.ParameterDefaults.Count == 6, "default presence lost");
            Check(Equals(loaded.ParameterDefaults[0].Value, 42), "integer default lost");
            Check(Equals(loaded.ParameterDefaults[1].Value, false), "false default lost");
            Check(Equals(loaded.ParameterDefaults[2].Value, "héllo"), "text default lost");
            Check(loaded.ParameterDefaults[3].Value is null, "explicit null default lost");
            Check(BitConverter.DoubleToUInt64Bits((double)loaded.ParameterDefaults[4].Value!) == 0x8000000000000000, "negative zero bits lost");
            Check(loaded.ParameterDefaults[5].Value is null, "value-type default lost");
            try { loaded.SetParameterDefault(0, null); throw new Exception("mutable snapshot"); } catch (InvalidOperationException) { }
        }
        var context = new MetadataLoadContext([AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph))]);
        var parameters = context.Resolve(graph.Identity).GetTypes().Single(t => t.Name == "Defaults").GetMethods().Single().GetParameters();
        Check(parameters[3].HasDefaultValue && parameters[3].DefaultValue is null, "facade null default");
        method.SetParameterDefault(3, null);
        Check(!method.Definition.ParameterDefaults.ContainsKey(3), "clear default");
        try { method.SetParameterDefault(0, new("wrong")); throw new Exception("mismatched default accepted"); } catch (InvalidDataException) { }
        var corrupt = JsonNode.Parse(graph.WriteNativeAssembly())!;
        corrupt["functions"]![0]!["parameter_defaults"] = new JsonArray();
        Check(NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(corrupt.ToJsonString())).CreateReferenceAssembly(graph.CoreLibrary).Length > 0, "empty defaults should be absent");
        corrupt["functions"]![0]!["parameter_defaults"] = new JsonArray((JsonNode?)null);
        try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(corrupt.ToJsonString())); throw new Exception("misaligned defaults accepted"); } catch (InvalidDataException) { }
        Console.WriteLine("PASS parameter defaults: native/CLI/projection roundtrip, facade, explicit null/value-type default, exact float bits and malformed rejection");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
}
