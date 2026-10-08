# Native Console progression (development, 2026-10-08)

Latest lifetime checkpoint: [basic native GC](native-gc.md) now runs the repeated Raven
routing workload within a fixed 64 KiB heap. The sections below record the progression;
diagnostic-only collection restrictions do not describe the explicit `--native-gc` mode.

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

## Character output (2026-10-08)

`--bind-character-text` (requiring `--compile-system`) binds only the exact supplied
InternalCall contracts `CharFromString(String) -> Char` and `CharText(Char) -> String`.
The ordinary Raven `Char.FromString` and `Console.WriteLine(char)` wrappers compile
as managed bodies. Static primitive-owned wrappers are verified in their original
load-set scopes, reported as `staticPrimitiveOwners`, then lowered to private free
functions because they have no receiver. This avoids rebinding their owner to the
backend verifier's separate bundled Char declaration; source code and metadata are
unchanged. There is no new compiler bridge encoding.

Characters retain immutable length-prefixed UTF-8 pointers through locals, arguments,
results and output borrows. `initobj Char` produces the image-owned NUL grapheme.
String-to-character conversion validates exactly one extended grapheme; empty,
multiple-grapheme and null text report interpreter-compatible RuntimeError with the
managed caller's location. Conversion back to String retains the same immutable bytes,
including embedded NUL and combining sequences, without normalization or allocation.
Pointers keep their original image/invocation lifetime. Character fields, erasure,
arrays, comparisons and generic payloads are not admitted in this slice.

The allocation-free `tools/aot-native-text` static library uses **unicode-segmentation
1.12.0**, exactly the interpreter's pinned implementation. Its private C symbol
`int32_t neoclr_is_single_grapheme_v1(const uint8_t *bytes, size_t length)` returns 1
only for one valid UTF-8 extended grapheme, otherwise 0; readable immutable storage
of that length is a caller precondition. It retains no pointer and requires no shared
Rust or managed framework. Build with `cargo build --locked --release --manifest-path
tools/aot-native-text/Cargo.toml`, then link the resulting archive alongside the AOT
object and Console adapter. Internal Rust panics abort; invalid guest text uses the
ordinary checked result, not panic. No change to the exported v3/v4 hosting layout.

`characters.neoil` compares default NUL, copied/output characters, decomposed accents,
emoji families, flags, CRLF, invalid and dynamically formatted inputs against the
interpreter's exact fault text. `characters.rvn`, `verify_interactive.py --characters`
and `characters-validation.json` add fresh Raven Console output, standalone deployment
and broken-pipe parity. The native archive is included in provenance hashes.

The [existing text design comparison](../../design/text-abstraction.md)
explains the distinction from .NET's UTF-16 Char and StringInfo text elements. Reusing
the platform's segmentation gives consistent character boundaries across execution
modes at the cost of linked Unicode tables and variable-length character storage.
This is semantic parity, not a performance claim or a new text model. Stream-backed
Write/ReadLine and broader numeric formatting remain subsequent Console work.

## Small-integer output (2026-10-08)

SByte, Int16 and UInt16 now pass through locals, fields, parameters, results and
output borrows. Unchecked `conv.i1`/`conv.i2` truncate and sign-extend; `conv.u2`
truncates and zero-extends. Like Byte, these storage types use the Int32 evaluation
stack category. Typed loads read one/two bytes with the correct extension, and typed
stores write one/two bytes. Padded private record/call slots retain their eight-byte
spacing; this is not a new public aggregate layout. Checked narrowing, other array
element types and wider integer formatting remain unsupported.

This follows the existing neoCLR CIL/CLR distinction between integer storage widths
and evaluation categories, reusing the scalar backend's arithmetic contract. The
benefit is consistent values at every storage boundary, including signed Console
output; the cost is explicit narrowing/extension in the private backend. There is
no claimed performance advantage or .NET API change. Primitive method/type
specialization also admits these small integer shapes within the existing bounds.

The storage-boundary test now compares 48 native/interpreter inputs across four
width/sign categories, including overflow truncation and sign boundaries.
`small-integers.rvn` uses typed forwarding methods and Console output for the signed
8/16-bit endpoints and unsigned 16-bit maximum. `verify_interactive.py --small-integers`
and `small-integers-validation.json` record fresh compilation, exact output/fault
parity and standalone deployment. The existing Int32 text binding provides formatting;
no new Console native service or public managed API is added.

## Wide and native-width integer output (2026-10-08)

`--bind-integer-text` (requiring `--compile-system`) admits exact InternalCall
`Int64ToString(Int64) -> String`, `UInt64ToString(UInt64) -> String`,
`IntPtrToInt64(IntPtr) -> Int64` and `UIntPtrToUInt64(UIntPtr) -> UInt64` contracts.
Ordinary Console/runtime-service wrappers still compile as managed bodies. Formatting
calls the linked `neoclr_int64_to_string_v1`/`neoclr_uint64_to_string_v1` functions,
with `(int64_t|uint64_t value, neoclr_aot_text_arena*, const neoclr_aot_text**)`
signatures and the same success/runtime-error/exhaustion statuses as Int32 formatting.
Native-width conversion preserves bits on the ARM64 target and imports no helper.
The existing `--bind-int32-to-string` capability remains separate.

UInt32 uses the Int32 evaluation category. Int64/UInt64 and native-width storage use
64-bit lanes through locals, fields, calls, return slots and borrows. Unchecked integer
conversions truncate or extend according to the opcode: signed Int32 widening sign
extends; `conv.u8`/`conv.u` zero extend its 32-bit bit pattern. Native integers are
64-bit for this target, not a portable assumption for future 32-bit targets. Int64
wrapping add/subtract/multiply also compile. Checked wide arithmetic, wide division/
remainder, general pointer operations, wide generic payloads and wide comparisons
remain unsupported; root exports remain Int32-only.

Decimal formatting uses bounded 21-byte local buffers, preserves signed/unsigned
endpoints and copies text to invocation-owned immutable storage. There is no shared
managed framework dependency or change to ABI v4's layout/lifetime. As with ordinary
.NET integer formatting, the value's signedness controls its decimal representation;
this experiment follows neoCLR's existing culture-independent service contract,
without adding CLR format providers or claiming equivalent culture APIs/performance.

`wide-integers.neoil` compares exact interpreter/native diagnostics for endpoints,
signed/unsigned widening, native-width conversions, narrowing, mixed record output
copies and wrapping arithmetic. A one-byte-short arena fails without changing result,
cursor or bytes; an exact-size arena succeeds across repeated calls with canaries.
`wide-integers.rvn`, `verify_interactive.py --wide-integers` and
`wide-integers-validation.json` exercise fresh Raven Console output and broken-pipe
parity in a standalone executable. The pinned Raven bridge accepts native-width
defaults but rejected explicit int-to-nint casts in the initial probe; nonzero native
conversions are consequently validated at CIL level. Invalid `UL`/minimum-literal
spellings in that probe were replaced with tested Raven casts/expressions, not new
compiler syntax support. Stream/interface calls and multiple closed carrier shapes
remain the next Write/ReadLine dependencies.

## Multiple closed value shapes (2026-10-08)

Closed-world selection now specializes several instantiations of the same value
carrier, replacing the earlier one-shape restriction. Each shape retains separate
storage and method identities even if its native lane widths equal another shape's.
Constructors retain their constructor names under private owners; calls bind exact
closed signatures. Repeated references reuse the same shape/body. Nongeneric static
companions still contribute metadata identity only.

Private cloned types, fields and methods receive distinct metadata tokens. Original
access/readonly facts remain active and original definitions are retained in reports,
with `typeArguments`, `compiledName`, `expandedIndex` and `compiledIndex` where
applicable. Runtime fault frames use original function names. The original explicit
load set is verified before cloning; single-artifact inputs requiring new type/method
tokens must also supply their declared verification dependencies. The older bounded
single-shape projection remains compatible. Generic reference classes, generic
lexical owners, constraints and interface dispatch remain unsupported.

Bounds remain 32 selected shapes (including nongeneric dependencies), 32 additional
function bodies and 128 selected functions. Discovery counts closed shapes, rather
than just source definitions. This is bounded monomorphization, not a stable generic
ABI or general trimming policy. It follows the existing .NET generic-specialization
comparison in the [value experiment](../aot-values/README.md): concrete layouts simplify
native calls but duplicate code/metadata. No code-size or performance advantage is
claimed, and there is no representation-based merging of distinct nominal shapes.

`multiple-values.rvn` combines Option<int>, Option<string>, input Option<byte> and
several Result payloads, including Error/None and both pattern forms. The fresh run
selects 24 types/55 functions, with 14 generic shapes and 23 additional function bodies.
`verify_interactive.py --multi-values` and `multiple-values-validation.json` retain
source/shape provenance and interpreter/native output, EOF and broken-pipe parity.
Focused CIL tests exercise two same-width generic layouts, cloned member returns and
faults, and excessive-shape rejection. This removes one of the ordinary stream
library's prerequisites; the next boundary is interface views and calls.


## Standard stream views (2026-10-08)

The explicit reference-arena profile now preserves nongeneric class interface views
through storage, casts, parameters and output borrows. Views retain the original
object pointer and identity; no wrapper is allocated. `isinst` returns null for a
mismatch, while `castclass` preserves null and reports the interpreter's RuntimeError
for a nonnull mismatch, at the original instruction. Generic interfaces, inherited
interfaces and interface method dispatch remain outside this slice.

`stream-views.rvn` compiles the ordinary Console.OpenStandardInput/Output/Error
factories and their source-owned constructors. The verified empty System.Object
base and ordinary constructor are privately renamed to avoid installing a second
runtime Object slot registry. Original load-set access/conformance checks precede
this projection; it admits no Object virtual slots or general class inheritance.
Reports retain the original identity and expose `objectBaseProjection` explicitly.

Like CLR interface references, these views preserve reference identity. The private
native type-tag checks provide a small closed-world implementation, at the cost of
excluding dynamic loading and broader inheritance; they establish no stable ABI.
The existing [research](../../native-execution-investigation.md) comparison remains
applicable. The region still retains all objects until invocation reset.

Seventeen Console and eighteen linking tests pass, including eight interpreter/native
cast and alias cases. `verify_interactive.py --stream-views` and
`stream-views-validation.json` record fresh Raven compilation, normal output, exact
broken-pipe fault parity, standalone execution and libSystem-only dependencies.
The pinned Raven compiler intermittently rejected unchanged standard-stream members
before the successful retry; this slice does not fix that producer issue. Next:
compile calls through these stream interfaces, beginning with standard input.


## Standard input dispatch (2026-10-08)

The reference-arena profile now dispatches bodyless nongeneric interface contracts
to exact public implicit implementations on classes constructed by the selected
program. Selection and specialization discover implementations to a fixed point,
including calls and constructions in their bodies. Unconstructed implementations
stay excluded; original load-set conformance/access verification still precedes
projection. `interfaceDispatch` reports the private contract, type and method rows.
Explicit implementations, interface inheritance/default bodies, generic reference
classes and class virtual/override dispatch remain unsupported.

Native dispatch compares private object tags and forwards the receiver, arguments,
result storage and fault context. Null receivers fail at the caller's callvirt;
implementation faults retain their ordinary managed frames without a synthetic
interface frame. This follows the CLR interface-call behavior at the language level.
Compared with general runtime dispatch machinery, the bounded linear tag table is
simple and statically linkable, but scales with selected implementations and supports
neither dynamic loading nor a stable external object ABI.

`input-stream.rvn` uses ordinary Console.OpenStandardInput, InputStream.Read and
Close, with byte-array mutation, zero-count reads, invalid-range and closed-stream
Result errors, successful bytes and EOF. Its selected graph needs 60 functions and
33 types, including the twelve-field StreamError carrier. The private profile now
allows 64 types and sixteen flattened lanes/fields, with 128-byte internal call
result storage; 128 functions, 32 extra function clones and the 64-KiB frame bound
remain. Rebuild private objects and hosts together; no stable aggregate ABI is promised.

