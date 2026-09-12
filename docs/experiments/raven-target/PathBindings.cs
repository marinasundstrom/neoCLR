using Mono.Cecil;

// Existing lexical path operations: declaration and admission share one catalog.
static class PathBindings
{
    static readonly (string Name, int Arity)[] Members = [("Combine", 2), ("GetFileName", 1)];
    public static string Declarations => "namespace IO { public static class Path { "
        + string.Join(" ", Members.Select(m => $"public static string {m.Name}({string.Join(',', Enumerable.Range(0, m.Arity).Select(i => "string value" + i))}) => default;")) + " } }";
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != "System.IO.Path") return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, _ => null);
        if (RuntimeSignatures.IsCore(reference.DeclaringType.Scope) && !reference.HasThis
            && !definition.IsVirtual && result == "String"
            && Members.Any(m => m.Name == reference.Name && m.Arity == args.Length)
            && args.All(a => a == "String"))
            return new("System.IO.Path::" + reference.Name, args, result);
        throw new InvalidDataException("Unsupported Path member: " + reference.FullName);
    }
}
