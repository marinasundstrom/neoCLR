using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text.Json;

// Read metadata only: never load or execute the inspected library.
using var stream = File.OpenRead(args[0]);
using var pe = new PEReader(stream);
var reader = pe.GetMetadataReader();
bool IsPublic(TypeDefinitionHandle handle)
{
    var type = reader.GetTypeDefinition(handle);
    return (type.Attributes & TypeAttributes.VisibilityMask) switch
    {
        TypeAttributes.Public => true,
        TypeAttributes.NestedPublic => IsPublic(type.GetDeclaringType()),
        _ => false
    };
}
string Name(TypeDefinitionHandle handle)
{
    var type = reader.GetTypeDefinition(handle);
    var parent = type.GetDeclaringType();
    var prefix = parent.IsNil ? reader.GetString(type.Namespace) : Name(parent);
    return (prefix.Length == 0 ? "" : prefix + ".") + reader.GetString(type.Name);
}
Console.WriteLine(JsonSerializer.Serialize(reader.TypeDefinitions.Where(IsPublic)
    .Select(Name).Order(StringComparer.Ordinal).ToArray()));
