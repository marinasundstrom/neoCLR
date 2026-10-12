// Declaration-only reference projection; implementation is native Raven source.
static class SslDeclarations
{
    public const string Source = """
namespace Networking.Security {
    public struct SslError {
        public struct InvalidCertificate { }
        public struct InvalidPrivateKey { }
        public struct KeyMismatch { }
        public struct InvalidTrust { }
        public struct Closed { }
        public struct AuthenticationFailed { }
        public struct InvalidConfiguration { }
    }
    public class SslClientAuthenticationOptions : Disposable {
        private SslClientAuthenticationOptions() { }
        public static Result<SslClientAuthenticationOptions, SslError> Create(byte[] trustPem) => default;
        public void Dispose() { }
    }
    public class SslServerAuthenticationOptions : Disposable {
        private SslServerAuthenticationOptions() { }
        public static Result<SslServerAuthenticationOptions, SslError> Create(byte[] certificatePem, byte[] privateKeyPem) => default;
        public void Dispose() { }
    }
    public class SslStream : IO.InputStream, IO.OutputStream, Disposable {
        public SslStream(IO.InputStream input, IO.OutputStream output) { }
        public SslStream(IO.InputStream input, IO.OutputStream output, bool leaveOpen) { }
        public bool IsAuthenticated => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, SslError>> AuthenticateAsClient(SslClientAuthenticationOptions options, string targetHost) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, SslError>> AuthenticateAsClient(SslClientAuthenticationOptions options, string targetHost, Concurrency.CancellationToken cancellationToken) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, SslError>> AuthenticateAsServer(SslServerAuthenticationOptions options) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, SslError>> AuthenticateAsServer(SslServerAuthenticationOptions options, Concurrency.CancellationToken cancellationToken) => default;
        public Concurrency.Tasks.Task<Result<int, IO.StreamError>> Read(byte[] buffer, int offset, int count) => default;
        public Concurrency.Tasks.Task<Result<int, IO.StreamError>> Read(byte[] buffer, int offset, int count, Concurrency.CancellationToken cancellationToken) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, IO.StreamError>> Write(byte[] buffer, int offset, int count) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, IO.StreamError>> Write(byte[] buffer, int offset, int count, Concurrency.CancellationToken cancellationToken) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, IO.StreamError>> Flush() => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, IO.StreamError>> Flush(Concurrency.CancellationToken cancellationToken) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, IO.StreamError>> Shutdown() => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, IO.StreamError>> Shutdown(Concurrency.CancellationToken cancellationToken) => default;
        public void Close() { }
        public void Dispose() { }
    }
}
""";
}
