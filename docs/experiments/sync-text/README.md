# Separate synchronous text wrappers — 2026-10-20

Development contract: StreamReader and StreamWriter remain async-default;
SyncStreamReader and SyncStreamWriter are explicit synchronous alternatives.
Each consumes the corresponding byte capability. Factories return Result without
performing I/O. Valid providers currently initialize successfully; conversion and
I/O errors occur during operations. Constructors are private. There is no
SupportedModes flag and no automatic sync-over-async or async-over-sync adapter.

The author specifically requested synchronous escape hatches for inherently
synchronous providers. Console now exposes SyncTextReader/SyncTextWriter and
SyncInputStream/SyncOutputStream, and its text reads and Write return directly.
No Task/Promise is created merely to wrap its blocking host transport. Terminal
readiness, cancellation and key-event integration remain separate work.

Both families use the same encoding implementations and shared line/error and
encoder-progress helpers. Each wrapper owns its buffer and codec state; it does
not allocate a forwarding strategy or facade. Execution loops are separate so
sync operations do not create Tasks. Writer buffers are reused. Result, String,
codec and wrapper allocations remain; no zero-allocation or throughput claim.

Use one buffering wrapper per stream direction. Async operations reject overlap
with Busy. Cleanup releases the operation guard on cancellation; a writer whose
transfer was abandoned rejects further encoding because partially drained output
cannot safely be replayed. Host faults terminate execution; no exception-unwinding
guarantee is introduced.

The sync fixture checks fragmented graphemes, peek reuse, counts, line reads,
EOF, limit retry, strict ASCII errors, UTF-8 byte counts, flush versus finalization,
repeat Finish, closed state and ownership. The console-text-reading fixture checks
real pending async reads alongside direct Console calls. Runtime tests check
selected-window snapshots, uninitialized-byte rejection and native view admission.
Interpreter, JIT and AOT pass in [results.json](results.json). The extended
[async/cancellation consumer](../console-text-reading/results.json) also passes all
three engines. Compiler 100f98a2f, libraries target/sync-text-libraries-v8, bundle
target/sync-text-bundle-v1/bundle, apps target/sync-text-app-v1 and
target/sync-text-async-app-v3. Both focused Rust tests and four native API snapshot
tests pass; the website builds 4674 pages with 124 legacy pages retained.

Compared with .NET's unified StreamReader/StreamWriter sync and async surface,
this deliberately makes provider requirements visible in separate types. The
benefit is no unsupported-mode test or implicit blocking adapter; the cost is
four classes and separate execution loops. Grapheme text and UTF-8 result bounds
remain neoCLR-specific. Sources:
[Stream](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream?view=net-10.0),
[StreamReader implementation](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/IO/StreamReader.cs).
