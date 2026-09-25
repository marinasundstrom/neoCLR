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
    static bool PublicType(TypeDefinition type) => (type.IsPublic || type.IsNestedPublic)
        && (type.DeclaringType is null || PublicType(type.DeclaringType));
    public static string Type(TypeDefinition type) => ".origin " + JsonSerializer.Serialize(new {
        assembly = type.Module.Assembly.Name.FullName,
        module = type.Module.Name,
        name = type.FullName,
        token = type.MetadataToken.ToUInt32(),
        publicly_visible = PublicType(type),
        declaring_type_token = type.DeclaringType is { } parent ? (uint?)parent.MetadataToken.ToUInt32() : null,
        field_tokens = type.Fields.Select(f => f.MetadataToken.ToUInt32()).ToArray(),
        property_tokens = ApplicationTypes.RuntimeProperties(type).Select(p => p.MetadataToken.ToUInt32()).ToArray()
    });
    public static string Method(MethodDefinition method, bool explicitReceiver = false) => ".origin " + JsonSerializer.Serialize(new {
        assembly = method.Module.Assembly.Name.FullName,
        module = method.Module.Name,
        name = method.Name,
        token = method.MetadataToken.ToUInt32(),
        member_access = method.IsPublic ? "Public" : method.IsPrivate ? "Private"
            : method.IsAssembly ? "Assembly" : method.IsFamily ? "Family"
            : method.IsFamilyOrAssembly ? "FamilyOrAssembly" : method.IsFamilyAndAssembly ? "FamilyAndAssembly" : "CompilerControlled",
        parameter_tokens = (explicitReceiver ? new uint[] { 0 } : Array.Empty<uint>())
            .Concat(method.Parameters.Select(p => p.MetadataToken.RID == 0 ? 0U : p.MetadataToken.ToUInt32())).ToArray()
    });
}
