using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class SourceAttributeChecks
{
    internal static void Check(string path)
    {
        var assembly = AssemblyDefinition.ReadNativeAssembly(File.ReadAllBytes(path));
        var views = new MetadataLoadContext([assembly]).Resolve(assembly.Identity).GetTypes();
        var root = views.Single(t => t.Namespace == "System" && t.Name == "Object");
        var attribute = views.Single(t => t.Namespace == "System" && t.Name == "Attribute");
        var marker = views.Single(t => t.Namespace == "System.Runtime.CompilerServices" && t.Name == "UnionAttribute");
        var union = views.Single(t => t.Name == "Choice");
        if (!attribute.IsAbstract || !ReferenceEquals(attribute.BaseType, root) || !ReferenceEquals(marker.BaseType, attribute))
            throw new Exception("Source attribute inheritance was lost");
        if (!ReferenceEquals(union.GetCustomAttributes().Single(a => a.Name == "UnionAttribute").GetAttributeType(), marker))
            throw new Exception("Union metadata does not use the canonical source marker");
    }
}
