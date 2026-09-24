# String storage and reference identity investigation

**Development, 2026-09-24.** Shared immutable String storage is implemented.
String reference identity is now enabled in development through the shared owner.
Earlier sections retain the investigation history; the final section records the implemented contract.

## Original scenario and measured baseline

An application puts the same text in a local, record field and array, passes it as
Object, then casts it back. It should preserve the reference while content equality
remains independent of identity. Before this migration `Value::String(String)` was cloned by slot
reads and other value copies. String-to-Object/interface conversion allocates a
managed wrapper; conversion back reads a copied payload. Object ToString copies
text again. ReferenceEquals deliberately rejects String, even through aliases.

The original [experiment](experiments/string-storage/README.md) measured owned
Value clones and a private Rust `Arc<str>` candidate. With 10,000 clones:

| UTF-8 bytes per value | Owned baseline allocations | Cumulative requested bytes | Shared-clone allocations/bytes |
| ---: | ---: | ---: | ---: |
| 0 | 0 | 0 | 0 / 0 |
| 32 | 10,000 | 320,000 | 0 / 0 |
| 4,096 | 10,000 | 40,960,000 | 0 / 0 |
| 65,536 | 10,000 | 655,360,000 | 0 / 0 |

Each clone is immediately dropped. These are allocator requests in a Release build,
not peak live memory, a byte-copy instrument, timing results or an end-to-end speedup.
Construction is outside the measured region: Arc requires a control block and
allocation, and converting an existing owned String to `Arc<str>` may copy its bytes.
Atomic reference counting also has a per-clone/drop cost. Empty owned strings already
avoid a text allocation. That `Arc<str>` candidate was not integrated with Value or the VM.
The production clone measurement below now uses `Arc<String>`.

The current-VM cast baseline allocates 1,000 wrappers for 1,000 casts of the same
String local. An eight-object limit produces 125 collections, peak eight and zero
live wrappers at completion. This demonstrates conversion allocation separately from
payload clone requests; it does not measure all bytes allocated by the VM.

The prototype's Weak handle stops upgrading after the final strong owner is dropped.
That checks logical ownership release; the observer itself retains the Arc control
block until it is dropped, so this is not a physical deallocation measurement.

## .NET baseline and layers

Primary sources reviewed 2026-09-24:

- [C# reference types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/reference-types#the-string-type): String is immutable; references can alias the same object, while string equality compares contents.
- [.NET 10 String source](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/String.cs): ToString returns the same String instance; storage is UTF-16 and runtime allocation participates in the CLR heap.

The checked .NET 10 comparison preserves identity across assignment, Object conversion,
casts, record fields, arrays, ToString and a forced collection. It separately constructs
equal text with a different identity. It does not assert interned literal behavior.

The intended neoCLR reference behavior should match those cases while retaining
UTF-8 and grapheme APIs. No language syntax or new public API is needed for these
cases. Runtime representation, tracing/ownership and host boundaries must enforce
them; compiler-only adaptation cannot recover identity once text has been copied.
Interning, normalization, nullable String storage and generic math are separate work.

## Alternatives

| Candidate | Benefit | Costs and unresolved points |
| --- | --- | --- |
| Keep owned inline text and wrappers | No migration; current content behavior works | Repeated payload copies; no coherent String identity across conversions |
| Shared immutable leaf payload, initially behind an internal text handle | Cheap clones and one identity for aliases; a leaf cannot create a reference cycle by itself | Refcount/control-block cost; host lifetime differs from tracing GC; accounting and wrapper identity must use the text owner rather than wrapper allocation |
| Put every String in the managed tracing heap | One reference model and heap-owned identities; direct root tracing | Changes literal/native-result creation and all storage paths; new allocation safe points, heap-count behavior and host/worker transfer rules |

A wrapper cache alone is insufficient: current String clones have already lost
origin identity. Content interning would also merge separately created equal strings
and impose a different allocation policy. Neither is selected as an identity repair.

## Original provisional direction

Proceed toward one internal immutable text handle and shared payload, while keeping
String's public meaning unchanged. The measured copy costs justify that next
implementation slice, but do not select `Arc<str>` as the final runtime allocator.
Prototype evidence is limited to clone cost, alias identity, Rust-container ownership
and release after the last owner. It is not proof of guest tracing-GC integration.

The first integration gate should preserve one text owner across ordinary VM copies,
then across Object/interface views and back to String. Identity must follow that
owner, not a conversion wrapper. Do not enable ReferenceEquals until the round-trip
and lifetime matrix passes. Empty strings need a deliberate policy; interning remains
optional and is not required for assignment identity.

Compare `Arc<str>` with a handle that adopts an owned String buffer before selecting
construction behavior. Keep the payload immutable and free of managed-reference
backlinks so reference counting does not create cycles. Do not expose native addresses
as guest identities or hashes. A public Rust Value payload change is a host API
migration even when guest metadata is unchanged.

## Migration map and validation gates

- **Slots, arrays and fields:** Value clone/slot reads, erased Result/Option payloads,
  class fields and record components must share a text owner. Check independent
  equal text, aliases, empty strings, embedded NUL, combining sequences and emoji.
- **VM conversions and dispatch:** literals, castclass/isinst, interface views,
  Object ToString and casts back must retain ownership. Current wrapper allocation
  safe points must remain correct until wrappers are removed or reused.
- **GC and accounting:** String is currently a leaf in `gc::trace`; text allocations
  are not separate managed objects. A shared-leaf design must release payloads when
  the last host/frame/heap owner goes away. Verify locals, operand stacks, fields,
  arrays, cycles of containing objects, failures and execution teardown. Track text
  bytes separately rather than implying `heap_objects` bounds them.
- **Native and host boundaries:** constructors, concat/slice, UTF-8 decoding, files,
  console and reflection produce text. Debug snapshots and returned Values retain
  host copies today. Decide whether the new host result owns text beyond execution.
- **Workers:** workers currently transfer owned input/result Strings and enforce a
  result/output byte quota. Keep accounting explicit if bytes become shared; do not
  silently treat sharing as quota-free or share mutable worker state.
- **Compiler/library consumers:** keep String operators, typed Equals and Object
  equality consistent; run record and collection samples. Keep content hashes
  independent of identity. No change to UTF-8/grapheme indexing is implied.

Before claiming a performance improvement, measure construction and destruction,
retained bytes, small-string overhead, representative app allocations and worker
boundaries, not just repeated clone allocation counts. Enable guest String identity
only after .NET-comparison cases and the GC/host-lifetime gates pass.


## Implemented shared-text phase (2026-09-24)

The VM now stores `Value::String(StringValue)`, an immutable `Arc<String>`
leaf. Slot reads, local/field/array copies, Object wrapper reads and String Object
ToString retain its owner rather than cloning text bytes. Construction from an owned
String adopts its buffer. Unlike the earlier `Arc<str>` candidate, this avoids copying
a producer's existing buffer, at the cost of a separate control-block allocation,
String capacity/header overhead and atomic reference counting. It is not an inline
.NET UTF-16 managed object, an intern pool or a guest identity contract.

Tracing GC still owns wrappers and containing objects. Sweeping them drops their
text owners; a retained host Value can keep text alive beyond the heap. Tests check
rooted arrays, sweep, retained host copies and last-owner release. Text remains a
leaf outside managed object-count statistics. Array quotas count bytes per logical
occurrence, even for aliases. Worker input/result boundaries use owned text and keep
their existing byte quotas. No global text-byte quota or end-to-end speedup is claimed.

Rust host migration: construct `Value::String(owned.into())`, borrow with
`text.as_str()`, and extract owned text with `text.into_owned()`. The latter
copies if other owners remain. Source rustdoc covers this host-only type; no guest
signature or compiler bridge change is required. String/Char content semantics,
grapheme operations and UTF-8 validation are unchanged. String ReferenceEquals and
identity/base hashing remain rejected: wrapper allocation is not String identity.

The Release probe now compares an owned String baseline with production Value clones.
Construction is excluded; zero clone allocations does not measure atomic costs.
The next gate is identity-preserving conversions and their lifetime/GC tests, before
enabling guest identity.

## VM conversion ownership gate (2026-09-24)

The private `string_ownership` tests exercise the actual selected System library,
serialized metadata, verification and host invocation. They compare Arc owners rather
than contents or text-buffer addresses. In particular, empty text needs an owner
comparison: independent empty String buffers can have the same data pointer.

The matrix covers Object casts, successful type tests, Equatable interface views,
Object ToString, local byrefs, class/value fields, arrays and erased payloads. Empty
text, embedded NUL, combining text and emoji must keep the original owner. Separate
equal host inputs must remain separate owners. This validates the current internal
representation without promising interning behavior for future text producers.

Additional checks retain a wrapper only on the operand stack while repeated casts
force collection; all temporary wrappers must be reclaimed without replacing its
text owner. A host-held result must survive execution/heap destruction, then release
its last owner. Cyclic class fields must release text both after normal completion
and after an intentional guest Fault.

This closes the first conversion/teardown gate against the alias/field/array cases
in the existing .NET comparison. It does **not** enable guest ReferenceEquals or
identity hashes. Existing wrapper IDs must not become String IDs. The next slice
must define owner-based reference comparison, stable identity/base hashing without
exposing addresses, and null/mixed-type behavior together. Content hashes remain
separate from that contract; culture, casing and comparer design stay future work.

Run `cargo test --lib string_ownership`. These are internal runtime checks; there
is no new guest API, compiler policy or RavenDoc signature in this slice.

## Future text and comparer direction

The author requested a coherent System.Text API and general comparer infrastructure,
especially string comparers, as later work. Compare .NET's
[StringComparer](https://learn.microsoft.com/en-us/dotnet/api/system.stringcomparer?view=net-10.0),
which pairs equality/hashing and ordering under explicit ordinal or culture policies.
The initial design should keep those policies separate from text ownership. Evaluate
an explicit ordinal policy first; case folding, culture data, normalization, default
collection comparers and the UTF-8 versus UTF-16 ordering difference need separate
contracts and tests. Reusing policy objects can keep collection hashes consistent
with equality, but adds API surface and, for culture support, data/runtime costs.
No comparer API, namespace move or culture behavior is implemented in this slice.

Planned text work also includes ToUpper/ToLower-style casing and comparison methods.
.NET is a reference, not an API-copy requirement: adapt names and contracts where a
concrete benefit justifies the compatibility cost. Culture selection, Unicode casing
(including length changes), ordering and equality/hash consistency need explicit
choices and tests. Additional casing/comparison APIs remain planned; the existing
bounded ordinal helpers retain their current contract.

## Sequence construction — development, 2026-09-24

The author requested String construction from char arrays, then suggested Sequence<char>
and explicit Count. String now implements Sequence with interface-only Count, public
Length and a read-only grapheme indexer. The constructor snapshots Count characters
through one iterator traversal and concatenates exact UTF-8 text. No normalization,
identity or interning behavior is introduced. A mutable input must stay stable during
construction; later mutation is independent. Concatenation can merge grapheme boundaries.

Compared with [.NET String constructors](https://learn.microsoft.com/en-us/dotnet/api/system.string.-ctor?view=net-10.0),
this uses the existing sequence abstraction instead of an array-only contract and
operates on graphemes instead of UTF-16 code units. Snapshot allocation and scanning
indexes are costs, not a claimed performance improvement. See the
[checked sample and limitations](experiments/string-sequence/README.md).

The author also asked about Iterable<char>, possibly with an explicit count.
Assistant recommendation: keep Sequence for this slice; a later growable-buffer
constructor could consume Iterable without a count. An explicit count would require
choosing between an exact-length invariant, prefix operation and capacity hint.
No Iterable overload or count parameter has been added or approved.

Author clarification: after the semantics slices, let real cases drive API additions
and behavior. Revisit inexpensive choices as evidence emerges; do not select an
Iterable construction contract ahead of a concrete need. The .NET 10 constructor
reference above was consulted on 2026-09-24. The executable comparison in
`experiments/string-storage/dotnet` checks array copying, empty input and UTF-16
length; it does not imply identical Unicode units between the platforms.

## Shared-owner identity — development, 2026-09-24

The author accepted the next bounded semantics slice: reference comparison and
identity/base hashing together. String now identifies its immutable shared text owner,
not its temporary Object/interface wrapper. Aliases retain identity through conversions,
fields, arrays, erasure, ToString, GC and host results. Independently constructed equal
text remains distinct, including empty strings. Literal interning is not introduced;
callers must not depend on identity of separately evaluated literals or text producers.

ReferenceEquals compares owner pointers internally without exposing addresses.
Explicit Object base Equals uses reference identity; virtual String Equals/GetHashCode
retain exact content behavior. Each shared owner carries a process-local 32-bit hash
seed assigned with a relaxed atomic counter. The existing integer mixer produces its
identity hash. Counter wrap and collisions are allowed: neither the seed nor hash is
used to determine equality. No address, persistent ID or cryptographic guarantee is
provided. Host clones retain the seed after their originating heap is destroyed.
Converting to owned Rust text and reconstructing StringValue creates a new owner.
Worker text serialization likewise does not transport identity across isolates.

The previous Arc<String> becomes an Arc of text plus its hash seed. This adds one
atomic operation per owner creation and payload storage/padding; cloning remains a
shared-owner operation. A lazily allocated hash would avoid eager assignment but adds
synchronization/state; address-derived hashes would expose allocation details. This
simple internal choice remains replaceable. No performance improvement is claimed.

Comparison sources reviewed 2026-09-24:
[.NET 10 ReferenceEquals](https://learn.microsoft.com/en-us/dotnet/api/system.object.referenceequals?view=net-10.0)
separates reference identity from contents;
[RuntimeHelpers.GetHashCode](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.compilerservices.runtimehelpers.gethashcode?view=net-10.0)
provides identity-oriented hashing independent of overrides. neoCLR uses its existing
explicit Object base-call path rather than adding that helper API. The extended .NET
probe checks alias hashes before/after GC without requiring unequal hashes for distinct
objects. UTF-8 storage and lack of interning remain differences from the CLR model.

Validation uses the expanded String Object Raven sample, Object identity/content
regressions, and private conversion/GC/host/teardown checks. Compiler metadata, emission
and Runtime Contract settings are unchanged; only runtime behavior and documentation
change. The next checkpoint is a bounded Object/value consistency review, with later
API additions driven by real application cases.

Validation outcome: ten String ownership/storage tests plus a forced identity-hash
collision test pass. Object equality/identity validation covers 30 cases: 29 passed
in the suite, and the new alias/base-call case passed its focused rerun after fixing
local-declaration ordering in the test fixture. The Raven sample reclaimed 446
allocations across ten collections; the .NET baseline, API snapshot and 522-page
website build pass. No publication or SDK release is performed.


## Explicit interning exploration — 2026-09-24

Following the author's request, the [repeated-identifier experiment](experiments/string-interning/README.md)
uses a test-only bounded pool over existing String owners. Four tests cover canonical
returns, unchanged prior references, exact text, quotas, separate pools and lifetime
through GC/host retention. The .NET 10 comparison passes. Strong scoped retention is
viable, but production ownership (execution or runtime session), exhaustion behavior
and API exposure remain undecided. No automatic interning or public method is added.


## Execution-owned interning — development, 2026-09-24

The subsequent integration selects one strong pool per interpreter execution. This
uses an existing lifetime boundary: LoadedProgram is immutable metadata, while each
host invocation and isolated worker creates mutable execution state. A shared host
session would introduce a new abstraction and retention policy without a current case
requiring it. Repeated-invocation checks contrast independent equal inputs with inputs
whose owner the host already shares; pool separation does not undo normal aliasing.

String.Intern now returns the canonical owner within that execution. It does not
rewrite earlier references, normalize text or intern literals automatically. Entry and
unique UTF-8 payload quotas use the existing terminal resource-budget model, with
InternPoolLimitExceeded rather than a recoverable domain Result. Defaults are 4096
entries/1 MiB; overhead, inputs and total memory remain separate accounting questions.
The pool drops on completion, faults or host cancellation; returned Strings retain
ordinary ownership afterward. Workers have independent inherited limits. A future
suspended execution must retain its pool as execution state, not create a new pool
on each resumption; runtime-owned suspension is not implemented by this slice.

This differs deliberately from .NET's longer-lived pool. It bounds retention with the
current runtime's lifecycle, but provides no canonical identity guarantee across host
invocations. Public API, quota and migration details are in the
[checked sample](experiments/string-interning/README.md). No IsInterned method or public
pool class is added. Runtime Contract configuration and Raven emission remain unchanged.
