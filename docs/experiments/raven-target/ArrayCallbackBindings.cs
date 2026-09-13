using Mono.Cecil;

static class ArrayCallbackBindings
{
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != "System.Array" || reference.Name != "ForEach") return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis
            || reference is not GenericInstanceMethod method || method.GenericArguments.Count != 1
            || definition.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant))
            throw new InvalidDataException("Unsupported Array.ForEach signature.");
        var element = GenericUnionBindings.Type(method.GenericArguments[0]);
        var (args, result) = RuntimeSignatures.Match(reference, definition, t => ManagedArrayBindings.Type(t) ?? DelegateBindings.Type(t) ?? GenericUnionBindings.Type(t));
        if (element is null || !ManagedArrayBindings.Defaultable(element) || result != "noresult"
            || !args.SequenceEqual(new[] { $"arrayref<{element}>", $"System.Func<{element},Void>" }))
            throw new InvalidDataException("Unsupported Array.ForEach element or callback.");
        return new($"System.Array::ForEach<{element}>", args, result);
    }
}
