using Mono.Cecil;

// Directional, blocking file streams. Raw resource IDs never enter the public contract.
static class StreamBindings
{
    const string Prefix = "System.Streams.";
    public static bool IsName(string name) => name is Prefix + "FileInputStream" or Prefix + "FileOutputStream";
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope) && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public const string Declarations = """
        namespace Streams {
            public sealed class FileInputStream {
                private FileInputStream(int handle) { }
                public static Result<FileInputStream, StreamError> Open(string path) => default;
                public Result<int, StreamError> Read(byte[] buffer, int offset, int count) => default;
                public void Close() { }
            }
            public sealed class FileOutputStream {
                private FileOutputStream(int handle) { }
                public static Result<FileOutputStream, StreamError> CreateNew(string path) => default;
                public Result<int, StreamError> Write(byte[] buffer, int offset, int count) => default;
                public Result<PropagationUnit, StreamError> Flush() => default;
                public void Close() { }
            }
        }
        """;
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
            || !definition.IsPublic || definition.IsVirtual || definition.HasGenericParameters
            || !definition.DeclaringType.IsSealed || definition.DeclaringType.HasGenericParameters
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported file stream signature.");
        return new(definition.IsStatic ? args : new[] { owner }.Concat(args).ToArray(), result,
            $"call {(definition.IsStatic ? "" : "instance ")}{owner}::{definition.Name}({string.Join(',', args)})");
    }
}