The focused interface test covers two implementations, an excluded unused
implementation, null receivers and exact implementation-fault traces. The value
suite checks sixteen-lane calls and oversized-layout/type rejection. All 78 focused
Console, linking and value tests pass. Fresh standalone
sample evidence is recorded by `verify_interactive.py --input-stream` and
`input-stream-validation.json`. After the first successful fresh compilation, the
pinned producer rejected unchanged standard-input members on two retries. Validation
therefore reused that successful metadata artifact with `--reuse-compilation`,
checking source/compiler/dependency hashes and retaining the original successful
command and assembly hash. This does not claim the producer issue is fixed. Standard
output/error Write/Flush bindings are the next Console slice; text readers/writers
and Console.ReadLine still need further library dependencies.


## Standard output and error streams (2026-10-08)

`--bind-console-stream-output` explicitly binds the exact supplied InternalCall
ConsoleWriteBytes(Boolean, arrayref<Byte>, Int32, Int32) and ConsoleFlush(Boolean)
services. It requires `--compile-system --reference-arena`. Ordinary factories,
Write/Flush/Close methods, range/status decoding and Result carriers remain compiled
CIL. Native objects import only the selected service symbols, which the standalone
host links from `console.c`; no shared neoCLR or .NET framework is introduced.

The byte adapter borrows array contents synchronously. It validates signed ranges
before the 65,536-byte operation limit, skips host I/O for zero counts, routes the
Boolean channel to stdout/stderr, and never closes host resources. Stdout flushes
through the last LF and stderr on every nonempty write, matching the interpreter's
StdioConsole policies; explicit Flush handles remaining buffered bytes. Nonnegative
counts represent successful writes; -7/-8/-10 represent range/limit/I/O failures.
Generated service lowering maps these to the existing erased Int32/Byte contract,
checks impossible host counts, and turns unknown adapter failures into IoFailure.
Null arrays report RuntimeError with the managed caller's original fault frame.

Result<unit, StreamError> requires inhabited Void storage: unit now travels through
locals, parameters, fields, closed generic payloads and output borrows as one private
zero-valued lane. It remains distinct from no-result methods. A focused test checks unit
initialization/copy/forwarding without corrupting an adjacent Int32 field.

`output-stream.rvn` exercises ordinary stdout/stderr interfaces, NUL bytes, zero
counts, invalid ranges, flush, close and closed-wrapper errors. A broken pipe returns
a recoverable Flush error; this app handles it with exit 6 and no fault diagnostic.
This is consistent with the existing neoCLR stream Result design; unlike CLR stream
exceptions, I/O failure here is an explicit return value. Native execution preserves
that choice rather than introducing another error model.

The selected sample has 43 functions and 24 types. `output-stream-validation.json`
records compiler/interpreter/native evidence and executable-only deployment with
libSystem as the only dynamic dependency. The first fresh Raven compilation is
reused after backend additions via matching producer/source hashes. Twenty-one
Console and forty-three value tests pass, including service opt-in/impostor rejection,
33 adapter-status cases, null fault parity, range/limit/channel checks and broken
pipe checks for line output, stderr and explicit flush. Full text Console.Write,
Out/Error writers and ReadLine remain the next dependencies to compile.


## Closed reference types and inherited interfaces (2026-10-08)

The explicit reference-arena profile now specializes bounded closed generic classes
and interfaces alongside value shapes. Each closed class keeps its own object tag,
field layout and member bodies. Implicit interface dispatch resolves exact closed
source signatures before private generic erasure; interface inheritance, safe class/
interface upcasts and inherited member calls retain the same underlying pointer.
Original source conformance/access verification remains required, including single-
module generic reference inputs. Constraints, generic instance methods, explicit
implementations, class inheritance beyond the empty Object base and class virtual
slots remain unsupported. All existing shape/function/frame limits remain.

`generic-views.neoil` exercises Cell<Int32>/Cell<Byte>, inherited Read/Mutable views,
mutation, narrow return values, invalid cross-instantiation casts and null callvirt.
Native output and exact fault diagnostics match the interpreter. The full focused
set passes (22 Console, 18 linking and 43 value tests); the inherited-call test also
passes after adding implicit parent-interface argument compatibility.

As with CLR generics/interfaces, closed instances remain nominally distinct and
upcasts preserve identity. This POC specializes bodies rather than sharing generic
code, trading a simple private layout/dispatch model for larger native images and
explicit discovery limits. It remains invocation-region storage, not CLR-style GC.

The real ArrayList<byte>/Sequence<byte> Console consumer exposed `array.reserve`
as the next dependency after generic admission. It must retain checked unreadable
slots rather than silently substituting zero-initialized newarr. Full Console.Write
also reaches String-valued erased runtime services; those are later slices.


## Checked byte capacity and ArrayList (2026-10-08)

`array.reserve Byte` now allocates invocation-owned backing capacity with a private
initialization byte for each payload byte. Stores publish slots; indexed reads of
unwritten slots report the same RuntimeError and instruction as the interpreter.
Ordinary newarr remains zero-initialized. Aliases share both bytes and publication
state. Range/null/array-limit checks and native-memory exhaustion retain their
existing order and result-publication rules. ConsoleWriteBytes checks the selected
range's initialization before host output, while invalid ranges and zero counts
retain their recoverable service outcomes.

For now, a selected program containing a reservation rejects every element-address
instruction before emission. This conservative restriction avoids losing publication
state through raw interior pointers; initialization-aware borrows remain future
work. Other array element types are still unsupported. Reserved storage uses twice
the byte payload space plus its header and retains old ArrayList buffers until the
invocation ends; this is a correctness foundation, not an allocation optimization.
See the existing [reserved-capacity design](../../reserved-array-capacity.md) for the
comparison with CLR uninitialized allocation and its different read contract.

`collections.rvn` compiles ordinary ArrayList<byte> construction, Add/growth, indexing
and inherited Sequence/Collection member calls, then prints through Console. The
15-function/nine-type graph grows through capacities 0/4/8/16 and observes mutation
through its interface alias. `verify_interactive.py --collections` and
`collections-validation.json` record the successful fresh compiler artifact reused
after backend additions, exact interpreter/native output and broken-pipe fault
parity, plus executable-only deployment and libSystem-only dynamic dependencies.

This artifact selects the seed's empty System.Console owner. It receives a private
nominal name to avoid collision with the backend verifier's bundled seed; its original
shape, access facts, method bodies and source diagnostics remain. The existing Object
base projection uses the same structural renaming helper. `staticOwnerProjections`
records these changes; literal strings are never rewritten. The object allocator's
ceiling now also matches the admitted sixteen padded fields (136 bytes including the
header), with exact-fit, exhaustion and neighboring-buffer canary checks.

Focused tests cover uninitialized/initialized and alias reads, null/range/limit faults,
zero capacity, arena exhaustion, ordinary default arrays, native output validation
order and early borrow rejection. Full Console.Write still requires String-bearing
erased services and text operations; the collection dependency now compiles normally.


## String-valued erased services (2026-10-08)

The private System.Value transport now pairs its I32 discriminant with an I64
payload lane, admitting exact String packing, type tests and recovery alongside
Int32/Byte/Boolean/unit. Literal and invocation-arena pointers retain all address
bits through locals, calls and output borrows; null String remains a String-tagged
null. Primitive payloads normalize before widening and narrow on recovery. Wrong
payload extraction still reports RuntimeError at the original instruction, and
System.Value still has no default initialization. Character, object, array and
aggregate erasure remain unsupported.

Static generic helpers may now specialize String arguments, enabling the ordinary
RuntimeServices.IsValue<String>/UnpackValue<String> bridge helpers. The internal
payload representation is not a stable interop ABI: rebuild related native bodies
together. The exported Int32 entry and caller-owned context layout remain ABI v4.
All String lifetimes remain unchanged: render any dynamic fault message before
resetting or releasing its invocation region.

`erased-text.neoil` tests UTF-8/NUL literal payloads, minimum-Int32 formatted text,
closed generic identity, output copying, empty and null strings, and both directions
of invalid extraction against interpreter results and exact fault diagnostics.
Existing primitive erased-value and native input/output adapter tests also pass.
This is a bounded tagged transport, unlike the CLR's general boxing/object model;
it avoids native managed-object boxing allocation for these admitted payloads at
the cost of excluding general erasure and GC lifetimes. Full Console.Write now
reaches the selected-function/clone budget, which needs a measured extension before
its remaining text services can be admitted.


## UTF-8 byte counts and slices (2026-10-08)

`--bind-utf8-text` (with `--compile-system --reference-arena`) now admits the exact
reserved StringByteCount and StringSliceUtf8 InternalCalls. Ordinary String and
encoder wrappers remain compiled CIL. Counts use UTF-8 bytes; slices preserve bytes,
including NUL, and require scalar boundaries. Negative/out-of-range requests return
an erased Byte(1), invalid boundaries Byte(2), and successful slices an erased String.
Range validation precedes boundary checks, and recoverable errors allocate nothing.
Null native text arguments fault with RuntimeError. Successful slices occupy the
caller-owned invocation arena; exhaustion reports NativeMemoryLimitExceeded without
publishing a result or advancing its cursor. These helpers add no shared runtime.

The focused `utf8-text.neoil` consumer compares 135 native/interpreter executions,
including exact dynamic fault messages for successful slices, UTF-8/NUL data, nulls,
empty slices and both invalid index classes. Adapter tests cover exact fit, exhaustion,
canaries, large indices/counts and failure publication. Exact-service admission and
missing/duplicate capability options are tested. Unlike CLR String.Substring's UTF-16
indexing, this follows neoCLR's existing UTF-8 API: byte offsets avoid transcoding in
stream encoders but callers must respect scalar boundaries. This is existing platform
semantics, not a new claim of superiority or a stable native ABI.

The full Console.Write graph still needs byte-value-array encoding and managed byte
array interface views, followed by a measured extension of selection budgets.


## Immutable encoded byte values (2026-10-08)

The UTF-8 capability also binds exact Utf8Encode(String) -> Byte[]. Its native
producer stores UTF-8 bytes in an invocation-owned immutable value-array snapshot,
limited to 65,536 bytes. Admitted code can copy/pass/return these values, inspect their
length and read indexed bytes. The ordinary bridge loop copies them into mutable
`arrayref<Byte>` storage. Value and managed arrays are distinct types; this does not
turn value arrays into aliased mutable references. Element mutation/borrows, defaults,
array-valued fields and general value-array producers remain outside the profile.

Immutable snapshots share backing storage safely. Replacing an initialized local,
argument or output slot still requires equal lengths, matching the interpreter's
fixed-extent value-array contract; a mismatch faults at the store. Unassigned local
slots have a private zero marker, never an observable default Byte[] value. Null
native String arguments fault; range errors, the array limit and arena exhaustion
retain their existing fault codes and original managed caller frames.

`utf8-encode.neoil` runs 17 interpreter/native comparisons for multibyte UTF-8/NUL,
empty/null input, snapshots across same-length replacement, rejected extent changes,
indexed bounds and an ordinary managed copy loop. Adapter checks exercise exact-fit
allocation, failure publication, 65,536/65,537-byte limits and canaries. Unlike .NET's
managed byte[] returned by Encoding.UTF8.GetBytes, the existing neoCLR native service
returns value bytes and its temporary bridge copies them into a managed array. This
keeps that contract intact but costs an additional retained arena allocation/copy;
it is not a proposed permanent encoding optimization.


## Ordinary String instance wrappers (2026-10-08)

The verified public nonvirtual String wrappers now compile as private functions with
an explicit String receiver. Their ordinary CIL bodies and argument indices stay
unchanged; selected source identities remain in reports and fault frames. This
projection is needed because the backend's bootstrap String definition belongs to a
different module. Original load-set verification still precedes projection. It does
not admit String constructors, field storage, virtual members or generic instance
methods. Current String CIL uses direct instance calls: null receivers reach the
ordinary body, and native services validate null arguments with RuntimeError, exactly
as the interpreter does. No additional null-call convention is imposed by AOT.

The tested `utf8-text.rvn` consumer exercises GetUtf8ByteCount and SliceUtf8 through
real library Result patterns (`let ... else`, `if let`), valid UTF-8/NUL, empty slices,
invalid boundaries and ranges. Run `verify_interactive.py --utf8-text` with the pinned
producer arguments above. [Validation](utf8-text-validation.json) records the producer
command, artifact hashes, interpreter/native output and exact broken-pipe fault parity,
and deployment into an executable-only directory with an empty environment. Only
libSystem is dynamically linked. The successful source compilation from run 2 is
reused with verified matching source/producer hashes after backend fixes; the report
records that provenance rather than claiming a fresh compiler run.

