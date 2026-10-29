# FileSystemWatcher

Development API in `System.IO`, declared by System.Runtime. Watches a physical
directory through an invocation-owned runtime adapter and a shared native provider. `System.Storage` retains
its provider-resolved File/Directory model; this API uses native filesystem paths.

## Types and members

| Type/member | Contract |
| --- | --- |
| `FileSystemWatcher(path: string)` | Constructs a stopped watcher; Filter is `*`, IncludeSubdirectories is false. No I/O. |
| `FileSystemWatcher(path: string, filter: string)` | Constructs a stopped watcher with a basename filter. |
| `Path: string` | Read/write physical directory path. Relative paths resolve at TryStart. Change only while stopped. |
| `Filter: string` | Read/write basename pattern, supporting `*` and `?`. Empty and `*.*` match all names. No directory separators or recursive glob syntax. |
| `IncludeSubdirectories: bool` | Read/write recursive watching, configurable while stopped. |
| `EnableRaisingEvents: bool` | Starts/stops; enabling panics on failure. Use TryStart for typed errors. |
| `TryStart() -> Result<unit, FileSystemWatcherError>` | Establishes the OS subscription before success; repeated starts are harmless. Disposed returns Closed. |
| `Stop()` | Cancels delivery, discards queued notifications and closes the native subscription. Retains handlers for restart. Idempotent. |
| `Dispose()` | Stops permanently and releases all registered callbacks. Idempotent. |
| `OnCreated`, `OnChanged`, `OnDeleted` | Accept `(FileSystemWatcher, FileSystemEventArgs) -> Void`; return Disposable registrations. |
| `OnRenamed` | Accepts `(FileSystemWatcher, RenamedEventArgs) -> Void`; returns a Disposable registration. Either old or new basename may match Filter. |
| `OnError` | Accepts `(FileSystemWatcher, ErrorEventArgs) -> Void`; returns a Disposable registration. Re-read state after an error. |
| `FileSystemEventArgs.ChangeType` | Created, Deleted, Changed or Renamed. |
| `FileSystemEventArgs.FullPath`, `Name` | Absolute physical path and path relative to the watched root; platform directory separators. |
| `RenamedEventArgs.OldFullPath`, `OldName` | Previous absolute and root-relative paths; inherits FileSystemEventArgs. |
| `ErrorEventArgs.Error` | Typed FileSystemWatcherError; no exception payload. |

Event argument constructors are internal. `WatcherChangeTypes` uses Created=1,
Deleted=2, Changed=4, Renamed=8, All=15. All is provided for familiar naming; this
slice does not expose a change-type mask or WaitForChanged.

`FileSystemWatcherError` values are InvalidPath=1, NotFound=2, AccessDenied=3,
IoFailure=4, Closed=5, BufferOverflow=6, InvalidEncoding=7,
PlatformNotSupported=8 and ResourceLimit=9. InvalidPath includes a non-directory
root or invalid filter. Runtime allocation faults remain terminal faults, not recoverable watcher errors.

## Delivery and lifetime

Use a watcher from one invocation/owner queue. OS threads enqueue owned native
records; a repeating timer drains at most 64 notifications every 50 ms on the
invocation completion queue. Await asynchronous work to allow that queue to run.
No callback runs during registration. The queue holds at most 1,024 records,
each at most 64 KiB. Overflow discards the pending batch and reports BufferOverflow;
subsequent notifications can resume. OS loss/rescan signals also report overflow.

Callbacks are synchronous and may stop, restart or dispose the watcher. Each event
uses a registration snapshot: newly registered handlers start with later events;
disposed registrations are skipped and release their captures. Dispose does not
undo a callback already running. Callback panics propagate. There is no guarantee
of one event per operation, ordering across watchers, or complete rename pairs.
A platform may report a rename as deletion/creation. Ambiguous rename information
reports IoFailure so consumers can rescan rather than receive fabricated names.

Filters match basenames, including descendant basenames when recursive. Matching
is case-sensitive on Unix; Windows compares Unicode lowercase scalar mappings.
This is not full .NET OrdinalIgnoreCase/DOS wildcard compatibility. `?` consumes a
Unicode scalar. Invalid UTF-8 filenames report InvalidEncoding; no lossy names are
fabricated. The root is canonicalized when watching starts. Symlinks are not a
security sandbox, and network/pseudo filesystems may not deliver notifications.

Dispose watchers explicitly to release resources promptly. The interpreter's file
resource registry and the native host's invocation resource scope also close
abandoned watchers on teardown, including fault paths. This is native resource
cleanup, not managed finalization. The provider accepts up to 64 active watchers
per process; ResourceLimit reports exhaustion.

## Deployment and platform status

The OS provider is statically linked into the runtime and matching native hosts.
The native project builder selects it when the watcher binding is used; JIT bundle
preparation includes it in the helper. No separate DLL lookup or loader environment
variable is needed. Use a matching runtime/compiler/class-library/host bundle.
The private codegen option `--bind-file-watch` requires managed system compilation,
reference arena and native GC; the native host must provide the invocation scope
already used for timer and task delivery.

Windows and macOS are the release priorities. The provider selects ReadDirectoryChangesW,
FSEvents and inotify through notify 8.2.0. Only macOS execution is locally qualified
in this slice; Windows/Linux qualification remains open. Other targets return
PlatformNotSupported. No silent polling fallback is enabled. See
[design and platform gates](https://github.com/marinasundstrom/neoCLR/blob/main/docs/file-system-watcher.md).

## Configuration example

The [executable sample](https://github.com/marinasundstrom/neoCLR/tree/main/samples/file-watcher)
uses these registrations, with Change/Rename/Rescan scheduling a debounced TryLoad:

```raven
watcher = FileSystemWatcher(directory, "appsettings.json")
_ = watcher.OnChanged(Change)
_ = watcher.OnCreated(Change)
_ = watcher.OnDeleted(Change)
_ = watcher.OnRenamed(Rename)
_ = watcher.OnError(Rescan)
```

Automatic configuration reload and provider-level storage change notifications
remain separate work. Native event declarations are not yet supported by the
Raven native emitter; disposable registrations are the explicit initial API.
NotifyFilter, Filters collections, InternalBufferSize tuning, synchronizing objects
and WaitForChanged are not implemented.
