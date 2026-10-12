// Declaration-only API reference projection. Native Raven library bodies execute.
static class CompressionDeclarations
{
    public const string Source = """
namespace IO.Compression {
    public class GZipStream : InputStream, OutputStream {
        private GZipStream() { }
        public static GZipStream CreateCompressor(OutputStream output) => default;
        public static GZipStream CreateCompressor(OutputStream output, bool leaveOpen) => default;
        public static GZipStream CreateDecompressor(InputStream input) => default;
        public static GZipStream CreateDecompressor(InputStream input, bool leaveOpen) => default;
        public Concurrency.Tasks.Task<Result<int, StreamError>> Read(byte[] buffer, int offset, int count) => default;
        public Concurrency.Tasks.Task<Result<int, StreamError>> Read(byte[] buffer, int offset, int count, Concurrency.CancellationToken cancellationToken) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Write(byte[] buffer, int offset, int count) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Write(byte[] buffer, int offset, int count, Concurrency.CancellationToken cancellationToken) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Flush() => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Flush(Concurrency.CancellationToken cancellationToken) => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Finish() => default;
        public Concurrency.Tasks.Task<Result<PropagationUnit, StreamError>> Finish(Concurrency.CancellationToken cancellationToken) => default;
        public void Close() { }
    }
}
""";
}
