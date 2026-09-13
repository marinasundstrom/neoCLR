using Mono.Cecil;

// The CLI uses Int32 stack values for Boolean. The interpreter retains Boolean.
static class BooleanBindings
{
    public const string Declaration = "public struct Boolean { public int CompareTo(bool other) => 0; }";
    public static bool Converts(string source, string target) => source == "Int32" && target == "Boolean" || source == "Boolean" && target == "Int32";
    public static string Convert(string source, string target) => source == "Int32" && target == "Boolean"
        ? "call RuntimeBooleanFromInt32(Int32)\n" : source == "Boolean" && target == "Int32" ? "call RuntimeBooleanToInt32(Boolean)\n" : "";
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != "System.Boolean") return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, _ => null);
        if (RuntimeSignatures.IsCore(reference.DeclaringType.Scope) && reference.HasThis && !definition.IsVirtual
            && reference.Name == "CompareTo" && result == "Int32" && args.SequenceEqual(new[] { "Boolean" }))
            return new("System.Boolean::CompareTo", ["Boolean&", "Boolean"], result, Instruction: "call instance System.Boolean::CompareTo(Boolean)");
        throw new InvalidDataException("Unsupported Boolean member.");
    }
    public const string Adapters = """
        .function RuntimeBooleanFromInt32(Int32 value) -> Boolean
            ldarg value
            ldc.i4 0
            beq False
            ldarg value
            ldc.i4 1
            beq True
            fault "Non-canonical CLI Boolean"
        False:
            ldc.bool false
            ret
        True:
            ldc.bool true
            ret
        .end
        .function RuntimeBooleanToInt32(Boolean value) -> Int32
            ldarg value
            brfalse False
            ldc.i4 1
            ret
        False:
            ldc.i4 0
            ret
        .end
        """;
}
