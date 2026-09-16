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
            ("LocalDateTime", ["Int64"], "System.LocalDateTime"),
            ("PathCombine", ["String", "String"], "String"),
            ("PathGetFileName", ["String"], "String"),
            ("WriteAllText", ["String", "String", "Int32"], "Int32"),
            ("CharCategory", ["Char"], "Int32"),
            ("TypeName", ["System.RuntimeTypeHandle"], "String"),
            ("TypeEquals", ["System.RuntimeTypeHandle", "System.RuntimeTypeHandle"], "Boolean"),
            ("TypeArgumentCount", ["System.RuntimeTypeHandle"], "Int32"),
            ("TypeArgument", ["System.RuntimeTypeHandle", "Int32"], "System.RuntimeTypeHandle"),
            ("TypeShape", ["System.RuntimeTypeHandle", "Int32"], "Boolean"),
            ("TypeDisplayName", ["System.RuntimeTypeHandle", "Int32"], "String"),
            ("TypeInfo", ["System.RuntimeTypeHandle"], "System.Introspection.TypeInfo"),
            ("TypeFields", ["System.RuntimeTypeHandle", "Int32"], "arrayref<System.Introspection.FieldInfo>"),
            ("TypeMethods", ["System.RuntimeTypeHandle", "Int32"], "arrayref<System.Introspection.MethodInfo>"),
            ("TypeProperties", ["System.RuntimeTypeHandle", "Int32"], "arrayref<System.Introspection.PropertyInfo>"),
            ("TypeBaseType", ["System.RuntimeTypeHandle"], "System.Option<System.Type>"),
            ("TypeElementType", ["System.RuntimeTypeHandle"], "System.Option<System.Type>"),
            ("TypeInterfaces", ["System.RuntimeTypeHandle"], "arrayref<System.Type>"),
            ("TypeGenericArguments", ["System.RuntimeTypeHandle"], "arrayref<System.Type>"),
            ("TypeEnumNames", ["System.RuntimeTypeHandle"], "arrayref<String>"),
            ("TypeEnumUnderlying", ["System.RuntimeTypeHandle"], "System.Type")
        }).ToArray();
    static string CSharp(string type) => type switch {
        "Double" => "double", "String" => "string", "Int32" => "int", "Char" => "char",
        "Boolean" => "bool", "Int64" => "long",
        _ when type.StartsWith("System.") => type,
        _ when type.StartsWith("arrayref<") => CSharp(type[9..^1]) + "[]",
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
        var (args, result) = RuntimeSignatures.Match(reference, definition,
            t => ReflectionBindings.Type(t) ?? ProcessBindings.ArrayType(t) ?? GenericUnionBindings.Type(t));
        if (!Members.Any(m => m.Name == reference.Name && m.Args.SequenceEqual(args) && m.Result == result))
            throw new InvalidDataException("Unsupported runtime service signature: " + reference.FullName);
        if (reference.Name == "LocalDateTime")
            return new("System.LocalDateTime::FromUnixTimeTicks", args, result);
        if (reference.Name == "TypeInfo")
            return new("System.Introspection.TypeInfo::FromHandle", args, result);
        if (result.StartsWith("arrayref<"))
        {
            var element = result[9..^1];
            var name = "RuntimeService" + reference.Name;
            var parameters = string.Join(",", args.Select((t, i) => t + " arg" + i));
            var loads = string.Join("\n", args.Select((_, i) => "ldarg arg" + i));
            var signature = string.Join(",", args);
            Helpers[name] = $".function {name}({parameters}) -> {result}\n.local {element}[] source\n.local {result} destination\n.local Int32 index\n{loads}\ncall neoCLR.Runtime.{reference.Name}({signature})\nstloc source\nldloc source\nldlen\nconv.i4\nnewarr {element}\nstloc destination\nldc.i4 0\nstloc index\nbr Test\nCopy:\nldloc destination\nldloc index\nldloc source\nldloc index\nldelem {element}\nstelem {element}\nldloc index\nldc.i4 1\nadd\nstloc index\nTest:\nldloc index\nldloc source\nldlen\nconv.i4\nblt Copy\nldloc destination\nret\n.end\n";
            return new(name, args, result);
        }
        return new("neoCLR.Runtime." + reference.Name, args, result);
    }
    static readonly Dictionary<string, string> Helpers = new();
    public static void Reset() => Helpers.Clear();
    public static string Adapters => string.Join("\n", Helpers.Values);
}
