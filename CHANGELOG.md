# Changelog

Notable changes to neoCLR, grouped by release and development date. Every commit
updates the Unreleased section; related changes on the same date share an entry.
Published sections are frozen. See the [maintenance workflow](docs/changelog.md).

## Unreleased

### 2026-09-08

#### Added

- Added System.Comparable<T> with a readonly managed receiver and value input,
  plus scalar CompareTo implementations, including unsigned
  extremes and .NET-compatible floating-point NaN ordering. Added Iterable<T> and
  Iterator<T> with MoveNext/Current and inherited Disposable; List<T> now inherits
  Iterable<T>, requiring GetIterator from implementers. ArrayList iterators retain
  their initial managed buffer/extent and can outlive stack descriptors; writes to
  retained storage remain visible, unlike .NET List mutation invalidation. Added
  Neo conformance to bundled interfaces and inherited-library-interface projection,
  samples, tests and API/.NET comparison
  documentation. Fixed generic static calls being misidentified as delegate type
  names, and updated library body checks to recognize bodyless delegate contracts.
  No new opcode or artifact format; iterable for syntax remains planned.
  Planned the following release-focused fundamental API pass in library-preview.

- Neo ordinary class declarations with body fields, field initializers, bounded
  parameterless-constructor synthesis and implicit parameterless base chaining.
  Added default(T) over checked runtime initobj; defaults do not run constructors
  or invent null/invalid managed references. Records retain positional construction.
  Added samples, grammar, tests, a pinned .NET comparison and responsibility guidance:
  synthesis is language policy, storage/reference validity is runtime enforcement.
  Base-first initializer ordering deliberately differs from C#. Nullability remains
  planned; class/default become reserved words, with no opcode/artifact change.

- Added delegates as the shared runtime callable abstraction, with
  language function values/lambdas built upon them. Excluded a separate universal
  function-object runtime model. Added a typed-delegate contract/runtime audit, pinned
  .NET behavioral probes and runnable Neo interface-adapter lifetime/GC tests. The
  implementation now adds nominal generic IL delegates, checked delegate.bind and
  ordinary Invoke calls, with heap receiver retention, virtual/interface binding,
  reference/output checks, GC tracing, source debugging and separate binding edges
  in closed call graphs. Neo adds delegate declarations, explicit source method-group
  binding, contextual method-group conversion and function-style invocation of
  delegate expressions. Bundled Func supports zero to four inputs, including Void
  results instead of a separate Action family; Array.ForEach consumes Func<T,Void>.
  Added samples, grammar synchronization and regression coverage. New delegate
  artifacts require this runtime; Rust exhaustive enum matches need updating.
  Neo now lowers contextual lambdas to existing delegates and generated managed
  capture cells/environments, sharing mutable bindings across returned/nested closures
  and using fresh range-loop captures. Heap-reference validation remains enforced;
  out, uninitialized-local and constructor-this captures are rejected. Added a runnable
  closure sample, synchronized grammar, debugger/GC/lifetime tests and .NET capture
  probes. Generated neoCLR.Compiler names are reserved; captured storage incurs
  additional heap allocations. Natural lambda inference, stack-only closures,
  multicast, variance and nullable metadata remain later
  work. Aligned platform/Neo roadmaps and the compiler inference role.

- Generic free functions and static methods across runtime/IL and Neo, with independent
  method parameters, explicit type arguments, simultaneous owner/method substitution, verification,
  host invocation and distinct closed call graphs. Added regression tests and a
  pinned .NET comparison. Neo adds namespace-qualified functions, static members,
  argument-based type inference and explicit type arguments, with a runnable example,
  grammar and constructor-status guidance. Managed-reference arguments preserve
  existing lifetimes; invalid substituted shapes fault. Generic instance methods, constraints and guest
  generic-method reflection remain deferred. New metadata requires this runtime;
  existing artifacts remain compatible, while Rust metadata literals need new fields.

- Default interface bodies now execute in runtime/IL and Neo with class precedence,
  most-specific selection, diamond ambiguity checks, qualified replacements and
  reabstraction. Managed interface receivers retain original owners and readonly/output
  checks; verification, reflection, closed graphs, debugger/source traces, samples and
  tests cover defaults. Includes the contract audit and pinned .NET comparison probes.
  Updated older Neo regression expectations for abstract declarations and defaults.
  New body/replacement artifacts require this runtime; existing bodyless interfaces
  remain compatible. Output completion retains the existing runtime return check.