Selection limits now admit 256 functions, 128 closed types and 64 function clones.
The measured Console.Write graph already needs 151 functions, 68 types and 37 clones
before nominal array dispatch, exceeding the previous 128/64/32 profile. Focused
boundary tests accept the new limits and reject the next function/clone or oversized
type discovery. Stack/frame, field and arena limits are unchanged. These are explicit
POC limits, not neoCLR platform promises. Like ordinary .NET instance methods, wrapper
logic stays managed compiler input; the private projection is a temporary bootstrap
verification detail, not a public calling convention.

Full Console.Write next reaches boxed empty union cases and managed-array interface
dispatch. Exact String pattern casts were already admitted. ReadLine follows the writer path.


## Boxed empty union cases (2026-10-08)

The reference-arena profile now lowers `box` for closed empty value records, including
specialized generic records. Each operation allocates a distinct eight-byte header
with the record's private type tag. Aliases preserve reference identity, separate
boxing operations remain distinct, Object views succeed, and unrelated class casts
retain the interpreter's fault behavior. The original boxing instruction is lowered
to a private helper after full source verification; helpers count toward the function
budget and are reported separately in `emptyRecordBoxes`/`emptyRecordBoxSites`.
Resource failures retain the original box/caller instruction frames, omitting the
synthetic helper. The native entry never publishes a result on failure.

This admits the generated EncoderState.Value/HasValue bodies without replacing union
logic. [The Raven sample](boxed-cases.rvn) prints HasValue for Ready, NeedsOutput and
Finished through ordinary Console.WriteLine. Its [fresh-producer validation](boxed-cases-validation.json)
records standalone interpreter/native output and exact broken-pipe diagnostics,
executable-only deployment and libSystem as the sole dynamic dependency. Focused
native tests cover generic empty records, identity, Object views, invalid casts,
exact-fit/exhaustion, canaries and original fault sites; negative tests reject payload
records, missing arena capability and helper-budget overflow.

Like CLR boxing, each admitted value gets distinct reference identity. This bounded
implementation supports only empty records and invocation lifetimes: payload boxing,
unboxing, boxed value interface dispatch, virtual Object methods and GC remain future
work. The cost is a small arena allocation retained until invocation reset. The
writer graph now reaches the nominal byte-array-to-Sequence<byte> interface boundary.


## Console text writers and nominal byte-array views (2026-10-08)

Managed byte arrays now keep their verified nominal backing identity during closed
specialization and dispatch selection. Array/interface/nominal views share the same
pointer: Count, indexed access and iterator implementations execute the ordinary
backing type's CIL. Its single intrinsic storage field denotes the array itself;
there is no separately allocated wrapper. Ordinary and reserved byte-array headers
both participate in dispatch, while reserved initialization checks remain active.
No interface is inferred from a class name or field shape: original assembly backing
metadata and conformance must verify before this private projection. Backing field
replacement/borrowing and ordinary class allocation of an array remain rejected.
Reports include the source backing definition and compiled type index.

The Raven `text-writer.rvn` app now compiles ordinary Console.Write(string/int),
Console.Error and StreamWriter.Write/Flush, including their encoders, Result branches,
ArrayList buffers and nominal array views. It exercises empty writes, UTF-8/NUL,
stdout/stderr and a multibyte character across the 256-byte encoder boundary.
`verify_interactive.py --text-writer` checks exact interpreter/native output and
broken-pipe faults, object imports and executable-only deployment with libSystem as
the sole dynamic dependency. [Validation](text-writer-validation.json) records the
successful fresh producer run and the native artifacts. The pinned producer also
returned its known intermittent missing-Console-member diagnostics in runs 3/4;
run 5 succeeded with the identical source/producer inputs, without source workarounds. This reaches the requested ordinary text-writing path;
Console.ReadLine is the next sample-driven step.

Focused array-view tests compare ten native/interpreter executions covering ordinary
and reserved arrays, mutation through an interface, alias identity, nominal views,
empty/null arrays, unwritten slots, bounds and unrelated interface casts. Negative
tests reject missing/corrupt backing contracts and attempted class allocation.
Like CLR arrays, these arrays expose shared identity through collection interfaces.
neoCLR's existing contract expresses that relationship with verified library backing
metadata and ordinary CIL methods; the private header/field lowering is backend
implementation detail. The bounded cost remains invocation-retained arrays and
closed dispatch targets; this is not general array covariance or GC support.

## Console line input and strict UTF-8 decoding (2026-10-08)

The ordinary `Console.ReadLine(128)` → `TextReader` → `StreamReader` → decoder
path now compiles alongside the prompt writer. The [Raven reader](text-reader.rvn)
uses `let Ok(...) else` and `if let Some(...)` to distinguish read errors, a line,
and end of input. The only new native binding is exact reserved
`neoCLR.Runtime.Utf8Decode(arrayref<Byte>) -> Value`, under `--bind-utf8-text`;
public reader bodies, buffering, line limits and union branches remain ordinary CIL.

Decoding validates every initialized array slot before checking strict scalar UTF-8.
Malformed sequences return the existing erased Byte error; valid text is copied to
invocation-owned immutable storage. Embedded NUL is retained. Null/unwritten inputs
fault, while exhaustion publishes neither a result nor an advanced arena cursor.
The focused tests compare all 256 single-byte inputs and scalar boundary, overlong,
surrogate, out-of-range, truncated and misplaced continuation sequences with the
interpreter. Direct adapter tests cover empty/exact-fit storage, canaries, exhaustion,
invalid-input allocation avoidance and unread-slot precedence.

The combined reader/writer graph selects 268 functions, 110 types and 73 clones
before empty-record boxing helpers. Private limits are now 512 functions (including
helpers), 128 types and 128 clones. Nested values permit 32 flattened lanes while
retaining the 16 direct-field bound. Matching call-result storage is 256 bytes and
reference allocation allows up to 264 bytes including its header. Boundary tests
cover call returns, object allocation/canaries, function/clone ceilings and oversized
layouts. This is a bounded experimental backend contract, not a public stable ABI.

`verify_interactive.py --text-reader` compares ASCII, UTF-8, NUL, LF/CRLF, empty
lines, EOF, final unterminated lines, malformed input and overlong lines, plus exact
broken-pipe fault diagnostics. Its interpreter budget is explicitly ten million
instructions: the default budget expires inside the managed decoder before the
129-byte line reaches the sample's limit. Native execution has no instruction fuel
counter. The sample host still retains allocations in a 64 KiB invocation arena;
it does not promise the public 65536-byte maximum fits that host resource budget.

Compared with .NET Console.ReadLine, neoCLR preserves its existing explicit
`Result<Option<string>, TextReadError>` and UTF-8 byte-limit contract. The AOT path
reuses that behavior rather than introducing another reader implementation. Its
current cost is invocation-retained decoder buffers and copied immutable text;
long-running input needs a managed lifetime policy or suitable host invocation
boundaries. No throughput or allocation advantage is claimed.

[Standalone reader evidence](text-reader-validation.json) records ten matching input
cases, exact broken-pipe traces/exit status, executable-only deployment and only
libSystem dynamically linked. It reuses the successful fresh compilation from run 1
with matching source/producer hashes; run 2 raises the interpreter instruction budget
to test the actual line-limit result. Default-limit and consecutive-read coverage
remain the next slice.

## Consecutive Console reads (2026-10-08)

The [session sample](console-session.rvn) exercises the default `ReadLine()` overload,
explicit invalid/excessive byte limits, `Console.In.ReadLine`, and `TextReader.Close`.
It reads three lines through separate wrappers, verifies the closed wrapper returns
`TextReadError.Closed`, and continues reading through Console afterward. The ordinary
library owns these semantics; this slice adds executable coverage, not new bindings.
Nested `let Error(TextReadError.Case) else` and `if let Some(...)` patterns compile.

[Validation](console-session-validation.json) covers six input streams: three normal
lines, Unicode/CRLF/final unterminated input, empty lines, immediate EOF, early EOF,
and invalid UTF-8. Every stdout/stderr/exit matches the interpreter, including the
prompt's broken-pipe user fault. The executable still runs alone with an empty
environment and only libSystem dynamically linked. The explicit ten-million-instruction
interpreter budget remains part of the recorded command.

The pinned producer returned missing-Console-member diagnostics for source-path runs
1–4. A byte-identical copy under `/tmp` compiled successfully; the [observed producer
command and hashes](console-session-producer.json) are preserved and run 5 reuses that
artifact after source/producer hash checks. A single-processor attempt did not remove
the diagnostics. The cause remains unresolved: this is producer evidence, not a claim
that source relocation fixes the compiler. Default and bounded line input now have
standalone coverage. The remaining object WriteLine path needs virtual Object display
and, for its fallback implementation, runtime type metadata.

## Object Console output through verified overrides (2026-10-08)

The [object writer](object-writer.rvn) uses `Console.WriteLine(object?)` for an
application class with an explicit `ToString` override and for null. Closed selection
now discovers concrete override targets for the verified `System.Object.ToString`
slot, including classes constructed inside selected override bodies. It accepts
rootless classes with the interpreter's verified implicit Object contract, or direct
Object-derived classes. Same-named non-overrides do not qualify.

Only this closed slot is projected to a private dispatcher. Override bodies remain
ordinary CIL, receiver identity is preserved, and existing callvirt checks fault on
null at the caller. The dispatcher adds no synthetic managed frame. Original load-set
verification runs before projection; private virtual flags are cleared only for the
reported slot and its targets; the private slot gets a collision-free name to
avoid creating a nonvirtual hiding relationship under a declared Object base. The `objectDisplayDispatch` inventory records exact
compiled type/method targets. No new reserved runtime service or public API is added.

This reuses the [.NET/Object comparison](../../object-model-review.md) and the current
interpreter's override validation in `src/inheritance.rs`. Like CLR virtual calls,
Object-view calls select the concrete override and preserve a different meaning for
a direct base call. A closed native type-tag dispatcher was chosen over a new public
formatting service or a vtable ABI: it keeps ordinary method bodies and the existing
private header, at the cost of a target test per candidate. No speed advantage is
claimed. Stable vtables and metadata remain future design work.

The bound is deliberately conservative: every constructed class in this selected
program must have a qualifying override when the Object display slot is used.
Default Object display and explicit base calls still require type metadata; deeper
class inheritance, array display and boxed receiver display are not admitted here.
Programs that combine this slot with boxing/array construction are rejected rather
than silently dispatching those receivers incorrectly. Focused tests compare dynamic
UTF-8/NUL text, null faults and faults inside overrides with the interpreter; negative
cases cover missing overrides, non-overrides, private invalid overrides, direct base
calls and missing arena capability.

Declared-base tests also exposed a pre-existing constructor admission gap: without
its base initializer the interpreter faults at constructor return, while the earlier
native lowering could succeed. The profile now requires a derived reference
constructor to start with `ldarg 0; call Base::.ctor()`. Direct reference-constructor
calls are restricted to that single initializer; other allocations use `newobj.ctor`.
Conditional/missing/repeated initialization and constructor chaining are outside this
bounded native profile. The positive tests exercise both verified rootless overrides
and explicit Object bases with initialization; negative cases prevent the previously
admitted mismatch. A future construction-state analysis can admit more CIL shapes.

## Current Console coverage

These are development AOT capabilities, exercised by the linked samples and focused
contract tests. They are not a claim that the whole runtime library compiles.

| Console path | Native coverage |
| --- | --- |
| ReadByte | Byte/EOF/error union outcomes |
| WriteLine | String, blank line, Boolean, Char and integer overloads |
| Write | String and Int32 through ordinary text writers |
| OpenStandardInput/Output/Error | Non-owning stream construction and supported Read/Write/Flush/Close calls |
| In / Out / Error | Text-reader ReadLine/ReadToEnd and text-writer Write/Flush, with non-owning Close |
| ReadLine / ReadLine(maxUtf8Bytes) | UTF-8 lines, EOF, typed errors, default/explicit limits and consecutive reads |
| WriteLine(object?) | Null and qualifying explicit class ToString overrides; no default metadata formatting or boxed/array receiver support |

