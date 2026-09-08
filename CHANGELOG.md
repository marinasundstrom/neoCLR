# Changelog

Notable changes to neoCLR, grouped by release and development date. Every commit
updates the Unreleased section; related changes on the same date share an entry.
Published sections are frozen. See the [maintenance workflow](docs/changelog.md).

## Unreleased

### 2026-09-09

- Added initial runtime-enforced notvoid/notreference generic constraints on types and
  methods, neoIL .constraint directives and Neo where clauses. Restrictions apply to
  the outermost argument; symbolic forwarding is rechecked at concrete resolution.
  Added metadata/host/source/IL regressions and design guidance. Base/interface
  constraints and constrained-member lookup remain next; notnull awaits nullable
  metadata. Older tools reject constrained artifacts; Neo reserves the new keywords.

- Added plain generic Neo record declarations with explicit positional construction,
  substituted fields, generic function forwarding and heap construction. Emitted
  generic definitions reuse the existing runtime metadata and lifetime rules; reference
  payloads preserve identity. Added an example, grammar/API guidance, .NET comparison
  and regressions. Generic record methods/classes, source unions, inheritance and
  source constructor inference remain separate slices.

- Updated the order workflow to use imported inferred Ok/Error cases and conditional
  Option/Result bindings, with explicit PurchaseError conversion for nested errors.
  Added zero-quantity scenario coverage and regressions for rejected-purchase side effects,
  exact stock exhaustion, lookup identity and receipt snapshots. Documented the
  remaining nested-carrier annotation cost and next generic-source-type investigation;
  runtime reference, inference and conversion contracts are unchanged.

- Added wildcard imports of marked bundled union cases in Neo: `import System.Result.*`
  enables `Ok(42)`/`Error("message")`, and `import System.Option.*` enables Some/None.
  Public carrier constructors supply the imported case definitions. Short names work
  in type positions and explicit/inferred constructor calls, preserving shadowing,
  ambiguity diagnostics, reference intent and runtime lifetime checks. Updated the
  example and design guidance, with nested inference and lookup regressions.

- Added argument-based generic library constructor inference in Neo, so
  `System.Result.Ok(42)` constructs an independent `Ok<int>` before any carrier
  conversion. Inference preserves reference payloads, requires evidence for every
  owner parameter and never uses a carrier target to override argument types.
  Explicit type arguments remain available; arbitrary external assembly imports
  remain future work.
  Updated the example, language guidance and Raven/.NET comparison, with regressions
  for nested inference, once-only evaluation and incompatible carrier targets.
  Split source-call and constructor lowering to reduce recursive compiler stack use.

- Added implicit Neo case-to-carrier conversion for marked bundled union types using
  the unique public constructor accepting the exact case value type. Expected types
  in annotations, returns, arguments and storage supply the carrier; ordinary unmarked
  constructors do not enable conversion. Added Raven implementation research, an
  updated case example and type/reference/lifetime regressions. Runtime instructions
  are unchanged.

- Added explicit public library constructor calls in Neo, including standalone
  System.Result.Ok<T>/Error<E> and System.Option.Some<T>/None values and explicit
  overloaded carrier construction. Metadata supplies parameter context for reference,
  readonly, delegate and Void payloads; arguments execute once. Added an example,
  API/grammar guidance and regressions.

- Added file-wide Neo imports of declared source union cases, including constructor
  calls and type annotations, with duplicate-import handling, ambiguity diagnostics
  and local/declaration precedence. Updated the order workflow to import PurchaseError
  cases and documented grammar and .NET lookup comparison. Generic source union
  declarations remain future work;
  emitted case identities and runtime contracts are unchanged.

- Added a reference-view example covering generic forwarding, base/interface views,
  readonly fields and collection elements, with identity/virtual-dispatch checks,
  GC-pressure coverage and readonly-container write rejection. Documented the distinction
  between replacing a stored readonly reference and writing through a readonly holder.

### 2026-09-08

- Changed Neo reference-to-reference assignment to copy the reference into mutable
  locals, captured bindings, writable fields and array slots, consistent with binding
  initialization and reference-valued collection setters. Breaking source change:
  use an explicitly value-typed intermediate for the former referent-copy behavior;
  immutable bindings now reject reference RHS reassignment. Value RHS and output
  writes retain target semantics. Added migration guidance, .NET comparison and
  assignment/storage/lifetime regressions; existing compiled IL is unchanged.

