using Mono.Cecil;

static class ReaderBindings
{
    public const string Reader = "System.IO.TextReader";
    public const string Stream = "System.IO.StreamReader";
    public static bool IsName(string name) => name is Reader or Stream;
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope)
        && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public static bool SameType(TypeReference left, TypeReference right) => IsName(left.FullName)
        && right.FullName == left.FullName && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace IO {
            public interface TextReader {
                Result<string, TextReadError> ReadToEnd(int maxUtf8Bytes);
                void Close();
            }
            public sealed class StreamReader : TextReader {
                public StreamReader(InputStream input) { }
                public StreamReader(InputStream input, bool leaveOpen) { }
                public Result<string, TextReadError> ReadToEnd(int maxUtf8Bytes) => default;
                public void Close() { }
            }
        }
        """;
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = reference.Name switch {
            ".ctor" when owner == Stream => (args.Length == 2 ? "System.IO.InputStream,Boolean" : "System.IO.InputStream", "noresult"),
            "ReadToEnd" => ("Int32", "System.Result<String,System.IO.TextReadError>"),
            "Close" => ("", "noresult"),
            _ => throw new InvalidDataException("Unsupported reader member.")
        };
        if (!definition.IsPublic || definition.IsStatic || definition.HasGenericParameters
            || definition.DeclaringType.HasGenericParameters || definition.IsConstructor != construct
            || !reference.HasThis || definition.IsVirtual != !construct
            || (owner == Stream ? !definition.DeclaringType.IsSealed || definition.IsFinal != !construct
                : !definition.DeclaringType.IsInterface || !definition.IsAbstract || definition.HasBody)
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported reader signature.");
        return construct ? new(args, owner, $"newobj instance {owner}::.ctor({string.Join(',', args)})")
            : new(new[] { owner }.Concat(args).ToArray(), result,
                $"{(owner == Reader ? "callvirt" : "call")} instance {owner}::{reference.Name}({string.Join(',', args)})");
    }
    public static ResultBindings.Binding? BindContract(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != Reader) return null;
        var call = Bind(reference, definition, false)!;
        return new(Reader + "::" + reference.Name, call.Arguments, call.Result, Instruction: call.Instruction);
    }
}