The next sample should combine parsing and repeated console interaction within an
explicit host lifetime budget. Before calling this general Console support, address
default Object display/type metadata, broader receiver dispatch and long-lived
allocation reclamation. ReadToEnd has its own bounded consumer below.
The pinned Raven producer's source-path/member-lookup issue also remains a separate
integration limitation. Trimming, stable external ABI and HTTP stay later steps.

[Object writer validation](object-writer-validation.json) records exact native/interpreter
output for the override and null, matching broken-pipe frames and exit 1, and the
standalone dependency check. It reuses the [observed producer command](object-writer-producer.json)
with a byte-identical `/tmp` input after the checked-in-path attempt reported missing
WriteLine overloads. The final validation records the compiler build containing both
the private slot rename and tightened constructor admission. Focused dispatch tests
also vary source-origin display names: the native slot name controls override choice.

## Reading the remaining Console input (2026-10-08)

`Console.In.ReadToEnd(maxUtf8Bytes)` now compiles its ordinary StreamReader and
decoder CIL. The [reader sample](read-to-end.rvn) reads one line and then the remaining
input through the same reader, prefixes the result with `String.Concat`, checks EOF
with a zero-byte limit, and verifies closed-reader and invalid-limit outcomes. It
continues to exercise nested `let Error(...) else`, `let Ok(...) else` and `if let`
patterns. No reader-specific native shortcut is introduced.

The existing `--bind-utf8-text` capability now binds the exact reserved
`neoCLR.Runtime.StringConcat(String, String) -> String` service. Its C adapter copies
both immutable UTF-8 inputs into one aligned arena allocation, including embedded
NUL. Null inputs and size overflow retain RuntimeError; the native invocation budget
reports NativeMemoryLimitExceeded. Failure publishes neither an output pointer nor
an updated cursor. Ordinary static String wrappers use the existing private static
primitive projection; reports preserve their actual String/Char owner and source
identity. Original load-set verification still precedes projection.

This reuses the [String contract and .NET comparison](../../text-model.md): immutable
concatenation preserves neoCLR's UTF-8 contents rather than changing guest APIs or
adopting UTF-16 storage. The benefit is one service shared by ordinary library and
application CIL. The costs are copied bytes and invocation-retained intermediates;
repeated concatenation may accumulate quadratic copying/storage. This slice does
not claim throughput improvement or general long-running Console memory management.
The sample's 32-byte remainder limit fits the host's 64 KiB arena; the public 65536-byte
limit is not a guarantee that every native host budget can satisfy a read.

Focused tests compare eleven interpreter/native cases with nested calls, repeated
concatenation, UTF-8/NUL and null faults. Adapter tests cover identical/arena-backed
inputs, alignment padding, empty strings, exact-fit/exhaustion, overflow, invalid
arena state, canaries and unchanged failure outputs. Exact opt-in and source-service
conflict rejection remain covered.

`verify_interactive.py --read-to-end` covers nine inputs including LF/CRLF, EOF,
Unicode/NUL, an exact byte limit, an excessive remainder, invalid UTF-8 and an
unfinished scalar. `--isolated-compilation` explicitly stages byte-identical source
and compiler output in a temporary directory, records the paths/source hash, and
preserves a hashed artifact in the validation directory. This reproduces the temporary
source-path workaround without hiding it. The producer lookup issue remains unresolved.
Compilation reuse now verifies that preserved artifact hash before native validation;
the validation report also hashes the driver itself.

The [final validation](read-to-end-validation.json) passes all nine inputs and exact
broken-pipe fault/exit parity. The executable runs alone with an empty environment;
its only dynamic dependency is macOS libSystem. The [producer record](read-to-end-producer.json)
preserves the successful fresh isolated compilation and the earlier static-String
admission failure; final validation reused its hash-verified artifact after that
admission gap was fixed. All 43 Console tests and 18 linking tests pass.

## Raven lookup fixes and fresh native qualification (2026-10-08)

The earlier intermittent producer failures are now reduced to shared Raven lookup
bugs, rather than source-path behavior. The primitive CLI bootstrap has a limited
Console declaration alongside the native System.Runtime Console. Qualified expression
lookup selected the first namespace candidate, and source-assembly metadata lookup
could return a referenced CLI declaration. Wildcard imports had another first-candidate
path. Raven now keeps source-assembly lookup local and uses its existing compilation
type-selection policy for qualified and imported namespace types. Closure Object
resolution uses the owning compilation's selected special type. No new native mapping,
CLI encoding or runtime service is introduced.

The general fixes are on Raven `codex/source-object-metadata-resolution`
(`b7a22b9e1`, `8b46ab9eb`) and `main` (`9c7db32a2`, `26cc6caae`). The native backend
and tested compiler build remain on the former branch. Ordinary CLI regressions fail
on unmodified main and qualify the fixes independently of neoCLR; 68 focused import,
namespace, alias and lookup tests pass on each branch. Earlier qualification also
passed 46 lookup/closure/root tests on the integration branch and 22 applicable tests
on main. This is modern .NET evidence, not a .NET Framework/NanoFramework matrix.

The reader sample now uses `import System.*` and ordinary `Console` calls.
[Fresh validation](raven-lookup-validation.json) compiles the checked-in source without
isolated staging or compilation reuse, then compares interpreter and standalone native
behavior. All nine inputs and exact broken-pipe fault/exit parity pass; the executable
runs alone with an empty environment and only macOS libSystem as a dynamic dependency.
Earlier validation files retain their original producer limitations and hashes.
No claim is made that all native emitter or lookup gaps are closed.

The driver now records and checks adjacent compiler DLLs and dependency/runtime
configuration JSON, including the binding and native-emission implementation assemblies.
A driver-DLL hash alone was insufficient to detect those implementations changing.
Legacy reports lacking the complete producer hashes are rejected for reuse; a negative
check confirms a missing Raven.CodeAnalysis hash rejects before artifact copying.
This strengthens validation provenance without changing guest compilation semantics.

## Interpolation argument conversions (2026-10-08)

Raven's synthesized String.Concat calls previously omitted argument conversions.
`"Value: $value"`, `"${value}"` and `"Value: " + value` could therefore reach native
metadata verification with Int32 where Object was required (NEOMETA003). The shared
binder now applies ordinary argument conversion and parameter-array mapping after
existing overload selection. Raven integration commit `9d2f6ae4e` and main commit
`45650a975` carry the same fix; 11 focused tests pass on each, including emitted .NET
execution, null text and evaluation order. Three new conversion regressions failed
on both branches before the fix. This changes no Runtime Contract defaults or ABI.

[interpolation.rvn](interpolation.rvn) freshly compiles into native CIL, verifies and
runs integer interpolation/addition for 42 and both Int32 endpoints, plus null text.
Its standalone AOT admission still rejects boxed Object display, and rejection must
publish no object file. This is the next native profile boundary, not a compiler
emission error. A richer Counter probe also ran in native CIL; its class without a
ToString override hits the separate default-display metadata guard during AOT selection.

[interpolation-text.rvn](interpolation-text.rvn) exercises the supported String-only
path, preserving Unicode and embedded NUL through native CIL and standalone ARM64.
The executable runs alone with an empty environment and links only libSystem; host
adapters are linked into it. [Validation](interpolation-validation.json) records fresh
commands and compiler, bundle, adapter and artifact hashes. Long command output is
compacted with full-output hashes; the current driver also compacts long output. Reproduce with:

```sh
SDKROOT="$(xcrun --show-sdk-path)" python3 docs/experiments/aot-console/verify_interpolation.py \
  --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/neoclr-native-poc --output /tmp/fresh-interpolation
```

Compared with .NET's Object formatting/boxing support, this is a smaller explicit
native profile. Normalizing calls in Raven preserves semantic information for either
backend; it does not supply native boxed primitive dispatch or general type metadata.
The existing bounded text arena, private ABI and UTF-8 contracts remain unchanged.

## Boxed Int32 Console display (2026-10-08)

The next bounded profile admits `box Int32` and dispatches verified Object.ToString
calls on those boxes through the existing Int32 UTF-8 formatter. Each boxing operation
allocates a distinct 16-byte arena object: private eight-byte type tag, four-byte copied
payload and zeroed padding. Changing the original value cannot change the snapshot.
The tag is a generated private record identity, not an exported native type or stable
ABI. Existing Object views and reference equality preserve identity. Empty-record
boxing and verified class overrides retain their previous behavior.

Object display with Int32 boxes requires both `--reference-arena` and
`--bind-int32-to-string`. The ordinary Console.WriteLine(Object) wrapper compiles;
no guest method is replaced by a host Console shortcut. Boxing and formatting share
the bounded invocation arena. Allocation/formatting exhaustion records
NativeMemoryLimitExceeded at the original caller instruction, does not publish the
entry result, and omits private helper frames. No host adapter or public ABI changes.

[boxed-int32.rvn](boxed-int32.rvn) is the fresh Raven consumer. The updated
[driver](verify_interpolation.py) qualifies it alongside the text interpolation
control and the remaining mixed interpolation rejection. [Evidence](boxed-int32-validation.json)
records the exact tool/bundle hashes and standalone interpreter/native parity for
Int32 minimum, -1, 0, 1, 42 and maximum. The executable runs alone with an empty
environment and only libSystem dynamically linked. Long diagnostics are compacted
with their full-output hashes. The earlier interpolation evidence is historical.

The 44 focused Console AOT tests pass, including a new metadata consumer
[boxed-int32.neoil](boxed-int32.neoil). It checks distinct boxes, copied values,
Object views, exact integer output, both allocation failures and formatting failure
with exact managed stack traces, untouched result storage and arena canaries. Missing
formatter opt-in rejects before publishing an object file.

This reuses the [Object/CLR comparison](../../object-model-review.md) and
[boxing contract](../../boxed-interface-values.md): copied boxing and reference
identity match the intended CLR ergonomics, while this implementation deliberately
uses invocation lifetime instead of a managed collector. A formatter shortcut on an
unboxed value would lose observable identity; full general boxed dispatch would require
more metadata and lifetime machinery. This bounded representation preserves identity
at an allocation/copy cost; no performance advantage is claimed. Wider primitive
boxes, unboxing, boxed interfaces, general Object methods and reclamation remain open.

Mixed integer interpolation now advances past boxing admission and fails at the
String-to-Object conversion: immutable UTF-8 text pointers do not yet have identity-
preserving Object views. That is the next bounded step. Arrays and non-Int32 boxes
combined with Object display still reject. No Raven compiler or Runtime Contract
change is needed for this slice; the producer is the previously fixed integration
branch, with its general binder fixes already on Raven main.

## String Object views and mixed interpolation (2026-10-08)

The next slice closes the String-to-Object boundary for the bounded reference-arena
profile. [interpolation.rvn](interpolation.rvn) now compiles into a standalone ARM64
executable: integer endpoints, value-only interpolation, string addition and null
text match native CIL interpretation. The ordinary String.Concat(Object, Object),
Object.ToString and Console wrappers remain in the compiled call graph. The updated
[driver](verify_interpolation.py) also reruns String-only interpolation and boxed Int32
controls. [Evidence](string-object-views-validation.json) records fresh compiler/bundle
hashes, isolated execution, libSystem-only dynamic linkage, and exact output-fault
message/trace/exit parity on a broken pipe. Earlier rejection evidence is historical.

Private String Object views tag the low bit of an aligned immutable UTF-8 pointer.
Null remains zero; converting back to String removes the tag, and Object.ToString
returns the original text. Casts distinguish text before reading an object header.
Failed type tests produce null; invalid casts and null virtual calls keep the
interpreter's fault codes and caller traces. Reference equality normalizes String
and Object views to the same identity. This is an internal compiled representation,
not a stable external object layout, metadata sidecar or public API change. Host
adapters continue receiving ordinary untagged text pointers; ABI v4 context layout
and existing native adapter signatures are unchanged.

Identity requires fresh storage for separately evaluated literals. When the selected
program exposes text identity through casts or ref.eq, every executed ldstr copies
its image template into the invocation arena using the existing private concat
adapter with empty text. Aliases reuse that storage, so repeated upcasts do not allocate
wrappers. Programs without observable text identity retain the previous image-literal
path. Exhaustion during materialization reports NativeMemoryLimitExceeded at the ldstr
caller site and leaves the entry result untouched. This can increase arena use for
loops and string-heavy programs; there is no collection or reclamation in this POC.

