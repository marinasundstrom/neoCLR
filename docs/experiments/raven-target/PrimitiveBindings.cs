using Mono.Cecil;

// Storage types remain exact; values loaded onto the CLI stack are normalized.
static class PrimitiveBindings
{
    public static readonly string[] Types = ["SByte", "Byte", "Int16", "UInt16", "Char", "UInt32", "Int64", "UInt64", "Single", "IntPtr", "UIntPtr"];
    static readonly string[] CharacterMethods = ["IsDigit", "IsNumber", "IsLetter", "IsUpper", "IsLower", "IsSeparator", "IsControl", "IsPunctuation", "IsSymbol", "IsSurrogate", "IsHighSurrogate", "IsLowSurrogate", "IsAscii", "IsAsciiDigit", "IsLetterOrDigit", "IsWhiteSpace"];
    public static string Stack(string type) => type switch {
        "SByte" or "Byte" or "Int16" or "UInt16" or "Char" or "UInt32" => "Int32",
        "UInt64" => "Int64", "Single" => "Double", _ => type
    };
    public static string Project(string source)
    {
        foreach (var type in Types)
            source = source.Replace($"public struct {type} {{ }}", $"public struct {type} {{ public int CompareTo({type} other) => 0; "
                + (type == "Char" ? string.Join(" ", CharacterMethods.Select(n => $"public static bool {n}(char value) => false;")) : "") + " }");
        return source;
    }
    public static bool IsReceiver(string type) => Types.Contains(type) || type is "Int32" or "Double";
    public static string? Type(TypeReference type) => type.IsValueType
        && type.MetadataType is MetadataType.SByte or MetadataType.Byte or MetadataType.Int16 or MetadataType.UInt16
            or MetadataType.Char or MetadataType.UInt32 or MetadataType.Int64 or MetadataType.UInt64
            or MetadataType.Single or MetadataType.IntPtr or MetadataType.UIntPtr ? type.Name : null;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope)) return null;
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, Type);
        if (reference.HasThis && !definition.IsVirtual && reference.Name == "CompareTo"
            && result == "Int32" && args.SequenceEqual(new[] { owner }))
            return new("Runtime" + owner + "CompareTo", [owner + "&", owner], result);
        if (!reference.HasThis && owner == "Char" && CharacterMethods.Contains(reference.Name)
            && result == "Boolean" && args.SequenceEqual(new[] { "Char" }))
            return new("System.Char::" + reference.Name, args, result);
        throw new InvalidDataException("Unsupported primitive member: " + reference.FullName);
    }
    public static string Adapters => string.Join("\n", Types.Select(t => $".function Runtime{t}CompareTo({t}& source,{t} other) -> Int32\nldarg source\nldarg other\ncall instance System.{t}::CompareTo({t})\nret\n.end\n"));
    public static string? Default(string type) => Types.Contains(type) ? type switch {
        "Single" => "ldc.r4 0", "Int64" or "UInt64" => "ldc.i8 0",
        "IntPtr" => "ldc.i4 0\nconv.i", "UIntPtr" => "ldc.i4 0\nconv.u", _ => "ldc.i4 0"
    } : null;
}
