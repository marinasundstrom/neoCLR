# Direction and migration

All planned capabilities and substantive revisions to implemented behavior follow
the [research and design comparison](design-research.md): establish the .NET/CLR
baseline, evaluate alternatives and justify improvements with evidence. Each roadmap
phase includes that research before its contract is settled.

The [platform backlog](platform-backlog.md) records the next broad capabilities:
binding immutability/readonly references, inheritance, nullable slots, enums/flags, delegates/lambdas, generic constraints, runtime async,
dynamic hooks, and a more useful fundamental library. Familiarity primarily means C#/.NET APIs and observable behavior, not matching
source syntax or runtime internals. Improve contracts without legacy constraints.

## Immediate preview priority

The [library-focused preview plan](library-preview.md) sets the immediate order:
shared type relationships and interface inheritance, class inheritance and virtual
behavior, then foundational library contracts and practical Neo programs. Simple
text/console improvements can progress alongside that groundwork. The long-term aim
is a library comparable in role and breadth to the .NET BCL, adapted to neoCLR; the
next preview promises a bounded useful subset. The broader exploration order below
is subordinate to these end-to-end needs.

## Groundwork checkpoint

The [runtime groundwork review](runtime-groundwork-review.md) led to implemented
[stored/returned readonly signatures](readonly-storage.md). Immutable bindings remain
a language responsibility; runtime-protected slots are not on the immediate track.
[Interface inheritance](interface-inheritance.md) now provides transitive contracts,
base views and reflection. [Inherited value layout and BaseType](inherited-layout.md) establish the first
record-base metadata/storage slice. [Base-reference views](base-views.md) preserve
complete owners. [Inherited methods, virtual dispatch and abstract classes](class-dispatch.md)
are now implemented. [Constructor chaining](constructor-chaining.md) and the
[MemberInfo descriptor hierarchy](reflection-hierarchy.md) exercise these foundations.
[Inherited class interface implementations and virtual override selection](class-interface-dispatch.md)
and [explicit interface implementations](explicit-interfaces.md) are implemented.
[Default interface bodies](default-interface-implementations.md) now execute in the
runtime and Neo. [Generic free/static functions](function-generics.md) now supply
an independent callable-definition building block. [Ordinary Neo classes and default(T)](classes-and-defaults.md)
now project construction/default rules without adding nullable signatures. The
[delegate contract and executable groundwork](delegate-contract.md) are recorded.
Typed delegates, the Func family and a ForEach consumer are now implemented; runtime nullability
and defaultability remain separate foundations for broader automatic initialization. Languages will build function values and lambdas on delegates;
ordinary functions will not become a separate runtime object model.
Applying default bodies to a useful library capability remains planned;
see the [reflection/interface plan](reflection-hierarchy-plan.md). Activation ownership, external roots and cleanup need decisions before
escaping callbacks or suspension.

## Projected exploration order

This is a proposed task sequence, not a release schedule. Each exploration should
produce a reviewed contract and a bounded implementation plan before growing syntax
or framework surface. Reorder when an end-to-end program exposes a stronger need.
The completed memory and Neo foundations remain the starting point.

