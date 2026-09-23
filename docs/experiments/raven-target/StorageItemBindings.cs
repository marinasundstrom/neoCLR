using Mono.Cecil;

// Public item kinds are interfaces; implementation state belongs to providers.
static class StorageItemBindings
{
    public const string Root = "System.Storage.StorageItem";
    public static bool IsName(string name) => name is Root or "System.Storage.File" or "System.Storage.Directory";
    public static bool Assignable(string source, string target) => target == Root && source is "System.Storage.File" or "System.Storage.Directory";
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace Storage {
            public interface StorageItem { Path Path { get; } string Name { get; } }
            public interface File : StorageItem {
                Result<Streams.InputStream, Streams.StreamError> OpenRead();
                Result<Streams.OutputStream, Streams.StreamError> CreateNew();
            }
            public interface Directory : StorageItem {
                Result<Collections.Sequence<StorageItem>, StorageLookupError> GetItems(int maxItems);
                Result<StorageItem, StorageLookupError> GetItem(Path relativePath);
                Result<Directory, StorageLookupError> GetDirectory(string name);
                Result<Directory, StorageLookupError> GetDirectory(Path relativePath);
                Result<File, StorageLookupError> FileAt(string name);
                Result<File, StorageLookupError> GetFile(string name);
                Result<File, StorageLookupError> GetFile(Path relativePath);
            }
        }
        """;
    public static ResultBindings.Binding? BindContract(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = (owner, reference.Name) switch {
            (Root, "get_Path") => ("", "System.Storage.Path"),
            (Root, "get_Name") => ("", "String"),
            ("System.Storage.File", "OpenRead") => ("", "System.Result<System.Streams.InputStream,System.Streams.StreamError>"),
            ("System.Storage.File", "CreateNew") => ("", "System.Result<System.Streams.OutputStream,System.Streams.StreamError>"),
            ("System.Storage.Directory", "GetItems") => ("Int32", "System.Result<System.Collections.Sequence<System.Storage.StorageItem>,System.Storage.StorageLookupError>"),
            ("System.Storage.Directory", "GetItem") => ("System.Storage.Path", "System.Result<System.Storage.StorageItem,System.Storage.StorageLookupError>"),
            ("System.Storage.Directory", "GetDirectory") => (args.Length == 1 && args[0] == "System.Storage.Path" ? "System.Storage.Path" : "String", "System.Result<System.Storage.Directory,System.Storage.StorageLookupError>"),
            ("System.Storage.Directory", "FileAt") => ("String", "System.Result<System.Storage.File,System.Storage.StorageLookupError>"),
            ("System.Storage.Directory", "GetFile") => (args.Length == 1 && args[0] == "System.Storage.Path" ? "System.Storage.Path" : "String", "System.Result<System.Storage.File,System.Storage.StorageLookupError>"),
            _ => throw new InvalidDataException("Unsupported storage item member.")
        };
        if (!definition.DeclaringType.IsInterface || definition.DeclaringType.HasGenericParameters
            || !definition.IsPublic || !definition.IsAbstract || !definition.IsVirtual || !definition.IsNewSlot
            || definition.IsFinal || definition.IsStatic || definition.HasBody || definition.HasGenericParameters
            || !reference.HasThis || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported storage item signature.");
        return new(owner + "::" + reference.Name, new[] { owner }.Concat(args).ToArray(), result,
            Instruction: $"callvirt instance {owner}::{reference.Name}({string.Join(',', args)})");
    }
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct)
    {
        if (Type(reference.DeclaringType) is null) return null;
        if (construct) throw new InvalidDataException("Storage interfaces cannot be constructed.");
        var call = BindContract(reference, definition)!;
        return new(call.Arguments, call.Result, call.Instruction!);
    }
    public static void Validate(ModuleDefinition module)
    {
        foreach (var (name, count) in new[] { (Root, 2), ("System.Storage.File", 2), ("System.Storage.Directory", 7) })
        {
            var type = module.GetType(name);
            if (type is null || !type.IsPublic || !type.IsInterface || type.HasFields || type.HasGenericParameters
                || type.Methods.Count != count || (name == Root ? type.HasInterfaces : type.Interfaces.Count != 1 || type.Interfaces[0].InterfaceType.FullName != Root))
                throw new InvalidDataException("Unsupported storage item metadata: " + name);
            StorageHierarchy.Validate(type);
            foreach (var method in type.Methods) _ = BindContract(method, method);
        }
    }
}
