using Mono.Cecil;

// Raven propagation uses ordinary out on both paths. The historical Neo profile
// retains its separate conditional-output snapshot.
static class PropagationLibrary
{
    public static bool IsContract(TypeDefinition type) => type.FullName == "System.Propagatable`3";
    public static bool IsOutput(MethodDefinition method, ParameterDefinition parameter) =>
        IsContract(method.DeclaringType) && method.HasThis && !method.HasGenericParameters
        && method.ReturnType.MetadataType == MetadataType.Boolean && method.Parameters.Count == 1
        && parameter == method.Parameters[0] && parameter.IsOut && !parameter.IsIn
        && parameter.ParameterType is ByReferenceType { ElementType: GenericParameter payload }
        && payload.Type == GenericParameterType.Type && payload.Owner == method.DeclaringType
        && (method.Name == "TryGetOutput" && payload.Position == 1
            || method.Name == "TryGetResidual" && payload.Position == 2);
}