| Order | Projected tasks | Exit criterion / demonstration |
| --- | --- | --- |
| 1. Reference access contracts | Preserve readonly permissions in signatures, storage, calls and returns, with verifier/reflection support and Neo projection. Keep immutable bindings in the language. | Raw IL cannot upgrade readonly access; locals remain replaceable under their declared signature; aliases and reference-containing values follow documented rules. |
| 2. Inheritance and object model | First establish transitive interface inheritance and shared type relationships; then specify base metadata, field layout, construction order, virtual slots, override validation and base-value copy rules. Implement one inheritance chain, base-reference conversion, full-owner GC tracing and reflection of inherited members. | A derived value can be used locally and on the managed heap, dispatched through a base reference, and inspected without losing derived identity or bypassing lifetime checks. |
| 3. Nullable signatures and storage | Follow the [explicit null-state direction](nullability.md): non-nullable defaults, nullable values/references by signature, Option for optionality, distinct uninitialized storage, conversions and checked access. Add metadata, runtime storage, verifier/GC rules and a small Neo projection. | Store, copy, clear and inspect a genuinely nullable slot; absent access fails predictably, and clearing a heap reference removes its GC edge. |
| 4. Enums and flags | Specify underlying integral types, enum identity, named constants, a flags designation, conversions and unknown-value rules. Add metadata, typed bitwise operations, reflection/formatting and a bounded Neo projection. | A typed options value combines and tests flags, round-trips through an artifact, and formats named and unnamed combinations according to a documented contract. |
| 5. Generic constraints | Specify base/interface constraints plus not-null, not-void and not-reference scopes. Implement metadata/substitution validation and negative cases at loading and relevant invocation boundaries; project a bounded subset in Neo. | A small generic API accepts valid arguments and rejects invalid ones consistently from source and IL/artifacts. |
| 6. Managed delegates and lambdas | Follow the [delegate direction](delegates.md): a shared runtime callable abstraction, with language function values/lambdas built upon it. Typed static and heap-bound delegates, Func<Void> and ForEach are implemented. Next specify C#-style closure environments, escaping closure ownership, equality and optional multicast semantics. Evaluate function pointers only where needed for invocation or interop. | A callback retains a managed receiver or eligible capture, runs through a typed delegate, and cannot retain a dead frame reference. |
| 7. Runtime suspension and async | Specify suspended activation ownership, references across suspension, resume/completion, cancellation and Fault/error propagation. Implement a minimal suspension primitive and evaluate a Task-like library/Neo projection. | One suspend/resume program preserves live roots, rejects invalid lifetimes, reports completion/cancellation, and exposes suspended state in the debugger. |
| 8. Dynamic binding hooks | Choose a concrete dynamic-object use case. Specify operations, hook discovery, lookup/fallback, access checks and cache invalidation; then implement a bounded binder and source demonstration. | A hooked member operation has predictable success and missing-member behavior while retaining runtime type and lifetime checks. |

Enums and flags can move earlier as an independent bounded slice; they need not
wait for an Object hierarchy.

Fundamental framework work runs throughout this sequence. With inheritance, decide
which useful base types need Object and establish Equals/GetHashCode/ToString contracts.
With nullability and constraints, exercise collection and optional-value APIs. With
delegates, add a real callback consumer; with async, choose a bounded I/O or host
completion scenario. Add text, collections, streams/files and other framework classes
as those programs need them, with consistent APIs rather than isolated demonstrations.

The first [readonly input-parameter implementation](readonly-parameters.md) is complete;
readonly receivers and the initial collection-getter review are also complete.
[Stored/returned readonly signatures](readonly-storage.md) are also implemented.
The [mutability decision](mutability.md) keeps binding immutability in Neo while
reference permissions remain runtime contracts. Inheritance and capture rules must
preserve that shared permission model.

Keep these early decisions explicit:

- Explore inheritance and nullability together before freezing shared metadata or
  assignability rules, even if their first implementations are separate slices.
- Define constraint semantics before exposing broad generic declarations in Neo.
- Establish closure and suspended-activation lifetimes before escaping lambdas or await.
- Keep dynamic binding hooks distinct from ordinary virtual/interface dispatch.
- At each slice, update runtime enforcement, applicable verifier/GC/reflection/debugger
  support, Neo examples, API documentation and the changelog. Record unsupported cases.

The runtime may own more of these mechanisms than the CLR does when that provides
consistent behavior to every frontend. Choose that boundary deliberately; familiar
API behavior does not prescribe the implementation. The [platform backlog](platform-backlog.md)
contains the unresolved choices and links to the existing design documents.

## Existing foundations and longer-term direction

