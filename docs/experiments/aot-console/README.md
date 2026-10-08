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
