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
