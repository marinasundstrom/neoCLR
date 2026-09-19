using Mono.Cecil;

// The existing conditional-output ABI is narrower than ordinary CLI out: a miss
// leaves the destination untouched. Only the exact propagation contract admits it.
static class PropagationLibrary
{
    public static bool IsContract(TypeDefinition type) => type.FullName == "System.Propagatable`3";
    public static bool IsConditionalOutput(MethodDefinition method, ParameterDefinition parameter) =>
        IsContract(method.DeclaringType) && method.HasThis && !method.HasGenericParameters
        && method.ReturnType.MetadataType == MetadataType.Boolean && method.Parameters.Count == 1
        && parameter == method.Parameters[0] && parameter.IsOut && !parameter.IsIn
        && parameter.ParameterType is ByReferenceType { ElementType: GenericParameter payload }
        && payload.Type == GenericParameterType.Type && payload.Owner == method.DeclaringType
        && (method.Name == "TryGetOutput" && payload.Position == 1
            || method.Name == "TryGetResidual" && payload.Position == 2);
}
