using Mono.Cecil;

// Provisional TCP client surface. Private operation IDs never enter application APIs.
static class SocketBindings
{
    const string Prefix = "System.Networking.Sockets.";
    public static bool IsName(string name) => name is Prefix + "Socket" or Prefix + "SocketConnectCompletion" or Prefix + "SocketTransferCompletion";
    public static bool IsProvider(TypeDefinition type) => IsName(type.FullName) && type.FullName != Prefix + "Socket";
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope) && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace Networking.Sockets {
            public sealed class Socket {
                public Socket(long handle) { }
                public static Tasks.Task<Result<Socket, SocketError>> Connect(string address, int port) => default;
                public Tasks.Task<Result<int, SocketError>> Receive(byte[] buffer, int offset, int count) => default;
                public Tasks.Task<Result<int, SocketError>> Send(byte[] buffer, int offset, int count) => default;
                public void Close() { }
                public static SocketError DecodeError(byte code) => default;
            }
            public sealed class SocketConnectCompletion {
                public SocketConnectCompletion(Tasks.Promise<Result<Socket, SocketError>> source) { }
                public void Start(string address, int port) { }
                public void Complete() { }
            }
            public sealed class SocketTransferCompletion {
                public SocketTransferCompletion(Tasks.Promise<Result<int, SocketError>> source) { }
                public void StartReceive(long handle, byte[] buffer, int offset, int count) { }
                public void StartSend(long handle, byte[] buffer, int offset, int count) { }
                public void Complete() { }
            }
        }
        """;
    public static void Project(ModuleDefinition module) {
        foreach (var type in module.Types.Where(t => IsName(t.FullName))) {
            if (IsProvider(type)) type.Attributes = (type.Attributes & ~TypeAttributes.VisibilityMask) | TypeAttributes.NotPublic;
            foreach (var method in type.Methods.Where(m => IsProvider(type) || m.IsConstructor || m.Name == "DecodeError"))
                method.Attributes = (method.Attributes & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Assembly;
        }
    }
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct, bool library)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        const string Error = Prefix + "SocketError";
        const string Socket = Prefix + "Socket";
        var expected = (owner, definition.Name) switch {
            (Socket, "Connect") => ("String,Int32", $"System.Tasks.Task<System.Result<{Socket},{Error}>>", true),
            (Socket, "Receive" or "Send") => ("arrayref<Byte>,Int32,Int32", $"System.Tasks.Task<System.Result<Int32,{Error}>>", false),
            (Socket, "Close") => ("", "noresult", false),
            (Socket, ".ctor") when library => ("Int64", "noresult", false),
            (Socket, "DecodeError") when library => ("Byte", Error, true),
            (Prefix + "SocketConnectCompletion", ".ctor") when library => ($"System.Tasks.Promise<System.Result<{Socket},{Error}>>", "noresult", false),
            (Prefix + "SocketTransferCompletion", ".ctor") when library => ($"System.Tasks.Promise<System.Result<Int32,{Error}>>", "noresult", false),
            (Prefix + "SocketConnectCompletion", "Start") when library => ("String,Int32", "noresult", false),
            (Prefix + "SocketTransferCompletion", "StartReceive" or "StartSend") when library => ("Int64,arrayref<Byte>,Int32,Int32", "noresult", false),
            (_, "Complete") when library && IsProvider(definition.DeclaringType) => ("", "noresult", false),
            _ => throw new InvalidDataException("Unsupported socket member.")
        };
        if (definition.IsConstructor != construct || definition.IsStatic != expected.Item3
            || !(definition.IsPublic || library && definition.IsAssembly)
            || definition.IsVirtual || definition.HasGenericParameters || !definition.DeclaringType.IsSealed
            || definition.DeclaringType.HasGenericParameters || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported socket signature.");
        var signature = string.Join(',', args);
        return construct ? new(args, owner, $"newobj instance {owner}::.ctor({signature})")
            : new(definition.IsStatic ? args : new[] { owner }.Concat(args).ToArray(), result,
                $"call {(definition.IsStatic ? "" : "instance ")}{owner}::{definition.Name}({signature})");
    }
}