Current sequence: the direct managed-reference/GC foundation is implemented; select
the next slices around a concrete end-to-end scenario. The [optional object hierarchy](object-hierarchy.md)
is a planned subsequent area. Adapt memory
layout and base-reference tracing during inheritance work; keep allocation mode
independent of inheritance and value equality independent of reference access.
The [Neo concept language](neo.md) now supplies the first source-to-runtime scenario.
Keep this small companion compiler updated alongside neoCLR for testing and
explanation, without making a full-fledged compiler a current goal. The
[Neo slice plan](neo-roadmap.md) records completed control flow, union-aware matching
and the bounded console calculator, with the next platform areas kept separate.
Future [library/compiler bootstrapping](neo-bootstrapping.md) is an exercise to pursue
only as the required language and library capabilities become useful.

The platform name is undecided; neoCLR names the runtime only. The existing code
is a small semantic testbed. It is not a commitment to Rust for every component,
JSON for distribution, or the exact instruction extensions used here.

The first publication target is defined in the [public source Preview 1 plan](preview-1.md).
Its scope is grounded in six [Raven-like programs and IL mappings](preview-1-programs.md),
with hand-authored IL acceptance fixtures retained alongside the growing Neo subset.
Use its required capabilities and release gates to prioritize work; the broader
directions below are not all prerequisites for the first release.

