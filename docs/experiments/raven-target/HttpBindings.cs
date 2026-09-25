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
            public enum HttpStatusCode { Continue = 100, SwitchingProtocols = 101, OK = 200, Created = 201, Accepted = 202, NoContent = 204, ResetContent = 205, PartialContent = 206, MultipleChoices = 300, MovedPermanently = 301, Found = 302, SeeOther = 303, NotModified = 304, TemporaryRedirect = 307, PermanentRedirect = 308, BadRequest = 400, Unauthorized = 401, Forbidden = 403, NotFound = 404, MethodNotAllowed = 405, RequestTimeout = 408, Conflict = 409, Gone = 410, LengthRequired = 411, PreconditionFailed = 412, RequestEntityTooLarge = 413, RequestUriTooLong = 414, UnsupportedMediaType = 415, RequestedRangeNotSatisfiable = 416, ExpectationFailed = 417, UnprocessableContent = 422, TooManyRequests = 429, InternalServerError = 500, NotImplemented = 501, BadGateway = 502, ServiceUnavailable = 503, GatewayTimeout = 504, HttpVersionNotSupported = 505 }
            public struct HttpError { public struct InvalidUri { } public struct NameResolution { } public struct Transport { } public struct InvalidRequest { } public struct Protocol { } public struct Unsupported { } public struct LimitExceeded { } public struct TimedOut { } public struct Handler { } public struct UnsuccessfulStatus { } }
            public interface HttpHandler {
                Tasks.Task<Result<HttpResponse, HttpError>> Send(HttpRequest request, Concurrency.CancellationToken cancellationToken);
            }
            public sealed class HttpClient {
                public HttpClient() { }
                public HttpClient(HttpHandler handler) { }
                public Option<string> BaseUri { get; set; }
                public Tasks.Task<Result<HttpResponse, HttpError>> Get(string url) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Get(string url, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<string, HttpError>> GetString(string url) => default;
                public Tasks.Task<Result<string, HttpError>> GetString(string url, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Get(Uri uri) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Get(Uri uri, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<string, HttpError>> GetString(Uri uri) => default;
                public Tasks.Task<Result<string, HttpError>> GetString(Uri uri, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Post(string url, HttpContent content) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Post(string url, HttpContent content, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Post(Uri uri, HttpContent content) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Post(Uri uri, HttpContent content, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Put(string url, HttpContent content) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Put(string url, HttpContent content, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Put(Uri uri, HttpContent content) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Put(Uri uri, HttpContent content, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Patch(string url, HttpContent content) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Patch(string url, HttpContent content, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Patch(Uri uri, HttpContent content) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Patch(Uri uri, HttpContent content, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Delete(string url) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Delete(string url, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Delete(Uri uri) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Delete(Uri uri, Concurrency.CancellationToken cancellationToken) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Send(HttpRequest request) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Send(HttpRequest request, Concurrency.CancellationToken cancellationToken) => default;
            }
            public sealed class HttpSocketHandler : HttpHandler {
                public HttpSocketHandler() { }
                public Tasks.Task<Result<HttpResponse, HttpError>> Send(HttpRequest request) => default;
                public Tasks.Task<Result<HttpResponse, HttpError>> Send(HttpRequest request, Concurrency.CancellationToken cancellationToken) => default;
            }
            public sealed class HttpHeader {
                public HttpHeader(string name, string value) { }
                public string Name => default;
                public string Value => default;
                public static Collections.Sequence<string> FindValues(Collections.Sequence<HttpHeader> headers, string name) => default;
            }
            public sealed class HttpRequest {
                private HttpRequest(string host, int port, string target) { }
                public static Result<HttpRequest, HttpError> Get(string url) => default;
                public static Result<HttpRequest, HttpError> Get(Uri uri) => default;
                public static Result<HttpRequest, HttpError> Post(string url, HttpContent content) => default;
                public static Result<HttpRequest, HttpError> Post(Uri uri, HttpContent content) => default;
                public static Result<HttpRequest, HttpError> Put(string url, HttpContent content) => default;
                public static Result<HttpRequest, HttpError> Put(Uri uri, HttpContent content) => default;
                public static Result<HttpRequest, HttpError> Patch(string url, HttpContent content) => default;
                public static Result<HttpRequest, HttpError> Patch(Uri uri, HttpContent content) => default;
                public static Result<HttpRequest, HttpError> Delete(string url) => default;
                public static Result<HttpRequest, HttpError> Delete(Uri uri) => default;
                public HttpContent Content => default;
                public Collections.Sequence<HttpHeader> Headers => default;
                public Collections.Sequence<string> GetHeaderValues(string name) => default;
                public Result<HttpRequest, HttpError> WithHeader(string name, string value) => default;
                public Result<Collections.Sequence<byte>, HttpError> Encode() => default;
                public static Result<HttpRequest, HttpError> FromIncoming(string method, string target, string host, Collections.Sequence<HttpHeader> headers, Collections.Sequence<byte> body) => default;
                public string Method => default;
                public string Host => default;
                public int Port => default;
                public string Target => default;
            }
            public sealed class HttpResponse {
                public HttpResponse(int statusCode, Collections.Sequence<HttpHeader> headers, Collections.Sequence<byte> body) { }
                public HttpResponse(HttpStatusCode statusCode, Collections.Sequence<HttpHeader> headers, Collections.Sequence<byte> body) { }
                public HttpStatusCode StatusCode => default;
                public bool IsSuccessStatusCode => default;
                public Collections.Sequence<HttpHeader> Headers => default;
                public Collections.Sequence<string> GetHeaderValues(string name) => default;
                public HttpContent Content => default;
            }
            public sealed class HttpContent {
                public HttpContent(Collections.Sequence<byte> bytes) { }
                public HttpContent(Collections.Sequence<byte> bytes, string contentType) { }
                public static HttpContent FromText(string text) => default;
                public Option<string> MediaType => default;
                public Collections.Sequence<byte> Bytes => default;
                public Tasks.Task<Result<string, string>> ReadText() => default;
            }
            public sealed class HttpServer {
                public HttpServer(Networking.Sockets.Socket listener) { }
                public static Result<HttpServer, HttpError> Listen(string address, int port, int backlog) => default;
                public Result<int, HttpError> GetLocalPort() => default;
                public Tasks.Task<Result<PropagationUnit, HttpError>> ServeOne(Func<HttpRequest, Tasks.Task<Result<HttpResponse, HttpError>>> handler) => default;
                public void Close() { }
                public static Result<Collections.Sequence<byte>, HttpError> EncodeResponse(HttpResponse response) => default;
            }
            public sealed class HttpRequestDecoder {
                public HttpRequestDecoder() { }
                public bool Complete => default;
                public Result<bool, HttpError> Push(byte value) => default;
                public Result<HttpRequest, HttpError> Finish() => default;
            }
            public sealed class HttpServerExchange {
                public HttpServerExchange(Networking.Sockets.Socket listener, Func<HttpRequest, Tasks.Task<Result<HttpResponse, HttpError>>> handler) { }
                public Tasks.Task<Result<PropagationUnit, HttpError>> Start() => default;
            }
            public sealed class HttpResponseDecoder {
                public HttpResponseDecoder() { }
                public bool Complete => default;
                public Result<bool, HttpError> Push(byte value) => default;
                public Result<HttpResponse, HttpError> Finish() => default;
            }
            public sealed class HttpExchange {
                public HttpExchange(HttpRequest request, Concurrency.CancellationToken token) { }
                public Tasks.Task<Result<HttpResponse, HttpError>> Execute() => default;
            }
        }
        """;
    public static void Project(ModuleDefinition module)
    {
        foreach (var type in module.Types.Where(t => IsName(t.FullName)))
            foreach (var method in type.Methods.Where(m => m.Name == "FindValues" || m.Name == "FromIncoming" || m.Name == "EncodeResponse" || type.Name == "HttpRequest" && m.Name == "Encode" || type.Name == "HttpContent" && m.Name == "get_MediaType" || type.Name == "HttpServer" && m.IsConstructor))
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
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type, allowInternal: library);
        var request = Prefix + "HttpRequest";
        var response = Prefix + "HttpResponse";
        var outcome = $"System.Result<{response},System.Web.Http.HttpError>";
        var task = $"System.Tasks.Task<{outcome}>";
        var expected = (owner[Prefix.Length..], definition.Name) switch {
            ("HttpServer", ".ctor") when library => ("System.Networking.Sockets.Socket", "noresult", false),
            ("HttpServer", "Listen") => ("String,Int32,Int32", $"System.Result<{Prefix}HttpServer,System.Web.Http.HttpError>", true),
            ("HttpServer", "GetLocalPort") => ("", "System.Result<Int32,System.Web.Http.HttpError>", false),
            ("HttpServer", "Close") => ("", "noresult", false),
            ("HttpServer", "ServeOne") => ($"System.Func<{request},{task}>", "System.Tasks.Task<System.Result<Void,System.Web.Http.HttpError>>", false),
            ("HttpServer", "EncodeResponse") when library => (response, "System.Result<System.Collections.Sequence<Byte>,System.Web.Http.HttpError>", true),
            ("HttpRequest", "FromIncoming") when library => ($"String,String,String,System.Collections.Sequence<{Prefix}HttpHeader>,System.Collections.Sequence<Byte>", $"System.Result<{request},System.Web.Http.HttpError>", true),
            ("HttpHeader", "FindValues") when library => ($"System.Collections.Sequence<{Prefix}HttpHeader>,String", "System.Collections.Sequence<String>", true),
            ("HttpRequest" or "HttpResponse", "GetHeaderValues") => ("String", "System.Collections.Sequence<String>", false),
            ("HttpRequest", "get_Headers") => ("", $"System.Collections.Sequence<{Prefix}HttpHeader>", false),
            ("HttpRequestDecoder", ".ctor") when library => ("", "noresult", false),
            ("HttpRequestDecoder", "get_Complete") when library => ("", "Boolean", false),
            ("HttpRequestDecoder", "Push") when library => ("Byte", "System.Result<Boolean,System.Web.Http.HttpError>", false),
            ("HttpRequestDecoder", "Finish") when library => ("", $"System.Result<{request},System.Web.Http.HttpError>", false),
            ("HttpServerExchange", ".ctor") when library => ($"System.Networking.Sockets.Socket,System.Func<{request},{task}>", "noresult", false),
            ("HttpServerExchange", "Start") when library => ("", "System.Tasks.Task<System.Result<Void,System.Web.Http.HttpError>>", false),
            ("HttpClient", ".ctor") when args.Length == 0 => ("", "noresult", false),
            ("HttpClient", ".ctor") => (Prefix + "HttpHandler", "noresult", false),
            ("HttpClient", "Get" or "GetString" or "Delete") => (args.Length > 0 && args[0] == "String" ? "String" : "System.Uri",
                definition.Name != "GetString" ? task : "System.Tasks.Task<System.Result<String,System.Web.Http.HttpError>>", false),
            ("HttpClient", "Post" or "Put" or "Patch") => ((args.Length > 0 && args[0] == "String" ? "String" : "System.Uri") + "," + Prefix + "HttpContent", task, false),
            ("HttpRequest", "Post" or "Put" or "Patch") => ((args.Length > 0 && args[0] == "String" ? "String" : "System.Uri") + "," + Prefix + "HttpContent", $"System.Result<{request},System.Web.Http.HttpError>", true),
            ("HttpRequest", "get_Content") => ("", Prefix + "HttpContent", false),
            ("HttpRequest", "WithHeader") => ("String,String", $"System.Result<{request},System.Web.Http.HttpError>", false),
            ("HttpRequest", "Encode") when library => ("", "System.Result<System.Collections.Sequence<Byte>,System.Web.Http.HttpError>", false),
            ("HttpContent", "get_MediaType") when library => ("", "System.Option<String>", false),
            ("HttpContent", "FromText") => ("String", Prefix + "HttpContent", true),
            ("HttpClient", "get_BaseUri") => ("", "System.Option<String>", false),
            ("HttpClient", "set_BaseUri") => ("System.Option<String>", "noresult", false),
            ("HttpHandler", "Send") => (request + ",System.Concurrency.CancellationToken", task, false),
            ("HttpClient" or "HttpSocketHandler", "Send") => (request, task, false),
            ("HttpSocketHandler" or "HttpResponseDecoder", ".ctor") => ("", "noresult", false),
            ("HttpHeader", ".ctor") => ("String,String", "noresult", false),
            ("HttpHeader", "get_Name" or "get_Value") => ("", "String", false),
            ("HttpRequest", "Get" or "Delete") => (args.Length == 1 && args[0] == "String" ? "String" : "System.Uri", $"System.Result<{request},System.Web.Http.HttpError>", true),
            ("HttpRequest", "get_Method" or "get_Host" or "get_Target") => ("", "String", false),
            ("HttpRequest", "get_Port") => ("", "Int32", false),
            ("HttpResponse", ".ctor") => ($"{(definition.Parameters[0].ParameterType.FullName == Prefix + "HttpStatusCode" ? Prefix + "HttpStatusCode" : "Int32")},System.Collections.Sequence<{Prefix}HttpHeader>,System.Collections.Sequence<Byte>", "noresult", false),
            ("HttpResponse", "get_IsSuccessStatusCode") => ("", "Boolean", false),
            ("HttpResponse", "get_StatusCode") => ("", Prefix + "HttpStatusCode", false),
            ("HttpResponse", "get_Headers") => ("", $"System.Collections.Sequence<{Prefix}HttpHeader>", false),
            ("HttpResponse", "get_Content") => ("", Prefix + "HttpContent", false),
            ("HttpContent", ".ctor") => (args.Length == 2 ? "System.Collections.Sequence<Byte>,String" : "System.Collections.Sequence<Byte>", "noresult", false),
            ("HttpContent", "get_Bytes") => ("", "System.Collections.Sequence<Byte>", false),
            ("HttpContent", "ReadText") => ("", "System.Tasks.Task<System.Result<String,String>>", false),
            ("HttpResponseDecoder", "get_Complete") when library => ("", "Boolean", false),
            ("HttpResponseDecoder", "Push") when library => ("Byte", "System.Result<Boolean,System.Web.Http.HttpError>", false),
            ("HttpResponseDecoder", "Finish") when library => ("", outcome, false),
            ("HttpExchange", ".ctor") when library => (request + ",System.Concurrency.CancellationToken", "noresult", false),
            ("HttpExchange", "Execute") when library => ("", task, false),
            _ => throw new InvalidDataException("Unsupported HTTP member.")
        };
        if (owner is Prefix + "HttpClient" or Prefix + "HttpSocketHandler"
            && definition.Name is "Get" or "GetString" or "Send" or "Post" or "Put" or "Patch" or "Delete"
            && args.LastOrDefault() == CancellationBindings.Token)
            expected.Item1 += "," + CancellationBindings.Token;
        var virtualMember = IsContract(owner) || owner == Prefix + "HttpSocketHandler"
            && definition.Name == "Send" && args.Length == 2;
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
