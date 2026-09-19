using Mono.Cecil;

static class UnicodeScalarBindings
{
    const string Owner = "System.Text.UnicodeScalar";
    static readonly string[] Methods = ["IsDigit", "IsNumber", "IsLetter", "IsUpper", "IsLower", "IsSeparator", "IsControl", "IsPunctuation", "IsSymbol", "IsAscii", "IsAsciiDigit", "IsLetterOrDigit", "IsWhiteSpace"];
    public static string Declarations => "namespace Text { public static class UnicodeScalar { "
        + string.Join(" ", Methods.Select(m => $"public static bool {m}(uint value) => default;")) + " } }";
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != Owner) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, PrimitiveBindings.Type);
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis
            || definition.IsVirtual || !definition.IsPublic || !definition.IsStatic || definition.HasGenericParameters
            || !Methods.Contains(reference.Name) || result != "Boolean" || !args.SequenceEqual(new[] { "UInt32" }))
            throw new InvalidDataException("Unsupported Unicode scalar signature: " + reference.FullName);
        return new(Owner + "::" + reference.Name, args, result);
    }
}