- Explicit interface implementations via runtime declaration-to-body mappings, IL
  .override directives and Neo `func Interface.Member` bodies. Added inherited and
  redeclared mappings, generic analysis, qualified private reflection names, examples,
  regression tests and a pinned .NET comparison. Bodies use managed receivers and
  remain separate from class virtual slots. New mapping artifacts require this runtime;
  IsVirtual and one inherited reimplementation edge deliberately differ from .NET,
  documented as preview choices. Default interface bodies remain planned.

- Interface implementations now inherit through class bases and dispatch mapped virtual
  members to concrete overrides, including through base views. Added generic target
  analysis, inherited GetInterfaces results, frame/heap examples, lifetime/readonly
  regression tests and a pinned .NET comparison. No artifact schema change; inherited
  interface programs previously rejected are now supported. Explicit interface
  implementations and default implementations are planned as separate follow-up slices.

- Reflection member descriptors now derive from an abstract MemberInfo base with
  shared Name/DeclaringType storage and internal chained constructors. Instance
  readers use readonly managed receivers; Neo resolves inherited bundled class
  members and borrows readonly temporaries without evaluating them twice. Added
  frame/heap reference examples, constructor/snapshot parity and GC tests, .NET
  comparison and migration guidance. Query results remain declared-only snapshots.
  Descriptor field order is preserved; receiver contracts, member rows and declaring
  owners change, so recompile applications against the matching System library.
  MethodBase and default interface implementations remain planned.

- Managed constructor chaining using byref .ctor receivers, call/newobj and Neo
  init declarations with explicit base initializers. One unpublished owner retains
  inherited and own fields; verifier/runtime checks enforce initialization, base
  completion and restricted receiver access. Added direct managed stfld writes,
  including reference fields, GC roots and debugger inspection of construction
  storage, examples, regression tests and a .NET comparison. Managed stfld produces
  Void; value-form stfld remains a record update. Old root value constructors remain
  supported; derived constructors require managed receivers. New constructor/field
  operands need this runtime revision. Reflection hierarchy migration remains next.

- Inherited managed-receiver methods, class virtual/override dispatch and abstract
  records/methods in Neo and IL. Base views select implementations using the complete
  concrete owner; readonly/output/return contracts are validated and abstract values
  cannot be instantiated or imported. Added Type.IsAbstract and MethodInfo virtual,
  override and abstract flags, examples, tests and a .NET comparison probe. Rust
  metadata literals require the new flags; new metadata/descriptor layouts require
  a matching runtime/System library. Generic class runtime dispatch works; closed
  generic class target inference remains explicitly unsupported. Planned reflection
  hierarchy migration after constructor chaining, and default interface implementations
  as a separate library-driven slice.

- Managed base-reference views through castclass and implicit/explicit Neo ancestor
  projections. Views preserve location identity, derived GetType, complete-owner GC
  lifetime and readonly access. ldfld now reads managed record references directly.
  Whole-value reads/writes/reset/out through projected views fault to prevent slicing;
  downcasts and native casts remain unsupported. Added source/artifact and unchecked
  runtime tests, examples and API/IL migration documentation. Rust Instruction
  matches must handle CastClass; new opcode artifacts require this runtime revision.

- Preliminary record-base metadata and inherited value layout through IL `.extends`
  and Neo record bases. Aggregate construction/defaulting includes inherited fields,
  generic bases substitute recursively, and field visibility retains its declaring
  owner. Added Type.BaseType as Option<Type>, a sample and regression tests. Base
  types with instance methods or implemented interfaces, native inherited layouts,
  base-reference conversions and virtual dispatch remain unsupported pending the
  next object-model slice; no implicit Object base or value slicing is introduced.
  Rust TypeDef literals now require the base field; older JSON artifacts default it
  to absent.

- Interface inheritance in IL and Neo: transitive generic contracts, diamond
  deduplication, base-interface reference projections, inherited method dispatch,
  load-time cycle/conflict checks and closed dispatch analysis. Managed views retain
  owner identity, GC lifetime and readonly access. Type.GetInterfaces now includes
  transitive bases; member enumeration stays declared-only. Added a runnable sample,
  artifact/runtime tests and design/migration documentation. Class inheritance,
  variance and default interface implementations remain planned.

