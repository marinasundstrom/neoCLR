using Mono.Cecil;

// Selected source-projected families, not a manually maintained case ABI.
static class PayloadUnionBindings
{
    const string Owner = "System.Web.Http.HttpError";
    static readonly Dictionary<string, TypeDefinition> Types = new();
    public static void Reset(ModuleDefinition core)
    {
        Types.Clear();
        if (core.GetType(Owner) is not { } root) return;
        if (!ApplicationTypes.IsStandardLibraryUnion(root))
            throw new InvalidDataException("Invalid payload union reference: " + Owner);
        RavenUnionMetadata.ValidateNestedCases(root);
        foreach (var type in new[] { root }.Concat(root.NestedTypes))
            Types.Add(type.FullName.Replace('/', '.'), type);
    }
    public static bool IsType(string name) => Types.ContainsKey(name);
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && IsType(type.FullName.Replace('/', '.')) && type.Resolve() is { IsValueType: true } definition
        && Types[type.FullName.Replace('/', '.')] == definition ? type.FullName.Replace('/', '.') : null;
    static string? Map(TypeReference type) => type.MetadataType == MetadataType.Object ? "System.Object"
        : Type(type) ?? ErrorBindings.Type(type) ?? EnumBindings.Type(type);
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct = false)
    {
        if (Type(reference.DeclaringType) is not { } owner) return null;
        var signature = RuntimeSignatures.Match(reference, definition, Map);
        if (construct != definition.IsConstructor)
            throw new InvalidDataException("Payload union constructor mismatch.");
        var name = ApplicationTypes.MethodName(definition);
        var output = ApplicationTypes.IsConditionalUnionOutput(definition);
        if (construct)
            return new(owner + "::" + name, signature.Args, owner,
                Instruction: $"newobj instance {owner}::.ctor({string.Join(',', signature.Args)})");
        return new(owner + "::" + name,
            (reference.HasThis ? new[] { owner + "&" } : []).Concat(signature.Args).ToArray(), signature.Result,
            output ? 1 : -1,
            Instruction: $"call {(reference.HasThis ? "instance " : "")}{owner}::{name}({string.Join(',', signature.Args)})");
    }
}
