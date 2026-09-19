using Mono.Cecil;
using System.Text.Json;

// Descriptive metadata only: executable binding continues to use checked signatures.
static class SourceMetadata
{
    static string Reference(AssemblyNameReference reference) => reference.Name switch {
        CoreDeclarations.Identity or "mscorlib" => "System.Runtime",
        _ => reference.FullName
    };
    public static string Assembly(ModuleDefinition module) => ".assembly " + JsonSerializer.Serialize(new {
        name = module.Assembly.Name.Name,
        full_name = module.Assembly.Name.FullName,
        modules = module.Assembly.Modules.Select(m => m.Name).ToArray(),
        references = module.Assembly.Modules.SelectMany(m => m.AssemblyReferences).Select(Reference).Distinct().ToArray()
    });
    public static string Type(TypeDefinition type) => ".origin " + JsonSerializer.Serialize(new {
        assembly = type.Module.Assembly.Name.FullName,
        module = type.Module.Name,
        name = type.FullName,
        token = type.MetadataToken.ToUInt32(),
        declaring_type_token = type.DeclaringType is { } parent ? (uint?)parent.MetadataToken.ToUInt32() : null,
        field_tokens = type.Fields.Select(f => f.MetadataToken.ToUInt32()).ToArray(),
        // Only implementation-owned properties currently have executable accessors
        // projected by this importer; application properties are still omitted.
        property_tokens = Array.Empty<uint>()
    });
    public static string Method(MethodDefinition method, bool explicitReceiver = false) => ".origin " + JsonSerializer.Serialize(new {
        assembly = method.Module.Assembly.Name.FullName,
        module = method.Module.Name,
        name = method.Name,
        token = method.MetadataToken.ToUInt32(),
        parameter_tokens = (explicitReceiver ? new uint[] { 0 } : Array.Empty<uint>())
            .Concat(method.Parameters.Select(p => p.MetadataToken.RID == 0 ? 0U : p.MetadataToken.ToUInt32())).ToArray()
    });
}