- Planned a library-focused next preview toward .NET BCL familiarity, separating
  runtime/library API contracts from Neo projection. Prioritized shared type
  relationships, interface inheritance and class inheritance, followed by useful
  text/collection APIs and practical Option/Result examples; documented scope,
  comparison sources and validation gates. These additions are plans, not newly
  implemented inheritance or library APIs.

- Readonly managed input parameters in Neo and IL, enforced by live reference
  capabilities across frame/heap storage, derived addresses, copies and interface
  views. Restricted writes and writable forwarding fault even without verification.
  Added partial verifier diagnostics, ParameterInfo.IsReadOnly, debugger markers,
  metadata validation, examples and a reproducible .NET 10 comparison. Documented
  the const-reference analogy and JIT limits: readonly views may observe writes
  through other aliases and do not imply globally immutable memory. Readonly
  instance receivers now share this enforcement, including virtual interface dispatch,
  Neo readonly methods and MethodInfo.IsReadOnly. ArrayList Count/Capacity/Item
  getters and List Count/Item contracts permit readonly observation; external List
  implementations must update their getter receiver contracts. Reference storage
  signatures now include ReadOnlyByRef in locals, returns, fields, arrays and generic
  arguments, with readonly T& syntax in Neo and IL. Storage/call/return boundaries
  narrow writable inputs or reject readonly-to-writable mismatches; verifier joins
  preserve compatible readonly access. Type.IsReadOnly and qualified reflection
  signatures expose the contract. Unqualified writable destinations/results that
  previously carried restricted references now fail at the boundary. Reassemble
  external System artifacts for the expanded reflection descriptor layout; readonly
  metadata requires this runtime, and Neo now reserves the readonly keyword.

#### Changed

- Recorded explicit nullability as a planned type-signature characteristic across
  values and managed references, non-nullable by default. Null is a special state
  distinct from present zero/default payloads and uninitialized storage; Option
  remains preferred for domain optionality. Documented composition, clearing/GC
  obligations and .NET comparisons. No nullable behavior is implemented by this slice.

- Established a research-backed .NET/CLR comparison workflow for every roadmap
  capability and substantive revision of existing features. Added primary-source
  starting evidence, per-area research questions and decision/validation criteria;
  clarified that runtime mutability placement remains a candidate to evaluate.
  Added a code-backed runtime groundwork review after the readonly receiver milestone,
  covering storage/reference contracts, type relationships, nullable initialization,
  activation ownership, GC roots, cleanup and persistent state. Recorded proposed
  dependencies and acceptance cases, and paused feature work for that assessment
  before resuming the readonly signature implementation.
  The reviewed foundations are proposals, not newly implemented capabilities.
  Expanded the reference/storage proposal with recursive permission signatures,
  boundary narrowing/rejection, invariant containers, verifier joins and separate
  protected-slot initialization. Added a reproducible probe of the current
  verify-then-fault return/local gap and scoped acceptance cases.

- Planned runtime-enforced immutable storage and readonly reference/receiver
  capabilities, with Neo syntax and diagnostics above them. Documented their
  independence, shallow boundaries, alias checks and initialization/re-entry
  decisions; placed this foundation first in the exploration roadmap. The
  earlier protected-slot proposal is superseded: immutable bindings remain a language
  feature and are removed from the immediate runtime plan. Readonly reference
  contracts remain runtime-enforced, without making local bindings write-once.

- Recorded the planned platform backlog: inheritance, nullable slots, enums/flags, delegates and
  lambdas, generic constraints (including not-null/not-void/not-reference), runtime
  async, dynamic hooks and fundamental framework growth. Documented open contracts,
  projected roadmap tasks and exit criteria, including integral enum representation,
  typed flag operations and reflection/formatting decisions. Familiarity targets C#/.NET APIs and
  observable behavior rather than syntax or internals; these are plans,
  not implemented capabilities.

- Neo bracket syntax now projects bundled single-index Item getters and setters,
  including virtual interface dispatch and reference-valued elements. Collection
  samples use brackets instead of direct accessor calls; metadata and IL retain
  their accessor methods. Documented setter replacement and addressability rules.

