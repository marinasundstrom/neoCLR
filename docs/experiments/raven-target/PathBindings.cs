using Mono.Cecil;

// Validated Storage values coexist with the existing native string helpers.
static class PathBindings
{
    const string Owner = "System.Storage.Path";
    public static bool IsName(string name) => name == Owner;
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && !type.IsValueType && IsName(type.FullName) ? Owner : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace Storage { public sealed class Path {
            private Path(string text) { }
            public static string Combine(string value0, string value1) => default;
            public static string GetFileName(string value0) => default;
            public static Result<Path, InvalidPathError> Parse(string text) => default;
            public string Text => default;
            public bool IsAbsolute => default;
            public bool IsRelative => default;
            public bool Equals(Path other) => default;
            public string ToString() => default;
        } }
        """;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (Type(reference.DeclaringType) is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = reference.Name switch {
            "Combine" => ("String,String", "String", true),
            "GetFileName" => ("String", "String", true),
            "Parse" => ("String", "System.Result<System.Storage.Path,System.Storage.InvalidPathError>", true),
            "get_Text" or "ToString" => ("", "String", false),
            "get_IsAbsolute" or "get_IsRelative" => ("", "Boolean", false),
            "Equals" => (Owner, "Boolean", false),
            _ => throw new InvalidDataException("Unsupported Path member: " + reference.FullName)
        };
        if (!definition.IsPublic || definition.IsVirtual || definition.HasGenericParameters
            || !definition.DeclaringType.IsSealed || definition.DeclaringType.HasGenericParameters
            || definition.IsStatic != expected.Item3 || reference.HasThis == expected.Item3
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported Path signature.");
        return new(Owner + "::" + reference.Name,
            definition.IsStatic ? args : new[] { Owner }.Concat(args).ToArray(), result,
            Instruction: $"call {(definition.IsStatic ? "" : "instance ")}{Owner}::{reference.Name}({string.Join(',', args)})");
    }
}
