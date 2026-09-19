using Mono.Cecil;

static class RuntimeFailureBindings
{
    const string Owner = "System.Runtime.CompilerServices.RuntimeFailure";
    public const string Declarations = """
        namespace Runtime.CompilerServices {
            public static class RuntimeFailure {
                public static void Terminate(string message) { }
            }
        }
        """;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, Func<TypeReference, string> map)
    {
        if (reference.DeclaringType.FullName != Owner) return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis
            || reference.HasGenericParameters || reference.ExplicitThis || definition.IsVirtual || reference.Name != "Terminate")
            throw new InvalidDataException("Unsupported terminal failure intrinsic.");
        var signature = RuntimeSignatures.Match(reference, definition, map);
        if (!signature.Args.SequenceEqual(new[] { "String" }) || signature.Result != "noresult")
            throw new InvalidDataException("Invalid terminal failure signature.");
        return new("", signature.Args, signature.Result, Instruction: "call neoCLR.Runtime.Fault(String)\npop");
    }
}