- Neo now accepts checked local array extents (`let a: int[3] = [1, 2, 3]`) and
  managed heap initializer braces (`new int[3] { 1, 2, 3 }`). Nonempty
  lists require exact counts and evaluate once in order; they support element types
  without defaults. Local extent annotations lower to T[] with initialization checks;
  they do not introduce fixed-extent metadata types. Previous forms remain accepted.
  Updated the executable array example and synchronized grammar documentation.
  Samples omit optional empty braces for default-initialized arrays.


## 0.1.0-preview.2 — 2026-09-08

Source-only prerelease. See [release notes](docs/preview-2-release-notes.md).

Changes since [v0.1.0-preview.1](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.1).
Publication requires the exact-commit validation linked from the GitHub release.

### 2026-09-08

#### Fixed

- Source validation disables Git line-ending conversion when archiving, preserving
  notice hashes across Windows and Unix. Its text I/O explicitly uses UTF-8, and
  CI runs every platform job even if another job fails.

#### Added

- **Preliminary reference identity:** Neo ReferenceEquals and IL ref.eq compare
  initialized managed locations across frame/heap storage, concrete/interface views,
  and interior paths. They preserve value equality behavior and reject raw pointers;
  no Object root, boxing, native address or identity hash is introduced.

- **Neo output parameters:** unconditional `out name: Foo&` declarations, explicit
  `out destination` arguments, and typed uninitialized var locals now project the
  existing runtime contracts. Source/interface forwarding and conditional library
  TryGet calls are demonstrated end to end. Caller initialization is checked by the
  verifier; callee read/assignment obligations remain runtime-enforced.

- **Reflection introspection:** added live managed-reference GetType discovery through
  `ref.type`, including concrete interface targets and interior field types, without
  boxing or retaining inspected objects. Existing declared methods keep their dispatch.
  The collection example now checks its concrete implementation through an interface.
  System.Type now enumerates fields, methods and
  properties through independent FieldInfo, MethodInfo and PropertyInfo records.
  ParameterInfo exposes names, positions, types and output contracts. Queries support
  a documented BindingFlags subset, accessor metadata, closed generic substitution,
  implemented interfaces and array/reference/pointer shape inspection. Metadata
  queries preserve managed-reference signatures and do not invoke accessors or retain
  guest objects. Includes Neo/IL examples and an [API guide](docs/reflection.md).
- **Interactive terminal debugger:** launch with `neoclr debug` to inspect call
  frames, arguments, locals, evaluation stacks, managed heap objects, GC counters and
  tracked native memory. Supports pause/continue, breakpoints, IL step-into, source
  step-over, step-out and bounded live snapshots. Neo source mappings survive artifact
  round trips and appear in Fault traces. Includes a controller API for embedding.
  See the [debugger guide](docs/debugger.md).
- **Managed and frame-owned arrays:** T[] owns copied elements; T[]& references an
  array location. Added newarr, array.create, ldlen, ldelem, stelem and ldelema, with
  bounds/type checks, lifetime validation, GC tracing and payload budgets. Neo exposes
  array literals, fixed-length annotations, heap allocation, indexing, Length and
  element references. The native System.Array<T> buffer API remains separate.
  See [managed arrays](docs/managed-arrays.md).
- **Neo interface declarations and conversions:** records can implement interfaces,
  and existing managed references can convert to implemented interface views for
  virtual dispatch. Views preserve frame or heap provenance without boxing.
  See [Neo interfaces](docs/neo-interfaces.md).
- **Reproducible candidate validation:** added a source-archive validator that checks
  tracked membership and dependency notices, runs all test targets on a selected Rust
  toolchain, compares nine Neo source/artifact programs and exercises native interop.
  The six existing CI jobs now run the archive check and retain reports/checksums.
  Actual publication still requires successful evidence for the selected commit.
- **Release changelog workflow:** reconstructed the published baseline and subsequent
  development history. Added a per-commit update rule, same-date consolidation and
  protection of published entries, with repository instructions for future work.

#### Changed

- **Fresh Neo declaration storage:** repeated local declarations and compiler
  temporaries now reset their storage before initialization, allowing differently
  sized array and nested-record results on successive loop iterations. The new
  `local.reset` instruction faults while aliases to the old local remain live;
  ordinary assignment retains its existing shape and reference-preservation rules.

