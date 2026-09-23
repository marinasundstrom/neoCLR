using Mono.Cecil;

static class FileSystemBindings
{
    public const string Name = "System.Storage.FileSystem";
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && !type.IsValueType && type.FullName == Name ? Name : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == Name
        && right.FullName == Name && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace Storage {
            public sealed class FileSystem : StorageProvider {
                public FileSystem(string root) { }
                public Result<StorageItem, StorageLookupError> GetItem(Path path) => default;
                public Result<File, StorageLookupError> GetFile(Path path) => default;
                public Result<Directory, StorageLookupError> GetDirectory(Path path) => default;
            }
        }
        """;
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct)
    {
        if (Type(reference.DeclaringType) is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = reference.Name switch {
            ".ctor" => ("String", "noresult"),
            "GetItem" => ("System.Storage.Path", "System.Result<System.Storage.StorageItem,System.Storage.StorageLookupError>"),
            "GetFile" => ("System.Storage.Path", "System.Result<System.Storage.File,System.Storage.StorageLookupError>"),
            "GetDirectory" => ("System.Storage.Path", "System.Result<System.Storage.Directory,System.Storage.StorageLookupError>"),
            _ => throw new InvalidDataException("Unsupported FileSystem member.")
        };
        if (!definition.IsPublic || definition.IsStatic || definition.HasGenericParameters
            || !definition.DeclaringType.IsSealed || definition.DeclaringType.HasGenericParameters
            || definition.IsConstructor != construct || !reference.HasThis
            || definition.IsVirtual != !construct || definition.IsFinal != !construct
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported FileSystem signature.");
        return construct ? new(args, Name, $"newobj instance {Name}::.ctor(String)")
            : new(new[] { Name }.Concat(args).ToArray(), result, $"call instance {Name}::{reference.Name}({string.Join(',', args)})");
    }
}
