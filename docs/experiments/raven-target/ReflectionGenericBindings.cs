using Mono.Cecil;

// Exact typed activation facade; no arbitrary generic reflection admission.
static class ReflectionGenericBindings
{
    const string Owner = "System.Runtime.Reflection.TypeReflectionExtensions";
    public static bool IsGeneric(MethodDefinition method) => method.DeclaringType.FullName == Owner
        && method.Name == "CreateInstance" && method.IsPublic && method.IsStatic
        && !method.IsVirtual && !method.IsAbstract && !method.ExplicitThis
        && method.CallingConvention == MethodCallingConvention.Generic
        && method.GenericParameters.Count == 1 && !method.GenericParameters[0].HasConstraints
        && method.GenericParameters[0].Attributes == GenericParameterAttributes.NonVariant
        && method.Parameters.Count == 2 && method.Parameters[0].ParameterType.FullName == "System.Introspection.TypeInfo"
        && (RuntimeSignatures.IsCore(method.Parameters[0].ParameterType.Scope) || ApplicationTypes.IsLibrary(method.Parameters[0].ParameterType))
        && method.Parameters[1].ParameterType is ArrayType array && array.IsVector && array.ElementType.MetadataType == MetadataType.Object
        && method.ReturnType is GenericInstanceType result && result.ElementType.FullName == "System.Result`2"
        && RuntimeSignatures.IsCore(result.Scope) && result.GenericArguments.Count == 2
        && result.GenericArguments[0] is GenericParameter parameter && parameter.Owner == method && parameter.Position == 0
        && result.GenericArguments[1].FullName == "System.Runtime.Reflection.ReflectionError" && RuntimeSignatures.IsCore(result.GenericArguments[1].Scope);
    public const string Implementation = Owner + ".CreateInstanceTyped";
    public static CollectionBindings.Binding Bind(MethodReference reference, MethodDefinition definition, bool construct, bool library)
    {
        if (construct || !IsGeneric(definition) || reference is not GenericInstanceMethod generic || generic.GenericArguments.Count != 1 || !RuntimeSignatures.IsCore(reference.DeclaringType.Scope))
            throw new InvalidDataException("Unsupported typed reflection activation: " + reference.FullName);
        LibraryImplementation.CheckExtensionContract(definition, definition);
        var signature = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type, allowOpenMethodParameters: library);
        var argument = RuntimeSignatures.Map(generic.GenericArguments[0], GenericUnionBindings.Type);
        return new(signature.Args, signature.Result, $"call {Implementation}<{argument}>({string.Join(',', signature.Args)})");
    }
}
