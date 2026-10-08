using Mono.Cecil;

// Temporary CLI reference projection; executable implementation is Raven-owned.
static class StringBuilderBindings
{
    public const string Name = "System.Text.StringBuilder";
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && !type.IsValueType && type.FullName == Name ? Name : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == Name
        && right.FullName == Name && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace Text {
            public sealed class StringBuilder {
                public StringBuilder() { }
                public StringBuilder(int maxUtf8Bytes) { }
                public int Utf8ByteCount => default;
                public int MaxUtf8Bytes => default;
                public StringBuilder Append(string text) => default;
                public StringBuilder AppendLine(string text) => default;
                public StringBuilder AppendLine() => default;
                public StringBuilder Clear() => default;
                public override string ToString() => default;
            }
        }
        """;
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct)
    {
        if (Type(reference.DeclaringType) is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = reference.Name switch {
            ".ctor" when args.Length == 0 => ("", "noresult"),
            ".ctor" => ("Int32", "noresult"),
            "get_Utf8ByteCount" or "get_MaxUtf8Bytes" => ("", "Int32"),
            "AppendLine" when args.Length == 0 => ("", Name),
            "Append" or "AppendLine" => ("String", Name),
            "Clear" => ("", Name),
            "ToString" => ("", "String"),
            _ => throw new InvalidDataException("Unsupported StringBuilder member.")
        };
        if (!definition.IsPublic || definition.IsStatic || definition.HasGenericParameters
            || !definition.DeclaringType.IsSealed || definition.DeclaringType.HasGenericParameters
            || definition.IsConstructor != construct || !reference.HasThis
            || definition.IsVirtual != (reference.Name == "ToString")
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported StringBuilder signature.");
        return construct ? new(args, Name, $"newobj instance {Name}::.ctor({string.Join(',', args)})")
            : new(new[] { Name }.Concat(args).ToArray(), result, $"call instance {Name}::{reference.Name}({string.Join(',', args)})");
    }
}
