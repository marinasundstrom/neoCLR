using Mono.Cecil;

// A namespace function using Raven's existing CLI container metadata contract.
static class FaultBindings
{
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (NamespaceFunctions.Owner(reference.DeclaringType) != "System" || reference.Name != "Fault") return null;
        var signature = RuntimeSignatures.Match(reference, definition, _ => null);
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis || definition.IsVirtual
            || !NamespaceFunctions.IsContainer(definition.DeclaringType)
            || !signature.Args.SequenceEqual(new[] { "String" }) || signature.Result != "noresult")
            throw new InvalidDataException("Unsupported System.Fault signature.");
        return new("System.Fault", signature.Args, signature.Result);
    }
}
