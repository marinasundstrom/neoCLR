using Mono.Cecil;

// Provider byte access is independent of text helpers and descriptor lookup.
static class StorageProviderBindings
{
    public const string Name = "System.Storage.StorageProvider";
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == Name
        && right.FullName == Name && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace Storage {
            public interface StorageProvider {
                Result<Streams.InputStream, Streams.StreamError> OpenRead(Path path);
                Result<Streams.OutputStream, Streams.StreamError> CreateNew(Path path);
            }
        }
        """;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        var owner = InterfaceBindings.Type(reference.DeclaringType);
        if (owner is null || owner != Name) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = reference.Name switch {
            "OpenRead" => ("System.Storage.Path", "System.Result<System.Streams.InputStream,System.Streams.StreamError>"),
            "CreateNew" => ("System.Storage.Path", "System.Result<System.Streams.OutputStream,System.Streams.StreamError>"),
            _ => throw new InvalidDataException("Unsupported storage provider member.")
        };
        if (!definition.DeclaringType.IsInterface || definition.DeclaringType.HasGenericParameters
            || !definition.IsPublic || !definition.IsAbstract || !definition.IsVirtual || !definition.IsNewSlot
            || definition.IsFinal || definition.IsStatic || definition.HasBody || definition.HasGenericParameters
            || !reference.HasThis || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported storage provider signature.");
        return new(owner + "::" + reference.Name, new[] { owner }.Concat(args).ToArray(), result,
            Instruction: $"callvirt instance {owner}::{reference.Name}({string.Join(',', args)})");
    }
    public static void Validate(ModuleDefinition module)
    {
        foreach (var (name, members) in new[] {
            (Name, new[] { "OpenRead", "CreateNew" }) })
        {
            var type = module.GetType(name);
            if (type is null || !type.IsPublic || !type.IsInterface || type.HasGenericParameters
                || type.HasFields || type.HasInterfaces || type.Methods.Count != members.Length
                || members.Any(m => type.Methods.Count(method => method.Name == m) != 1))
                throw new InvalidDataException("Unsupported storage provider metadata: " + name);
            foreach (var method in type.Methods) _ = Bind(method, method);
        }
    }
}
