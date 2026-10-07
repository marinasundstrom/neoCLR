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
String results escaping the invocation. At this slice, String fields and defaults were still rejected. The subsequent
text-bearing-value slice below admits copied String fields and null defaults; String
erasure, arbitrary host inputs and persistent guest state remain rejected.
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

## Text-bearing values and patterns (2026-10-08)

Copied records can now contain String fields, including nested values and closed
String-valued generic carriers. Text pointers still refer only to the image or the
current invocation arena; copying/replacing a field does not free or mutate its text.
Private aggregate slots are eight-byte padded lanes with individual I32/I64 types,
so a pointer between byte/integer fields is never truncated or misaligned. Up to eight
flattened lanes (64 private bytes) remain admitted; these slots are not a public
aggregate ABI. Public scalar entry and context layouts are unchanged.

String defaults use the interpreter's null representation, including inactive union
payloads. Exact String-to-String `isinst`/`castclass` and String `ref.isnull` support
Raven's generated payload pattern checks. General object casts/dispatch are still
rejected. Native WriteLine and failure services check null String arguments before
reading the pointer and preserve the interpreter's RuntimeError code/message/trace.
This deliberately retains existing neoCLR behavior rather than adopting .NET's null
Console string formatting behavior as an unrelated change. No new source/compiler
bridge encoding is introduced.

`text-values.rvn` exercises nested record copies, mutation, control-flow joins,
Some<string>, None and both `let ... else` and `if let`; run
`verify_interactive.py --text-values` for fresh Raven/interpreter/native evidence.
`text-records.neoil` additionally tests invocation-produced text in mixed records,
output copies and interior field borrows, compared with interpreter output at Int32
endpoints. Erased String payloads, arbitrary host String inputs, persistent storage,
reference objects, arrays and virtual dispatch remain outside this profile. The
bounded text-region ownership contract still applies to every copied pointer.

This is the representation layer needed by .NET-like reference-bearing structs and
neoCLR's String-valued Option/Result APIs; it does not claim a general GC-compatible
layout or collection policy. It trades larger padded private storage for a simple
mixed-width implementation. No performance advantage is asserted. ReadLine remains
an ordinary stream-library consumer and is not replaced with a special Console body.

## Bounded reference storage (2026-10-08)

`--reference-arena` explicitly admits nongeneric, nonabstract reference classes with
no inheritance or virtual dispatch, using ABI v4's existing caller-owned storage.
Each allocation contains a private type index and up to eight padded field lanes.
Ordinary constructors, direct instance calls, fields, interior field borrows, output
reference copies, null defaults and same-type reference identity compile natively.
String and reference fields retain pointers; copying an object reference preserves
aliasing rather than copying its payload. Original verifier access/initialization and
borrow checks still run before projection. Unopted reference code remains rejected.

All objects, including cycles, live until the next entry resets the invocation region
or the host releases it. There is no per-object reclamation, external-resource cleanup,
reference counting or native tracing collector. A long-running stream program will
need reclamation beyond this bounded experiment. The Int32 root and absence of static
storage/host object exports prevent guest graphs from escaping this profile. The
region may hold both objects and formatted text; each concurrent invocation still
needs separate storage. Exhaustion preserves output storage and reports status 5.
Null field access reports status 6, NullReference, with the shared message and frames.
Direct `call` retains interpreter semantics: a null receiver faults when its body
dereferences it; `callvirt` null checks/dispatch are not silently substituted.

`reference-cell.neoil` tests shared mutations, output copies, self-cycles, null field
trace parity, exhaustion and repeated region reuse with canaries. The fresh Raven
`reference-cell.rvn` adds class accessors, a shared String field and numeric Console
output. `verify_interactive.py --references` and `references-validation.json` record
standalone output/fault parity against the pinned bundle. This is a storage foundation
for Console's ordinary stream objects, not completion of Write/ReadLine. Interface
views/dispatch, inheritance, arrays, generic reference classes and general managed
collection remain rejected.

The .NET/tracing comparison and cycle requirements remain those in the
[native ownership investigation](../../native-execution-investigation.md#reference-counting-as-an-early-native-experiment-2026-10-07).
An invocation region is useful here because no object can outlive the call; it avoids
adding incomplete retain/release behavior and reclaims cycles together. The cost is
retaining unreachable objects until reset. That bound must not be presented as normal
GC behavior or a production strategy. No speed or memory-efficiency claim is made.

## Nonvirtual callvirt (2026-10-08)

The reference-arena profile now accepts `callvirt` when the exact resolved target is
a nonvirtual class instance method. It performs the required null check at the caller
before entering the body. Direct `call` retains its prior behavior and faults only if
the body dereferences null. Original identities survive load-set projection and type
specialization; there is no name-only intrinsic or substitution of a virtual slot.
True virtual/override targets and all interface contracts remain rejected, including
bodyless interface declarations without an explicit abstract flag.

The same alias/cycle consumer now runs with direct and null-checked calls. Both
return 42 on ordinary input; null input has exactly the interpreter's code, message
and distinct stack location for each opcode. Negative interface and virtual tests
remain in place. This follows the existing neoCLR CIL/CLR call distinction and is
a prerequisite for ordinary class consumers, not implementation of stream interface
dispatch.

## Packed byte arrays (2026-10-08)

The reference-arena profile now supports managed `byte[]` (`arrayref<Byte>` in
neoIL), with zero-initialized allocation, reference copies, length, checked indexed
loads/stores and interior `Byte&` borrows. Array payloads are tightly packed; borrowed
byte stores write exactly one byte even though ordinary private record slots remain
eight-byte padded. Length uses the target UIntPtr lane and explicit Int32 conversion.
The original verifier still governs typed indexing, initialization and borrows.

The private allocation layout has a 16-byte header and a byte payload. It is not
a stable ABI or a general CLR array layout. Arrays share the invocation lifetime
and aliasing contract with objects. Negative lengths produce RuntimeError; null
access produces NullReference; unsigned bounds checks catch negative and oversized
indices as IndexOutOfRange. The experimental maximum is 65,536 payload bytes per
array (ArrayLimitExceeded/status 7), separately from the host's total arena capacity
(NativeMemoryLimitExceeded/status 5). IndexOutOfRange is status 8. Zero-length arrays
are valid and still consume a header. Other element types/value arrays remain rejected.

`byte-array.neoil` compares interpreter/native returns and exact fault diagnostics,
including zero length, zeroed elements, negative lengths/indices, null, upper bounds
and a borrowed byte at the allocation's last byte with surrounding canaries.
`bytes.rvn` exercises shared indexed mutation and ordinary numeric Console output.
`verify_interactive.py --arrays` and `arrays-validation.json` record fresh compilation,
standalone deployment and broken-pipe fault parity. This supplies byte buffers needed
by Console streams; Write/ReadLine still need interface dispatch and more library
capabilities. As with .NET arrays, typed indexing is checked and reference assignment
preserves shared storage; the invocation arena is a deliberately bounded alternative
to the CLR managed heap, retaining unreachable arrays until reset. No performance
or collection equivalence is claimed.