- Added a reference-passing example and regressions for alias forwarding, explicit
  address creation/local retargeting and output writes. Corrected the new type-design
  notes: current `out Foo&` initializes Foo storage, not a caller's Foo& binding;
  reference-slot replacement remains unsupported. Confirmed explicit reference creation
  instead of implicit argument borrowing and documented the existing assignment difference from C#.

- Added Neo `if let` and `let … else` case bindings with single evaluation, scoped
  immutable bindings and checked failure-path exits. The order workflow now uses
  a custom PurchaseError union and guards instead of nested success matches.
  Added grammar, usage documentation and control-flow/capture regressions; runtime
  instructions are unchanged. Recorded contextual borrowing as a future language
  experiment, without changing explicit reference syntax.

- Added non-generic Neo union declarations from existing source types or inline
  nested record cases. Carriers generate one constructor per variant, support exact
  case-to-carrier conversion and exhaustive whole-variant matching, and retain the
  existing erased-storage/GC rules. Added a sample, metadata/lifetime tests and type/API
  design guidelines. `union` and `case` are now reserved identifiers. Generic case
  inference and imported shorthand remain future work; no new runtime opcode is added.

- Added ordinary Result<T,E>.Ok/Error library factories, usable from Neo with an
  explicitly closed owner such as Result<int,string>.Ok(42). The order workflow now
  uses Result instead of a placeholder outcome record. Unique library static signatures
  now supply argument context so managed-reference payloads retain identity. Typing and frame
  storage checks remain unchanged. Documented imported case construction, generic
  inference and declared-case carrier conversion as a separate compiler plan, with
  standalone variant types and both existing-type and inline-case union declarations.

- Changed ArrayList assignment to share count and buffer coherently across growth
  using managed backing state. Added readonly Copy() for independent sequences with
  shallow element copies. Breaking library layout/behavior change: rebuild artifacts
  and replace reliance on independent copied counts with Copy(). Each independent list
  now allocates an additional managed state object; no runtime opcode changed.
  GC-pressure tests, including reflection collections, account for that retained state.

- Added an executable order-workflow experiment and a .NET 10 comparison to assess
  explicit managed-reference ergonomics, with focused tests for receipt snapshots,
  list-copy aliasing and callback capture lifetimes. Recorded observed friction,
  language/library distinctions and provisional alternatives; no runtime contracts
  changed. See [reference experience](docs/experiments/reference-experience/README.md).

## 0.1.0-preview.3 — 2026-09-08

A source-only preview focused on runtime-library foundations and the object/callable
features needed to exercise them in Neo. See [release notes](docs/preview-3-release-notes.md)
for examples, compatibility guidance and exact-commit validation requirements.

### 2026-09-08

#### Added

- Runtime-enforced readonly managed parameters, receivers and storage signatures,
  including fields, locals, arrays, returns and generic arguments. Derived references
  preserve permissions; writable-to-readonly narrowing is allowed and permission
  escalation faults. Neo adds readonly syntax; reflection and debugging expose the
  qualifiers. Readonly remains shallow and does not imply immutable backing memory.
- Class/record inheritance and inherited value layout, managed base-reference views,
  virtual/override dispatch and abstract classes/methods. Base views retain the complete
  owner and prevent value slicing. Type.BaseType returns Option<Type>. No mandatory
  Object base, implicit boxing or public downcast facility is introduced.
- Managed constructor chaining with unpublished construction storage, base completion
  and initialized-field checks. Neo adds ordinary classes, body fields, initializers,
  init declarations, bounded parameterless-constructor synthesis and default(T).
  Defaults do not invoke constructors or invent invalid references. Base-first
  initializer ordering deliberately differs from C#; positional records remain.
- Interface inheritance, inherited class implementations, explicit declaration-to-body
  mappings and default interface bodies, including most-specific dispatch, diamond
  ambiguity checks and reabstraction. Neo supports qualified implementation bodies;
  managed receiver lifetime, readonly and output contracts remain enforced.
- Generic free functions and static methods with independent method parameters,
  explicit type arguments, substitution, host invocation and closed call graphs.
  Neo adds namespace functions, static members and argument-based generic inference.
- Nominal generic delegates with checked delegate.bind, Invoke calls, heap receiver
  retention, virtual/interface binding and GC tracing. Neo adds delegate declarations,
  contextual method-group conversion, function-style invocation and contextual lambdas.
  Closures use managed capture cells/environments, share captured mutable bindings,
  and create fresh range-loop captures; unsafe frame/output/constructor captures are
  rejected. Func supports zero through four inputs, including Void results, and
  Array.ForEach consumes Func<T,Void> without a separate Action family.
