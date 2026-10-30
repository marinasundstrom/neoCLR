# Physical filesystem notifications

Author-directed development slice, 2026-10-29. The author requested a standalone
FileSystemWatcher that could support configuration reload. After considering
Microsoft.Extensions.FileProviders as a reference, the author selected the
standalone watcher first and clarified that System.Storage is its own WinRT-like
provider/item model, not a copy of FileProviders. The watcher belongs in System.IO;
provider-level change notification remains open. Windows and macOS are the current
release priorities; broader target coverage follows explicit qualification.

## Public contract and .NET comparison

[System.IO.FileSystemWatcher](https://learn.microsoft.com/en-us/dotnet/api/system.io.filesystemwatcher?view=net-10.0)
supplies the naming baseline: path/filter, recursive watching, enable/disable,
created/changed/deleted/renamed notifications and errors. Events are hints to
re-read current state, not a journal or an atomic-write guarantee. The
[member reference](../api-docs/file-system-watcher.md) documents exact current
behavior and gaps; no complete .NET compatibility is claimed.

The author subsequently directed a single function callback receiving a union,
with invariants represented by the payload types (2026-10-30). Subscribe accepts
`(FileSystemNotification) -> Void` and returns a Disposable. Created/Changed/Deleted
carry FileSystemChange; Renamed carries FileSystemRename with both paths; a
RescanRequired case carries the reason. The kind cannot disagree with the payload.
Paths retain the canonical root and validated relative name, deriving FullPath at
the boundary. Rename paths must share a root. No independently supplied full-path
property or redundant ChangeType is retained.

This deliberately differs from .NET's separate events and EventArgs hierarchy:
it enables exhaustive Raven matching and structurally coherent payloads, at the
cost of source compatibility with the initial development API and .NET event idioms.
The author made direct function callbacks the general notification convention,
accepted through constructors or registration methods as lifetime needs dictate.
Library event support will come later; this is an intentional API direction, not
merely a workaround for the current native event-emission gap.

Settings can change
only while stopped; this avoids implicit OS resubscription and its observation gap,
at the cost of requiring an explicit Stop/change/Start sequence. NotifyFilter,
Filters, buffer tuning, WaitForChanged and synchronization objects remain gaps.

The API uses physical path strings, typed Result setup errors and UTF-8 text.
It leaves System.Storage's logical paths, provider lookup and File/Directory
interfaces intact. The linked .NET FileProviders API is useful background for
future provider-level watching, not an accepted replacement storage model:
[IFileProvider](https://learn.microsoft.com/en-us/dotnet/api/microsoft.extensions.fileproviders.ifileprovider?view=net-11.0-pp),
[IChangeToken](https://learn.microsoft.com/en-us/dotnet/api/microsoft.extensions.primitives.ichangetoken?view=net-11.0-pp).

## Provider and runtime boundary

The shared Rust provider pins [notify 8.2.0](https://docs.rs/notify/8.2.0/notify/):
FSEvents on macOS, ReadDirectoryChangesW on Windows, inotify on Linux. It translates
owned native event data into bounded packets. Filtering happens before enqueueing;
paired renames match either basename. Unknown rename data and backend failures
report errors so callers can rescan, preserving independently known changes in the
same batch. Root metadata changes are ignored; removal of the watched root reports
an error. Overflow clears the queued batch and preserves
an explicit loss signal. No filesystem polling fallback is silently substituted.

We considered a separately deployed FFI library: no filesystem operation needs
privileged guest access, and its C ABI is small. The prototype exposed compiled
interop gaps in string/buffer marshalling and resource lifetime. Instead, the same
provider is statically shared behind a private InternalCall adapter. The benefit is
consistent interpreter/native transport and invocation-owned cleanup; the cost is
a new runtime binding and host packaging dependency. Revisit direct FFI when both
compiled marshalling and invocation-owned foreign-resource leases are available.
No general interop behavior or Raven compiler semantics change in this slice.

OS callbacks retain only native paths, queue data and synchronization state. They
never enter managed code. A managed 50 ms Timer drains at most 64 records per tick
on the invocation completion queue. The interpreter owns subscriptions in its
file-resource registry. The native adapter uses the existing invocation scope
already shared by sockets, timers and scheduling; scope exit closes remaining
subscriptions on success/fault. This temporary host container name does not place
filesystem semantics in the public networking API. Handle ownership is checked;
IDs do not repeat or expose OS descriptors. Explicit disposal remains recommended.

The private FileWatch transport uses operation 0=open, 1=read, 2=close. Success
carries Int64 handle, UTF-8 String packet or Int32 zero; Byte carries an error code.
Native packets are kind byte plus four NUL-delimited UTF-8 paths (new absolute,
new relative, old absolute, old relative). Only the runtime adapter reads this
encoding; it is not a public or persistent data format. Error codes, bounds and
public argument types are documented in the reference. Native host allocation
publishes a rooted string only after native dequeue; managed callbacks never run
inside the adapter.

## Portability gates

Windows and macOS need actual platform runs, not just successful cross-compilation.
The common test driver exercises the source API through native execution and the
interpreter, with real create/atomic-replace/write operations, recursive filtering,
stop/restart and disposal. Provider tests cover wildcard/Unicode matching, rename
pairs, overflow/loss recovery and native handle release. Native invocation teardown
must also release abandoned handles. Keep backend selection separate from evidence.
Windows is unqualified until its host build/run passes; Linux and other targets
are follow-ups. Unsupported targets report PlatformNotSupported. Filesystem and
OS limits, permissions, mount behavior and unowned files can still affect delivery.

[The sample](../samples/file-watcher/README.md) demonstrates debounced configuration
reload while preserving prior values after malformed input. It does not change
the storage-provider contract. [Neo.Configuration](configuration.md) now offers
opt-in AddJsonFile reload using this watcher; provider change tokens remain separate work.

## Validation receipt

See [the focused macOS evidence](validation/file-system-watcher-20261029/README.md)
for native/interpreter lifecycle, ownership, provider and configuration-sample results.
Windows execution remains a release qualification gate.

The [2026-10-30 union callback receipt](validation/file-system-watcher-union-20261030/README.md)
records the replacement contract, consumer migration and constructor/type rejection checks.
