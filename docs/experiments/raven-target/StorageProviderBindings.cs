using Mono.Cecil;

// Providers resolve logical addresses; returned items own byte-access behavior.
static class StorageProviderBindings
{
    public const string Name = "System.Storage.StorageProvider";
    public static bool IsName(string name) => name == Name;
    public static bool SameType(TypeReference left, TypeReference right) => IsName(left.FullName)
        && right.FullName == left.FullName && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace Storage {
            public interface StorageProvider {
                Result<StorageItem, StorageLookupError> GetItem(Path path);
                Result<File, StorageLookupError> GetFile(Path path);
                Result<Directory, StorageLookupError> GetDirectory(Path path);
            }
        }
        """;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        var owner = InterfaceBindings.Type(reference.DeclaringType);
        if (owner is null || !IsName(owner)) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = (owner, reference.Name) switch {
            (Name, "GetDirectory") => ("System.Storage.Path", "System.Result<System.Storage.Directory,System.Storage.StorageLookupError>"),
            (Name, "GetFile") => ("System.Storage.Path", "System.Result<System.Storage.File,System.Storage.StorageLookupError>"),
            (Name, "GetItem") => ("System.Storage.Path", "System.Result<System.Storage.StorageItem,System.Storage.StorageLookupError>"),
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
            (Name, new[] { "GetItem", "GetFile", "GetDirectory" }) })
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
