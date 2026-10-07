# Native Console progression (development, 2026-10-08)

The ordinary Raven `Console.ReadByte` wrapper now compiles to ARM64 together with
its Result/Option construction, generic seed helpers and failure path. Enable
`--compile-system --bind-user-fault --bind-console-read-byte` with an explicit
System seed and library load set. Inspection and emission use identical admission.
Only an exact, verified `neoCLR.Runtime.ConsoleReadByte() -> System.Value`
InternalCall is bound; managed functions bearing that name are rejected. Original
module/definition identities remain in the selection report. Both source-owned and
legacy seed services are supported; the public Console wrapper is never replaced.

`console.h` defines the experimental linked C input contract: 0–255 byte, -1 EOF,
-2 unavailable, -3 read failure. Codegen translates these outcomes to the existing
private erased transport; other results become an invalid Int32 status, reaching
Console's ordinary `System.Fail("invalid native I/O status")` path. The C adapter
uses stdin and distinguishes EOF from `ferror`. Embedding hosts may replace the
adapter, including returning unavailable. There is no global guest value buffer or
managed runtime dependency. Input is synchronous and can block; cancellation,
worker contexts and concurrent host service isolation are not provided by this POC.

The flag implies caller-owned fault ABI v3. Successful reads and expected errors
leave fault details clear; faults preserve managed wrapper/caller frames without a
synthetic input-service frame. The C startup/render helper comes from
[shared fault diagnostics](../aot-fault-details/README.md).

## Evidence

`verify.py` reuses the already compiled and provenance-recorded Raven
[read-byte.rvn](../aot-input/read-byte.rvn) / `ReadByte.pe` fixture. Its `let ... else`
and `if let` paths execute with byte 42, other bytes, NUL, 255 and EOF. Interpreter
and native exit results agree. Closed stdin exercises actual read failure;
replacement C adapters exercise unavailable, read failure and invalid statuses.
The executable runs alone with an empty environment and depends only on libSystem.
`validation.json` records inputs, hashes, commands and results; it is development
evidence against the pinned October 7 bundle, not release qualification.

```sh
SDKROOT="$(xcrun --sdk macosx --show-sdk-path)" \
python3 docs/experiments/aot-console/verify.py \
  --runtime target/debug/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/neoclr-native-poc --output /tmp/aot-console-input
```

Focused Rust tests cover all 256 bytes, distinct transport outcomes, invalid
statuses, opt-in requirements, duplicate options and attempts to bind managed bodies.

## Design and next slices

This implements the existing [Console contract and .NET comparison](../../console-io.md)
without changing its API. Like the established native Hello World experiment,
platform I/O is linked into the executable. Compared with .NET's runtime/library
implementation, this experiment retains neoCLR's explicit byte/EOF/error union
contract and compiles ordinary library code. The benefit is testing the real wrapper
without a managed runtime; the cost is an experimental platform adapter and limited
value/text support. No performance improvement is claimed.

Next: link WriteLine into the value backend so an interactive union consumer can
print prompts/outcomes. The broader Console class still needs dynamic text ownership,
numeric formatting, reference objects, arrays and stream dispatch for Write, ReadLine,
In/Out/Error and standard streams. The current literal-only String profile does not
provide these capabilities. These are implementation gaps, not permanent API limits.

## Module-local service resolution

A follow-up preserves the interpreter's local InternalCall binding rule when a seed
and a source-owned library declare the same service. AOT now pins a name-only call
to the exact local service definition before scope flattening. Original load-set
verification still runs first; ordinary managed calls do not gain local preference.
A duplicate-service test verifies the selected library identity, and all 18 explicit
load-set tests pass. The combined Raven input/WriteLine probe drove this correction;
its output service still requires the next explicit binding. No public API changes.

## Interactive output (2026-10-08)

`--bind-console-write-line` binds the exact verified
`neoCLR.Runtime.WriteLine(String) -> Void/noresult` InternalCall and implies ABI v3. The
ordinary `WriteLine(string)`, `WriteLine(bool)` and `WriteLine()` library wrappers
compile unchanged. The linked UTF-8 adapter from the scalar experiment accepts a
byte pointer and explicit length, preserving embedded NUL; it writes LF and flushes.
Nonzero adapter results become RuntimeError with the standardized message and
managed callers, excluding the native service from the trace. Partial output cannot
be rolled back. This slice adds no dynamic String allocation or public guest API.

