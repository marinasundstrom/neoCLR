using Mono.Cecil;

static class Utf8Bindings
{
    public const string Owner = "System.Text.Utf8";
    public const string Bytes = "System.Collections.Sequence<Byte>";
    public const string Decoded = "System.Result<String,System.Text.InvalidUtf8Error>";
    public const string Declarations = "namespace Text { public static class Utf8 { public static Collections.Sequence<byte> Encode(string text) => default; public static Result<string, InvalidUtf8Error> Decode(Collections.Sequence<byte> bytes) => default; } }";

    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != Owner) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition,
            type => CollectionBindings.Type(type) ?? GenericUnionBindings.Type(type));
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis
            || definition.IsVirtual || !definition.IsPublic || !definition.IsStatic
            || definition.HasGenericParameters || reference is GenericInstanceMethod
            || !(reference.Name == "Encode" && args.SequenceEqual(new[] { "String" }) && result == Bytes
                || reference.Name == "Decode" && args.SequenceEqual(new[] { Bytes }) && result == Decoded))
            throw new InvalidDataException("Unsupported UTF-8 signature: " + reference.FullName);
        return new(Owner + "::" + reference.Name, args, result);
    }
}
