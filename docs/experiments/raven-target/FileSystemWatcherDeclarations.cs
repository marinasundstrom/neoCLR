// Documentation/compiler-reference projection only; these bodies never execute.
static class FileSystemWatcherDeclarations
{
    public const string Source = """
    namespace IO {
        public enum WatcherChangeTypes { Created = 1, Deleted = 2, Changed = 4, Renamed = 8, All = 15 }
        public enum FileSystemWatcherError { InvalidPath = 1, NotFound = 2, AccessDenied = 3, IoFailure = 4, Closed = 5, BufferOverflow = 6, InvalidEncoding = 7, PlatformNotSupported = 8, ResourceLimit = 9 }
        public class FileSystemEventArgs {
            internal FileSystemEventArgs() { }
            public WatcherChangeTypes ChangeType { get; }
            public string FullPath { get; }
            public string Name { get; }
        }
        public sealed class RenamedEventArgs : FileSystemEventArgs {
            internal RenamedEventArgs() { }
            public string OldFullPath { get; }
            public string OldName { get; }
        }
        public sealed class ErrorEventArgs {
            internal ErrorEventArgs() { }
            public FileSystemWatcherError Error { get; }
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
            public Disposable OnCreated(Action<FileSystemWatcher, FileSystemEventArgs> callback) => default;
            public Disposable OnChanged(Action<FileSystemWatcher, FileSystemEventArgs> callback) => default;
            public Disposable OnDeleted(Action<FileSystemWatcher, FileSystemEventArgs> callback) => default;
            public Disposable OnRenamed(Action<FileSystemWatcher, RenamedEventArgs> callback) => default;
            public Disposable OnError(Action<FileSystemWatcher, ErrorEventArgs> callback) => default;
        }
    }
    """;
}