The standalone host ignores SIGPIPE so broken stdout pipes become ordinary I/O
faults rather than signal termination. Embedding hosts own their process signal
policy; the adapter itself does not change it. This matches the Rust CLI's broken-pipe
behavior. Closed-fd stdout was not a valid interpreter parity oracle because Rust
may ignore that error; use the broken-pipe test.

`interactive.rvn` reads the real nested Result/Option, prints prompts and branch
outcomes, and exercises Boolean and empty-line overloads. `verify_interactive.py`
compiles fresh Raven metadata and compares interpreter/native output, exit codes and
broken-pipe diagnostics exactly. `interactive-validation.json` records the pinned
compiler/bundle and standalone evidence. The pinned compiler intermittently reports
missing Console/union members on unchanged source; a fresh retry compiled the sample.
This is a recorded compiler reproducibility limitation, not an AOT fix.

Run the verifier with the same arguments as `verify.py`, additionally passing
`--compiler /path/to/rvnc.dll`, and choose a fresh output directory. Focused backend
tests also exercise Unicode/NUL bytes, output failure, unchanged result storage and
reuse of the fault context after success. Next: invocation-owned dynamic UTF-8 text
for numeric formatting, then the stream/reference machinery behind Write/ReadLine.

## Numeric output and text lifetime (2026-10-08)

`--bind-int32-to-string` admits the exact verified
`neoCLR.Runtime.Int32ToString(Int32) -> String` service. Ordinary Console integer and
byte overloads now compile their existing conversion/output wrappers. No Console
method is substituted. `numbers.rvn` prints an input byte and both signed Int32
endpoints; `verify_interactive.py --numeric` builds fresh Raven metadata and compares
native/interpreter output and broken-pipe diagnostics. See `numbers-validation.json`.

This producer requires **experimental ABI v4**, declared in `text-arena.h`. The
selection report names `caller-owned-text-arena-v4`, and the object exports
`neoclr_entry_v4` instead of v3. The context contains the unchanged fault-record prefix
and a host-owned text arena (buffer, capacity, cursor). The host supplies aligned
storage, separate for each concurrent invocation, nonoverlapping with context/result.
Each entry clears fault details and resets the arena cursor. Allocation never escapes
the buffer, and exhaustion returns explicit status 5, NativeMemoryLimitExceeded,
with the shared standard message and managed caller frames. The experimental native
text budget is distinct from interpreter heap limits; identical allocation accounting
is not claimed. Malformed arena state maps to RuntimeError.

Strings use immutable length-prefixed UTF-8, as image literals do. Allocation aligns
headers, preserves old strings across later conversions, and publishes a pointer only
on success. Pointer copies across locals, calls and output slots remain valid until
this invocation's storage is reused/released. **Render dynamic fault messages before
the next entry call or releasing the buffer.** The Int32 root signature prevents guest
String results escaping the invocation. Reference fields, String erasure, arbitrary
host String inputs, persistent guest state and default/null strings remain rejected.
The POC does not use reference counting, finalizers, a tracing collector, TLS or a
process-global allocation list. The host example chooses a 64 KiB stack buffer;
that size is sample policy, not a platform default.

Compared with the [.NET/tracing and ownership alternatives](../../native-execution-investigation.md#reference-counting-as-an-early-native-experiment-2026-10-07),
this is deliberately narrower: a bounded invocation region makes numeric Console
output executable without a native root scanner or retain/release lowering. Its cost
is retaining all produced text until the invocation ends or the next entry resets it;
long-running producers can exhaust the buffer even if old strings are no longer used.
This is not a claim of equivalent managed memory behavior or a production strategy.
A general native heap still needs the recorded cycle, roots, reference-bearing values
and cleanup work. No performance improvement is asserted.

Tests preserve two simultaneous strings across a nested call, format Int32 endpoints,
retain a dynamic UserFault message, exhaust both first and later allocations, check
unchanged result/cursor/output and surrounding canaries, reject malformed arena state,
and reuse a context after failure. Versions 2/3 remain unchanged when no dynamic text
producer is selected. The next Console dependencies are wider numeric/text primitives
and the ordinary stream/reference path used by Write and ReadLine.
