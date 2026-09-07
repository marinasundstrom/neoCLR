# Work log

## 2026-09-06 — Initial interpreter and assembler proof of concept

Started from an empty repository. Added a standalone Rust crate with a loader,
typed metadata/IR, line-oriented assembler, iterative interpreter, and command-line
`assemble`, `check`, and `run` operations. JSON serialization is a temporary means
of testing the interpreter and assembler independently; final CLI-based binary
encoding is a separate milestone.

Implemented owned records, explicit shared heap references, first-class Void,
constructed Option/Result types, free functions, locals, arithmetic, branches,
terminal Faults, and three bootstrap library functions. The interpreter checks
executed operations and bounds execution resources. No guest null or exception
instructions exist.

Added HelloWorld, a feature tour containing all 35 implemented instruction forms,
and an intentionally failing Fault sample. The feature tour checks copy isolation,
heap aliasing, unit-valued unions, recoverable numeric errors, and control flow.
CLI integration tests cover assembly to disk, metadata checking, execution, output,
refusal to overwrite existing output, and terminal Fault exit status.

Recorded the type/storage philosophy, prototype contracts, opcode reference,
array/pointer proposals, migration direction, and next milestones. The README
contains build/run instructions and explicit implementation boundaries. Clarified
that assembly source syntax can change independently while metadata and CIL should
remain CLI-based wherever the intended semantics permit. Ordinary arithmetic wraps;
checked arithmetic uses the familiar `.ovf` forms and terminal Faults.

Validation on macOS ARM64 with Rust 1.95.0:

- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo test --locked`: 19 integration tests passed; several are table-driven.
- `cargo test --locked --release --test runtime`: all 12 runtime tests passed.
- HelloWorld prints `Hello, world!` and returns Void.
- Feature tour prints `All feature checks passed.` and returns `Ok(Void)`.
- JSON round-trip execution agrees with execution from assembled source.

Added Linux/macOS/Windows CI. Only local macOS execution has been observed so far;
the CI matrix has not been run in this session.

Open work includes a full verifier, actual CLI binary metadata/CIL emission,
owned arrays and checked borrows, general generics/unions, interfaces, library
modules, heap reclamation, .NET migration tooling, and future async/native interop.
Physical native-stack allocation has not been demonstrated by this interpreter.

## 2026-09-06 — Explicit call signatures and overloads

Calls now use `Name(Type, ...)`, including `Name()` for parameterless functions.
HelloWorld uses `call System.Console.WriteLine(string)`. Primitive aliases are
normalized by the assembler; structured metadata retains canonical types. Nested
constructed types and their internal commas are accepted in parameter lists.

Function identity is name plus ordered parameter types, enabling overloads by
arity/type/order. Return types alone cannot distinguish overloads. The loader
rejects duplicate signatures and unresolved call targets; execution checks actual
arguments against the explicitly selected target. `.entry Main` selects `Main()`
regardless of declaration order. Exact intrinsic signatures are reserved while
other overloads of their names are allowed.

Added an Int32 overload of Console.WriteLine and an overload demonstration program.
Updated existing samples/tests to specify call signatures. Prototype metadata
format is now version 2, with call operands carrying name and parameter type array;
format 1 modules must be reassembled. The output remains temporary JSON IR.

Validation: all 28 integration tests pass, including nine new overload tests for
parsing, dispatch, nested types, metadata round trips, duplicate/unknown targets,
entry selection, and intrinsic overloads. Formatting and Clippy checks pass.

## 2026-09-06 — Inline declarations and platform-written System library

Added `.function Describe(int32) -> string` declarations using the same parameter
parser as call operands. Migrated samples and runtime library source to inline
signatures. Legacy headers with `.param` remain accepted, but cannot be mixed with
an inline parameter list.

Moved public System APIs out of interpreter intrinsic dispatch into
`runtime/System.neoil`. The library contains five ordinary functions, including
Result-based Divide and Abs implementations. Added signed `div` with truncation
toward zero and terminal Faults for invalid arithmetic; all 36 instruction forms
are represented in samples. Three host primitives remain for I/O and temporary
Int32 parsing/formatting. These host boundaries are documented.

Added library modules without entry points, bootstrap linking, cached default
System compilation, and an explicit compiled-library CLI/embedding execution path.
Tests prove a supplied serialized library body executes and that library calls use
guest frames/budgets. General dependency resolution and final CLI binary emission
remain unimplemented.

Recorded .NET/CLR semantics as the default outside intentional deviations. Added an
assembler design document requiring eventual full platform expressiveness, with
.NET assembly language as the capability baseline. The current parser and metadata
subset, including parameter-only overload keys, are not a permanent ceiling.

Validation: `cargo test --locked` passes all 36 integration tests; `cargo fmt --check`,
`cargo clippy --locked --all-targets -- -D warnings`, and `git diff --check` pass.

## 2026-09-06 — Native implementation metadata and API familiarity

Added `.methodimpl InternalCall` on bodyless runtime function declarations. It
emits `impl_flags: 4096`, using the CLR InternalCall value. Ordinary functions use
zero. This implements the metadata target for a future familiar MethodImpl attribute
form; general custom-attribute syntax remains pending.

Moved host binding into an explicit typed registry. Calls resolve a declaration
first, then its implementation flag selects IL or native execution. The binder
checks name, ordered parameters, and return type. Unknown bindings/flags, native
bodies/locals, and missing declarations fail validation. Tests confirm a familiar
native helper name without the flag runs its IL body instead of triggering host
code. The System library now declares its three host dependencies explicitly.

Added an API policy: retain familiar .NET namespaces, type/member names, overloads,
and behavior except where intentional platform semantics require change. Marked
Int32.Divide as an experimental extension and runtime helper names as implementation
details. API familiarity does not require identical implementation, and prototype
limitations are not permanent API design decisions.

Validation: all 41 integration tests pass, including five new metadata/binding
checks. Clippy, formatting, and diff whitespace checks pass.

## 2026-09-06 — Canonical types and declared methods

Added canonical System primitive definitions and alias resolution, explicit type
representation metadata, method owners, and static/instance call form. Methods can
be declared inside types; free functions remain available. The System APIs now have
declaring types, and Int32.ToString executes as an instance method. Added the Point
method sample and metadata/receiver tests. Format 3 requires reassembling earlier
application and System artifacts.

Instance methods currently consume read-only value snapshots: receiver at argument
zero, declared parameters afterward. Mutable/by-reference receivers are unimplemented;
no Rust borrowing semantics or hidden heap allocation are implied. Method calls check
receiver types and distinguish static/instance overloads explicitly.

Added foundational Ptr<T>/T* signatures, distinct from Ref<T>. These do not yet expose
raw address values or executable pointer operations. Recorded that the VM memory model
is independent of Rust, and that reference-counted Ref is the intended first explicit
ownership policy. Current Ref storage remains an arena and general generic definitions
are still pending. Added an allocation-encoding proposal separating storage,
construction, and optional ownership abstractions.

Validation: all 49 integration tests pass, including eight new type/method/pointer
signature tests. Clippy and formatting pass. The types sample prints 42, 0, 42 and
returns Void without explicit heap allocations.

Allocator follow-up: updated the allocation proposal to select an allocator rather
than permanently encode only stack versus heap. Allocator selection is distinct
from construction and lifetime ownership. Documented moving-GC requirements for
roots/handles, allocator lifetime, and Ref-counted ownership composition. No new
allocation opcode or managed-memory behavior is claimed by this documentation.

## 2026-09-06 — Defer ownership management; prioritize heap and pointers

Recorded the explicit-memory philosophy: languages may hide ownership wrappers or
insert operations, but metadata/IL must make applicable lifetime behavior explicit.
Plain T and Ptr<T> do not imply counted ownership. Ref<T> is a deferred explicit
counted wrapper; generic support and type-aware copy/move/destruction would be
needed. The split between library implementation and optional VM counter/retain/
release support remains open. Rust Clone/Drop are not the guest specification.

Clarified that the environment chooses allocation implementation and optional
collection services while preserving declared contracts. Requiring a concrete
allocator operand on every allocation was an earlier proposal, not a settled rule.

Updated README, roadmap, and design notes to prioritize heap allocation and pointer
operations. Reference counting, GC, automatic lifetime management, and allocator
policy integration are deferred. No runtime behavior changed in this documentation
update; existing type-system work is preserved. Validation: diff whitespace and
local Markdown links checked; no additional runtime tests needed for these edits.

## 2026-09-06 — Optional parameter and local names

Chose consistent `name: Type` syntax for inline parameters, legacy .param declarations,
and locals (`.local point: Point`). Type-only declarations and numeric instruction
operands remain valid. ldarg/ldloc/stloc also accept names, resolved to indices by
the assembler. Instance methods expose `this` at argument zero and offset declared
parameter names appropriately. Call references remain type-only.

Added optional name arrays alongside parameter/local type signatures. Names survive
metadata serialization without affecting overload identity or execution. The loader
checks array alignment, valid names, duplicate names, and the reserved instance
receiver alias. Earlier format-3 artifacts with omitted names still load.

Named runtime library parameters and added a sample using a named Point local.
Validation: all 55 integration tests pass, including six new name-resolution and
metadata tests. Clippy, formatting, and diff whitespace checks pass.


## 2026-09-06 — Native heap allocation and pointers

Added real native allocations with explicit free, native-address pointer values,
native-sized pointer storage, sequential record layout, and twelve instructions:
sizeof/alignof, heap.alloc/free, ptr.null/cast/add, ldflda, ldobj/stobj, and
ldind.i4/stind.i4. Allocation consumes an element count; pointer offsets use bytes.
Construction remains a separate value operation. Copies of pointer-bearing records
copy addresses without imposing ownership or automatic retain/release behavior.

Interpreter side tables check allocation identity, initialization, bounds, alignment,
and invalid release. Pointer stores retain diagnostic tracking alongside real address
bytes; overlapping nonpointer stores discard it. Native interop, external addresses,
native integers, and stack address operations remain pending. The current checks are
prototype restrictions, not a universal memory-management model. Ref's existing
execution-owned arena is unchanged; RC, GC, and allocator policy remain deferred.

Recorded the platform principle: types provide data and behavior, while allocation
and lifetime remain separate choices. Preserve familiar CLR behavior except for
intentional departures, and distinguish incomplete implementation from new semantics.
Added a pointer sample and the detailed heap/pointer contract; updated README,
assembly reference, semantics, memory/type notes, and roadmap.

Validation: all 72 integration tests pass, including 17 pointer tests covering native
address dereference from the host, pointer width/storage/casts, recursive pointer
fields, aggregate layout/copying, lifetime and invalid-access diagnostics, quota
reclamation, and metadata validation. Clippy and formatting pass on macOS ARM64.
The pointer sample prints 42 and returns Ok(Void), with all its allocations freed.
Cross-platform execution remains subject to the existing CI matrix.

## 2026-09-06 — Native integers and address conversions

Added canonical System.IntPtr/System.UIntPtr primitive definitions with nint/nuint
aliases, native layouts, and native storage loads/stores. Extended integer arithmetic
to native width, added unsigned checked arithmetic, div.un and clt.un, and kept
signedness controlled by opcodes. Added conv.i/conv.u/conv.i4 and ptr.fromint T.
Allocation counts accept native integers and ptr.add accepts signed native offsets.

Integer conversions discard diagnostic pointer identity. Reconstruction resolves
current live native storage at an address; unknown addresses can be represented but
cannot yet be accessed. Documented allocation reuse implications and the remaining
mixed-type evaluation-stack limitations. Native interop, other scalar widths, and
full IntPtr/UIntPtr library members remain future work.

Recorded the direction of modeling more contracts through explicit types, including
Ref<T> as a possible counted-ownership abstraction. This does not introduce reference
counting or couple plain values and pointer allocations to an ownership policy.

Validation: all 80 integration tests pass, including eight new native-integer tests.
Clippy, formatting, and diff whitespace checks pass. The native-integer sample
prints 42, returns Void, and frees its allocation. Local validation is macOS ARM64;
the tests derive expected native widths from the host for the existing CI matrix.


## 2026-09-06 — Fixed-width integer fundamentals

Implemented SByte/Byte, Int16/UInt16, Char, UInt32, Int64/UInt64 as canonical System
primitives with native layouts. Added ldc.i8, fixed-width integer conversions, and
byte/short/32-bit unsigned/64-bit/native indirect operations. Int64 participates in
wrapping and checked arithmetic, signed/unsigned division, and comparisons.

Separated declared integer storage from evaluation-stack categories: narrow integers
load as Int32; UInt32 and UInt64 preserve bits in Int32 and Int64. Storage boundaries
truncate to the declared width. Applied this to locals, parameters, returns, fields,
and native memory. Kept the remaining native/Boolean normalization and generic
constructor limitations explicit rather than treating them as platform decisions.

Added the integer sample and storage/opcode documentation. Recorded valid UTF-8 as
a proposed String direction, with Rune/scalar versus Char/UTF-16 code-unit distinctions;
String.Length/indexing and migration contracts remain undecided. This proposal does
not change the current String API or introduce Unicode scalar validation for Char.

Validation: all 86 integration tests pass, including six new integer suites covering
aliases/layout, metadata round-trip, exact 64-bit values, narrow storage boundaries,
sign/zero extension, overflow and invalid memory access. Clippy and formatting pass
on macOS ARM64. The sample prints 255 and -1, checks 64-bit storage, and frees its
allocations. Cross-platform execution remains covered by the existing CI matrix.


## 2026-09-06 — Bitwise, shift, remainder, and negation instructions

Added and/or/xor/not, neg, shl/shr/shr.un, and rem/rem.un across Int32, Int64, and
native integer stack categories. Shift counts accept Int32 or native integers.
Signedness remains opcode-controlled, including when the storage signature is unsigned.
No ownership or allocation behavior changes.

Documented wrapping negation, divisor-zero Faults, and deterministic prototype
choices at CLI platform-dependent boundaries: masked shift counts and a Fault for
signed minimum rem -1. These choices are explicit rather than accidental Rust
semantics. Added a packed-field sample and opcode coverage for all ten instructions.

Validation: all 92 integration tests pass, including six new bit-operation tests
covering every integer stack category, high bits, negative and oversized counts,
sign behavior, minimum values, invalid operands, and instruction-located Faults.
Clippy and formatting pass on macOS ARM64. The bits sample prints 18, -1, 5, -4
and returns Void. Existing integer, pointer, library, and CLI tests remain passing.


## 2026-09-06 — Floating-point fundamentals

Added canonical Single/Double types, native layouts, ldc.r4/r8 constants, floating
conversions, ckfinite, and floating indirect loads/stores. Arithmetic, negation,
remainder, and comparisons now accept the internal floating category. Added cgt/cgt.un
for both integer and floating categories. Binary64 is the explicit F representation;
Single stores and conv.r4 round to binary32. Integer-to-Single conversion avoids
an intermediate double-rounding error.

Constant metadata stores IEEE bits so JSON preserves signed zero and non-finite
constants. NaN comparisons, signed zero, infinity, unchecked conversion saturation,
and pending API/verifier boundaries are documented. Invalid integer-only operations
on floating operands still Fault. No ownership or allocation policy changed.

Validation: all 100 integration tests pass, including eight floating-point tests
covering type identity, metadata bit round-trip, native storage, Single precision
across locals/calls/fields, a double-rounding regression, non-finite arithmetic,
comparison behavior, conversions, and instruction-located Faults. Clippy and formatting
pass on macOS ARM64. The sample prints 42 and frees its allocations. Existing integer,
pointer, library, and CLI suites remain passing; other hosts still require CI execution.


## 2026-09-06 — Checked numeric conversions

Implemented all 20 conv.ovf integer/native destination forms, including unsigned-source
.un variants. Integer range checks preserve full precision. Floating inputs truncate
before bounds checking; non-finite and out-of-range results Fault with instruction
locations. Power-of-two exclusive upper bounds avoid the rounded Int64/UInt64 maximum
pitfall. Source interpretation remains explicit and independent of storage names.

Added an executable sample, opcode reference entries, and checked-conversion contracts.
These operations use existing stack categories and do not change unchecked conversion,
allocation, or ownership behavior. Recoverable conversions still require a validating
Result-returning library surface rather than catching a Fault.

Validation: all 106 integration tests pass, including six new checked-conversion tests
covering every opcode, exact integer boundaries, unsigned source interpretation, native
widths, floating-point boundary rounding, non-finite values, invalid operands, metadata
round-trip, and sample execution. Clippy and formatting pass on macOS ARM64. The sample
prints 42 and 255 and returns Void. Other hosts remain subject to the existing CI matrix.


## 2026-09-06 — First P/Invoke slice

Added optional native-import metadata and `.pinvoke "library" "entry" cdecl` for
free functions/static methods. Native declarations cannot contain IL/locals, be
entry points, or combine with InternalCall. Metadata validation does not load libraries.

Added libloading/libffi dispatch for numeric scalars, pointers, and void returns.
Calls preserve declared ABI storage widths and normalize results back to the guest
stack. Libraries load lazily and are retained with Execution. Tracked stale pointers
Fault before dispatch; returned foreign addresses can be forwarded without claiming
guest ownership. String/Boolean/Char marshalling, struct-by-value, other conventions,
and direct interpreter access to foreign memory remain pending.

Safe Rust embedding APIs reject executed imports; a separate unsafe run_with_native
entry point makes the ABI/trust contract explicit. CLI run enables native execution
for its selected program. Native memory ownership, side effects, and unsupported
initialization/provenance tracking are documented rather than inferred.

Added a C-ABI native fixture, portable build helper, guest allocation/mutation/free
sample, and CI sample commands. The new dependencies require a native build toolchain;
the default build uses vendored libffi. The System InternalCall registry remains separate.

Validation: all 113 integration tests pass, including seven native interop tests for
all supported scalar signatures, mixed integer/floating ABI arguments, native pointer
mutation/returns, foreign pointer forwarding, lazy loading, safe embedding rejection,
metadata shape checks, and loader/symbol failure diagnostics. Clippy and formatting
pass on macOS ARM64. The native sample prints 42 and frees its guest allocation.
Linux/Windows execution remains for the existing CI matrix to verify.

## 2026-09-06 — Typed and block memory operations

Added `initobj T`, `cpobj T`, `initblk`, and `cpblk` to metadata, assembly parsing,
and interpretation. Typed operations use existing native layouts and alignment;
block operations use byte alignment and explicit counts. Copies snapshot before
mutation so overlapping ranges work. No allocation, constructors, ownership, or
memory-management policy is implied.

Block copies preserve initialization bits and complete pointer identities. Partial
writes invalidate overlapping identities. Zero-length operations validate addresses
but preserve tracking, also correcting zero-sized `stobj Void` writes that previously
could discard a pointer identity without changing its bytes. Unsupported native
layouts remain rejected. Documented the prototype's block semantics relative to CLR.

Added the memory sample and nine integration tests covering metadata roundtrip,
record copy/init, overlap, fill truncation, uninitialized copies, checked ranges,
counts/types/alignment, native count widths, stale and partial pointer copies,
zero-length operations, and explicit freeing. The sample prints 42 twice.

Validation: all 122 integration tests pass on macOS ARM64; formatting and clippy
pass. Linux and Windows execution remain for the existing CI matrix.

## 2026-09-06 — Frame-local byte allocation

Implemented `localloc`: an explicit byte count produces aligned, uninitialized
Byte* storage owned by the current function invocation. The evaluation stack must
otherwise be empty. Each frame releases its local buffers on return. Heap and local
buffers share existing budgets and pointer diagnostics, while `heap.free` rejects
local buffers even after pointer/address conversion. Escaped tracked pointers become
stale on return; copied record values remain independent.

The interpreter uses host buffers rather than the host call stack. This implements
frame lifetime without adding GC, reference counting, implicit type-based ownership,
or a guest borrow model. Method initialization flags and argument/local addresses
remain pending. Documented the lifetime and native interop contracts and updated the
allocation proposals and roadmap.

Added a sample constructing a local Point, passing its pointer to another function,
and returning a value copy. Eight tests cover metadata roundtrip, initialization,
manual-free rejection, escaped pointers (including heap-stored aliases), nested frame
lifetimes, byte-budget reclamation, retained heap storage, limits, stack/count checks,
zero-size allocations, and primitive alignment.

Validation: all 130 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The stack sample prints 42 and leaves no live allocations.
Linux and Windows execution remain for the existing CI matrix.

## 2026-09-06 — Switch tables and false branches

Added `switch (Label, ...)` and `brfalse Label` to assembly, metadata, validation,
and interpretation. Switch labels resolve to instruction indices in the temporary
JSON format, including repeated and backward targets. Selection uses an unsigned
Int32 index; out-of-range indices fall through. Empty tables still consume the index.
The loader validates every table target, including unreachable instructions.

`brfalse` mirrors the existing Boolean-only `brtrue` contract. Broader CLR conditional
operands and short aliases remain explicitly documented compatibility work. No static
stack verifier or automatic union matching is implied. Documented the future mapping
from instruction indices to CIL relative byte offsets.

Added a loop/dispatch sample and eight tests for roundtrip execution, target selection,
negative and boundary indices, empty tables, preserved stack entries, backward and
repeated targets, both false-branch paths, malformed/unresolved labels, invalid
serialized targets, operand types, underflow, and instruction budgets.

Validation: all 138 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The sample prints zero, one, two, and outside table in order.
Linux and Windows execution remain for the existing CI matrix.

## 2026-09-06 — Type-first parameter and local syntax

Changed inline parameters to `Type` or `Type name`, and locals to `.local Type`
or `.local Type name`, following the revised assembly syntax decision. Legacy
`.param` declarations use the same type-first order. Names remain unquoted and
optional; the former colon syntax is rejected. Call signatures remain type-only,
and name tables and numeric instruction operands retain their existing encoding.

The parser recognizes a complete type before separating an optional trailing name,
so whitespace in generic arguments and pointer types remains valid. Migrated the
System library, samples, test programs, and current documentation. Historical work
log entries retain the earlier decision for context; this entry supersedes it.

Added coverage for nested/spaced generic and pointer declarations, omitted names,
execution through numeric slots, and rejection of old syntax, quoted names, and
extra names. Existing checks cover duplicate names, reserved names, overload identity,
instance receiver indices, and metadata roundtrip.

Validation: all 141 integration tests pass on macOS ARM64; clippy, formatting, and
diff checks pass. The names sample prints 42 and returns Ok(Void).

## 2026-09-06 — Integer and pointer branch conditions

Extended `brtrue` and `brfalse` beyond Boolean operands to Int32, Int64, IntPtr,
UIntPtr, native pointers, and the prototype Ref arena. Integer zero and pointer
address zero test false; current Ref values are always non-null, including index
zero. Small and unsigned integer storage values use their normalized stack categories.
Both branch paths consume the condition and preserve older stack entries.

Pointer conditions do not dereference memory or validate lifetime: nonzero foreign,
one-past-end, and stale pointers test true. Value records, String, Error, unions,
Void, and floats remain invalid conditions, without implicit truth conversion or
an inferred class/reference distinction. Documented the supported CLR-like zero/null
contract and remaining byref/alias work.

Updated the control-flow sample to test a null pointer and use an integer loop
condition. Added three table-driven tests spanning both branches, integer widths,
zero/nonzero boundaries, normalized small integers, native addresses, stale and
foreign pointers, Ref index zero, stack preservation, and rejected value categories.
Updated earlier Boolean-only rejection tests to use unsupported floating operands.

Validation: all 144 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The control-flow sample retains its expected four output lines.
Linux and Windows execution remain for the existing CI matrix.

## 2026-09-06 — Argument-slot assignment

Implemented `starg` in metadata, assembler name/index resolution, validation, and
interpretation. Stores replace a typed slot in the current invocation and consume
one value. Declared storage contracts handle narrow-integer truncation and Single
rounding. Instance methods resolve `this` to slot zero and explicit parameters to
subsequent slots, matching `ldarg`.

Replacing a value receiver, record, or pointer argument does not write back to the
caller's slot. Pointer assignment has no ownership or allocation side effects.
The existing neoCLR value receiver model also permits replacing the local `this`
value. Argument addresses and short instruction encodings remain pending.

Added a sum-down sample that modifies its parameter while preserving the caller's
local. Five tests cover named/numeric equivalence and metadata roundtrip, storage
conversions including Void, instance receiver/parameter indices and value copies,
pointer replacement, malformed operands, load-time bounds, bad types, and underflow.

Validation: all 149 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The sample prints 6 followed by 3. Linux and Windows execution
remain for the existing CI matrix.

## 2026-09-06 — Direct comparison branches

Added ten familiar CIL opcodes: beq, bne.un, bgt/blt/bge/ble and their .un variants.
The assembler resolves labels to metadata instruction indices; validation checks all
targets. Each instruction consumes two operands, preserves older stack values, and
either branches or falls through without producing a Boolean.

Ordered branches reuse existing integer/floating comparison rules, including opcode
signedness and unordered .un behavior. Floating >= and <= use the appropriate opposite
unordered comparison before inversion, so NaN cannot accidentally take an ordered
branch. Equality branches retain ceq's exact-type value/pointer equality, including
the prototype's structural equality extensions. Pointer ordering and mixed numeric
categories remain unsupported without explicit conversions.

Added an executable sample covering all ten opcodes and seven tests spanning signed
and unsigned integer boundaries, native widths, NaN in either operand position,
infinities, signed zero, equality/pointer behavior, metadata roundtrip, invalid targets,
mixed-type rejection, backward branches, instruction budgets, and stack preservation.

Validation: all 156 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The sample prints Comparison branches passed. Linux and Windows
execution remain for the existing CI matrix.

## 2026-09-06 — Compact CIL constant and slot spellings

Added ldc.i4.m1/0–8, ldc.i4.s, ldarg.0–3, ldloc.0–3, stloc.0–3, and
ldarg.s/starg.s/ldloc.s/stloc.s source aliases. Short slot operands support numeric
indices and optional names, including instance this. The assembler enforces signed
8-bit constant and unsigned 8-bit slot limits, then emits one canonical instruction.
Existing metadata, runtime execution, and label indices are unchanged.

Documented that JSON retains no byte-width preference and that binary compactness
is still future writer work. Short branch forms remain pending. Added a compact
sample and five tests for every fixed alias, canonical output equivalence, named
short forms, boundary values/indices, malformed spellings, missing slots, roundtrip
execution, instance receiver indexing, and labels around compact instructions.

Validation: all 161 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The compact sample prints 42. Linux and Windows execution remain
for the existing CI matrix.

## 2026-09-06 — Identifier mappings and qualified field aliases

Recorded the clarified contract: names are context-specific assembly/metadata aliases
for normalized indices, not necessarily higher-level source identifiers. Parameters,
locals, and fields have independent scopes, with field names unique per declaring type.

Extended ldfld/stfld/ldflda authoring to accept Type::Field aliases. Resolution happens
after declarations so forward-declared types work, then emits the same numeric index
as explicit index syntax. The qualifier selects an assembly mapping table and is not
retained as a runtime receiver-type assertion. Existing storage and index checks still
apply; CLI field tokens and general external-module field resolution remain pending.

Updated the names sample and documentation. Three tests cover all three field opcodes,
forward declarations, exact normalized metadata equivalence, loading/execution,
independent scopes, and source-line diagnostics for unknown/malformed names.

Validation: all 164 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The names sample prints 42 and returns Ok(Void). Linux and Windows
execution remain for the existing CI matrix.

## 2026-09-06 — Sequential record packing and minimum size

Added optional packing and minimum_size metadata with `.pack` and `.size` assembly
directives. Record fields retain declaration order, with placement alignment capped
by packing. Nested records retain their internal layouts. Minimum size reserves
space without truncating fields; final stride rounds up to record alignment.
Omitted metadata preserves previous layouts. Invalid controls and primitive overrides
are rejected during validation, even without an executing layout instruction.

The packed layout applies to allocation, size/alignment queries, field addresses,
and whole-record memory operations. Individual field pointers retain their natural
typed-access alignment checks. Documented this boundary and deferred unaligned
prefixes, explicit field offsets/overlap, and struct-by-value native calls. Layout
controls do not introduce an allocation or ownership policy.

Added a packed Packet sample and six tests for packing sizes, offsets/alignment,
minimum reservation and stride, nested placement, invalid source/metadata, omitted
metadata compatibility, packed field alignment Faults, roundtrip execution, and free.

Validation: all 170 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The layout sample prints 42 and 8, then releases its allocation.
Linux and Windows execution remain for the existing CI matrix.

## 2026-09-06 — Unaligned memory access prefix

Implemented unaligned. with alignment values 1, 2, and 4 for typed/indirect loads
and stores and byte-block operations. Typed accesses relax natural alignment to the
specified cap while retaining all other pointer, type, initialization, and bounds
checks. The prefix modifies one following instruction and is represented explicitly
in prototype metadata. Validation rejects unsupported targets, invalid/dangling or
repeated prefixes, and control flow entering the modified operation past its prefix.
Current value-based ldfld/stfld are not prefix targets; use ldflda plus indirect access.

Updated the packed layout sample. Six tests cover typed/indirect roundtrip, one-access
scope, alignment promises, preserved memory diagnostics, source/metadata validation,
branch and switch entry restrictions, and block operations.

Validation: all 176 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The packed layout sample prints 42 and 8 through a prefixed load.
Linux and Windows execution remain for the existing CI matrix.

## 2026-09-06 — Prioritize union and generic fundamentals

Recorded the user's priority: execution/metadata fundamentals before broad type-system
features and reflection, with a representation for library-defined Option and Result.
Proposed union for tagged typed payloads and enum for integer-backed named constants.
Outlined closed type references, indexed generic parameters, stable case tags and
payload-field indices, general construction/test/extraction operations, and a staged
migration away from hard-coded Option/Result runtime cases. Names remain mappings.

The proposal distinguishes interpreter values from a future native ABI, preserves
Void payloads and value copying, and avoids implicit null sentinels or ownership.
Updated the roadmap to focus next on generic references/substitution, union metadata
and execution, then platform-written System definitions. Reflection is not required.
This is design documentation, not an implementation of general unions or generics.

## 2026-09-06 — Generic type references and field substitution

Implemented indexed TypeParameter and Constructed type signatures, optional generic
parameter name tables, and `.type Pair<T, !1>` declarations. Field names for generic
parameters resolve to indices; indexed references are canonical. The parser now
accepts user-defined constructed type syntax, while validation resolves definitions,
checks arity, rejects open references outside their type context, and validates names
and nesting. Existing builtin Option/Result/Ref/Ptr encodings remain reserved.

Added field-signature substitution APIs for closed types, preserving nested wrappers,
pointer layers, and constructed arguments without recursively expanding definitions.
Void remains an ordinary type argument. The APIs support interpreter implementation;
no guest reflection facility is added. Generic construction, native layouts, methods,
constraints, and variance remain explicitly unsupported. This is the prerequisite
for general union case definitions and execution, not a claim of complete generics.

Added a generic-pointer signature sample and six tests for named/indexed equivalence,
Void and nested substitution, recursive reference signatures, arity/context rejection,
malformed metadata, legacy definitions, substitution errors, and sample execution.
Updated the roadmap and union proposal to mark this foundation complete.

Validation: all 182 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The sample prints Generic signatures passed. Linux and Windows
execution remain for the existing CI matrix.

## 2026-09-06 — Ordinary type convention for unions

Adopted a .NET 11-inspired attribute/member convention for ordinary carrier types.
Replaced the earlier case-table/opcode proposal and removed the uncommitted general
union experiment, including its dedicated type/value category, assembler directives,
opcodes, and experiment-only sample/tests. The committed generic metadata foundation
and existing bootstrap Option/Result behavior are preserved.

Documented ordinary variant and optional synthesized case types, typed access,
explicit None, Void payloads, and distinct Ok/Err wrappers for Result<T,T>. Recorded
that generic records alone do not solve inactive payload storage. The exact custom
attribute/member contract, generic members, addressable output initialization, and
storage remain pending; the convention is not claimed as executable support.

Updated README, assembler documentation, generic metadata notes, and roadmap.
The next implementation slice is closed generic record construction and field access.

Validation: all 182 integration tests pass on macOS ARM64 after removing the
experiment. Formatting, clippy, and diff checks pass. Linux and Windows execution
remain for CI. No published module format or committed runtime behavior changed.

## 2026-09-06 — Closed generic record values

Implemented newobj with a closed type reference and substituted field signatures.
Record values retain their complete Type identity, including generic arguments;
locals, arguments, returns, copies, and field updates preserve that identity.
Nested records, empty generic records, Void fields, and ordinary storage/stack
conversions work without adding a union category or memory-management policy.
Qualified field aliases accept closed generic owners and normalize to indices.

Preserved the legacy serialized name operand for non-generic newobj; constructed
operands use structured type signatures. Invalid/open operands are rejected by the
loader. The Rust embedding API now uses Instruction::New(Type) and Object { ty,
fields } rather than name-only construction/values. Native generic layouts and
methods on generic definitions remain pending. The next type-system slice is
members on generic types, with substitution of their signatures and execution.

Added examples/generic-values.neoil and six integration tests covering roundtrip,
field alias normalization, independent copies, nested identity, storage conversions,
Void/empty values, recursive pointer signatures, and malformed operands/access.
Updated README, type documentation, assembler reference, union sequence, and roadmap.

Validation: all 188 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The new sample prints Generic values passed. Linux and Windows
execution remain for CI.

## 2026-09-06 — IL methods on generic types

Implemented static and instance methods on generic definitions. Declaring-type
parameter aliases normalize to indices across signatures, locals, and typed IL
operands. Calls name constructed owners; execution substitutes type arguments into
method copies, including nested call/construction/pointer operands. Receivers carry
exact closed identities and retain value-copy behavior. Definitions remain open in
serialized metadata and are not rewritten by execution.

Extended validation to method parameter contexts and symbolic calls, while retaining
branch/slot checks on unused definitions. Closed types are checked at call time;
parameter-dependent native layouts are checked at the executed memory operation.
Overloads that collide after substitution produce an ambiguity Fault. Method-level
generics, definition-token overload disambiguation, generic native imports, and a
specialization cache remain pending. Open methods cannot serve as entry points.

Added a Box<T> sample and seven tests covering roundtrip/indexed metadata, static and
instance calls, copied updates, Void/nested values, byte storage, symbolic forwarding,
recursion limits, overload collisions, invalid metadata/owners, and substituted
native layout operands. Updated the type documentation, README, roadmap, and union
prerequisites to reflect the implemented member support.

Validation: all 195 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The sample prints 42 and Generic methods passed. Linux and Windows
execution remain for CI.

## 2026-09-06 — Native storage for closed generic records

Extended sequential native layout to substitute closed generic field types, applying
packing and minimum size per instantiation. Layout recursion tracks complete closed
types, allowing finite Box<Box<Int32>> nesting while rejecting recursive values and
bounding expanding generic recursion. Recursive pointer fields remain pointer-sized.

Native loads reconstruct complete generic type identities. Heap allocation, typed
loads/stores, field addresses, initialization, and copying now work for supported
closed records using the existing pointer checks. No allocation or ownership policy
is added by the type. String, Error, bootstrap unions, and Ref still lack native
layouts, and record-by-value P/Invoke remains unsupported.

Added examples/generic-memory.neoil and eight integration tests for packed/nested
layouts, size reservations, initialized copying, exact loaded types, zero-sized Void
records, unaligned field access, preserved pointer lifetime tracking, recursion limits,
and rejected unsupported field layouts. Updated the earlier unsupported-layout test
to use Box<String>, and refreshed README, generic/pointer documentation, and roadmap.

Validation: all 203 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The sample prints 42 and 4. Linux and Windows execution remain for CI.

## 2026-09-06 — Marker custom-attribute metadata

Added constructor-reference marker attributes on types and methods/functions, with
.custom instance Type::.ctor() assembly syntax. Metadata validates closed owners,
parameterless instance constructor identity, and Void returns without executing
attribute constructors. Repeated entries preserve order; empty lists are omitted
for legacy module compatibility. Added the .ctor member name with instance/Void
signature checks; newobj retains existing field-based construction.

Defined System.Runtime.CompilerServices.UnionAttribute in the platform-written library
as an ordinary marker. It adds no special union behavior. Attribute arguments,
additional targets, AttributeUsage, inheritance, reflection, and automatic constructor
invocation remain pending. Existing InternalCall and P/Invoke metadata are unchanged.

Added an annotated generic-type sample and six tests covering metadata roundtrip,
non-execution, library linking, repeated/forward references, malformed metadata,
source placement, and closed attribute owners. Updated README, assembler reference,
type notes, roadmap, and union convention; added custom-attributes documentation.

Validation: all 209 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. The sample prints Attribute metadata passed. Linux and Windows
execution remain for CI.

Implementation pauses after this slice at the user's request for a strategy review.
The review must include a future high-level language and migration of the runtime
library from authored IL to that language, targeting the same metadata and IL.

## 2026-09-06 — Construction and initialization strategy proposal

Kept runtime implementation paused and documented proposed contracts for values,
storage, addressed receivers, construction, initialization, and recoverable factories.
Recorded scoped byrefs as a possible first verified subset without adopting Rust
exclusivity or implicit ownership. Distinguished addressable interpreter values from
native layout, and partial initialization from null/default values.

Proposed verifier/member-reference/visibility foundations before mutable receivers
and constructor execution. Identified versioned migration questions for current
newobj/stfld semantics and conditional-output initialization for union extraction.
Included the future high-level compiler and incremental runtime-library migration
through the same metadata/IL, without requiring compiler self-hosting.

These are discussion proposals; no receiver, ownership, constructor, opcode, or
binary-format decision was implemented. README and roadmap link the proposal and
record the strategy pause. Validation: documentation diff checks pass; runtime code
is unchanged and the previously passing 209-test suite was not rerun.

## 2026-09-06 — Addressed-access strategy proposal

Refined the proposed boundary between plain values, native pointers, and checked
read-only/writable addressed access. Kept ownership independent and permitted aliases
without assuming Rust-style exclusivity. Specified a possible first subset limited
to initialized local/argument/field locations, reference locals, and call forwarding;
reference returns, aggregate containment, native conversions, and initialization
capabilities remain separate design questions.

Recorded location identity across slot replacement, read-only access versus immutable
data, zero-sized values, generic fields, and addressable String locals without native
String layout. Included source-to-IL mutation behavior and proposed verifier acceptance
cases. Recommended the basic control-flow verifier as the next executable slice when
implementation resumes. Runtime implementation remains paused; no proposed signature,
receiver mode, opcode, or lifetime behavior is implemented.

Validation: documentation diff checks pass. Runtime code is unchanged; the previously
passing 209-test suite was not rerun.

## 2026-09-06 — Explicit control-flow verifier foundation

Started the executable verifier foundation following the strategy discussion. Added
verify/verify_with_library APIs and a verify CLI command for source or serialized
modules. The pass analyzes validated linked IL without execution, reports per-function
maximum stack/reachability, and rejects underflow, inconsistent join heights, invalid
returns, reachable fallthrough, and locals uninitialized on any incoming path.

Worklist propagation intersects local assignment facts and revisits weakened joins.
All IL methods are analyzed, including unused/open generic definitions. Instruction
stack effects are exhaustive; calls produce the inhabited return value and newobj
consumes one logical value per declared field, including Void fields. Native methods
remain metadata-validated without execution. Fault and ret terminate paths.

Verification is opt-in; check, assemble/load, and runtime diagnostic behavior remain
unchanged. This is not type or memory-safety verification. Typed stack states and
reference/constructor verification are pending, and the prior addressed-access
proposals are not implemented by this slice.

Added nine tests covering every shipped IL sample, stack maximum/reachability, joins,
switches, loop backedges, late-arriving assignment facts, generic bodies, prefix
validation, metadata rejection, CLI source/JSON inputs, and the explicit type-safety
limitation. Added verification documentation and updated strategy status/README.

Validation: all 218 integration tests pass on macOS ARM64. After a clippy-only
condition rewrite, the nine verifier tests pass again; formatting, clippy, and diff
checks pass. CLI verification of generic-methods succeeds without execution. Linux
and Windows execution remain for CI.

## 2026-09-06 — Typed evaluation-stack verification

Extended the explicit verifier with abstract stack types and exact type joins.
Calls use resolved/substituted signatures; receivers, argument/local stores, returns,
record construction and field access, numeric categories, and typed pointer/memory
operands are checked before execution. Existing stack-height and definite-assignment
analysis remains in place. The interpreter's callable resolution helper is shared
internally with the verifier; runtime execution semantics are unchanged.

Distinguished stored types from evaluation-stack normalization, including symbolic
loaded generic parameters. Byte/UInt32 loads become Int32, UInt64 becomes Int64,
and Single becomes Double. Raw ldcase/heap.load payloads and exact heap.store rules
retain their current bootstrap semantics. Unconstrained open operations that cannot
be proven are conservatively rejected rather than erasing parameter normalization.

Added eight focused tests covering wrong calls/receivers/returns/stores, same-height
type conflicts, generic record identity, storage normalization, numeric/branch types,
pointer operand contracts, raw union/heap payloads, and open generic normalization.
Updated the earlier verifier tests and CLI message for typed checks. All shipped
examples still verify without execution. Refreshed verification/strategy documentation.

Verification remains opt-in and does not prove pointer validity, memory initialization,
active union cases, arithmetic success, or every generic specialization constraint.
No byref, constructor, ownership, or mandatory-verification contract is added.

Validation: all 226 integration tests pass on macOS ARM64; formatting, clippy, and
diff checks pass. CLI typed verification of generic-methods succeeds. Linux and
Windows execution remain for CI.

## 2026-09-06 — Execution modes as platform architecture

Recorded the user's requirements for embedding, native AOT, and interpretation/JIT/AOT
as architectural concerns across the whole platform. Corrected the earlier framing
around hosting: hosting consumes execution contracts, and does not define mode semantics.
Documented common metadata/IL, backend lowering, target ABI, generics, runtime services,
Fault propagation, capability discovery, and explicit fallback decisions.

Recorded native executable/library AOT experiments, a minimal embedding experiment,
and a future high-level compiler feeding the same platform model. Recommended stable
module/type/member identities as the next common prerequisite. Mixed execution,
code-sharing strategy, native backend selection, public ABI, and memory-management
protocols remain open; no runtime implementation or mode policy was introduced.

Updated README, roadmap priorities, and format direction. Validation: documentation
diff checks pass. Runtime code is unchanged; the previously passing 226-test suite
was not rerun.

## 2026-09-06 — Function definition identities and generic call binding

Added module-local function definition rows to metadata. The assembler assigns rows
in declaration order; linking preserves each source module's rows and rejects supplied
identities that disagree with the definition table. Legacy metadata without identities
remains accepted. Call and attribute constructor references can select a definition
explicitly with `@ Module:index`, retaining signature checks and no name fallback.

The linker binds IL calls before generic specialization. An open generic call retains
its chosen declaration when substitution makes another overload's parameters identical.
The interpreter and verifier share this resolution, and verifier reports distinguish
source definition identities from linked array positions. Binding changes only the
linked copy, leaving input artifacts intact.

Added a sample and seven tests for overload collisions, serialization, library row
preservation, legacy metadata, invalid identities/signatures, and attribute references.
Updated the missing-native-declaration fixture to regenerate row identities after
removing a definition, preserving its original missing-overload assertion. Documented
syntax, compatibility, and architectural limits. These identities survive linking and
specialization, not arbitrary rebuilds; module versioning and type identities remain
future work. No JIT, native AOT backend, or public invocation ABI is added.

Validation: all 233 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The member-identities sample executes both
intended overloads. Linux and Windows execution remain for CI.

## 2026-09-06 — Type definition identities and closed signature keys

Added module-local type definition rows, independent of function rows. Assembly
assigns canonical identities; loading checks supplied identities and derives omitted
legacy rows on resolution copies. Linking preserves System's original rows and
rejects duplicate linked identities. Normalization now covers both definition tables.

Added Rust APIs for resolving a closed signature with the bundled or supplied System
library. Structural keys retain the definition row and ordered generic arguments,
including nested constructions and distinct pointer/bootstrap wrapper signatures.
Primitive aliases resolve to the same canonical definition without conflating storage
types through stack normalization. Void remains usable throughout signatures.
Resolution shares canonical signature, arity, closedness, and nesting checks, performs
no execution, and does not mutate input modules or require a native layout.

Added six tests covering row serialization/validation, independent definition tables,
closed generic keys, primitive alias/System row preservation, wrappers/Void, invalid
or open signatures, nesting limits, and legacy application/library metadata. Updated
the README, identity documentation, generic metadata, and architecture priorities.

Type lookup and interpreter value/layout checks still use the prototype's globally
unique names. General module-scoped resolution, module revisions, persistent caching,
and a prepared hosting context remain future work. Keys are not stable across rebuilt
artifacts, and no ownership or native ABI policy is introduced.

Validation: all 239 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. Linux and Windows execution remain for CI.

## 2026-09-06 — Shared loaded program snapshot

Added LoadedProgram as an immutable owner of validated, linked metadata with calls
bound before specialization. Preparation snapshots an application and bundled or
supplied System library; System alone can be prepared for analysis. Run, verification,
and closed type identity helpers now use this shared boundary. Retaining the object
avoids relinking between those operations, while the existing free helpers remain
available. The internal linked Module is not exposed for mutation or serialization.

Each execution starts fresh guest state and resource limits. Returned execution state
belongs to that run, and a Fault does not alter the loaded metadata. Preparation does
not execute code, activate native imports, or make typed verification mandatory.
Native execution remains a separate unsafe operation with its existing contract;
foreign process-global state is not isolated by fresh guest executions.

Added six tests for retained generic bindings/type identities, source independence,
custom library snapshots, repeated execution and recovery after a limit Fault,
explicit verification, metadata rejection, library-only analysis, and inactive native
imports during preparation. Added a Rust embedding sample that prepares/verifies
HelloWorld once, drops its source, and runs twice. Updated architectural and API docs.

This preparation boundary precedes module-scoped lookup and revision identity; those
remain pending. No arbitrary function invocation, persistent guest session, native
hosting ABI, or JIT/AOT backend interface is introduced.

Validation: all 245 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The embedding sample prints Hello, world!
twice. Linux and Windows execution remain for CI.

## 2026-09-06 — Explicit module sets

Extended the linker and LoadedProgram with an explicit set of additional library
modules. Each source is checked for supported format, unique nonempty module name,
entry-point restrictions, and canonical definition rows before combination. System
remains independently validated. The combined set retains duplicate-symbol checks,
and method declarations must belong to their declaring type's source module.

Added assemble_modules and load_modules for groups of neoIL sources and JSON
artifacts. Group assembly resolves field-name aliases after all type declarations,
including external generic records. Forward and mutual module references can resolve
without staging incomplete artifacts through standalone validation. Returned Modules
remain separate source artifacts; linked indices may change with supplied order,
while module-local identities and explicit call targets remain unchanged.

Added seven tests for cross-module generic calls/field aliases, module-order independence,
separate artifact roundtrips and legacy identities, malformed load sets, duplicate or
missing symbols, mutual references, and cross-module method declaration rejection.
Added a three-module neoIL sample with a Rust launcher, and documented the loading APIs.

Symbols still share one namespace. The caller supplies the complete load set; imports,
visibility, scoped type references, artifact revision constraints, resolver callbacks,
and a multi-module CLI remain future work. No execution or ownership policy changes.

Validation: all 252 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The modules example verifies and prints 42.
Linux and Windows execution remain for CI.

## 2026-09-06 — Explicit direct module reference lists

Added optional module references metadata and `.references (Module, ...)` authoring.
An explicit empty list allows local definitions and implicit System; omission retains
legacy load-set visibility. The loader rejects missing declared modules, duplicate or
self references, and checks uses against each declaring source's direct list without
granting transitive visibility. System remains independently validated.

Checks cover recursive type signatures, fields, locals, typed operands, calls,
attribute constructors, entry references, and root type identity queries. Explicit
function rows do not bypass the lists. Group assembly checks qualified field aliases
before normalizing them to indices. References govern encoded names/definitions, not
all values flowing through calls or a security boundary.

Added seven tests for roundtrips, implicit System, explicit call rows, direct rather
than transitive dependencies, type uses and queries, attributes and entry points,
malformed lists, and legacy behavior. Updated the three-module sample with explicit
Application → Operations → Models references and documented syntax and compatibility.
Names still share one namespace; scoped operands and artifact revisions remain pending.

Validation: all 259 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The updated modules example prints 42.
Linux and Windows execution remain for CI.


## 2026-09-06 — Explicit scoped type operands

Added `[Module]Type` source operands and structural Scoped metadata for named types,
including generic arguments, pointer elements, method owners, and attribute constructors.
Primitive spellings normalize to canonical System names. Declaration names remain
unqualified, and qualifiers cannot apply to contextual parameters or bootstrap wrappers.

Preparation checks module origin before lowering scopes to the current unique-name
signatures. Source artifacts retain their scoped operands through assembly and JSON
roundtrip. Generic substitution and function definition binding remain intact. Group
field aliases and loaded type identity queries accept scoped signatures, and explicit
reference lists still apply. A wrong scope fails without unqualified fallback.

Added six tests covering generic calls/signatures/aliases, source preservation and
roundtrip, wrong scopes and reference restrictions, native pointer/storage operations,
attribute constructors, canonical primitive syntax, malformed signatures, and System
analysis. Updated the three-module sample to use scoped Box operands and refreshed
syntax, identity, loading, and architectural documentation.

This does not yet allow duplicate type names across modules: execution and layout
still use normalized name-based keys. Lower-level raw Module/Type helpers expect
resolved signatures. Internal scoped keys, module revisions, and binary encoding
remain future work; no opcode or ownership policy changes were introduced.

Validation: all 265 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The updated modules example verifies and prints
42. Linux and Windows execution remain for CI.


## 2026-09-06 — Exact artifact revision labels and dependency pins

Added optional `.revision` labels to module metadata and included the label in type
and function definition identities. Canonical row validation now checks revision as
well as module name and index. Legacy missing identities derive the containing
module's revision, while unversioned artifacts retain their existing JSON encoding.

Added exact dependency pins through `.references (Models#build-1)` and explicit
function/attribute row syntax `@ Models#build-1:0`. Name-only dependencies remain
unpinned; explicit row identities always match exactly, including absence of a
revision. A mismatching or missing declared revision fails without name fallback.
Scoped type names continue to select the single supplied module, and resolved type
keys and verifier function identities retain its revision.

Added six tests for serialization/execution, replacement and unused dependency pins,
explicit row matching, distinct revision type keys, stale/legacy rows, malformed
labels, duplicate module sets, System pins, and unversioned encoding. Existing identity
fixtures now state their unversioned component. Added a revision-pinned library sample
and refreshed syntax, identity, and architectural documentation.

Labels are producer assertions, not content hashes or compatibility guarantees.
Producers must assign new labels for distinct artifacts. Side-by-side versions,
content verification, version ranges, and persistent cross-build handles remain
future work. No runtime ownership or execution-mode policy changes were introduced.

Validation: all 271 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The revisions example verifies and prints 42.
Linux and Windows execution remain for CI.


## 2026-09-07 — CLI module sets and mixed source/artifact inputs

Added repeatable --module inputs and an optional --system selection to assemble,
run, check, and verify. The existing positional System argument for run remains
supported. Mixed neoIL and JSON inputs now resolve as one explicit module set,
including scoped field aliases against imported legacy metadata. Assembly validates
all inputs before creating its one requested artifact and preserves no-overwrite
behavior; dependencies remain unchanged.

Added ModuleInput and assembler::read_modules as the shared reader for mixed sets,
with existing source-only and JSON-only group helpers routed through it. Definition
rows are derived in a resolution context without rewriting absent legacy rows in
returned artifacts. The CLI prepares LoadedProgram against the selected System from
initial resolution, fixing the former requirement that an application first resolve
against bundled System before using a custom runtime library.

Added four CLI integration tests for source-set execution/checking/verification,
staged mixed compilation and artifact preservation, custom revision-pinned System
selection including the legacy positional form, malformed flags, and revision mismatch
failure before output creation. Documented runnable module and artifact workflows.

Module discovery, automatic builds, scoped internal type keys, and side-by-side
versions remain pending. Native execution trust, verification policy, and default
resource limits are unchanged.

Validation: all 275 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. CLI tests run the scoped and revision-pinned
samples and confirm their outputs. Linux and Windows execution remain for CI.


## 2026-09-07 — Resolved static function invocation

Added LoadedProgram::resolve_function and borrowing LoadedFunction handles for closed
static IL functions. Resolution observes signatures, scoped owners, explicit revision
rows, and the root's direct reference list. Handles retain their selected specialized
function and expose its identity and signature without permitting mutation.

Invocation accepts exact primitive storage Values, including String, Error, and Void;
guest argument loads still use the normal evaluation-stack conversions. Argument count
and concrete value checks happen before execution. Each invocation owns fresh state
and limits, uses the existing interpreter, and starts directly in the target function
without a synthetic caller frame or instruction charge. Faults leave the handle reusable.
An explicit unsafe variant enables native imports under the existing trust contract.

Instance receivers, aggregate/pointer/Ref inputs, and direct native declaration targets
remain unsupported; IL wrappers can call native declarations. Outputs retain the existing
Execution ownership model and can include guest-created aggregates/Result values. This
is a Rust embedding subset, not a native ABI, persistent session, or backend interface.

Added six tests for overload selection and reuse, exact Byte/Single/Void argument rules,
invalid arguments, guest budget accounting and Fault recovery, generic/revision/scoped
resolution, reference restrictions, unsupported targets, and per-invocation output and
Result values. Added an entry-point-free library and Rust invocation sample; updated
hosting and architecture documentation.

Validation: all 281 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The invocation sample prints Hello, world!,
Int32(42), and Int32(60). Linux and Windows execution remain for CI.


## 2026-09-07 — Validated owned record inputs

Extended resolved static function invocation to accept owned record inputs, including
nested and closed generic records. Function resolution builds a schema from substituted
fields, independently of native layout. Primitive leaves remain exact storage Values.
Import validates nominal tags, concrete value shapes, field counts, and nested values
before execution, and checks/normalizes scoped record tags for the loaded representation.

Recursive by-value schemas are rejected. Import schemas are bounded to depth 64 and
16,384 total nodes across input parameters, in addition to existing signature limits.
Stored pointer, Ref, and bootstrap union inputs remain unsupported; unused phantom
generic arguments do not imply stored pointers. Record results can be imported again
as owned data against the destination schema, not as preserved execution identities.
No constructors, lifetime extension, or implicit memory management are introduced.

Added six tests for independent record updates and result reuse, malformed/fake tags
and fields, scoped tag checks, nested generic Byte and Void fields, nested unsupported
storage, phantom type arguments, recursion, and schema depth. Updated the previous
primitive-only rejection test and added a record invocation sample. Refreshed API,
architecture, and ownership-boundary documentation.

Validation: all 287 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The record sample preserves the original sum
Int32(42) while its updated copy produces Int32(62). Linux and Windows remain for CI.

## 2026-09-07 — Explicit copied instance invocation

Extended LoadedProgram resolution to instance IL methods. LoadedFunction exposes the
closed receiver type separately from declared parameters and provides invoke_instance
and its unsafe native-enabled counterpart. Static and instance entry points enforce
the call kind. Receiver and argument schemas share the existing bounded, exact owned
primitive/record validation; faults distinguish the receiver from declared arguments.

Receivers enter the ordinary interpreter frame as owned values in the existing this
slot. Updates do not write back to a retained host copy. Closed generic and primitive
methods work without a wrapper frame, relinking, or additional guest instruction cost.
Addressed mutation, shared receiver lifetimes, pointer/Ref and union inputs, automatic
construction, and a stable native hosting ABI remain deferred.

Added five tests covering copied generic receivers and result reuse, malformed receivers,
argument diagnostics before execution, safe/native call-kind enforcement, primitive
methods, exact guest budgets, scoped tags, Void, and unsupported stored fields. Added
an IL/Rust sample and updated invocation, ownership, architecture, and roadmap docs.

Validation: all 292 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The sample preserves Int32(21) and produces an
updated Int32(42). Linux and Windows remain for CI.

## 2026-09-07 — Validated bootstrap Option and Result inputs

Extended the owned invocation schema to the existing bootstrap Option/Result values,
including nested payloads and fields in generic record receivers. Import checks the
canonical type tag, permitted case, and exact payload storage value. None requires
its existing Void placeholder; Some<Void> and Result<T,T> retain distinct case identities.
Scoped record and union tags normalize recursively, and faults identify case payloads
and nested fields before guest execution.

Resolution validates every alternative, including unselected cases. Pointer/Ref
payloads, recursive by-value schemas, and schemas exceeding shared depth/complexity
limits fail resolution. Supported outputs can be imported again as owned data. This
adds no IL or metadata categories and does not change the ordinary-type union direction.

Added six tests covering case round trips, exact storage, malformed tags and payloads,
nested scoped records, generic receivers, unsupported alternatives, recursion, depth,
and branching complexity. Updated earlier rejection tests and added an IL/Rust sample
plus invocation and architecture documentation.

Validation: all 298 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The sample reports Succeeded and Failed from
Result inputs and reuses an Option<String> result containing Hello, world!.
Linux and Windows remain for CI.

## 2026-09-07 — Cooperative host cancellation

Added ExecutionOptions with existing Limits and an optional shared CancellationToken.
Entry helpers and loaded-program/static/instance invocation APIs accept either options
or explicit Limits. Token clones share a monotonic request flag; cancellation is
idempotent and cannot be reset. It introduces no guest async or exception mechanism.

The interpreter polls at execution entry and before guest instructions. Observed
cancellation returns a terminal, instruction-located Fault through existing teardown,
without invalidating the program or function handle. Loading and input validation run
first; native calls and in-progress instructions cannot be interrupted. Polling charges
no guest instructions or frames. Execution results retain their existing thread-local
ownership; only the cancellation request is shared across threads.

Added six tests covering token behavior, entry helpers, static and instance invocation
in safe/native modes, validation and limit precedence, cross-thread cancellation,
program reuse, and unchanged guest budgets/Faults. Added an embedding sample and
specified races, native limitations, teardown, and future backend polling boundaries.

Validation: all 304 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The sample cancels Spin at instruction zero
and then successfully invokes Ready on the same program. Linux and Windows remain
for CI.

## 2026-09-07 — Bounded closed call-graph analysis

Added LoadedProgram::analyze_reachability for explicit closed FunctionRef roots.
The report preserves selected definition identities, specialized owners/signatures,
return types, implementation kinds, and call-site edges. Distinct closed owners and
bound generic overloads remain distinct; recursive calls reuse graph nodes. Root
order and IL call order determine report-local indices, independently of supplied
module order for equivalent load sets.

Analysis follows all syntactic calls, including unreachable IL, without execution or
native library loading. Runtime InternalCall and P/Invoke declarations are terminal
nodes; native import metadata is retained. Root references are checked against the
root module, while transitive calls retain the loader-validated declaring-module rules.
A function-count bound rejects expanding generic call graphs without partial output.
Pointer signatures and import roots do not require host input schemas.

Added seven tests for HelloWorld, recursive and unreachable calls, native imports,
generic overload identities, closed instance owners, open-root rejection, transitive
references/revisions, deterministic order, and bounded expanding instantiations.
Added a reporting sample and documented conservative scope and AOT work still needed:
layout closure, runtime services, capabilities, and code generation.

Validation: all 311 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The sample reports Main -> System.Console.WriteLine
-> neoCLR.Runtime.WriteLine without executing HelloWorld. Linux and Windows remain for CI.

## 2026-09-07 — Runtime-service requirements and stack-trace priority

Extended closed call-graph nodes with direct runtime-service uses and source sites.
The catalog distinguishes native/frame allocation, pointer memory, the bootstrap
reference arena, validated parsing/formatting/output helpers, and native interop.
Instruction classification is exhaustive; helper classification uses the existing
runtime binding registry. Ordinary record construction implies no allocator service.

Added distinct aggregate requirements and per-use missing-service diagnostics for an
explicit supplied service set. These are planning results, not proof of opcode, layout,
ABI, or native backend support. Updated the graph sample and documented conservative
unreachable-code handling, closed generic sites, and opaque import dependencies.

Added six tests for HelloWorld, allocator-free value operations, closed generic memory
sites, frame/reference separation, validated binding classification, and unreachable
memory/native requirements. All 317 integration tests pass on macOS ARM64; formatting,
clippy with warnings denied, and diff checks pass. Linux and Windows remain for CI.

Recorded the user's new stack-trace requirement as the next priority: owned logical
Fault snapshots, guest-artifact debug source resolution, and ordinary runtime-backed
System.Diagnostics.StackTrace/StackFrame types. Faults remain unrecoverable runtime/system
errors. The proposal separates current capture from Fault-time capture and records
identity, lifetime, bounds, and backend mapping requirements. No stack-trace implementation
is claimed in this slice.

## 2026-09-07 — Owned Fault stack snapshots

Added optional StackTrace snapshots to execution Faults with owned StackFrame entries
and explicit IL instruction locations. Frames retain selected module/revision/member
identity, closed owner, parameter signature, and method name, innermost first. Caller
frames retain call sites while cancellation and instruction exhaustion identify the
next instruction. Formatting includes the full captured chain after program teardown.

Centralized capture around interpreter execution, including instruction failures,
fallthrough, cancellation, and resource limits. Early execution rejection records the
requested root at instruction zero. Loader, verifier, entry selection, and host input
faults have no invented execution trace. Native-boundary faults show guest callers;
foreign frames and native unwinding are not invented.

Capture keeps at most 64 frames and marks truncation. Frame-vector reservation is
fallible, but metadata cloning still uses ordinary Rust allocation; this does not promise
diagnostics after catastrophic host OOM or native process failure. Snapshots retain no
arguments, locals, guest allocations, or interpreter frame pointers. Rust Fault literals
now require stack_trace; guest metadata/IL and value semantics remain unchanged.

Added seven tests for nested ownership/call sites, generic binding identities,
truncation, execution limits, fallthrough, early cancellation, absent non-execution
traces, and native boundaries. Added IL/Rust samples and updated the diagnostic roadmap.
Guest debug-source mappings and System.Diagnostics.StackTrace/StackFrame remain pending.

Validation: all 324 integration tests pass on macOS ARM64; focused trace tests were
repeated after a formatting refinement. Formatting, clippy with warnings denied, and
diff checks pass. The sample prints Validate -> Process -> Main after dropping its
loaded program. Linux and Windows remain for CI.

## 2026-09-07 — Explicit target layout and runnable-program priorities

Separated scalar layout inputs from host process properties with TargetLayout,
memory::layout_for, and LoadedProgram::layout_of. Pointer size/alignment and Int64,
Single, and Double alignment are explicit and validated. Sequential packing, minimum
size, nested generic fields, and existing layout limits use the selected descriptor.
Interpreter memory operations retain the host descriptor; foreign layout queries are
planning calculations and do not change execution or native marshalling.

Added six tests for pointer/native-integer widths, independent scalar alignment,
packing and nested generics, invalid descriptors, existing type/storage checks, and
host execution consistency. Added a sample comparing explicit 32/64-bit choices and
documented that a data layout is not a complete native ABI or executable allocation.

Updated priorities from the user's clarification: simple runnable programs without
extensive OOP, a demonstrative primitive-backed type set, useful String APIs, and
recoverable Error versus terminal Fault/stack-trace behavior. Arrays are promoted to
this initial milestone, with initialization, bounds, copy/alias, and backing-storage
contracts to settle before array IL. Rich source diagnostics and broader object-model
features follow these fundamentals. No array implementation is claimed in this slice.

Validation: all 330 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The sample reports Packet sizes 16/24 and offsets
[0,4,8]/[0,8,16] for its two explicit descriptors. Linux and Windows remain for CI.

## 2026-09-07 — Candidate language frontends and migration order

Recorded modified C# or a subset of Raven as candidate neoCLR compiler frontends,
without selecting an implementation. The initial compiler targets small programs and
the same metadata/IL as the assembler, then enables incremental runtime-library authoring.
Extensive OOP and broad .NET compatibility are not prerequisites for that subset.
Existing .NET source migration follows the OOP/runtime features those programs require,
with explicit adaptations for neoCLR semantics. Updated the roadmap, architecture, and
README. Documentation-only change; diff checks pass, with no runtime changes or tests needed.

## 2026-09-07 — Initial String API and recoverable text errors

Added ordinary platform-library String.Concat, Equals, IsEmpty, GetUtf8ByteCount,
and SliceUtf8 methods. Equality and emptiness execute IL; concatenation, byte count,
and checked byte slicing use three validated InternalCall helpers. The service report
classifies these helpers as StringOperations. No new IL or object-model feature is added.

UTF-8 byte naming makes the units explicit without selecting Length/general indexing
semantics. SliceUtf8 returns Result<String,Error> for invalid ranges or code-point
boundaries, including empty ranges inside an encoding. Empty strings, embedded NULs,
combining marks, and supplementary scalars retain their exact content. Equality is
ordinal and does not normalize text. Size/allocation failures remain terminal Faults;
source decoding, Rune, grapheme APIs, and native string marshalling remain deferred.

Added six tests for the runnable Unicode sample, owned concatenation, ordinal equality,
byte counts, successful/invalid slicing, repeated invocation, Fault traces, and service
planning. Updated two library tests with the expanded method/import counts. Refreshed
text, runtime-library, service, and roadmap documentation.

Validation: all 336 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The CLI sample prints Hello, neoCLR!, byte count
10, the globe slice, and Invalid boundary handled, then returns Void. Linux and Windows
remain for CI.

## 2026-09-07 — Explicit Array<T> buffer library subset

Added System.Array<T> as an ordinary Data:T*/Length:Int32 record with six platform-IL
methods: Allocate(length, initialValue), get_Length, Get, Set, GetElementAddress, and
Free. Construction explicitly allocates native storage and initializes each element.
Access checks signed bounds and computes offsets with checked native-integer arithmetic.
No new runtime binding, instruction, array type category, GC, or ownership policy is added.

Descriptor copies alias the same allocation without acquiring ownership; element access
uses existing value-copy/storage rules. Free releases the buffer once, not pointees or
element resources. Empty and Void-element buffers work; supported elements require
native layouts. String, Ref, Error, and bootstrap union element storage remains unsupported.
Owned array values and recoverable accessors are separate future work, not silently
assigned descriptor semantics. Documented raw descriptor invariants and execution lifetime.

Added seven tests for initialized typed elements, alias mutation, record element copying,
empty/Void arrays, bounds and negative-length Fault traces, dangling aliases/double free,
resource limits, serialization, and service planning. Updated the library method count
and added successful and deliberate bounds-Fault samples.

Validation: all 343 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The main sample prints 10, 42, 10 and frees its
allocation. The bounds sample exits with a terminal Fault showing GetElementAddress,
Get, and Main. Linux and Windows remain for CI.

## 2026-09-07 — Sample build-and-test instructions

Reorganized the README around a complete sample workflow: prerequisites, automatic
bundled System assembly, quick execution, separate build/assemble/verify/run commands,
optional compiled System selection, and the test suite. Added representative String,
Array, and deliberate bounds-Fault samples, expected outcomes, Windows executable naming,
and output overwrite behavior. Retained the broader sample inventory and module-set
instructions. Documentation-only change; commands match the current CLI and diff checks
pass. No runtime tests were rerun.

## 2026-09-07 — Basic Error methods and recoverable failure sample

Added System.Error.FromMessage, get_Message, and ToString as platform-library IL
methods over two declared InternalCall helpers. Runtime strings can now construct
owned Error values and retrieve their messages. Service planning reports ErrorValues.
No new instructions, exception handling, or automatic Error stack capture are added.

Added a free-function sample that propagates parse errors, constructs a domain error,
reports Result cases, and continues execution. Documented the API and its separation
from terminal Faults. Five tests cover the sample, empty/Unicode/NUL messages, owned
host round trips, input and execution Fault distinctions, and service reachability.

Validation: all 348 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The sample verifies and prints 42, InvalidInt32,
Expected a positive number: -1, and Execution continued, then returns Void. Linux and
Windows remain for CI.

## 2026-09-07 — Record declaration-nullability exploration

Recorded the proposed future model of explicit nullable declarations rather than
nullable type identities. Listed open questions about eligible representations,
return/accessor contracts, generics, verification, initialization, native boundaries,
and method compatibility. Preserved Option for semantic absence and kept allocation
and ownership independent. No implementation or final syntax is chosen.

Validation: documentation-only change; diff checks pass. Runtime tests were not rerun.

## 2026-09-07 — Refine the tooling-nullability candidate

Recorded compiler/tooling enforcement with persisted declaration annotations as a
candidate, including strict compiler diagnostics across compilation boundaries.
Annotations should be consistent without a class/struct split or automatic
Nullable<T> wrapping. Actual null representation remains an independent open question;
annotations alone cannot add a distinguishable null state to fully occupied storage.
No runtime behavior or metadata format changed.

Validation: documentation-only change; diff checks pass. Runtime tests were not rerun.

## 2026-09-07 — Prioritize capability-rich runnable demonstrations

Reaffirmed useful small applications as the immediate priority, without requiring an
extensive runtime library or object model. Proposed bounded UTF-8 file input and a
parse/report application as a smaller integration candidate. Recorded Socket primitives
and a later HttpClient-style library as possible demonstrations, not mandatory next work.
Each integration should define its narrow host boundary, recoverable errors, resource
lifetime, runnable example, and validation. No integration API is selected or implemented.

Validation: documentation-only change; diff checks pass. Runtime tests were not rerun.

## 2026-09-07 — Bounded UTF-8 file input demonstration

Added System.IO.File.ReadAllText(path, maxBytes) as ordinary platform IL over one
explicit InternalCall, classified as FileInput for service planning. The blocking
reader enforces the byte limit against actual reads, decodes UTF-8 strictly, preserves
text content, closes its handle on every return path, and returns expected I/O failures
as Error results. Allocation reservation failures remain terminal Faults.

Added a sample that reads external fixtures, doubles a parsed number, handles invalid
content, and continues. Five tests cover serialization/verification/execution, exact
and exceeded bounds across chunks, Unicode/NUL/BOM handling, invalid UTF-8, missing
files, argument errors, service discovery, and execution Fault context. Documented
commands, error classifications, filesystem access, and blocking/cancellation limits.

The user's priority clarification puts console input/output next, including EOF,
recoverable errors, and visible prompts. Networking remains distant work after a
more robust basic library; the file experiment does not expand that scope.

Validation: all 353 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The CLI sample verifies and prints 42,
InvalidInt32, and File input handled, then returns Void. Linux and Windows remain for CI.

## 2026-09-07 — Establish a fundamentals-first implementation method

Recorded the user's emphasis on avoiding premature primitives and library abstractions.
Each slice should start from a small program, identify a demonstrated gap, separate
VM fundamentals from library policy and host operations, and validate the resulting
contract. Clarified that IL wrappers alone do not move policy into platform code and
that bootstrap helpers are not automatically permanent runtime services. Console I/O
remains next, without requiring a Stream hierarchy or general I/O framework.

Validation: documentation-only change; diff checks pass. No runtime behavior changed.

## 2026-09-07 — Minimal host console and interactive IL demonstration

Added an optional host Console to ExecutionOptions with synchronous raw byte input
and immediate line output. Default Rust embedding retains captured output and reports
ConsoleUnavailable for reads. The CLI explicitly selects StdioConsole and flushes each
line before returning to guest execution, preserving prompts before input and avoiding
duplicate output. Shared console objects retain their external input position across
otherwise fresh executions.

System.Console.ReadByte is a platform-IL wrapper over one declared InternalCall,
returning Result<Option<Byte>,Error>. EOF is ordinary absence; input failures are Error
values. Existing Void-returning WriteLine reports host write failure as a terminal Fault.
Documented blocking calls, partial/external side effects, embedding defaults, and the
experimental Rust API addition. No Stream API or general text reader was introduced.

The sample parses at most nine ASCII digits using ordinary arithmetic and branches,
then doubles the result. It handles empty input, EOF, nondigits, and input failures.
A Byte local bridges raw bootstrap case payloads to normal integer stack loads;
existing union instruction semantics remain unchanged. General line decoding remains
future platform-library work rather than another high-level host helper.

Eight tests cover the runnable/serialized/verified sample, immediate prompt ordering
in a real CLI subprocess, default and injected hosts, exact byte/EOF values, shared
input position, expected errors, output Fault context, cancellation, and service
planning. Updated cancellation callers and runtime-library counts.

Validation: all 361 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. Source and assembled sample execution print
the prompt, then 42 for input 21, and return Void. Linux and Windows remain for CI.

## 2026-09-07 — Prioritize ordinary union contracts and bootstrap removal

Recorded construction/initialization, property/accessor metadata, and accessibility
as near-term ordinary-type fundamentals for expressing a union contract. Current
.ctor-shaped and get_Message-shaped methods do not settle these contracts. Plan the
minimal dependency order around a carrier/variant example, not extensive OOP.

Made removal of all six bootstrap union instructions and the special Option/Result
runtime encodings an explicit completion condition. First replace library, host,
and sample uses with ordinary construction, members, fields, and branches; then
remove special dispatch and explicitly migrate or reject old serialized artifacts.
No union instructions or encodings were removed in this documentation slice.

Validation: documentation-only changes; diff checks pass. Runtime tests were not rerun.

## 2026-09-07 — Invoke record constructors with whole-value initialization

Added signature-based `newobj instance Type::.ctor(...)`, normalized to `newobj.ctor`
in prototype metadata. Constructor references use normal overload binding, generic
substitution, accessibility, module-reference validation and reachability. Execution
supplies a fresh receiver slot in an ordinary guest frame and returns the initialized
owner value when the constructor returns Void. Nonempty receivers require explicit
whole-value initialization through `starg this`; empty records start complete.
Verifier dataflow and unverified execution reject early receiver reads and returns
without initialization. Constructor frames participate in limits and Fault traces.

Added a generic public-constructor/private-field sample and ten integration tests
covering round trips, overloads, named/indexed slots, control-flow joins, invalid
storage/returns, empty records, cross-module access, reachability and frame limits.
Existing aggregate newobj and ordinary value-receiver calls retain their semantics.
Documented this bounded subset and updated the MVP/roadmap. Field-by-field receiver
initialization, addressed access, carrier storage, allocation/ownership policy and
ordinary Option/Result migration remain future work.

Validation: all 396 integration tests pass; formatting, Clippy with warnings denied,
and diff checks pass. The sample assembled to JSON, passed the typed verifier, and
ran from that artifact, printing 42 and its construction message before returning
Void. No push or publication performed.

## 2026-09-07 — Explicit value storage and an ordinary carrier prototype

Added System.Value and explicit value.pack/value.is/value.unpack operations. They
store one complete typed value, test exact closed identity, and perform checked
value extraction without consulting union metadata. Generic operands use existing
substitution, access and module checks. Packing preserves storage types; extraction
uses normal evaluation-stack normalization. ValueStorage service reporting records
the backend requirement without prescribing a guest allocator. Recursive erased
payloads have depth/complexity limits. Native layout and host inputs containing the
new representation are explicitly unsupported in this slice.

An ordinary Outcome<T,E> sample uses public overloaded constructors and a private
erased field, preserving Success/Failure identity when T and E are identical. This
proves an interpreter carrier representation without inactive/default fields or
union-specific instructions. It does not migrate System.Option/Result or settle
the final compiler-recognized member convention. Documented representation/copy
semantics, pointer lifetime, native/host boundaries and remaining migration work.

Validation: all 406 integration tests pass, including ten new storage/carrier tests;
formatting, Clippy with warnings denied, and diff checks pass. The sample assembled
to JSON, passed verification and ran from that artifact with expected output.
No push or publication performed.

## 2026-09-07 — Define Preview 1 through Raven-like program contracts

Named the first public source milestone Preview 1, replacing MVP terminology in
active planning docs while retaining a link from the old mvp.md path. Added a separate
document with six original Raven-like pseudocode programs: hello/free functions,
console computation and recoverable errors, constructed values and ordinary
alternatives, an explicit buffer, terminal Fault traces, and minimal type inspection.
Each records acceptance behavior, IL lowering and current gaps. Pseudocode is clearly
distinguished from runnable IL, an actual Raven syntax contract, or a compiler deliverable.

Updated the release checklist and README/roadmap to derive scope from these programs,
and recorded the now-executable carrier-storage foundation. The final System carriers,
bootstrap removal and guest type inspection remain release gates. No high-level compiler,
Stream API or broader reflection was added to the target.

Validation: local links in changed planning documents and diff checks pass. Extracted
the complete P1 IL mapping, assembled it to JSON, verified it, and checked its expected
HelloWorld output. Runtime validation remains the 406 passing integration tests and
format/Clippy checks from the preceding implementation slice. No release or push performed.

## 2026-09-07 — Explicit property metadata and accessor associations

Added optional property records to type metadata, with name, static/instance kind,
index parameters, value type, and getter/setter method references. The assembler
accepts property blocks with explicit .get/.set signatures. Preparation normalizes
scopes and generic parameters and binds accessor identities in the prepared copy.
Validation checks signature uniqueness, available accessors, owner/module, call kind,
parameter and return types, and supplied method identities.

Access remains ordinary method calls. No property instruction, backing storage,
receiver mutation rule, accessibility, reflection API, or union behavior was added.
System.Error.Message and System.Array<T>.Length now associate existing accessors.
The sample reads a generic Box property and Error.Message. Documented syntax,
indexed properties, serialized compatibility, and copied-receiver limitations.

Six tests cover the generic executable/serialized sample, indexed pointer-backed
getter/setter calls with overloads, malformed source and JSON, forged accessor tokens,
scoped/module-reference validation, legacy omission, and library associations.

Validation: all 367 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The sample verifies and prints 42 and
Properties describe ordinary methods, then returns Void. Linux and Windows remain for CI.

## 2026-09-07 — Record explicit inheritance policy

Recorded future explicit control over type derivability and hierarchy closure as
separate contracts. Listed defaults, permitted-subtype declarations, indirect derivation,
module boundaries, generics, and versioning as open questions. Kept the policy separate
from allocation/value semantics and from ordinary union representation.

Validation: documentation-only change; diff checks pass. No runtime behavior changed.

## 2026-09-07 — Initial method accessibility

Added public/internal/private method metadata and optional declaration modifiers.
Legacy omissions remain public. Internal uses the declaring module/revision boundary;
private uses the declaring type definition, including generic instantiations. Private
free functions are rejected in favor of module-internal helpers.

Loading checks every explicit call, including unreachable calls, and execution checks
resolved caller/callee access. Host member resolution requires public methods regardless
of symbolic or bound references. Explicit local entry selection can designate a non-public
method; a foreign non-public entry is rejected. Read-only analysis and property metadata
references do not grant invocation access. No field/type visibility, inheritance, or
construction semantics were added; full representation protection remains future work.

The sample combines a public factory/property reader, a private same-type helper,
an internal free function, and an internal entry. Seven tests cover round trips,
legacy defaults, generic type identity, permitted/denied calls, entry selection,
module revision boundaries, bound-token host checks, and private property accessors.
The user reaffirmed familiar .NET-style access levels for now; a redesigned model
is explicitly deferred rather than added to this slice.

Validation: all 374 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The sample verifies, prints 42, and returns Void.
Linux and Windows remain for CI.

## 2026-09-07 — Field accessibility and direct construction checks

Added optional public/internal/private field metadata and assembler modifiers, preserving
public defaults and visibility through generic substitution. Ordinary field reads,
updated-copy writes, and field-address operations check the declaring type/module
identity. The typed verifier checks reachable operations with inferred receivers;
unverified execution checks actual receiver types and reports Fault traces. Direct
field-based newobj requires access to every initialized field, checked during loading
and execution. Public factories on the declaring type can construct private fields.

Added a generic Box sample with a private field, public factory/property reader, and
WithValue method returning an updated copy. It prints both the original 21 and updated
42. Six tests cover generic metadata/serialization, copy behavior, forbidden direct
construction including unreachable code, indexed and named field access, field addresses,
module revision boundaries, legacy defaults, and the trusted host-import boundary.

Documented that raw memory operations and owned Rust host record imports retain their
explicit low-level/trusted semantics. These checks do not establish constructor provenance,
make layouts secret, or sandbox native code. Type visibility and full construction rules
remain future fundamentals; no union instructions or ownership policies were added.

Validation: all 380 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The sample verifies, prints 21 and 42, and returns
Void. Linux and Windows remain for CI.

## 2026-09-07 — Top-level type visibility

Added public/internal type declarations with public legacy defaults; private top-level
types are rejected. Loading checks explicit type uses across signatures, locals,
fields/properties, attributes and typed operands, plus resolved call signatures that
could expose an unspelled return type. Member access checks respect declaring-type
visibility. Host invocation checks closed owner/parameter/return types as well as method
access. Read-only identity/layout/reachability inspection remains available.

Generic bodies retain their open-context permissions: a public library can operate on
a caller-supplied internal type through T without acquiring permission to name that type
explicitly. Ordinary field operations check declared field-type accessibility. Existing
raw-memory and trusted host-data limitations remain documented.

Six tests cover the serialized sample, foreign type references, implicit return/field
exposure, host invocation versus inspection, cross-module generic code, and legacy/invalid
metadata. The sample uses an internal helper type and prints 42.

Validation: all 386 integration tests pass on macOS ARM64; formatting, clippy with
warnings denied, and diff checks pass. The sample verifies and executes successfully.
Linux and Windows remain for CI.

## 2026-09-07 — Define the public source MVP

The user selected a runnable source preview with assembler, interpreter, library, and
samples as the first publication target. Added an MVP capability/gap table, implementation
order, required offline demonstrations, release acceptance checklist, and explicit
exclusions. Remaining feature gates center on construction/carrier storage, ordinary
Option/Result, removal of bootstrap union operations, and minimal read-only guest type
inspection. Binaries, a high-level compiler, networking, broad reflection, and executing
JIT/AOT backends are outside this release target.

Recommended and included type descriptors, identity/name inspection, and closed generic
arguments as the minimal reflection scope; member enumeration, dynamic invocation and
reflective mutation remain deferred. Publication gates include tested platform/toolchain
evidence, clean-clone instructions, an owner-approved license, source provenance/notices,
versioning and release notes. No license was selected, release tag created, push performed,
or publication made. README and roadmap now link to the MVP plan.

Validation: documentation-only plan; diff checks pass. Runtime validation remains the
386 passing integration tests recorded for the completed type-visibility slice.

## 2026-09-07 — Clarify freedom from legacy API constraints

Recorded that neoCLR need not reproduce CLR/.NET legacy structures or every historical
behavior. Familiar concepts remain useful for understanding and migration, while explicit
structural and behavioral improvements are valid design choices. Reflection in particular
should model neoCLR and can evolve with it; the MVP's read-only subset is not a commitment
to the .NET reflection hierarchy or a stable compatibility surface. No scope expansion
or runtime implementation change was made.

Validation: documentation-only changes; diff checks pass. Runtime tests were not rerun.

## 2026-09-07 — Ordinary System.Option/Result library carriers

Implemented System.None, Some<T>, Ok<T>, Err<T>, Option<T> and Result<T,E> as ordinary
platform-defined types. Nineteen new IL methods provide constructors, read-only payload
properties, variant predicates and checked wrapper accessors. Carriers hold one private
System.Value; no interpreter dispatch, native intrinsic or union opcode was added.
Selected and documented the Preview 1 constructor/query convention for future tools,
including exact wrapper identity, suffix restrictions and behavioral obligations.

Added an executable ordinary-union sample and eight integration tests for serialized
round trips, equal success/error payload types, Void/None distinctions, arbitrary error
payloads, nesting, failed accessors, private representation, independent copies, and
execution without marker metadata. Fully qualified System.Option/Result select ordinary
types while unqualified spellings and existing APIs remain explicit bootstrap migration
debt. Updated Preview 1's Raven-like IL mappings and current library documentation.

Validation: all 414 integration tests pass; formatting, Clippy with warnings denied,
diff checks and local documentation links pass. The sample assembled to JSON, passed
the typed verifier and ran with expected output. Host/native adapter migration and
bootstrap removal remain unfinished; no push or publication performed.

## 2026-09-07 — Bounded host input for ordinary carriers

Added explicit erased-value input schemas so System.Value and records containing it
can be supplied to loaded functions. Concrete payload types are validated against the
loaded module, then imported with exact primitive/record rules and scoped-name checks.
Shared per-argument/receiver value-depth, value-count and dynamic-schema budgets prevent
nested erasure from resetting complexity limits. Pointer and Ref payloads remain
unsupported. Validation checks shape, not private-constructor provenance or union
behavior; no carrier-name or marker dispatch was added.

Added a Rust/IL example that constructs System.Result<Int32,String> in guest code and
passes it through the host to another invocation, plus six tests for reuse, scoped
normalization, malformed trees, pointer rejection, bounded nesting/complexity and the
trusted structural boundary. Updated hosting/value-storage and Preview 1 documents.
Recorded Rust-like Map/MapError/AndThen as future ordinary library composition APIs,
with generic methods/callables as prerequisites; access-modifier redesign stays deferred.

Validation: all 420 integration tests pass; formatting, Clippy with warnings denied,
diff checks and local documentation links pass. The new hosting example prints
Int32(42). Existing API/native/bootstrap host paths still need migration before union
opcode removal. No push or publication performed.

## 2026-09-07 — Migrate Int32.Parse to ordinary Result

Changed System.Int32.Parse to return ordinary System.Result<Int32,Error>. Its existing
InternalCall service now returns an erased Int32 or Error primitive; platform IL
constructs the appropriate Ok/Err wrapper and Result. The runtime helper no longer
constructs a bootstrap union for parsing. Service analysis includes ValueStorage for
InternalCall declarations returning System.Value. Updated every checked-in Parse
caller, including the Error sample, and adapted host tests to ordinary carrier access.
The file example explicitly converts its still-bootstrap file error in IL.

Documented the changed public/native return contracts, caller migration, unchanged
parsing grammar and remaining bootstrap APIs. Added four tests covering limits and
invalid input, the native payload protocol, old-contract rejection and service graphs.

Validation: all 424 integration tests pass; formatting, Clippy with warnings denied,
diff checks and local documentation links pass. The Error sample assembled to JSON,
verified and ran with unchanged output and no bootstrap union instructions. No push
or publication performed.

## 2026-09-07 — Record ordinary nested cases through generic-union companions

Recorded the user's clarified design: a non-generic Result companion contains ordinary
nested Result.Ok<T> and Result.Error<TError> case types, while Result<T,TError> is the
separate generic carrier. Nesting and union membership imply no inheritance. This
replaces the tentative selective outer-parameter capture exploration; no such semantics
were implemented. The existing top-level library wrappers remain unchanged.

Identified name-plus-generic-arity type identity as the first implementation prerequisite,
followed by real nested ownership and generic case definitions under the non-generic
companion. Updated Preview 1 and the union convention with that order and documented
required cross-layer validation and tests. General nesting under generic outer types
remains a separate contract; no new assembler or metadata encoding is claimed.

Validation: documentation links and diff checks pass. Runtime validation remains the
424 passing tests plus format/Clippy checks from the completed Parse migration.
No push or publication performed.

## 2026-09-07 — Distinguish type definitions by generic arity

Implemented same-name type definitions with different generic arities. Generic method
definitions now carry explicit open constructed owners, resolving ambiguity between
non-generic companions and generic carriers. Definition/field/layout lookup, member
binding, type identity, scoped references, accessibility and host schemas select the
correct arity. Method signatures distinguish owners, while conflicting free-function
signatures retain the existing reserved-name check. No nesting or union dispatch was
added in this slice. Older generic-method artifacts require reassembly.

Added a sample and five tests covering arity-zero/one/two methods, round trips, layout,
host receivers/identity, cross-module visibility/reference checks, private isolation
and malformed metadata. The full suite caught a reserved free-function collision
regression; restored that check and reran the full suite successfully.

Validation: all 429 integration tests pass; formatting, Clippy with warnings denied,
diff and documentation-link checks pass. The sample assembled, verified and executed
from JSON with expected output. No push or publication performed.

## 2026-09-07 — Ordinary nested type ownership

Added optional declaring-type definition IDs to prototype metadata and nested `.type`
declarations to the assembler. Nested case types under non-generic companions own
all their generic parameters locally. Qualified references use existing type lookup,
method binding, record storage, layout, verification and host input machinery.
There are no new instructions or union-specific runtime semantics.

The loader validates owner existence, same module/revision, matching immediate
qualified names and bounded acyclic ownership. Public/internal nested type access
includes enclosing visibility. Generic outer nesting and private nested types are
explicitly deferred; private member access retains exact declaring-type semantics.
A dotted top-level name alone does not establish ownership.

Recorded that a future Raven-like frontend can expose one union declaration while
lowering its carrier and non-generic companion to separate ordinary types. Added
`examples/nested_types.neoil`, which prints 42, and four integration tests covering
serialized execution, nested generic host receivers and identities, Void payloads,
multiple levels, module linking, effective visibility and malformed ownership.
The existing System wrapper names remain intact pending the next migration slice.

## 2026-09-07 — Typed slots and explicit erased storage

Clarified the `System.Value` boundary after nested-case work. Locals and fields are
always typed slots containing complete values; `System.Value` is an explicit erased
carrier for heterogeneous slots, with visible pack/is/unpack operations. It is not
a universal base class or implicit boxing target. A native backend may lower a slot
to pointer, layout metadata and size/alignment, while `Ptr<T>` remains the explicit
unmanaged capability and does not acquire ownership. `System.Object` is reserved
for a future common object API and is not introduced into Preview 1.

The storage decision was refined: `System.Value` is not the mechanism for C-style
unions or overlays. Those belong to an explicit future type/layout description with
size, alignment, offsets and discriminant or unsafe access rules. This preserves the
low-level memory model and leaves allocator and heap management independent of the
metadata carrier used for checked heterogeneous values.

Refined the companion rule: non-generic unions may nest their variants directly;
only generic carriers require a separate non-generic companion to keep variant
parameters independent. Runtime companion declarations are now present alongside
the existing top-level wrappers, allowing incremental caller migration.

Array representation was clarified: `System.Array<T>` is a typed, non-owning view
over `T*` plus length, independent of whether the storage is frame-local or heap
allocated. This follows the same primitive-wrapper model as `System.Int32`; type
safety comes from typed values and checked pointer provenance, not a value/reference
classification.

Recorded the platform policy principle: neoCLR standardizes low-level typed values,
array views, pointers, allocation regions, layouts and checked memory operations as
VM capabilities. Language authors may add safety or ownership profiles above that
surface; C#-specific historical restrictions are not implicit runtime rules.

Added the interrupt/native architecture note: execution modes share explicit
interrupt state and safepoints, while native calls declare ABI, layout, pointer and
ownership boundaries. Managed language profiles may restrict these capabilities,
but the low-level VM does not remove them.

Reaffirmed the main memory theme: explicit allocation, addresses, layouts, lifetimes
and destruction are the default VM contract. Reference counting, garbage collection,
arenas and ownership analysis are optional abstractions that languages may layer on
top and lower explicitly.

Public positioning was clarified: neoCLR is managed and type-safe, but intentionally
low-level. Typed pointers, explicit memory access and native calls are first-class
escape hatches; language-level ergonomics remain optional layers above the VM.

Recorded the native-lowering requirement: typed stack effects, explicit control
flow and memory operations form a portable IL subset that interpreters, JITs and
NativeAOT backends must share. Allocation, I/O, provenance checks, interrupts and
native interop remain explicit runtime-service boundaries.

Defined the opcode policy: preserve CLI mnemonics and useful stack semantics where
they fit, document neoCLR deviations and additions, and mark temporary bootstrap
instructions for explicit removal at a metadata-version break.

Recorded dynamic dispatch as an optional VM service. Typed calls remain directly
verifiable and lowerable; dynamic call sites explicitly request a handler that owns
language lookup while preserving result typing, accessibility, resource limits,
interrupts and stack traces.

Recorded the compatibility strategy: retain familiar CLR metadata, signatures,
visibility, properties, calls and stack effects where useful; make fundamental
improvements explicit, versioned and diagnosable so language migration remains
incremental.

Added a complete opcode status inventory to the neoIL reference, grouping
CLI-aligned instructions, explicit neoCLR memory/value operations, and temporary
bootstrap union operations.

Recorded typed parse errors as a future ordinary union domain. `Result<T,TError>`
can carry cases such as invalid format or overflow without exceptions; the preview
continues to use `System.Error` until the carrier migration is complete.


## Canonical nested-case API migration

Replaced the temporary ReadByteTyped, ReadAllTextTyped and AbsTyped alternatives with
canonical Console.ReadByte, File.ReadAllText and Math.Abs APIs returning ordinary
nested System.Result/Option cases. Console/file host bindings return bounded erased
payloads (Byte/Void/Error and String/Error respectively); library IL owns all case
construction, with no bootstrap union opcodes in these methods or native union values
at these boundaries. Explicit System.Value storage remains the existing preview
representation, not a new memory-management policy.

Migrated the interactive console and file examples and their host expectations. Fixed
Option/Result predicates to recognize nested cases as well as the older ordinary wrapper
family; this corrects EOF recognition in the single-byte console example. Also updated
stale parsing consumers in errors/types samples and made carrier/library tests assert
behavior instead of obsolete exact method counts. Added EOF, full-byte-range, read
failure, missing-console and canonical-API regression coverage.

This is a breaking System library/native binding contract: reassemble old applications
and System artifacts together. The old Typed names are removed, and callers must use
GetOkCase/GetErrorCase/GetSomeCase plus case Value accessors. JSON encoding has not
changed; this does not imply old library contracts remain compatible. Divide, slicing,
older ordinary wrappers and other bootstrap union uses remain explicit Preview 1 debt.

Next slice: split the System library sources by class or feature into namespace folders,
retaining one assembled System module and a reproducible source build.

Validation: the full `cargo test --locked --no-fail-fast` suite passes, as do
`cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and
`git diff --check` on macOS ARM64.

## System sources organized by namespace

Split the monolithic runtime source into class/feature files under System and
neoCLR/Runtime namespace folders. Generic Option/Result companions stay with their
carriers. The ordered System.neoil manifest still produces one System module and
preserves all pre-split metadata rows and instruction bodies.

Added an explicit file source loader for quoted relative includes, shared by the
CLI and Cargo build script. Bundled source comes from the manifest at build time;
there is no runtime filesystem dependency and no independently maintained combined
copy. String-based assembly remains filesystem-independent. Existing explicit System
assembly commands continue to work. Loader tests cover relative paths, nested includes,
spaces, malformed syntax, missing files, cycles, bundled/source parity, and HelloWorld
against an explicitly assembled System artifact. Assembler diagnostics currently use
expanded line numbers; source-map support is deferred.

Validation: pre-split and manifest assembly produce identical serialized metadata/IL;
all 21 source-loader, CLI, library and native-binding tests pass. Formatting, clippy
with warnings denied, and diff checks pass on macOS ARM64.

## Runtime Result API review

Audited all six public Result-returning methods and their native failure branches.
Documented proposed operation-specific error contracts in runtime-error-contracts.md,
including Parse's missing format/overflow distinction, checked arithmetic, UTF-8 ranges,
console EOF versus failures, and bounded file input. The review specifies canonical API
migration, ordinary nested error cases, structured native outcomes, and acceptance tests.
It does not implement these proposed types or silently change output Fault policy.