The unifying direction is [values with explicit capabilities](memory-model.md#values-and-explicit-capabilities):
reference access and ownership are choices layered on typed values. Runtime reference
tracking and optional verification provide enforceable contracts without requiring
Rust-style exclusive borrowing from every language targeting the platform.

## Architectural targets and next priorities

Interpretation, JIT compilation, and native AOT are platform-wide architectural
requirements. Embedding and a future high-level language are additional consumers
of the same semantic model. See [execution architecture](execution-architecture.md)
for shared contracts, capability boundaries, unresolved choices, and staged experiments.
[Closed call-graph analysis](reachability.md) now provides bounded traversal of explicit
roots and closed generic calls for backend planning. [Explicit target layout](target-layout.md) is now available; native compilation still needs
layout closure and opcode/ABI capability checks. [Runtime-service planning](runtime-services.md)
now reports direct service uses and compares them with a supplied service set.
Only interpretation is implemented today; this does not make interpreter internals
the permanent platform ABI.

Module-local function identities and generic call binding are now implemented; see
[function identities](member-identities.md). [Type definition rows and closed signature keys](type-identities.md)
are also implemented, along with [exact revision labels and pins](module-revisions.md).
Content provenance and general module-scoped resolution remain shared foundations. [Scoped operands](scoped-types.md)
now check origin, while duplicate type names still require new internal keys.
[LoadedProgram](loaded-program.md)
now shares a prepared metadata snapshot across execution and analysis, with
[explicit module sets](module-sets.md) supporting additional libraries. Optional
[direct module reference lists](module-references.md) constrain metadata uses while
retaining legacy compatibility. Invocation,
target-layout, runtime-service, and Fault boundaries
then support a minimal hosting experiment and an early native AOT experiment.
[Static and instance invocation](invocation.md) is now available as a Rust embedding subset
with exact primitive and [validated record inputs](record-inputs.md), explicitly copied
receivers, [validated ordinary Option/Result inputs](union-inputs.md), and fresh state.
[Cooperative cancellation](cancellation.md) now supports stopping interpreter execution
from the host. Guest addressed mutation and retaining T& locals/returns are
implemented with managed field addresses and runtime checks. A returned reference
may target storage owned by an active outer frame or the managed heap; current-frame
escapes are rejected. Heap-backed host results can be inspected through their owning
execution, while reference inputs across executions remain unsupported. Neo targets
the same metadata/IL and may enable incremental library migration.
Native backend/code-sharing choices remain open; no hidden fallback or
universal ownership policy is implied.

## Current priority: a small runnable platform

The implemented foundation is summarized above and in the
[library foundation](#implemented-library-foundation). Before expanding it, keep
the following decisions explicit. These are proposed follow-up work, not additions
to the published Preview 1 release gates.

| Decision | Why it matters next | Recommended next step |
| --- | --- | --- |
| Payload layout and ownership | String, errors and ordinary union carriers have no native layout, so the native Array buffer cannot hold them; managed ArrayList now can | Specify copy, replacement, release and active-payload rules; prove nested Result and String collection workloads before retiring System.Value |
| Scoped type identity | Scoped source operands currently normalize to unique names; colliding names cannot coexist across modules | Carry resolved definition identities through signatures and caches before general loading or cross-build compilation caches |
| Verification and required Faults | Verification is optional and pointer side tables include prototype diagnostics | Classify required checks, verified preconditions and optional diagnostics before an optimized backend; decide which execution profiles require verification |
| Managed reference extensions | Field paths, heap/frame references and readonly signatures are implemented; richer type relationships and lifetime analysis remain | Preserve access and owner identity through future base projections, nullable signatures and captures |
| Text APIs | UTF-8 bytes, UTF-16 code units and user-visible text elements need distinct contracts | Set indexing and decoding contracts before introducing Length/indexers; add a byte-to-text workload with explicit decoding errors |
| Native execution and hosting | Call graphs and target layouts are available, but they do not establish ABI or complete backend support | Validate layout/opcode closure, Fault propagation, cancellation and value lifetimes in a small AOT executable/export experiment |
| Frontend scope | Neo exercises library ergonomics against the existing IL | Keep the concept compiler and executable samples updated with runtime contracts |

Prioritize the storage/lifetime design and a small frontend workload over broad new
library APIs. Keep binary round-trip and AOT experiments bounded so they test the
shared contracts before any public ABI is frozen. General OOP, networking and async
remain future work. [Tracing GC](garbage-collection.md) manages the prototype heap,
including cycles, with direct heap-backed T&. Advanced collector policies remain planned.

The [lifecycle direction](lifecycle.md) develops the next storage foundation:
GC-managed heap references, stack-backed byref calls and separate value cleanup rules.
Preserve values by default and explicit reference passing; the runtime handles
retention and validity without manual reference management. Prioritize managed
reference copying/release and safe escape before user destructor execution.
[Clonable<T>](cloning.md) now provides explicit cloning independently of ordinary
value copies. The proposal's Fault cleanup limits remain recommendations, not an
implemented destruction guarantee.

Ordinary [Disposable and Closable<E>](disposal.md) interfaces now provide explicit
cleanup and fallible completion. The draft sample tests value-state changes through
borrowed receivers; resource-owning storage and automatic cleanup remain the next
lifecycle work, not behavior granted by interface conformance.

The first milestone is building and running simple programs without extensive OOP.
Prioritize the complete path from assembler through metadata/IL, runtime library,
execution, recoverable Error results, and unrecoverable Fault diagnostics. Add core
mechanics when these programs need them; broader OOP and backend features can wait.

Showing what the platform can do is a priority. Aim for small but functionality-rich
applications, not only isolated opcode examples. A broad runtime library is not a
prerequisite: a narrow host integration plus platform-written library code can provide
a useful end-to-end demonstration. Select implementation slices by the applications
they enable, while keeping their contracts usable by future compilers and backends.

Use the following implementation method for each slice:

1. Select a small executable program and identify what the existing IL and library
   cannot express. Prefer composing existing capabilities when they suffice.
2. Separate a missing execution or metadata fundamental from library policy and an
   operation that requires the host. Extend the appropriate layer only as needed.
3. Implement library logic in platform IL, using explicit host bindings for the
   necessary external operations. Record temporary bootstrap helpers and the missing
   fundamentals that would let their logic move into platform code.
4. Validate the contract with the runnable program and focused failure cases, document
   it, and commit the bounded slice. Let demonstrated needs justify later abstractions.

A Stream-style API, general I/O hierarchy, or collection of new runtime intrinsics is
not a prerequisite for console I/O. Do not infer that every familiar library method
needs its own permanent VM service. The goal is enough fundamentals to implement useful
libraries, not merely a growing set of host-implemented methods behind IL wrappers.

The immediate demonstration set should include:

- HelloWorld, arithmetic, branches, free functions, and explicit pointer allocation.
- Primitive-backed System types with a small useful method surface. System.Int32 already
  demonstrates Parse, Divide, ToString and Equals. Basic interfaces and read-only type
  inspection are implemented; broader object-model features remain later work.
- [System.Array<T> buffer descriptors](arrays-and-pointers.md) now provide explicit
  allocation/free, initialization, length, and checked access in ordinary library IL.
  Descriptor copies alias storage; owned array values and broader element types remain
  separate work. Empty and Void-element buffers are supported.
- A usable String API on the existing UTF-8 String value/type. [Initial String members](text-model.md)
  now provide concatenation, ordinal equality, emptiness, explicit UTF-8 byte count, and
  checked byte slicing with Result errors. General indexing and decoding remain separate.
- Recoverable failures through Result and terminal runtime/system Faults with preserved
  stack traces. [Initial Error methods](errors.md) now construct and expose messages;
  a runnable sample reports parse and domain errors and continues. Error handling is
  a milestone of its own, not incidental plumbing.

[Owned Fault snapshots](stack-traces.md) are implemented. Guest StackTrace/StackFrame
integration remains an intended runtime-library capability; source-line resolution and
rich formatting can follow the core programs. [Explicit target layout](target-layout.md)
now separates storage calculations from the host. This does not require starting native
code generation before the interpreter/library demonstration is coherent.

## Console fundamentals and next text requirements

The [minimal console boundary](console-io.md) now provides raw byte input, EOF as
ordinary absence, recoverable input errors, and immediate output through a host-supplied
console. An interactive IL program parses ASCII digits and computes a result. General
line reading is deferred until the necessary byte/string operations can support its
implementation in platform code. The file-input demonstration below is a completed
bounded experiment, not a reason to move on to networking.

Continue from concrete programs to identify missing byte/string fundamentals. Defer
general streams, buffering frameworks, and async abstractions. The current byte-read
and line-output boundary is not a commitment to a complete console or I/O API design.

## Demonstrations and later external capabilities

The first integration now provides [bounded UTF-8 text-file reading](file-input.md):
a small application reads input, parses values, reports recoverable errors, and prints
a computed result. System.IO.File.ReadAllText is a platform-IL wrapper over an explicit
blocking host service. It exercises external data, strings, Result handling, and library
composition without requiring a general stream hierarchy.

Implement only the missing mechanics needed by the selected demo. Keep platform-facing
methods in library IL and define a narrow, explicit binding to host services. Decide
the input size limit, text decoding behavior, expected I/O Error results, and resource
cleanup before implementing the boundary. Demonstrate both successful execution and
an ordinary failure that the application handles. Document build/run commands and
test the service contract without depending on an external network.

Socket primitives followed by an HttpClient-style library are a distant possible
demonstration, after the basics and a more robust base class library are in place.
Build that in layers when useful: explicit socket operations and lifetimes first,
then protocol/library behavior. Blocking I/O, cancellation, buffer
transfer, and eventual async support need deliberate contracts. Networking is not
the next mandatory feature, and runtime async or extensive OOP need not block a
simpler useful integration.

Keep nullability exploration, broad reflection/OOP, .NET migration, and a complete
framework behind the immediate goal of demonstrable applications. Existing fault
diagnostics and explicit memory semantics remain part of every integration's contract.

## Candidate high-level compiler targets

A modified C# dialect or a subset of Raven are candidate frontends for neoCLR. No
language or compiler implementation is selected yet. The first frontend should compile
small programs against the implemented platform subset, producing the same metadata/IL
as the assembler. It need not wait for extensive OOP, a complete runtime library, or
compatibility with arbitrary existing .NET programs.

Start with primitive values, free/static functions, locals, control flow, calls, and
explicit Error/Fault behavior; include strings and arrays as their core contracts become
available. Match compiler-produced programs against equivalent handwritten IL samples.
Unsupported language features should produce clear diagnostics. The frontend must
respect neoCLR's real Void value, value semantics, explicit allocation, and Result/Option
contracts rather than silently inheriting incompatible source-platform behavior.

Use this frontend to implement a few runtime-library functions and types incrementally,
keeping the assembler available as the low-level authoring and validation tool. Compiler
self-hosting and a broad language feature set are not prerequisites.

Existing .NET code migration is a later milestone, after the relevant OOP and runtime
features exist. Familiar syntax and APIs help that future path but do not establish
semantic or binary compatibility. Select a real source library then, identify its
required features, and make every necessary semantic adaptation explicit. Preserve the
migration principles below without letting broad compatibility delay the initial
runnable platform and language subset.

## Strategy review and verifier foundation

Following the strategy review, the first [verifier pass](verification.md) is implemented.
It checks evaluation-stack types, operands, definite local initialization, returns,
and reachable fallthrough. It is explicit rather than mandatory. Stable member
identities and whole-value constructor verification are also implemented.
[Call-scoped references](reference-slots.md), output contracts and reference receivers
are implemented, with [documented verifier limits](verification.md).

The [construction proposal](construction-and-initialization.md) and
[addressed-access proposal](addressed-access.md) contain broader design discussions; implemented whole-slot references and receivers
are specified in [reference contracts](reference-slots.md), while the
[constructor subset](constructors.md) keeps its existing initialization model. A small high-level compiler can eventually
produce the same metadata/IL as the assembler, enabling incremental runtime-library
migration without requiring compiler self-hosting.

## Implemented foundation: heap allocation and pointers

The initial subset implements native allocation/free, layout, casts, byte offsets,
field addresses, and indirect loads/stores. Construction remains separate from
storage. See [heap and pointers](heap-and-pointers.md) for current checks and limits.
Native integers/address conversions are also implemented. Next pointer capabilities
include direct foreign memory access, broader P/Invoke marshalling, explicit field
offsets and broader ABI controls. ldloca/ldarga now address ordinary slots through
managed references; they do not expose those slots as native pointers. Sequential
record packing and minimum-size reservations are implemented. Frame-local byte allocation
(`localloc`) and typed/block memory initialization and copying are implemented. Checked numeric conversions are now implemented. Integer and floating-point
layouts, arithmetic, and indirect access are now available, along with a first
[native interop subset](native-interop.md). Diagnostic side tracking must not
become a compulsory ownership policy for the platform.

Automatic retention and destruction are outside this implemented pointer subset.
The next [managed-reference gate](managed-reference-implementation.md) extends
T&/ByRef; the current Ref arena may be replaced in a breaking preview revision.
Allocator/collector integration remains in [memory layers](memory-model.md) and
[allocation proposals](allocation-encoding.md).

## Implemented library foundation

Ordinary Option<T> and Result<T,TError> now use constructors, properties, restricted
representation, nested generic case types and ordinary calls/branches. The generic
metadata foundation and marker attributes are implemented. Format 4 removed the
old union-specific instructions and runtime categories; older artifacts require
reassembly. See [union conventions](union-convention.md).

TryGet overloads extract Some/None or Ok/Error case values into caller-provided
managed output slots. The metadata contract is out(true) Case&; successful direct
Boolean branches establish initialization. Console/file samples demonstrate this
pattern with typed recoverable errors. Raw-pointer TryGet variants were removed.

The carriers still use temporary [System.Value storage](value-storage.md). Its
retirement is a separate representation and lifetime migration, not an unfinished
union-opcode removal. The Preview 1 plan records the current retention boundary.

Basic interfaces support managed views and explicit receiver modes. List<T> is the
small collection contract; [Equatable<T>](equality.md) supplies Equals(T) for Int32,
String, System.Type and user-defined implementations. No automatic equality comparer,
hashing, interface variance or ownership abstraction is implied.

[Properties](properties.md), including indexers, and the implemented
[accessibility model](accessibility.md) remain sufficient for this preview. Broader
construction, inheritance, field-reference and access-model changes require their
own contracts and demonstrable need.

## Later milestones

1. `brfalse`, label-based `switch` tables, and equality/ordered comparison branches
   are implemented. Conditional branches
   support Boolean, integer, pointer, and prototype Ref operands. Compact constant and slot aliases are implemented; short branches remain pending.
   Stack-height joins, definite local initialization, reachable returns, and maximum
   stack analysis are implemented by the explicit verifier, along with typed stack
   states and instruction operand checks. Decide when verification becomes mandatory.
2. Arrays are promoted to the initial runnable milestone above; rectangular shapes,
   nonzero lower bounds, covariance, and richer collections remain later work.
3. Implement a CLI-based binary reader/writer for the supported subset, preserving
   standard table/heap/token and opcode encodings where semantics permit. Define
   versioned extensions only for required deviations; see [format direction](format-direction.md).
4. Build on the existing explicit module sets, typed errors and ordinary generic
   carriers. Extend loading and library capabilities when real programs require them,
   preserving explicit storage and lifetime contracts.
5. Basic borrowed interface dispatch and ArrayList<T>/List<T> are implemented.
   Extend them only with explicit receiver, lifetime and backend contracts; see
   [the interface subset](interfaces.md).
6. Build .NET metadata/IL inspection and translation for a supported subset, with
   actionable diagnostics for semantic differences.
7. Extend CLR-style T&/ByRef with GC-rooted heap references, safe escapes and value
   cleanup under the [managed-reference plan](managed-reference-implementation.md).
   Ref<T> is a historical proposal, not a required ownership wrapper. Evaluate native
   interop, concurrency and runtime async against these lifetime contracts.

The assembler must grow toward full platform expressiveness, with .NET ilasm as
the capability baseline; see [assembler design](assembler-design.md).

## Migration principles

Use [ECMA-335](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
metadata and instruction concepts as the baseline vocabulary. Preserve familiar
namespaces, signatures where meaning permits, and ordinary arithmetic/control-flow
structure. Each incompatible behavior needs an explicit mapping or diagnostic.

Prefer a source recompile path first. Imported .NET class instances generally need
an explicit identity/ownership representation to retain aliasing and lifetime; converting them to frame-owned
copies silently would change programs. Imported structs can often remain owned
values, but boxing, reflection, interface dispatch, and layout still need work.

Translate void-return calls with their changed stack effects: neoCLR produces a
real Void value. Insert an explicit discard when adapting a CIL caller that
expects no result. Introduce `Option` for APIs whose nullable values mean absence;
do not rewrite null tests mechanically when null is a deliberate reference state.

Exception-heavy APIs need signature and control-flow adaptation to `Result`,
including explicit resource cleanup. A translation tool must diagnose unsupported
handlers rather than discard them. Faults cannot stand in for ordinary recoverable
exceptions without changing the contract. A future external .NET bridge may catch
host exceptions at that boundary and return structured Errors, but guest exception
semantics should not leak in.

Array covariance, byrefs, unsafe pointer arithmetic, reflection, dynamic code,
finalizers, disposal, async state machines, and unchecked integer overflow all
require deliberate treatment. Ordinary `add`/`sub`/`mul` already retain wrapping semantics; checked `.ovf`
operations terminate with Faults rather than throw. Surface required changes early
through a compatibility report rather than promise binary execution.

No .NET importer, .NET source compiler, bridge, binary metadata writer, or compatibility
analyzer is implemented yet. A useful migration success criterion is a small real
library recompiling with localized, explained changes and equivalent observable
behavior in the supported subset.

## Deferred design question: declaration nullability

Explore explicit nullable annotations on locals, parameters, fields, and properties,
separate from type identity. Resolve enforcement, generic composition, and boundary
contracts before implementation; compiler/tooling-only enforcement is a candidate.
Use consistent annotations without class/struct rules or an automatic Nullable<T>
rewrite, while deciding actual null storage separately. Preserve Option<T> for semantic absence. See the
[nullability design questions](type-system.md#future-exploration-nullability-on-declarations).

## Deferred design question: inheritance openness

When inheritance is introduced, explicitly model whether a type permits derivation
and whether its hierarchy is closed to unlisted subtypes. Determine defaults, permitted
subtypes, module/version boundaries, and validation before exposing a closed-world
guarantee. See [inheritance policy](type-system.md#future-exploration-inheritance-openness-and-closed-hierarchies).
This does not expand the immediate ordinary-type and union-foundation slices.
