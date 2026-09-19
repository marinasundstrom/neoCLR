using Mono.Cecil;

// Native metadata factories store a vector, not a managed array reference. This
// bootstrap-only read view preserves that ABI while Raven owns the copy loop.
static class ParameterSnapshotBindings
{
    const string Owner = "System.Runtime.CompilerServices.ParameterSnapshot";
    public const string Vector = "System.Introspection.ParameterInfo[]";
    public const string Declarations = """
        namespace Runtime.CompilerServices {
            public sealed class ParameterSnapshot {
                private ParameterSnapshot() { }
                public int Length => default;
                public System.Introspection.ParameterInfo Get(int index) => default;
            }
        }
        """;
    public static string? Type(TypeReference type) => type.FullName == Owner
        && RuntimeSignatures.IsCore(type.Scope) && !type.IsValueType ? Vector : null;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, Func<TypeReference, string> map)
    {
        if (Type(reference.DeclaringType) is null) return null;
        if (!reference.HasThis || definition.IsVirtual || definition.HasGenericParameters || reference.ExplicitThis)
            throw new InvalidDataException("Invalid parameter snapshot operation.");
        var signature = RuntimeSignatures.Match(reference, definition, map);
        if (reference.Name == "get_Length" && signature.Args.Length == 0 && signature.Result == "Int32")
            return new("", [Vector], "Int32", Instruction: "ldlen\nconv.i4");
        if (reference.Name == "Get" && signature.Args.SequenceEqual(new[] { "Int32" }) && signature.Result == "System.Introspection.ParameterInfo")
            return new("", [Vector, "Int32"], signature.Result, Instruction: "ldelem System.Introspection.ParameterInfo");
        throw new InvalidDataException("Unsupported parameter snapshot operation.");
    }
}
