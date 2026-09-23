using Mono.Cecil;

// Bounded accumulator contract; no generic component hashing is implied.
static class HashCodeBindings
{
    public const string Owner = "System.HashCode";
    public const string Declarations = "public struct HashCode { private int State; public HashCode() { State = 0; } public void Add(int value) { } public void Add(string value) { } public int ToHashCode() => default; public static int Combine(int first, int second) => default; }";
    public static string? Type(TypeReference type) => type.FullName == Owner && RuntimeSignatures.IsCore(type.Scope) && type.Resolve()?.IsValueType == true ? Owner : null;
    public static void ProjectLayout(ModuleDefinition module)
    {
        if (module.GetType(Owner) is { } type) { type.PackingSize = -1; type.ClassSize = -1; }
    }
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (Type(reference.DeclaringType) is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, Type);
        var valid = reference.HasThis
            ? reference.Name == "Add" && args.Length == 1 && args[0] is "Int32" or "String" && result == "noresult"
                || reference.Name == "ToHashCode" && args.Length == 0 && result == "Int32"
            : reference.Name == "Combine" && args.SequenceEqual(new[] { "Int32", "Int32" }) && result == "Int32";
        if (!valid || definition.IsVirtual) throw new InvalidDataException("Unsupported HashCode member: " + reference.FullName);
        var inputs = reference.HasThis ? new[] { Owner + "&" }.Concat(args).ToArray() : args;
        return new("", inputs, result, Instruction: $"call {(reference.HasThis ? "instance " : "")}{Owner}::{reference.Name}({string.Join(',', args)})" + (result == "noresult" ? "\npop" : ""));
    }
    public const string ConstructorAdapter = ".function RuntimeNewHashCode() -> System.HashCode\nnewobj instance System.HashCode::.ctor()\nret\n.end\n";
}
