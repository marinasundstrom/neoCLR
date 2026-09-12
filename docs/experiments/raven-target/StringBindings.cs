using Mono.Cecil;

// One catalog drives reference declarations and the existing immutable String APIs.
static class StringBindings
{
    sealed record Member(string Name, string[] Parameters, string Result, bool Instance = false, bool ByRefReceiver = false);
    static readonly Member[] Members = [
        new("Concat", ["String", "String"], "String"),
        new("CompareOrdinal", ["String", "String"], "Int32"),
        new("Equals", ["String"], "Boolean", true, true),
        new("ContainsOrdinal", ["String"], "Boolean", true, true),
        new("StartsWithOrdinal", ["String"], "Boolean", true, true),
        new("EndsWithOrdinal", ["String"], "Boolean", true, true),
        new("GetUtf8ByteCount", [], "Int32", true),
        new("IsEmpty", [], "Boolean", true)
    ];
    static string CSharp(string type) => type switch { "String" => "string", "Int32" => "int", "Boolean" => "bool", _ => throw new InvalidDataException(type) };
    public static string Declarations => "public sealed class String { " + string.Join(" ", Members.Select(m =>
        $"public {(m.Instance ? "" : "static ")}{CSharp(m.Result)} {m.Name}({string.Join(',', m.Parameters.Select((p, i) => CSharp(p) + " value" + i))}) => default;")) + " }";
    public sealed record Binding(string[] Arguments, string Result, string Instruction);
    public static Binding? Bind(MethodReference reference, MethodDefinition definition, bool callvirt)
    {
        if (reference.DeclaringType.FullName != "System.String" || !RuntimeSignatures.IsCore(reference.DeclaringType.Scope)) return null;
        var signature = RuntimeSignatures.Match(reference, definition, _ => null);
        var member = Members.SingleOrDefault(m => m.Name == reference.Name && m.Instance == reference.HasThis
            && m.Result == signature.Result && m.Parameters.SequenceEqual(signature.Args))
            ?? throw new InvalidDataException("Unsupported String member: " + reference.FullName);
        if (definition.IsVirtual || reference.DeclaringType.IsValueType || callvirt && !reference.HasThis)
            throw new InvalidDataException("Unsupported String receiver contract.");
        var args = member.Instance ? new[] { "String" }.Concat(member.Parameters).ToArray() : member.Parameters;
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
