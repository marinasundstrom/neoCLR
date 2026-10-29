using Mono.Cecil;

// A namespace function using Raven's existing CLI container metadata contract.
static class FaultBindings
{
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (NamespaceFunctions.Owner(reference.DeclaringType) != "System" || reference.Name != "Panic") return null;
        var signature = RuntimeSignatures.Match(reference, definition, _ => null);
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis || definition.IsVirtual
            || !NamespaceFunctions.IsContainer(definition.DeclaringType)
            || !signature.Args.SequenceEqual(new[] { "String" }) || signature.Result != "void")
            throw new InvalidDataException("Unsupported System.Panic signature.");
        return new("System.Panic", signature.Args, signature.Result);
    }
}
