using Mono.Cecil;

static class ReaderBindings
{
    public const string Reader = "System.IO.TextReader";
    public const string Stream = "System.IO.StreamReader";
    public const string Writer = "System.IO.TextWriter";
    public const string StreamWriter = "System.IO.StreamWriter";
    public static bool IsContract(string name) => name is Reader or Writer;
    public static bool IsProvider(TypeDefinition type) => type.FullName is "System.IO.ConsoleInputStream" or "System.IO.ConsoleOutputStream";
    public static bool IsName(string name) => name is Reader or Stream or Writer or StreamWriter or "System.IO.ConsoleInputStream" or "System.IO.ConsoleOutputStream";
    public static void Project(ModuleDefinition module) {
        foreach (var type in module.Types.Where(IsProvider)) {
            type.Attributes = (type.Attributes & ~TypeAttributes.VisibilityMask) | TypeAttributes.NotPublic;
            foreach (var method in type.Methods.Where(m => m.IsConstructor))
                method.Attributes = (method.Attributes & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Assembly;
        }
    }
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public static bool SameType(TypeReference left, TypeReference right) => IsName(left.FullName)
        && right.FullName == left.FullName && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace IO {
            public interface TextWriter {
                Result<int, StreamError> Write(string text);
                Result<int, StreamError> WriteLine(string text);
                Result<PropagationUnit, StreamError> Flush();
                void Close();
            }
            public sealed class StreamWriter : TextWriter {
                public StreamWriter(OutputStream output) { }
                public StreamWriter(OutputStream output, bool leaveOpen) { }
                public Result<int, StreamError> Write(string text) => default;
                public Result<int, StreamError> WriteLine(string text) => default;
                public Result<PropagationUnit, StreamError> Flush() => default;
                public void Close() { }
            }
            public sealed class ConsoleInputStream : InputStream {
                public ConsoleInputStream() { }
                public Result<int, StreamError> Read(byte[] buffer, int offset, int count) => default;
                public void Close() { }
            }
            public sealed class ConsoleOutputStream : OutputStream {
                public ConsoleOutputStream(bool error) { }
                public Result<int, StreamError> Write(byte[] buffer, int offset, int count) => default;
                public Result<PropagationUnit, StreamError> Flush() => default;
                public void Close() { }
            }
            public interface TextReader {
                Result<Option<string>, TextReadError> ReadLine(int maxUtf8Bytes);
                Result<string, TextReadError> ReadToEnd(int maxUtf8Bytes);
                void Close();
            }
            public sealed class StreamReader : TextReader {
                public StreamReader(InputStream input) { }
                public StreamReader(InputStream input, bool leaveOpen) { }
                public Result<Option<string>, TextReadError> ReadLine(int maxUtf8Bytes) => default;
                public Result<string, TextReadError> ReadToEnd(int maxUtf8Bytes) => default;
                public void Close() { }
            }
        }
        """;
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null || IsProvider(definition.DeclaringType)) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = reference.Name switch {
            ".ctor" when owner == Stream => (args.Length == 2 ? "System.IO.InputStream,Boolean" : "System.IO.InputStream", "noresult"),
            ".ctor" when owner == StreamWriter => (args.Length == 2 ? "System.IO.OutputStream,Boolean" : "System.IO.OutputStream", "noresult"),
            "Write" or "WriteLine" when owner is Writer or StreamWriter => ("String", "System.Result<Int32,System.IO.StreamError>"),
            "Flush" when owner is Writer or StreamWriter => ("", "System.Result<Void,System.IO.StreamError>"),
            "ReadLine" => ("Int32", "System.Result<System.Option<String>,System.IO.TextReadError>"),
            "ReadToEnd" => ("Int32", "System.Result<String,System.IO.TextReadError>"),
            "Close" => ("", "noresult"),
            _ => throw new InvalidDataException("Unsupported reader member.")
        };
        if (!definition.IsPublic || definition.IsStatic || definition.HasGenericParameters
            || definition.DeclaringType.HasGenericParameters || definition.IsConstructor != construct
            || !reference.HasThis || definition.IsVirtual != !construct
            || (owner is Stream or StreamWriter ? !definition.DeclaringType.IsSealed || definition.IsFinal != !construct
                : !definition.DeclaringType.IsInterface || !definition.IsAbstract || definition.HasBody)
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported reader signature.");
        return construct ? new(args, owner, $"newobj instance {owner}::.ctor({string.Join(',', args)})")
            : new(new[] { owner }.Concat(args).ToArray(), result,
                $"{(IsContract(owner) ? "callvirt" : "call")} instance {owner}::{reference.Name}({string.Join(',', args)})");
    }
    public static ResultBindings.Binding? BindContract(MethodReference reference, MethodDefinition definition)
    {
        if (!IsContract(reference.DeclaringType.FullName)) return null;
        var call = Bind(reference, definition, false)!;
        return new(reference.DeclaringType.FullName + "::" + reference.Name, call.Arguments, call.Result, Instruction: call.Instruction);
    }
}
