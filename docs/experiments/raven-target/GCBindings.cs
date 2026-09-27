using Mono.Cecil;

// Execution-local diagnostics and control; no generation/byte accounting promises.
static class GCBindings
{
    public const string Owner = "System.Runtime.GC";
    public static readonly string[] Counters = ["CollectionCount", "AllocatedObjectCount", "HeapObjectCount", "PeakHeapObjectCount", "ReclaimedObjectCount", "HeapObjectLimit"];
    public static string Declarations => "namespace Runtime { public static class GC { "
        + string.Join(" ", Counters.Select(name => $"public static long {name} => default;"))
        + " public static void Collect() { } public static void KeepAlive(object? value) { } } }";

    public static void Validate(ModuleDefinition module)
    {
        var type = module.GetType(Owner);
        if (type is null || !type.IsPublic || !type.IsAbstract || !type.IsSealed
            || type.HasGenericParameters || type.HasFields || type.HasInterfaces
            || type.Methods.Count != Counters.Length + 2)
            throw new InvalidDataException("Unsupported GC metadata.");
        foreach (var method in type.Methods) Bind(method, method);
    }

    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != Owner) return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis
            || reference.HasGenericParameters || !definition.IsPublic || !definition.IsStatic)
            throw new InvalidDataException("Unsupported GC member.");
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var valid = reference.Name switch {
            "Collect" => args.Length == 0 && result == "noresult",
            "KeepAlive" => args.SequenceEqual(new[] { "System.Object" }) && result == "noresult",
            _ => Counters.Any(name => reference.Name == "get_" + name) && args.Length == 0 && result == "Int64"
        };
        if (!valid) throw new InvalidDataException("Unsupported GC signature.");
        return new($"{Owner}::{reference.Name}", args, result);
    }
}
