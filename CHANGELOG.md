# Changelog

Notable changes to neoCLR, grouped by release and development date. Every commit
updates the Unreleased section; related changes on the same date share an entry.
Published sections are frozen. See the [maintenance workflow](docs/changelog.md).

## Unreleased

### 2026-09-08

#### Added

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
  implementations must update their getter receiver contracts. General storage/return
  declarations remain future work. Reassemble
  external System artifacts for the expanded reflection descriptor layout; readonly
  metadata requires this runtime, and Neo now reserves the readonly keyword.

#### Changed

- Established a research-backed .NET/CLR comparison workflow for every roadmap
  capability and substantive revision of existing features. Added primary-source
  starting evidence, per-area research questions and decision/validation criteria;
  clarified that runtime mutability placement remains a candidate to evaluate.
  Added a code-backed runtime groundwork review after the readonly receiver milestone,
  covering storage/reference contracts, type relationships, nullable initialization,
  activation ownership, GC roots, cleanup and persistent state. Recorded proposed
  dependencies and acceptance cases, and paused further feature implementation.
  The reviewed foundations are proposals, not newly implemented capabilities.

- Planned runtime-enforced immutable storage and readonly reference/receiver
  capabilities, with Neo syntax and diagnostics above them. Documented their
  independence, shallow boundaries, alias checks and initialization/re-entry
  decisions; placed this foundation first in the exploration roadmap. These
  protected-slot contracts are not yet implemented; readonly inputs and receivers
  are implemented as described above.

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
