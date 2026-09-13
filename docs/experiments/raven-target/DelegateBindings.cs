using Mono.Cecil;

// Existing nominal Func family; target admission and closure lowering live in UnionImport.
static class DelegateBindings
{
    static readonly Dictionary<string, string[]> Shapes = new();
    public static void Reset() => Shapes.Clear();
    public static bool IsType(string type) => Shapes.ContainsKey(type);
    public static string[] Signature(string type) => Shapes[type];
    public static string Declarations => "public abstract class Delegate { } public abstract class MulticastDelegate : Delegate { } "
        + string.Join("\n", Enumerable.Range(0, 5).Select(n => "public delegate TResult Func<"
            + string.Join(',', Enumerable.Range(0, n).Select(i => "T" + i).Append("TResult")) + ">("
            + string.Join(',', Enumerable.Range(0, n).Select(i => "T" + i + " arg" + i)) + ");"));
    public static string? Type(TypeReference type)
    {
        if (type.IsValueType || type is not GenericInstanceType g || !RuntimeSignatures.IsCore(type.Scope)
            || g.GenericArguments.Count is < 1 or > 5 || g.ElementType.FullName != "System.Func`" + g.GenericArguments.Count) return null;
        var signature = g.GenericArguments.Select(t => CollectionBindings.Type(t) ?? ProcessBindings.ArrayType(t) ?? GenericUnionBindings.Type(t)).ToArray();
        if (signature.Any(t => t is null)) return null;
        var name = "System.Func<" + string.Join(',', signature) + ">";
        Shapes[name] = signature.Select(t => t!).ToArray();
        return name;
    }
    public static void Constructor(MethodReference reference, MethodDefinition definition)
    {
        if (Type(reference.DeclaringType) is null || reference.Name != ".ctor"
            || reference.CallingConvention != MethodCallingConvention.Default || definition.CallingConvention != MethodCallingConvention.Default
            || reference.HasGenericParameters || reference is GenericInstanceMethod
            || definition.DeclaringType.BaseType?.FullName != "System.MulticastDelegate" || !definition.DeclaringType.IsSealed
            || !reference.HasThis || reference.ExplicitThis || !definition.IsConstructor || !definition.IsPublic
            || reference.ReturnType.MetadataType != MetadataType.Void || definition.ReturnType.MetadataType != MetadataType.Void
            || reference.Parameters.Count != 2 || definition.Parameters.Count != 2
            || reference.Parameters[0].ParameterType.FullName != "System.Object" || reference.Parameters[1].ParameterType.MetadataType != MetadataType.IntPtr
            || !reference.Parameters.Select(p => p.ParameterType.FullName).SequenceEqual(definition.Parameters.Select(p => p.ParameterType.FullName)))
            throw new InvalidDataException("Unsupported delegate constructor.");
    }
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool callvirt)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, t => Type(t) ?? CollectionBindings.Type(t) ?? GenericUnionBindings.Type(t));
        var signature = Shapes[owner];
        if (!reference.HasThis || !callvirt || reference.Name != "Invoke" || !args.SequenceEqual(signature[..^1]) || result != signature[^1])
            throw new InvalidDataException($"Unsupported delegate invocation: {reference.FullName}; callvirt={callvirt}, actual={string.Join(',', args)} -> {result}, expected={string.Join(',', signature)}.");
        return new(owner + "::Invoke", new[] { owner }.Concat(args).ToArray(), result == "Void" ? "noresult" : result,
            Instruction: $"callvirt instance {owner}::Invoke({string.Join(',', args)})" + (result == "Void" ? "\npop" : ""));
    }
}
