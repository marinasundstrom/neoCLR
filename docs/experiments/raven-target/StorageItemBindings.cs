using Mono.Cecil;

// Provider-bound addresses: construction and properties perform no storage lookup.
static class StorageItemBindings
{
    public static bool IsName(string name) => name is "System.Storage.File" or "System.Storage.Directory";
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string FileMembers = """
        public File(StorageProvider provider, Path path) { }
        public Path Path => default;
        public string Name => default;
        public Result<Streams.InputStream, Streams.StreamError> OpenRead() => default;
        public Result<Streams.OutputStream, Streams.StreamError> CreateNew() => default;
        """;
    public const string Declarations = """
        namespace Storage { public sealed class Directory {
            public Directory(StorageLookup provider, Path path) { }
            public Path Path => default;
            public Result<File, StorageLookupError> FileAt(string name) => default;
            public Result<File, StorageLookupError> GetFile(string name) => default;
            public Result<File, StorageLookupError> GetFile(Path relativePath) => default;
        } }
        """;
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null || definition.IsStatic) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = (owner, reference.Name) switch {
            ("System.Storage.File", ".ctor") => ("System.Storage.StorageProvider,System.Storage.Path", "noresult"),
            ("System.Storage.Directory", ".ctor") => ("System.Storage.StorageLookup,System.Storage.Path", "noresult"),
            (_, "get_Path") => ("", "System.Storage.Path"),
            ("System.Storage.File", "get_Name") => ("", "String"),
            ("System.Storage.File", "OpenRead") => ("", "System.Result<System.Streams.InputStream,System.Streams.StreamError>"),
            ("System.Storage.File", "CreateNew") => ("", "System.Result<System.Streams.OutputStream,System.Streams.StreamError>"),
            ("System.Storage.Directory", "FileAt") => ("String", "System.Result<System.Storage.File,System.Storage.StorageLookupError>"),
            ("System.Storage.Directory", "GetFile") => (args.Length == 1 && args[0] == "System.Storage.Path" ? "System.Storage.Path" : "String", "System.Result<System.Storage.File,System.Storage.StorageLookupError>"),
            _ => throw new InvalidDataException("Unsupported storage descriptor member.")
        };
        if (!definition.IsPublic || definition.IsVirtual || definition.HasGenericParameters
            || !definition.DeclaringType.IsSealed || definition.DeclaringType.HasGenericParameters
            || !reference.HasThis || definition.IsConstructor != construct
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported storage descriptor signature.");
        return construct
            ? new(args, owner, $"newobj instance {owner}::.ctor({string.Join(',', args)})")
            : new(new[] { owner }.Concat(args).ToArray(), result, $"call instance {owner}::{reference.Name}({string.Join(',', args)})");
    }
}
