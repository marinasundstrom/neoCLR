using Mono.Cecil;

// One catalog drives reference declarations and the existing immutable String APIs.
static class StringBindings
{
    sealed record Member(string Name, string[] Parameters, string Result, bool Instance = false, bool ByRefReceiver = false);
    static readonly Member[] Members = [
        new("Format", ["System.Globalization.Culture", "String", "System.Object"], "String"),
        new("Format", ["System.Globalization.Culture", "String", "System.Object", "System.Object"], "String"),
        new("Format", ["System.Globalization.Culture", "String", "System.Object", "System.Object", "System.Object"], "String"),
        new("Format", ["System.Globalization.Culture", "String", "arrayref<System.Object>"], "String"),
        new("Format", ["String", "System.Object"], "String"),
        new("Format", ["String", "System.Object", "System.Object"], "String"),
        new("Format", ["String", "System.Object", "System.Object", "System.Object"], "String"),
        new("Format", ["String", "arrayref<System.Object>"], "String"),
        new("Join", ["String", "arrayref<String>"], "String"),
        new("IsNullOrEmpty", ["String"], "Boolean"),
        new("IsNullOrWhiteSpace", ["String"], "Boolean"),
        new("CompareTo", ["String"], "Int32", true),
        new("Equals", ["String", "System.StringComparison"], "Boolean", true),
        new("Intern", ["String"], "String"),
        new("Concat", ["String", "String"], "String"),
        new("Concat", ["System.Object", "System.Object"], "String"),
        new("op_Equality", ["String", "String"], "Boolean"),
        new("op_Inequality", ["String", "String"], "Boolean"),
        new("Compare", ["String", "String"], "Int32"),
        new("Compare", ["String", "String", "System.StringComparison"], "Int32"),
        new("CompareOrdinalIgnoreCase", ["String", "String"], "Int32"),
        new("CompareOrdinal", ["String", "String"], "Int32"),
        new("Equals", ["String"], "Boolean", true, true),
        new("Equals", ["System.Object"], "Boolean", true),
        new("GetHashCode", [], "Int32", true),
        new("ToString", [], "String", true),
        new("ReplaceLineEndings", [], "String", true),
        new("ReplaceLineEndings", ["String"], "String", true),
        new("Trim", [], "String", true),
        new("TrimStart", [], "String", true),
        new("TrimEnd", [], "String", true),
        new("Replace", ["String", "String"], "String", true),
        new("Contains", ["String"], "Boolean", true),
        new("Contains", ["String", "System.StringComparison"], "Boolean", true),
        new("StartsWith", ["String"], "Boolean", true),
        new("StartsWith", ["String", "System.StringComparison"], "Boolean", true),
        new("EndsWith", ["String"], "Boolean", true),
        new("EndsWith", ["String", "System.StringComparison"], "Boolean", true),
        new("ContainsOrdinal", ["String"], "Boolean", true, true),
        new("StartsWithOrdinal", ["String"], "Boolean", true, true),
        new("EndsWithOrdinal", ["String"], "Boolean", true, true),
        new("GetUtf8ByteCount", [], "Int32", true),
        new("get_Item", ["Int32"], "Char", true, true),
        new("get_Length", [], "Int32", true),
        new("GetIterator", [], "System.Collections.Iterator<Char>", true, true),
        new("GetScalars", [], "System.Collections.Sequence<UInt32>", true),
        new("ToUpperInvariant", [], "String", true),
        new("ToLowerInvariant", [], "String", true),
        new("get_IsEmpty", [], "Boolean", true),
        new("SliceUtf8", ["Int32", "Int32"], ResultBindings.Slice, true)
    ];
    static string ParameterName(Member member, int index) => member.Name switch {
        "Format" when member.Parameters[0] == "System.Globalization.Culture" => index == 0 ? "culture" : index == 1 ? "format" : member.Parameters[index].StartsWith("arrayref") ? "arguments" : "arg" + (index - 2),
        "Format" => index == 0 ? "format" : member.Parameters[index].StartsWith("arrayref") ? "arguments" : "arg" + (index - 1),
        "Compare" => index == 0 ? "left" : index == 1 ? "right" : "comparison",
        "Concat" or "CompareOrdinal" or "CompareOrdinalIgnoreCase" => index == 0 ? "left" : "right",
        "Join" => index == 0 ? "separator" : "values",
        "Intern" or "IsNullOrEmpty" or "IsNullOrWhiteSpace" => "text",
        "Equals" => index == 0 ? "other" : "comparison",
        "CompareTo" => "other",
        "ReplaceLineEndings" => "replacementText",
        "Replace" => index == 0 ? "oldValue" : "newValue",
        "Contains" or "ContainsOrdinal" => index == 0 ? "substring" : "comparison",
        "StartsWith" or "StartsWithOrdinal" => index == 0 ? "prefix" : "comparison",
        "EndsWith" or "EndsWithOrdinal" => index == 0 ? "suffix" : "comparison",
        "SliceUtf8" => index == 0 ? "byteStart" : "byteLength",
        _ => throw new InvalidDataException("Missing String parameter name: " + member.Name)
    };
    static string CSharp(string type) => type switch { "System.Globalization.Culture" => "System.Globalization.Culture", "System.StringComparison" => "StringComparison", "String" => "string", "arrayref<String>" => "string[]", "arrayref<System.Object>" => "params object?[]", "System.Object" => "object?", "Char" => "char", "System.Collections.Iterator<Char>" => "Collections.Iterator<char>", "System.Collections.Sequence<UInt32>" => "Collections.Sequence<uint>", "Int32" => "int", "Boolean" => "bool", ResultBindings.Slice => "Result<string, Text.Utf8SliceError>", _ => throw new InvalidDataException(type) };
    public static string Declarations(bool results, bool collections) => "public sealed class String { " + string.Join(" ", Members.Where(m => (results || m.Result != ResultBindings.Slice)
        && (collections || m.Name is not ("GetIterator" or "GetScalars" or "get_Item" or "Compare" or "CompareOrdinalIgnoreCase"))).Select(m =>
        m.Name == "get_Item" ? "public char this[int index] => default;" :
        m.Name == "get_Length" ? "public int Length => default;" :
        m.Name == "get_IsEmpty" ? "public bool IsEmpty => default;" : m.Name is "op_Equality" or "op_Inequality"
            ? $"public static bool operator {(m.Name == "op_Equality" ? "==" : "!=")}(string left, string right) => default;"
            : (collections && m.Name == "GetUtf8ByteCount" ? "[System.Obsolete(\"Byte counting will move to explicit encoding APIs. This string member is scheduled for removal.\")] " : collections && m.Name == "SliceUtf8" ? "[System.Obsolete(\"Byte slicing will move to explicit encoding and buffer APIs. This string member is scheduled for removal.\")] " : "") + $"public {(m.Name is "GetHashCode" or "ToString" || m.Name == "Equals" && m.Parameters.SequenceEqual(new[] { "System.Object" }) ? "override " : m.Instance ? "" : "static ")}{CSharp(m.Result)} {m.Name}({string.Join(',', m.Parameters.Select((p, i) => (m.Name is "IsNullOrEmpty" or "IsNullOrWhiteSpace" ? "string?" : CSharp(p)) + " " + ParameterName(m, i)))}) => default;")) + (collections ? " public char this[Index index] => default; public string this[Range range] => default; public Optional<char> Get(int index) => default; public Optional<char> Get(Index index) => default; public Result<string, InvalidRangeError> Slice(Range range) => default; int Collections.Collection<char>.Count => default; public String() {} public String(Collections.Sequence<char> characters) {} public static string CreateFromCharacters(Collections.Sequence<char> characters) => default;" : "") + " }";
    public static bool IsSequenceConstructor(MethodDefinition method) => IsSequenceConstructor(method, true);
    public static bool IsSequenceConstructor(MethodDefinition method, bool requireCore) =>
        method.DeclaringType.FullName == "System.String" && (!requireCore || RuntimeSignatures.IsCore(method.DeclaringType.Scope))
        && method.IsConstructor && method.IsPublic && !method.IsStatic && !method.IsVirtual
        && !method.HasGenericParameters && !method.ExplicitThis
        && method.CallingConvention == MethodCallingConvention.Default && method.ReturnType.MetadataType == MetadataType.Void
        && method.Parameters.Count == 1
        && CollectionBindings.Type(method.Parameters[0].ParameterType) == "System.Collections.Sequence<Char>";
    public static string? Construct(MethodReference reference, MethodDefinition definition) {
        if (reference.DeclaringType.FullName != "System.String" || !RuntimeSignatures.IsCore(reference.DeclaringType.Scope)) return null;
        if (!IsSequenceConstructor(definition) || !reference.HasThis || reference.ExplicitThis || reference.HasGenericParameters
            || reference.CallingConvention != MethodCallingConvention.Default || reference.Name != ".ctor"
            || reference.ReturnType.MetadataType != MetadataType.Void || reference.Parameters.Count != 1
            || CollectionBindings.Type(reference.Parameters[0].ParameterType) != "System.Collections.Sequence<Char>")
            throw new InvalidDataException("Unsupported String constructor: " + reference.FullName);
        return "System.Collections.Sequence<Char>";
    }
    public sealed record Binding(string[] Arguments, string Result, string Instruction);
    public static Binding? Bind(MethodReference reference, MethodDefinition definition, bool callvirt)
    {
        if (reference.DeclaringType.FullName != "System.String" || !RuntimeSignatures.IsCore(reference.DeclaringType.Scope)) return null;
        var signature = RuntimeSignatures.Match(reference, definition, t => t.FullName == "System.Object" && (t.MetadataType == MetadataType.Object || RuntimeSignatures.IsCore(t.Scope)) ? "System.Object" : GlobalizationBindings.Type(t) ?? EnumBindings.Type(t) ?? ManagedArrayBindings.Type(t) ?? CollectionBindings.Type(t) ?? ResultBindings.Type(t));
        var member = Members.SingleOrDefault(m => m.Name == reference.Name && m.Instance == reference.HasThis
            && m.Result == signature.Result && m.Parameters.SequenceEqual(signature.Args))
            ?? throw new InvalidDataException("Unsupported String member: " + reference.FullName);
        if ((definition.IsVirtual && !definition.IsFinal) || reference.DeclaringType.IsValueType || callvirt && !reference.HasThis)
            throw new InvalidDataException("Unsupported String receiver contract.");
        var args = member.Instance ? new[] { "String" }.Concat(member.Parameters).ToArray() : member.Parameters;
        if (member.Name is "op_Equality" or "op_Inequality")
            return new(args, "Boolean", "call RuntimeStringEquals(String,String)"
                + (member.Name == "op_Inequality" ? "\nldc.bool false\nceq" : ""));
        return new(args, member.Result, member.Instance
            ? $"call RuntimeString{member.Name}({string.Join(',', args)})"
            : $"call System.String::{member.Name}({string.Join(',', args)})");
    }
    public static string Adapters() => string.Join("\n", Members.Where(m => m.Instance).Select(m =>
        $".function RuntimeString{m.Name}({string.Join(',', new[] { "String source" }.Concat(m.Parameters.Select((p, i) => p + " value" + i)))}) -> {m.Result}\n"
        + (m.ByRefReceiver ? ".local String receiver\nldarg source\nstloc receiver\nldloca receiver\n" : "ldarg source\n")
        + string.Concat(m.Parameters.Select((_, i) => $"ldarg value{i}\n"))
        + $"call instance System.String::{m.Name}({string.Join(',', m.Parameters)})\nret\n.end\n"));
}
