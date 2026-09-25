using Mono.Cecil;

// Provisional TCP client surface. Private operation IDs never enter application APIs.
static class SocketBindings
{
    const string Prefix = "System.Networking.Sockets.";
    public static bool IsName(string name) => name is "System.Networking.Dns" or "System.Networking.DnsCompletion" or Prefix + "Socket" or Prefix + "SocketConnectCompletion" or Prefix + "SocketTransferCompletion";
    public static bool IsProvider(TypeDefinition type) => IsName(type.FullName) && type.FullName != Prefix + "Socket" && type.FullName != "System.Networking.Dns";
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope) && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace Networking {
            public sealed class Dns {
                public static Tasks.Task<Result<Collections.Sequence<IPAddress>, DnsError>> GetHostAddressesUntil(string hostName, long deadline) => default;
                public static Tasks.Task<Result<Collections.Sequence<IPAddress>, DnsError>> GetHostAddressesUntil(string hostName, long deadline, Concurrency.CancellationToken cancellationToken) => default;
                public static Tasks.Task<Result<Collections.Sequence<IPAddress>, DnsError>> GetHostAddresses(string hostName) => default;
                public static Tasks.Task<Result<Collections.Sequence<IPAddress>, DnsError>> GetHostAddresses(string hostName, Concurrency.CancellationToken cancellationToken) => default;
                public static DnsError DecodeError(byte code) => default;
            }
            public sealed class DnsCompletion {
                public void StartUntil(string hostName, long deadline) { }
                public DnsCompletion(Tasks.Promise<Result<Collections.Sequence<IPAddress>, DnsError>> source, Concurrency.CancellationToken token) { }
                public void Start(string hostName) { }
                public void RequestCancellation() { }
                public void Complete() { }
            }
        }
        namespace Networking.Sockets {
            public sealed class Socket {
                public static Tasks.Task<Result<Socket, SocketError>> ConnectUntil(Collections.Sequence<string> addresses, int port, long deadline) => default;
                public static Tasks.Task<Result<Socket, SocketError>> ConnectUntil(Collections.Sequence<string> addresses, int port, long deadline, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<int, SocketError>> ReceiveUntil(byte[] buffer, int offset, int count, long deadline) => default;
                public Tasks.Task<Result<int, SocketError>> ReceiveUntil(byte[] buffer, int offset, int count, long deadline, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<int, SocketError>> SendUntil(byte[] buffer, int offset, int count, long deadline) => default;
                public Tasks.Task<Result<int, SocketError>> SendUntil(byte[] buffer, int offset, int count, long deadline, Concurrency.CancellationToken cancellationToken) => default;
                public Socket(long handle) { }
                public static Tasks.Task<Result<Socket, SocketError>> Connect(Networking.IPAddress address, int port) => default;
                public static Tasks.Task<Result<Socket, SocketError>> Connect(Networking.IPAddress address, int port, Concurrency.CancellationToken cancellationToken) => default;
                public static Tasks.Task<Result<Socket, SocketError>> Connect(Collections.Sequence<Networking.IPAddress> addresses, int port) => default;
                public static Tasks.Task<Result<Socket, SocketError>> Connect(Collections.Sequence<Networking.IPAddress> addresses, int port, Concurrency.CancellationToken cancellationToken) => default;
                public static Tasks.Task<Result<Socket, SocketError>> ConnectUntil(Collections.Sequence<Networking.IPAddress> addresses, int port, long deadline) => default;
                public static Tasks.Task<Result<Socket, SocketError>> ConnectUntil(Collections.Sequence<Networking.IPAddress> addresses, int port, long deadline, Concurrency.CancellationToken cancellationToken) => default;
                public static Result<Socket, SocketError> Listen(Networking.IPAddress address, int port, int backlog) => default;
                public static Tasks.Task<Result<Socket, SocketError>> Connect(string address, int port) => default;
                public static Tasks.Task<Result<Socket, SocketError>> Connect(string address, int port, Concurrency.CancellationToken cancellationToken) => default;
                public static Tasks.Task<Result<Socket, SocketError>> Connect(Collections.Sequence<string> addresses, int port) => default;
                public static Tasks.Task<Result<Socket, SocketError>> Connect(Collections.Sequence<string> addresses, int port, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<int, SocketError>> Receive(byte[] buffer, int offset, int count) => default;
                public Tasks.Task<Result<int, SocketError>> Receive(byte[] buffer, int offset, int count, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<int, SocketError>> Send(byte[] buffer, int offset, int count) => default;
                public Tasks.Task<Result<int, SocketError>> Send(byte[] buffer, int offset, int count, Concurrency.CancellationToken cancellationToken) => default;
                public static Result<Socket, SocketError> Listen(string address, int port, int backlog) => default;
                public Tasks.Task<Result<Socket, SocketError>> Accept() => default;
                public Tasks.Task<Result<Socket, SocketError>> Accept(Concurrency.CancellationToken cancellationToken) => default;
                public Result<int, SocketError> GetLocalPort() => default;
                public void Close() { }
                public static SocketError DecodeError(byte code) => default;
            }
            public sealed class SocketConnectCompletion {
                public void StartAddressesUntil(Collections.Sequence<string> addresses, int port, long deadline) { }
                public SocketConnectCompletion(Tasks.Promise<Result<Socket, SocketError>> source, Concurrency.CancellationToken token) { }
                public void Start(string address, int port) { }
                public void StartAddresses(Collections.Sequence<string> addresses, int port) { }
                public void StartAccept(long handle) { }
                public void RequestCancellation() { }
                public void Complete() { }
            }
            public sealed class SocketTransferCompletion {
                public void StartReceiveUntil(long handle, byte[] buffer, int offset, int count, long deadline) { }
                public void StartSendUntil(long handle, byte[] buffer, int offset, int count, long deadline) { }
                public SocketTransferCompletion(Tasks.Promise<Result<int, SocketError>> source, Concurrency.CancellationToken token) { }
                public void StartReceive(long handle, byte[] buffer, int offset, int count) { }
                public void StartSend(long handle, byte[] buffer, int offset, int count) { }
                public void RequestCancellation() { }
                public void Complete() { }
            }
        }
        """;
    public static void Project(ModuleDefinition module, bool libraryBootstrap = false) {
        foreach (var type in module.Types.Where(t => IsName(t.FullName))) {
            if (IsProvider(type)) type.Attributes = (type.Attributes & ~TypeAttributes.VisibilityMask) | TypeAttributes.NotPublic;
            foreach (var method in type.Methods.Where(m => IsProvider(type) || m.IsConstructor || m.Name == "DecodeError" || m.Name.EndsWith("Until", StringComparison.Ordinal)))
                method.Attributes = (method.Attributes & ~MethodAttributes.MemberAccessMask)
                    | (libraryBootstrap && !IsProvider(type) && method.Name.EndsWith("Until", StringComparison.Ordinal)
                        ? MethodAttributes.Public : MethodAttributes.Assembly);
        }
    }
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct, bool library)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type, allowInternal: library);
        const string Error = Prefix + "SocketError";
        const string Socket = Prefix + "Socket";
        var expected = (owner, definition.Name) switch {
            ("System.Networking.Dns", "GetHostAddressesUntil") when library => ("String,Int64", "System.Tasks.Task<System.Result<System.Collections.Sequence<System.Networking.IPAddress>,System.Networking.DnsError>>", true),
            ("System.Networking.DnsCompletion", "StartUntil") when library => ("String,Int64", "noresult", false),
            (Socket, "ConnectUntil") when library && args[0] == "System.Collections.Sequence<System.Networking.IPAddress>" => ("System.Collections.Sequence<System.Networking.IPAddress>,Int32,Int64", $"System.Tasks.Task<System.Result<{Socket},{Error}>>", true),
            (Socket, "ConnectUntil") when library => ("System.Collections.Sequence<String>,Int32,Int64", $"System.Tasks.Task<System.Result<{Socket},{Error}>>", true),
            (Socket, "ReceiveUntil" or "SendUntil") when library => ("arrayref<Byte>,Int32,Int32,Int64", $"System.Tasks.Task<System.Result<Int32,{Error}>>", false),
            (Prefix + "SocketConnectCompletion", "StartAddressesUntil") when library => ("System.Collections.Sequence<String>,Int32,Int64", "noresult", false),
            (Prefix + "SocketTransferCompletion", "StartReceiveUntil" or "StartSendUntil") when library => ("Int64,arrayref<Byte>,Int32,Int32,Int64", "noresult", false),
            ("System.Networking.Dns", "GetHostAddresses") => ("String", "System.Tasks.Task<System.Result<System.Collections.Sequence<System.Networking.IPAddress>,System.Networking.DnsError>>", true),
            ("System.Networking.Dns", "DecodeError") when library => ("Byte", "System.Networking.DnsError", true),
            ("System.Networking.DnsCompletion", ".ctor") when library => ("System.Tasks.Promise<System.Result<System.Collections.Sequence<System.Networking.IPAddress>,System.Networking.DnsError>>,System.Concurrency.CancellationToken", "noresult", false),
            ("System.Networking.DnsCompletion", "Start") when library => ("String", "noresult", false),
            (Socket, "Listen") when args[0] == IPAddressBindings.Root => (IPAddressBindings.Root + ",Int32,Int32", $"System.Result<{Socket},{Error}>", true),
            (Socket, "Listen") => ("String,Int32,Int32", $"System.Result<{Socket},{Error}>", true),
            (Socket, "Accept") => ("", $"System.Tasks.Task<System.Result<{Socket},{Error}>>", false),
            (Socket, "GetLocalPort") => ("", $"System.Result<Int32,{Error}>", false),
            (Prefix + "SocketConnectCompletion", "StartAccept") when library => ("Int64", "noresult", false),
            (Socket, "Connect") when args[0] == IPAddressBindings.Root => (IPAddressBindings.Root + ",Int32", $"System.Tasks.Task<System.Result<{Socket},{Error}>>", true),
            (Socket, "Connect") when args[0] == "System.Collections.Sequence<System.Networking.IPAddress>" => ("System.Collections.Sequence<System.Networking.IPAddress>,Int32", $"System.Tasks.Task<System.Result<{Socket},{Error}>>", true),
            (Socket, "Connect") when args[0] == "System.Collections.Sequence<String>" => ("System.Collections.Sequence<String>,Int32", $"System.Tasks.Task<System.Result<{Socket},{Error}>>", true),
            (Prefix + "SocketConnectCompletion", "StartAddresses") when library => ("System.Collections.Sequence<String>,Int32", "noresult", false),
            (Socket, "Connect") => ("String,Int32", $"System.Tasks.Task<System.Result<{Socket},{Error}>>", true),
            (Socket, "Receive" or "Send") => ("arrayref<Byte>,Int32,Int32", $"System.Tasks.Task<System.Result<Int32,{Error}>>", false),
            (Socket, "Close") => ("", "noresult", false),
            (Socket, ".ctor") when library => ("Int64", "noresult", false),
            (Socket, "DecodeError") when library => ("Byte", Error, true),
            (Prefix + "SocketConnectCompletion", ".ctor") when library => ($"System.Tasks.Promise<System.Result<{Socket},{Error}>>,System.Concurrency.CancellationToken", "noresult", false),
            (Prefix + "SocketTransferCompletion", ".ctor") when library => ($"System.Tasks.Promise<System.Result<Int32,{Error}>>,System.Concurrency.CancellationToken", "noresult", false),
            (Prefix + "SocketConnectCompletion", "Start") when library => ("String,Int32", "noresult", false),
            (Prefix + "SocketTransferCompletion", "StartReceive" or "StartSend") when library => ("Int64,arrayref<Byte>,Int32,Int32", "noresult", false),
            (_, "Complete" or "RequestCancellation") when library && IsProvider(definition.DeclaringType) => ("", "noresult", false),
            _ => throw new InvalidDataException("Unsupported socket member.")
        };
        // Only selected operation overloads admit a trailing public cancellation token.
        if (owner is "System.Networking.Dns" or Socket
            && definition.Name is "GetHostAddresses" or "GetHostAddressesUntil" or "Connect" or "ConnectUntil" or "Accept" or "Receive" or "ReceiveUntil" or "Send" or "SendUntil"
            && args.LastOrDefault() == "System.Concurrency.CancellationToken")
            expected.Item1 += (expected.Item1.Length == 0 ? "" : ",") + "System.Concurrency.CancellationToken";
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
