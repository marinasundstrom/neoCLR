using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class StorageAttributeChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("StorageAttributes", new(1, 0, 0, 0)), core);
        var attribute = graph.Definition.MainModule.ImportReference(core, "Tests", "MarkerAttribute");
        CustomAttributeDefinition Marker(string value) => new(attribute, [new(PrimitiveType.String, value)]);
        var owner = graph.AddClass("Tests", "Storage");
        owner.AddField("Instance", PrimitiveType.Int32).Definition.CustomAttributes.Add(Marker("instance"));
        owner.AddField("Static", PrimitiveType.Int32, isStatic: true).Definition.CustomAttributes.Add(Marker("static"));
        var getter = owner.AddMethod("get_Value", new(PrimitiveType.Int32, []));
        getter.LoadConstant(42); getter.Return();
        owner.AddProperty("Value", PrimitiveType.Int32, getter, null).Definition.CustomAttributes.Add(Marker("property"));
        var binary = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        foreach (var assembly in new[] { AssemblyDefinition.ReadAssembly(graph.Write(), false), AssemblyDefinition.ReadNativeAssembly(binary), RuntimeAssemblyContainer.ReadCliProjection(binary) })
        {
            var view = new MetadataLoadContext([assembly]).Resolve(assembly.Identity).GetTypes().Single(t => t.Name == "Storage");
            foreach (var field in view.GetFields())
                Check(field.GetCustomAttributes().Single().GetArguments().Single().Value is string value && value == (field.IsStatic ? "static" : "instance"), "field target changed");
            Check(view.GetProperties().Single().GetCustomAttributes().Single().GetArguments().Single().Value is "property", "property annotation lost");
            Check(view.GetMethods().Single().GetCustomAttributes().Count == 0, "property annotation leaked onto getter");
        }
        Console.WriteLine("PASS instance/static field and property attributes: native/CLI roundtrip, projection and introspection");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
}
