# Minimal console I/O

The initial console boundary provides raw byte input and line output. It is a small
embedding contract, not a guest Stream API. Parsing and control flow in the demonstration
are ordinary platform IL; line decoding and a general text reader are deferred.

## Platform methods

`System.Console.ReadByte() -> System.Result<System.Option<Byte>,System.ConsoleReadError>` provides raw byte input.
Its IL body calls the explicitly declared `neoCLR.Runtime.ConsoleReadByte` InternalCall.
The host returns an explicitly erased Byte, Void (EOF), or Int32 status. Platform IL
constructs the ordinary nested cases with exact Byte storage:

- Ok(Some(byte)): one byte, including zero or values above 127.
- Ok(None): end of input, separate from an empty line or an input error.
- Error(ConsoleReadError.Unavailable): no host console was supplied.
- Error(ConsoleReadError.ReadFailed): the host reported an input failure.

Input has no implicit UTF-8 decoding, newline conversion, or -1 EOF sentinel. The sample stores a raw Byte case payload in a Byte local before loading it for
arithmetic. The local load converts it to the Int32 evaluation-stack category.
Both console samples use ordinary case constructors, predicates and checked accessors.
ReadByte contains no union-specific IL. The former ReadByteTyped name is removed.

Existing WriteLine(String) and WriteLine(Int32) retain their Void return type. When a
host console is supplied, output is delivered immediately, before another guest
instruction runs. Host output failure is currently a terminal Fault with a logical
stack trace; the existing signature cannot carry a recoverable Error. An explicit
Result-returning output contract is a future decision. Partial output is possible on
failure and already delivered output survives later Faults or cancellation.

## Hosting and isolation

ExecutionOptions.console accepts an optional Arc<dyn Console>. The Rust Console trait
has two synchronous operations: read_byte and write_line. This boundary introduces no
guest interface dispatch, stream hierarchy, ownership model, or async execution.
Implementers return I/O errors rather than panic, synchronize their own state, and make
successful line writes visible before returning. Host callback panics are not converted
to guest Errors or Faults.

Without a console, existing embedding behavior is preserved: WriteLine collects lines
in Execution.output and input returns ConsoleUnavailable without touching process stdin.
With a console, output goes only to that host and Execution.output stays empty. The
same option applies to entry execution and resolved static/instance invocation.

Cloning options shares the host console and its input position; it does not clone or
reset input. Guest frames and allocations remain fresh for each invocation. Hosts can
supply a new console for independent sessions, and tests can use an in-memory console
without changing process-wide input/output. Concurrent sessions sharing a console may
interleave calls; no whole-session transaction or exclusivity is implied.

The CLI explicitly selects StdioConsole. It reads process stdin as bytes, writes UTF-8
text with LF endings to stdout, and flushes each line before returning. Program output
is no longer delayed until successful completion, and is not printed a second time from
the Execution result. The final `=> value` remains a CLI diagnostic after success.

```rust
let options = neoclr::ExecutionOptions {
    console: Some(std::sync::Arc::new(neoclr::StdioConsole)),
    ..neoclr::ExecutionOptions::default()
};
let execution = program.run(options)?;
```

This is an experimental Rust API addition: explicit ExecutionOptions struct literals
must include console or use a default update. Existing callers passing Limits directly
retain captured output and unavailable input.

Calls are blocking. Cooperative cancellation and instruction limits cannot interrupt
an in-progress host read/write. StdioConsole retries interrupted reads; custom hosts
define their own retry behavior. Runtime-service planning reports ConsoleInput and
ConsoleOutput independently; discovery performs no console I/O and is not authorization.

## Demonstration

From the repository root, build and verify, then run interactively:

```sh
cargo build --locked
cargo run --locked -- verify examples/console_input.neoil
cargo run --locked -- run examples/console_input.neoil
```

Enter `21` followed by Enter. The prompt appears before reading, then the program prints
`42` and returns Void. To test redirected input from a POSIX shell:

```sh
printf '21\n' | ./target/debug/neoclr run examples/console_input.neoil
```

Expected stdout:

```text
Enter an unsigned number (up to nine digits):
42
=> Void
```

ReadNumber in the sample accepts up to nine ASCII digits and doubles the value in Main.
LF terminates input; CR bytes are ignored for CRLF compatibility. EOF after digits
finishes a number, while EOF without digits returns None. An empty line, a nondigit,
or a tenth digit produces an application Error. This is a one-shot demonstration:
on an invalid byte it returns immediately without draining the remaining input.
It is not a general ReadLine or Int32.Parse implementation. Nine digits keep the doubled
result within Int32 range without needing an additional arithmetic error policy.

The sample can be assembled to JSON and run with the same commands as the README's
HelloWorld workflow. Tests cover redirected CLI prompt ordering, byte values, EOF,
input/output failures, host isolation/defaults, shared input position, verification,
and service analysis.

Next text APIs should follow the byte/string primitives needed to implement their logic
in platform code. Stream, socket, buffering-framework, and asynchronous APIs remain
outside this slice.


ConsoleReadError is an ordinary non-generic carrier with nested Unavailable/ReadFailed
cases, constructor overloads, predicate properties and checked case accessors. ToString
preserves ConsoleUnavailable/ConsoleReadFailed for display. Those strings do not determine
case identity. Its native helper uses erased Int32 status 1 for Unavailable and 2 for
ReadFailed; Byte remains reserved for successful data, including 0 and 255. Unknown
statuses or payload types Fault. The interactive sample explicitly converts these errors
into an application-level string message when combining input and validation failures.
Reassemble applications and System together for the new error parameter and native payload.

## Development Console streams — 2026-09-23

The [Console guide](../api-docs/console.md) documents the new text/byte surface.
Console stays a static class. Rust hosts can implement `write_bytes(error, bytes)`
and `flush(error)`; their defaults return Unsupported so existing hosts still
compile. StdioConsole implements both channels. These methods are synchronous.
Without a live host, Execution.stdout and Execution.stderr capture exact bytes;
Execution.output remains the legacy line record rather than a complete rendering
of stream writes. Legacy WriteLine also contributes text plus LF to stdout capture.
With a live host, output goes to the host and capture vectors stay empty.
Closing a guest wrapper never closes the process channel. Worker/debugger hosts
need byte-hook implementations before they support the new output streams.

### Object and scalar WriteLine — 2026-09-25

WriteLine(object?) dispatches virtual ToString; null writes an empty line. Direct
Boolean, Char and integral overloads avoid boxing. Signed/unsigned 64-bit decimal
formatters are private runtime services, also used after native-sized conversion;
small integer overloads widen to Int32. Those services are classified as string
operations. No compiler setting or Runtime Contract configuration changes.

