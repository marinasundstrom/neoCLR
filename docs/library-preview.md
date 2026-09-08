# Library-focused next preview

Status: planned direction, recorded 2026-09-08. This does not announce a version,
release date or completed library equivalence.

The long-term goal is a foundational library comparable in role and breadth to the
.NET base class library: the types and services developers expect for ordinary
programs, with familiar names and behavior adapted to neoCLR's contracts. The next
preview should demonstrate meaningful progress toward that goal through useful
programs. A collection of isolated instruction demonstrations is insufficient.

## Runtime, library and Neo responsibilities

The runtime provides validated type relationships, dispatch, managed storage,
reference access and lifetime rules. The library defines reusable APIs over those
mechanisms: text, collections, I/O, equality, formatting and recoverable errors.
Neo projects those APIs through properties, indexers, automatic managed-reference
access and exhaustive union matching. Library contracts must remain usable from IL
and other frontends; Neo syntax is not the platform API contract.

Preserve familiar System names and member behavior where appropriate. Document
intentional differences: values by default, explicit managed references, Option
for domain absence, Result for recoverable failure, and interface names without I.
Do not require Object ancestry or heap allocation merely to use a library contract.
Immutable bindings remain a language feature. Explicit nullable type signatures
remain a separate planned storage capability, not a prerequisite for Option APIs.

## Starting point and gaps

The library already contains primitive types, String operations, Console and File
helpers, Option/Result, ArrayList/List, cleanup interfaces and type introspection.
These are bounded implementations, not complete equivalents of their .NET namesakes.
The calculator already demonstrates parse and division results, EOF and retries,
but manually reconstructs lines from Console.ReadByte. That work belongs in a useful
library API once its encoding, buffering and error contracts are defined.

Class inheritance and virtual overrides are missing foundations. Interface
inheritance also needs a deliberate contract; existing concrete-to-interface dispatch
is not a substitute for transitive interface relationships. Collections, readers
and streams should guide these mechanisms before their public hierarchies expand.

## Ordered slices

1. **Shared type relationships and interface inheritance.** The initial
   [interface slice](interface-inheritance.md) is implemented; general class/base
   relationships remain ahead. Specify and implement
   multiple base interfaces, generic substitution, transitive conformance and
   conversion between derived and base interface references. Preserve the complete
   owner, reference identity, readonly capability and lifetime through each view.
   Reject cycles, unresolved members and incompatible contracts. Resolve diamond
   inheritance by declaration identity; unrelated conflicting declarations must not
   be selected by traversal order. Define ambiguity and redeclaration rules before
   implementation. Exercise a small collection observation hierarchy through Neo,
   raw IL, artifacts and reflection. Default interface bodies and variance can wait.
2. **Class inheritance and virtual behavior.** The preliminary
   [inherited-layout slice](inherited-layout.md) is implemented; base views and
   virtual dispatch remain ahead. Build on the same relationships:
   optional single base class, inherited layout, construction, virtual slots,
   overrides and base references. Decide abstract/sealed rules, inherited interface
   implementations and base-value copying before exposing corresponding APIs. A
   base reference must keep the full derived owner alive and dispatch correctly
   without silently boxing or promoting a frame value. Implement the bounded Neo
   projection and debugger/reflection visibility together. See the
   [object-model direction](object-hierarchy.md).
3. **Fundamental library contracts.** Establish useful Object methods where an
   Object base is chosen, alongside equality/formatting contracts available to types
   without that base. Keep value equality/hashing separate from reference identity.
   Review collections against the inherited interfaces, including readonly receivers,
   reference elements, ownership and mutation. Identify naming migrations explicitly;
   the existing List interface and ArrayList implementation must not be silently
   advertised as exact .NET equivalents. Add text and line-input facilities with
   specified Unicode, EOF, input limits and recoverable-error behavior. Simple APIs
   that do not depend on inheritance may be implemented alongside slices 1–2.
4. **Practical examples and completion.** Simplify the calculator using line input;
   add a bounded text-data report that reads, parses, validates and collects results;
   add a polymorphic consumer that accepts a base/interface reference to both a
   frame-owned and a heap-owned implementation. Prefer ordinary library usage over
   artificial runtime probes. Add lookup returning Option when a real example needs
   it; a dictionary requires an equality/hash contract first. A reader/stream hierarchy
   is a candidate consumer of inheritance, not a commitment to an entire I/O stack.

This sequence brings object-oriented groundwork forward because library contracts
need it. It does not require finishing async, delegates, dynamic binding, nullable
storage or every generic constraint before improving the library. Enumeration and
callback APIs must state their missing dependencies instead of simulating them with
misleading sample-only interfaces.

## .NET comparison and provisional choices

Primary sources consulted 2026-09-08: current C# specification documentation and
.NET 10 API documentation. These establish language/API baselines; they are not a
completed study of CLI metadata or a pinned CoreCLR implementation.

- [C# interface inheritance](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/interfaces)
  permits multiple base interfaces. Reuse that familiar relationship model, while
  explicitly adapting conversions to managed reference views. Flattening member
  lists alone would lose declaration identity and obscure diamond conflicts.
- [.NET inheritance](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/object-oriented/inheritance)
  supplies a familiar single-class-base and override model. neoCLR deliberately
  separates inheritance from value/reference use and permits types without Object.
  This adds layout, copy and lifetime decisions rather than eliminating their cost.
- [Int32.TryParse](https://learn.microsoft.com/en-us/dotnet/api/system.int32.tryparse?view=net-10.0)
  represents success with a Boolean and an output parameter. neoCLR's existing
  Result-based Parse keeps either the value or typed failure in one result that Neo
  can match exhaustively. That improves explicit error composition but requires
  handling code and a documented result representation; it implies no speedup.
- [Dictionary.TryGetValue](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.dictionary-2.trygetvalue?view=net-10.0)
  is the baseline for a proposed Option-returning lookup. Option expresses absence
  independently of the payload's default value. Compare naming and migration costs
  before adding this API; do not change familiar indexer failure behavior silently.

.NET can also host option/result abstractions; the proposed advantage is a consistent
library convention and direct Neo support, not a claim that these patterns are
impossible on .NET. Runtime faults remain distinct from recoverable library errors.
Before implementing inheritance, extend this comparison with relevant CLI metadata,
small .NET probes and neoCLR positive/negative tests under the
[research workflow](design-research.md).

## Acceptance and presentation

Each slice updates API documentation, Neo projection, tests and the changelog.
Document receiver/argument access, copying or retention, errors, supported inputs and
unsupported cases. Test behavior from source and serialized artifacts; exercise
invalid relationship and readonly/lifetime bypasses through raw IL. Include generic
and diamond interface cases, inherited dispatch, GC retention and reflection tests.

Ship a walkthrough with exact run commands, deterministic sample input/output and
recoverable-error paths. Demonstrate that absent data, invalid data and runtime faults
are different outcomes. Explain stack/heap and reference choices where the program
actually needs them. Retain the existing calculator as a baseline until its replacement
has equivalent EOF, invalid-input and bounded-input coverage. Apply normal release
validation when a preview is prepared; this plan does not authorize publication.

The broader library backlog includes richer text, collections and enumeration,
readers/streams/files, time, environment and diagnostics, followed by callback and
async consumers as platform support arrives. Track these as coverage gaps, not as
features promised for this preview. Claim concrete capabilities and limits, not
production readiness or .NET binary/source compatibility.
