using Mono.Cecil;

// Existing numeric parsing contract; error/carrier mechanics live in ResultBindings.
static class Int32Bindings
{
    public const string Declarations = "public struct Int32 { public static Result<int, Int32ParseError> Parse(string value) => default; }";
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != "System.Int32") return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, ResultBindings.Type);
        if (RuntimeSignatures.IsCore(reference.DeclaringType.Scope) && !reference.HasThis
            && reference.Name == "Parse" && args.SequenceEqual(new[] { "String" })
            && result == "System.Result<Int32,System.Int32ParseError>")
            return new("System.Int32::Parse", args, result);
        throw new InvalidDataException("Unsupported Int32 member: " + reference.FullName);
    }
}
