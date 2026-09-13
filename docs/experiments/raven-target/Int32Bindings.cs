using Mono.Cecil;

// Existing numeric Result contracts; error/carrier mechanics live in ResultBindings.
static class Int32Bindings
{
    public const string Declarations = "public struct Int32 { public static Result<int, Int32ParseError> Parse(string value) => default; public static Result<int, IntegerDivisionError> Divide(int dividend, int divisor) => default; public bool Equals(int other) => false; public int CompareTo(int other) => 0; public string ToString() => default; }";
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != "System.Int32") return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, ResultBindings.Type);
        if (RuntimeSignatures.IsCore(reference.DeclaringType.Scope) && reference.HasThis
            && reference.DeclaringType.IsValueType && (!definition.IsVirtual || definition.IsFinal)
            && ((reference.Name == "Equals" && result == "Boolean" && args.SequenceEqual(new[] { "Int32" }))
                || (reference.Name == "CompareTo" && result == "Int32" && args.SequenceEqual(new[] { "Int32" }))
                || (reference.Name == "ToString" && result == "String" && args.Length == 0)))
            return new("RuntimeInt32" + reference.Name, new[] { "Int32&" }.Concat(args).ToArray(), result);
        if (RuntimeSignatures.IsCore(reference.DeclaringType.Scope) && !reference.HasThis
            && ((reference.Name == "Parse" && args.SequenceEqual(new[] { "String" })
                && result == "System.Result<Int32,System.Int32ParseError>")
            || (reference.Name == "Divide" && args.SequenceEqual(new[] { "Int32", "Int32" })
                && result == "System.Result<Int32,System.IntegerDivisionError>")))
            return new("System.Int32::" + reference.Name, args, result);
        throw new InvalidDataException("Unsupported Int32 member: " + reference.FullName);
    }
    public static string Adapters => """
        .function RuntimeInt32Equals(Int32& source,Int32 other) -> Boolean
        ldarg source
        ldarg other
        call instance System.Int32::Equals(Int32)
        ret
        .end
        .function RuntimeInt32CompareTo(Int32& source,Int32 other) -> Int32
        ldarg source
        ldarg other
        call instance System.Int32::CompareTo(Int32)
        ret
        .end
        .function RuntimeInt32ToString(Int32& source) -> String
        ldarg source
        ldobj Int32
        call instance System.Int32::ToString()
        ret
        .end
        """;
}