- Comparable<T>, readonly comparison/equality receivers, Iterable<T> and Iterator<T>
  with Disposable, and inherited Iterable support on List<T>. ArrayList iterators
  retain their original buffer and extent; writes remain visible, unlike .NET List's
  mutation invalidation. Find, FindIndex and Exists use Func predicates; Find returns
  Option<T>. Added Neo conformance to bundled interfaces and library callback examples.
- Abstract MemberInfo as the shared base of MethodInfo, FieldInfo and PropertyInfo,
  with readonly managed readers and internal chained constructors. Reflection adds
  base/abstract/virtual/override/readonly facts and transitive interface introspection;
  member queries remain declared-only metadata snapshots.
- String.CompareOrdinal over UTF-16 code-unit ordering, plus readonly ContainsOrdinal,
  StartsWithOrdinal and EndsWithOrdinal over valid UTF-8 storage. Added Unicode 16
  System.Char predicates and Neo single-UTF-16-unit character literals, including
  surrogate escapes; pinned category data is reproducible and attributed.
- Int32 Math.Min/Max/Sign and Result-based Clamp; fundamental Double arithmetic,
  rounding, exponential/logarithmic and trigonometric helpers. Preserved documented
  .NET NaN/signed-zero behavior and ties-to-even rounding. Neo adds finite Double
  literals and same-type numeric operations, without implicit widening.
- Separate Date and Time values with validated Result factories, readonly components,
  equality/ordering, Gregorian day numbers and 100 ns ticks. Clock.GetLocalNow captures
  local Date, Time and UTC-offset seconds in one host reading using Chrono. Documented
  precision, timezone fallback and the trusted host-import boundary.
- Read-only Environment APIs for per-execution guest arguments, Result-based current
  directory and Result/Option variable lookup. CLI run/debug forward arguments after
  `--`, with the guest input path first. Missing and empty variables remain distinct.
- Lexical Path.Combine/GetFileName and bounded UTF-8 File.WriteAllText returning
  Result<Void,FileWriteError>. Output byte limits are checked before opening; writes
  create/replace regular files without adding a BOM and are not atomic. A runnable
  file-report example combines arguments, paths, bounded I/O and Result handling.

#### Changed and migration

- Neo arrays accept checked local extents and optional heap initializer braces;
  samples omit empty braces. Single-index Item properties use bracket syntax,
  including interface dispatch and reference-valued elements.
- Recompile applications and external System libraries together. New readonly,
  inheritance, constructor, implementation-mapping and delegate metadata/opcodes need
  this runtime; the JSON format remains provisional. Rust metadata literals and
  exhaustive enum matches may need updating.
- Equatable, scalar Equals, comparison contracts and reflection descriptor readers
  now use readonly managed receivers. IL callers/implementers must match them; Neo
  borrows automatically. List implementations must supply inherited GetIterator and
  the current readonly getter contracts. Host wrappers must supply managed receivers.
- Derived constructors use managed receivers. Managed stfld produces Void; the
  value-form instruction remains a record update. Whole-value reads, resets, writes
  and out through projected base views fault to prevent slicing.
- Neo reserves class, default and readonly and generated neoCLR.Compiler names.
  Closures incur managed heap allocations. Rust ExecutionOptions gains arguments;
  exhaustive literals must initialize it or use defaults.
- Runtime immutable-slot proposals were superseded: immutable bindings remain a
  language feature. Nullable type signatures for both values and references are
  planned, with null distinct from zero/default and uninitialized storage; they are
  not implemented. Option remains preferred for domain optionality.
- Kept future enums/flags, constraints, async, dynamic hooks and broader framework
  growth on a research-backed .NET/CLR roadmap. LINQ, globalization, date/time
  parsing/formatting, generic instance methods, multicast/variance, reflective
  invocation and broader native/resource lifetime features remain later work.

#### Validation and packaging

- Expanded archive validation to 26 Neo source/artifact scenarios plus native interop,
  including semantic checks for clock readings, guest arguments and report output.
  CI covers Linux/macOS/Windows on stable and minimum Rust 1.85.0; publication requires
  a successful six-job run on the exact versioned candidate.
- Refreshed notices for all 47 locked registry packages with 94 byte-hashed license
  texts, preserving upstream bytes across checkouts. Added Char data attribution,
  API/.NET comparison probes, migration guides and synchronized Neo grammar/examples.
- Fixed the legacy Math.Abs boundary assertion to select its Int32 parameter signature
  rather than rejecting the Double overload. Recorded local validation and archived
  evidence; published Preview 1 and Preview 2 history remains unchanged.

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
