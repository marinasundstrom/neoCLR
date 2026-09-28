using Mono.Cecil;

// CLI delegate metadata is transport only. Imported callable identity is a structural
// Function shape; target admission and closure lowering live in UnionImport.
static class FunctionBindings
{
    static readonly Dictionary<string, string[]> Shapes = new();
    public static void Reset() => Shapes.Clear();
    public static bool IsType(string type) => Shapes.ContainsKey(type);
    public static string[] Signature(string type) => Shapes[type];
    public static string Declarations => "public abstract class Delegate { } public abstract class MulticastDelegate : Delegate { } "
        + string.Join("\n", Enumerable.Range(0, 5).Select(n => "public delegate TResult Func<"
            + string.Join(',', Enumerable.Range(0, n).Select(i => "T" + i).Append("TResult")) + ">("
            + string.Join(',', Enumerable.Range(0, n).Select(i => "T" + i + " arg" + i)) + ");"))
        + string.Join("\n", Enumerable.Range(0, 5).Select(n => "public delegate void Action"
            + (n == 0 ? "" : "<" + string.Join(',', Enumerable.Range(0, n).Select(i => "T" + i)) + ">")
            + "(" + string.Join(',', Enumerable.Range(0, n).Select(i => "T" + i + " arg" + i)) + ");"));
    // Metadata transport for the target-only synthesized instance property.
    public static void Project(ModuleDefinition module)
    {
        var info = module.GetType("System.Introspection.MethodInfo");
        foreach (var type in module.Types.Where(t => t.Namespace == "System"
            && (t.Name.StartsWith("Func`") || t.Name == "Action" || t.Name.StartsWith("Action`"))))
        {
            var getter = new MethodDefinition("get_Function", MethodAttributes.Public
                | MethodAttributes.HideBySig | MethodAttributes.SpecialName, info);
            type.Methods.Add(getter);
            type.Methods.Add(new MethodDefinition("ToString", MethodAttributes.Public
                | MethodAttributes.HideBySig, module.TypeSystem.String));
            type.Properties.Add(new PropertyDefinition("Function", PropertyAttributes.None, info) { GetMethod = getter });
        }
    }
    public static string? Type(TypeReference type)
    {
        if (type.IsValueType || !RuntimeSignatures.IsCore(type.Scope)) return null;
        var generic = type as GenericInstanceType;
        var arguments = generic?.GenericArguments.ToArray() ?? [];
        var elementName = generic?.ElementType.FullName ?? type.FullName;
        var isAction = elementName == "System.Action" && arguments.Length == 0
            || arguments.Length is >= 1 and <= 4 && elementName == "System.Action`" + arguments.Length;
        if (!isAction && !(arguments.Length is >= 1 and <= 5 && elementName == "System.Func`" + arguments.Length)) return null;
        var signature = arguments.Select(t => CollectionBindings.Type(t) ?? ProcessBindings.ArrayType(t) ?? GenericUnionBindings.Type(t)).ToArray();
        if (isAction) signature = signature.Append("Void").ToArray();
        if (signature.Any(t => t is null)) return null;
        var name = "fn<" + string.Join(',', signature) + ">";
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
        var (args, result) = RuntimeSignatures.Match(reference, definition, t => Type(t) ?? CollectionBindings.Type(t) ?? GenericUnionBindings.Type(t), allowOpenMethodParameters: GenericUnionBindings.ParameterMap is not null);
        var signature = Shapes[owner];
        if (reference.HasThis && callvirt && reference.Name == "ToString"
            && args.Length == 0 && result == "String")
            return new(owner + "::ToString", [owner], result,
                Instruction: $"callvirt instance {owner}::ToString()");
        if (reference.HasThis && callvirt && reference.Name == "get_Function"
            && args.Length == 0 && result == "System.Introspection.MethodInfo")
            return new(owner + "::get_Function", [owner], result,
                Instruction: $"callvirt instance {owner}::get_Function()");

        if (!reference.HasThis || !callvirt || reference.Name != "Invoke" || !args.SequenceEqual(signature[..^1]) || (result != signature[^1] && !(result == "noresult" && signature[^1] == "Void")))
            throw new InvalidDataException($"Unsupported delegate invocation: {reference.FullName}; callvirt={callvirt}, actual={string.Join(',', args)} -> {result}, expected={string.Join(',', signature)}.");
        // Source unit Functions use an inhabited Void result. CLI Action is transport:
        // binding adapters create that value, and CLI void invocation discards it.
        return new(owner + "::Invoke", new[] { owner }.Concat(args).ToArray(), result,
            Instruction: $"callvirt instance {owner}::Invoke({string.Join(',', args)})" + (result == "noresult" ? "\npop" : ""));
    }
}