Compared with [.NET Console.WriteLine](https://learn.microsoft.com/en-us/dotnet/api/system.console.writeline?view=net-10.0),
object/null handling follows the same basic contract. neoCLR offers exact overloads
for narrow/native integers too, uses invariant decimal output, and Char is grapheme
text rather than a UTF-16 code unit. There is no formatting-provider contract.
The cost avoided is temporary boxed scalar allocation; output still creates text.
Floating-point formatting remains deferred; its current Object fallback does not
promise numeric output. This does not broaden ToString semantics for other types.


## Development terminal host transport — 2026-10-20

The Rust embedding boundary now provides `Console::capabilities()` and
`Console::open_terminal(TerminalOptions)`. This is a host foundation, not a Raven
key-input API or an asynchronous console implementation. Existing custom hosts
retain unsupported defaults and can supply a synthetic `TerminalInput` implementation.
The class-library API remains Raven-owned; OS mode changes belong at the host boundary.

`ConsoleCapabilities` reports stdin/stdout/stderr terminal detection separately;
`terminal_input` reports whether an adapter can attempt acquisition. Acquisition
can still fail. No field asserts UTF-8, ANSI, a keyboard protocol or font coverage.
`StdioConsole` currently implements terminal acquisition on POSIX systems; Windows
reports no terminal-input adapter until native Unicode integration is implemented.
Redirected stdin is rejected, without opening a different controlling terminal.

An acquired `TerminalInput` owns input exclusively across all `StdioConsole`
instances. Legacy byte reads and a second session fail with `WouldBlock` while
ownership is held. A blocking legacy read also holds this lease. Other libraries
and external code reading process stdin are outside this coordination contract.
On POSIX, legacy reads now use the same unbuffered descriptor, avoiding hidden
standard-library read-ahead when switching between line/byte and terminal input.

`try_read(&mut [u8])` fills caller-owned storage, returning `Data(count)`, `Pending`
or `End`. Empty destinations fail with `InvalidInput`; reads after successful close
fail with `NotConnected`. No key decoding, Unicode normalization or String/Promise
construction occurs at this boundary. The caller must supply an encoding-aware
layer: POSIX terminal bytes are not guaranteed to be UTF-8. VMIN/VTIME zero plus a
zero-time readiness check avoid waiting for input; no file-description flags change.

`read(&mut [u8], Option<Duration>)` is the synchronous wait capability. `None`
waits until bytes or end; a duration bounds the wait and returns `Pending` on
timeout; zero delegates to `try_read`. Custom hosts default to `Unsupported` for
nonzero waits. POSIX uses `poll`, rounding fractional milliseconds up and retaining
one deadline across interrupted waits. Long waits are split at the poll integer
limit. Empty buffers and closed sessions retain the same errors. Successful reads
use the supplied buffer directly, with no intermediate array, String, Task or Promise.
This is a Rust host API only; no guest ABI or native intrinsic is added. The existing
host transport is the appropriate layer for OS readiness, while key interpretation
remains intended for the Raven library. It is not an async scheduler or cancellation
mechanism. Compared with .NET's blocking ReadKey, this lower byte boundary deliberately
separates waiting from decoding and echo, enabling caller-owned storage.

Sessions disable canonical input, echo, extended input processing, byte stripping,
newline translation and software output-flow control. Output processing is retained.
By default ISIG remains enabled; `capture_interrupts: true` disables POSIX signal
characters, including Ctrl+C, Ctrl+\ and Ctrl+Z, so they arrive as input bytes.
This option is broader than .NET's TreatControlCAsInput; the explicit name and scoped
ownership expose the actual host behavior instead of promising unsupported fidelity.

`close()` restores the saved terminal settings without flushing queued input and
releases ownership. BSD may set the kernel-managed PENDIN queue-state flag when
canonical mode resumes; the restoration tests exclude that flag. It is idempotent after success; restoration errors retain ownership
for an explicit retry. Drop attempts restoration and releases the lease even if the
host refuses restoration (for example after hangup). A duplicated close-on-exec
file descriptor keeps restoration attached to the acquired device if stdin is replaced.
There is no process signal/exit handler yet: abort, default signal termination and
forced process exit do not run Rust Drop. Embedders must arrange orderly session
cleanup; default Ctrl+C interruption is not yet an orderly cancellation facility.

Focused evidence: `cargo test --lib console::` exercises a real pseudo-terminal,
nonblocking idle reads, preserved UTF-8/control/escape bytes, capture/default ISIG,
close/drop restoration, closed/empty-buffer errors, redirected input, input ownership
and hangup. `cargo run --example terminal_input` is an interactive byte diagnostic;
it now waits directly in the host rather than sleeping between polls. It reports
allocator calls around each read, excluding acquisition and diagnostic output.
The [reproducible PTY check](experiments/terminal-input/README.md) measured zero
Rust allocator calls in five fragmented reads and verified terminal restoration;
this is not a guest key-decoder or throughput claim.

Next integration requirements are invocation-owned cleanup, scheduler readiness and
cancellation, the matching JIT/AOT provider, Windows Unicode input, and Raven key,
modifier and text events. Do not expose a Task that merely wraps a blocking read or
advertise guest support before those paths are qualified. See the
[design comparison](design/console-input-review-20261020.md).


## Development shared text reads — 2026-10-20

Console.In now caches one invocation-owned StreamReader. PeekChar, ReadChar, counted
Read and existing line reads share its cursor and lookahead; this supersedes fresh
input-wrapper behavior. Out/Error and raw stream factories remain fresh wrappers.
Close closes the shared text reader, not process stdin. Raw reads bypass text
lookahead and must not be mixed with buffered text reads. The
[three-engine consumer](experiments/console-text-reading/README.md) validates mixed
reads, byte bounds, fragmented graphemes and pending-source suspension. The
[API guide](../api-docs/console.md#grapheme-peek-and-counted-reads-development) documents
the complete text contract. Existing console host reads still block; the POSIX
terminal transport is not yet an async guest provider.

## Internal key decoding foundation — 2026-10-20

The [Raven decoder kernel](experiments/console-key-decoder/README.md) now consumes
borrowed UTF-8 bytes and returns value results with input-relative text windows.
It covers a bounded legacy navigation/function-key profile, modifiers, incomplete
input and strict UTF-8 errors. This is internal groundwork; Console still has no
public key-read API. Invocation cleanup, shared cursor ownership and matching
native providers remain required. The 384-call native diagnostic adds no managed
allocations over baseline; CPU/throughput and interpreter allocations are unmeasured.

## Invocation terminal ownership foundation — 2026-10-20

The Rust invocation now owns a private terminal service. Its root shutdown path
first requests terminal reads to stop, then joins managed/native jobs, then closes
the terminal session. The root scope also performs cleanup on unwinding, even when
other references keep the invocation alive. Explicit restoration errors are reported;
failed close retains the session for retry, with a final best-effort close at drop.
This is **internal integration**: no guest binding currently opens this service,
and the standalone JIT/AOT host still requires a matching provider.

The service admits one session, with process-unique, non-reused private handles.
Foreign/stale handles reject; overlapping reads or close during an active read
return WouldBlock. Reads take the session out of the mutex while waiting and return
it through a scope guard, including on a host read panic. Shutdown waits for that
guard without holding the lock. All these operations must run outside managed graph
access. Successful reads fill caller-owned storage directly; no buffer snapshots
or per-read managed objects are introduced by this service.

For cancellation, the service bounds host waits to 20 ms and checks the invocation
cancellation token between waits. It preserves the caller's overall timeout rather
than restarting it per fragment. This is synchronous blocking OS readiness with
periodic cancellation checks, not scheduler-backed async input. It can wake roughly
50 times per second while idle. A future native wake handle could remove that cost;
there is no throughput or end-to-end allocation claim for this checkpoint. Hosts
must honor timeout contracts; arbitrary blocking foreign code cannot be interrupted.
Bytes already transferred remain a successful read if cancellation races completion.

Validation: `cargo test --release -p neoclr --lib invocation` covers root-scope
cleanup, retained invocations, unwind, cancellation while idle, shutdown during a
read, overlap, stale handles, retryable close failures, finite deadlines, host read
panic cleanup and real POSIX pseudo-terminal mode restoration, alongside existing
invocation/worker regressions. Existing decoder and transport allocation evidence
is unchanged and does not establish a complete Console.ReadKey path.

## Native terminal transport checkpoint — 2026-10-20

The C console provider now has a private POSIX terminal adapter matching the Rust
transport's caller-buffer behavior. The existing native byte reader uses unbuffered
`read` on POSIX, removing FILE read-ahead that could strand queued input when switching
to terminal mode. A provider-local atomic lease excludes legacy byte reads while a
terminal session is active and rejects a second session. Overlapping legacy reads
also reject with the existing ReadFailed status instead of waiting on stdio locking,
matching the Rust provider’s exclusive-input policy. This does not coordinate
unrelated C libraries, direct descriptor reads, or independently linked Rust/C
providers operating concurrently; guest invocation binding must select one owner.

The caller zero-initializes and owns session storage, serializes access, and must not
copy an active session. Open duplicates stdin with close-on-exec and saves termios;
close restores that device even if stdin has been replaced. Failed restoration keeps
ownership for retry. Final dispose attempts restoration and releases the descriptor
and lease even if restoration fails, corresponding to the Rust best-effort Drop path.
There is no automatic process-signal cleanup or native invocation registration yet.

The private header in `docs/experiments/aot-console/console.h` describes open/read/
close/dispose statuses. Read borrows a writable byte buffer of 1–65536 bytes; zero
wait is nonblocking, -1 waits indefinitely, and positive milliseconds bound the total
wait across interruptions. Bytes, pending and end are distinct. Poll rounds fractional
milliseconds up. No key inference, UTF-8 decoding, Task, Promise or intermediate
buffer is introduced. Ctrl+C/quit/suspend capture disables ISIG explicitly. Windows
returns unsupported for terminal acquisition rather than treating redirected bytes
as keys.

[Six native executable checks](experiments/terminal-input/README.md#native-posix-transport-checkpoint)
qualify transport behavior and restoration on macOS arm64. The existing output
adapter regression also passes. Decoder-to-provider guest bindings and native
invocation cleanup remain pending; this is not public Console.ReadKey support.

## Native root-thread invocation scopes — 2026-10-20

Native console, JIT, HTTP and retained-session entry hosts now register a private
terminal scope around guest execution and dispose it after normal or fault return.
Restoration occurs before fault rendering and heap release. The scope is separate
from GC resource callbacks, whose contract prohibits I/O. It uses caller-owned
storage and thread-local scope lookup, with no managed roots or per-read result
allocation. This does not change the native context or guest metadata layout.

`neoclr_terminal_scope_enter_v1`/`leave_v1` require zero-initialized stable storage,
non-null context identity, and nested LIFO teardown. Open returns a positive private
handle; read/close require the same context and handle. Handles do not reuse during
the loaded provider's lifetime. Scope leave performs final disposal and removes
registration even when restoration reports an error. Explicit close remains
retryable; normal host failure results report cleanup errors. Abort, forced exit
and external nonlocal jumps are not orderly guest fault returns and do not acquire
an automatic restoration guarantee.

This initial native scope belongs to its entry thread. Unattached worker threads
reject rather than borrowing another invocation's input; native worker attachment
and cancellation remain gaps. Rust's invocation service has broader shared-worker
ownership, so end-to-end parity is not yet established. There are still no guest
terminal acquisition or key-read bindings. The scope is a private native ownership
contract, not a public hosting API or a permanent thread restriction on Console.

[Eleven native checks and the existing Raven text consumer](experiments/terminal-input/README.md#native-invocation-scope-checkpoint)
qualify macOS host cleanup and regression behavior. Windows terminal acquisition
remains unsupported, and the changed Windows host has not been tested in this slice.


## Public Raven key input — 2026-10-20

Console.OpenKeyInput now acquires the invocation-owned terminal service and returns
ConsoleKeyReader. Its synchronous ReadKey uses the bounded Raven decoder, returns
value results and avoids per-key text/array allocation. The private scalar bridge
is implemented by interpreter, JIT and AOT. See the [public key API](../api-docs/console.md#terminal-keys-development)
and [PTY/allocation qualification](experiments/console-read-key/README.md). Earlier
checkpoint descriptions above record the staged implementation; public guest
binding is now complete for the documented POSIX entry-thread scope. Text-to-key
handoff, native worker attachment and Windows terminal support remain open.