The metadata consumer [string-object-views.neoil](string-object-views.neoil) checks
repeated execution of the same literal, distinct equal text, repeated Object upcasts,
mixed-view identity, downcasts, wrong-class/boxed-value tests, nulls and Unicode/NUL
output. It compares invalid-cast and null-call diagnostics exactly with interpretation,
and checks literal-allocation exhaustion and caller storage. Forty-six Console AOT
tests pass, including rejection without object publication for String interface casts
and CharText producers in the identity-observable profile. Those paths need native
interface metadata and fresh Char-to-String ownership respectively. Wider boxed
formatting, general Object methods and escaping native references remain unsupported.

The comparison reuses [String shared-owner identity](../../string-storage-design.md#shared-owner-identity--development-2026-09-24)
and the [Object model review](../../object-model-review.md): .NET-like reference identity
is the ergonomic target, while neoCLR retains UTF-8 and non-interned literal evaluation.
Allocating an Object wrapper on every cast would require extra arena storage and
special equality handling; returning the image literal directly would incorrectly
merge repeated evaluations. The tagged view preserves current identity without wrapper
allocation, at the cost of alignment/tag handling and identity-sensitive literal
materialization. This is a provisional internal implementation, not a measured
performance improvement. String interfaces and CharText identity are the next bounded
representation gaps; a mixed input/Result/formatting sample can drive their selection.

The comparison also found and fixed interpreter String ref.eq inconsistency in
`8027ec93`, independently covered by six ownership/GC/conversion tests. Raven compiler
behavior, the explicit target configuration, metadata encoding and native bundle
libraries are unchanged by either slice.

## Fresh Char-to-String identity (2026-10-08)

The CharText restriction above is now closed for the bounded reference-arena profile.
When text identity is observable, the existing explicit `--bind-character-text`
binding copies Char's immutable grapheme bytes into a fresh String owner on each
conversion. It reuses the same private arena-copy path as identity-sensitive literal
evaluation. Copying/aliasing the resulting String preserves identity; independently
converting the same Char produces distinct identities. Without observable identity,
the previous allocation-free representation remains valid and unchanged. CharFromString
validation still uses the pinned grapheme adapter, and no boxed Char dispatch is added.

The [metadata consumer](character-text-identity.neoil) checks two conversions of a
default NUL Char, fresh identities, repeated String/Object aliases, downcasts, invalid
casts and null faults against the interpreter. Arena exhaustion leaves the caller's
result untouched and reports the original CharText call site with no synthetic frame.
All 47 Console AOT tests pass, including existing grapheme validation, argument binding,
String identity and reference-profile coverage. String interface casts still reject.

The [Raven consumer](character-text-identity.rvn) combines ordinary Console.WriteLine(char)
wrappers with integer interpolation. Seven cases cover ASCII, å, emoji, a combining
sequence, a family ZWJ sequence, a flag and NUL. [Fresh validation](character-text-identity-validation.json)
compares interpreter/standalone output and exact broken-pipe faults, runs the ARM64
executable alone with an empty environment, and checks that only libSystem is dynamically
linked. The grapheme library is statically linked and included in the recorded hashes.
Reproduce with the previous driver arguments plus `--sample character-text-identity`;
repeat `--sample` to select several consumers, or omit it to qualify all four.

This extends the [existing String ownership comparison](../../string-storage-design.md#shared-owner-identity--development-2026-09-24)
and [text model](../../text-model.md). .NET Char is a UTF-16 code unit; neoCLR Char is
an extended grapheme with UTF-8 text. The native path preserves neoCLR's existing
fresh String conversion semantics instead of merging identities through shared Char
storage. The cost is bounded copying/allocation when observable; there is no performance
claim or change to the platform's text semantics. Arena exhaustion remains a POC resource
policy; general reclamation and escaping native references remain future work. No public
API, compiler, metadata, Runtime Contract, adapter signature or ABI layout changes.

## Verified String interface views (2026-10-08)

The next bounded slice admits String interface casts and identity without claiming
String interface method dispatch. Before the private projection removes primitive
metadata and erases generic arguments, AOT records which selected closed interfaces
String implements in the verified load set. The table maps to private compiled type
indices and includes inherited interface relationships. It is build evidence, not a
runtime metadata sidecar or stable ABI. Unknown/unverified conformance still rejects.

String-to-interface and Object-to-interface casts use that table. Matching views retain
the same tagged String owner; unrelated interfaces and mismatched generic arguments
produce null for isinst or the interpreter's RuntimeError for castclass. Views round-trip
through Object and String without wrapper allocation or identity changes. Null guards
remain unchanged. The old blanket rejection of programs combining String identity and
interface casts is removed. Any selected interface instance method supported by String
still rejects when text identity is observable, until native String interface dispatch
is implemented; no placeholder method or fabricated default result executes.

The [metadata consumer](string-interface-views.neoil) and focused tests cover inherited
View<String>/Root<String>, rejected Root<Int32>, unrelated interfaces, repeated literal
identity, aliases, String round trips, null calls, invalid casts and arena faults. A
valid String interface-call program verifies in the interpreter but rejects native
compilation before publishing an object file. All 48 Console AOT tests pass.

The [Raven consumer](string-interface-views.rvn) uses the real EquatableTo<string>
contract, casts its view to Object, narrows back to String and prints Unicode/NUL and
empty text. [Fresh validation](string-interface-views-validation.json) records retained
closed conformance, tool/bundle hashes, standalone interpreter/native output parity,
exact broken-pipe fault parity, executable-only/empty-environment execution and only
libSystem linked dynamically. Run the existing driver with `--sample string-interface-views`
to reproduce this slice; its default now qualifies five consumers.

This reuses the [Object/CLR comparison](../../object-model-review.md) and existing
[interface model](../../class-semantics.md). Like the CLR, a view preserves reference
identity and depends on actual closed interface conformance rather than a name match.
The POC stores only the selected conformance facts; full metadata and dispatch would
support more programs at higher implementation cost. Keeping the original conformance
snapshot avoids treating erased arguments or absent private metadata as proof that
String does not implement an interface. Compilation uses additional bounded temporary
storage; casts allocate no wrappers. No performance improvement is claimed. General
String interface dispatch is the next gap, naturally driven by EquatableTo<string>.Equals.
Raven/compiler configuration, metadata format, public APIs and the private context ABI
are unchanged; this is native-backend work consuming existing emitted contracts.

## String content equality prerequisite (2026-10-08)

Investigating EquatableTo<string>.Equals exposed a smaller prerequisite: its library
body ultimately uses neoCLR's typed String ceq, which the AOT value profile previously
rejected. This slice admits String ceq and compiles exact UTF-8 content comparison.
Equal addresses and two nulls compare equal; one null compares unequal; otherwise the
native code compares lengths and then bytes, including embedded NUL. Empty strings,
case and normalization differences follow the interpreter. Reference identity remains
the separate ref.eq operation. No service binding, allocation, adapter or ABI change
is needed for the comparison itself; ordinary library equality bodies stay compiled.

The [metadata consumer](string-equality.neoil) compares 12 cases with interpretation,
including both null orders, two nulls, empty text, unequal prefixes, non-ASCII text,
embedded NUL, case differences, decomposed/composed text and a dynamically concatenated
String. It checks that only the explicit concatenation consumes arena space. All 49
Console tests and 45 value-profile tests pass.

The [Raven consumer](string-equality.rvn) exercises ordinary == and != through parameterized
calls and a parameterized Join, so the dynamic case cannot disappear through source
constant folding. [Fresh evidence](string-equality-validation.json) records tool/bundle
hashes, exact interpreter/native output, broken-pipe fault parity and standalone ARM64
execution with only libSystem dynamically linked. Run the existing driver with
`--sample string-equality`; its default now qualifies six consumers.

This restores the existing [text equality contract](../../text-model.md) in native
execution. .NET-like String content equality is the ergonomic target; the instruction
being compiled is neoCLR's typed String ceq, not a claim of identical CLI instruction
semantics. The simple length/byte loop preserves UTF-8 semantics without allocating or
normalizing. No speedup is claimed; vectorization or library-assisted comparison should
be considered only with a relevant benchmark. String interface method dispatch remains
explicitly rejected: receiver projection and dispatch must still be implemented before
EquatableTo<string>.Equals calls become supported. No Raven compiler, Runtime Contract,
metadata format or public API change is introduced by this prerequisite.


## String interface method dispatch (2026-10-08)

The next slice closes the interface-call gap above. Closed-world discovery retains
verified String conformance, resolves the exact public implicit implementation of each
reached closed contract, and selects its ordinary CIL body and dependencies. After
specialization, only proven reached interface shapes remain on the private String
metadata. The native dispatcher distinguishes tagged String owners from class headers,
removes the private tag, and calls the projected String body with an explicit receiver.
Object.ToString's text/boxed-Int32 shortcuts now apply only to that exact display contract.
The private inspection report records `stringInterfaceDispatch` contract/target rows;
these targets participate in signature checks and recursion rejection.

The [metadata consumer](string-interface-dispatch.neoil) tests inherited generic views,
equal and unequal Unicode/NUL text, a null argument, a null receiver, a class implementing
the same contract, and a fault thrown inside the String implementation. Native diagnostics
match interpreter messages and frames exactly, without an extra dispatcher frame. A
recursive implementation rejects before publication, as does a borrowed String receiver.
All 50 Console and 45 value tests pass.

The [Raven consumer](string-interface-equality.rvn) calls EquatableTo<string>.Equals on
empty, Unicode, embedded-NUL, case/normalization-different and dynamically concatenated
Strings. [Fresh evidence](string-interface-equality-validation.json) records matching
interpreter/native output, exact broken-pipe faults, and standalone ARM64 execution with
only libSystem dynamically linked. Use `verify_interpolation.py --sample
string-interface-equality` with the usual compiler/runtime/bundle/output arguments;
the default now covers seven consumers. The compiler remains Raven integration revision
2acfd40ec (shared compiler code 9d2f6ae4e); exact binaries and bundle hashes are recorded.

The ergonomic comparison is .NET String's IEquatable<string> use: consumers call the
ordinary equality contract through an interface. Implementation uses neoCLR's existing
UTF-8 content semantics and verified CIL body, plus a bounded private tagged receiver
instead of claiming CLR object layout or dispatch machinery. No allocation is needed
for an interface wrapper, but this profile still materializes identity-observable literals
in its invocation arena; no performance improvement is claimed. Conformance discovery
conservatively selects String implementations of reached contracts. Unsupported bodies,
borrowed receivers, explicit implementations and general virtual dispatch still reject.
See the existing [text model](../../text-model.md) and [Object review](../../object-model-review.md).

No Raven emission, Runtime Contract configuration, CLI bridge encoding, public API,
metadata format or native context ABI changes. This remains experimental backend work.
Line-oriented input already has standalone coverage in the reader/session slices above.
The next consumer combines repeated reads with parsing and union outcomes.

## Interactive Int32 parsing (2026-10-08)

The [parse session](parse-session.rvn) combines the existing Console.ReadLine path
with ordinary Int32.Parse, numeric output, String equality and union patterns. It reads
at most three 32-byte lines, accepts `quit`, and handles EOF, read errors, parsed values,
InvalidFormat and Overflow. Both `let <pattern> else` and `if let <pattern>` are exercised.
This closes the next integration gap; line input itself was already implemented above.

`--bind-integer-text` now also binds the exact verified reserved
`neoCLR.Runtime.ParseInt32(String) -> Value` InternalCall. Only that service maps to
`neoclr_parse_int32_v1`; the public Int32.Parse wrapper and its Result construction stay
compiled from CIL. Static Int32 methods use the same private owner-free projection
already used for static String/Char wrappers. No metadata, public API, Raven emitter,
Runtime Contract configuration or native context ABI changes are introduced.

The allocation-free adapter scans the explicit UTF-8 length, including embedded NUL,
accepting only `[+-]?[0-9]+`. It continues grammar validation after detecting overflow,
so malformed text wins over range errors. Unsigned magnitude arithmetic handles
-2147483648 without signed overflow. Success returns an erased Int32; malformed/range
errors return Byte(1)/Byte(2) for the managed wrapper. Null input faults without
publishing output. Ordinary same-named methods and missing opt-in cannot acquire the
binding. The adapter is linked into the executable, not loaded from a shared framework.

This reuses the existing [numeric contract comparison](../../design/numeric-contracts.md):
.NET Parse/TryParse is the ergonomic baseline, while neoCLR's existing Result API and
whole-text ASCII grammar remain authoritative. A locale-dependent C parser or .NET-like
implicit whitespace policy would change that contract. A small bounded-integer adapter
was chosen over adding a second parsing policy; the cost is maintaining parity between
interpreter and native implementations. No throughput advantage is claimed. Other
numeric parsers remain separate native work.

Focused checks cover 154 native/interpreter comparisons: both endpoints, sign/zero forms,
long leading zeros, overflow, invalid syntax, Unicode digits, embedded NUL, every ASCII
suffix after an overflowing prefix, and null-fault diagnostics. The caller supplies zero
arena capacity during parsing to verify allocation independence. The Console suite has
52 passing tests, including exact-binding rejection.

Run `verify_interactive.py --parse-session` with the usual compiler/runtime/bundle/output
arguments. The session host retains its 64 KiB invocation arena; the three-line limit is
sample policy, not a replacement for eventual native reclamation or execution budgets.
The next useful consumer is a bounded request-line/route parser fed through Console,
so text parsing can drive the path toward HTTP before adding socket lifetime and I/O.

[Standalone evidence](parse-session-validation.json) qualifies 11 input streams against
explicit expected output and the interpreter: limits, signs/zeros, overflow versus
malformed syntax, whitespace/NUL/Unicode, quit, EOF, final unterminated input, CRLF,
invalid UTF-8 and excessive line length. Native stdout/stderr/exit and the broken-pipe
fault trace match exactly. The executable runs alone with an empty environment and only
libSystem dynamically linked. The final run reuses the first run's successful fresh
Raven compilation after producer/source hash checks, then rebuilds the native image.
The evidence embeds the producer command and artifact hash; no compiler fix or source
relocation was needed. The supplied bundle and compiler binaries are identified by hash.


## Bounded request-line and route consumer (2026-10-08)

The [Raven consumer](request-line.rvn) reads one line with a 96-byte limit, recognizes
`GET /health HTTP/1.1` and `GET /items/<id> HTTP/1.1`, and produces an application-owned
Route union. Parsing returns Result<Route, string>; input uses Result<Option<string>,
TextReadError>. Ordinary `let ... else` and `if let` branches handle those outcomes.
Item IDs use the existing Int32 parser and reject negative values; signs and leading
zeros retain that parser's policy. Slices and public parser/union bodies stay CIL.

This is a small consumer to drive AOT toward HTTP. It is not the library HttpServer
parser or RoutePattern implementation and does not replace either. It performs no
socket I/O, header/body parsing, URL decoding or HTTP response serialization. Tabs and
NUL are explicitly rejected, unknown targets/methods/versions return sample diagnostics,
and query/fragment-bearing item IDs fail integer parsing. It is not a general request
validation or security boundary. Console's existing line-ending/EOF policy applies;
CRLF and a final unterminated line are accepted here.

The missing native dependencies were the reserved StringContainsOrdinal,
StringStartsWithOrdinal and StringEndsWithOrdinal services. `--bind-utf8-text` now binds
only their exact `(String, String) -> Boolean` InternalCall signatures, with the existing
reference-arena capability. Their linked adapters compare explicit UTF-8 lengths/bytes,
allocate no memory and publish a normalized Boolean. Empty patterns match; embedded NUL
is ordinary text; null operands fault without publishing output. Ordinary same-named
methods cannot acquire a native binding. The public String wrappers remain compiled.
No Raven emitter, Runtime Contract setting, metadata schema, public API or context ABI
changes are required.

This restores the [existing ordinal matching contract](../../ordinal-text.md), whose
.NET comparison distinguishes explicit ordinal matching from culture-sensitive prefix/
suffix defaults. Exact matching of valid Unicode text can operate directly on UTF-8;
no normalization, case folding or grapheme segmentation is intended. Prefix/suffix use
length-checked byte comparisons. Substring search uses a simple scan with worst-case
O(text bytes × pattern bytes) work, so this slice makes no throughput improvement claim.
The motivating input is bounded; general high-volume search should be measured before
choosing an optimized search implementation. Lifetime/reclamation and execution budgets
remain separate native work.

All 53 Console tests pass. The new focused test compares 60 native/interpreter results
and faults across empty, shorter/longer, case-different, repeated-prefix, Unicode,
normalization-different and NUL patterns, plus both null-operand positions. Zero-capacity
arena checks establish that matching allocates nothing, and negative checks cover
missing opt-in and ordinary same-named methods. Existing parser and reader coverage
remains part of that focused Console suite.

Run `verify_interactive.py --request-line` with the usual compiler/runtime/bundle/output
arguments. The executable retains the existing 64 KiB invocation arena. The next sample
should attempt the existing RoutePattern API against an in-memory target, to discover
remaining library/backend dependencies before adding network ownership and I/O.

[Fresh standalone evidence](request-line-validation.json) records 18 input streams with
explicit expected output and interpreter/native parity: both routes, numeric limits,
signs, empty/negative/overflow IDs, query-bearing IDs, an unknown Unicode target,
unsupported method/version, NUL/tab, root path, trailing version text, EOF, invalid UTF-8
and the byte limit. Exact broken-pipe fault diagnostics also match. The freshly compiled
Raven artifact records 217 functions and 97 types in its selection inventory; the
inspection inventory and producer/bundle hashes are recorded in the evidence. The
executable runs alone with an empty environment and only libSystem dynamically linked.

## String-array storage discovered by RoutePattern (2026-10-08)

The next consumer uses the existing library: [RoutePattern.Parse, Match and GetInt32](route-pattern.rvn)
against `/items/{id}` and `/items/42?detail=1`. It references the supplied Runtime,
Web, Networking and Data modules; importing only Runtime does not expose the Web API.
Original load-set verification runs before native selection. The first native blocker
was `specialization requires closed reference-free local value types: ArrayRef(String)`.
ArrayList<string> uses those slots for route segments, parameter names and captures.

This slice admits `arrayref<String>` locals, parameters and fields, ordinary newarr,
checked array.reserve, length, indexed loads/stores, null tests and reference identity.
The private allocator stores aligned String-owner pointers; ordinary arrays start with
null slots, reservations have separate initialization markers. Reads of unwritten slots
fault, while explicitly storing null makes a slot readable. Assignments retain array
identity; copying an element preserves its String owner. Replacing one slot does not
change another slot pointing at the previous immutable String. Null/bounds/length faults
retain source instruction/stack information, and allocation failure publishes neither
an array nor a changed cursor. The invocation arena retains all referenced storage.

[The focused metadata fixture](string-arrays.neoil) compares nine native/interpreter
executions, covering aliases, Unicode/NUL content, String-owner identity, replacement,
unwritten reads, explicit null, default null, empty arrays and range/length faults.
Canaries and a zero-capacity host exercise exhaustion without output/cursor publication.
All 54 Console and 45 value tests pass. String element borrows, value arrays and nominal
array/interface casts remain outside this profile; ArrayList<string>'s own class/interface
dispatch can use the supported raw storage without needing those array views.

[The Raven storage consumer](string-array-storage.rvn) runs standalone with aliased
String arrays and mutation. [Evidence](string-array-storage-validation.json) records
exact output/fault parity, producer/bundle hashes and executable-only deployment.
Run `verify_interpolation.py --sample string-array-storage`; the default now covers
eight consumers. No new runtime service binding or guest API was needed: the private
allocator is a backend implementation of the existing array instructions.

This extends the existing [managed array contract](../../managed-arrays.md) and
[checked reservation design](../../reserved-array-capacity.md). Like .NET reference
arrays, assignment shares identity and ordinary slots start null. neoCLR's checked
reservation additionally distinguishes an unwritten slot from a stored null. The native
experiment uses eight bytes per slot plus one marker byte per reserved slot; the
interpreter uses its own managed representation. No space or speed advantage is claimed.
The private 65536-element and invocation-byte limits still apply.

The author clarifies that the existing HTTP API drives dependency discovery, including
GC integration if needed. This arena does not reclaim overwritten owners or unreachable
arrays during an invocation, and it is not a long-running server memory policy. Collector
integration must account for native roots, array element tracing and any required store
barriers, following the [existing GC contracts](../../runtime-gc.md). No collector or
stable object/array ABI is selected by this storage slice.

The same RoutePattern consumer now passes native admission. `verify_route_pattern.py`
compiles it fresh, checks interpreter output, emits/links a standalone image and checks
native output plus exact broken-pipe fault parity. It records admission failures when
new dependencies arise. This is a small real-library execution check, not qualification
of every RoutePattern operation or the HTTP server. Expand route outcomes and repeated
request/lifetime coverage next, then follow the HTTP library's actual dependencies.

[Real-library standalone evidence](route-pattern-validation.json) records fresh compilation
and `42` output through the public Parse/Match/GetInt32 path, plus exact output-fault
parity. The executable runs alone with an empty environment and only libSystem dynamically
linked. Both library and tooling identities are hashed; the sample remains within the
existing 64 KiB host arena. Native dependencies are recorded separately from the supplied
managed load set; these are input libraries, not deployment dependencies.

## Route outcomes and sustained allocation (2026-10-08)

[The lifetime consumer](route-lifetime.rvn) reuses one RoutePattern across requests.
It checks decoded numeric capture with a query, normal no-match, Int32 overflow,
malformed escapes on an otherwise mismatching target, UTF-8 capture, one-time percent
decoding, rejection of decoded separators, and trailing-slash distinction. It also
checks missing captures and invalid/duplicate-parameter patterns. A capture made before
the loop is read afterward to exercise a reference that must survive later allocations.
Each temporary request runs in a separate CheckRequest call, so its local capture and
intermediate collections are no longer needed after return.

`verify_route_lifetime.py` runs the normal Main → Run(16) entry under interpretation
and native code, checking output and exact broken-pipe traces with the same managed
frames. It also emits Run(Int32) as a separate diagnostic export, resolving its identity
from Main's verified call target. That export varies request count without recompiling
or adding a guest measurement API. Its host records charged arena bytes after an
invocation, validates the output sentinel/fault contract and checks an allocation canary.
No guest buffers or patterns are reset between requests.

The measured invocation arena grows as follows (bytes include retained live storage,
unreachable allocations and padding; this is not live-heap size or process RSS):

| Requests in one invocation | Arena bytes charged |
| --- | ---: |
| 0 | 2,433 |
| 1 | 3,457 |
| 8 | 9,625 |
| 16 | 16,818 |
| 32 | 31,202 |
| 64 | 59,970 |
| 128 | 117,507 |

The 1 MiB buffer used for successful measurement runs is diagnostic headroom, not a
proposed server configuration or fix. With a fixed 64 KiB budget, 128 requests fault
with NativeMemoryLimitExceeded at 65,484 charged bytes, without publishing a result.
It is expected that these resource outcomes differ from a collecting interpreter;
normal routing and managed fault semantics remain the parity requirements.

For the same 16-request source entry, interpreter diagnostics report 439 tracked
allocations, peak 64 objects, eight allocation-pressure collections, and one final
collection. Pressure collections reclaim 45–54 objects apiece while the pattern and
retained capture survive. Final live count is zero. These are interpreter object
counts, not comparable byte totals: they establish actual reclamation during execution,
not an allocation-size or throughput advantage over the native representation.

This makes native reclamation the next requirement exposed by the HTTP dependency path.
Increasing the arena or resetting it per request would leave the unresolved lifetime
contract hidden; the latter would invalidate a retained pattern/capture in this model.
Follow the existing nonmoving tracing direction and validate native roots before enabling
reclamation. The next bounded backend slice should establish typed allocation descriptors
and root reporting for the currently admitted objects, arrays, Strings and managed borrows.
The current `Profile::pointer_lanes` is ABI-width information: it also marks Int64/UInt64
and native integers as 64-bit lanes. It must not be used as a GC reference map.

The existing [.NET and native GC comparison](../../native-execution-investigation.md)
and [interpreter collection contract](../../garbage-collection.md) remain the baseline:
automatic lifetime management must retain aliases and cycles without guest retain/release.
A first nonmoving collector avoids pointer relocation but still needs roots and tracing;
fragmentation, collection latency and throughput remain validation questions. This slice
implements the workload and allocation diagnostics, not a native collector or GC API.

[Recorded lifetime validation](route-lifetime-validation.json) includes input hashes,
compiler/link commands, interpreter collection events, native allocation measurements,
and exact broken-pipe fault parity. Both native artifacts run in an executable-only
directory with an empty environment and link dynamically only to libSystem.
Reproduce with a fresh output directory and a matching compiler/native bundle:

```sh
SDKROOT="$(xcrun --show-sdk-path)" python3 docs/experiments/aot-console/verify_route_lifetime.py \
  --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/neoclr-native-poc --output target/aot-route-lifetime
```

## Typed native tracing layouts (2026-10-08)

AOT `--inspect` now includes `traceLayout` for successfully admitted value profiles,
computed from the same prepared module and profile used by native lowering. Indices
refer to that selected module, not the original all-declarations inventory. The private
`neoclr-native-trace-layout-v1` report describes each function's arguments, locals and
result, plus concrete reference-object payloads. Rejected programs have a null layout;
an admitted scalar profile outside the value profile reports an explicit unavailability
reason. Inspection remains read-only and does not execute generated code.

Layouts contain sparse typed slots, independent of ABI integer width. Int64/UInt64 and
native integers have no trace slots. String/Char slots distinguish image-or-arena text;
object/interface views identify the private tagged-String convention. Erased payloads
trace only String tag 4. Inline records flatten reference offsets, reference edges stop
recursive expansion, and String arrays require tracing initialized text slots. Nominal
byte-array backing views refer to the array itself. A managed borrow requires its owner
to survive even when the pointee is an integer; pointee tracing alone is insufficient.

Byte offsets use padded eight-byte storage lanes. Object payload offsets start after
the separately reported eight-byte header. Function layouts describe possible storage,
not stack-frame locations or currently initialized/live roots. They must not be scanned
as root maps: evaluation stacks, initialization, safepoint spills, pending allocation
inputs/results, dynamic array descriptors, host/fault roots and borrow owner recovery
still need implementation. No descriptors are emitted into object files and no GC runs.
This is the first typed-layout foundation, not completion of the reclamation plan.

[RoutePattern layout evidence](route-trace-layout-validation.json) records the real
lifetime consumer's admission and descriptors. Four focused layout tests and five
inspection tests cover integer exclusion, nested offsets, owner-dependent borrows,
cycles, array aliases, rejected admission and deterministic read-only inspection.
Prior standalone routing/lifetime results remain the execution baseline because this
slice changes diagnostics only. Reusing the existing CLR/native GC comparison, typed
storage information supplies one input a precise collector needs; it does not provide
CLR-style safepoint/liveness metadata or establish a performance advantage.

## Preparing native local root storage (2026-10-08)

Ordinary value-profile function prologues now clear the local lanes identified by typed
tracing recipes. Reference slots and managed-borrow slots start at zero; erased values
clear both their discriminator and payload. Nested inline records clear only their
traceable lanes, leaving integer-only lanes alone. This subsumes the earlier ByteValues
unassigned-marker initialization. `traceLayout.functions[].localSeedLanes` exposes the
per-local, zero-based lane selection; these are eight-byte storage lanes, not source
field indices. Native service/dispatch wrappers do not allocate these managed locals.

This is internal storage preparation. Zeroed bytes do not assign a guest variable or
produce a valid default String/Char/ByteValues value. The ordinary verifier still rejects
unassigned reads and invalid erased defaults. A borrow slot starts at null; a later
non-null borrow still requires owner recovery and safe pointee tracing. Arguments,
evaluation-stack values, constructor/call-result scratch, host roots and fault messages
are not covered by this local initialization step.

The added stores make source lowering explicit about a safe initial reference state.
Until root registration makes those stores observable to a collector, optimizer retention
is not a GC guarantee. No root frame is registered, no allocation point scans these slots,
and no storage is reclaimed. Future safepoint code must publish the relevant storage and
preserve stores/spills before collection, then unregister roots on success and fault paths.

Compared with the existing CLR/root-map baseline, clearing reference slots is only one
part of making locals safe to scan: CLR-style liveness/safepoint information remains open.
Seeding adds prologue stores and may retain assigned but dead references once scanning
exists; it is a correctness foundation, not an allocation or throughput optimization.

[Local-storage validation](route-root-seed-validation.json) records five layout tests,
54 Console tests, five inspection tests and 45 value tests, including rejection of
unassigned reads. The freshly compiled standalone routing consumer retains exact output
and broken-pipe fault parity, retained captures, libSystem-only dependencies and unchanged
arena measurements (117,507 bytes at 128 requests; clean failure with a 64 KiB budget).
No collector or reduced-memory claim follows from these initialization checks.

## Pre-operation stack root plans (2026-10-08)

`traceLayout.preOperationPlans` now derives plans from the same checked CFG stack shapes
used by value lowering. At each reachable call, constructor, array allocation/reservation
or String literal materialization, it records stack values, flattened lane bases, typed
root layouts and required spill lanes. All calls and text materializations are conservative
candidates; a particular call may not allocate and a literal may remain in image data.
Unreachable IL contributes no point, but remains subject to normal backend admission.

The plan separates the stack prefix retained by the caller from operands about to be
consumed by the operation. Both can hold the only reference to an object. Erased values
need their discriminator and payload spilled together; ordinary wide integers do not
become roots. Slot/tag offsets inside each layout are relative to its `laneBase`, not
the complete evaluation stack. `requiredSpillLanes` uses absolute flattened stack lanes.
These plans describe pre-lowering operand representations, before any call-boundary
String/Object tagging or receiver conversion.

A constructor consumes explicit arguments before its new receiver exists. Its receiver
layout is reported separately for activation after allocation/value initialization and
before the constructor call. The eventual result is likewise separate and inactive before
the operation; it becomes valid only on success. No-result calls report no result layout.
A pending output borrow identifies an address/owner obligation; it does not prove the
pointee initialized or permit tracing unassigned caller storage.

Function indices join the existing argument/local layouts. Native adapter and dispatch
bodies receive an explicit `native-body-requires-separate-plan` coverage marker instead
of plans for their synthetic IL. Those bodies can allocate or forward calls and remain
mandatory work. Wrapper classification follows the current backend substitutions and
must be extended with new native bindings. Local liveness, spills, root-frame publication
and cleanup, adapter internals, pending result storage, borrow owner/initialization
tracking and host/fault roots are still incomplete. This is analysis, not emitted native
safepoints or permission to run a collector.

[Routing root-plan evidence](route-root-points-validation.json) records actual consumer
admission, plan counts, selected function excerpts and native-body gaps. Seven unit tests
and six inspection tests pass, including branch joins, dead IL, retained stack-only text,
erased payloads, integer exclusion, constructor staging, array lengths and output borrows.
Code generation and runtime behavior are unchanged; the preceding standalone validation
remains the execution baseline.

This continues the existing CLR/root-map comparison: metadata describing locals alone
cannot cover evaluation-stack temporaries across allocating calls. Conservatively keeping
the full traceable stack simplifies the first implementation but can retain dead values;
it is not CLR register-liveness metadata or a performance improvement. Native registration
must make the planned spills observable and validate nested calls and fault cleanup before
reclamation is enabled.

## Executable stack-root probes (2026-10-08)

`--probe-stack-roots` now emits the planned stack spills before each covered operation
and calls the private probe callback (now `neoclr_probe_stack_roots_v2`). It requires `--fault-details` or an existing
context-enabled binding, preserving that profile's entry ABI. It is off by default;
ordinary emission adds no callback import or probe data. Inspection uses the same option
and reports `capabilities.stackRootProbes`.

The v2 callback receives a diagnostic frame pointer, IL instruction, a read-only array of
64-bit lanes, its count, and an image-owned JSON plan with byte length and a trailing NUL.
The [private header](root-probe.h) states the contract. Each point overwrites a per-function
scratch snapshot: required pointer/discriminator lanes come from the actual current SSA
values, 32-bit tags are extended, and numeric holes are zero. The buffer is included in
the existing 64 KiB frame-storage budget. Published callback arguments make the spills
observable in executable code, unlike unused stores intended for future registration.

Snapshots may be observed through the active diagnostic chain during callbacks. The hook
must not allocate, reenter, retain frames beyond their lifetime, mutate roots or trigger
collection. Traceable argument/local addresses are now exposed as described below; no
future result/constructor receiver is active in this pre-operation snapshot. Native adapters remain uncovered. Complete root coverage, host roots, owner recovery and result activation still precede GC.
The callback is diagnostic only; it is not a stable ABI or the proposed native metadata
interop interface. The default runtime and managed contracts are unchanged.

The [diagnostic adapter](root-probe.c) reads the initialized snapshots without following
pointers and counts calls per thread. Its counter accumulates across entry invocations;
the diagnostic chain is thread-local rather than a process-global collector registry. The route harness reports counts only in its
measurement mode, leaving normal output/fault comparisons unchanged. Reproduce the
existing lifetime driver with `--probe-stack-roots`; it links this adapter explicitly.

The ARM64 native probe test validates exact retained String and erased String values,
zero numeric holes, nested calls, a divide fault with unpublished result, and successful
reentry. It also checks the import is absent without the option. Six fault-detail tests
and six inspection tests pass alongside the two probe tests. The real routing execution
is recorded in [probe validation](route-root-probe-validation.json): 35,402 callbacks at
128 requests, unchanged 117,507-byte arena usage, exact broken-pipe fault parity and
libSystem-only dynamic dependencies. The fixed 64 KiB run still fails cleanly.

Compared with the existing CLR root-map baseline, this demonstrates stack materialization
at selected boundaries but still lacks complete root lifetime registration and safepoint
coverage. The opt-in path adds stores, static JSON and synchronous calls; it is a
correctness instrument with overhead, not a performance optimization or collector.

## Diagnostic frame lifetimes (2026-10-08)

The initial lifetime slice linked a 48-byte, stack-owned frame on entry to every ordinary managed
body, including bodies without a pre-operation point. Each frame records its host context,
function index, prior frame and latest initialized stack snapshot. A callee's callback can
observe its suspended callers' snapshots. Native adapter/dispatch wrappers remain omitted;
this is still an incomplete root set and must not be used to collect.

After lowering, a pass inserts `neoclr_probe_leave_v1` before every return in the completed
function IR. This covers normal returns, explicit faults, arithmetic/null checks and
propagated call failures without relying on each emitter branch to remember cleanup.
The frame storage counts toward the existing 64 KiB frame budget. The host entry ABI is
unchanged. Default, uninstrumented builds add no frame storage or callback dependency.

The private callback ABI changes from `neoclr_probe_stack_roots_v1` to v2, taking a frame
pointer instead of a function index; function/context identities are in the frame.
The initial frame contract added enter/leave hooks; the storage extension below uses
`neoclr_probe_enter_v2` with the existing `neoclr_probe_leave_v1`.
Recompile/relink diagnostic hosts and images together against [root-probe.h](root-probe.h).
This does not change guest APIs or select a stable external GC/hosting ABI.

The supplied adapter uses a thread-local linked chain and validates LIFO removal. It
clears removed frames, exposes head/depth for host assertions, and reads ancestor
snapshots without following managed pointers. A frame can describe a nested host context;
context identity does not turn the chain into isolated collector root sets. Asynchronous
collection, reentry from hooks, native unwinding and longjmp remain unsupported.

The ARM64 lifecycle test enters three functions, validates the caller-only retained String
from the deepest function, returns normally, propagates a deepest-frame divide fault,
and reenters successfully. All nine entries have matching leaves, peak depth is three,
and the chain is empty after each host return. Fault trace/result behavior is preserved.
The real route host now requires an empty chain on every normal and fault return;
[frame validation](route-probe-frame-validation.json) records those execution checks.
Fourteen focused probe, inspection and fault-detail tests pass.

This follows the existing CLR/shadow-stack comparison: lexical frame lifetime and cleanup
are necessary but insufficient for a precise collector. Arguments/locals, initialized
borrow pointees, constructor/result activation, native adapters and host/fault roots still
need coverage. Diagnostic ancestor scanning adds overhead; no speed or memory claim is made.

## Argument and local storage in diagnostic frames (2026-10-08)

Ordinary instrumented functions now publish addresses of their traceable argument and
local lanes alongside the evaluation-stack snapshot. Each table entry contains a slot
address, read width and flags; an image-owned JSON descriptor identifies argument/local
index, padded lane and typed layout. Numeric-only slots are excluded. A borrow entry
publishes the address of the slot holding the borrow, never the borrowed pointee itself.

Arguments are copied before publication. Local reference lanes and erased tags are seeded
before publication, so an unassigned String or erased local is observable as internal zero
storage without becoming guest-readable. Guest definite assignment remains unchanged.
The table points to live frame storage: a subsequent store is visible to nested callbacks
without refreshing a snapshot. Discriminators are read as four bytes and reference/payload
lanes as eight; argument-tag padding need not be initialized and must not be read.

The private frame grows from 48 to 72 bytes, plus 16 bytes per selected storage lane.
Both allocations count toward the 64 KiB frame budget. `neoclr_probe_enter_v2` receives
the table and JSON descriptor; stack observation remains `neoclr_probe_stack_roots_v2`
and removal remains `neoclr_probe_leave_v1`. Recompile/relink instrumented images and
hosts together against the matching header. Guest metadata and entry ABI v3/v4 are unchanged.
Default emission still publishes no diagnostic frame or table.

The adapter reads only the specified bytes at each slot address. Flag 1 marks a
discriminator; flag 2 marks a borrowed-address slot. Neither flags nor JSON authorize
following borrowed pointees, treating numeric erased payloads as references or running a
collector. Reference-valued locals remain conservatively retained after assignment;
precise liveness, owner recovery and output-pointee initialization remain unresolved.
Native adapters, pending constructor/results and host/fault roots still need coverage.

Validation extends the three-level ARM64 lifecycle test to inspect erased arguments,
zeroed unassigned local roots, and a later local String write through the suspended parent
frame. A separate adapter test passes an inaccessible borrowed pointee and succeeds by
reading only the slot. Fifteen focused probe, inspection and fault-detail tests pass.
[Routing storage evidence](route-root-storage-validation.json) records standalone
execution, ancestor storage reads, output/fault parity and empty chains at host return.

This follows the existing CLR/root-map comparison: typed slot locations and widths are
necessary inputs, not complete safepoint metadata. Address-table storage and diagnostic
reads add overhead. No collector, liveness precision or performance advantage is claimed.

## Constructor and call-result activation (2026-10-08)

Instrumented ordinary calls now publish transient storage in two phases. Phase 1 exposes
constructor storage only after reference allocation succeeds or inline storage is zeroed,
before calling the constructor. Its typed table points to live slots, so nested callbacks
can observe constructor writes through the suspended caller. Phase 2 exposes the result
only after a successful call/constructor status check. No-result ordinary calls have no
phase-2 event; failed calls do not publish their uninitialized result buffer.

`neoclr_probe_transient_v1` receives the frame, phase, bounded-width lane-address table and
image-owned typed layout. The next pre-operation snapshot retires the old transient view,
and leaving the frame clears it. Prior results remain in the diagnostic view until then,
so this is conservative retention rather than exact liveness. The adapter observes
transient slot bytes in ancestor frames without following pointers.

The private frame grows from 72 to 104 bytes and uses `neoclr_probe_enter_v3`; the transient
table also counts toward the existing 64 KiB frame budget. The stack observation and leave
hooks retain their current versions. Recompile/relink diagnostic images and hosts together
with the current header. Ordinary uninstrumented output and entry ABI v3/v4 are unchanged.
Earlier sections describe the incremental contracts; this is the current frame layout.

The ARM64 constructor test observes zeroed String storage before entry, the constructor's
later write through its caller's live table, and one successful result publication across
a successful and a faulting construction. The nested-call test observes four successful
erased-result publications across two successful invocations and none on its faulting
invocation. Sixteen focused tests pass. [Routing validation](route-transient-root-validation.json)
records real-library execution, frame cleanup, output/fault parity and unchanged arena use.

This remains diagnostic observation, not a complete collector root protocol. Publication
occurs after the callee returns: return-value protection during the callee-to-caller handoff,
transient roots inside native adapters, borrowed ownership/initialization and host/fault
roots still require explicit handling. Array/text service results are observed at later
stack points rather than by this call/constructor phase hook. Hooks cannot allocate,
collect or reenter; they must not use an incomplete root set to reclaim memory.
Compared with the existing CLR/root-map baseline, explicit activation avoids interpreting
uninitialized scratch as roots but does not yet close every safepoint gap. Tables and
callbacks add diagnostic storage/work, with no performance improvement claimed.

## Caller-owned result handoff (2026-10-08)

Ordinary instrumented calls with a result now add phase 3 before invoking the callee:
clear the traceable lanes of the caller-owned result buffer, then register their live
addresses in the caller's diagnostic frame. Integer-only lanes remain unobserved. Erased
scratch clears both tag and payload. This is internal pending storage, not a valid guest
result or permission to read a variable before assignment.

The callee writes into that same buffer on success. Its removal callback can still
observe the result through the parent frame, and the caller then transitions to the
existing phase 2 after checking the returned status. On failure, phase 2 is never
activated; normal fault cleanup removes the pending view with its frame. Constructors
retain their separate initialized-receiver protocol and no-result calls register no
result buffer. The next pre-operation point retires previous transient views.

The callback becomes `neoclr_probe_transient_v2`, accepting phase 3 as well as constructor
phase 1 and successful-result phase 2. Frame size remains 104 bytes and enter/leave hooks
retain their versions. Relink instrumented hosts/images together against the matching
header. Ordinary emission and entry ABI v3/v4 remain unchanged.

The three-level ARM64 test observes six cleared pending buffers, four successful results
at callee removal, and four later success-phase publications across success/fault/reentry.
The faulting invocation has neither a populated pending buffer nor a success publication;
all frames are removed. Sixteen focused tests pass. [Handoff validation](route-result-handoff-validation.json)
records the real routing consumer, output/fault parity and frame cleanup.

This closes the diagnostic handoff for ordinary managed calls and the caller-owned output
slot of bound services. It does not cover references held internally by native adapters,
borrowed pointee ownership/initialization, or host/fault roots. Hooks still cannot collect,
allocate or reenter. The existing CLR comparison applies: caller-owned pending storage
simplifies lifetime continuity but adds initialization and retains values conservatively;
it is not precise liveness metadata or a performance claim. The current exported root
returns only Int32; a future reference-returning host entry needs its own ownership contract.


### Native wrapper argument frames (2026-10-08)

With `--probe-stack-roots`, native service and interface-dispatch wrappers now join the
same diagnostic frame chain as ordinary functions. Before wrapper logic executes, their
arguments are copied into typed stack storage and the traceable lane addresses are
published. The wrapped IL body's locals are excluded: replacement bodies never initialize
that storage. Numeric-only arguments produce no root entries. The existing 64 KiB
per-function frame-storage limit includes the diagnostic frame, table and argument copies.

Every generated return unlinks the wrapper frame, including null checks before service
invocation, failed services, dispatch failures and successful forwarding. These are
private diagnostic frames, separate from the managed fault stack: synthetic wrappers
still do not appear in rendered guest faults. Caller-owned pending results continue to
provide the output-slot view established by the preceding slice.

Inspection keeps native bodies marked as requiring separate internal plans, with additive
`diagnosticFrame: typed-arguments-only` and `serviceInternalRoots: uncovered` fields.
The 104-byte frame and callback versions remain unchanged. Default emission still has
no probe calls. Diagnostic consumers must tolerate wrapper frames with an argument table
and no IL snapshot; the callback count continues to count IL snapshots only.

An ARM64 contract test observes the actual String argument from inside the Console output
service, with the wrapper linked above its caller and the same context identity. It checks
success, service failure, null before service invocation, frame removal and subsequent
reentry. The interface-dispatch consumer also runs with probes through its six normal/fault
modes and retains exact interpreter fault parity. Together with the existing 16 focused
root/inspection/fault checks, 18 tests pass. [Routing evidence](route-wrapper-root-validation.json)
records standalone output/fault parity and empty frame chains after host return.

As with the existing CLR root-map comparison, stack copies make boundary references
observable but add diagnostic work and conservative retention. Service-internal temporary
references, borrowed pointee ownership/initialization and host/fault roots remain open;
this does not authorize collection, mutation, allocation or reentry from probe hooks.
No stable hosting ABI, native GC or performance improvement is claimed.


### Fault-context root slots (2026-10-08)

The linked diagnostic adapter now exposes `neoclr_probe_fault_roots_v1(context, output,
capacity)`, a private read-only view of ABI v3/v4 fault storage. For an active fault it
returns the address of the message slot followed by initialized frame-name slots, each
an eight-byte text-reference slot. At most 65 entries are returned. Code-zero contexts
return zero without reading stale message/frame storage. Invalid frame counts, null
contexts and insufficient output capacity return -1 without partially writing a table.
The helper never dereferences text pointers; the context and writable output table must
be valid and nonoverlapping, and callers must not query while the context is changing.

The adapter observes these slots before removing a diagnostic frame and while observing
ancestor snapshots. The host can query the same context after the chain is empty. The
routing measurement host does so on success and fault returns. This is explicit enumeration,
not a global registry: the caller supplies the context, and the helper retains nothing.

Slot addresses remain valid only while their owning context exists. An active user message
may point into its context's arena; function names and standardized runtime messages are
image data. Entry reset clears the header and retires all prior fault roots, even though
unused frame-name storage can remain populated. Message bytes do not survive arena reset
or release under the existing v4 contract. Previously returned tables must not be treated
as active roots after reset. There is no new exported-entry or frame/callback ABI version;
the helper is an additive private symbol in the linked diagnostic adapter.

Seven focused tests pass: four root-probe checks (including bounded fault-slot enumeration,
short tables, invalid counts, stale code-zero storage and inaccessible text pointees), the
native-wrapper and interface-dispatch regressions, and a new dynamic user-fault consumer.
That consumer verifies two independent arena-backed messages after unwinding, successful
reentry retiring one fault while the other remains intact, and exact interpreter rendering
parity. [Routing evidence](route-fault-root-validation.json) records the real RoutePattern
workload with host-return enumeration and unchanged fault/output behavior.

The existing CLR/root-map comparison applies: this makes an owned fault context observable
with bounded stack work, but supplies neither GC handles nor runtime-owned exception objects.
General host root registration, borrowed ownership/initialization and native service
internals still need contracts before a collector can use the roots. Collection remains
disabled, and this slice does not change fault codes, standardized messages or exit status.


## HTTP socket-handle prerequisite — 2026-10-08

Native erased pack/test/unpack now admits Int64 and UInt64, preserving all 64 payload
bits and signed/unsigned nominal tags through locals and generic call/return paths.
Wrong-tag unpack returns RuntimeError without publishing the caller result. Closed
method/type specialization admits these scalar arguments, and 64-bit equality compares
all bits. Private tags 5/6 extend the experimental representation; String remains tag 4
and the two-lane erased layout/root rule is unchanged. This is not a stable public ABI.
The interpreter already supports these types; this aligns native observable behavior
with it and ordinary .NET signed/unsigned boxed-type distinctions, without adopting
.NET's object layout.

The Raven [socket-handle probe](../../../benchmarks/native-web/SocketHandle.rvn) runs
in both modes. The actual Socket.Listen/GetLocalPort/Close consumer now reaches native
service admission instead of rejecting UnpackValue<long>; SocketClose is its first
unbound contract. The full HTTP app additionally needs function-valued generic support.
[Focused validation](../../../benchmarks/native-web/handle-validation.json) includes
boundary values, high-bit inequality, mismatch faults and existing native GC tests.
No Raven compiler/CLI bridge or public library signature changes are involved.


## Native listener lifecycle — 2026-10-08

`--bind-socket-listener` now compiles the ordinary Raven Socket.Listen/GetLocalPort/Close
path with statically linked POSIX adapters and an explicit host-owned cleanup scope.
Typed errors and fault output match interpreter execution; outstanding listeners close
even when guest code faults. See the [contract, limits and validation](socket-listener.md).
Async accept/read/write and the full HTTP app remain work in progress.

## Stored callbacks on the HTTP path (2026-10-08)

The reference profile now executes bounded static/bound Function callbacks with managed
receiver retention and exact interpreter fault parity. See [the callback experiment](callbacks.md)
and its Raven consumers. Function arrays and asynchronous host completion remain open;
this does not yet make the HTTP app executable natively.
