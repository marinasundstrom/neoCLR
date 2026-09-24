using Mono.Cecil;

// Bounded development HTTP contracts. Transport helpers remain internal.
static class HttpBindings
{
    public const string Prefix = "System.Web.Http.";
    public static readonly string[] Names = ["HttpClient", "HttpHandler", "HttpRequest", "HttpResponse", "HttpContent", "HttpHeader", "HttpSocketHandler", "HttpResponseDecoder", "HttpExchange", "HttpServer", "HttpRequestDecoder", "HttpServerExchange"];
    public static bool IsName(string name) => Names.Any(n => name == Prefix + n);
    public static bool IsContract(string name) => name == Prefix + "HttpHandler";
    public static bool IsProvider(TypeDefinition type) => type.FullName is Prefix + "HttpResponseDecoder" or Prefix + "HttpExchange" or Prefix + "HttpRequestDecoder" or Prefix + "HttpServerExchange";
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope) && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
        namespace Web.Http {
            public interface HttpHandler {
                Tasks.Task<Result<HttpResponse, string>> Send(HttpRequest request);
            }
            public sealed class HttpClient {
                public HttpClient() { }
                public HttpClient(HttpHandler handler) { }
                public Tasks.Task<Result<HttpResponse, string>> Get(string url) => default;
                public Tasks.Task<Result<HttpResponse, string>> Send(HttpRequest request) => default;
            }
            public sealed class HttpSocketHandler : HttpHandler {
                public HttpSocketHandler() { }
                public Tasks.Task<Result<HttpResponse, string>> Send(HttpRequest request) => default;
            }
            public sealed class HttpHeader {
                public HttpHeader(string name, string value) { }
                public string Name => default;
                public string Value => default;
            }
            public sealed class HttpRequest {
                private HttpRequest(string host, int port, string target) { }
                public static Result<HttpRequest, string> Get(string url) => default;
                public Collections.Sequence<HttpHeader> Headers => default;
                public static Result<HttpRequest, string> FromIncoming(string target, string host, Collections.Sequence<HttpHeader> headers) => default;
                public string Method => default;
                public string Host => default;
                public int Port => default;
                public string Target => default;
            }
            public sealed class HttpResponse {
                public HttpResponse(int statusCode, Collections.Sequence<HttpHeader> headers, Collections.Sequence<byte> body) { }
                public int StatusCode => default;
                public Collections.Sequence<HttpHeader> Headers => default;
                public HttpContent Content => default;
            }
            public sealed class HttpContent {
                public HttpContent(Collections.Sequence<byte> bytes) { }
                public Collections.Sequence<byte> Bytes => default;
                public Tasks.Task<Result<string, string>> ReadText() => default;
            }
            public sealed class HttpServer {
                public HttpServer(Networking.Sockets.Socket listener) { }
                public static Result<HttpServer, string> Listen(string address, int port, int backlog) => default;
                public Result<int, string> GetLocalPort() => default;
                public Tasks.Task<Result<PropagationUnit, string>> ServeOne(Func<HttpRequest, Tasks.Task<Result<HttpResponse, string>>> handler) => default;
                public void Close() { }
                public static Result<Collections.Sequence<byte>, string> EncodeResponse(HttpResponse response) => default;
            }
            public sealed class HttpRequestDecoder {
                public HttpRequestDecoder() { }
                public bool Complete => default;
                public Result<bool, string> Push(byte value) => default;
                public Result<HttpRequest, string> Finish() => default;
            }
            public sealed class HttpServerExchange {
                public HttpServerExchange(Networking.Sockets.Socket listener, Func<HttpRequest, Tasks.Task<Result<HttpResponse, string>>> handler) { }
                public Tasks.Task<Result<PropagationUnit, string>> Start() => default;
            }
            public sealed class HttpResponseDecoder {
                public HttpResponseDecoder() { }
                public bool Complete => default;
                public Result<bool, string> Push(byte value) => default;
                public Result<HttpResponse, string> Finish() => default;
            }
            public sealed class HttpExchange {
                public HttpExchange(HttpRequest request) { }
                public Tasks.Task<Result<HttpResponse, string>> Execute() => default;
            }
        }
        """;
    public static void Project(ModuleDefinition module)
    {
        foreach (var type in module.Types.Where(t => IsName(t.FullName)))
            foreach (var method in type.Methods.Where(m => m.Name == "FromIncoming" || m.Name == "EncodeResponse" || type.Name == "HttpServer" && m.IsConstructor))
                method.Attributes = (method.Attributes & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Assembly;
        foreach (var type in module.Types.Where(IsProvider))
        {
            type.Attributes = (type.Attributes & ~TypeAttributes.VisibilityMask) | TypeAttributes.NotPublic;
            foreach (var method in type.Methods)
                method.Attributes = (method.Attributes & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Assembly;
        }
    }
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct, bool library)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        if (IsProvider(definition.DeclaringType) && !library)
            throw new InvalidDataException("HTTP implementation helpers are internal.");
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var request = Prefix + "HttpRequest";
        var response = Prefix + "HttpResponse";
        var outcome = $"System.Result<{response},String>";
        var task = $"System.Tasks.Task<{outcome}>";
        var expected = (owner[Prefix.Length..], definition.Name) switch {
            ("HttpServer", ".ctor") when library => ("System.Networking.Sockets.Socket", "noresult", false),
            ("HttpServer", "Listen") => ("String,Int32,Int32", $"System.Result<{Prefix}HttpServer,String>", true),
            ("HttpServer", "GetLocalPort") => ("", "System.Result<Int32,String>", false),
            ("HttpServer", "Close") => ("", "noresult", false),
            ("HttpServer", "ServeOne") => ($"System.Func<{request},{task}>", "System.Tasks.Task<System.Result<Void,String>>", false),
            ("HttpServer", "EncodeResponse") when library => (response, "System.Result<System.Collections.Sequence<Byte>,String>", true),
            ("HttpRequest", "FromIncoming") when library => ($"String,String,System.Collections.Sequence<{Prefix}HttpHeader>", $"System.Result<{request},String>", true),
            ("HttpRequest", "get_Headers") => ("", $"System.Collections.Sequence<{Prefix}HttpHeader>", false),
            ("HttpRequestDecoder", ".ctor") when library => ("", "noresult", false),
            ("HttpRequestDecoder", "get_Complete") when library => ("", "Boolean", false),
            ("HttpRequestDecoder", "Push") when library => ("Byte", "System.Result<Boolean,String>", false),
            ("HttpRequestDecoder", "Finish") when library => ("", $"System.Result<{request},String>", false),
            ("HttpServerExchange", ".ctor") when library => ($"System.Networking.Sockets.Socket,System.Func<{request},{task}>", "noresult", false),
            ("HttpServerExchange", "Start") when library => ("", "System.Tasks.Task<System.Result<Void,String>>", false),
            ("HttpClient", ".ctor") when args.Length == 0 => ("", "noresult", false),
            ("HttpClient", ".ctor") => (Prefix + "HttpHandler", "noresult", false),
            ("HttpClient", "Get") => ("String", task, false),
            ("HttpClient" or "HttpHandler" or "HttpSocketHandler", "Send") => (request, task, false),
            ("HttpSocketHandler" or "HttpResponseDecoder", ".ctor") => ("", "noresult", false),
            ("HttpHeader", ".ctor") => ("String,String", "noresult", false),
            ("HttpHeader", "get_Name" or "get_Value") => ("", "String", false),
            ("HttpRequest", "Get") => ("String", $"System.Result<{request},String>", true),
            ("HttpRequest", "get_Method" or "get_Host" or "get_Target") => ("", "String", false),
            ("HttpRequest", "get_Port") => ("", "Int32", false),
            ("HttpResponse", ".ctor") => ($"Int32,System.Collections.Sequence<{Prefix}HttpHeader>,System.Collections.Sequence<Byte>", "noresult", false),
            ("HttpResponse", "get_StatusCode") => ("", "Int32", false),
            ("HttpResponse", "get_Headers") => ("", $"System.Collections.Sequence<{Prefix}HttpHeader>", false),
            ("HttpResponse", "get_Content") => ("", Prefix + "HttpContent", false),
            ("HttpContent", ".ctor") => ("System.Collections.Sequence<Byte>", "noresult", false),
            ("HttpContent", "get_Bytes") => ("", "System.Collections.Sequence<Byte>", false),
            ("HttpContent", "ReadText") => ("", "System.Tasks.Task<System.Result<String,String>>", false),
            ("HttpResponseDecoder", "get_Complete") when library => ("", "Boolean", false),
            ("HttpResponseDecoder", "Push") when library => ("Byte", "System.Result<Boolean,String>", false),
            ("HttpResponseDecoder", "Finish") when library => ("", outcome, false),
            ("HttpExchange", ".ctor") when library => (request, "noresult", false),
            ("HttpExchange", "Execute") when library => ("", task, false),
            _ => throw new InvalidDataException("Unsupported HTTP member.")
        };
        var virtualMember = IsContract(owner) || owner == Prefix + "HttpSocketHandler" && !construct;
        if (definition.IsConstructor != construct || definition.IsStatic != expected.Item3
            || !(definition.IsPublic || library && definition.IsAssembly)
            || definition.IsVirtual != virtualMember || definition.HasGenericParameters
            || definition.DeclaringType.HasGenericParameters
            || (IsContract(owner) ? !definition.IsAbstract || !definition.DeclaringType.IsInterface : !definition.DeclaringType.IsSealed)
            || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported HTTP signature: " + reference.FullName);
        return construct ? new(args, owner, $"newobj instance {owner}::.ctor({string.Join(',', args)})")
            : new(definition.IsStatic ? args : new[] { owner }.Concat(args).ToArray(), result,
                $"{(IsContract(owner) ? "callvirt" : "call")} {(definition.IsStatic ? "" : "instance ")}{owner}::{definition.Name}({string.Join(',', args)})");
    }
    public static ResultBindings.Binding? BindContract(MethodReference reference, MethodDefinition definition)
    {
        var call = Bind(reference, definition, false, false)!;
        return new(reference.DeclaringType.FullName + "::" + reference.Name, call.Arguments, call.Result, Instruction: call.Instruction);
    }
}
