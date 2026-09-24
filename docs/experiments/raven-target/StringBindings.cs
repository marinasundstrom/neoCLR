using Mono.Cecil;

// One catalog drives reference declarations and the existing immutable String APIs.
static class StringBindings
{
    sealed record Member(string Name, string[] Parameters, string Result, bool Instance = false, bool ByRefReceiver = false);
    static readonly Member[] Members = [
        new("Intern", ["String"], "String"),
        new("Concat", ["String", "String"], "String"),
        new("op_Equality", ["String", "String"], "Boolean"),
        new("op_Inequality", ["String", "String"], "Boolean"),
        new("CompareOrdinal", ["String", "String"], "Int32"),
        new("Equals", ["String"], "Boolean", true, true),
        new("ContainsOrdinal", ["String"], "Boolean", true, true),
        new("StartsWithOrdinal", ["String"], "Boolean", true, true),
        new("EndsWithOrdinal", ["String"], "Boolean", true, true),
        new("GetUtf8ByteCount", [], "Int32", true),
        new("get_Item", ["Int32"], "Char", true, true),
        new("get_Length", [], "Int32", true),
        new("GetIterator", [], "System.Collections.Iterator<Char>", true, true),
        new("GetScalars", [], "System.Collections.Sequence<UInt32>", true),
        new("get_IsEmpty", [], "Boolean", true),
        new("SliceUtf8", ["Int32", "Int32"], ResultBindings.Slice, true)
    ];
    static string ParameterName(Member member, int index) => member.Name switch {
        "Concat" or "CompareOrdinal" => index == 0 ? "left" : "right",
        "Intern" => "text",
        "Equals" => "other",
        "ContainsOrdinal" => "substring",
        "StartsWithOrdinal" => "prefix",
        "EndsWithOrdinal" => "suffix",
        "SliceUtf8" => index == 0 ? "byteStart" : "byteLength",
        _ => throw new InvalidDataException("Missing String parameter name: " + member.Name)
    };
    static string CSharp(string type) => type switch { "String" => "string", "Char" => "char", "System.Collections.Iterator<Char>" => "Collections.Iterator<char>", "System.Collections.Sequence<UInt32>" => "Collections.Sequence<uint>", "Int32" => "int", "Boolean" => "bool", ResultBindings.Slice => "Result<string, Text.Utf8SliceError>", _ => throw new InvalidDataException(type) };
    public static string Declarations(bool results, bool collections) => "public sealed class String { " + string.Join(" ", Members.Where(m => (results || m.Result != ResultBindings.Slice)
        && (collections || m.Name is not ("GetIterator" or "GetScalars" or "get_Item"))).Select(m =>
        m.Name == "get_Item" ? "public char this[int index] => default;" :
        m.Name == "get_Length" ? "public int Length => default;" :
        m.Name == "get_IsEmpty" ? "public bool IsEmpty => default;" : m.Name is "op_Equality" or "op_Inequality"
            ? $"public static bool operator {(m.Name == "op_Equality" ? "==" : "!=")}(string left, string right) => default;"
            : $"public {(m.Instance ? "" : "static ")}{CSharp(m.Result)} {m.Name}({string.Join(',', m.Parameters.Select((p, i) => CSharp(p) + " " + ParameterName(m, i)))}) => default;")) + (collections ? " int Collections.Collection<char>.Count => default; public String() {} public String(Collections.Sequence<char> characters) {} public static string CreateFromCharacters(Collections.Sequence<char> characters) => default;" : "") + " }";
    public static bool IsSequenceConstructor(MethodDefinition method) =>
        method.DeclaringType.FullName == "System.String" && RuntimeSignatures.IsCore(method.DeclaringType.Scope)
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
        var signature = RuntimeSignatures.Match(reference, definition, t => CollectionBindings.Type(t) ?? ResultBindings.Type(t));
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
