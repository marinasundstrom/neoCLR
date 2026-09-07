# Familiar concepts, deliberate platform contracts

Use familiar .NET APIs and concepts wherever they fit neoCLR's intended model. We do
not need to reproduce CLR/.NET legacy structures, historical restrictions, or every
compatibility behavior. Deliberate improvements to structure and behavior are valid
platform decisions, not merely unavoidable exceptions to a compatibility requirement.
Familiarity is a goal independent of implementation:
a platform-written function, native runtime call, interpreter operation, and future
JIT implementation should expose the same documented consumer contract.

Prefer established namespaces, type/member names, overload families, parameter
order, and observable behavior when their contracts remain appropriate.
Implement a small coherent subset first instead of inventing unrelated substitutes
for missing framework functionality. A missing implementation remains a documented
gap, not a reason to redefine the API silently.

Intentional changes include Result-based recoverable errors, Option-based absence,
first-class Void, explicit ownership/sharing, free functions, and interface names
without the `I` convention. These need explicit API mappings. Implementation details
alone do not justify additional consumer-visible differences.

## Current examples

For each intentional departure, state the problem it solves, the resulting contract,
and the migration impact. Keep compatible behavior where useful, but do not add a
legacy abstraction solely because .NET exposes it. Preserve the small Preview 1 scope while
leaving room for better designs as the runtime develops.

| API | Familiar surface | Deliberate change or current limitation |
| --- | --- | --- |
| `System.Console.WriteLine(string/int32)` | Familiar name and overloads | Produces the real Void value; console buffering is a prototype limitation |
| `System.Int32.Parse(string)` | Familiar parsing entry point | Returns Result instead of throwing; culture and whitespace coverage remain incomplete |
| `System.Math.Abs(int32)` | Familiar name and numeric result on success | Returns Result for the overflow case |
| `System.Int32.Divide(int32,int32)` | Uses the familiar Int32 domain | Experimental neoCLR extension for recoverable division, not a claim of a matching .NET member |
| `neoCLR.Runtime.*` | Internal implementation boundary | Not intended as consumer-facing replacements for System APIs |

The current assembler models declaring types and static/instance methods alongside
free functions. Primitive members belong to canonical System type definitions.
Read-only receiver snapshots are a prototype limitation, not full .NET by-reference
receiver mechanics. Visibility, virtual/interface dispatch, complete signatures,
and assembly references remain to be added.

The Divide helper is a proof-of-concept extension whose final API location remains
open. Existing .NET APIs should be preferred when an appropriate equivalent exists.
BCL error/absence adaptations should be documented alongside behavior and migration
examples rather than buried in runtime implementation notes.

## Runtime implementation declarations

Use the recognizable `MethodImpl`/`InternalCall` metadata mechanism for functions
provided by native runtime code. The public library remains written for neoCLR and
calls declared helpers at an explicit boundary. A future source compiler should be
able to accept the familiar attribute form and lower it to the implementation flags.
Runtime binding is separate from public API naming and from native interop imports.
See [runtime library](runtime-library.md) for the implemented mapping and limits.

As APIs are added, tests should cover ordinary observable .NET behavior and the
intentional neoCLR adaptations separately. Do not promise compatibility for a
member until its supported inputs, outputs, errors, and other effects are defined.

## Collection namespace and naming

Place generic collections directly in System.Collections. Generic arity already
distinguishes type identities; a separate Generic namespace is unnecessary. The
preview implements System.Collections.ArrayList<T> for native-layout values, with
explicit allocation/release. A future list interface will use the name List<T>,
without an I prefix. No interface implementation is implied by the present type.
See [ArrayList](array-list.md) for the concrete API and current element limits.