- **ArrayList<T> now uses a managed T[]& backing array.** It grows automatically and
  no longer requires Free. Checked `array.alloc T` reserves uninitialized capacity
  without inventing default elements. Generic reference arguments and reference array
  elements allow ArrayList<Foo&> to store Foo references; GC traces initialized slots.
  All collection instance methods use managed-reference receivers. Copying a list
  descriptor copies Count and shares its backing array until growth; an explicit
  ArrayList<T>& shares the whole mutable descriptor. See [ArrayList](docs/array-list.md).
- **Reference-aware library calls in Neo:** added closed generic static member calls
  and implicit reference conversions for implemented bundled interfaces. A complete
  Neo ArrayList<Counter&> example now creates its owner, shares references through List,
  mutates a referenced object and demonstrates independent copied Count state.
  Existing library receivers use their declared
  value/reference contract, and unique instance methods supply contextual parameter
  types, including reference arguments. Exact-signature overload selection remains
  the current subset. Added the [library API design guide](docs/api-design.md).
- **Cross-layer lifetime documentation:** consolidated runtime and Neo rules for
  frame ownership, heap reachability, interior references, automatic managed-reference
  access and checked returns. Clarified the distinction between reference binding
  lifetime, target lifetime and resource cleanup. Reconciled older generic-reference
  and GC-monitoring descriptions with implemented arrays and debugger inspection.
  See [managed-reference semantics](docs/managed-reference-semantics.md).

### 2026-09-07

#### Added

- **Tracing garbage collection:** a single-threaded, nonmoving mark-and-sweep
  collector reclaims unreachable heap graphs, including cycles. Active frames,
  returned values, interior references and interface views preserve the required
  roots. Added heap statistics, bounded collection-event history, and CLI
  `--gc-stats` / `--gc-events`. Native allocation/free remains explicit and separate.
  See [garbage collection](docs/garbage-collection.md).
- **Managed-reference locals and returns:** checked T& locals, heap-backed reference
  fields and returned references extend the earlier call-scoped model. Returning a
  reference into an active caller or managed heap is supported; returning an address
  into the current frame faults, including indirect escapes. References neither
  promote locals nor require manual invalidation. See [reference slots](docs/reference-slots.md).
- **Managed initobj destinations:** default-initialize supported typed frame slots
  and addressed fields without a constructor or extra allocation. Initialization can
  satisfy output contracts. Unsupported defaults fault rather than manufacturing
  null managed references. See [managed initialization](docs/managed-initialization.md).
- **Neo companion compiler:** a small Raven-inspired language compiles `.neo` source
  to neoCLR, with records, functions, value/reference parameters, field access and
  explicit managed heap allocation. Added if/else, while, integer-range for, loop,
  break/continue, union-aware match expressions/statements and typeof(T). Public
  read-only System properties project through ordinary getters. Includes a bounded
  console calculator, executable examples, [language guide](docs/neo.md) and
  [grammar](docs/neo-grammar.md). Neo is maintained alongside runtime development.
- **Explicit lifecycle APIs:** System.Clonable<T>.Clone(), System.Disposable.Dispose()
  and System.Closable<E>.Close() are ordinary managed-reference interface contracts.
  Clone is independent of value copying; Dispose/Close require explicit calls.
  Includes [cloning](docs/cloning.md) and [disposal](docs/disposal.md) examples.

#### Changed

- **One managed-reference representation:** heap.new now returns T& directly, sharing
  ldobj, stobj and ldflda with frame-owned references. Removed the older Ref<T> /
  heap.load / heap.store path and advanced JSON artifacts from format 4 to format 5.
  newobj continues to construct values; explicit heap.new establishes heap storage.
  See [heap references and migration](docs/heap-references.md).
- **Transparent managed-reference access in Neo:** reading or assigning through T&
  automatically reads or updates the target. Forwarding preserves reference identity;
  reference formation remains explicit. Earlier development samples using `*age`
  now use `age`. Raw pointers retain separate low-level semantics and are not yet
  exposed by Neo syntax. Preview 1 itself did not contain a Neo compiler.
