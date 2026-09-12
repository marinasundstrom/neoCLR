using Mono.Cecil;

// Bounded Double methods from the existing runtime library.
static class DoubleBindings
{
    static readonly string[] Unary = ["Abs", "Sqrt", "Floor", "Ceiling", "Truncate", "Round", "Exp", "Log", "Log10", "Sin", "Cos", "Tan"];
    static readonly string[] Binary = ["Pow", "Min", "Max"];
    public static string MathDeclarations => string.Join(" ", Unary.Select(n => $"public static double {n}(double value) => 0;")
        .Concat(Binary.Select(n => $"public static double {n}(double left, double right) => 0;")));
    public const string Declarations = "public struct Double { public int CompareTo(double other) => 0; }";
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope)) return null;
        if (reference.DeclaringType.FullName == "System.Math" && reference.ReturnType.MetadataType == MetadataType.Double)
        {
            var (args, result) = RuntimeSignatures.Match(reference, definition, _ => null);
            if (!reference.HasThis && result == "Double" && args.All(a => a == "Double")
                && (args.Length == 1 && Unary.Contains(reference.Name) || args.Length == 2 && Binary.Contains(reference.Name)))
                return new("System.Math::" + reference.Name, args, result);
            throw new InvalidDataException("Unsupported Double Math signature.");
        }
        if (reference.DeclaringType.FullName != "System.Double") return null;
        var signature = RuntimeSignatures.Match(reference, definition, _ => null);
        if (reference.HasThis && reference.DeclaringType.IsValueType && !definition.IsVirtual && reference.Name == "CompareTo"
            && signature.Result == "Int32" && signature.Args.SequenceEqual(new[] { "Double" }))
            return new("RuntimeDoubleCompareTo", ["Double&", "Double"], "Int32");
        throw new InvalidDataException("Unsupported Double member.");
    }
    public const string Adapters = """
        .function RuntimeDoubleCompareTo(Double& source,Double other) -> Int32
        ldarg source
        ldarg other
        call instance System.Double::CompareTo(Double)
        ret
        .end
        """;
}
