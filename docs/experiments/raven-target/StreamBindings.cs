using Mono.Cecil;

// Directional, blocking file streams. Raw resource IDs never enter the public contract.
static class StreamBindings
{
    const string Prefix = "System.Streams.";
    public static bool IsCapability(string name) => name is Prefix + "InputStream" or Prefix + "OutputStream";
    public static bool Assignable(string source, string target) =>
        source == Prefix + "FileInputStream" && target == Prefix + "InputStream"
        || source == Prefix + "FileOutputStream" && target == Prefix + "OutputStream";
    public static bool IsName(string name) => name is Prefix + "FileInputStream" or Prefix + "FileOutputStream";
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && (IsName(left.FullName) || IsCapability(left.FullName)) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope) && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public const string Declarations = """
        namespace Streams {
            public interface InputStream {
                Result<int, StreamError> Read(byte[] buffer, int offset, int count);
                void Close();
            }
            public interface OutputStream {
                Result<int, StreamError> Write(byte[] buffer, int offset, int count);
                Result<PropagationUnit, StreamError> Flush();
                void Close();
            }
            public sealed class FileInputStream : InputStream {
                private FileInputStream(int handle) { }
                public static Result<FileInputStream, StreamError> Open(string path) => default;
                public Result<int, StreamError> Read(byte[] buffer, int offset, int count) => default;
                public void Close() { }
            }
            public sealed class FileOutputStream : OutputStream {
                private FileOutputStream(int handle) { }
                public static Result<FileOutputStream, StreamError> CreateNew(string path) => default;
                public Result<int, StreamError> Write(byte[] buffer, int offset, int count) => default;
                public Result<PropagationUnit, StreamError> Flush() => default;
                public void Close() { }
            }
        }
        """;
    public static ResultBindings.Binding? BindCapability(MethodReference reference, MethodDefinition definition)
    {
        var owner = InterfaceBindings.Type(reference.DeclaringType);
        if (owner is null || !IsCapability(owner)) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = (owner, reference.Name) switch {
            (Prefix + "InputStream", "Read") or (Prefix + "OutputStream", "Write") => ("arrayref<Byte>,Int32,Int32", "System.Result<Int32,System.Streams.StreamError>"),
            (Prefix + "OutputStream", "Flush") => ("", "System.Result<Void,System.Streams.StreamError>"),
            (_, "Close") => ("", "noresult"),
            _ => throw new InvalidDataException("Unsupported stream capability member.")
        };
        if (!definition.DeclaringType.IsInterface || definition.DeclaringType.HasGenericParameters
            || !definition.IsPublic || !definition.IsAbstract || !definition.IsVirtual || !definition.IsNewSlot
            || definition.IsFinal || definition.IsStatic || definition.HasBody || definition.HasGenericParameters
            || !reference.HasThis || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported stream capability signature.");
        return new(owner + "::" + reference.Name, new[] { owner }.Concat(args).ToArray(), result,
            Instruction: $"callvirt instance {owner}::{reference.Name}({string.Join(',', args)})");
    }
    public static void ValidateCapabilities(ModuleDefinition module)
    {
        foreach (var (name, members) in new[] {
            (Prefix + "InputStream", new[] { "Read", "Close" }),
            (Prefix + "OutputStream", new[] { "Write", "Flush", "Close" }) })
        {
            var type = module.GetType(name);
            if (type is null || !type.IsPublic || !type.IsInterface || type.HasGenericParameters
                || type.HasFields || type.HasInterfaces || type.Methods.Count != members.Length
                || members.Any(m => type.Methods.Count(method => method.Name == m) != 1))
                throw new InvalidDataException("Unsupported stream capability metadata: " + name);
            foreach (var method in type.Methods) _ = BindCapability(method, method);
        }
    }
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct, bool library)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = (owner, definition.Name) switch {
            (Prefix + "FileInputStream", "Open") or (Prefix + "FileOutputStream", "CreateNew") => ("String", $"System.Result<{owner},System.Streams.StreamError>", true),
            (Prefix + "FileInputStream", "Read") or (Prefix + "FileOutputStream", "Write") => ("arrayref<Byte>,Int32,Int32", "System.Result<Int32,System.Streams.StreamError>", false),
            (Prefix + "FileOutputStream", "Flush") => ("", "System.Result<Void,System.Streams.StreamError>", false),
            (_, "Close") => ("", "noresult", false),
            _ => throw new InvalidDataException("Unsupported file stream member.")
        };
        if (construct || definition.IsConstructor || definition.IsStatic != expected.Item3
            || !definition.IsPublic || definition.IsVirtual != !expected.Item3
            || definition.IsFinal != !expected.Item3 || definition.IsNewSlot != !expected.Item3 || definition.HasGenericParameters
            || !definition.DeclaringType.IsSealed || definition.DeclaringType.HasGenericParameters
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported file stream signature.");
        return new(definition.IsStatic ? args : new[] { owner }.Concat(args).ToArray(), result,
            $"call {(definition.IsStatic ? "" : "instance ")}{owner}::{definition.Name}({string.Join(',', args)})");
    }
}
