using Mono.Cecil;

static class ReaderBindings
{
    public const string Reader = "System.IO.TextReader";
    public const string Stream = "System.IO.StreamReader";
    public const string Writer = "System.IO.TextWriter";
    public const string StreamWriter = "System.IO.StreamWriter";
    public static bool IsContract(string name) => name is Reader or Writer or "System.Text.Encoding" or "System.Text.Decoder" or "System.Text.Encoder";
    public static bool IsProvider(TypeDefinition type) => type.FullName is "System.IO.ConsoleInputStream" or "System.IO.ConsoleOutputStream";
    public static bool IsName(string name) => IsContract(name) || name is "System.Text.EncoderProgress" or Stream or StreamWriter or "System.IO.ConsoleInputStream" or "System.IO.ConsoleOutputStream";
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
        namespace Text {
            public interface Encoding {
                Result<Collections.Sequence<byte>, EncodingError> Encode(string text);
                // Decode(ReadOnlySlice<byte>) is documented from native metadata.
                Decoder CreateDecoder();
                Encoder CreateEncoder();
            }
            public interface Encoder {
                Result<PropagationUnit, EncodingError> Accept(string text, bool final);
                // Drain(Slice<byte>) is documented from native metadata; no CLI view projection.
            }
            public sealed class EncoderProgress {
                public EncoderProgress(int bytesWritten, EncoderState state) { }
                public int BytesWritten => default;
                public EncoderState State => default;
            }
            public interface Decoder {
                // Decode(ReadOnlySlice<byte>, bool) is documented from native metadata.
            }
        }
        namespace IO {
            public interface TextWriter {
                Concurrency.Tasks.Task<Result<int, StreamError>> Write(string text);
                Concurrency.Tasks.Task<Result<int, StreamError>> WriteLine(string text);
                Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Flush();
                void Close();
            }
            public sealed class StreamWriter : TextWriter {
                public Concurrency.Tasks.Task<Result<int, StreamError>> Finish() => default;
                public StreamWriter(OutputStream output) { }
                public StreamWriter(OutputStream output, bool leaveOpen) { }
                public StreamWriter(OutputStream output, Text.Encoding encoding) { }
                public StreamWriter(OutputStream output, Text.Encoding encoding, bool leaveOpen) { }
                public Concurrency.Tasks.Task<Result<int, StreamError>> Write(string text) => default;
                public Concurrency.Tasks.Task<Result<int, StreamError>> WriteLine(string text) => default;
                public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Flush() => default;
                public void Close() { }
            }
            public sealed class ConsoleInputStream : InputStream {
                public ConsoleInputStream() { }
                public Concurrency.Tasks.Task<Result<int, StreamError>> Read(byte[] buffer, int offset, int count) => default;
                public Concurrency.Tasks.Task<Result<int, StreamError>> Read(byte[] buffer, int offset, int count, Concurrency.CancellationToken cancellationToken) => default;
                public void Close() { }
            }
            public sealed class ConsoleOutputStream : OutputStream {
                public ConsoleOutputStream(bool error) { }
                public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Write(byte[] buffer, int offset, int count) => default;
                public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Write(byte[] buffer, int offset, int count, Concurrency.CancellationToken cancellationToken) => default;
                public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Flush() => default;
                public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Flush(Concurrency.CancellationToken cancellationToken) => default;
                public void Close() { }
            }
            public interface TextReader {
                Concurrency.Tasks.Task<Result<Option<string>, TextReadError>> ReadLine(int maxUtf8Bytes);
                Concurrency.Tasks.Task<Result<string, TextReadError>> ReadToEnd(int maxUtf8Bytes);
                void Close();
            }
            public sealed class StreamReader : TextReader {
                public StreamReader(InputStream input) { }
                public StreamReader(InputStream input, bool leaveOpen) { }
                public StreamReader(InputStream input, Text.Encoding encoding) { }
                public StreamReader(InputStream input, Text.Encoding encoding, bool leaveOpen) { }
                public Concurrency.Tasks.Task<Result<Option<string>, TextReadError>> ReadLine(int maxUtf8Bytes) => default;
                public Concurrency.Tasks.Task<Result<string, TextReadError>> ReadToEnd(int maxUtf8Bytes) => default;
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
            ".ctor" when owner == Stream => (ConstructorArguments("System.IO.InputStream", args), "noresult"),
            ".ctor" when owner == StreamWriter => (ConstructorArguments("System.IO.OutputStream", args), "noresult"),
            "Write" or "WriteLine" when owner is Writer or StreamWriter => ("String", "System.Result<Int32,System.IO.StreamError>"),
            "Flush" when owner is Writer or StreamWriter => ("", "System.Result<Void,System.IO.StreamError>"),
            "Encode" when owner == "System.Text.Encoding" => ("String", "System.Result<System.Collections.Sequence<Byte>,System.Text.EncodingError>"),
            ".ctor" when owner == "System.Text.EncoderProgress" => ("Int32,System.Text.EncoderState", "noresult"),
            "get_BytesWritten" when owner == "System.Text.EncoderProgress" => ("", "Int32"),
            "get_State" when owner == "System.Text.EncoderProgress" => ("", "System.Text.EncoderState"),
            "Accept" when owner == "System.Text.Encoder" => ("String,Boolean", "System.Result<Void,System.Text.EncodingError>"),
            "Finish" when owner == StreamWriter => ("", "System.Result<Int32,System.IO.StreamError>"),
            "CreateEncoder" when owner == "System.Text.Encoding" => ("", "System.Text.Encoder"),
            "CreateDecoder" when owner == "System.Text.Encoding" => ("", "System.Text.Decoder"),
            "ReadLine" when owner is Reader or Stream => ("Int32", "System.Result<System.Option<String>,System.IO.TextReadError>"),
            "ReadToEnd" when owner is Reader or Stream => ("Int32", "System.Result<String,System.IO.TextReadError>"),
            "Close" when owner is Reader or Stream or Writer or StreamWriter => ("", "noresult"),
            _ => throw new InvalidDataException("Unsupported reader member.")
        };
        var virtualMember = !construct && owner != "System.Text.EncoderProgress"
            && !(owner == StreamWriter && reference.Name == "Finish");
        if (!definition.IsPublic || definition.IsStatic || definition.HasGenericParameters
            || definition.DeclaringType.HasGenericParameters || definition.IsConstructor != construct
            || !reference.HasThis || definition.IsVirtual != virtualMember
            || (owner is Stream or StreamWriter or "System.Text.EncoderProgress" ? !definition.DeclaringType.IsSealed || definition.IsFinal != virtualMember
                : !definition.DeclaringType.IsInterface || !definition.IsAbstract || definition.HasBody)
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported reader signature.");
        return construct ? new(args, owner, $"newobj instance {owner}::.ctor({string.Join(',', args)})")
            : new(new[] { owner }.Concat(args).ToArray(), result,
                $"{(IsContract(owner) ? "callvirt" : "call")} instance {owner}::{reference.Name}({string.Join(',', args)})");
    }
    static string ConstructorArguments(string stream, string[] args)
    {
        var suffix = args.Length switch {
            1 => "",
            2 when args[1] == "Boolean" => ",Boolean",
            2 => ",System.Text.Encoding",
            3 => ",System.Text.Encoding,Boolean",
            _ => throw new InvalidDataException("Unsupported stream constructor.")
        };
        return stream + suffix;
    }
    public static ResultBindings.Binding? BindContract(MethodReference reference, MethodDefinition definition)
    {
        if (!IsContract(reference.DeclaringType.FullName)) return null;
        var call = Bind(reference, definition, false)!;
        return new(reference.DeclaringType.FullName + "::" + reference.Name, call.Arguments, call.Result, Instruction: call.Instruction);
    }
}
