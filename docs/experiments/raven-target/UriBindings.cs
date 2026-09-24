using Mono.Cecil;

// Bounded immutable URI references; ordinary managed parsing and resolution.
static class UriBindings
{
    const string Owner = "System.Uri";
    public static bool IsName(string name) => name == Owner;
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && !type.IsValueType && IsName(type.FullName) ? Owner : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = "\n" + """
        #nullable enable annotations
        public sealed class Uri : Equatable<Uri> {
            private Uri(string text, string scheme, string authority, string path, string query, string fragment) { }
            public Result<Uri, UriError> Resolve(string reference) => default;
            public Result<Uri, UriError> Resolve(Uri reference) => default;
            public static Result<Uri, UriError> Parse(string text) => default;
            public string Text => default;
            public bool IsAbsolute => default;
            public bool IsRelative => default;
            public bool Equals(Uri other) => default;
            public override bool Equals(object? other) => default;
            public override int GetHashCode() => default;
            public override string ToString() => default;
        }
        #nullable restore annotations
        """ + "\n";
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (Type(reference.DeclaringType) is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = reference.Name switch {
            "Resolve" => (args.Length == 1 && args[0] == "String" ? "String" : Owner, "System.Result<System.Uri,System.UriError>", false),
            "Parse" => ("String", "System.Result<System.Uri,System.UriError>", true),
            "get_Text" or "ToString" => ("", "String", false),
            "get_IsAbsolute" or "get_IsRelative" => ("", "Boolean", false),
            "Equals" => (args.Length == 1 && args[0] == "System.Object" ? "System.Object" : Owner, "Boolean", false),
            "GetHashCode" => ("", "Int32", false),
            _ => throw new InvalidDataException("Unsupported Uri member: " + reference.FullName)
        };
        var objectOverride = definition.Name is "ToString" or "GetHashCode" || definition.Name == "Equals" && expected.Item1 == "System.Object";
        var typedEquality = definition.Name == "Equals" && expected.Item1 == Owner;
        if (!definition.IsPublic || definition.IsVirtual != (objectOverride || typedEquality)
            || definition.IsNewSlot != typedEquality || definition.IsFinal != typedEquality || definition.HasGenericParameters
            || !definition.DeclaringType.IsSealed || definition.DeclaringType.HasGenericParameters
            || definition.IsStatic != expected.Item3 || reference.HasThis == expected.Item3
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported Uri signature.");
        return new(Owner + "::" + reference.Name,
            definition.IsStatic ? args : new[] { Owner }.Concat(args).ToArray(), result,
            Instruction: $"call {(definition.IsStatic ? "" : "instance ")}{Owner}::{reference.Name}({string.Join(',', args)})");
    }
}
