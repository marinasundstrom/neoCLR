using Mono.Cecil;

// Raven's CLI namespace-member contract. Container spelling is not semantic.
static class NamespaceFunctions
{
    const string Marker = "System.Runtime.CompilerServices.TopLevelAttribute";
    public static bool IsContainer(TypeDefinition type) => type.IsPublic && type.IsAbstract && type.IsSealed
        && !type.HasGenericParameters && type.CustomAttributes.Any(a => a.AttributeType.FullName == Marker
            && RuntimeSignatures.IsCore(a.AttributeType.Scope));

    public static string Owner(TypeDefinition type) => IsContainer(type) ? type.Namespace : type.FullName;
    public static string Owner(TypeReference type) => type.MetadataType is MetadataType.Class or MetadataType.ValueType or MetadataType.GenericInstance
        ? Owner(type.Resolve()) : type.FullName;

    public static void ProjectFault(ModuleDefinition module)
    {
        var type = module.GetType("System.FaultFunctions");
        type.Name = "NamespaceMembers";
        var marker = module.GetType(Marker).Methods.Single(m => m.IsConstructor && !m.HasParameters);
        type.CustomAttributes.Add(new CustomAttribute(marker));
    }

    public static void ProjectMath(ModuleDefinition module)
    {
        var type = module.GetType("System.Math");
        type.Namespace = "System.Math";
        type.Name = "NamespaceMembers";
        var marker = module.GetType(Marker).Methods.Single(m => m.IsConstructor && !m.HasParameters);
        type.CustomAttributes.Add(new CustomAttribute(marker));
    }
}
