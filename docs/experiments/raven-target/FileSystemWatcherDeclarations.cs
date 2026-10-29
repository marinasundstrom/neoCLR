// Documentation/compiler-reference projection only; these bodies never execute.
static class FileSystemWatcherDeclarations
{
    public const string Source = """
    namespace IO {
        public enum FileSystemWatcherError { InvalidPath = 1, NotFound = 2, AccessDenied = 3, IoFailure = 4, Closed = 5, BufferOverflow = 6, InvalidEncoding = 7, PlatformNotSupported = 8, ResourceLimit = 9 }
        public sealed class FileSystemChange {
            internal FileSystemChange() { }
            public string FullPath { get; }
            public string Name { get; }
        }
        public sealed class FileSystemRename {
            internal FileSystemRename() { }
            public FileSystemChange OldPath { get; }
            public FileSystemChange NewPath { get; }
        }
        public struct FileSystemNotification {
            public struct Created { }
            public struct Changed { }
            public struct Deleted { }
            public struct Renamed { }
            public struct RescanRequired { }
        }
        public sealed class FileSystemWatcher : Disposable {
            public FileSystemWatcher(string path) { }
            public FileSystemWatcher(string path, string filter) { }
            public string Path { get; set; }
            public string Filter { get; set; }
            public bool IncludeSubdirectories { get; set; }
            public bool EnableRaisingEvents { get; set; }
            public Result<PropagationUnit, FileSystemWatcherError> TryStart() => default;
            public void Stop() { }
            public void Dispose() { }
            public Disposable Subscribe(Action<FileSystemNotification> callback) => default;
        }
    }
    """;
}
