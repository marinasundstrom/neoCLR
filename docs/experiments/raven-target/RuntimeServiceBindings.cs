using Mono.Cecil;

// Bootstrap-only access to existing typed host services. This is an authoring
// boundary, not a public API or a way to select arbitrary native entry points.
static class RuntimeServiceBindings
{
    const string Owner = "System.Runtime.CompilerServices.RuntimeServices";
    static readonly string[] UnaryMath = ["Abs", "Sqrt", "Floor", "Ceiling", "Truncate", "Round", "Exp", "Log", "Log10", "Sin", "Cos", "Tan"];
    static readonly string[] BinaryMath = ["Pow", "Min", "Max"];
    static readonly (string Name, string[] Args, string Result)[] Members =
        UnaryMath.Select(n => ("Math" + n, new[] { "Double" }, "Double"))
        .Concat(BinaryMath.Select(n => ("Math" + n, new[] { "Double", "Double" }, "Double"))).Concat(new (string Name, string[] Args, string Result)[] {
            ("PathCombine", ["String", "String"], "String"),
            ("PathGetFileName", ["String"], "String"),
            ("WriteAllText", ["String", "String", "Int32"], "Int32"),
            ("CharCategory", ["Char"], "Int32")
        }).ToArray();
    static string CSharp(string type) => type switch {
        "Double" => "double", "String" => "string", "Int32" => "int", "Char" => "char",
        _ => throw new InvalidDataException("Unsupported runtime service declaration.")
    };
    public static string Declarations => "namespace Runtime.CompilerServices { public static class RuntimeServices { "
        + string.Join(" ", Members.Select(m => $"public static {CSharp(m.Result)} {m.Name}({string.Join(',', m.Args.Select((t, i) => CSharp(t) + " arg" + i))}) => default;")) + " } }";

    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != Owner) return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis
            || !definition.IsPublic || !definition.IsStatic || definition.IsVirtual
            || definition.HasGenericParameters || reference is GenericInstanceMethod)
            throw new InvalidDataException("Unsupported runtime service call.");
        var (args, result) = RuntimeSignatures.Match(reference, definition, _ => null);
        if (!Members.Any(m => m.Name == reference.Name && m.Args.SequenceEqual(args) && m.Result == result))
            throw new InvalidDataException("Unsupported runtime service signature: " + reference.FullName);
        return new("neoCLR.Runtime." + reference.Name, args, result);
    }
}
