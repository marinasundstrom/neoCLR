# Runtime-library API plan

Planning baseline: 2026-09-13. Define the expected families now; implement coherent
subsets when a real program or compiler feature needs them. The aim is familiar C#
and other .NET-language ergonomics over neoCLR's documented differences, not a complete
BCL before the platform is useful. Follow the [platform direction](platform-direction.md).

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
| Collections | Arrays, ArrayList, Iterable/Iterator and prototype Where/Select/ToList | Dictionary/set-style lookup when an application needs keys; first specify equality/hashing, iteration and mutation contracts |
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

Use the existing order workflow as a baseline. First extend a small text-processing
example with Unicode scalar handling and explicit decoding failures. Add keyed lookup
only when that example needs it. Introduce buffer views when avoiding repeated copies
has a measured benefit. Test a clock through a deterministic date-dependent rule
before growing the calendar API. Select one bounded slice at a time; this is not a
mandate to implement every row now.

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