- **Documented platform direction:** clarified value/reference addressing, allocation
  and cleanup contracts while retaining CLR instruction/API familiarity. Recorded
  future designs for an optional Object hierarchy, native pinning and interop, and
  possible Neo library/compiler bootstrapping. These designs do not implement those
  features. See the [roadmap](docs/neo-roadmap.md) and [lifecycle design](docs/lifecycle.md).

### Migration notes for Preview 2

- Reassemble format-4 artifacts, including external System libraries, against the
  current runtime. Source and library artifacts must agree on signatures.
  New development instructions local.reset, ref.type and ref.eq require the current
  runtime; format 5 is still a provisional format, not a stable compatibility promise.
- Replace Ref<T> with T&, heap.load with ldobj T, and heap.store with stobj T.
  Remove a following pop if it consumed heap.store's old Void result: stobj produces
  no stack value. The runtime service is now ManagedHeap; heap.new also uses SlotReferences.
- Remove ArrayList.Free calls. Pass the collection receiver by managed reference.
  Use ArrayList<T>& to share one mutable list, or account for the documented
  descriptor-copy/backing-array sharing behavior. Native System.Array<T>.Free remains
  part of the separate native-buffer API.
- Earlier Neo development snapshots must remove explicit managed dereferences.
  Returning &local from its owning function remains invalid; allocate explicitly
  on the managed heap when a reference needs an independent lifetime.

### Remaining preview limits

- Guest destructors, finalizers, automatic scope cleanup and distinct runtime block
  lifetimes are not implemented. GC does not call Dispose or Close.
- Arrays retain fixed shape when replaced, including nested array fields. Neo renews declaration storage on each iteration; assignments to an existing
  array location still preserve its shape.
- The debugger requires launch-time integration; it does not attach to arbitrary OS
  processes or provide expression evaluation, memory editing, native-frame unwinding
  or editor/DAP integration.
- Reflection is metadata-only. Reflective invocation,
  field mutation, constructor queries, class hierarchies and module-level FunctionInfo
  queries remain future work.
- The compiler and library remain bounded prototypes. JIT/AOT execution, managed-object
  pinning, persistent host roots, cross-execution managed-reference inputs and a stable
  native ABI are not provided. New release claims require validation of the selected
  candidate; Preview 1's CI evidence does not certify current development.

## 0.1.0-preview.1 — 2026-09-07

Initial source-only prerelease, tagged at
[f11ec01](https://github.com/marinasundstrom/neoCLR/commit/f11ec01).
This historical section describes that tagged tree. The
[published release notes](docs/preview-1-release-notes.md) retain its full scope,
validation references and limitations.

### Added

- Standalone Rust interpreter, neoIL assembler, JSON metadata loader and optional
  typed control-flow verifier, using familiar CLI instructions where applicable.
- Primitive arithmetic/conversions, control flow, ordinary records, constructors,
  overloads, generic types, accessibility, properties and indexers.
- Platform-written System library with console and bounded file input, strings,
  numeric operations, ordinary Option/Result carriers and typed recoverable errors.
- Call-scoped managed references, output and conditional-output contracts, explicit
  reference receivers, interface views, List<T> and Equatable<T>.
- Native heap/pointer operations, frame-local localloc, native buffer arrays,
  native-layout ArrayList<T> with explicit release, and scalar/pointer P/Invoke.
- Minimal read-only type inspection, experimental Rust embedding, Fault stack traces,
  executable walkthroughs and source/artifact acceptance coverage.
- MIT licensing, locked dependency notices and source-preview packaging instructions.

### Compatibility and scope

- Prototype JSON format 4; no .NET assembly importer or stable artifact/ABI guarantee.
- Source-only distribution requiring Rust 1.85 or later and native build prerequisites.
- No Neo compiler, tracing GC or automatic destruction in this tagged preview.
  Raven-like examples in its documentation are explanatory pseudocode.

## Reconstruction provenance

The initial backfill on 2026-09-08 used the published tag and release notes, then
reviewed the 27 commits from that tag through reflection commit
[da2966d](https://github.com/marinasundstrom/neoCLR/commit/da2966d).
Development dates follow those commits. Related design, implementation and follow-up
documentation commits are consolidated into feature entries; superseded proposals
are not presented as implemented behavior. Subsequent commits maintain Unreleased
using the [workflow](docs/changelog.md).
