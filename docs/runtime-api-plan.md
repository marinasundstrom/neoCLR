# Runtime-library API plan

Planning baseline: 2026-09-13. Define the expected families now; implement coherent
subsets when a real program or compiler feature needs them. The aim is familiar C#
and other .NET-language ergonomics over neoCLR's documented differences, not a complete
BCL before the platform is useful. Follow the [platform direction](platform-direction.md).

The first [generic managed-array prototype](generic-managed-arrays.md) now runs in
the Raven profile, including declared Iterable<T>, indexed array loops and shared
extension-method use. NativeMemory replaces the old native descriptor without an
alias. The minimal counted/indexed collection prototype and its Map/filter/query
integration are now exercised by the [order workflow](raven-order-workflow.md#collection-integration-scenario-2026-09-13-source-slice); full compiler
projection of the generic array name and native conversion support remain bounded
follow-ups, not prerequisites for ordinary T[] use.

## Common platform contract and target capabilities (2026-09-14)

API planning should identify the core of neoCLR as a platform, not only a catalog
of useful classes. The author proposes using this work to determine the common
standard and which features or APIs may differ between targets. This is a design
objective; the current preview surface is not yet a normative platform standard.

For each API family, classify the proposed requirements and validate the boundary:

| Proposed category | Contract to establish |
| --- | --- |
| Common core | Required type semantics, metadata/IL contracts and minimum library APIs; identical observable guarantees across conforming targets |
| Optional capability | A coherent feature that a target can omit, with explicit dependencies, availability and compiler/runtime diagnostics |
| Host-specific service | A shared abstraction where useful, with documented host-dependent behavior, resources and implementations |

Possible subjects include basic type inspection versus dynamic invocation, file
access, clocks and native interop. These are candidates for classification, not a
settled decision about which APIs are mandatory. Different CPU architectures need
not imply different public APIs; constrained deployments may require different
capability sets. Keep those dimensions separate.

Compare the .NET distinction between a common API contract, target frameworks and
platform-specific APIs before selecting a neoCLR profile mechanism. Evaluate
reference surfaces, build-time capability validation and runtime discovery; do not
assume that a stub which fails at runtime is sufficient. Specify missing metadata,
unsupported capabilities and ordinary recoverable failures distinctly. Include
conformance tests for core guarantees and tests for both available and absent
capabilities. No new profile metadata or discovery API is implemented by this plan.

The [reflection-model review](reflection-model-review.md) is an initial case study.
Keep runtime implementation strategy separate from public API guarantees and
preserve familiar language ergonomics wherever possible.

## Immediate implementation priorities (2026-09-13)

The author prioritizes arrays, collection interfaces and basic implementations as
fundamental runtime-library building blocks. Deliver a small working prototype of
each necessary layer before expanding the wider API catalog. This supersedes the
earlier text-first implementation suggestion; the family table below is not ordered.

| Order | Bounded slice | Evidence needed before calling the slice usable |
| --- | --- | --- |
| 1 | Map ordinary arrays to the intended System.Array<T> definition | Existing T[] source and array IL still work; element type, invariant policy, core length/index access, member lookup and reflection agree on the generic shape; no wrapper allocation is required |
| 2 | Review and prototype the minimal iteration/read/mutation contracts | Complete member signatures, count/index and mutation capabilities, and clear treatment of fixed-size arrays; a Raven consumer reads through an interface. Reuse Iterable/Iterator where sound; do not select the whole proposed hierarchy by name alone |
| 3 | Adapt arrays and the existing ArrayList to that bounded contract | Ordinary indexing, iteration and growable-list operations work with class and value elements. An array does not claim unsupported Add/Remove behavior. Existing prototype queries continue to work |
| 4 | Add small keyed-lookup and uniqueness implementations | A HashMap/HashSet-style prototype serves a concrete application; key equality/hashing, duplicate behavior, lookup absence and mutation are explicit. Prefer existing Result/Option conventions over preserving a desired variance annotation |
| 5 | Extend text/encoding and injectable-clock APIs | Build on the collection foundation with small parsing/file and deterministic-time scenarios; expand only the operations those examples need |

Generic variance is supporting work for step 2, not a prerequisite for every useful
collection prototype. Add it only after validating declaration positions, metadata
mapping and dispatch; initially invariant contracts remain useful. The existing
[read-only adapter experiment](experiments/readonly-views/README.md) is evidence for
an API shape, not completion of steps 1–3.

Keep the [collection taxonomy review](collection-contracts.md) open. Linked lists,
immutable/frozen families, a complete comparer framework and full query coverage are
not prerequisites for this initial foundation. Do not add a concrete family without
a distinct operation or guarantee to demonstrate. Correctness fixes and already
recorded release/debugging requirements remain in scope alongside this ordering.

## Existing evidence and next additions

The [declaration inventory](experiments/raven-target/runtime-api-inventory.json) and
[Raven coverage audit](experiments/raven-target/runtime-api-coverage.json) are the
per-member source of truth. A declaration is not evidence that arbitrary use compiles
or runs. The following is a planning summary, not a new completeness claim.

| Family | Starting point | Next addition when justified by a consumer |
| --- | --- | --- |
| Primitive values and mathematics | Bounded numeric, Boolean, Char, parsing and Math APIs | Fill overload/behavior gaps exposed by normal Raven programs before adding unrelated utilities |
| Errors and absence | Result, Option, Void and propagation | Consistent typed failures and useful error context across new APIs; preserve ordinary call ergonomics |
| Text and encoding | UTF-8 storage, explicit byte counting/slicing, ordinal operations and Char predicates | Scalar iteration and validated construction; strict UTF-8/UTF-16 conversion; settle indexing before adding ambiguous Length/search-offset APIs |
| Collections | Arrays, ArrayList, Iterable/Iterator and prototype Where/Select/ToList | Follow the immediate priorities above: generic array mapping, minimal contracts, existing implementation alignment, then keyed lookup/uniqueness |
| Files and console | Bounded text-file and console operations with Result outcomes | Byte I/O and streaming only when whole-file processing is insufficient; define resource cleanup before adding long-lived readers/writers |
| Date/time | Date, Time, LocalDateTime and local-clock acquisition | An injectable clock when tests need controlled time; duration/instant and timezone work as scenarios require, with no immediate globalization expansion |
| Metadata and reflection | Type/member introspection and a bounded reflection surface | Fill discovery gaps needed by tools and serialization; dynamic invocation requires separate access/type/fault contracts |
| Callbacks | Func-family callbacks and Raven closures | Evaluate function types against existing delegate lowering; do not block callbacks on that decision |
| Buffers and resource lifetime | Managed arrays/byrefs and existing native facilities | A bounded view for parsers; then safe retention/cleanup and pinning contracts for streaming/native consumers |
| Async and diagnostics | Synchronous runtime, existing GC observations and terminal debugger | Task<Void>/suspension only with a concrete async consumer; retain Raven debugging as a separately scoped release requirement |

These families resemble the roles of .NET System, Collections, Text, IO and diagnostic
APIs. Reuse appropriate names and behavior. A candidate Dictionary, Rune, stream or
clock abstraction is not implemented merely because it appears here.

## Implement by scenario

Use the existing order workflow as a baseline. First exercise ordinary arrays and
ArrayList through the selected minimal interfaces. Extend order lookup and duplicate
checking to justify keyed collections, then add text-processing and controlled-time
examples. Select one bounded slice at a time; this is not a mandate to implement
every type in the family table or finish the entire collection taxonomy first.

Every addition needs a consumer-facing contract, .NET comparison, supported overloads,
error/absence behavior, a Raven sample and direct runtime checks where safety depends
on enforcement. Include editor discovery, packaged execution and the coverage audit. For substantive
API design, review relevant other platforms, .NET feedback and alternative libraries
under the [broader research scope](design-research.md#broader-api-review-scope-2026-09-13).
Do not require manual dereferencing, ownership bookkeeping or special allocations for
ordinary class/array usage. Method calls, properties, indexers, loops, generics and
callbacks should preserve the familiar experience wherever the contract permits.

## Which layer owns it?

Public System APIs belong in the runtime library where ordinary IL suffices. Use
runtime services for representation access, GC, native boundaries, lifetime enforcement
or other facilities library code cannot express. Keep those implementation entry points
behind the public API. Add an opcode only if existing operations cannot state the
necessary contract; a host-backed prototype helper is not automatically a permanent
instruction. Compiler work should primarily provide syntax, inference, flow analysis
and target-contract mapping. Safety rules promised across languages must also survive
handwritten IL and metadata, not depend solely on Raven diagnostics.


### First Map prototype and remaining work (2026-09-13)

The author selected Map<K,V> and a default implementation corresponding to .NET
Dictionary<TKey,TValue>. The first [Map prototype](map-contracts.md) now implements
invariant Map/MutableMap and HashMap in the Raven profile: Option lookup, Boolean
TryAdd, Set, count and snapshot keys. Hashing/equality callbacks are required; no
universal default comparer policy has been chosen. Storage and algorithms use
existing managed library mechanisms. Raven needs no compiler change for this slice.

Follow-up Map work must evaluate a paired comparer and defaults, uniform null-key
policy, removal and pair iteration before calling this a general Dictionary
replacement. The prototype deliberately exposes these gaps. Keep those decisions
separate from the following LINQ terminal outcome slice.


### First LINQ terminal slice (2026-09-13)

Implemented First/Last returning Option<T>, and Single returning
Result<T,SingleError> with Empty and Multiple cases, in the Raven profile.
Filtering composes through Where; the [query API](raven-query-api.md) records the
.NET comparison, normal-outcome disposal and terminal-fault limits. Predicate overloads were added
in a subsequent slice; aggregation and specialized paths remain open. This implementation is
separate from Map and the subsequently completed ArrayList filtering work.


### ArrayList filtering prototype (2026-09-13)

Implemented separately after LINQ terminals: direct-storage Find/FindLast and
FindIndex/FindLastIndex return Option; Exists and TrueForAll return Boolean;
FindAll returns an independent shallow list. See the [contract and migration](arraylist-filtering.md).
FindIndex changes from -1 absence to Option<Int32>. No Result is needed for these
complete outcomes; callback faults remain runtime faults. Range overloads,
RemoveAll and fallible predicates need a concrete scenario before expansion.

The author clarified the usage guideline: prefer suitable built-in operations for
concrete collection types to avoid query-object allocations; use LINQ through
interfaces and for composition. Specializing LINQ is explicitly allowed. Measure
allocations and describe runtime speed separately. Scalar ArrayList searches now
scan captured storage directly, without an iterator or query object.

### Next implementation track: Raven-authored runtime library (2026-09-13)

The author directs migration now, before further library growth makes translation
larger. After a tagged checkpoint, prioritize the [Raven library build path](runtime-library.md#raven-as-the-library-source-language-2026-09-13):
stable library identities and bootstrap metadata, a small scalar port, then a generic
method with compiler/importer gaps resolved. Raven becomes the ordinary library
source language; neoIL remains available for low-level implementation and testing.
This implementation track takes precedence over adding unrelated API families.
Preserve existing contracts and migrate verified bodies incrementally.

### Revised priority: evaluate target support first (2026-09-14)

The author paused the Raven library migration. The scalar pilot is preserved locally;
it is not an adopted System implementation. Prioritize [Raven compiler stabilization
and consistent target support](raven-target-evaluation.md): review general fixes for
integration, unify target configuration, then establish metadata-driven importer and
library-body capabilities. This supersedes the immediate migration track above.
