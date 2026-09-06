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
