using Mono.Cecil;

// Bootstrap-only typed access to erased case storage. Consumer references never
// expose this authoring API; arbitrary generic/native payloads are not admitted.
static class ValueStorageBindings
{
    const string Owner = "System.Runtime.CompilerServices.ValueStorage";
    public const string Declarations = """
        namespace Runtime.CompilerServices {
            public static class ValueStorage {
                public static Value Pack<T>(T value) => default;
                public static bool Is<T>(Value value) => default;
                public static T Unpack<T>(Value value) => default;
            }
        }
        """;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, Func<TypeReference, string> map)
    {
        if (reference.DeclaringType.FullName != Owner) return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis
            || reference is not GenericInstanceMethod method || method.GenericArguments.Count != 1
            || definition.GenericParameters.Count != 1 || definition.GenericParameters[0].HasConstraints
            || definition.GenericParameters[0].Attributes != GenericParameterAttributes.NonVariant)
            throw new InvalidDataException("Unsupported case storage intrinsic.");
        var argument = method.GenericArguments[0];
        var element = map(argument);
        if (!ErrorBindings.Cases.Any(c => c.Value.Any(n => element == c.Key + "." + n))
            || !(RuntimeSignatures.IsCore(argument.Scope)
                || ApplicationTypes.IsLibrary(argument) && ErrorCarrierLibrary.IsCase(argument.Resolve())))
            throw new InvalidDataException("Unsupported case storage payload.");
        var signature = RuntimeSignatures.Match(reference, definition, t => map(t), allowOpenMethodParameters: true);
        var expected = reference.Name switch {
            "Pack" => (element, "Value", "value.pack "),
            "Is" => ("Value", "Boolean", "value.is "),
            "Unpack" => ("Value", element, "value.unpack "),
            _ => throw new InvalidDataException("Unsupported case storage operation.")
        };
        if (!signature.Args.SequenceEqual(new[] { expected.Item1 }) || signature.Result != expected.Item2)
            throw new InvalidDataException("Invalid case storage signature.");
        return new("", signature.Args, signature.Result, Instruction: expected.Item3 + element);
    }
}
