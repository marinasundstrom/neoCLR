# Development conversation record

This is a record of conversations in which the project author directs neoCLR's
development with an AI coding assistant. Through this concrete project, readers should
gain insight into how the author works with AI and approaches software development as
a whole. The exchanges supply that insight through actual questions and actions, rather
than an invented account of motivations. It serves a purpose similar to meeting
minutes: preserving questions, proposals, decisions, corrections, actions and outcomes,
with attribution to the person or assistant that raised them. It is not a personality
profile, a polished manifesto or a verbatim transcript.

**Initial record compiled: 2026-09-09.** The project began in the author-provided
shared chat [Define neoCLR proof of concept](https://chatgpt.com/s/cx_6aa11941d6d881918b897324f3a94730),
read on that date. This single record follows selected exchanges from that founding
conversation through Preview 1 and subsequent development.

Individual message dates are unavailable: sequence is retained without invented dates.
The founding account combines both participants' visible shared-chat messages with
additional exchange and validation detail retained in this task. Historical test,
commit and release reports are not fresh validation. Technical comparisons record
what was said then, rather than asserting what a .NET release currently implements.
The post-Preview-1 retrospective explicitly marks unavailable assistant replies;
missing responses and outcomes are not inferred from later code. Blockquotes reproduce
available original wording; other text is a summary.

See the [changelog](../CHANGELOG.md) and linked design documents for technical status.

## 2026-10-09 — Website audience and editorial cleanup

- **Author:** asks for a concise website with detail where it matters, observing that
  historical information distracts readers. Directs continuation of a cleanup, with
  a landing page that immediately explains the product and the tone of a software
  project page rather than a preachy presentation.
- **Assistant:** identifies accumulated chronology in Direction and Native compilation,
  stale About information and repeated release notes. Proposes current-state feature
  guides, grouped homepage entry points and separate benchmark reporting.
- **Author clarification:** website and GitHub documentation need a clear divide;
  they present different views for different audiences and should not repeat the same
  details in the same way.
- **Assistant action:** rewrites the principal website pages, separates benchmarks,
  simplifies the website's metadata/architecture explanations, removes redundant
  feature-page release commentary and records the audience split in
  [website maintenance](design/feature-pages.md). Retains exact requirements for
  versioned runnable samples and links to implementation records. Repository contracts
  and published release history are not rewritten as part of this editorial change.

- **Author follow-up:** asks for distinct sidebar icons for non-type module members,
  correct constant grouping, and a better API reference landing page.
- **Assistant action:** applies the existing member glyphs in RavenDoc navigation,
  separates Constants from Functions, and replaces the API landing page's accumulated
  change notes with module browsing, subject entry points and short reading guidance.

- **Author follow-up:** points to `System.Linq.Operators` and asks to distinguish
  extension containers from static classes, give them a distinct icon, include
  receiver information and display `extension Operators for Iterable<T>`. Requests
  the presentation on Raven main and emphasizes sharing behavior through the similar
  metadata markers.
- **Assistant approach:** uses semantic extension receivers in the shared publisher,
  with a dedicated container icon and linked receiver types. Validates source,
  imported .NET and native reference rendering, preserving ordinary static classes.

- **Author refinement:** requests a receiver section with separately clickable type
  components, a Type parameters table like other generic types, and a separate
  Type extensions group on module pages showing each receiver in the list. Then
  directs renaming the visible Extension container label to Type extension and
  placing Receiver above Type parameters.
- **Assistant action:** reuses the shared contract-type formatter and type-parameter
  documentation table, adds Operators' element-type description, and separates
  extension listings from ordinary types. Original container parameter identity and
  constraints lost by imported metadata are not inferred from method parameters.

## 2026-10-09 — Architecture and metadata explanations

- **Author:** requests repository documentation and website pages for architecture
  and metadata, and asks that the work be committed. Adds diagrams, specifically
  suggesting runtime services shown as boxes in layers/levels.
- **Assistant:** documents the implementation with shared diagrams for the source
  pipeline, runtime/service layers and PE/NEOX/payload boundaries. Distinguishes
  current transport, native execution limits and future CLI-derived direction,
  and links the guides from the website and repository indexes.
- **Author clarification:** class-library APIs live in modules that form namespaces;
  modules are the primary code organization, rather than arbitrary namespaces. Asks
  for this terminology in architecture documentation, leaving the full explanation
  to the Modules feature page.
- **Assistant follow-up:** adds this explanation and module examples to both
  architecture pages, links the Modules feature page, and labels the library layer
  in the diagram as modules.
- **Scope:** documentation of existing behavior; no new runtime or format contract
  and no change in roadmap priority. See [architecture](architecture.md) and
  [metadata format](metadata-format.md). Validation and commit results are recorded
  in the accompanying task response; website publication is a separate operation.

## 2026-10-09 — Implement the module foundation

- **Author:** asks to implement module format and foundational neoCLR APIs, then
  Raven parsing/binding/symbol changes. Permits reuse of namespace declaration syntax
  with the `module` keyword as a starting point. Requests module terminology and a
  website feature page while retaining assembly ownership in generated API reference.
- **Author clarification:** one module per assembly is common, but assemblies may
  contain many modules, notably System.Runtime. Assembly and module names need not
  match. Submodules are a possible organizational shape.
- **Assistant:** chooses bounded slices for native metadata/API ownership, Raven's
  syntax and existing import/qualified lookup, and website documentation. Retains
  physical-image APIs explicitly rather than silently reinterpreting their identity.
- **Later author requirement:** “The Visual Studio Code experience for NeoCLR should
  of course say module instead of namespace.” The assistant extends target-aware
  symbol presentation, hover, completion descriptions and outline/workspace kinds,
  retaining ordinary .NET namespace presentation.
- **Implemented foundation:** [versioned declaration containers](declaration-modules.md),
  scoped authoring, reader/discovery views and runtime validation. Guest RuntimeContext,
  module-private access and re-exports remain open; those proposal details are not
  treated as author-approved completed contracts.
- **Compiler/consumer evidence:** file/block syntax, nested imports and module
  ownership round-trip; a checked sample returns 42 in interpreted and ARM64 native
  modes both within one assembly and across a separately compiled library. Focused
  compiler, RavenDoc and language-server tests cover presentation. Browser inspection
  confirms Module labels with real System.Runtime.dll assembly ownership and neoCLR
  branding. The feature page labels this as a development foundation; no installed
  VS Code client or guest module reflection is claimed as tested.

## 2026-10-08 — Move native compilation blockers into Raven

- **Author:** directs work to Raven `codex/source-object-metadata-resolution`,
  believed to be the neoCLR integration branch. Asks to investigate related lookup
  problems while keeping native compilation the primary goal, and compare with main
  so general fixes can be placed there too.
- **Assistant action:** confirms the branch, commits the completed neoCLR Console
  remainder-reader slice on main (`d7af270b`), and switches Raven to the requested
  integration branch. Reduces the intermittent Console errors to lookup before
  emission: a CLI bootstrap stub competes with the source-built native declaration.
- **Findings/actions:** reproduces failures in ordinary .NET tests on both compiler
  branches. Fixes source-assembly ownership, qualified type selection and wildcard
  import selection; updates closure Object resolution. Commits the shared fixes on
  both Raven branches. Strengthens producer reuse hashes to include compiler
  implementation assemblies, and changes the reader sample to imported Console calls.
- **Validation/outcome:** 68 focused import/namespace/lookup tests pass on each branch.
  The [Console qualification](experiments/aot-console/README.md#raven-lookup-fixes-and-fresh-native-qualification-2026-10-08)
  records the native consumer result and exact compiler evidence. Native backend
  availability on Raven main is not implied by the shared fixes; wider native emitter
  coverage and eventual HTTP AOT remain open. This compiler investigation supports
  the existing author-directed AOT goal, rather than changing the whole roadmap.

## 2026-10-07 — Investigate ARM64 JIT/AOT and independent hot reload

- **Author:** asks to investigate future JIT and AOT, emphasizing AOT; retain the
  interpreter because it eases some hot-reload scenarios, but do not tie hot reload
  to interpreted execution. States: “I would set ARM64 to be our primary target
  architecture”.
- **Assistant proposal:** investigate AOT first through shared semantic/runtime
  contracts, with Cranelift as an initial feasibility candidate and LLVM as an AOT
  alternative. Model hot reload as versioned body/module replacement across modes;
  distinguish reloadable from sealed AOT. Propose compatible body edits before state
  migration and macOS ARM64 before a Linux ARM64 qualification.
- **Action/outcome:** inspected current runtime foundations and primary platform
  documentation; recorded the [investigation](native-execution-investigation.md),
  linked architecture/design guidance, and updated roadmap and website proposals.
  This is research, with no native compilation or reload experiments performed.
- **Author follow-up:** “An attractive POC demo would be to AOT compile a web app.”
  Then clarifies: “But we should start simple”.
- **Assistant response/action:** records the web app as the motivating demo and
  proposes scalar native code, Hello World/runtime services, then one HTTP endpoint.
  A native launcher running interpreted handlers would not satisfy the AOT demo.
  Hot reload is a separate later experiment, not a prerequisite for the first demo.
- **Author benchmarking follow-up:** “Eventually we can compare with .NET and other
  languages/platform. Do benchmarking”.
- **Assistant response/action:** records a later comparative benchmarking stage,
  including .NET JIT/Native AOT and candidate additional platforms, equivalent
  workloads, startup/throughput/latency/memory/size measurements and reproducible
  evidence. Benchmark implementation and execution remain future work; this does
  not expand the first simple AOT proof.
- **Author implementation follow-up:** “Continue work on AOT”.
- **Assistant action/outcome:** implements an isolated Cranelift 0.121.2 scalar tool
  that emits ARM64 Mach-O objects and exports an Int32 function to a C executable.
  Focused tests establish native/interpreter parity at integer boundaries, reject
  unsupported inputs and check artifact format/no runtime imports. See the
  [experiment and evidence](experiments/aot-scalar/README.md). This bounded probe
  does not implement branches, native Fault propagation, GC, HTTP, JIT or hot reload,
  and does not change Raven or the CLI bridge. Production backend choice stays open.
- **Author branch correction:** requests a commit for each slice, then corrects the
  assistant's use of `codex/structural-types`: “It should be main”, followed by
  “Cherry pick changes over to main an continue from there”.
- **Assistant action:** preserves unrelated structural proposal edits in a named Git
  stash, cherry-picks the isolated scalar commit onto main, preserves newer native
  bootstrap/release documentation, and revalidates against main before continuing
  the control-flow slice. Original feature-branch evidence remains historical.
- **Author deployment clarification:** wants an executable with dependencies baked
  in and no shared framework/runtime requirement: “the goal is compiling CIL to
  native code”. Adds that establishing the foundation early will make implementation
  easier as the platform evolves. Then specifies Hello World first, followed by more
  complex samples until HTTP Server.
- **Assistant response/action:** records self-contained CIL AOT as the target and
  sample progression as the immediate sequence. Implements a neoIL-based native
  Hello World with startup/UTF-8 console code linked into its executable; checks
  independent execution, OS-only linkage, byte fidelity and service failures.
  CIL input remains pending; proposes exercising it next through shared verification,
  lowering and linking. These steps do not claim arbitrary .NET compatibility,
  freestanding execution or an implemented native HTTP Server.
- **Author trimming follow-up:** “In a future step, we will have to figure out trimming”.
- **Assistant response/action:** records trimming as later work, with code/metadata
  roots, reflection/dynamic use, initialization and runtime-helper dependencies to
  establish. Keeps the current non-trimming Hello World proof bounded.
- **Author input correction:** “We will compile neoCLR CIL into native code.” Then
  specifies: “So we will have to compile Hello World in Raven into it's metadata
  format with its IL, and then compile into native code”.
- **Assistant correction/action:** had started an isolated ordinary .NET assembly
  importer after interpreting CIL too broadly; removes that uncommitted experiment.
  Uses the existing native-enabled Raven compiler to emit Hello World in PE/#Neo,
  feeds its actual metadata/IL directly to the AOT backend, and adds a parameterless
  entry adapter plus checked bundled-console lowering. Tests PE/#Neo and standalone
  NEOX execution, interpreter parity, container rejection and standalone deployment.
  [Raven pipeline evidence](experiments/aot-hello/raven-validation.json) records the
  pinned compiler and build-time bootstrap limitations. General dependency compilation,
  richer samples, HTTP Server and trimming remain open; no Raven compiler change occurs.
- **Author next milestone:** asks for compiling types and members, especially value
  types so a sample can use unions such as Result and Some and exercise control flow
  or branching instructions.
- **Assistant response:** finish the Raven Hello World commit, then inspect actual
  emitted type/member metadata and select a bounded value-type slice. This direction
  does not imply that union or general member AOT is already implemented.
- **Assistant investigation:** compiles Counter and Some/None Choice probes with the
  pinned native-enabled Raven compiler, inspects their metadata/IL and executes them
  under the interpreter. Records constructor/field/byref/member and nested-payload/out
  requirements; AOT correctly rejects these still-unsupported declarations. The union
  needs explicit native Runtime references; an earlier host-bootstrap attempt rejects
  generated ToString. See the [inventory](experiments/aot-values/inventory.json).
- **Author reference-type question:** wonders whether reference counting should be
  the first implementation step, “Before we consider JIT for AOT”.
- **Assistant response/proposal:** treats counting as an experiment to compare with
  a simple native tracer, not an approved replacement. Notes the existing interpreter
  already collects cycles; plain counting requires a cycle policy, reference-bearing
  value handling and return/Fault cleanup. AOT/JIT and memory-management choices are
  independent. Records [sources, costs and validation needs](native-execution-investigation.md#reference-counting-as-an-early-native-experiment-2026-10-07).
  No reference-counting implementation or benchmark is performed.
- **Author console-input direction:** “When we have unions we can also do console
  inputs in sample”. **Assistant proposal:** read input with end/failure cases, parse
  an integer into Result and match success/error, exercising strings, input services
  and branches. This is a subsequent sample direction, not implemented input support.
- **Assistant continuation outcome:** implements a bounded flat-value backend and
  compiles the Raven Counter and copied Int32/Boolean record consumers to native ARM64.
  Adds constructors, fields/accessors, borrowed receivers, value copies and typed
  control flow with interpreter parity and Fault/rejection tests. Standalone executables
  need only the OS library. [Evidence and limits](experiments/aot-values/native-validation.json)
  distinguish this completed slice from still-unsupported nested unions and console input.
- **Open:** backend choice, OS qualification order, native ABI/GC/metadata contracts,
  edit compatibility, measurements and implementation scheduling. These details are
  assistant proposals, not subsequent author decisions. The active Raven work and
  isolation of structural Function experiments remain recorded.


- **Author follow-up during overloaded-member work:** “We might have to mangle function names?”
- **Assistant response:** explains that private native symbols already use per-function
  indices, avoiding overload collisions within the single compiled object. Calls first
  resolve the exact metadata definition; symbol naming is a later lowering concern.
  Proposes a future stable mangling contract for separate compilation that includes
  module/type identity, signatures and generic arguments. No stable format is selected.
- **Assistant action/outcome:** implements exact overload resolution with signature and
  optional definition-ID guards; tests Raven members, constructors and nominal output
  overloads. Records [the current symbol scheme and limits](experiments/aot-values/README.md#overloaded-members-and-native-symbols-2026-10-07).
  Separate-object linking, stable native names and general union compilation remain open.

- **Author direction:** explore metadata alongside a native image as an improved calling
  interface in the future, recognizing the need for stable ABI conventions; continue
  current work. **Assistant response:** records comparison with .NET Native AOT C exports
  and GIRepository metadata, pairing/layout/ownership tradeoffs and validation needs.
  Adds read-only AOT inspection to expose generated union requirements, not a native ABI.
- **Author milestone instruction:** continue slices until a working union sample app,
  then reassess which sample should drive further work. **Assistant proposal:** ordinary
  Raven Some/None with both branches and payload extraction first; a Result-based console
  input/parser afterward to drive strings, input, failure and lifetime handling before HTTP.
  Native union completion remains pending at this inspection slice.
- **Assistant implementation choice:** add an explicit closed-world direct-call mode to
  compile value-only union paths without unused generated formatting/boxing services.
  This brings limited code selection forward from later trimming; it reports exclusions
  and preserves the original artifact. It does not silently change whole-module admission
  or establish reflection-aware trimming.
- **Assistant completed outcome:** the ordinary Raven Some/None app now passes fresh
  producer/interpreter/native validation and executable-only deployment on macOS ARM64.
  It exercises both cases, patterns, non-matches and copies; selection reports ten compiled
  functions/three value types and twelve excluded generated methods/three attribute classes.
  Source readonly/access facts remain enforced. [Evidence and limits](experiments/aot-union/README.md).
- **Assistant reassessment:** recommends an interactive integer reader driven by actual
  Option/Result semantics. First add value-only Result/generic specialization, then define
  native UTF-8 input and lifetime handling, then parsing/output and EOF/error cases. HTTP,
  stable metadata/ABI and benchmarks remain later work; no memory-management choice or
  implementation approval beyond the completed union milestone is inferred.

## 2026-10-07: Continue native bootstrap development on main

- **Author:** Directed merging the native bootstrap branch into main and cleaning up
  branches so development can continue there.
- **Assistant action:** Combined native bootstrap with main's newer shared RavenDoc
  publisher and navigation, preserving both histories. This changes the development
  branch; it does not declare Windows qualification complete or publish a release.
  Cleanup is limited to branches whose tips are included in the merged history.

## 2026-10-07: Windows native release and simpler website instructions

- **Author:** Required Windows support for a native metadata release and asked to remove
  historical details and change notes from the website, using the changelog for history.
  Asked for simple installation instructions and prominent Raven navigation, with links
  to the language website and playground.
- **Assistant action:** Reduced setup to prerequisites, one matched download and running
  a sample; removed superseded bridge setup and accumulated integration reports. Added
  Raven navigation and external language links, retaining concrete target limits.
  Added a Windows native toolchain qualification workflow and replaced Unix-specific
  HTTP pipe readiness with a bounded portable reader.
- **Open:** Windows runner execution, installed editor acceptance and a qualified Windows
  download remain required. Local changes and CI configuration alone do not establish
  Windows support or publish the website. RavenDoc provider redesign remains deferred.

## 2026-10-02: Separate compiler semantics from metadata authoring

- **Author:** Asked for abstract Raven loader/emitter interfaces and an ILGenerator
  interface in the Cecil-like library, with instruction operations off the builders.
- **Assistant:** Identified Raven's existing IILBuilder and the library's direct
  MethodBuilder operations, initially discussing them together.
- **Author correction:** “You shouldn't mix the Raven IL builder with the Cecil-like
  builder API, because those are different boundaries.”
- **Assistant:** Accepted independent compiler and assembly-authoring contracts,
  connected only by the target backend adapter.
- **Author refinement:** Importers should populate symbols with all needed information;
  emitters should use the semantic model and not reuse loader/importer objects. Type
  resolution must account for each source while keeping those responsibilities separate.
- **Investigation:** The native emitter still reads native method/field definitions,
  passes a metadata resolver, and searches input definitions. Existing Raven IILBuilder
  also exposes reflection operands; portable lowering contracts offer a better starting
  point. These findings are current gaps, not completed fixes.
- **Recorded action:** Documented ownership, required semantic identities/signatures,
  native linkage and snapshot safeguards, and a bounded symbol-only function-reference
  migration in both repositories. Raven design commit: `7354fa3fd`. Library generator
  implementation and removal of importer coupling remain open. No migration is claimed.
- **Separate completed work:** Qualified native generic function lookup now executes
  across three assemblies; [seven-consumer evidence](experiments/extended-cli-metadata/native-qualified-functions-2026-10-02.json)
  preserves .NET controls and identifies the shared fix separately. That evidence does
  not establish the new architectural separation.

## 2026-10-02: Fail names the action; Fault names the result

- **Author:** “System.Fault method should be called System.Fail(message)” and “The Fault
  is a result of Fail”, while terminal propagation-failure support was being investigated.
- **Assistant:** Accepted the distinction and scoped a public API rename, retaining the
  host Fault type, fault classifications and low-level fault instruction. Updated Raven
  terminal-call recognition, reference generation, runtime source and consumers together.
- **Outcome:** Eight compiler flow tests, four source-admission checks, seven runtime
  tests and qualified/imported Fail execution passed; runtime and API snapshots were
  regenerated. [Evidence](experiments/extended-cli-metadata/system-fail-rename-validation-2026-10-02.json)
  distinguishes the native filesystem control from a still-rejected CLI importer case.
- **Direction:** The naming correction does not request exceptions or process abort.
  Generated propagation-guard/native metadata emission remains the subsequent integration
  work; renaming the existing runtime API alone does not solve that blocker.

## 2026-10-01: Return to the end-to-end case

- **Author:** “We should make this good enough for now. Record the direction. Work
  towards the working end-to-end case. Update the metadata library as you need to”.
- **Assistant:** Paused broad architecture migration and reran the unchanged collections
  sample. Binding and CLI control emission still succeed; direct native emission still
  rejects PendingOrder returning Option<Order>. This is investigation evidence, not a fix.
- **Finding:** Raven's portable admission currently admits external classes/interfaces,
  metadata imported nominal signatures use CLASS, and native projection reconstructs
  external nominal types without a value-category declaration. These must preserve the
  same value-type contract before enabling the union signature end to end.
- **Direction recorded:** Metadata work is now driven by integration blockers. The
  Cecil-like definitions, Reflection.Emit-style builders and definitions/metadata/PE
  boundaries remain the intended architecture, not prerequisites to further integration.
  Imported value signatures, downstream members/union operations and explicit native
  System identities remain open. See the roadmap and refreshed collections evidence.

## 2026-10-01: Explicit metadata and PE boundaries

- **Author:** Specified “builders -> definitions -> metadata -> PE” and readers/writers
  working in the corresponding directions.
- **Assistant:** Interpreted the write path as definition encoding followed by PE
  packaging, and the read path as PE extraction, metadata decoding and definition
  materialization. Builders remain optional helpers over definitions.
- **Action:** Recorded the target boundaries while completing direct generic type
  authoring/shared constraint storage. Identified current combined WriteImage encoding
  and packaging plus immutable reader snapshots as remaining architectural work.
  No separate metadata/PE layer API or editable reader was claimed as implemented.

## 2026-10-01: Reflection.Emit-style builders over Cecil-like definitions

- **Author:** Clarified that builder APIs should be much more similar to Reflection.Emit,
  easing generation without being a replacement for the Emit APIs.
- **Assistant:** Distinguished the Cecil-like definition/reference model from the
  Reflection.Emit-inspired generation convenience layer, with both sharing one graph.
  Did not infer a drop-in compatibility promise or a host Reflection.Emit dependency.
- **Action:** Recorded this direction alongside the property-definition migration.
  Existing Add-style entry points remain; future builder naming and generation patterns
  should follow the clarified direction. No wholesale API rename was performed.

## 2026-10-01: CLI/CIL baseline and first definition migration

- **Author:** “neoCLR CIL format will build on the same format. Extend or modify it.”
  Directed continued end-to-end work with appropriate adjustments.
- **Assistant action:** Began authored assembly/type/field construction with builders
  sharing those declarations, retaining existing encodings. Manual struct construction
  and optional method helpers execute on CLR and neoCLR (42); 76 C# contract groups
  and two Raven cross-assembly probes pass.
- **Open:** This is a partial migration. Canonical method/body definitions and loaded
  editing are still pending; the unchanged collections Option<Order> gate remains open.

## 2026-10-01: Definitions are the editable metadata model

- **Author:** Clarified the expected layering: “Builders -> Definitions -> Encoded
  assembly”. Definitions should be directly constructible; builders facilitate creating
  types and members. Supplied a Mono.Cecil example creating an assembly, a struct based
  on System.ValueType, a public field, and writing the assembly through definitions.
- **Assistant:** Acknowledged that the current implementation does not meet that model:
  loaded AssemblyDefinition objects are immutable snapshots while AssemblyBuilder owns
  a separate mutable graph consumed by the writers. Described the split as temporary,
  not the intended Cecil-like abstraction.
- **Assistant action:** Finished the already-running value-payload/address slice
  (633a4a4a), validated by 75 C# groups and CLR/native execution, then recorded unifying
  the definition model as the next priority before adding more builder-only features.
- **Proposed implementation sequence:** Make definitions authoritative for the editable
  assembly graph, make builders thin facades over the same objects, route writers through
  definitions, and progressively materialize supported loaded metadata/bodies into that
  same model. Validate direct definition construction and builder/definition mutation
  identity, retaining existing Raven/native executable checks. Unsupported loaded content
  must not be silently discarded during editing.
- **Author follow-up:** Endorsed setting the architectural direction and aligning with
  Mono.Cecil “when it matters”, then supplied https://github.com/jbevain/cecil.
- **Assistant action:** Reviewed Cecil's assembly/type definitions, member ownership,
  module imports and ILProcessor source, recording a sourced alignment map. No promise
  of drop-in compatibility or adoption of Cecil as a dependency was inferred.
- **Open:** This conversation records a correction and a refactor plan, not completion
  of the unified model. Full runtime-library emission and the collections union/import
  gate remain open. See the [API direction](../api-docs/experimental-metadata.md#definition-first-model-author-direction-2026-10-01).

## 2026-10-01: Existing samples remain the end-to-end acceptance target

- **Author:** Clarified that the main goal is Raven compiling neoCLR assemblies which
  neoCLR loads and executes. Existing compiling/executing samples predate this work
  and should assess progress.
- **Assistant:** Confirmed that API lifecycle work is supporting work and focused
  probes validate individual gaps, rather than replacing unchanged sample acceptance.
  Committed the imported-generic prerequisite, then reran the unchanged
  application-order-collections source through direct emission.
- **Observed outcome:** Binding and ordinary CLI control emission succeed; direct
  emission still rejects Register's MutableMap<int, Order>/ArrayList<Order>
  signatures, writes zero native bytes and is not executed. The generic-method slice
  did not close that nominal/generic-owner boundary. Prior legacy-bridge execution
  remains baseline evidence, not proof of direct emission.
- **Next work:** Drive imported nominal/generic type and member identity from this
  existing sample and its translated System dependency, retaining ordinary .NET
  TypeRef/TypeSpec/MemberRef shape. Constructor/instance calls, callbacks and union
  lowering remain subsequent sample requirements. Do not substitute a growing set
  of isolated synthetic successes for this end-to-end gate. See the
  [fresh checkpoint](experiments/extended-cli-metadata/collections-after-generic-imports.json).

## 2026-10-01: .NET metadata baseline and a complete Cecil-like lifecycle

- **Author:** Reaffirmed: “Our baseline is .NET metadata format and our future
  extensions” and said the Cecil-like API must allow inspecting, modifying and
  generating assemblies from nothing, without diverging too far in shape or behavior.
  Noted that assembly-level functions are already supported.
- **Assistant:** Agreed that ordinary metadata/IL keeps .NET shape and behavior;
  existing assembly functions retain their CLI `<Module>` compatibility representation.
  Explained that the active imported-generic slice uses GenericParam/MVAR and
  MethodSpec/MemberRef rather than a new format. Distinguished immutable loaded
  snapshots and editable producer graphs from the still-incomplete general loaded
  assembly editing lifecycle. Narrow import APIs are implementation limits, not a
  proposed permanent alternative object model.
- **Actions/outcome:** Recorded this direction in the roadmap and API reference while
  continuing the bounded library/application generic-call implementation. C# CLR
  execution and direct native binary execution are the acceptance checks. Full
  loaded read–edit–write support, imported nominal/generic owner identity and native
  compiler symbol loading remain open; this exchange does not claim their completion.

## 2026-10-01: Refactor parity before expanding native emission

- **Author:** Worried that codegen refactoring changed behavior. Suggested moving
  suitable semantic work into binding, then clarified: “I don't say that we should
  push everything up to the binder” and “the main task is to fix the issues due to
  the refactoring.” Directed continued end-to-end native emission after regressions.
- **Assistant:** Compared the shared compiler and integration branch with existing
  runtime cases and new Debug/Release return-value/fault tests. Corrected an initial
  regression claim when the decimal-loop console test passed isolated and serial runs.
  Recorded imported-carrier binding and loop-capture problems on both lines as open;
  did not claim full parity or that these problems were fixed.
- **Action/outcome:** Committed the bounded parity audit (`87ff03d3f`), then validated
  the real neoCLR CLI declaration profile for direct binary emission (`6f46bbade`).
  Four native binary cases verify/run, covering Hello World/function calls, Unit entry,
  arrays and owned interface dispatch without a host core reference. Mismatched core
  identities reject before output; the host-bootstrap control continues to pass.
- **Still open:** Implementation bootstrap, native metadata symbol loading, broader
  collection/generic imports and known carrier/capture defects. Feature branch work
  is not merged into main merely because the independent fixes were integrated.
- **Evidence:** [Assessment follow-up](experiments/extended-cli-metadata/readiness-assessment-2026-10-01.md#native-profile-gate-and-refactor-parity-follow-up).

## 2026-10-01: Integrating independently proven Raven fixes

- **Author:** Requested a specific fix branch based on main for binding and other
  fixes that benefit Raven independently, except where already isolated and committed.
  Added: “If we can prove these fixes, we can integrate them into main.”
- **Assistant:** Extracted six fixes from the metadata-consumer branch into
  `codex/compiler-fixes-from-neoclr`, preserving individual commits and replacing
  integration-only test assertions with ordinary C# compiler/runtime regressions.
- **Observed outcome:** Original main fails 16 of 23 regression cases; all 23 pass
  with the fixes, alongside 174 surrounding cases. Local Raven main was fast-forwarded
  from `d7040e21d` to `e5607ca17`. No push was performed. Modern .NET validation does
  not establish native neoCLR execution or full-suite qualification.
- **Still open:** Broader shared lowering/initialization abstractions require separate
  isolation and validation. The order-collections Option constructor mismatch is
  still unfixed. Native backend/metadata work remains on feature branches.
- **Evidence:** [Readiness assessment and independent commit inventory](experiments/extended-cli-metadata/readiness-assessment-2026-10-01.md#independent-raven-fixes-integrated-into-local-main).

## 2026-09-30: Measuring JSON versus native assemblies

- **Author:** “We will have to benchmark this. JSON vs Assembly”.
- **Assistant:** Commits the direct assembler first (`116be40e`), then measures
  release-mode decoding separately from common runtime phases and alternating
  fresh-process runs, with matching metadata/output and a warm filesystem cache.
- **Outcome:** On one Apple M1 run, System's native assembly is 63% smaller than
  pretty JSON and takes about 20% less time to decode through current paths. Full process time
  remains about 3.45 seconds with no meaningful observed improvement; linking/admission
  dominates. A direct-typed JSON diagnostic is faster than the current native decoder,
  identifying intermediate-tree removal as a candidate rather than claiming inherent
  binary-format superiority.
- **Author follow-up:** “Smaller files is an improvement. We will optimize the runtime
  in the future.” The assistant retains the size reduction as the demonstrated benefit,
  saves the benchmark baseline and leaves runtime optimization for later; metadata and
  compiler integration remain the next focus.
- **Limits:** Nine samples, two real artifacts, one machine; no cold-disk or memory
  claim. No decoder optimization was performed in this slice. [Raw evidence and method](experiments/extended-cli-metadata/json-vs-assembly-benchmark.md).

## 2026-09-30: Existing Raven samples and a future neoil assembly producer

- **Author:** “We will create a proper neoil assembler that can produce the assemblies.”
  Also asks to explore translating and running the output of existing Raven tests and
  samples, then directs work to continue.
- **Assistant:** Adds an experiment using the current Raven CLI bridge, comparing
  neoil, JSON and translated binary application outputs with established expectations.
  Keeps this distinct from direct native compiler emission and the future assembler.
- **Finding:** The matching Raven collection-profile System is larger than bundled
  System and exceeds the binary profile's input/item bounds. Retains the guards and
  explicitly uses the same JSON System to isolate application transport results.
- **Fixture correction:** Replaces obsolete Option carrier construction in the existing
  positional-pattern test with idiomatic cases; updates its separate wrong-arity
  diagnostic expectation. No compiler semantics change.
- **Outcome:** Fourteen applications pass binary execution against matching JSON
  System; six expected compiler rejections hold. FloatingMath passes source/JSON but
  exposes schema 2's unsupported UInt64 double-bit operands. This is a transport gap,
  not an observed floating arithmetic regression.
- **Evidence and remaining work:** [Sample experiment](experiments/extended-cli-metadata/raven-sample-translation.md).
  A proper neoil binary producer, larger-library representation and direct Raven
  class-library source emission remain separate follow-up work.

## 2026-09-30: Translated class-library artifacts as a compiler baseline

- **Author:** Proposes translating the current JSON into neoCLR assemblies, then
  clarifies: “We will have to work a way to compile the runtime class library from
  source, after this.” Translation is “a useful test. Of what the runtime can load”.
- **Assistant:** Distinguishes generated build manifests from executable format-5
  JSON; implements a native-only translation path preserving broader library metadata
  without inventing incomplete CLI declarations.
- **Outcome:** The whole current assembled System library translates, loads and
  verifies (117 types, 743 functions, 641 IL functions). Both Hello cases and a
  translated generic dependency chain execute against translated System. These are
  current-pipeline artifacts, not new Raven source compilation.
- **Author:** “We can use these translated assemblies to check whether the compiler
  is correct when compiling the source code”.
- **Assistant:** Agrees to use normalized metadata and behavioral comparisons as a
  regression baseline; token/MVID byte equality is not required. Notes that the
  baseline inherits the legacy path's assumptions and is not independent proof.
- **Open:** Direct Raven class-library source emission and broader symbol import.
  [Implemented transport and validation](design/extended-cli-metadata.md#existing-class-library-translation-checkpoint--2026-09-30).

## 2026-09-30: Metadata tooling on both platforms

- **Author:** “We eventually need to figure out how to build neoCLR metadata reader
  and writer support both for .NET and NeoCLR”.
- **Assistant proposal:** use one wire contract and shared conformance fixtures,
  with explicit reader/writer implementations for .NET tools and neoCLR. Distinguish
  native loader/tooling support from a library callable by neoCLR guest programs;
  neither the Python harness nor the .NET development bridge completes that goal.
- **Action:** records the [cross-platform architecture and delivery plan](design/extended-cli-metadata.md#reader-and-writer-support-on-net-and-neoclr),
  including cross-produced artifacts, offline reading/building, token/heap fixups,
  and later Raven adapters. No package name, implementation language or source-sharing
  strategy is settled by this documentation slice.
- **Evidence/context:** the preceding PE probe is committed as `b7f07407`. Ordinary
  .NET/Cecil inspection passes, but Cecil rewriting strips #Neo; that constrains the
  proposed writer architecture. It is not a test of runtime execution.
- **Author clarification:** libraries will serve Raven's compiler symbol loader and
  code generation; neoCLR also needs metadata libraries to load assemblies into
  Introspection and emit assemblies.
- **Assistant action:** makes these the explicit consumer contracts in the metadata
  and Introspection designs. Separates Raven symbol/code-generation adapters from the
  reusable libraries, and requires a neoCLR load-inspect-emit-reopen consumer as evidence.
- **Open:** production format recognition, real dependency binding, both platforms'
  library implementations, native/managed sharing and guest-language API coverage.

## 2026-09-30: Begin extended CLI metadata design

- **Author:** asks to start developing an extended CLI metadata format suitable for
  neoCLR proposal semantics, on a feature branch based on main, for later Raven integration.
  Clarifies: “We need to take into account structural types support.”
- **Assistant:** proposes retaining CLI declarations and adding versioned extension
  relationships and native type expressions. Includes structural arrays, tuples,
  Function types, unions/intersections, synthesized members and independent Object
  compatibility; distinguishes inspection from semantic and execution compatibility.
- **Action/outcome:** creates `codex/extended-cli-metadata` from main `3ac2ae7c`
  in a separate worktree, preserving the author's uncommitted structural-types draft.
  Writes the [design and acceptance sequence](design/extended-cli-metadata.md), updates
  roadmap/bridge/proposal context and records preliminary primary-source comparisons.
- **Author follow-up:** directs “Continue. Commit each slice”, then explicitly permits
  building on `codex/structural-types`.
- **Assistant action:** commits the design and standalone framing codec separately;
  adds structural signature payloads using that branch's Function no-result/output
  contracts and owned/reference array distinction. Fourteen focused tests and the
  fixture inspector pass; no runtime commits are merged or Raven behavior changed.
- **Open:** production byte assignments, CLI embedding, structural semantic decisions,
  runtime execution and later Raven integration. No implementation or format approval
  is inferred from this exchange; no permanent broader reprioritization is inferred.

## 2026-09-30: Independent library backport from Function work

- **Author:** asks to backport useful changes to main, specifically function type
  syntax instead of Func annotations and the OfType extension.
- **Assistant:** selects the source syntax migration and library-only lazy type
  filter for a main-based checkout. Keeps nominal delegate identity/serialization,
  comparer names and main's TypeInfo/MemberInfo surface. Adapts descriptor filtering
  tests to those existing contracts instead of importing FunctionTypeInfo or the
  nominal/structural reflection split.
- **Scope:** the Function runtime, delegate removal, Function target property,
  Function Object semantics remain on the feature
  branch. Query iterator state uses existing boxing, type tests, casts and disposal.
- **Outcome:** 62 query outcomes, 38 native cases and five compiled callback/async consumers pass;
  matching bootstrap/API snapshots and 1,800 website pages are checked. The audit
  also needed its independent generated-method caller/provenance fix from the
  feature branch. Stale Option/union spellings and reflection expectations in the
  query fixtures were refreshed.
- **Author follow-up:** asks which proposals can also move to main, then directs:
  “Yes, but keep them as not completed on main.”
- **Assistant action:** carries Delegates evolved and Callable interface as open,
  unimplemented proposals, links the earlier exploration and updates the proposal
  indexes. Leaves the unfinished structural-types draft and local edits on the
  feature branch. This does not select their full contracts for implementation.
- **Author direction after backport review:** “The idea is that we are going to
  put this experiment on hold and start working on Raven to enable better neoCLR
  target support”.
- **Assistant response:** finishes the bounded backport and records Raven target
  support as the immediate next focus. The Function experiment stays isolated and
  on hold; no new compiler implementation or backend choice is claimed here.
- **Evidence:** [query contract and validation](raven-query-api.md#runtime-type-filtering--2026-09-30-backport).

## 2026-09-27 — Focus on a minimal Web API

- **Author:** “We should focus on the Http story”, specifying the minimum features
  to serve a Web API, JSON serialization/deserialization with nested objects, and
  a WebApplication concept in a separate project with Minimal API endpoints.
  Suggests “Perhaps” a rudimentary SQL interface with a SQLite provider, referring
  to the existing proposal.
- **Assistant proposal:** nested typed JSON first, then a separate application
  layer and a small in-memory API consumer; SQLite remains an optional follow-on.
  Reuse existing HTTP and JSON foundations, revisit the 128-byte JSON bound for
  realistic payloads, and avoid treating modern protocols or the entire SQL
  proposal as prerequisites. Detailed contracts and ordering are assistant proposals.
- **Action/outcome:** updated the authoritative roadmap and theme trackers and
  wrote the [Web API plan](web-api-plan.md), with primary-source comparisons and
  focused acceptance cases. The prior POC remains complete. This exchange changes
  work priority; it does not implement a mapper, framework or database provider.
- **Author follow-up:** requests more presentable website samples organized around
  client/server cases instead of random snippets, then asks to commit and continue
  with the first slice.
- **Assistant action:** makes the existing station-report case lead the Web page,
  shows its tested server alongside the client, and records case-based website
  acceptance and editorial guidance. Nested JSON implementation follows this
  planning/presentation commit.
- **Author presentation correction:** asks for a general HttpClient sample showing
  its capabilities first, followed by a server sample and a client connecting to it.
  The assistant adds a standalone tested client example/capability overview ahead
  of the paired report case and orders that case server-first. The author then
  selects the heading “Case: Building a Http server app” and explains that cases
  should show small real-world or realistic uses in context, without needing large
  samples. The assistant moves the paired case to its own documentation page,
  links it from the client guide and cases navigation, and records the convention.
  The author further clarifies that this applies generally to the website, especially
  feature pages: the case and code should explain what a feature does and how it is
  used. The assistant retains essential server/client code on the feature page, with
  the dedicated case supplying the complete context.
- **Implementation outcome:** the first nested JSON slice is implemented with
  whole-input validation, bounded recursion and unchanged scalar/null policies.
  The focused consumer and nested station-report client/server/independent peers
  pass with zero final live objects; see [evidence](experiments/json-object-mapping/nested-validation.json).
  Website code is extracted from the tested case. WebApplication and SQLite remain
  future work; no release or deployment is implied.
- **Subsequent author direction:** after the payload-budget slice, asks to continue
  with collection serialization/deserialization, then adds enums and a Uuid type
  (noting that the platform does not yet have one), and explicitly asks the JSON
  serializer to recognize Option for JSON-nullable values.
- **Assistant plan:** implement typed arrays first; then enum and Uuid mappings and
  Option values. Proposes None as JSON null, Some as the contained mapping, missing
  Option properties as None, and missing required properties as errors. These are
  implementation choices in response to the author, not a quoted author specification.
  The payload budget is committed as `32e63a1e`; subsequent implementation evidence
  is recorded in the JSON design and changelog as each slice completes.
- **Author scope correction:** “We don't have to go all the way to building a web
  app infrastructure.” Proposes a route parser used within the handler to match
  paths, extract parameters and return their parsed form.
- **Assistant response:** keeps HttpServer as the host and proposes literal/named
  path matching with explicit Int32/Uuid parsing after the JSON work. Updates the
  roadmap and plan to defer WebApplication infrastructure. Route parsing remains
  planned; this exchange does not claim an implemented parser.
- **Author layering refinement:** suggests returning an application union whose cases
  carry route parameters, then clarifies that this is a convenience above basic
  route parsing, useful for match-based application dispatch rather than the
  fundamental result contract. The assistant agrees to retain independent parsing
  and typed value access, with optional application-owned union construction.
- **Author continuation:** asks to continue after that refinement. The assistant
  implements RoutePattern/RouteMatch and a station case with direct matching and
  explicit union mapping; JSON enum/Uuid/Option requests remain pending, rather than
  being treated as cancelled by this bounded routing slice. See the
  [route design](route-parsing.md) and [case evidence](experiments/http-routing/validation.json).
- **Author next idea:** after these primitives, proposes attributes on union cases
  that specify route patterns, with a dedicated route-to-union mapper. The assistant
  records this as the next design investigation above the independent parser;
  metadata retention, case construction, supported types and ambiguity rules remain
  to be validated. No attribute API or automatic mapper is claimed as implemented.
- **Author concrete syntax and reuse:** supplies AppRoutes with RoutePattern
  attributes and an UnmatchedPattern-marked case, then suggests the clearer name
  UnmatchedRoutePattern. Requests constructing a reusable parser object for the
  server lifetime so patterns are not reparsed for each request.
- **Author delegation:** permits Result-based unmatched handling and asks the
  assistant to choose what fits. The assistant selects a planned
  Result<AppRoutes, RouteMappingError> contract: NoMatch is distinct from invalid
  parameter/malformed target errors, while configuration errors fail mapper creation.
  This supersedes the proposed unmatched attribute for the initial mapper design;
  the earlier suggestion is retained here. The current explicit sample's Unmatched
  variant remains an application choice, not the future mapper contract.
- **Open:** attribute-driven union mapping, enum/Uuid/Option contracts, wider
  collection shapes and configurable budgets, and whether to select SQLite after
  the in-memory consumer.

## 2026-09-26 — Streaming, modern HTTP and a finite POC

- **Author:** requests continuation, then asks how the work fits future streaming
  HTTP. After the assistant describes body streaming, clarifies: “I don't mean just
  that, but supporting the newer features of HTTP”.
- **Assistant:** compares HTTP/2/3 requirements with the current HTTP/1.1 provider:
  common message APIs, protocol-specific framing, independent exchange/connection
  lifetime, async bodies, flow control, negotiation and trailers. Records future
  capabilities without treating the question as approval to implement those protocols.
- **Author:** “We should reach a point where we are done for now, for this POC. And
  we should track HTTP work, both client and server capabilities.” Then requests
  thematic consolidation of tracking documents “In a next step”.
- **Assistant action:** establishes the [HTTP capability tracker](http-capabilities.md)
  with client/server status, a bounded finish list and modern HTTP design direction.
  Selects completion of the underway known-length upload increment as the feature
  cutoff; records package/application evidence as outstanding gates rather than
  declaring the POC or release finished. Records thematic consolidation as next
  document-maintenance work, preserving design/test evidence and historical decisions.
- **Implementation evidence:** adds FromStream, explicit ownership and one-shot
  upload admission with 256-byte reads; focused independent-peer cases pass. The
  conditional expression-bodied Length getter emitted zero on the current compiler;
  an explicit getter and consumer assertion restore the required behavior. This is
  a local workaround, not a general compiler fix. Exact validation is retained in
  the [upload fixture](experiments/http-stream-upload/README.md).
- **Validation outcome:** 11 upload cases and additional Disposable/JSON/length
  assertions pass; the bridge conversion to Disposable was corrected after the
  consumer exposed its omission. All 623 signature checks, buffered HTTP/JSON peers,
  API/library snapshots and combined site checks pass. Packaged-SDK POC verification
  remains outstanding; no publication or full runtime suite was run.

## 2026-09-26 — Return to next-release feature delivery

- **Author correction:** “I think we should move forward and not focus on
  optimizations unless matters. We want to add more features for our next release.”
- **Assistant response:** moves feature delivery ahead of further quota profiling
  and timeout investigation. Proposes resuming the already requested stream-backed
  HTTP content feature, followed by bounded application-driven library additions,
  with at most one small companion task. Exact new signatures are not selected by
  this exchange.
- **Action:** updates the authoritative platform roadmap and supporting issue/HTTP
  plans. Preserves the earlier stabilization-first position as history and the
  existing measurements as evidence. Optimization becomes conditional on a concrete
  feature blocker, material supported-workflow impact or release criterion.
- **Outcome/open:** documentation reprioritized; no new API is implemented by this
  change. Known compiler and intermittent timeout findings remain open for release
  review or earlier repair if they block the selected feature. No release date or
  completion claim is inferred.

## 2026-09-26 — HTTP timeout localization and array-budget cost

- **Author:** requests continuation after the repeatability and compiler acceptance
  findings. No deadline-policy or runtime-architecture change is requested.
- **Assistant investigation:** uses a temporary native-service trace and a POST
  sampling profile. A successful baseline run spends 4.7 seconds between server
  request receipt and response-send admission, near the client transfer cap. The
  sampled interpreter work is dominated by aggregate array-budget accounting.
- **Implementation:** replace expanded per-child traversal buffers with borrowed
  sibling iterators, preserving ordering, logical quota counts, per-instruction
  enforcement and GC behavior. Remove trace instrumentation from runtime source;
  retain the diagnostic patch and evidence separately for reproduction.
- **Validation/outcome:** all 44 focused tests pass. Build the mapped apps once and
  alternate baseline/candidate runners on identical inputs. Successful candidate
  client execution takes 5.6–7.1 seconds versus baseline 16.0–18.2 seconds, with
  identical managed counts and zero final live objects. Each runner also times out
  once in three runs; no improved pass-rate or completed timeout-fix claim is made.
  See the [cost investigation](experiments/http-json/array-budget-cost.md).
- **Continued investigation:** five traced runs with the optimized walker pass,
  leaving the previous failure uncaptured. A further POST profile still attributes
  472 of 689 main-thread samples to quota accounting. The assistant implements
  per-slot numeric payload summaries with invalidation on writes/reset, retaining
  instruction-boundary quota checks and the existing deadlines. This is a bounded
  implementation choice, not a separately approved architecture change.
- **Follow-up outcome:** 88 focused mutation/quota/reference/GC tests pass. Both
  previous and summary runtimes pass three new identical-input exchanges; candidate
  client execution is 2.3–5.2 seconds versus 5.9–8.1 seconds, with identical managed
  counts and zero final live objects. Server timing is mixed. See the
  [summary evidence](experiments/http-json/slot-budget-summary.md); no improved
  pass-rate or timeout-root-cause claim is made.
- **Open:** capture an optimized-run failure at the native-operation boundary before
  choosing deadline or scheduling changes. The unresolved-call compiler defect and
  packaged release qualification remain separate. No full suite or website build.

## 2026-09-26 — Issue investigation and neoCLR fix priorities

- **Author:** requests investigation of the neoCLR and Raven GitHub issue lists,
  a fix roadmap explaining priorities and consequences, and work that could fit
  alongside a bigger feature. Clarifies “Focus is on neoCLR”, then directs
  “Update roadmap docs”.
- **Assistant investigation:** reads all 22 open neoCLR and nine open Raven issues,
  compares source and existing experiment records, and identifies partial/completed
  issue scope such as response association, field enumeration and explicit conversion.
  Highlights separately recorded missing-diagnostic/default-body failures and the
  managed-pair timeout evidence. No new runtime reproduction is claimed.
- **Assistant recommendation:** prioritize compiler correctness and the managed
  HTTP/JSON application, then bounded ownership and application-driven API gaps.
  Use narrow RavenDoc/status/constant/helper work as companion tasks; give importer,
  callable/type-system and bootstrapping proposals separate experiments and gates.
  Filter Raven work by its demonstrated benefit to neoCLR.
- **Actions/outcome:** update the [platform roadmap](platform-roadmap.md#issue-driven-priorities--2026-09-26),
  supporting roadmap navigation and the [complete issue triage](issue-fix-roadmap.md).
  Preserve historical checkpoints and distinguish recommendations from selected APIs.
  No compiler/runtime changes, issue updates, commits or releases performed.
- **Open:** execute the bounded compiler reproductions and subsequent application
  checks; settle proposed contracts through their recorded decision gates. The
  instruction to update documentation does not imply approval of every issue design.
- **Author follow-up:** requests a commit and continuation, believing compiler work
  has completed. Assistant confirms the generic-method fix and helper integration
  in neoCLR `afcc8c3d`, refreshes the pending plan before committing, and selects
  repeatability checks as the next bounded task. The independent unresolved-call
  defect remains a separate acceptance item.
- **Continuation outcome:** commit the reconciled plan as `73e27938`, then add
  serial repeat support to the existing HTTP verifier. The first managed-pair
  iteration times out; two client runs against Python pass with zero final live
  objects; all twelve independent server cases also pass with zero final live
  objects. The updated compiler still accepts `missing()` and emits an empty body.
  Observe concurrent background CPU activity and retain it as a limitation rather
  than claiming an isolated run or a known timeout cause. Preserve the
  [focused evidence](experiments/http-json/repeatability-20260926.md) and leave
  unrelated staged experiment build outputs untouched.

## 2026-09-24 — Enable future suspension without implementing it now

- **Author:** clarified that the intention is not to go all the way to runtime
  suspension yet. Requested structure/contracts that allow it later, a clear split
  between public and changeable integration contracts, and behavior that enables
  useful platform features before suspension itself is implemented.
- **Assistant:** agreed to retain generated state machines, separate application-facing
  behavior from transitional compiler/TaskQueue protocols and private implementation,
  and avoid making a general context framework a prerequisite for useful I/O.
- **Actions:** documented those tiers without pretending metadata-visible builder or
  TaskQueue APIs are already private. Added a private, traced ready callback/destination
  slot with acknowledgement only after active-frame installation. Current affinity
  stays unchanged; no saved runtime frame representation was added. All 28 targeted
  scheduler/TCP/worker checks and the combined website build passed.
- **Open:** reusable operation/socket integration, bounded completed outcomes and
  separately validated affinity migration. Runtime suspension remains deferred.

## 2026-09-24 — Runtime async and the scheduler boundary

- **Author:** “Continue. Do you take the direction with runtime async into account?
  And whether we need a scheduler concept instead of just task queues.”
- **Assistant:** acknowledged that generated state machines and TaskQueue are
  transitional. Reviewed the existing producer-queue affinity behavior and the
  latest TCP VM adapter; the adapter proves progress/rooting, not a scheduler model.
  Recommended an internal invocation scheduler with a compatibility queue adapter,
  keeping completion sources independent of callback versus future suspended-frame
  representation. A public Scheduler API is not selected by this recommendation.
- **Actions:** recorded responsibilities, .NET comparisons, tradeoffs, GC/lifetime
  invariants, portable wakeup requirements and a staged migration in the
  [runtime scheduling design](runtime-scheduling-design.md). Updated the roadmap to
  make that boundary the next checkpoint within socket work, and explained the
  distinction and current affinity limitation on the Tasks feature page.
- **Open:** implement and validate the internal boundary, select affinity migration,
  add reusable sockets/Task results and later runtime-owned suspension. These are
  plans; this conversation does not approve every proposed scheduler rule or establish
  that runtime suspension or a public scheduler has been implemented.

- **Follow-up implementation:** extracted a private invocation scheduler with a shared
  rotating completion policy, source GC roots and a durable worker wake latch.
  Retained the existing queue adapter and affinity behavior; runtime-frame suspension,
  public scheduler selection and nonblocking host yielding remain open. All 36 targeted
  scheduler/worker/TCP checks pass, as does the combined website build.

## 2026-09-24 — Socket APIs towards a web application

- **Author:** “Our first goal is the socket api - as per proposal. Keep the proposal
  as the direction.” Directed incremental implementation of the APIs needed for a
  web app running on neoCLR, establishing interfaces and behavior along the way.
- **Assistant:** selected bounded loopback TCP echo as the first application case,
  preserving async I/O, recoverable Result failures, portable sockets and common
  streams from the proposal. Updated the active roadmap rather than retaining the
  older instruction to postpone networking until a general semantics review.
- **Actions and checked outcome:** added the [socket design](socket-api-design.md)
  with candidate guest contracts and an isolated [transport implementation](experiments/socket-api/README.md).
  Eight Rust checks passed on Darwin arm64, with a net10.0 comparison baseline for
  endpoints, cancellation, short reads, EOF and half-close. The refusal fixture was
  corrected after a bound non-listening macOS port left connect pending. Existing
  uncommitted String work was preserved. No production runtime, compiler or public
  API was changed by this slice.
- **Open:** implement the Raven/VM bridge, pending-operation delivery, GC ownership,
  cancellation races and resource quotas before calling the socket API usable from
  neoCLR. The full echo and web-app goals are not complete; later TCP/HTTP details
  remain provisional. The original networking proposal text remains intact.

## 2026-09-24 — Upstream navigation and custom union classification

**Author direction.** Distinguished nested main navigation, authored per-section
`toc.yml` side navigation, and within-page outlines. Requested a simple RavenDoc
navigation model rather than copying every DocFX concept. Directed that general
improvements land on Raven main and then its neoCLR branch. Clarified that
StreamError and similar carriers should carry UnionAttribute and load as
IUnionSymbol, with cases presented as unions. Requested I/E/U/D/S type icons,
then C for classes; static membership remains an icon badge.

**Assistant implementation.** Added the shared navigation and compact browsing
features in Raven commit `c5838d3c7`, integrated locally on main, and brought them
and the preceding site publisher to the neoclr branch (`3db61296b`, `d8f31e475`).
Replaced the neoCLR publisher overlay with the upstream configuration. Added
independent class/struct metadata fixtures for the typed-case contract; corrected
a same-name case filter that could hide System.Storage.File. Updated runtime and
reference markers without changing carrier storage. Recognition and documentation
projection are separate from extraction lowering; no new pattern ABI is claimed.
The general union/icon fix was then committed on Raven main as `c629dc729` and
cherry-picked to neoclr as `927c99df2`; 23 focused compiler/publisher tests passed.
Validation and the final immutable generator pin are recorded with the code.
The author subsequently requested the Raven website's exact snippet rendering
method and then a view of the resulting site. The assistant extracted the shared
Highlight.js grammar and colors into assets consumed by both website and RavenDoc,
with a local pinned engine. The author directed that neoCLR union changes be
committed separately from the website migration. Bootstrap validation required
direct case construction and the existing upstream configured-unit identity fix
for imported Flush contracts, plus structural array identity for stream
interface implementation. The resulting bootstrap instruction fragments remain
unchanged. The author additionally requested an N favicon, shared dark-mode
support, a compact Light/Dark/Auto icon menu, an updated RavenDoc sample site and
updated RavenDoc documentation. These are implemented in the generic publisher;
neoCLR retains its branding. The union/runtime changes were committed separately in neoCLR as `417a3f9a`.
Raven main now contains the shared UI/sample/docs work at `69717b4df`, brought to
the neoclr branch as `1fb1f9904`. The combined neoCLR site passed 491 page/link
checks and 10 publisher tests. These local integrations are not publication or a
push to the remote repository.

## 2026-09-24 — One RavenDoc website, with page metadata and release status

**Author direction.** Requested migration to RavenDocs with appropriate customization
and an up-to-date build in the repository for CI/CD. Clarified that content should
be Markdown and the reference part of the same site, with no separate DocFX site.
Required clarity when documentation is published before its corresponding release.

**Assistant proposal and first implementation.** Located Raven's RavenDoc publisher,
probed its assembly/XML input and began converting the existing content. Proposed
an immutable portable publisher build, a reviewed site extension, shared branding
and a visible development notice. The first content pass carried over too much of
the old website's structure.

**Author corrections.** Said, “You can't just port the website content as is. We
need to adapt it”. Then requested HTML-page support and front matter, including
control over the page outline, as a CMS-like authoring capability. Clarified that
a custom landing page could use Markdown or HTML through that mechanism. Required
a hero and feature boxes, no purposeless outline or enclosing landing-page panel,
and a distinct hero background with feature boxes beneath.

**Documentation organization.** Asked to see the real generated API site with XML
or Markdown documentation, keeping feature/API guides alongside type and member
pages. The assistant built and opened the integrated API entry point, with guide
and generated-reference links together. Existing reference gaps remain explicit;
this migration does not claim complete coverage of every runtime API.

**Visual feedback.** The author pointed to the Raven and CloudShell landing pages
for inspiration, accepted the improved structure, criticized the colors and an
intrusive development notice, then said, “The boxes below are OK.” The assistant
retained the feature cards and adjusted the hero to slate/blue and the notice to
a compact development-status line.

**API navigation feedback.** The author found declaration modifiers distracting in
API lists and directed names by default, with icons indicating symbol kinds and
full signatures opt-in at generation. They then requested property and field types in the
form `MyProp: <type>`. The publisher follows this distinction and keeps full
declarations on detail pages. The author subsequently refined this to simple
name-first signatures such as `MyFunc(x: int) -> ()`, retaining type annotations
and parameter lists while omitting declaration keywords and distracting modifiers.
They noted that static does matter; the assistant added a trailing `[static]`
annotation to preserve name-first browsing. The author then suggested marking the
icon instead; the assistant agreed and implemented an `S` badge with a tooltip and
an accessible static-member label. Overloads now retain separate entries. The
author also requested a distinct interface icon; interfaces use an I glyph on
lists and detail pages. The author then requested a left-side API Browser with
expandable namespaces, explicitly inside RavenDocs, with an off-canvas default on
small screens and a modern documentation-site presentation. The assistant added
shared publisher navigation with active-type highlighting and a modal mobile drawer.

**Work and evidence.** Added Markdown/HTML content and scalar front-matter controls
in the repository-owned RavenDoc customization, a checksum-verified portable build
and updater, shared page chrome, generated reference and legacy redirects. The
[website build](../website/README.md), [API maintenance](../api-docs/README.md) and
[publisher source/pin](../tools/ravendoc/README.md) document the workflow. Local
publisher tests and complete-site link validation are the acceptance checks; this
entry does not claim CI execution, upstream integration or public deployment.
Publishing remains a separate manual operation on main. The author-directed task
does not change application-milestone ordering.

## 2026-09-24 — Nullable value declaration policy

**Author request.** Following the direction to prefer Option for absence and defer
nullable structs, the author requests an option in Raven to diagnose value types
when declared nullable: “Value types can’t be declared as nullable”.

**Assistant response and action.** Proposed an opt-in compiler policy while retaining
Raven's default .NET behavior and nullable reference annotations. Implemented the
general AllowNullableValueTypes option on an isolated main-based Raven branch, with
RAV0407, project settings and CLI switches; integrated the general changes separately
into Raven main and the neoCLR integration branch. neoCLR's shared build props select
false. Also completed the previously selected record Object.Equals annotation fix,
validated independently against ordinary .NET metadata before target integration.

**Evidence and limits.** The general policy passed 105 focused nullable/project checks,
a project-save check and five CLI default/override cases. Target integration exposed
a generated record-struct unboxing conversion failure; the assistant fixed narrowing
after the existing exact-type guard on the experimental branch. All 117 focused
integration checks and the record/Object sample checks then passed. See the
[integration notes](experiments/raven-target/README.md#development-compiler-policy-2026-09-24)
and [nullability policy](nullability.md#current-declaration-policy-2026-09-24).
This is source declaration validation; it does not implement nullable values, settle
metadata redesign, or promise Option components in generated records. Packaged SDK
updates and publication are separate work.

## 2026-09-24 — Future minimal HTTP application namespace map

**Author direction.** Described a future minimal HttpServer application using
System.Cryptography if HTTPS is supported, System.Data.Json,
System.Networking.Sockets, System.Web.Http and System.Text for encoding. Identified
potential longer-term needs for a better time API, String methods and StringBuilder,
and suggested that a conceptual WebApplication would probably live in System.Web.

**Assistant action and interpretation.** Extended the existing namespace
consideration in the [platform roadmap](platform-roadmap.md#minimal-http-application-dependencies--consideration-2026-09-24)
and linked it from the detailed HTTP plan. Recorded the names as candidates and
WebApplication as a layer above HTTP. Added a bounded .NET comparison and distinguished
TLS transport from cryptographic primitives. Suggested deadlines, HTTP dates, token
parsing and response construction as future validation cases; these remain assistant
proposals. No APIs were implemented, current work was not reprioritized, and exact
time/text contracts, TLS placement and application framework scope remain open.
Reviewed the website namespace overview: it intentionally inventories implemented
APIs, so these candidate namespaces remain in planning documentation.

## 2026-09-24 — Map record interfaces and extend component semantics

**Author question and direction.** Asked whether Raven should map the right
interfaces for records, then directed continuing the next slice.

**Assistant proposal.** Explained that Equatable<Record> was already mapped;
component comparison was the next gap. Proposed strings and nested record classes,
with nullable/boxed semantics and record structs to follow as separate work.

**Action.** Extended the target-specific synthesis on Raven's experimental branch:
string content equality/hash, typed nested equality/hash/display and declared
reference outputs for deconstruction. Added forward-reference and metadata-injected
null regression cases. Retained explicit diagnostics for nullable string/value and unsupported
component types. A follow-up admits nullable record references using existing
reference storage, and typed importer adapters preserve literal null argument
positions in constructors and calls. Validation passed: 46 compiler tests, 32 reference-slot tests, the expanded neoCLR
sample, nested-record editor completions and the combined site/API build. A separate
local SDK workspace preserves the prior snapshot. No general comparer API or Raven
main integration is claimed.
See [the decision and tradeoffs](hash-code-design.md#string-and-nested-record-components--2026-09-24)
and [the executable sample](experiments/records/README.md).

**Next issue.** The author directed continuing with the next issue. The assistant
selected boxed-value equality/hash and implemented a bounded Int32 compatibility case.
It retains box identity, copies and class defaults, while virtual Object calls compare
Int32 payloads and return the integer hash. Twelve runtime cases, 22 pinned .NET
assertions, the Raven sample and site/API build passed. This is a narrow interpreter
intrinsic, not a completed named-value-type override model or record-struct support;
see the [tradeoffs](object-model-review.md#boxed-int32-equality-and-hash--2026-09-24).

## 2026-09-24 — Refresh the SDKs for VS Code testing

**Author direction.** Said it was soon time to update the neoCLR and Raven SDKs
so they could test the latest work in VS Code.

**Assistant action and outcome.** Prepared an isolated local development bundle
with matching runtime, library, reference assembly, bridge, compiler and language
server. Added saved records, Storage and Console projects with build/run tasks.
All three projects built and ran; stdio language-server checks returned HashCode
and Concurrency completions. The Storage check exposed a Console.Error/Result.Error
name collision, fixed by qualifying Console calls in the maintained sample.
The compiler's supported record slice remains limited to integer record classes.
This is a local SDK snapshot, not a published release or a manual VS Code UI test.
See [the local workspace instructions](local-sdk-snapshot.md). Other SDK installations
were left in place.

## 2026-09-24 — Grow a website namespace overview with the APIs

**Author.** Requested a website namespace overview explaining what namespaces
contain, starting with the existing APIs and adding namespaces as APIs are added.

**Assistant/action.** Added an [on-site overview](../api-docs/namespaces.md), linked
from the homepage, API landing page and navigation. It describes current application
namespaces and runtime/compiler support, identifies incomplete member coverage,
and keeps proposed namespaces outside the implemented inventory. Corrected the
old namespace table's treatment of Console: it is a class in System. Recorded
maintenance guidance so the list grows with implemented APIs. This documentation
request does not change platform milestone priorities; publication remains separate.

**Validation.** The API snapshot check and combined website/link build passed in a
temporary copy using the committed compiler/runtime baseline plus these documentation
changes. The working copy snapshot is stale alongside concurrent compiler/runtime
edits; those edits were left untouched. Website tokenizer tests passed.

## 2026-09-23 — Queryable information without a filesystem-shaped core

**Author.** Suggested ways to fetch additional information/attributes from storage
objects, drawing on System.IO and Windows Runtime, while keeping the Storage model
abstract and independent of a particular backing model.

**Assistant.** Proposed treating metadata access as an extension to storage
capabilities, separating represented state from provider queries. Recorded typed
common information, optional interfaces and extensible queries as alternatives;
no signature or property-bag design was selected by the author.

**Action.** Added the direction and primary-source comparisons to the proposal,
roadmap and on-site future direction. Recorded availability, failure and freshness
as contracts to resolve. The interface/provider migration remains next; no metadata
query API is implemented by this update.

## 2026-09-23 — Select the proposal's interface-based Storage model

**Author.** Directed continued work toward the model in the Storage proposal, then
clarified: “File and Directory are interfaces extending StorageItem. They form a
closed hierarchy.” Existing classes should become provider-specific implementations.
Also identified GetItems returning StorageItem values and endorsed this as the
modern abstraction for different kinds of storage.

**Assistant response.** Recognized that the current byte-first StorageProvider and
separate StorageLookup do not express that model. Initially began moving lookup
onto a renamed provider contract, then discarded the unfinished edits after the
interface clarification; those edits were not committed. The last working descriptor
implementation remains intact. Clarified the distinction between a closed set of
item kinds and extensible provider implementations, and between single GetItem and
mixed-item GetItems enumeration.

**Action.** Updated the proposal and authoritative roadmap to select the public
interface hierarchy and provider-resolution model. Corrected the proposal's class
examples. Recorded the next migration and the required closed-root/open-branch
validation. Updated the website/API guide to distinguish the selected direction
from the current concrete implementation. No interface migration or GetItems
implementation is claimed by this direction update.

**Author clarification.** Providers may use concrete implementation classes
internally, but their contracts must expose the interfaces. The assistant confirms
that current GetFile returns Result<File, StorageLookupError>; future directory,
generic-item and enumeration contracts must similarly expose Directory/StorageItem,
without requiring consumers to name or cast to implementation classes.

**Subsequent implementation.** After the author's instruction to continue after the
metadata slice, migrated StorageItem/File/Directory to the selected interface
hierarchy. Moved descriptor behavior into sample provider implementations. Added
compiler/importer closure checks and mixed-item tests, without claiming raw neoIL
enforcement. Native static text helpers move to FileText in the development Raven
surface because the runtime cannot put static methods on an interface; legacy raw
aliases remain. Provider resolution and enumeration are still follow-up work.

**Open.** Concrete enumeration/task signatures, enforcement across compiler/importer/
runtime layers, and static helper compatibility during migration require implementation
validation. Optional filesystem topology and rich metadata remain deferred.

## 2026-09-23 — Minimal Storage surface and Windows Runtime lessons

**Author direction.** Learn from Windows Runtime APIs without adopting their full
surface: implement immediate needs, select file/directory properties for their value,
and “keep it very simple at start.” The author also permits work on the next slice
once the current one is done.

**Assistant proposal.** Separate provider-bound descriptors from opened streams;
start with Path and useful File.Name, explicit lookup and byte access. Defer cached
metadata, timestamps, sizes, enumeration and query objects. Review the sample's
redundant provider descriptor factory and mandatory text helpers before integrating
the provider contract. The sample subsequently implements derived names and removes
the provider factory;
these remain sample contracts pending platform integration. Removing mandatory
whole-text provider helpers remains proposed.

**Action.** Added a [WinRT/.NET comparison and scope decision](experiments/storage-provider/README.md#minimal-storage-design-winrt-lessons-2026-09-23)
and began integrating InputStream/OutputStream as shared platform interfaces,
removing sample disk adapters. File/Directory/provider integration remains the next
slice. Integration exposed an array-symbol identity bug in Raven, reproduced independently
with an imported ordinary .NET interface. The assistant fixed and tested it on a
main-based branch (21 focused checks), integrated `f0c06f7f8` into Raven main and
cherry-picked it as `5fa6516fe` into the isolated neoCLR feature branch. No neoCLR
policy was merged into Raven main. Flush subsequently exposed configured-unit
identity inside imported generic returns; the independent opt-in ValueTuple
regression and fix passed 39 focused checks, landed as `44ae9f242` on main and
`849347a97` on the integration branch. Platform validation is recorded in the changelog.

**Subsequent integration.** Added System.Storage.StorageProvider with only OpenRead
and CreateNew. The disk/memory sample extends it with provisional lookup/text
methods; its byte operations now dispatch through the core interface. Added generated
API documentation and a contract check whose provider has no text or lookup methods.
This implements the proposed separation at the platform boundary. File/Directory
integration and lookup capabilities remain next; this is an assistant implementation
choice within the requested POC, not an author-approved final Storage design.

**Descriptor integration.** The assistant subsequently integrated File and Directory,
preserving File's static native string helpers. Chose StorageLookup as an optional
interface extending byte access with GetFile, so byte-only providers remain valid.
Moved sample text conveniences out of descriptors and retained them on fixture
providers. Directory.FileAt now uses StorageLookupError. Added generated API member
coverage and a manual WriteAllText entry for DocFX's unit-result limitation. Concrete
host-provider integration remains next; richer metadata is still deferred. This
separation remains exploratory rather than a final author-selected API shape.

## 2026-09-23 — Storage and Streams as architectural exploration

**Author clarification.** “Look at this as exploration also, to investigate what
is best, and consistent with the architectural direction.”

**Assistant response and action.** Treat the disk application as evidence for
provider boundaries, directional stream capabilities, ownership/disposal, typed
errors and compatibility with future suspension/scheduling. Mark backend transport,
buffer shape and blocking dispatch as provisional, not public API commitments.
Recorded these evaluation questions in the roadmap and
[file-resource experiment](experiments/file-streams/README.md). The first host and
VM tests pass; public platform wrappers and the full acceptance application remain
pending. At the author's request to continue into the next slice, the assistant
built an application-owned Raven provider experiment. The same workflow runs against
disk and memory. Review exposed host path rules leaking into a generic Directory;
child-address resolution was moved into the provider and the tested revision keeps
memory addresses distinct. The provisional contract and tested source are browsable
in `/docs/`; byte-stream integration remains pending.

## 2026-09-23 — Public API documentation policy

**Author direction.** Add System.Concurrency to the API docs, keep documentation
updated as policy, and make it easily browsable. The author clarified: “So APIs
should be covered by the API docs”.

**Assistant action.** Added the namespace and Thread/ThreadPool member pages,
namespace links on the reference landing page, and an API-wide navigation label.
Recorded the requirement in AGENTS.md and the maintenance guide: public API changes
include reference coverage; renderer exceptions need linked manual entries. Listed
older coverage gaps explicitly rather than claiming the existing Tasks-focused
reference covers the whole library. Backfilling those gaps remains work.

## 2026-09-23 — Post-release concurrency, Storage and file streams

**Author direction.** Implement the concurrency rename, Task.Run and explicit
Thread APIs, Storage File/Directory abstractions, and a first Stream API for file
access. The requested end product is an application reading and writing disk files.

**Clarification.** Asked whether Task/Promise should remain in System.Tasks, the
author answered: “Keep System.Tasks; move Thread to System.Concurrency”.

**Assistant action.** Updated the authoritative roadmap with that sequence and
started the concurrency implementation. The retained Thread owns its completion
Task before Start; native completion must include thread teardown. Storage and
Streams implementation and the end-to-end file application remain pending.
**Subsequent author correction.** Task.Run should have overloads and return values,
“just like in .NET”; the author expects to revisit it with a clear suspension and
scheduling model. The assistant removed the uncommitted string-only Task.Run
facade and recorded the broader contract as pending design, including captured
state and async-delegate behavior. The author further described Task.Run as “spawn work concurrently”. The assistant
recorded that submission starts work and await observes completion. This does not
defer explicit Thread, Storage or Streams work.

**Implementation outcome.** Explicit Thread now retains its pending Task before
Start, rejects a second start and completes on the construction queue after native
thread termination. Eight source contract cases and eleven default-queue cases
pass. Runtime/API snapshots and development documentation were refreshed. The
website retains its Preview 9 worker sample separately from the development API.
Storage, Streams and the final file application remain pending.

See [the checkpoint](platform-roadmap.md#post-release-concurrency-direction--2026-09-23).

## 2026-09-23 — Continue async release preparation

**Author request.** “Continue with the release”.

**Assistant actions.** Pushed the corrected implementation candidate e9bb28a to a
separate release-validation branch for six-job source CI. Rebuilt and checked its
package, including an interactive VS Code build/run and Task type hover in a named
profile. Recorded results and a release-note draft. Asked for a version preference,
suggesting 0.1.0-preview.9; no answer was available at preparation time. The assistant
used that next preview number as a candidate-preparation choice, not an attributed
author decision or publication approval. Updated Cargo metadata and kept publication
pending the versioned candidate's own validation.

**Open.** Final candidate source/platform checks, artifact hashes and publication.
See [the release plan](async-preview-plan.md) and [candidate evidence](async-preview-candidate-validation.json).

## 2026-09-23 — Broader concurrency and tracked threads after release

**Author direction.** “After the release, we will rename System.Threading as
System.Concurrency. This signals a broader area”. The author noted that Thread may
not be available on every platform and suggested `System.Concurrency.Threads` as a
possible package, explicitly not necessarily its namespace. Two scenarios are
needed: executing work for a result, and retaining a Thread object to control and
await its termination.

**Author clarification.** Proposed `Thread.Run(() => 42, ...)` for the first case,
and construction followed by `thread.Start()` and `thread.Task` for the second.
The first example annotated an awaited result as `Task<int>`.

**Assistant response and actions.** Distinguished the returned `Task<int>` from the
`int` produced by awaiting it. Recorded the selected post-release namespace rename
and the proposed API shapes in the [roadmap](platform-roadmap.md) and
[concurrency direction](concurrency-direction.md), with current isolated-worker
limits, .NET comparison and sample/validation questions. Updated website future
direction. No runtime APIs were renamed or implemented.

**Author further clarification.** Concurrency should encompass alternatives to
threads. The platform should primarily expose abstractions and expose lower-level
primitives only when appropriate. Suggested workers or tasks for a possible
WebAssembly target without a public Thread API; this was exploratory, not a
blanket claim about WebAssembly support.

**Assistant interpretation and action.** Recorded separate roles for Task completion,
worker execution and explicit Thread lifecycle. Proposed reconsidering the placement
of portable result-oriented submission rather than fixing Thread.Run as universal.
Clarified that Tasks do not inherently create parallel execution. Updated the
roadmap, design note and website; backend and public API choices remain open.

**Author submission follow-up.** Suggested that Task remains useful for any work
and could offer a method similar to Task.Run, with the platform determining how it
executes concurrently.

**Assistant response and action.** Recorded Task.Run-style submission as the preferred
portable direction to explore, preserving explicit thread lifecycle as a separate
scenario. Updated current design and website wording while retaining the earlier
Thread.Run suggestion here. Scheduling, progress and isolation need a portable
contract; concurrent execution does not necessarily promise parallel execution.
This is a post-release proposal, not an implemented API.

**Author confirmation.** System.Concurrency is the namespace for concurrency,
including threading. Thread remains an explicit thread API that may not exist on
all platforms. Task is a general abstraction and API, not inherently tied to threads,
and will provide concurrent execution using the platform's mechanism; a WebAssembly
target might use Web Workers.

**Assistant action.** Made these roles explicit in the roadmap, design note and
website. Task's concurrent-submission role is selected direction; exact method
signatures and a Web Workers implementation remain undecided and unimplemented.
A separate public Worker API is not a prerequisite for that Task backend.

**Open.** Scheduling policy for Run, result types, lifecycle control, ownership,
platform capability and migration contracts. Package naming is tentative. This does
not add a release gate or replace the Streams/Storage/Encoding sequence.

## 2026-09-23 — Plan an async and Tasks release checkpoint

**Author request.** “We should also decide when to make a new release focused around
async and task bits”. This arrived while the assistant was validating host
cancellation during worker-result delivery.

**Assistant recommendation.** Start stabilization of the current supported Task
surface now and release when a fresh extracted evaluator bundle and exact-candidate
validation pass. Do not wait for HTTP or all of M1. Keep the ordinary isolated-worker
APIs with their documented blocking joins; leave the notification adapter, guest
operation cancellation tokens and real I/O experimental or unimplemented.

**Actions.** Committed the bounded runtime fix as `13c41ed` after 60 targeted tests,
a host/guest-IL sample and website checks. Added [a release checkpoint plan](async-preview-plan.md),
linked it into the authoritative roadmap, recorded package/platform/documentation
gates and corrected stale Task documentation about cancelled await and default dispatch.
The website now explains the planned release boundary.

**Author follow-up decisions.** Suggested focusing on Streams and Storage APIs,
perhaps Encoding, before networking. Then accepted the proposed release scope:
“But your scope for the release looks great. Make that the immediate priority”.
The roadmap now places release stabilization first and those API foundations before
networking afterward; this does not adopt the proposal texts as final contracts.

**Website maturity wording.** The author requested a “preview” label beside neoCLR
in the header, then suggested “experiment” as the better term. The assistant used
“Experimental” beside the name on every page, retaining preview terminology for
individual releases. The author also clarified that neoCLR encompasses application
and development workflows, not only the runtime and class library, and selected
the phrase “Experimental application platform”. The homepage and overview now use
that framing while separating implemented components from future aims.

**Open.** Candidate revisions, version/date, clean packaged evidence and final release
validation remain unset or open. No tag, push, package publication or website deployment is implied.

## 2026-09-19 — Implement the first feature guide

**Author direction.** Continue after recording the feature-page plan.

**Assistant action.** Built the first Introspection guide with one runnable Raven
walkthrough, generated excerpts, shared expected output, downloads, design tradeoffs,
preview limits and feedback questions. Updated the homepage's superseded Type/Info
and class-based descriptions, and added nested-page/anchor validation to the site
builder. The guide is a local development artifact; no push or publication occurred.

**Follow-up question.** The author asked whether TypeInfo already extends MemberInfo.
The assistant clarified that it remains separate: the sealed MemberInfo cases are
FieldInfo, MethodInfo and PropertyInfo. Nested-type membership is a future option
requiring a declaring-type contract that also handles top-level types. The website
work does not change this API boundary.

## 2026-09-19 — In-depth feature pages for the next release

**Author proposal.** Create dedicated feature pages for the next release, going into
particular APIs in depth and showing more substantial samples.

**Assistant response and action.** Proposed a page structure covering the problem,
runnable walkthrough, API behavior/tradeoffs, preview limits and feedback questions.
Suggested Introspection first, drawing code from tested samples, with strings after
its later API/runtime story. Recorded a [content and delivery plan](design/feature-pages.md)
and linked it from the roadmap/design index. No feature page or website publication
is claimed; this documentation planning slice is separate from the closed API work.

## 2026-09-19 — Close the Introspection story

**Author decision.** With discovery, metadata tokens and Sequence collection contracts
in place, close this story for now. The String API and runtime string behavior are
a later story.

**Assistant response.** Accepted that boundary. Finish validation, keep discovery
and Sequence migration as separate commits, and refresh the runnable VS Code snapshot.
Do not begin string implementation or dynamic loading as part of closing this story.

## 2026-09-19 — Finish the Introspection collection migration

**Author direction.** “Migrate the remaining Introspection contracts from returning
array to sequence.”

**Assistant action.** Committed the preceding discovery/token slice separately as
92bd616, then changed the remaining TypeInfo/member collection contracts to
Sequence<T>, including parameter and enum-name results. Updated reference metadata,
Raven implementations, samples and checks together. Preserved native array storage
internally, independent snapshots, indexing, enumeration and query extensions.
The migration replaces Length with Count and requires consumer rebuilds; it does
not introduce covariance or promise immutable concrete storage. Dynamic loading
remains future context work, and strings follow the minimal Introspection boundary.

## 2026-09-19 — Dynamic loading remains context-owned future work

**Author direction.** “We might need to solve dynamic assembly loading later. It does
belong to RuntimeContext”.

**Assistant response.** Agreed that future loading/resolution belongs to RuntimeContext,
while AssemblyInfo remains descriptive. Kept this preview's queries over the already
loaded catalog; no loader, search policy, unloading or API signature was implemented
or selected in response. Unavailable references currently produce an explicit fault.

## 2026-09-19 — Finish minimal introspection before strings

**Author direction.** Implement ExecutingAssembly, ReferencedAssemblies and
MetadataToken before moving to strings; finish the minimal Introspection slices.
The author reiterated considering interface collection returns rather than arrays.

**Assistant response and action.** Kept these as the remaining preview boundary and
selected Sequence<T> for the new assembly/module collections because it supports
iteration, Count and indexing without mutation methods. Internal snapshot arrays
remain permitted. Implemented the supporting source assembly/module/token metadata
and validated import/execution, artifact round trips and scope checks. Public
assembly discovery and token interface properties remain in progress, so strings
have not started. The completed TypeInfo acquisition slice is f8104d1; a fresh,
verified VS Code workspace is available at ~/.neoclr/experiments/typeinfo-20260919/demo.

**Subsequent implementation.** Added the two sealed assembly/module interfaces and
Raven providers, ExecutingAssembly, direct ReferencedAssemblies and MetadataToken on
Info interfaces. The running sample reports Demo referencing System.Runtime and its
module's Widget type; editor checks expose all eight interfaces. New collection
returns use Sequence<T>. The assistant selected an explicitly bounded loaded-catalog
reference model: an unavailable reference faults, rather than loading or disappearing.
Discovery covers retained definitions; old array returns remain for later review.
Source-caller, token, generic-definition, resource and admission tests accompany the
change. The [implemented contract](introspection-design.md#minimal-discovery-and-token-apis-implemented--2026-09-19)
records these assistant implementation choices and their compatibility costs.

## 2026-09-19 — Assembly references and Info metadata tokens

**Author additions.** Requested AssemblyInfo.ReferencedAssemblies, if that is the
right owner, and specified that the executing assembly must show System.Runtime as
a reference. Then requested MetadataToken on the Info classes and clarified: “MetadataToken
should be on the interfaces.”

**Assistant response and action.** Agreed that references belong to assembly metadata,
with RuntimeContext owning runtime discovery. Committed to preserving direct references
and the bootstrap-to-System.Runtime identity mapping without implicit loading.
Explained that MetadataToken is module-scoped and must not be a renamed
DefinitionIndex. Recorded primary .NET comparisons and the bootstrap complication:
separately compiled library slices can reuse source token rows when merged. These
requirements are part of the next discovery/metadata slice, not implemented claims.
See [the contract and validation needs](introspection-design.md#assembly-references-and-metadata-tokens--2026-09-19).

## 2026-09-19 — Unified type acquisition implemented

**Author corrections.** Remove the remaining Type class, implement Object.GetType
(the selected spelling replaces GetTypeInfo), and expose ExecutingAssembly on
RuntimeContext. Keep introspection the common foundation for later reflection and emit.

**Assistant action and outcome.** Replaced the Raven public/runtime Type class with
TypeInfo and migrated structural signatures and saved samples. Added Object.GetType
and the production RuntimeContext.Current handle resolver. Validated allocation
identity through base/interface, boxed, string and array references; fixed native
snapshot materialization to preserve the public interface view. All 73 slices
regenerate reproducibly, 62 focused runtime tests and 33 admission cases pass.
The saved-project suite passed 43 cases before exposing the native-result defect;
after correction, all 31 remaining cases passed. Editor checks hide public Type,
its old Info hop and runtime providers, while identifying Info interfaces correctly.

**Remaining work.** ExecutingAssembly and assembly/module discovery are not part of
this completed acquisition slice. The compiler reference still needs an internal
empty System.Type shell for CLI closed-hierarchy attribute tokens; it is not a
public API or executable runtime type. The historical Neo profile remains separate.

## 2026-09-19 — A coherent API foundation for preview feedback

**Author clarification.** The unified introspection API should later expand into
reflection and emit. The author reaffirmed string APIs as the next step after this
foundation: the next preview should let evaluators understand the API direction
and give feedback about what could work better. In the preceding corrections,
the author requested removal of System.Type, selected `Object.GetType()` (replacing
the earlier GetTypeInfo spelling), and requested `RuntimeContext.ExecutingAssembly`.

**Assistant response and action.** Kept type acquisition and assembly discovery in
the current implementation scope, with invocation and emit deferred. Recorded
introspection followed by a minimal string surface as the preview sequence, with
runnable examples and explicit provisional contracts as the basis for evaluation.
Updated the current design spelling while preserving earlier decisions in this
record. The acquisition migration is in progress; this entry does not report its
completion, a finished string API, or a published preview.

## 2026-09-19 — String handling after basic introspection

**Author direction.** After the basic Introspection API, focus on neoCLR string
handling and implement a minimal UTF-8-related surface. The author left encoding
undecided and explicitly said not to add specialized Utf8String classes yet.

**Assistant response and action.** Preserved introspection as the current priority,
recorded string handling as the next step, and narrowed the implementation scope of
the larger string proposal without deleting its earlier ideas. Encoding and Char
representation are not selected by this message. No string implementation change
was made in response to this sequencing instruction. See [the roadmap](roadmap.md#immediate-sequence-clarified-2026-09-19).

## 2026-09-19 — TypeInfo as a member

**Author proposal.** “We could technically make TypeInfo part of the MemberInfo
hierarchy so it can represent a nested type.”

**Assistant response.** Agreed that this would allow nested types in member queries
and add TypeInfo as a fourth public match case. Identified the required ownership
change: top-level types have no declaring type, unlike nested types. Recommended
optional ownership in the unified TypeInfo model, compared .NET's corresponding
inheritance and nullable DeclaringType, and recorded a separate wrapper as an
alternative. The existing three-case implementation is not evidence that this
extension or the unified acquisition API is complete. See [the proposal](introspection-design.md#proposed-typeinfo-member-case--2026-09-19).

## 2026-09-19 — Exhaustive matching over the public member cases

**Author clarification.** “So if you have a MemberInfo and you do a match, then you
must cover all inheriting interface types.”

**Assistant action and evidence.** Removed the shared storage base from MemberInfo's
direct implementations so its permitted cases are exactly FieldInfo, MethodInfo
and PropertyInfo. Compiled a separate consumer with three public interface arms
and ran it against real descriptors. Removing each arm produces RAV2100. Added
reference type-test execution to neoCLR because the importer previously rejected
Raven's emitted isinst instruction. Recorded the remaining generic Raven diagnostic
issue: errors currently name hidden leaves rather than the public covering interfaces.
See [the contract and limitation](introspection-design.md#exhaustiveness-over-memberinfo).

## 2026-09-19 — Seal the introspection model

**Author decision.** “At the moment, we will model the introspection model as a
sealed hierarchy, which Raven supports also for interfaces.”

**Assistant response and action.** Changed the in-progress public Info declarations
to sealed interfaces and internal providers to closed families/leaves. Replaced the
proposed external-provider sample with a rejection case, and began preserving and
checking Raven's permitted-type metadata in the target reference. Recorded the
extensibility tradeoff and the distinction between Raven/importer enforcement and
raw neoIL runtime enforcement in the [design](introspection-design.md). This decision
does not settle the separate collection-return question or complete RuntimeContext.

## 2026-09-19 — Collection interfaces versus array results

**Author question.** During the Info-interface migration, the author asked whether
APIs should ever return arrays, or choose an interface conveying the collection.

**Assistant response.** Recommended capability-based result contracts, with existing
Sequence<T> (iteration, Count, indexed reads) as a candidate for materialized
introspection results. Distinguished read-only access from immutable storage and
snapshot semantics, and retained buffers/caller-owned storage as possible array
uses. Compared .NET's array/enumerable reflection surfaces and documented costs and
validation needs in the [design](introspection-design.md). This is a proposal; the
author has not yet selected a return type or a library-wide array policy. The
ongoing provider migration retains existing array results pending that API slice.

## 2026-09-19 — Close the source-port execution gate

**Author direction.** Continue implementation, retaining separately committed slices
and the requirement that programs targeting neoCLR run.

**Assistant action and outcome.** The complete source-port Rust baseline exposed a
lost declaring-type relationship on imported Option/Result cases. Preserved their
lexical nesting in target output (`d21c6de`), verified exact owner identity for all
four cases, regenerated 73 source slices and passed 12 union-admission and 34 focused
runtime tests. Raven already emitted correct nested CLI metadata; only its feature
branch documentation changed (`819e73ac6`). The final Rust batches now total 1246
passing tests across 178 integration binaries plus unit tests, with no unresolved
failure. All 65 saved-project cases also pass against the corrected library.
See [the validation record](raven-library-port-validation.md) for the
scope of reruns and the remaining API boundary.

**Still open.** RuntimeContext/AssemblyInfo discovery, the public Info-interface
migration, canonical Object.GetTypeInfo acquisition and the refreshed VS Code API
are separate work. Source-port execution success does not establish those APIs.

## 2026-09-19 — Optional services associated with RuntimeContext

**Author suggestion.** “Other things might later be tied to the runtime context.
Perhaps an optional GC, instead of a GC class.” This is a future possibility, not a
request to remove the existing collector or implement a GC-free execution mode now.

**Assistant response and action.** Recorded an optional context-associated collector
capability in the [maintained design](introspection-design.md), comparing it with
.NET's static System.GC API. Proposed separating capability availability from the
underlying memory-management policy. Heap scope, shared collectors, rooting and
lifetime rules remain open, as do API names and representation of absence. This
documentation update does not expand the initial assembly-discovery implementation.

## 2026-09-19 — RuntimeContext owns assembly discovery

**Author direction.** RuntimeContext will expose the executing assembly as
AssemblyInfo, whose modules and types can be queried. The author then clarified:
“In essence, RuntimeContext will hold a lot of the static methods that Assembly
hold in .NET. Making RuntimeContext the true context.”

**Assistant response.** Accepted context-owned discovery, with AssemblyInfo and
ModuleInfo describing their entities. The assistant proposed executing-assembly
traversal as the first preview slice and left assembly loading outside that initial
scope; the author has not selected exact member spellings or a complete list of
transferred .NET operations.

**Action and status.** Updated the [maintained design](introspection-design.md)
to replace the tentative all-assemblies entry point with the selected minimum and
remove the obsolete value.Type acquisition wording. Recorded the .NET comparison,
frame-selection validation need, benefits and compatibility costs. Production
interface/provider migration and the refreshed VS Code API remain open; the
System.Runtime project identity slice alone does not deliver this API.

## 2026-09-19 — Documentation first, then the remaining Raven library port

**Author direction.** “Let's port the rest of the System.Runtime library from neoIL
to Raven code.” The author reserves API alignment with the current proposals for
the next step, supplies the original proposals, and requests documentation
organization first, committed as its own slice.

**Assistant proposal and action.** Added categorized documentation indexes and an
original-proposal index linking the supplied texts to maintained design notes.
Preserved original proposal bodies, existing document paths and published notes.
The current project remains `System.rvnproj`; the System.Runtime assembly identity
is still planned. Source migration will preserve the existing API in this step.

**Author follow-up.** After porting, review Raven changes so bug fixes and general
improvements are on Raven main while the neoCLR target remains separate.

**Assistant response and current action.** Accepted that review as part of this
work. The collection-contract port exposed a generic-interface base-scope error;
three reduced cases reproduce it on an isolated branch from Raven main using
ordinary .NET references. Integrated the independently validated fix on local Raven
main (`2e3856a6b`, 87 focused checks) and cherry-picked it to the experimental branch.
The branch audit also reproduced a delegate bridge InvalidProgramException with
ordinary .NET metadata and integrated its fix on main (`5a37cd56c`, 19 focused checks).
The neoCLR branch remains separate. Deferred general candidates and policy boundaries
are recorded in the [branch audit](raven-library-port-validation.md#raven-branch-audit).

**Author acceptance criterion.** “The criteria for the port to be done is that
neoCLR is in a functioning state. Programs targeting neoCLR run.” The assistant
accepted compile/import/verify/execute checks against the rebuilt library as the
acceptance gate, rather than source conversion alone.

**Assistant-reported outcome.** The documentation organization is committed separately
as `f957b84`. The library grows from 28 to 47 Raven source slices, covering fundamental
and collection contracts, clocks/local time, native integers, Int32, Console,
Environment and file reads. Saved-program validation found and fixed lost parameter
names in imported library metadata; existing introspection output now passes.
All 64 saved-project cases pass against the rebuilt library, along with controlled
process/file checks and clean snapshot regeneration. This demonstrates functioning
programs, not completion of every source migration. String, unions/error carriers,
remaining descriptors and runtime adapters still contain handwritten neoIL.

**Follow-up direction and response.** The author asked for the next step. The
assistant proposed String/Error, followed by unions/error carriers, descriptors and
array/delegate adapters. The author said “Continue” and subsequently directed:
“After this work. Continue with the remaining slices and don't stop until finished.
Commit separately”. The assistant accepted separate validated commits and continued
without starting proposal API alignment.

**Descriptor follow-up.** The assistant preserved runtime snapshot field order and
names while moving the inherited MemberInfo family, parameter copying and accessor
selection into Raven. The target importer admits a bootstrap-only vector read view;
consumer metadata is unchanged. Nine admission cases, 23 reflection tests and the
freshly compiled reflection program pass. This is the 66th source slice; array,
delegate and remaining intrinsic helpers are still open. The separate ordinary .NET
constructor-arity correction is integrated on Raven main (`d7292b935`), rather than
merging the neoCLR branch.

**Trying the finished port.** The author asked to try the port in VS Code once it is
finished. The assistant committed to completing migration and execution checks first,
then preparing a workspace with the updated Raven compiler and neoCLR library.
VS Code preparation is pending; no successful editor run is claimed yet.

**BindingFlags correction.** The author said “BindingFlags should become a normal
Enum”. The assistant discarded an uncommitted struct-shaped authoring approach and
changed the source to a normal `[System.Flags] public enum`, retaining the six
existing Int32 values. Intrinsic enum operations are compiler/runtime lowering, not
source-authored struct methods. Import exposed a missing CLI backing-field flag;
an ordinary .NET regression reproduced it independently. Raven main now contains
`266b457f5` (13 focused checks), cherry-picked to the neoCLR branch as `6219343b9`.
The source admission checks pass; the slice's execution results are recorded in
[port validation](raven-library-port-validation.md).

**Branch cleanup direction.** After the port, the author asked to clean up side
branches created in Raven while retaining its long-lived neoCLR branch alongside
ordinary branches. Asked about its name, the author explicitly chose “Rename it to
neoclr”. The assistant will remove temporary integrated fix branches and rename the
integration branch after the final review; cleanup is pending at this point.

**Final helper decision.** The source-body audit found TypeOf<T>.Of as the last
handwritten managed helper after the array port. The assistant proposed migrating
it; the author instead directed removal because typeof is now supported. The
assistant removed its runtime/reference declaration and importer mapping, updated
active samples/tests to typeof or type tokens, and preserved published preview
notes as historical records. Existing compiled helper calls require rebuilding.
The ownership check now finds only generated declarations/bodies and explicit
runtime services in the selected Raven profile; final execution validation remains
open until the final gate below is recorded.

**Object.GetType follow-up.** The author suggested that Object may need GetType,
explicitly placing it in API alignment and the smallest feature set for the preview
release. The assistant distinguished declared-type typeof(T) from runtime-object
type discovery and recorded GetType as a candidate for that next stage. No Object
GetType implementation or final API decision is claimed by this port; removing the
old declared-type TypeOf<T>.Of helper remains the current action.

**Compatibility principle.** The author clarified that .NET compatibility should
provide the minimum familiar ergonomics and developer experience, while leaving
neoCLR free to choose its own design. The project is still in development and may
break compatibility. The assistant accepted .NET as the comparison baseline,
committed to documenting benefits/costs and migration consequences, and carried
that principle into preview API alignment, including the Object.GetType candidate.
This does not claim that the proposed preview API has already been implemented.

**Final compiler review and branch cleanup.** The assistant independently reproduced
editor host-reference injection with ordinary .NET references and integrated its
fix on Raven main (`6c2ccb13e`, 65 workspace checks). A known-value deconstruction
change was initially described as an optimization; independent testing instead
showed a real ref-struct InvalidProgramException. Main `f7f3f0c6d` fixes it, with 31
checks covering value-copy mutation, references and null/type checks. Both fixes
were applied to the neoCLR branch without merging target policies into main.
The integration branch is now `neoclr`; the eight temporary integrated fix branches
were deleted. Existing ordinary branches were preserved.

**Editor preparation outcome.** The assistant created an isolated local development
snapshot and VS Code profile, tested the exact saved-project command, and opened
Main.rvn. The initial ArrayList/Option/Result program executed with expected output.
Headless editor checks passed across the current target APIs; actual VS Code logs
confirm the selected server started and opened the demo. This is a local snapshot,
not a release. Instructions and reproducible tooling are in the
[local build guide](raven-port-local-build.md). The broad final runtime run remains
open until its result is recorded below.

**Next implementation direction.** After the local port workspace was prepared,
the author directed establishing the System.Runtime project (currently System),
then completing the basic System.Runtime.RuntimeContext with model implementations.
The author explicitly retires System.Type in favor of System.Introspection.TypeInfo
and selects Object.GetTypeInfo() as the canonical instance method. This replaces
the earlier tentative Object.GetType name; the earlier discussion remains recorded.
The assistant accepted that sequence, began checking the proposal and executable
context prototype, and kept the outstanding port validation running. No completed
production context migration is claimed at this point.

**Project boundary slice.** The assistant renamed the shared source project to
System.Runtime.rvnproj and changed its implementation assembly identity to
System.Runtime. All 73 slices compile/import with unchanged executable bodies;
snapshot and ownership checks pass. The private compiler reference remains an
explicit mapped contract in this slice. The author observed that the open VS Code
sample still showed class-based Info types and no RuntimeContext; the assistant
confirmed that snapshot was the completed source-port baseline and that production
API migration had not yet landed, then committed to updating it after migration.
The broad port run subsequently exposed missing declaring-type metadata for
flattened generated union cases; that correction remains open at this point.

**String/Error slice.** Raven sources now own both method surfaces, with explicit
importer checks for intrinsic storage, mixed String receivers and opaque Error
receivers. Native ownership and the existing parameter metadata remain unchanged.
Eleven admission checks pass, including rejected storage mutation and fabricated
opaque construction. All 64 saved-program cases also pass against the committed String/Error library
(`8744717`) and a fresh consumer core. The empty-declaration slice then exposed
source shadowing of an explicitly configured unit assembly. The assistant reduced
it independently with .NET ValueTuple, integrated the general fix on Raven main
(`c17cb8397`, 19 passing focused checks), and cherry-picked it to the neoCLR feature
branch (`b6f12353f`, 24 passing feature checks). Five empty errors and Void bring
the port to 55 source slices; nine admission cases, 14 Rust checks and four saved
programs pass. Unions/error carriers, descriptors and adapters remain pending.

**Open.** Complete and validate the remaining Raven source port, then align API
contracts in the subsequent step. Documentation organization does not itself
implement proposal APIs or rename assemblies. See the [documentation index](README.md)
and [original proposals](proposals/README.md).

**Typed error carrier slice.** The assistant reports seven more Raven source owners
(62 total), with checked erased storage and constructor projection. Fourteen admission
checks, 25 focused Rust tests and all 64 saved-program cases pass. Generic unions,
descriptors and adapters remain open; continuation and separate commits remain the
author's instruction.

**Generic-union work in progress.** The propagation declaration is committed separately
(`49e7307`), with five admission checks and 20 focused runtime checks. Authoring the
remaining carrier bodies exposed a general Raven out-forwarding diagnostic defect.
An independent ordinary .NET regression failed before the fix; the assistant reports
41 checks passing and integrates `5f6e17347` on Raven main, with feature cherry-pick
`2d2a1d586`. Carrier source migration remains in progress at this entry.

**Generic-union outcome.** Raven source now owns Option/Result carriers and cases
(65 slices total). The assistant reports 12 admission cases, 41 focused Rust tests
and all 64 saved-program cases passing. Earlier 14 typed-carrier and 17 foundation
admission checks still pass. Readonly adapters and literal Boolean return emission
were corrected during verification. The separate unqualified generic constructor
lookup investigation reproduces incorrect ordinary .NET execution (0 instead of 42);
a namespace-arity candidate fix now passes its initial focused checks, with broader
validation and integration still pending at this entry.

## 2026-09-17 — Plan the minimal System.Runtime assembly

**Author direction.** Plan a System.Runtime project/assembly supplying the minimal
types a managed application needs, substantially those being implemented in System.
The author keeps the next release focused on API shape and a running demonstration.

**Assistant proposal and action.** Recorded a staged assembly plan: inventory actual
dependencies, establish one foundation identity, migrate reference generation and
runtime contracts together, then execute the POC against it. Namespaces retain their
separate responsibilities; Reflection and Emit remain optional capabilities.
Updated library documentation and website with explicitly planned status.

**Open.** Exact minimal membership and reference/implementation identity mapping
require the inventory. No project or assembly has been renamed in this documentation
slice. See [the plan and .NET comparison](system-runtime-assembly.md).

## 2026-09-17 — RuntimeContext owns typeof acquisition

**Author direction.** Runtime*Info implementations must remain hidden; handle-based
type acquisition and RuntimeContext should return the same or value-equivalent
TypeInfo. The author selected `RuntimeContext.Current.GetTypeInfoFromHandle`,
contrasting it with C#/Raven's System.Type factory, and explicitly requested the
neoCLR Raven compiler change. The release goal is a running API-boundary POC,
not complete implementations.

**Assistant action.** Added an opt-in Raven compiler contract on the existing neoCLR
feature branch and a minimal runtime-backed context in the isolated Raven probe.
The sample uses actual typeof(Date), returning the descriptive interface and
querying fields without reflection invocation. The check executes the source entry
unchanged; separate test instrumentation compares underlying runtime identities.
Updated the proposal, integration documentation and website excerpt.

**Boundary and remaining work.** The default .NET compiler path remains System.Type.
The POC uses a lightweight context facade and hidden adapters over existing runtime
types, not a completed loader or replacement core. Production identity migration,
context lifetime and full cross-origin equivalence remain open.
See [the executable probe](experiments/raven-target/introspection-v1/README.md).

### Runtime-owned descriptor acquisition — 2026-09-17

**Author clarification:** Runtime*Info implementations should not be visible, but
the runtime must know which implementation to produce from a handle or type-of
instruction result. RuntimeTypeInfo is returned as TypeInfo. RuntimeContext should
obtain the same object or a value-equivalent description.

**Assistant response:** Recorded hidden public implementation types, runtime-owned
resolution and shared identity/equivalence across type-of and RuntimeContext as
requirements. Identical allocation is not mandated. The current prototype's direct
internal adapter construction is bootstrap-only; it does not implement the final
resolver, context discovery or equivalence contract. Exact lowering and lifetime
remain open, with validation requirements in the [proposal](introspection-design.md).

### Refined interface-based introspection proposal — 2026-09-17

**Author direction:** Supplied a simplified proposal and asked to update the docs
before continuing work. `*Info` types are interfaces without I-prefixes. The public
Type/TypeInfo, Assembly/AssemblyInfo and Module/ModuleInfo pairs disappear. V1 is
runtime-backed through RuntimeContext; full MetadataContext and typed introspection
are deferred. Reflection binds/executes against a context; Emit generates without
implicitly loading. Prefer collection queries over BindingFlags.

**Assistant action:** Recorded the [new proposal](introspection-design.md), preserved
earlier positions as superseded history, and updated the migration docs and website.
Set aside the uncommitted MemberInfo abstract-class source-port experiment because
it pursued the superseded public shape. Existing committed runtime APIs remain
working migration baselines. Proposed a minimal executable interface/provider probe
before the coordinated runtime identity migration; it is not the full v1 API.

**Author follow-up:** "Keep the usage of binding flags for now".
**Assistant response:** Retain BindingFlags and existing filtering semantics in the
current interface migration and samples. Collection-style querying is a future
option, not a requirement to replace flags in this slice.

**Implementation outcome:** Added an isolated Raven TypeInfo/MemberInfo interface
probe backed by current runtime types and field snapshots. Verified interface
emission, DeclaringType returning TypeInfo, execution and rejection of Invoke/Info
on the minimal interface. Existing three introspection consumers and eleven focused
runtime tests pass. No production descriptor identity or RuntimeContext implementation
is claimed. The website describes the direction and prototype status without the
BindingFlags detail, as requested by the author.

### Shared introspection model clarification — 2026-09-16

**Author correction:** Limit the API to necessary concepts. Introspection need not
be independent of the runtime. System.Runtime.Reflection and System.Runtime.Emit
are additional capabilities that may attach operations and supply constructs within
the same descriptive model, with clear boundaries.

**Assistant response:** Acknowledged having overextended the separation into runtime
independence. The next port retains existing runtime services and adds no offline
provider or alternative descriptor hierarchy. The shared model remains descriptive;
invocation and builders are additional capabilities.

**Implementation follow-up:** Ported the existing TypeInfo queries and internal
handle factory to Raven. Kept runtime metadata services and the instance Info
property; adapted query results to managed arrays. Library import now checks
internal factory contracts and bootstrap generation preserves their visibility.
Flag parameters use the descriptive name `flags`. No runtime Reflection/Emit APIs
were added. The member hierarchy remains the next source-port work.
The author also requested sample updates. Updated the executable Raven examples
to show the shared TypeInfo shape and named `flags` arguments, with readable blocks.

**Further author requirement:** Future Emit must avoid the problems of combining
types from runtime Reflection, MetadataLoadContext and Emit implementations.
**Assistant response:** Recorded cross-context identity, provenance and binding as
requirements of the shared model, with mixed-origin signatures and unfinished-builder
tests. Common API shapes alone do not establish interoperability. Exact binding APIs
remain future design; the current TypeInfo port does not implement Emit.

### ParameterInfo source migration — 2026-09-16

**Author direction:** Continue the API restructuring and Raven migration in
separately committed slices.

**Assistant action:** Selected ParameterInfo as a bounded first descriptor port.
Its six getters and private constructor are now Raven-authored, with an importer
check protecting runtime snapshot layout. Existing factories and public contracts
are preserved. Added positive/negative authoring checks and reused executable
introspection consumers. TypeInfo query adapters and member inheritance remain
subsequent work. Documentation and website status identify the completed subset.

**Validation:** 21 focused runtime tests, positive/negative ParameterInfo authoring
checks and three executable Raven consumers passed. Bootstrap hashes and the
website build also passed; this is local development validation.

### First namespace migration slice — 2026-09-16

**Author direction:** Start implementing the new structure, keep documentation and
the website current, and commit each feature slice separately.

**Assistant action:** Moved existing descriptors and BindingFlags into
System.Introspection in the runtime, reference metadata, Raven Type source and
consumers. Regenerated bootstrap artifacts and coverage inventories. Preserved
query behavior and documented the breaking namespace migration. Info remains an
instance property and descriptor bodies remain neoIL for this bounded slice;
their Raven port and extension boundary are still open. Website status reflects
this distinction. No Raven compiler changes were required.

**Validation:** 49 focused runtime tests passed, along with Type authoring checks,
signature checks, and three Raven consumers executing against the migrated
namespace. The old descriptor namespace is rejected. Bootstrap hash and coverage
checks and the website build validate generated records and presentation.

### Proposal-driven Raven library migration — 2026-09-16

**Author direction:** Commit the proposal records and prioritize moving existing
APIs into their proposed namespaces and structure, with the goal of porting the
runtime class library from neoIL to Raven.

**Assistant recommendation:** Start with the existing introspection descriptors,
following the completed Type port, then migrate the existing time family. Update
reference metadata, importer mappings, generated artifacts and consumers with each
slice. New globalization, stream and async implementations remain separate work.
Recorded the sequence in the [Raven library plan](raven-system-library.md).
This exchange records direction and a proposed sequence, not completed API moves.

**Subsequent author clarification:** The next release should have the API shapes in
place even without complete implementations, serving as a demo or POC.
**Assistant response and action:** Updated the library-preview objective and Raven
migration plan to prioritize namespaces, signatures and capability relationships
across all seven proposal families, alongside working examples for implemented
paths and explicit declaration-only status for unfinished areas. Full backends are
not required before exposing the proposed structure. No API implementation changed
as part of recording this clarification.

**Website follow-up:** The author asked to show these improvements on the website.
Updated What's next with the demo/POC objective, migration priorities and all seven
proposal families, explaining benefits and distinguishing current implementations
from proposed API shapes. Website publication is separate from this source update.

### File-system capability proposal — 2026-09-16

**Author proposal:** Treat the file system as an explicit capability rather than a
collection of globally ambient static gateways. Keep `Path` as a pure value, use a
`FileSystem` context with a sensible default and injectable implementations, and
obtain `File`, `Directory` and `FileSystemEntry` handles from it. Keep streams as the
byte-I/O abstraction and avoid making the filesystem interface responsible for text,
encoding or buffering.

**Assistant action and outcome:** Added the [file-system capability proposal](filesystem-design.md)
with the current static File API as baseline, alternatives including an injectable
facade and runtime intrinsics, a System.IO.Abstractions comparison, capability/error/
ownership boundaries and validation requirements. No implementation or existing API
changes were made; handle lifetime, path semantics, enumeration, sandboxing,
cancellation and async contracts remain provisional.

### Globalization, introspection and streams proposals — 2026-09-16

**Author proposals:** Define globalization as immutable culture data resolved from
`CultureId` and supplied through a contextual provider; separate core type identity,
descriptive introspection and optional runtime Reflection/Emit capabilities; and model
streams as composable synchronous/asynchronous byte capabilities rather than one large
`.NET Stream` class. The Stream proposal keeps partial I/O, explicit buffering,
text-over-bytes readers, and FileSystem as the source of stream capabilities.

**Assistant action and outcome:** Added [Globalization API](globalization-design.md),
[Stream API](stream-design.md), and the new Introspection/Reflection section in the
[reflection review](reflection-model-review.md). Compared the proposals with current
.NET conventions and relevant independent abstractions, and recorded tradeoffs,
validation requirements and open contracts. No runtime or public API implementation
changed; the existing static file, bounded text, reflection and console surfaces remain
the implementation baseline.

### Ongoing Type/Reflection separation — 2026-09-15

**Subsequent corrections and scope.** The author placed TypeInfo in System.Reflection,
kept MemberInfo as the member hierarchy's base, and noted that TypeInfo can also be
a member for nested types. Closed hierarchies remain a desired direction, not an
implemented property of this port. The author then explicitly prioritized porting
Type alone while retaining reflection implementations in NeoIL. Type must retain
only basic identity/shape information. The author clarified that TypeInfo can use
the same descriptor while exposing the full metadata model. Finally, descriptive
parameter names across the Raven-authored APIs were requested as the next slice.

**Current action.** Implemented the Raven Type source and checked import path,
with TypeInfo retaining an opaque handle and performing queries on demand. Migrated
Raven reference metadata and examples to `.Info`. Earlier statements that a failed
export check prevented the port were premature: the causes included parameter names,
private constructor/static factory admission and self types inside arrays. Earlier
documentation conflating member descriptors with TypeInfo subclasses is superseded
by these corrections. See [the migration](raven-reflection-api.md).

**Author's direction:** Keep `Type` focused on the necessary identity and shape
information. Reflection and runtime member lookup should be separate; a future
`Type.Info` may expose that lookup explicitly, acknowledging that it can be costly.
TypeInfo and its member descriptors should form a closed algebraic hierarchy.

**Assistant's response:** Accepted this as the next API boundary. Existing member
queries remain available during the transition; the reflection hierarchy and native
descriptor construction must be formalized before moving the implementation from
NeoIL to Raven. The proposal and its compatibility status are recorded in
[reflection-hierarchy-plan](reflection-hierarchy-plan.md) and
[type inspection](type-inspection.md).

## Founding conversation — from the initial brief through Preview 1

### O1. The author sets the experiment's boundaries

**Author's brief:** Build an independent, cross-platform, .NET-derived runtime, starting
with a small interpreter proof of concept. Metadata and instructions should remain
familiar, with deliberate breaking changes where useful. Types should not inherently
choose value or reference semantics; values begin with stack allocation unless requested
otherwise. Void should be usable in generics, functions need no class container, and
Option/Result should model optionality and recoverable errors. Terminal errors are
Faults. The library should remain recognizable without the interface I-prefix.

**Follow-up direction:** Include an assembler, HelloWorld and feature demonstrations,
and document the work. Assembly spelling may differ while metadata and IL stay familiar.

**Assistant's reported action:** Built the interpreter and assembler with 35 instruction
forms, samples, README and design notes; reported 19 passing tests. On the author's
request, committed the initial proof of concept as `112fb7d`. Binary emission was
explicitly unfinished; the artifact was temporary JSON.

### O2. Familiarity becomes an API and metadata requirement

**Author's direction:** Calls need signatures for overloading. Implement the runtime
library in the platform's own IL, with explicit native implementation mappings akin to
CLR InternalCall. Keep familiar consumer APIs even where implementation differs.

**Assistant's reported action:** Added signature-based calls and overloads, a platform
System library and validated native bindings; committed `1899cd2` after the author
requested a commit. The author then prioritized the type system before native interop.

**Assistant's proposal:** Separate type identity and behavior from storage and access;
use canonical primitives and type-owned methods, then address mutation and interfaces.
The proposed `Ref<T>` and future ownership rules were suggestions, not settled policy.

### O3. Memory policy is explored, then deliberately deferred

**Author's questions:** How should stack and heap construction be encoded? Could an
allocation environment select an allocator, with optional GC or counted ownership?
The author rejected assuming Rust's memory model and explored an explicit counted
`Ref<T>` wrapper, while leaving language conveniences to compilers.

**Assistant's response:** Proposed separate allocation, construction and lifetime
services, and pointed out that copying a record containing a counted owner would need
corresponding retain/release behavior. Moving collectors would need references or
handles that can follow relocation, rather than unrestricted native addresses.

**Author's decision:** Defer memory management; implement heap allocation and pointers
first, but preserve the ideas.

**Outcome reported:** Canonical types, type-owned methods and the memory direction were
committed as `44ee213`. Counted ownership and GC remained deferred. This is an early
position, subsequently revised toward managed GC in the continuation; it must not be
retold as though that later decision had already been made.

**Additional author examples retained in this task:** An inline `int[10]` might
imply stack storage; a heap allocation might produce a pointer or later be held by
Ref<int[]>. System.Array<T> could provide a familiar view over separately chosen
storage and ownership. These examples did not settle escaping lifetimes or GC.

### O4. The author revisits assembly syntax and insists on native pointers

**Author's request:** Preserve optional parameter/local names even though execution uses
indices. The author initially left declaration syntax to the assistant, then explored
name-first versus type-first notation.

**Assistant's response and action:** Chose consistent `name: Type` syntax and argued that
low-level execution did not require types before names. Later, the author explicitly
requested `Type optionalName`; the assistant reported migrating declarations, samples
and documentation in `0ba69e3`. When the author clarified that identifiers are only
contextual mappings, the assistant documented numeric normalization and added field
aliases in `848dcf6`.

**Pointer exchange:** The assistant initially recommended tracked allocation-plus-offset
pointers for diagnostics. The author specified native pointers similar to .NET.
The assistant reported native addresses, explicit allocation/free, field access and
loads/stores; the pointer and named-slot work was committed as `519df99`, followed by
native integers in `2d6bfbe`. A later C-ABI P/Invoke sample modified guest memory through
native code (`588b60d`). External boundaries and lifetime guarantees remained limited.

### O5. Fundamentals precede larger frameworks

**Author's direction:** Get fundamentals working and commit each slice. The author also
asked whether UTF-8 should be standard.

**Assistant's response:** Recommended valid UTF-8 with scalar iteration and explicit
UTF-16 compatibility, leaving length/indexing contracts to a separate decision.
Reported successive integer, bitwise, floating-point, checked conversion, memory-copy,
frame-allocation and branch slices. These continuations authorized progress; they are
not evidence that the author independently endorsed every low-level implementation detail.

**Author's next priority:** Establish the type capabilities needed for Option/Result
before reflection. **Assistant's action:** Proposed unions for tagged payloads and enums
for integer constants, then implemented generic metadata (`66cd94a`).

### O6. The author stops a union implementation to question its layer

**Author's intervention:** A union is a carrier accepting variant types. After referencing
Raven, the author asked whether unions should have IL support at all or just be a type
convention, and asked the assistant to stop.

**Assistant's response:** Recommended ordinary carrier/case types, recognizable metadata
and methods, with storage treated separately. It acknowledged that the union implementation
was still uncommitted. Following the author's suggestion of a .NET 11-like convention,
it proposed constructors for alternatives and typed extraction without forcing boxing.

**Action reported:** Removed the uncommitted union-specific machinery and documented the
convention in `378295f`. Existing bootstrap Option/Result behavior remained, so this did
not yet remove every special union operation. Generic record construction, methods and
native storage followed (`7ca411d`, `d813e20`, `c57910e`).

### O7. Pause feature accumulation and examine architectural prerequisites

**Author's direction:** Stop after the current slice, reconsider implementation order,
and identify opportunities relative to CLR. Keep a future high-level language and
library implementation in view.

**Assistant's response:** Paused after marker attributes (`d87c8a6`) and proposed a shared
metadata/IL contract for assembler, compiler and execution backends. It identified copied
receivers, construction, uninitialized storage, resource-owning copies and interface
views as unresolved contracts. It distinguished value semantics from physical placement.

**Outcome reported:** Documented construction and addressed-access proposals (`e43ae0a`,
`21d0897`), explicitly without implementing them yet. After further continuation, added
control-flow and typed verification (`e6988c1`, `2ef3f52`). Verification remained opt-in,
with value-dependent validity checks left to execution.

### O8. Embedding and native compilation influence the whole architecture

**Author's direction:** Provide hosting like CLR, Mono or a script engine, and consider
interpretation, JIT and NativeAOT as architectural targets.

**Assistant's proposal:** Separate runtime services, execution contexts and invocations;
share semantics across backends and make fallback explicit. **Author's correction:**
The execution-mode requirement concerned the entire platform, not specifically hosting.

**Action reported:** Corrected the scope and documented execution architecture in
`6f9aeae`. Subsequent implementation included function/type identities, reusable loaded
programs, module sets, dependency references and revisions, typed invocation and
cancellation. Bounded call-graph analysis (`131a3b5`) prepared for compiled execution;
it did not implement native code generation.

### O9. Useful small programs guide scope

**Author's direction:** Prioritize simple but functional programs, Fault stack traces,
primitive wrappers, strings and arrays. A modified C# or Raven subset might later
produce the same IL; first explain how programs can be built and run.

**Assistant's actions reported:** Documented the progression in `0094515`, implemented
String methods (`3d2d2fb`) and added requested build/assemble/verify/run instructions
(`735576c`). Neither frontend had yet been selected.

**Nullability exploration:** The author suggested explicit declaration-level nullability,
then considered strict compiler/tooling enforcement without Nullable<T> wrapping.
The assistant noted that inline values still require an absence representation and
recorded the candidates (`9ec6206`, `c9c001f`), with no runtime change. Later selection of
type-encoded nullability belongs to the continuation, not this earlier exchange.

**Scope correction:** When networking was mentioned as a future demonstration, the author
reasserted console I/O and fundamentals, cautioning against premature Stream-style APIs.
The assistant recorded a workflow of starting with a runnable program, finding the gap
in existing IL, and distinguishing library logic from host operations (`1286b34`).
Console I/O followed in `220c2c8`.

### O10. Ordinary unions drive ordinary object capabilities

**Author's direction:** Implement constructors, properties/accessors and accessibility
so unions can be expressed as ordinary types; remove special union instructions.
Use familiar .NET access rules for now, leaving redesign for later.

**Assistant's reported actions:** Added property metadata (`e1f2987`), method and field
access enforcement (`a32a8dc`, `c62d5e7`), then constructors (`7a029f4`). Ordinary
Option/Result carriers in platform IL followed in `5606e20`, with boundary migration
still required.

**Author's publication decision:** Define the MVP as a runnable source preview, include
basic reflection, use Raven-like pseudocode to explain IL, and call it Preview 1.
**Outcome:** The assistant recorded six illustrative program mappings (`3b441ee`);
a working language compiler was not a Preview 1 requirement.

### O11. Nested cases, storage concerns and incomplete attempts remain visible

**Author's clarification:** Cases are ordinary nested types, not an inheritance scheme.
For generic unions, Raven uses a non-generic companion owning independently generic
cases; non-generic unions can nest their cases directly.

**Assistant's actions reported:** Recorded that model (`7fe18e2`), implemented nested
ownership (`270fc1f`) and refined the direct/companion rule (`f1a35eb`). The migration
included reverted attempts and changing diagnoses of arity/scope problems. Those reports
are not one verified root cause. The assistant eventually reported companion declarations
in `82a37e3`, still with old wrappers during transition.

**Author's concern:** System.Value might become hidden boxing; explicit low-level
storage was preferable to prematurely fixing that abstraction. **Assistant's response:**
Distinguished typed erasure from pointer-backed storage and documented its limits.
Later, the author directed its eventual retirement once ordinary storage/references
could replace it; that migration requirement was recorded in `322b1bb`.

### O12. Publication requires explaining limits, not only listing features

**Author's questions:** What remains before a preview, and what will reviewers need
answered? The author selected MIT and authorized the proposed preparation slices.

**Assistant's response:** Prioritized finishing union migration, specifying storage
costs, minimal reflection, reproducible demonstrations and platform validation. It
warned against confusing familiar metadata with binary compatibility or interpreter
checks with a sandbox. MIT licensing was reported in `1b7b950`.

**Author's further exploration:** Could Void* carry arbitrary union payloads?
**Assistant's response:** Yes, with type/lifetime responsibility and a clear copying
contract; an owning carrier copy could otherwise leave dangling aliases.
**Action:** A pointer-carrier sample demonstrated stack/heap borrowing, aliasing and
explicit release (`511a312`), without new instructions. Later copy and borrowed-pointer
TryGet demonstrations (`6972853`) retained those lifetime obligations.

### O13. Collections expose the need for checked reference receivers

**Author's requests:** Add ArrayList<T> directly under System.Collections; explore a
small interface slice for the preview. Be explicit that an interface is a reference
view from the high-level language's perspective.

**Assistant's actions reported:** Added ArrayList (`a9a58e9`) and borrowed interface
dispatch (`76f90e5`). It disclosed that receivers were copied: pointer-backed mutation
worked, but inline fields did not write back.

### O14. Correct TryGet to return a case through an output reference

**Author's direction:** Ordinary library APIs should pass safe references unless a
particular API specifically needs native pointers. Reference parameters and receivers
were requested for caller-provided output storage and interface access. The author
asked for documentation showing `&` in Raven-like pseudocode.

**Specific API correction:**

> The general convention for TryGet in unions is Option<T>.TryGet(out Some<T>& value) etc

**Assistant's response:** Adopted overloads selected by the case-reference type, with
familiar `out` in illustrative source and an explicit assignment-on-success contract
in neoIL metadata. At that stage the library used `out(true)` and callers branched on
the Boolean before reading a newly initialized output. Extraction copied a case value;
it did not return a borrowed address into carrier payload storage.

**Author's follow-up:**

> Remove the trygetpointer methods

**Action and observed outcome:** Removed TryGetOkPointer/TryGetErrorPointer from the
pointer-carrier sample and its callers, tests and active documentation. The aliasing
demonstration instead read the carrier after mutation. The combined output-reference
and overloaded-union slice was committed as
[a2de4c4](https://github.com/marinasundstrom/neoCLR/commit/a2de4c4).
The assistant reported the relevant tests, sample verification and Clippy passing.
Historical work-log descriptions of the superseded API were retained as history.

### O15. Follow the API decision into examples and contract checks

**Author's question:** What should happen next?

**Assistant's proposed order:** Integrate case-based TryGet into console/file programs,
review reference contracts, refresh the preview plan, and complete release validation.
It identified System.Value retention as a remaining preview boundary rather than
claiming that Void* alone solved payload storage. The author's subsequent continuations
requested more work; they are not recorded as explicit approval of every detail.

**Actions:** Migrated four console/file samples to TryGet and Value properties
([cdaf3f5](https://github.com/marinasundstrom/neoCLR/commit/cdaf3f5)); then added reference
regressions for conditional forwarding, aliased output slots, receiver mutation and
interface contract matching ([f167512](https://github.com/marinasundstrom/neoCLR/commit/f167512)).

**Assistant's results:** Reported 28 relevant sample tests passing for the migration,
then 38 reference/interface tests and strict Clippy for the review. No runtime change
was needed in that review. The documentation distinguished caller-side static
initialization proofs from per-invocation assignment checks and their conservative limits.
These counts describe the reported runs then, not the current test-suite size.

### O16. Add a familiar equality contract without changing primitive equality rules

**Author's request:** Add a preferably generic equatable interface, based on the .NET
interface, so its type argument specifies what is being compared.

**Assistant's proposal and action:** Added System.Equatable<T> with Equals(T other),
then implementations for Int32, String and System.Type and a Point example using
explicit interface views. It reported this as ordinary library/interface work requiring
no new opcode, automatic boxing, hashing or implicit collection comparer selection.

**Validation issue retained:** A full regression run exposed a test assuming the first
library method was a native declaration. The new interface changed that ordering.
The assistant corrected the test to locate the intended declaration by name, reran it
and the remaining suites, and reported combined coverage of 521 passing tests plus
Clippy. Committed as [50335a5](https://github.com/marinasundstrom/neoCLR/commit/50335a5).

**Still separate:** Broader primitive coverage, hashing, ordering and collection comparer
policy were not implied by this one-method interface. The author's request was not
recorded as a request for all of those facilities at once.

### O17. Articulate the platform direction, while leaving later lifetimes open

**Author's observation:** The platform's direction was becoming clearer: values by
default, explicit reference passing and optional capabilities, with runtime tracking
that need not use a Rust-style borrow checker.

**Assistant's response:** Distinguished checking a live, correctly typed and initialized
slot from requiring exclusive borrowing. Multiple writable aliases were allowed in
the implemented call-scoped subset; a language could impose stronger rules and a future
backend could remove checks it proved unnecessary. Stored, escaping or cross-thread
references still required explicit contracts.

**Action:** Recorded that distinction and refreshed preview gates in
[2d2d7ac](https://github.com/marinasundstrom/neoCLR/commit/2d2d7ac).
This was not a report that all longer-lived references or concurrency had been solved.

### O18. Turn release readiness into evidence, and correct a Windows-only failure

**Assistant's proposed work after the author's continuations:** Verify the minimum
Rust toolchain and clean-source instructions, then audit the source package and draft
release notes. Those were proposed implementation steps, not new author-specified
features.

**Performed checks:** Installed Rust 1.85.0, built a clean tracked-source snapshot and
reported one uninterrupted 521-test run and 13 documented workflow steps passing.
Declared the minimum in Cargo and added minimum/stable CI coverage
([195c68c](https://github.com/marinasundstrom/neoCLR/commit/195c68c)).
Then inventoried 20 locked dependencies, preserved 42 notice texts and checked a
368-file source archive ([a133046](https://github.com/marinasundstrom/neoCLR/commit/a133046)).
The assistant explicitly bounded provenance and platform claims rather than treating
local success as completed cross-platform release validation.

**Author's report:** The recently pushed changes failed Actions.
**Assistant's investigation:** Both Windows jobs failed a test whose LF-only multiline
replacement did not match a CRLF included sample. The intended mutation was absent,
so output remained 42 rather than 52. Linux and macOS had passed. The fix normalized
the source, required exactly one edit marker and exercised LF and CRLF input, with a
separate Rust 1.85 run using a physically CRLF sample
([4fb1f9e](https://github.com/marinasundstrom/neoCLR/commit/4fb1f9e)).

**Author's confirmation:** “It succeeded”. The assistant initially acknowledged this
as the author's report; it later inspected
[run 34145375938](https://github.com/marinasundstrom/neoCLR/actions/runs/34145375938)
and confirmed all six jobs passed. Reported success and independently checked success
are separate steps in the record.

### O19. Request finalization, then publish only after candidate checks

**Author's later comparison:** Some aspects felt closer to WebAssembly than CLR.
**Assistant's response:** Compared separation of execution mechanisms from language
policy, while locating neoCLR's CLR influence in typed metadata, generic types and
members. It explicitly distinguished typed/native-pointer facilities from adopting
WebAssembly linear memory. This was an architectural comparison, not a decision to
implement a WebAssembly backend or import its memory model.

**Author's request:** First asked for a GitHub tagline. The assistant supplied the
experimental, .NET-inspired description emphasizing values and explicit memory and
references. The author then directed:

> Proceed. We need to finalize the release and add badges to README.

**Assistant's concrete plan:** Selected v0.1.0-preview.1 for a source-only GitHub
prerelease, added CI/version/license/minimum-Rust badges, and stated that publication
would follow successful CI on the exact release commit. The version choice was the
assistant's implementation decision under the release request, not a separately
quoted author selection.

**Actions and verified outcome:** Committed and pushed
[f11ec01](https://github.com/marinasundstrom/neoCLR/commit/f11ec01), verified its source
archive membership and notice hashes, and built an extraction on Rust 1.85.0.
[Run 34146500779](https://github.com/marinasundstrom/neoCLR/actions/runs/34146500779)
passed all six jobs before the assistant created the
[Preview 1 prerelease](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.1).
Tool results confirmed the release was published rather than draft, marked prerelease,
and targeted that commit. Uploaded asset hashes and the fetched tag were checked
against the validated local archive and commit.

**Scope at publication:** Source and checksums were published; no prebuilt runtime,
crates.io package, high-level compiler or JIT/AOT backend was claimed. The conversation
then continued into new development. Later GC, Neo and reference work below must not
be read as features retrospectively shipped in Preview 1.

## Post-Preview-1 continuation — author-side retrospective

Recorded 2026-09-09. **Assistant response unavailable for these earlier exchanges.**
Each entry summarizes the author's questions or directions; references to a change
of position are based on subsequent author messages. The entries do not imply that
the assistant originated, endorsed or implemented a particular answer.


### 1. Start with a runtime experiment, and examine its foundations

The opening request in this continuation was to inspect the prototype against its design goals, identify
what should be fixed before proceeding, and expose the choices that future features
would create. Cleanup and separate commits were part of that process.

The author then articulated the central direction:

> We are working towards a runtime platform where value semantics are default for type, passing by reference when explicitly wanting to, lifetimes are deterministic. Pointers are a low-level construct available for native interop.

Destructors, scope exit, the last active reference, Disposable/Closable and Clonable
were explored as related but distinct responsibilities. These were early questions,
not a final declaration that all managed heap cleanup would be deterministic.

### 2. Explicit references should still be managed

The author repeatedly distinguished choosing reference semantics from manually
managing a reference's validity. A reference should behave transparently once obtained;
its allocation location should not dictate how the caller accesses its members.

The conversation explored returning a reference to a local counter. Later direction
made the escape rule explicit: a function must not return an address into its own
frame. Returning a reference into an argument owned by an outer frame is different,
as illustrated by returning &counter.Age from a Counter& parameter. The runtime
should validate this and fault on an invalid escape.

This is a meaningful evolution to retain: the attractive source example prompted a
question about allocation and lifetime, rather than settling that question by syntax
alone. Later discussion also left copying a value onto the heap as a possible explicit
future operation.

### 3. Reuse managed references and familiar instructions

Ref<T> was explicitly described as a proposal, not the chosen abstraction. The author
preferred reusing CLR-style managed references to express reference intent.

> We should keep close to .NET CLR instruction set and semantics when we can, unless we deviate to improve. We are still in a preview and you are allowed to make breaking changes

The discussion of newobj, a possible newval instruction and the existing initobj
asked whether the model could use consistent, familiar patterns before introducing
new machinery. Preview compatibility was not a reason to preserve an unsound choice.

See [managed-reference semantics](managed-reference-semantics.md).

### 4. Retain garbage collection and managed productivity

The author subsequently directed the project to implement garbage collection for
managed heap memory. The platform was still intended to offer the productivity of a
managed runtime, alongside explicit storage and reference choices.

This qualified the earlier lifetime discussion: frame-owned values and managed heap
objects need different lifetime mechanisms. Native pointers remained a separate
interop capability. Pinning, calls into native code and native consumption of managed
data were raised as future considerations, while the immediate scope stayed focused
on a sound memory-management starting point. GC monitoring should grow when needed.

### 5. Treat reference use as an addressing choice, not a different kind of object

An Object hierarchy was considered useful for library familiarity, but the author
did not want every runtime type forced to inherit from Object. Equality and hashing
were discussed in terms of value content, with reference identity a separate question.

> There is no inherent value-typeness or reference-typeness in our system. It's just an addressing mode.

The sequencing mattered too: reach a memory-management milestone, then explore the
object model and adapt memory management as inheritance required it.

### 6. Build Neo to test and explain the platform

A small high-level language with a Raven-like syntax was proposed as an end-to-end
exercise and named Neo. Documentation, runnable examples and a grammar were requested.
Control flow and later patterns/unions would make the examples more expressive.

The compiler should stay updated as neoCLR evolved, but it was not intended to become
a complex, full-fledged compiler. Rewriting parts of the runtime library in Neo, and
perhaps bootstrapping the compiler, were possibilities for the future rather than
immediate commitments.

The language therefore became both a demonstration and a way to expose gaps in the
runtime's contracts.

### 7. Make reference access transparent, including collections and views

The author objected to samples explicitly dereferencing managed references: accessing
an int& or Foo& should be handled automatically by the compiler. Pointers were a
separate case.

Arrays raised the same storage question: an owned int[3] value and an int[]& referring
to a managed heap array should fit one consistent model. Interfaces then supplied a
practical test of reference views and virtual dispatch. The usual I-prefix naming
convention was not wanted.

Later, ArrayList<Foo&> made the consequence concrete: the list stores references,
Add accepts a reference, and accessing an element remains transparent. Its internal
buffer should be a managed array reference. These requests connected the abstract
model to ordinary library use.

### 8. Make the running system inspectable and its contracts reviewable

A live debugger was requested to show call frames, stack memory and heap memory.
The author chose an interactive terminal interface and immediately connected it to
the need for Neo-to-IL source mapping. Stepping into and over calls, and instructions
for using the debugger, were part of that concern.

Managed references also prompted a review of library API parameters, an API design
document, and reflection/introspection. MethodInfo, FieldInfo and PropertyInfo were
preferred for Type; FunctionInfo was only relevant if module-level functions were
being queried. The hierarchy could wait until the platform supported it properly.

### 9. Make progress understandable outside the conversation

The author requested a changelog reconstructed from previous work and maintained
with every commit, keeping published sections unchanged. Release preparation included
asking what still needed fixing, rather than equating a growing feature list with
readiness.

Small sample details mattered as well: uninitialized array allocation should not
require empty braces, and examples should use indexer syntax rather than accessor
method spellings. Familiarity should be visible in actual code.

### 10. Put characteristics at the layer that can justify them

The roadmap grew to include inheritance, nullability, delegates, generic constraints,
async, dynamic dispatch, enums and a useful base library. The author clarified:

> The familiarity is mostly in APIs and behaviors.

The project should research .NET/CLR alternatives, while remaining free to avoid
complexity imposed by compatibility constraints elsewhere. That freedom was not an
instruction to put everything into the runtime.

Immutable bindings were explicitly kept at the language level for now, because a
universal runtime slot mechanism lacked a concrete justification. Nullability was
examined separately: slot property or type property? The eventual direction was an
explicit type characteristic for both values and references, with null a special
state rather than simply zeroed data. Option remained the preferred expression of
many forms of optionality.

### 11. Let useful library scenarios determine the next building blocks

The next preview should demonstrate a recognizable runtime library and the benefits
of Option and Result. That raised dependencies on inheritance, interface inheritance,
abstract classes and constructor chaining. Base-class use should be a reference view
of the concrete object, not a sliced value copy.

Delegates were retained as a useful platform abstraction on which languages could
build, including future closures. Inference and automatic function-to-delegate
conversion could reduce language friction. Comparable, Iterable, Iterator, character
helpers, math and separate date/time types followed as practical library needs.
Globalization and sophisticated formatting could wait; retrieving the system's local
date and time was already a useful bounded scenario. LINQ was also deferred.

### 12. Question whether the experiment is worth its friction

The author did not treat explicit references as a proven usability improvement:

> Is the friction caused by this really worth it? Right now we don't know what the real experience will be using this language.

The discussion contrasted design-testing samples with code resembling real applications.
Long experience with C# was acknowledged as part of the perspective, while recognizing
that unfamiliarity alone does not establish that a model is bad. The response was to
try practical scenarios and investigate whether problems came from runtime behavior,
library implementations or the language projection.

The cleanup discussion was revisited in that context. A value facade around heap
storage might suggest destructor-based cleanup, but the author later acknowledged:

> You are right. Since they are heap allocated resources, we don't need manual cleanup

This does not settle cleanup for every external resource. It records a correction
about managed heap storage. See [the reference-experience experiments](experiments/reference-experience/README.md).

### 13. Use real workflows to refine unions and type-design guidance

Raven's Result.Ok/Result.Error projection led to questions about generic inference.
The author explained independent case types: Ok<T> need not know E, yet can be accepted
by a Result<T,E> carrier. Each carrier constructor defines an accepted variant type.
Source union declarations, if let and let … else were requested to make this usable
in the order-workflow example.

Guidance for type authors was also requested: small immutable data often suits copying;
shared mutable state often suits references. But size alone does not define the contract:

> You need to look at a type as the sum of its parts and what the contract guarantees and don't. Passing reference into something has an implication.

A type's contained references, copying behavior and promises matter alongside its
surface syntax. See [type-design guidelines](type-design.md) and
[the case-construction discussion](result-construction.md).

### 14. Explore reduced syntax, then retain explicit borrowing

Removing the explicit & when passing a value to a reference parameter was considered
as a possible ergonomic improvement. The author then settled the immediate direction:

> Explicitness with & is the right way to go here

A separate clarification concerned assignment. Foo& assigned to Foo& should copy the
reference and retarget the binding, not copy the contents into the previous target.
The example let foo2 = foo after let foo = new Foo() was intended to preserve the
same target. Member access such as foo2.Bar() follows that reference automatically.

The rejected shorthand and the assignment correction should both remain in the
record: explicit borrowing and transparent use of an existing reference are different
choices, not contradictory goals.

### 15. Separate platform capability from language usability

Generic constraints and calls through generic receivers brought the boxing comparison
back into focus. The author observed that neoCLR's architecture need not use boxing
to the same degree as .NET, but also distinguished that capability from evidence about
how pleasant a high-level language is to use.

> One interesting thing to see is how few changes we need to make to the instruction set to accommodate our model.

The direction was to fix essential Neo behavior for demonstrating both high- and
low-level patterns, then return to unfinished unions. This observation about the
instruction set was an architectural interest, not a benchmark result.

### 16. Add tools and features when a concrete need makes them useful

The author requested readable compiler IL for debugging, noting that binary instruction
encoding could remain a future step. Enums with flags had a specific first use in
BindingFlags. That immediately led to another connection:

> This ties into constant support

The possibility of revisiting the feature later was explicitly retained. These are
examples of identifying an adjacent abstraction without requiring the current slice
to solve its entire future design.

## Recent exchanges — both sides available, recorded 2026-09-09

These entries follow the available conversation order. The recording date does not
supply a missing timestamp for an individual message.

### 17. Finish essential receiver behavior, then return to unions

**Author's direction:** Fix the essentials in Neo because it tests and demonstrates
both high- and low-level patterns, then finish unions. The author also observed how
few instruction-set changes the model appeared to require.

**Assistant's response:** Proposed finishing and committing generic receiver adaptation
before moving to generic source carriers. Described the receiver work as establishing
behavior, with future JIT optimization still unproven. In response to the instruction-set
observation, pointed to changes in the type system, verifier and lifetime rules, while
retaining much of the familiar instruction vocabulary.

**Subsequent direction:** The author said to continue.

**Reported outcome:** The assistant committed receiver adaptation and generic carriers
separately (`1081870`, `32de966`). Inline generic cases remained unfinished. These are
assistant-reported implementation outcomes, not additional statements by the author.

### 18. Request readable IL for debugging

**Author's direction:** After the next slice, consider a disassemble command, or at
least a way to show the IL resulting from Neo code. Binary instruction encoding was
acknowledged as future work.

**Assistant's proposal:** First expose the existing compiler lowering as readable
source-to-IL output. Treat arbitrary-artifact disassembly as a separate writer rather
than require a binary format before providing useful inspection.

**Status at that exchange:** Recorded as upcoming tooling work; no claim that a full
disassembler had been requested and approved in detail.

### 19. Use BindingFlags to motivate enums, and identify the connection to constants

**Author's direction:** Implement enums with flags support, citing a concrete need in
BindingFlags. A later continuation moved work into that slice.

**Assistant's proposal:** Begin with Int32-backed enums, nominal identity, named values,
zero defaults and unnamed integer values. Preserve BindingFlags bit values and filter
behavior. Reuse integer instructions and typed methods; leave other widths and formatting
for later. During implementation the assistant reported keeping existing factories
compatible and adding enum reflection queries.

**Author's observation:**

> This ties into constant support

**Assistant's response:** Distinguished named enum literals from writable storage and
proposed using literal metadata as a starting point for general constants. Placed
constant-expression evaluation in Neo and encoded type/range validation in the runtime;
kept ordinary immutable bindings separate.

**Author's qualification:**

> We might revisit this feature later

**Assistant's response:** Agreed to keep enum/literal support provisional and finish
validation of the bounded slice, leaving general constants for a later revisit.

**Reported outcome:** Enum/BindingFlags work was committed as `78f418b`. The record
does not treat the author's qualification as approval of every implementation detail.

### 20. Implement the first IL inspection command

**Author's direction:** Continue.

**Assistant's action and explanation:** Implemented emit-il for one Neo source file,
with stdout or new-file output, original source mappings, verification without guest
execution, and no-overwrite behavior. Documented that it is source emission rather
than JSON artifact disassembly.

**Assistant's validation report:** Existing CLI regressions passed. A new test initially
failed because parallel tests selected the same temporary directory; the assistant
changed the test helper and reran it. Reported all ten selected CLI tests and clippy
passing, then committed `6888ffd`.

**Still open:** Artifact disassembly and binary encoding. Neither was represented as
completed by this command.

### 21. Decide what this record should preserve

**Author's request:** Keep a record of the thoughts and direction given during the
project, so others can understand how the author works creatively as a developer
using AI.

**Assistant's first proposal:** Write a development reflection, separating stated views
from editorial synthesis and preserving changes of mind.

**Author's clarification:**

> Yes, keep a timeline of directions and questions.

**Assistant's response:** Proposed an ordered timeline without fabricated historical
dates, plus repository instructions to maintain it.

**Author's further clarification:** The conversation itself is important. The author
compared the purpose to meeting minutes: record what the author said, what the assistant
might have proposed in return, the questions and decisions, so people can see the
collaboration. The author also reiterated: “keep a record”.

**Assistant's correction:** Changed this document from a reflective account toward an
attributed conversation record. Agreed to record direction/question, response/proposal,
subsequent decision and open issues, and to identify unavailable earlier replies instead
of reconstructing them.

**Recording convention now requested:** Preserve the exchanges and the author's role
in directing them. Do not substitute an account of what the assistant thinks the process
says about the author's personality or development philosophy.

**Author's definition of outcome:** “Actual direction” means the outcome: what action
was taken. The author then identified the original shared chat as the project's start.

**Assistant's initial action:** Read that source and added a separate, linked record of its
founding exchanges through Preview 1. Updated this record and repository instructions
to include actions and outcomes, distinguishing reported work from proposals and
unavailable evidence. The original conversation reveals earlier positions on memory
management and nullability that must remain visible alongside later changes.

**Author's purpose clarified:** The core purpose is to give others insight into how the
author works with AI and approaches software development as a whole, using neoCLR as
the concrete case. This purpose guides which exchanges to retain, not just which
technical milestones to list.

**Author's next request:** Merge the origins and timeline documents.

**Assistant's action:** Consolidated them here in conversation order, combining the
founding account with the retained detailed exchanges and removing the separate origin
file. Kept the distinctions between historical reports, observed validation and missing
replies. The README and workflow now lead to this single record.

### 22. Ask where performance-oriented complexity should be visible

Recorded 2026-09-09.

**Author's observation:** Developers in many high-level environments expect an object
variable to contain a reference, automatically followed on member access. C# makes a
stronger distinction through structs and classes, while Java historically separates
primitives from objects. The author sees .NET's struct, reference and readonly features
as ways to unlock performance beyond familiar object behavior, often inside library
implementations rather than directly in ordinary consumer code. Java's work on value
classes approaches the problem differently.

**Assistant's response:** Distinguished copying an object reference from passing the
variable's storage by reference. Framed neoCLR's question as whether choosing value or
reference use independently of the type is worth the added decisions. A language could
supply defaults without removing that runtime capability; no such change was selected.

**Comparison checked:** Microsoft's [performance guidance](https://learn.microsoft.com/en-us/dotnet/csharp/advanced-topics/performance/)
explicitly connects structs, ref access and readonly access with reducing allocation
and copying. Structs are also ordinary modeling tools, and public APIs such as Span<T>
expose these contracts: they are not exclusively hidden implementation optimizations.
OpenJDK's [Valhalla project overview](https://github.com/openjdk/valhalla-docs/blob/main/site/_index.md)
and [background design notes](https://github.com/openjdk/valhalla-docs/blob/main/site/design-notes/state-of-valhalla/01-background.md)
describe value objects without identity, enabling optimized representations. This records
the project's design direction, not a claim that all Valhalla features have shipped.
Removing identity is different from neoCLR's choice to reference a mutable value.

**Assistant's interpretation and open question:** The useful comparison is where each
platform places complexity and how much reaches application code. neoCLR may unify
runtime capabilities yet expose more decisions in Neo. Library and language defaults
need evaluation with real programs; a more uniform runtime alone does not establish
better usability or performance.

**Action:** Recorded these two exchanges and checked the comparison against primary
sources. Runtime semantics, syntax and roadmap priorities remain unchanged.

### 23. Reconsider universal value defaults in favor of ordinary class references

Recorded 2026-09-09 at the author's explicit request.

**Author's exploration:** Languages should not have to live with the platform's universal
value-default decision. Ordinary class use should match the expectation that a variable
holds a reference, with automatic member access. Value behavior should be an opt-in;
explicit reference passing is more appropriate when accessing those values without
copying. Requiring ampersands throughout ordinary application code was identified as
the wrong experience. The author connected this reconsideration to understanding the
.NET class/value distinction and its integration of primitive wrappers into the hierarchy.

**Assistant's response:** Identified this as a revision of the original universal
value-default premise. Distinguished passing an object reference by value from passing
the caller's slot by reference. Suggested ordinary class references, opt-in value
classes and separate output/rebinding contracts, while retaining managed references
for interior access, value borrowing, readonly access and lifetime checks.

**Assistant's proposal, not an author decision:** Express a type's intended usage default
in metadata for consistency across languages. Keep that contract separate from physical
allocation. The author has not yet selected the metadata encoding, syntax, hierarchy
rules or migration strategy; the assistant's preference is not recorded as an approved
implementation design.

**Author's subsequent clarification:** “I don't say we should go down this route just yet.”
The class-reference/value-opt-in model is an alternative to investigate, not a selected
platform direction or authorization to implement a migration. The assistant had framed
the discussion too strongly as a change in direction and corrected this record.

**Outcome and next action:** The author requested this timeline entry and a search for
existing design/architecture evaluation documents. The assistant found the
[reference-experience evaluation](experiments/reference-experience/README.md),
[type/API design discussion](type-design.md),
[runtime groundwork review](runtime-groundwork-review.md),
[execution architecture](execution-architecture.md) and
[design research process](design-research.md). The experiment is the closest existing
assessment of reference friction; the groundwork review is an earlier milestone-specific
architecture assessment, not a current audit. Existing implementation documentation still
describes the value-default model. This entry records the alternative and the author's clarification. The current
value-default model remains in place; no replacement has been selected.

**Author's further reasoning:** Reference behavior is the familiar expectation for
ordinary class use, with heap management largely transparent to application developers.
The author sees explicit value behavior as a type-design intention that should surface
when it matters, and finds Java's modifier-based evolution of existing wrapper classes
potentially less intrusive than C#'s class/struct split. These observations continue
the exploration; they do not supersede the clarification that no route has been chosen.

**Assistant's qualification:** This expectation is common, not universal across modern
languages. Value semantics concern copying and identity, not necessarily a short
lifetime: values can live inline in long-lived heap objects. Valhalla's opt-in also
restricts identity and mutation; it is not just a storage hint or C# mutable-struct
behavior under another spelling. Its wrapper migration therefore needs separate
consideration from neoCLR's ability to reference mutable values.

**Evidence and action:** Added this reasoning to the existing entry, with no code or
architecture migration. The [C# struct specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/structs)
and [Java team's value-class demonstration](https://inside.java/2025/10/27/try-jep-401-value-classes/)
support the distinctions. The Java demonstration includes Integer and LocalDate in
preview mode; it does not establish universal deployment or allocation elimination.

**Author's clarification about lightweight values:** The earlier lifetime observation
was general: a value can live within a long-lived heap object, while still being
lightweight in representation. **Assistant's response:** Acknowledged that its lifetime
qualification had been narrower than the author's point required.

**Author's preference for the explored model:** The author now understands why C# uses
the term struct and recognizes that .NET and Java's emerging value behavior differ,
with architecture and backward compatibility informing those designs. This was not a
request to adopt Java's behavior. The preferred concept is a value class that can
preferably be inlined, rather than a separate struct equivalent.

**Assistant's interpretation:** Described an ordinary class with an explicit value
contract, with copying, mutation, identity and reference access still requiring separate
decisions. Inline storage would be enabled by the contract, not promised in every use.
This interpretation does not establish those unresolved semantics as agreed decisions.

**Action and scope:** At the author's request to record, appended these clarifications
to the ongoing exploration. The preference concerns the shape of a possible value-class
model; the broader decision to replace current defaults remains open. No runtime,
metadata or language change was implemented.

### 24. Separate storage, mutability, identity and equality

Recorded 2026-09-09.

**Author's observation:** A possible value class resembles a record locked against
modification. The author wants to distinguish the individual characteristics, especially
storage and reference/value access, and points to C# record classes as objects that
participate in value equality through Equals/GetHashCode while retaining reference use.
Record structs and record classes combine these properties differently.

**Assistant's response:** Distinguished equal contents from being the same object.
A record class can preserve observable reference identity while providing generated
value equality; assigning it still copies the reference. A record struct copies its
value. Neither record syntax nor value equality alone guarantees deep immutability.
Ordinary classes can opt into value equality, but do not all do so automatically.
The [C# record reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/record)
provides the comparison, including equal records with distinct reference identities.

**Open design question:** Which characteristics should a possible value-class declaration
combine, and which should remain independent? Accessing storage through a reference
need not, by itself, settle the language's equality or object-identity contract.

**Action:** Recorded the distinction and checked the C# comparison. No particular
combination of semantics or change to neoCLR's current defaults was selected.

### 25. Explore runtime contracts, then return to the intended object experience

Recorded 2026-09-09 at the author's request.

**Author's question:** Should these characteristics be runtime functionality or
conventions built on existing interfaces and other facilities?

**Assistant's provisional answer:** Equality, hashing and record conveniences could
use ordinary methods and compiler generation. Storage, copying and reference validity
need shared execution contracts. Immutability or absence of identity would need runtime
enforcement if optimizations relied on them. Suggested identifying the missing guarantee
before adding a single category that bundles several characteristics together.

**Author's alternative:** Under a possible reference-default model, neoCLR could mark
some types as inline, implementing struct-like behavior directly without a ValueType
base-class split. The author proposed an illustrative IL declaration:

```text
.type inline value Int32
```

The author considered primitives with object-like wrappers but no separately declared
instance data, and explicitly left open whether the inline flag should trigger value
semantics. This was exploratory spelling, not an implemented or selected syntax.

**Assistant's response:** Distinguished an inline representation hint from a contract
that makes assignment copy contents. Suggested that `value` and `inline` could express
separate concerns, while noting that primitive payload representation is another matter:
no declared fields alone does not establish an integer's runtime representation.
Whether flags belong to a type or each use remained open. These were assistant proposals,
not decisions attributed to the author.

**Author's clarification of the priority:** Preserve object-like qualities on primitive
types and allow user-defined types to have that behavior. The low-level representation
need not mirror the high-level abstraction. The author compared the exploration to
Java's effort to bridge primitive and wrapper types through runtime optimization with
semantic consequences, without requesting adoption of Java's model. The aim is familiar
.NET usage while improving runtime architecture and design, rather than reproducing
its historical decisions. The experiment remains useful for learning about alternatives.

**Assistant's correction and suggested evaluation:** Acknowledged moving too quickly
toward modifiers and categories. Reframed the goal as methods, interfaces, generics and
coherent APIs for primitive and user-defined data, with efficient representation and
precise copying, aliasing, mutation and identity contracts. Proposed testing one small
API with both a built-in number and a user-defined value, including generic/interface
use. That test was suggested, not performed or selected as the next implementation task.

**Action and remaining scope:** Recorded this exchange. No new metadata flag, value-class
category, convention or default behavior was implemented. The broader model remains
under evaluation; this entry does not override the author's earlier clarification that
changing routes has not been decided. Prior .NET/Valhalla comparisons in entries 22–24
provide context; the Java analogy here is attributed reasoning, not a new release or
performance claim.

### 26. Evaluate improvements by continuity of developer experience

Recorded 2026-09-12.

**Author's evaluation criteria:** An improved .NET should not fundamentally change how
developers work. The author considers it probably wise to associate reference/value
semantics with types, while leaving their representation open. The platform should
avoid introducing manual management and should simplify inconsistencies in the familiar
experience. Runtime APIs should probably favor Result and Option unions; expected,
recoverable failures should prefer Result, with exceptions reserved for irrecoverable
Faults. Keep the .NET instruction set as the basis, make the platform easy for languages
such as Raven and C# to target, and support nullable values even if APIs prefer to avoid
them. Nullability could later have an updated or unified model.

**Author's architectural emphasis:** Distinguish language responsibilities from runtime
responsibilities. Runtime implementation details can remain hidden from developers while
still providing meaningful benefits. Familiar usage does not require identical internals.

**Assistant's assessment:** These criteria shift the evaluation toward whether runtime
changes preserve productive, familiar source-level behavior. The existing experiment's
managed references and storage machinery can be assessed as implementation capabilities,
without requiring all their distinctions to appear in everyday source syntax. Type-level
defaults remain a proposed direction, not an implemented replacement for current semantics.

**Tradeoff identified by the assistant:** Moving expected failures from exceptions to
Result changes API and control-flow contracts. Familiar targeting should therefore be
tested with concrete language mappings, including how existing exception-based code
would migrate; familiar instruction spelling alone does not establish compatibility.
Fault containment and cleanup, nullable representation, type defaults and their metadata
encoding remain contracts to evaluate, not details settled by this discussion.

**Action and scope:** Recorded the criteria and assessment. No runtime or compiler change
was made, no syntax or metadata encoding selected, and no architecture migration started.
Existing [reference-experience evidence](experiments/reference-experience/README.md),
[type-design discussion](type-design.md) and [design research process](design-research.md)
remain starting points for a subsequent evaluation rather than proof of the proposed model.

### 27. Evaluate three projections before changing implementation

Recorded 2026-09-12.

**Author's question and instruction:** Asked what to do next, then directed the assistant
to continue with its proposed bounded architecture evaluation of the order workflow.

**Assistant's proposal:** Compare current explicit modes, shared type-declared defaults
and language-only defaults; inspect construction, assignment, parameters, collections,
interfaces and Result/Option use. Produce concrete code comparisons and a recommendation
before changing runtime behavior.

**Action and findings:** Wrote the [evaluation](reference-defaults-evaluation.md), inspected
metadata, lowering, slot and return checks, emitted the current workflow's IL, and ran
22 focused tests plus the SDK 10.0.100 C# baseline. All passed. Existing sharing/snapshot
behavior works; cleaner spelling can plausibly use current signatures. Reference-slot
outputs remain a real compatibility gap, while shared declaration intent matters for
importers defining new uses of a type. More automatic heap construction can add facade
allocations; no performance improvement was inferred.

**Assistant's recommendation, not a selected migration:** Test shared usage defaults that
lower to explicit signatures, with a bounded opt-in compiler/import experiment before
considering an enforced nominal category. No experimental compiler was needed to establish
the existing lowering evidence in this slice. Illustrative alternatives were not executed.
The author has not yet selected the recommended implementation or changed current defaults.

### 28. Keep runtime improvement primary and Neo as its test frontend

Recorded 2026-09-12.

**Author's clarification:** Runtime behavior and Neo language behavior are separate
levels. Both can improve, but the main focus is the runtime. Neo exists primarily to
demonstrate and test patterns by compiling to IL and running on neoCLR. Seek some
compatibility with the existing CLR, avoid excessive divergence, retain the good parts
and their foundations, and improve what can usefully be improved now.

**Assistant's response:** Acknowledged that improving Neo's projection is not a substitute
for evaluating runtime contracts. The earlier compiler/import experiment remains a
possible validation tool, not the project's primary objective. Proposed first assessing
reference/slot distinctions, construction/copying, initialization/nullability and generic
interface use against CLR behavior. The exact compatibility level is still open; no
binary compatibility commitment was inferred from the author's statement.

**Action:** Updated the [evaluation](reference-defaults-evaluation.md) to put that runtime
assessment before the optional compiler experiment. Preserved the previous findings and
recorded why the priority changed. No runtime or language implementation changed.

### 29. Make an existing Raven compiler target the runtime and its library

Recorded 2026-09-12.

**Author's goal:** C# and Raven should target neoCLR with limited adaptation because
its runtime interface remains similar to CLR. The author clarified that exact exception
compatibility is not required: this is a new platform, and migrated code should adapt
expected failures to Result instead of centering its design on exceptions.

**Assistant's response:** Distinguished source targeting from unchanged binary execution
and separated ABI concerns from metadata, IL, library and behavioral contracts. Identified
cleanup/Fault boundaries and exception-centered API migration as explicit design work.

**Author's proposed experiment:** Address the lack of binary metadata support and add a
neoCLR target to the existing Raven compiler. It should compile against neoCLR's runtime
surface and load neoCLR's own library. The author then requested documentation and a
new branch for this experiment.

**Assistant's proposed order:** Inspect Raven's backend, define the artifact/library
contract from actual dependencies, then compile and run a Console HelloWorld before
expanding to user-defined types and Result. Raven's backend details are not yet verified.

**Action:** Created `codex/raven-neoclr-target` from `64024d6` and documented the
[experiment](raven-target-experiment.md), including the first acceptance milestone,
binary-format choices and scope limits. Linked it from compatibility and format guides.
No Raven/compiler or binary-loader implementation was performed in this documentation
slice; no new reference/value default was selected.

### 30. Inspect the existing Raven backend before inventing a target

Recorded 2026-09-12.

**Author's instruction:** Continue with the planned experiment slices, beginning with
Raven inspection. **Assistant's action:** Located Raven, read its repository instructions
and compiler-investigation skill, and pinned its clean checkout at d92b02812740ae052f277c23151e9cc208f7672d.

**Findings:** Raven imports PE symbols with MetadataLoadContext and emits through
PersistedAssemblyBuilder/ManagedPEBuilder plus Cecil normalization. It already has a
target-core-library option and retargeting tests. Those hooks do not provide a full
neoCLR target: .NET framework discovery, host core identities, Unit/void mapping and
exception-based framework projections still need explicit target contracts.

**Outcome:** Produced the [integration map](raven-backend-integration-map.md), with pinned
source links, adaptation points and a proposed minimal Console probe. Source and test
bodies were inspected; no Raven build/test or target execution was claimed. Raven was
not modified. The next slice should define the minimal library/call contract and run
an emission/reference-facade probe before selecting a binary format.

## 31. Minimal Raven target probe — recorded 2026-09-12

**Author direction:** Continued the experiment after its backend investigation. The
standing direction is to test existing compilers against neoCLR's own library and
binary surface, with the runtime as the primary focus.

**Assistant action:** Built the pinned Raven compiler and added an emission-only
Console declaration probe plus a normal .NET control. Inspected the output references,
helper definitions, methods and entry point; tested nonexistent members and omission
of the Console library. Raven source was not modified.

**Outcome:** Target Console binding and core-reference rewriting worked. The omitted
library still resolved through the host environment, so the isolation negative failed.
The declaration fixture remains incomplete and output retains mscorlib. These findings
are recorded in the [minimal contract](raven-minimal-target.md) and reproducible report;
no neoCLR execution or complete compiler target is claimed.

**Assistant proposal, still open:** Evaluate signature-driven CLI-to-runtime translation
for no-result calls alongside the binary container decision. This may preserve the
existing writer and inhabited Void model, but requires explicit validation and debugging
mapping. The author has not selected that implementation or changed class/value defaults.

## 32. Binary profile and dependency audit — recorded 2026-09-12

**Author direction:** Continued the Raven target experiment after the emission probe.

**Assistant response/action:** Investigated the explicit host-assembly seeding in Raven's
metadata setup. Added an experiment resolver that only accepts supplied assemblies and
checks metadata reference resolution, with positive and negative fixtures. Documented
CLI PE reuse and signature-driven translation as the bounded experiment direction.

**Outcome:** The audit rejects the incomplete target and host-fallback artifacts. It
accepts self-contained metadata and an explicitly supplied dependency, and detects
missing or mismatched identities/types/members. This is a tested tooling boundary, not
a fix to Raven's binder or an implemented runtime loader. See the
[binary-profile decision](raven-binary-profile.md) for evidence and limits.

**Still open:** The compiler needs isolated target resolution and complete core
declarations. Generated-helper policy, input stack validation and reader implementation
remain before execution. The assistant's format choice is scoped to this experiment;
the author's continuation is not recorded as explicit approval of every design detail.

## 33. Isolated compiler metadata import — recorded 2026-09-12

**Author direction:** Continued after the binary-profile and dependency-audit slice.

**Assistant action:** Implemented opt-in explicit-only metadata imports in Raven on
`codex/neoclr-target-resolution`, selecting the metadata core independently of the
compiler host. Preserved default .NET import behavior and blocked incremental state
reuse across import policies. Updated the neoCLR probe to test target Console binding
and missing-library diagnostics through the new API.

**Outcome:** Raven commit `1d7341fa64a66b514e5e68031b6d072d8140ea3a` passed 15 focused
framework/import tests and the .NET 10/11 target matrix. The updated probe binds the supplied Console fixture and rejects omitted
Console instead of finding it on the host. The core reference artifact remains
incomplete; this is compiler integration progress, not runtime execution. Exact
revisions and validation are linked from the [probe instructions](experiments/raven-target/README.md).

**Still open:** Complete neoCLR core declarations, generated-helper handling, profile
verification and binary loading. No change to neoCLR's class/value defaults is implied.

## 34. Minimal core declarations — recorded 2026-09-12

**Author direction:** Continued the experiment and specified that any Raven repository
work must remain isolated on a feature branch. The assistant confirmed Raven was already
on `codex/neoclr-target-resolution`; this slice required no additional Raven changes.

**Assistant action:** Built a small metadata-only core reference assembly and compiled
the static-call corpus with that single reference. Added negative Console/signature
checks and corrected Cecil inspection order so synthesized in-memory references are
not reported as artifact dependencies.

**Outcome:** The Console, empty/nested-call and Int32-return examples bind and emit with
closed metadata dependencies. This is a reference surface for testing; neoCLR has not
executed these binaries. Earlier application mscorlib inventory entries included a
synthetic Cecil reference; the old incomplete fixture's real dependency remains.

**Still open:** Bind the reference surface to real System implementations, decide
helper admission, and implement/validate the binary reader and call translation. The
[core declaration contract](raven-core-declarations.md) records the evidence and limits.

## 35. Compatibility work belongs primarily in neoCLR — recorded 2026-09-12

**Author direction:** Make as few changes to Raven as possible. Generally useful metadata
improvements can belong there, but most work should improve neoCLR's CLR compatibility.

**Assistant proposal/action:** Reconsidered the earlier blanket Void adaptation and added
an explicit no-result return convention to neoCLR for static IL methods. Calls and returns
use the existing instructions, with verifier/runtime enforcement. Existing inhabited Void
methods retain their behavior. No Raven changes were needed.

**Author clarification:** Void should remain a real type usable as a generic argument,
without necessarily changing ordinary IL return behavior. The author pointed to runtime
async as a comparison and emphasized preserving the metadata/instruction interface.

**Assistant response:** Kept type/storage semantics separate from the return convention,
added generic Void regression tests, and recorded the runtime-async design comparison as
future groundwork rather than expanding this slice into async implementation.

**Further author direction:** Align with .NET value-type/reference-type semantics later.
Avoid working around that model unless necessary to demonstrate the project's direction.
The assistant treats this as the experiment's intended direction, not an implemented
migration or a requirement to redesign the type model in this slice.

**Outcome:** Focused runtime tests exercise empty/nested calls and the existing Console
library boundary. Binary reading and execution of Raven artifacts remain separate work.
The [return-convention document](no-result-methods.md) records the restricted first subset,
host-envelope behavior and the still-needed library binding policy.

## 36. Void type identity versus stack representation — recorded 2026-09-12

**Author clarification:** “In the neoCLR, void will be treated as a real type and the
compiler targeting it has to adapt to it”, followed by “Even if void doesn't really
put anything on the stack.”

**Assistant response:** Distinguished real type participation from evaluation-stack
representation. The previous no-result slice does not make inhabited generic Void's
current internal representation a permanent contract. Targeting compilers must understand
neoCLR's Void semantics; binary generic encoding and uniform zero-stack handling remain
open implementation work.

**Action/outcome:** Updated the [return-convention document](no-result-methods.md).
Continued the independent [standard CIL decoder](cil-decoder.md) groundwork with byte
offsets and token preservation. This does not implement the broader Void representation
change or load Raven binaries.

## 37. Familiar binary contracts and a useful Raven milestone — recorded 2026-09-12

**Author direction:** New runtime semantics should not automatically change metadata or
the instruction set. Preserve .NET's major behavioral expectations, retain intentional
Void and library improvements, and make adaptation easy for existing compilers.

**Further clarification:** The mission is a minimal, useful Raven subset targeting neoCLR.
Aligning value/reference type semantics is central; the original explicit-reference model
should not drive compatibility workarounds.

**Assistant response:** Agreed to finish the bounded PE/body reader already in progress,
then prioritize runtime type classification using a class-alias/value-copy Raven scenario.
Binary parsing is supporting work, not the primary acceptance milestone.

**Action/outcome:** Added standard PE32 container and tiny/fat method-body inspection with
.NET-emitted fixture comparisons. Updated the [experiment order](raven-target-experiment.md#next-milestone-useful-raven-subset).
The type migration and actual execution of Raven binaries remain unimplemented; Raven
itself was unchanged by this slice.

## 38. Raven must consume the runtime library — recorded 2026-09-12

**Author clarification:** The goal is staged support for Raven compiling to neoCLR-compatible
IL and consuming neoCLR's runtime class library to build simple programs.

**Assistant response:** Made the end-to-end path the next acceptance test: compile Raven,
import a bounded emitted program, and execute its Console call against the real neoCLR
System implementation. Proposed a neoCLR-owned experiment bridge, with no further Raven
changes, while the native reader's metadata binding remains incomplete.

**Action so far:** Completed nominal class reference defaults. The subsequent bridge now imports the four static Raven programs and all four verify/run
on neoCLR, with HelloWorld using the real System.Console. The [workflow and recorded
results](experiments/raven-target/README.md#first-runtime-library-execution-milestone)
distinguish this tooling path from direct native PE loading.

## 39. VS Code as a target development environment — recorded 2026-09-12

**Author directive:** Have the development experience for targeting neoCLR in VS Code.

**Assistant response:** Proposed staged editing/build/run support for Raven targeting
neoCLR, then diagnostics and source-mapped debugging. Kept host .NET debugging distinct
from integration with neoCLR's own execution/debugger.

**Author correction:** Full debugging is not the MVP. The minimum is code completion in
Raven files/projects targeting neoCLR; focus on target support first.

**Assistant response/action:** Corrected the [experiment plan](raven-target-experiment.md#vs-code-development-experience)
to require completion against the actual target library, with compiler/project and language
server settings agreeing. Debugging is optional later work. The command-line Raven/library
path works; target-aware completion remains planned.

## 40. Unions belong in the MVP — recorded 2026-09-12

**Author directive:** Demonstrate union usage in the MVP as well.

**Assistant proposal:** Use a Raven program consuming a real neoCLR Result/Option API,
with pattern matching for both outcomes. Proposed Math.Abs(Int32) to show successful
absolute value and overflow, preserving the runtime's Result-based contract.

**Action/outcome:** Added this to the [MVP acceptance criteria](raven-target-experiment.md#mvp-acceptance-criteria-and-union-directive-2026-09-12).
The current slice expands real Math/Console consumption and demonstrates completion through
Raven's compiler API using the same target declarations. Generic union import and the
union demonstration remain the next support work; neither is claimed complete.

### Follow-up evidence — recorded 2026-09-12

**Assistant investigation and action.** Following the author's continuation, added a
Raven sample consuming Math.Abs as Result and matching its success/error case types.
Binding against isolated declarations succeeds, but emission fails while resolving
Result as a host runtime type. No union program has executed. The assistant proposes
fixing target metadata emission before expanding the neoCLR importer; this is a
technical proposal, not an additional author decision. Raven remains unchanged in
this slice. The [probe and next steps](raven-target-experiment.md#union-metadata-probe-and-emission-blocker-2026-09-12)
record the evidence and distinguish the remaining compiler, library, and runtime work.

### Emission follow-up — recorded 2026-09-12

**Assistant action and corrected finding.** On the next continuation, the assistant
changed Raven on its existing feature branch to preserve target metadata types during
retargeted emission and correct generic/byref member references. The sample emits and
passes the dependency audit. Inspection also showed that the earlier declaration
lacked the Value property needed for Raven to recognize a union: its binding success
alone had not established union matching. The probe now includes that recognition
contract and checks for both case extractors. No runtime union execution is claimed.
The [follow-up](raven-target-experiment.md#target-metadata-emission-follow-up-2026-09-12)
records this correction and the remaining API-projection and importer work. No new
author design decision is inferred from continuation.

## 41. A concrete POC demonstration — recorded 2026-09-12

**Author direction.** The author wants a demo of a .NET-like runtime with its own
runtime class library, Result and Option rather than exception/null-centered flows,
Void as a valid type parameter, and Raven targeting with basic VS Code completion.
The author identifies this combination as sufficient to distinguish and justify the
experience; this does not request broad compatibility or a full debugger.

**Assistant response and action.** The assistant acknowledged these as the POC
priorities. During the current Result slice, the first Raven Result sample executed
on neoCLR against the real Math and Result implementations, printing 42 and Overflow.
The assistant added control-flow/initialization validation, malformed-input probes,
run instructions and the executable regression artifact. Option, generic Void from
Raven, and actual VS Code integration remain unfinished; compiler completion alone
is not reported as editor completion. The [POC priorities](raven-target-experiment.md#poc-demonstration-priorities-2026-09-12)
distinguish the author's criteria from the assistant's proposed next-step order.

## 42. A locally testable VS Code integration — recorded 2026-09-12

**Author direction.** During the editor slice, the author requested instructions for
actually testing the integration in VS Code, observing that no available build yet
supported it, and suggested a local updated SDK and VS Code extension build. The author
then clarified the term “VS Code extension.”

**Assistant response and action.** The assistant added project-level explicit metadata
configuration in Raven on its isolated branch, prepared an editable Result demo, and
built and installed a local VSIX containing the updated compiler library/language server.
Headless LSP checks covered Math, Console and System names; a manual VS Code interaction
visibly returned Abs, Max, Min and Sign. The temporary UI test edits were discarded. A separate SDK bundle was built and
installed alongside the existing SDK, with its version command checked.
The assistant documented the setup and the distinction between editor support and the
separate runtime import pipeline in the [walkthrough](experiments/raven-target/VSCODE.md).

**Outcome and remaining work.** Target-aware completion now works in the installed
extension. This does not make the regular Raven Build/Run buttons neoCLR commands.
Connecting saved project edits to compilation/import/execution remains the next slice.
The local build is experimental and is not a published release.

## 43. Finish the development loop, then propagation — recorded 2026-09-12

**Author direction.** While the assistant connected saved projects to runtime execution,
the author requested union propagation, then suggested it could be the next milestone.
The author subsequently directed completion of this milestone, leaving interfaces and
related capabilities for later.

**Assistant response and action.** The assistant kept the current milestone focused on
saved-source build/run tasks and recorded propagation as next. Compilation exposed an
inferred .NET framework attribute; a small Raven project-system fix removed that inference
for explicit-core targets. The new bridge reuses the editor project model and existing
runtime admission, with isolated outputs for each build.

**Outcome.** Result, Option and generic Void execute from project sources. A saved edit
changes the output, while compiler/import failures stop without running stale artifacts.
Propagation and interfaces remain planned, not reported as implemented. The dedicated
VS Code tasks still differ from Raven's normal toolbar pipeline.

## 44. Close the POC and establish interfaces first — recorded 2026-09-12

**Author direction.** The author stated that union propagation can wait until the
interface contract exists, asked what should come next, and proposed closing and tagging
this milestone. This revises the ordering recorded in entry 43; it does not erase it.

**Assistant response and action.** The assistant assessed the bounded POC as complete,
recorded its evidence and remaining limits, and chose a local annotated experiment tag,
`milestone/raven-poc-2026-09-12`, pinning the Raven dependency. This does not publish a
release or push either repository. The next proposed milestone is a documented interface
contract followed by a concrete class/interface dispatch example from Raven on neoCLR.
Union propagation is explicitly deferred; the detailed interface scope remains to be
validated against CLI metadata, runtime behavior and the end-to-end example.

## 45. The existing runtime library is the desired demo — recorded 2026-09-12

**Author direction.** After closing the bounded POC, the author said the eventual goal
is to run the same, adapted runtime library surface previously used for Neo, then
clarified: that is the desired demo.

**Assistant response.** The assistant distinguished the completed integration checkpoint
from this broader goal. The existing neoCLR library remains the implementation foundation;
Raven's declaration assembly is its compiler-facing view. The assistant proposed using
an existing collection/iteration contract for the interface milestone so it advances
that demonstration rather than only exercising a disconnected interface example.

**Action and open scope.** Recorded the shared-library direction in the runtime library,
experiment and roadmap documents. The collection slice is a proposal; its exact API
adaptations and runtime/import requirements remain to be assessed. No new library or
interface implementation is claimed by this documentation change. The prior POC tag
remains unchanged, and propagation remains deferred until the interface contract exists.

## 46. Type categories, including arrays, replace the old default — recorded 2026-09-12

**Author correction.** “We intend to go away from the value by default and the explicit
references, so we align with the .NET semantics around the categories of types. And
that also applies to arrays.” This clarifies the earlier migration direction after the
assistant repeatedly preserved Neo's original array/value model in runtime slices.

**Assistant response.** Acknowledged that .NET-style type categories are the intended
ordinary model. The old value-by-default and explicit-reference projection is transitional,
not a second public model to preserve. Normal class/interface/array references remain
distinct from managed byrefs to slots. The recent arrayref/array.new split is migration
machinery rather than the desired final user experience.

**Action and outcome.** Updated the roadmap, class-semantics and managed-array documents
to distinguish the target from current implementation. The implementation has not been
migrated by this documentation change. Upcoming slices need to migrate Neo/library
classifications and array spelling/lowering alongside the Raven collection work. Earlier
positions and implementation descriptions remain available; separate stack-buffer
facilities are not implied to be ordinary arrays.

## 47. .NET is the baseline; faults are not exception classes — recorded 2026-09-12

**Author direction.** The author said to stay with .NET semantics unless an improvement
is found. They accepted Result-based class-library error flow, while clarifying that
exception-like terminal failures may mean faults: “I don't however think we should model
Exceptions as classes.” The corrected message uses exception semantics, not Result
semantics, for that fault distinction.

**Assistant response.** Proposed expected recoverable failures as Result data and
terminal runtime faults reported at the host boundary with diagnostics. No guest
Exception hierarchy is required. The assistant emphasized completing the type-model
migration rather than extending the old defaults.

**Action and outcome.** Inspected `src/lib.rs`: Fault is already a host-side Rust record
with diagnostic fields, documented as uncatchable by guest code. Updated the API policy,
error policy, roadmap and experiment documents. Corrected stale text that still called
the selected type model an undecided investigation, and an old roadmap instruction to
insert dummy Void values for CLI calls. This commit changes documentation only. Cleanup,
containment and future async fault boundaries remain open; no catchable guest exception
mechanism or runtime migration is claimed by this action.

## 48. Keep Neo outside the CLR/Raven experiment — recorded 2026-09-12

**Author correction.** After the assistant began changing Neo array lowering, the
author said “leave neo out of this” and clarified: “We work with neoCLR and Raven
as we adapt neoCLR to get those CLR semantics that are familiar to developers.
Neo doesn't need to be updated here”. This narrows the scope of the migration in
entries 46–47; it does not reverse the runtime type-category direction.

**Assistant response and action.** Acknowledged the correction and removed the
uncommitted Neo frontend changes and the runtime array-construction helper instructions
introduced solely for them. Updated the roadmap and active design documents to exclude
Neo migration from this experiment. Existing committed neoCLR ordinary array/class
reference support and Raven array import remain intact. No Raven files were changed.

**Next work.** Adapt the actual collection/iterator runtime library and admit its
class/interface calls through the Raven bridge, with runtime and Raven acceptance tests.
That implementation remains pending; this scope correction does not complete it.

## 49. Configure Raven's library contracts by target — recorded 2026-09-12

**Author direction.** The author asked for a way to specify classes and contracts whose
names differ in neoCLR, including interfaces without the I prefix and iteration APIs.
Raven must keep targeting .NET while also supporting the experimental neoCLR target.

**Assistant response.** Proposed a target contract description for compiler-recognized
roles, including types, members and signatures. Distinguished Iterable (source) from
Iterator (cursor), and ordinary API name binding from compiler-driven iteration lowering.
The assistant proposed validating actual target symbols rather than global name replacement.

**Action and outcome.** Recorded the [target-contract proposal](raven-target-contracts.md)
and inspected Raven's existing core metadata options and explicit .NET iteration names.
No Raven changes were made. The concurrently implemented runtime collection profile
adapts existing algorithms and passes runtime tests; it does not yet enable Raven's
collection import or implicit iteration. Compiler configuration, target diagnostics and
.NET/neoCLR editor regression tests remain planned on Raven's feature branch.

## 50. Target iteration lowering and the cleanup gap — recorded 2026-09-12

**Author direction.** The author clarified that when Raven targets neoCLR, foreach-style
lowering should use Iterable and that target's contract. They asked to note the missing
Dispose call and consider fixing it. They subsequently proposed a finally-like alternative,
possibly defer in Raven, given neoCLR's absence of recoverable exception handling.
They concluded: “That should be noted for now”.

**Assistant response.** Agreed to select actual target interfaces and method signatures
rather than only rename emitted references. Reported that Raven's existing iterator-loop
emitter does not automatically dispose on its default .NET path either. Proposed treating
that as a separate cleanup issue. Distinguished a possible defer syntax from runtime
support for cleanup, and suggested evaluating finally-only CLI regions without guest
Exception classes. These cleanup choices are proposals, not author-approved implementations.

**Action and outcome.** Implemented configurable synchronous iteration in Raven's feature
branch (`c28657859dda6b473d182445d91e5d44469a6974`), with 41 focused tests passing.
The neoCLR probe selects Iterable/Iterator and executes a for-loop sample covering normal
completion, break and return. Recorded [cleanup/defer discussion](raven-target-contracts.md#recorded-cleanup-discussion-no-implementation-yet)
for future work as requested; neither automatic disposal nor defer was implemented.
Project configuration and VS Code integration of this option remain pending.

## 51. Scoped cleanup and the target boundary — recorded 2026-09-12

**Author direction.** The author proposed deterministic destructors using Disposable,
Closable or an iteration of those contracts, then clarified that the intended feature
is deterministic resource cleanup, separate from deconstruction. They explicitly directed
that possible NeoCLR-specific behavior must leave existing CLR support working the same.
Future NeoCLR behavior may differ; that is not a direction to change the .NET target.

**Assistant proposal.** Distinguish cleanup responsibility owned by a scope from an
object's memory lifetime: multiple reference aliases must not each trigger disposal
when leaving scope. Disposable/Closable are candidate hooks. Cleanup failure policy,
including Result-returning close operations, remains to be designed. These are proposals,
not implemented destruction or ownership semantics.

**Action and outcome.** Recorded the boundary in both target-contract documents. Kept
cleanup/defer out of the current slice, following the earlier direction to note it only.
The separate project/editor integration now selects iteration through evaluated project
properties, runs the adapted collection library and verifies completion and inferred
loop-binding types through the language server. Default project settings remain unchanged.
See [target integration and cleanup boundary](raven-target-contracts.md#project-and-editor-integration-2026-09-12).

## 52. Fundamental demonstration, with Raven isolated — recorded 2026-09-12

**Author direction.** Full integration is not necessary. Demonstrate fundamental things
already shown with Neo: .NET-aligned value versus reference categories and Raven adapting
to neoCLR's runtime library and type names. Evaluate changes to Raven separately as the
experiment advances and differences become clearer. Keep Raven on a “Separate branch,
as today”.

**Assistant proposal and action.** Proposed combining the existing collection and union
profiles in one small program, leaving propagation and cleanup deferred. Kept Raven on
`codex/neoclr-target-resolution`; this slice changes no Raven files. Added a product
workflow using List/ArrayList, Option, Result and Option<Void>, and paired checks for
class/array aliases and value-carrier copying. Recorded the bounded scope and separate
future evaluation of Raven changes; no merge or general Raven adoption is implied.

**Outcome.** The combined probe compiles, imports and executes the success, overflow
and missing-product paths on neoCLR. The project/editor verification evidence is linked
from the [fundamental demo](experiments/raven-target/README.md#fundamental-raven-demonstration).
These examples establish a useful subset, not full migration or arbitrary type import.

## 53. Trying the tools locally before a public bundle — recorded 2026-09-12

**Author direction.** Asked to update the SDK and VS Code extension so they could try
the experiment, and asked whether more work was needed first. Added that, when ready,
the project should produce a build other people can try themselves.

**Assistant response.** Proposed testing the current bounded demo now, using a local
experimental SDK/extension and a prepared workspace. Proposed a later downloadable bundle
with pinned neoCLR/Raven tools, the extension, demo and a clear build/run path. Local
feedback should expose usability problems before distributing it more broadly.

**Subsequent direction.** The author proposed merging neoCLR work into neoCLR main,
then updating documentation and producing a public build/release. They also suggested
publishing matching Raven tools with NeoCLR. The assistant agreed that the tools should
be pinned experimental SDK/extension artifacts, without merging Raven's branch or
presenting them as a general Raven release.

**Propagation follow-up.** The author proposed adding propagation before the public
preview for a complete NeoCLR union story, and specified that its interface belongs in
the NeoCLR runtime library. The assistant proposed completing the local installation
first, then validating Raven against that runtime-owned contract while preserving its
.NET behavior. This is the next feature slice, not functionality in the current build.

**Release verification.** The author also requested checking match expression and
statement syntax and documenting what works and does not work for the release. The
assistant agreed to require compilation and execution evidence, not just parsing or
completion. The pre-merge full neoCLR suite subsequently found a generic Clonable
assembly-resolution failure; merge readiness remains pending that regression.

**Scope.** Public distribution remains a future goal, not a release announced or
published by this exchange. Raven remains isolated on its experiment branch. See the
[local installation instructions](experiments/raven-target/VSCODE.md) for installation
status and the existing bounded-target limitations.

## 54. Normal output and the release API boundary — recorded 2026-09-12

**Author direction.** Asked whether `=> Void` was test output and agreed that it should
not appear during normal execution. Requested propagation and APIs for reading/writing
text files for a complete release; reflection APIs would also be nice to have.

**Assistant action and scope.** Removed the default runner return-value suffix and
introduced opt-in `--show-result` diagnostics on stderr. Updated CLI/demo expectations.
Recorded propagation and text-file APIs as required Raven-target release coverage and
reflection as desirable. Existing Neo/runtime implementations are not evidence that
Raven can already consume all those APIs; target admission and end-to-end checks remain
necessary. This output slice does not implement those later library features.

## 55. Demonstrate the foundations already built — recorded 2026-09-12

**Author clarification.** The goal is to demonstrate the fundamentals of the new
platform, its differences from .NET and a strong foundation. A complete foundation is
not required; the building blocks already present are important for showing what the
platform is about.

**Assistant response.** Kept the preview tied to the bounded demo: familiar type/IL
behavior, the target library, Result/Option with propagation, basic file APIs and Raven
tooling. Regression repair supports reliability of existing capabilities; it is not a
reason to expand toward a complete platform or runtime library before the preview.

**Date/time addition to the demo.** The author asked to include the date/time libraries
as relatable evidence of the platform, even if incomplete. The assistant proposed
exposing the existing bounded date/time and local-clock surface through Raven, with
formatting/globalization still outside scope. This is release-demo scope, not a claim
that Raven import already supports those APIs.

**Acceptance emphasis.** The author emphasized that the fundamentals, error handling,
and basic VS Code completion through Raven integration are key. The assistant agreed
to treat their combined end-to-end demonstration as acceptance criteria rather than
feature count or completeness of the entire platform.

## 56. First propagation library building block — recorded 2026-09-12

**Prior author direction.** Propagation should be part of the public preview, and its
interface belongs to the NeoCLR runtime library. Raven remains an isolated experiment.

**Assistant proposal.** Inspection found that Raven's protocol includes two extraction
methods and a static residual factory, while neoCLR supports only instance interface
members. Proposed a prefix-free Propagatable extraction interface and concrete carrier
factories, explicitly documenting that this does not yet enforce a static abstract
interface contract. This is a provisional implementation choice, not an author decision
to permanently omit static interface support.

**Action and outcome.** Added the interface and Result/Option methods, including Void
absence and conditional initialization. Four new tests and related runtime suites pass;
the existing Raven demo still executes. The [contract and limitations](propagation-contract.md)
record the .NET comparison and next Raven projection/import work. No `?` support or
Raven changes are claimed by this first slice.

**Subsequent implementation outcome (same date).** The assistant added a target-selected
Raven protocol and a bounded Result<Int32,OverflowError> `?` program. Execution verifies
success continuation and early residual return. Inspection found implicit exception
capture in the default Raven lowering; the opt-in target uses result-only lowering,
while existing .NET propagation tests still pass. Import checks reject malformed
contracts and unsupported thrown values. Option/Void projection and refreshed tools
remain open; continuation was not treated as approval of a permanent protocol design.

## 57. Familiar ArrayList construction — recorded 2026-09-12

**Author direction.** Put desired initial capacity in the constructor and provide a
parameterless constructor, resembling .NET List. The assistant proposed retaining
Allocate for compatibility; the author corrected this: “No need to keep compat”.

**Action.** Replaced Allocate with constructors and migrated active examples/tests.
Default construction starts empty with zero capacity; explicit capacity reserves storage.
The Raven projection uses ordinary class construction. Historical published examples
retain their recorded API. Validation includes capacity, growth and negative arguments.

## 58. Void as a type versus a return marker — recorded 2026-09-12

**Author question and correction.** Asked whether Void should be neoCLR's unit type,
then rejected Option<Void> as the motivating example: None already handles absence,
whereas Result<Void, MyError> expresses completion or failure. Void can remain a
technically valid type argument. Asked whether methods should always return a unit
value or preserve CLR void behavior.

**Assistant proposal and author decision.** Proposed unit semantics for Void in the
type system, preserving the CLR no-result stack convention for void-returning calls,
with compiler adaptation where a value is needed. The author replied: “Agree then.”

**Action and outcome.** Documented the [chosen split](void-semantics.md). The bounded
Raven bridge now executes Option<int> absence propagation and Result<Void,OverflowError>
completion/error propagation. The compiler fix prevents an unused propagated Void
value remaining on the stack. Refreshing the local SDK/extension was explicitly
requested. Installed local SDK/extension 0.1.12-neoclr.4 and verified target completion,
Option output inference and the saved combined workflow. The [setup instructions](experiments/raven-target/VSCODE.md)
identify the fresh demo and remaining preview limits. No public release was made.

## 2026-09-12 — Existing runtime APIs define the Raven POC scope

**Author's direction.** “We want all libraries that we implemented when experimenting
with Neo to be available in the POC release,” clarified as “all runtime APIs.” The
author then described Raven targeting neoCLR as a good starting point even without
the CLR's full feature support.

**Assistant response and action.** The assistant distinguished runtime-library API
coverage from complete CLR/compiler feature parity, and proposed finishing the file
projection, then auditing the existing runtime against Raven's declarations and
importer. The preview acceptance criteria now require that inventory and closure of
its API gaps. The [file projection](raven-file-api.md) subsequently passed compilation,
import, runtime execution and rejection checks (`e2d3e94`); full library coverage is
not yet achieved. Added a [regenerable source inventory and coverage plan](raven-runtime-api-coverage.md)
for the remaining API work. Neo frontend changes are not part of this migration, and Raven remains an experiment
on its separate branch.

**Open work.** Make each existing public API usable through the Raven target and test
representative behavior, while documenting language/runtime feature limitations and
any contract adaptation. This direction expands the release gate; it does not claim
that today's bounded importer already exposes the library.

## 2026-09-12 — A separate, lightweight Raven distribution

**Author's direction.** The special Raven build shipped with neoCLR should not go
through Raven's full release cycle. Test that it works with neoCLR, package it
separately, and document a repeatable procedure in this repository for later releases.

**Assistant response and action.** The assistant agreed to pin the compiler revision,
package SDK/VSIX assets separately, and validate the neoCLR integration. Added an
[experimental release procedure](experiments/raven-target/RELEASING.md) covering
focused tests, packaged-build smoke checks, provenance and repeat releases. It records
the remaining source-bridge distribution gap explicitly. No package or public release
has been produced as part of documenting this procedure.

## 2026-09-13 — Visible rough edges are part of the POC

**Author's observation.** People evaluating the runtime library will see rough edges:
some APIs resemble .NET, some differ, and some are unfinished or still to be decided.
The author explicitly described that visibility as positive: “That is a good thing.”

**Assistant response.** The assistant agreed that the POC should make the distinction
between familiar behavior, deliberate differences and provisional/missing work visible
so people can evaluate the experiment. It proposed keeping implementation and test
evidence clear while recording this framing.

**Author's clarification.** The author hopes evaluators will give feedback on the
APIs. neoCLR can design its own contracts while retaining familiarity and some code
compatibility. Result-based error handling is an expected compatibility break; the
underlying platform still follows similar principles to .NET.

**Assistant response.** The assistant agreed to distinguish familiar platform
principles from exact API compatibility, and to present provisional contracts as
candidates for feedback rather than settled designs.

**Action and outcome.** Added this framing to the [preview introduction](raven-preview-acceptance.md)
and [API design policy](api-design.md). The runtime-library coverage plan continues
to identify actual gaps, including error-case APIs. The current String slicing work
has passed runtime/propagation checks; this framing does not describe unsettled APIs
as complete or stable.

## 2026-09-13 — Complete the existing runtime API pass before stopping

**Author direction.** “I mean don't stop work until Runtime API is done.” This
clarified the completion boundary: the APIs already developed for neoCLR must be
usable in the Raven proof of concept, following the earlier direction to leave the
Neo frontend out and keep Raven changes on its experimental branch.

**Assistant actions and outcome.** Completed the consecutive API projection and
validation slices, including reflection, native buffers, interfaces, arrays and
nested collection/union payloads. The assistant used CLR-style boxing for copied
value-interface views, documenting its allocation and identity costs; direct value
operations remain available. Internal reserved array capacity tracks unreadable
slots instead of inventing default union cases. These implementation choices are
assistant decisions, not additional author quotations or inferred approvals.

Raven regressions uncovered by the integration included generic metadata fields
and pointers, unsigned array reads, member resolution and value-property setters.
Fixes and focused tests remain on `codex/neoclr-target-resolution`. The
[source-by-source API audit](raven-runtime-api-coverage.md) records the projection
and remaining importer limits. Nine validation suites passed using the packaged
bridge outside both development checkouts. SDK/VSIX `0.1.12-neoclr.6` were installed
locally; the installed server passed protocol checks and the saved demo ran.
[Build provenance](experiments/raven-target/local-toolchain.json) records exact
source revisions, hashes and scope; [VS Code instructions](experiments/raven-target/VSCODE.md)
provide the runnable entry point. No public release was made. Full compiler parity,
general application-type admission and fault-unwind cleanup remain outside this
completed existing-library pass.

## 2026-09-13 — Present the runtime and Raven as the release

**Author direction.** The author specified a runtime preview with neoIL samples,
Raven samples and the special Raven distribution that can target neoCLR, accompanied
by release documentation. The old Neo language material should be separated from the
main experiment. The purpose is to show “that we're actually using a real CLR language
and adapting it for our new CLR-like runtime” and demonstrate the platform changes
working in Raven.

**Assistant response and action.** Organized the release guide around two entry
points: direct runtime execution using neoIL, and Raven compiled against the adapted
runtime library. Added executable neoIL examples for class aliasing/value copying
and Result<Void, Error>, with a repeatable verifier/output check and bundle inclusion.
Marked the Neo guide and plan as historical, retaining their implementation and prior
published notes. The primary README now links the runtime/Raven walkthrough.

**Outcome and remaining work.** Both new neoIL samples verified and ran against the
validated local adapted library. The source packaging recipe now includes them; this
does not retroactively change the existing .6 archive. A fresh candidate, its complete
asset attribution and package validation, and publication remain release work. No
claim of general Raven/CLR binary compatibility accompanies these demonstrations.

## 2026-09-13 — Keep ordinary void-return spelling familiar

**Author question.** “Why does neoil use ‘noresult’?” The author asked whether `void`
was expected. **Assistant response.** The assistant explained that `noresult` exposed
the prototype distinction between an empty return stack and the first-class Void unit
value, and proposed retaining that distinction internally instead of requiring a new
public return keyword.

**Action.** Added lowercase `void` in neoIL return position for the existing no-result
calling convention and updated the new preview samples. `System.Void` explicitly
names the unit type; uppercase `Void` and `noresult` retain legacy behavior. Tests
cover stack preservation, rejection of a stray unit return and generic unit storage.
The binary CLI boundary still uses its existing VOID marker; no new opcode or binary
signature encoding is introduced. See [Void semantics](void-semantics.md).

## 2026-09-13 — A small website for the project

**Author direction.** Create a small website presenting the project clearly, host it
with GitHub Pages, and trigger build/publication in a separate GitHub Action.

**Assistant action.** Built a static landing page centered on the current runtime and
Raven integration, with code excerpts sourced from executable samples, library scope,
preview boundaries and links to documentation. Earlier Neo work is linked as history.
The page distinguishes published previews from the locally validated Raven candidate.
Desktop/mobile layout and navigation were inspected in a browser. A separate Pages
workflow builds on relevant changes and deploys only from main; pull requests do not
receive deployment permissions. GitHub Pages was enabled with GitHub Actions as the publishing source. The
[build and deployment run](https://github.com/marinasundstrom/neoCLR/actions/runs/34745892835)
succeeded for `edae382`; the [live site](https://marinasundstrom.github.io/neoCLR/)
was opened and its HTML/CSS/icon bytes verified against the build. Website publication
does not publish or tag the runtime/Raven release candidate.

## 2026-09-13 — Publish the runtime/Raven preview

**Author direction.** Continue with the release, following the earlier request to
ship the runtime and neoIL samples together with Raven examples, the special Raven
distribution and documentation.

**Assistant actions.** Selected Preview 4 and prepared its versioned source commit
`c135659`. CI exposed a stale release-script assumption about the old stdout return
suffix; the check was corrected to request diagnostic results on stderr, retaining
its environment/clock/file assertions. Rebuilt the runtime distribution, tested all
ten package suites, and waited for the six exact-source CI jobs to pass. Published
Raven's experimental branch so the compiler revision is accessible, without changing
Raven main or making a normal Raven/Marketplace release.

**Outcome.** [Preview 4](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.4)
was published with the runtime, SDK, VSIX, source, notices, checksums and validation
evidence. The website now links the release; the local demo uses the released build.
The [validation record](preview-4-validation.md) distinguishes source platform checks
from macOS arm64 binary support and preserves the POC's limitations.

## 2026-09-13 — Rust dependency and alternative implementation languages

**Author questions.** Asked whether C++ or another implementation language would
be better for neoCLR and remove its Rust dependency, then specifically asked what
would be lost without Rust's memory-management model.

**Assistant assessment.** Distinguished the implementation's memory safety from
the guest heap and GC: changing language does not inherently change neoCLR's
managed type or memory contracts. Rust ownership checks protect implementation
lifetimes; C++ RAII and smart pointers provide resource management but do not supply
the same general borrow checking. C++ is a viable runtime implementation choice,
as CoreCLR's native VM illustrates, with more responsibility for lifetime correctness.
Rust does not prove guest GC rooting or unsafe/FFI code correct. A rewrite has no
demonstrated performance benefit here. Provisionally favor keeping Rust unless a
concrete tooling, integration or maintenance need justifies migration; this is an
assistant recommendation, not an author decision.

**Evidence and action.** Inspected Cargo.toml, the execution architecture and native
memory/interop call sites. `otool -L target/release/neoclr` listed only macOS
libSystem, demonstrating no separate dynamically linked Rust library for that local
binary, not all distribution targets. Build-time Rust remains required. Consulted
the [Rust ownership documentation](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html),
[unsafe documentation](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html),
[linkage reference](https://doc.rust-lang.org/reference/linkage.html), and
[CoreCLR VM build sources](https://github.com/dotnet/runtime/blob/main/src/coreclr/vm/CMakeLists.txt)
on 2026-09-13. These are general comparisons against live documentation/main,
not a pinned implementation study or benchmark. The existing
[architecture](execution-architecture.md) already separates semantics from Rust
representations and leaves a native hosting ABI pending. Only this discussion and
its changelog entry were recorded; no implementation migration was performed.
The desired degree of build-tool independence and any migration decision remain open.

## Maintaining the conversation record

Append significant exchanges with the date on which they are recorded. Capture the
author’s question/direction, the assistant’s response or proposal, any subsequent
author decision/correction, the action taken and outcome, and remaining open questions.
Link evidence where available; explicitly say when an outcome is only reported or
unavailable. A plan is not a performed action. Do not require every exchange to contain
all parts; record what actually happened. Distinguish a recorded date from the date of an earlier
remark. Preserve earlier positions when thinking changes, and link a later entry back
to them. Correct factual transcription mistakes transparently rather than inventing
a cleaner history.

Attribute the author's statements separately from assistant proposals, implementation
outcomes and editorial interpretation. Use direct quotations only when the original
wording is available. Do not infer approval from silence, turn a possibility into a
commitment, or log every routine continuation as a new design decision.

The changelog records changes to the project; this document records the conversations
that direct it. An implementation result is not a substitute for the preceding question,
nor evidence of author approval beyond what was actually said. Only conversation actually available to the
maintainer can be recorded—this is not an automatic capture of conversations elsewhere.


## 2026-09-13 — Application programs and Raven targeting findings

**Author direction.** After Preview 4, the author asked what should follow and
approved three proposed slices: application-defined types, interface dispatch, and
an order-workflow application using the runtime library. The author then emphasized
that integration exposes Raven bugs and should help stabilize the compiler for other
frameworks, citing the existing nanoFramework adaptation as prior experience. The
primary purpose remains making neoCLR work with Raven.

**Assistant response and action.** The assistant distinguished compiler correctness
from configurable target contracts (Iterable/Iterator) and deliberate neoCLR differences
(generic Void, Result). The first application class/value sample now executes through
the bridge, including ArrayList storage. Raven metadata emission exposed a mixed
MetadataLoadContext/TypeBuilder generic signature failure and an incorrect external
self-assembly reference; fixes and focused tests remain on the experimental branch.
See [application types and findings](raven-application-types.md).

**Author clarification.** Generic Void also matters for a future Task<Void> and
async/await story. The assistant recorded that as a design requirement, without
claiming async implementation or choosing a task contract. Interface and workflow
slices remain in progress at this entry; the published Preview 4 artifacts are unchanged.


**Further direction in the same exchange.** The author added delegates and lambdas
to the support goals, and required debugging support for a future release. The
assistant placed delegate/lambda work after the class/interface foundation and
recorded a future Raven debugging milestone in the [roadmap](roadmap.md). The
interface/inheritance sample now executes, revealing another Raven compiler bug:
implicit interface implementation overrode declared abstract/virtual method flags.
The fix remains on the experimental branch. These findings reinforce the author's
distinction between compiler stabilization, target-specific contract mappings and
intentional runtime changes; they do not turn this into a general Raven release.


**Outcome of the initial three slices.** Application types and interface/inheritance
support are committed separately. The [order workflow](raven-order-workflow.md) now
executes with OrderStore dispatch, ArrayList<Order>, Option<Order>, Result<Void,
FileWriteError> propagation and file I/O. Isolated checks confirm that failed writes
preserve both queued order state and the previous report. Delegate/lambda work remains
in progress; debugging remains a future release requirement.


**Delegate/lambda outcome.** The requested follow-up now executes instance method
groups and Raven-generated closure objects, including mutable shared captures and a
callback returned from its creating function. A further compiler issue was found:
interface/abstract method groups used ldftn, while explicit base groups could use
virtual binding. The experimental Raven correction preserves ordinary CLR behavior;
its tests include actual invocation and binding-time null failure. The neoCLR bridge
uses the existing Func family and documents its adapters and limits in
[delegate/lambda support](raven-delegates-lambdas.md). No async or debugger implementation
is implied by this result, and published Preview 4 tools are unchanged.


### 2026-09-13 — Modern library APIs and a replaceable clock

**Author direction.** While the application toolchain was being packaged, the author
asked that future runtime-library design feel modern, including base types and
namespace-level functions, without carrying legacy APIs forward merely for familiarity.
Date/time might instead center on a clock implementation that can be mocked.

**Assistant response.** Proposed separating date/time values from time acquisition,
with an injected system or controlled test clock. Compared the idea with the shipped
.NET TimeProvider/FakeTimeProvider baseline, rather than claiming .NET lacks it.

**Action and outcome.** Recorded the policy in [API design](api-design.md), with
alternatives, placement and future validation in [clock design](local-clock.md#future-clock-contract-review-2026-09-13)
and a backlog entry. No clock redesign was implemented; the current toolchain refresh
continues. Exact contracts and namespace-function projection remain open.


### 2026-09-13 — Pattern ergonomics, extension methods and prototype LINQ

**Author direction.** During the Int32 completion investigation, the author asked for
readable union patterns using case imports and target-member binding. The example
matched File.ReadAllText with `Ok(let text)` and `Error(let error)`, and the author
clarified: “That means supporting destructuring.” Preliminary pattern support was
identified as important for promoting neoCLR.

**Assistant proposal and action.** Prioritized the existing constructor-completion
bug first, then investigated target union metadata, payload deconstruction and generic
case emission. The completion fix was committed on Raven at 55c0f7ef5 and selected in
the local demo. Pattern work remains in progress at this entry; do not infer delivered
support from the proposed syntax alone.

**Further direction.** The author observed that Raven features such as extension
methods can work through existing runtime constructs and library contracts, then
requested prototype LINQ for the next release. The assistant proposed validating
extension methods after patterns and starting with Where/Select on Iterable/Iterator.
The roadmap/backlog record that proposal and unresolved laziness/cleanup semantics;
no LINQ implementation is claimed here.


**Pattern slice outcome (2026-09-13).** The source experiment now uses the existing
member-union and Deconstruct contracts, rather than adding Raven named-case attributes
to the target library. The initial attribute approach was discarded after exposing
incompatible case lookup behavior. The assistant corrected generic case inference,
value deconstruction emission, primitive CLI signatures and scope retention in Raven;
these corrections are committed as `04c953d67` on its experimental branch. The neoCLR bridge provides
payload deconstruction adapters and preserves definite assignment across nested
pattern branches. The order workflow now demonstrates the author's imported-case
syntax, including String and application-object payloads. The executable
[pattern matrix](raven-match-matrix.md) records the tested boundary; LINQ and extension
method validation remain upcoming work. Existing archived builds are unchanged.

The assistant also prepared a separate local source-backed pattern demo with an
updated language server. The previous edited demo was preserved. Validation passed
299 Raven tests, 50 saved-project checks, the match matrix, workflow state checks and
payload completion/hover checks; this did not publish or replace the archived SDK.

**Extension slice outcome (2026-09-13).** Following the author's request to explore
extension methods before prototype LINQ, the assistant reproduced missing generated
marker metadata in the target declaration image. Supplying those metadata-only
dependencies lets nongeneric Raven extensions execute through existing static calls;
no compiler change or new opcode was needed. The sample exercises integer receivers,
shared ArrayList references, Iterable receivers and captured callbacks. Completion
also exposes the extension. Generic application extensions remain rejected by the
bridge. The assistant retained deferred Where/Select as the intended next contract
and identified generic query bindings and iterator representation as prerequisites,
not completed query support. See [the implemented boundary](raven-extension-methods.md).
Archived packages and the author's edited demo were left unchanged.


**Query slice outcome (2026-09-13).** Continuing the author's prototype LINQ direction,
the assistant chose generic runtime-library operators with exact bridge bindings,
rather than expanding generic application-body importing. Where/Select remain deferred;
ToList performs explicit materialization. Their implementation uses ordinary classes,
interfaces and delegates, without new opcodes. Tests exercise callback timing, caching,
repeat enumeration, value/reference payloads and GC-held captures. The Void exercise
found a missing method-argument projection in the bridge and that was corrected;
explicit Func<T, System.Void> works while unconstrained Unit inference remains separate.
A custom Raven Iterable/Cursor probe failed during compiler emission and remains open;
the low-level custom iterator fixture verifies acquisition and disposal directly.
[The API contract](raven-query-api.md) records these outcomes and the retained cleanup
gap. Raven and legacy Neo sources and archived builds were not changed in this slice.


**Custom iterator follow-up (2026-09-13).** The assistant reduced the failure to
serialization of generic interface MethodImpl signatures by the persisted metadata
writer. Raven now normalizes those declarations through its existing target-method
reference path; both ordinary CLR and target emission execute the regression test.
A second bridge issue required closing definition-relative interface signatures
before matching application methods. The former failing repro is now a passing
[custom Iterable sample](experiments/raven-target/samples/application-iterable.rvn).
This resolves the previously recorded compiler gap without new runtime behavior.


## 2026-09-13 — array queries need an array contract

The author reported a reflection-array `for` loop rejected by the importer (`bge`)
and then `GetMethods().ToList()` missing from editor lookup. They directed that the
fixes belong in the next release. The assistant first fixed branch admission and
proposed array-specific query overloads backed by a sequence adapter.

The author clarified the intended .NET-like Iterable/Iterator pattern, correcting
an initial reference to ArrayList with: “ArrayList already does that. I mean Array”.
They added: “Anyway, we should make sure that the same pattern exists in neoCLR.”

The assistant changed the implementation to a runtime array-to-Iterable view with
independent library iterators and opt-in Raven target-symbol support, removing the
query-only overload approach. The development query tests cover arrays in interface
locals, parameters and returns, independent positions, mutation visibility and GC;
the runtime tests check reference identity and invalid/null dispatch. See the
[array/query contract](raven-query-api.md). Preview 5 remains frozen; this work is for
a subsequent release, not a claim that the reported programs work in its artifacts.


The author then reiterated the compatibility goal: Raven should need only minimal
changes, such as selecting different target interfaces, while the runtime supplies
the behavior required by ordinary emitted IL. The assistant agreed to keep contract
selection target-neutral and to investigate missing runtime support before adding
NeoCLR-specific compiler lowering. The array implementation follows that split:
Raven projects the configured interface; neoCLR owns reference identity and dispatch.


## 2026-09-13 — Stabilizing ordinary Raven programs

**Author:** “Let's stabilize neoCLR.” The preceding direction was to keep Raven
changes minimal and let neoCLR implement the contracts ordinary CLR features need.

**Assistant proposal and action:** Focus on correctness rather than new features.
A basic array query with `value > 1` reproduced an importer rejection of `cgt`,
although the runtime already implemented it. Admitted the existing numeric comparison
and branch families with operand/control-flow validation. Boundary tests then exposed
Raven's signed emission for unsigned comparisons and unordered floating comparison
errors; corrected those in the existing Raven experiment branch, without adding a
neoCLR-specific compiler workaround.

**Outcome and limits:** The updated query regression suite passes, including integer
boundaries and NaN. The .NET execution test also passes. A wider Raven test run found
an independent mixed `ulong`/`long` binding failure, reproduced with unchanged compiler
code; it remains open. These are next-build fixes, not a new published release or a
claim that the entire platform is stable. See the [comparison contract](raven-primitive-api.md#numeric-comparisons-after-preview-5).

## 2026-09-13 — Mutable array invariance and read-only views

**Author:** Asked to resolve array co-/contravariance in neoCLR. The assistant initially
proposed CLR-style array covariance with checked writes. The author corrected that
proposal: a typed mutable array, including Foo[], should not become an array of a
supertype such as Object[]. The author then identified a possible exception for a
read-only array or a read-only interface, while leaving its mechanism open.

**Author's reasoning:** Described array covariance as a historically useful feature
from before generics, but unsuitable for the intended modern type system. This is the
author's evaluation, not a newly established history of the CLR design.

**Assistant response and action:** Accepted invariant mutable arrays as the direction.
Distinguished a read-only element interface from an immutable binding or readonly
reference to the array slot. Added early cast rejection and regression coverage on
top of existing exact array storage rules. Documented both the compatibility cost
and the future read-only projection in the [array variance contract](array-variance.md).

**Open:** Covariant read-only projection API, generic variance metadata and target-aware
Raven editor diagnostics. No new read-only array feature or generic variance behavior
was implemented in this slice.

## 2026-09-13 — Preserve CLR ergonomics while selecting improvements

**Author's direction:** Define what neoCLR can improve when starting without legacy
constraints, while preserving useful .NET foundations. The value/reference type split
still serves the project and should not be the current redesign focus. Spans and
managed references may warrant more consistent runtime support.

**Author's additions:** Reaffirmed UTF-8 strings and requested modern string, character
and encoding APIs. Asked to revisit nullability and its metadata, while acknowledging
that another solution might not be better. Requested an API scope that can be implemented
incrementally as consumers need it. Emphasized preserving C# and other languages'
ergonomics on the platform.

**Further questions:** Asked whether delegates might become function types, without
choosing that change. Wants runtime support for async/await; its model remains open,
and tasks still seem relevant. Identified the implemented array-covariance change as
something to observe for type-system consistency without losing ergonomics. Reaffirmed
building date/time toward instantiable, mockable environmental APIs.

**Assistant proposals:** Keep familiar type categories and compiler boundaries.
Separate public API shape from runtime representation: language function types need
not replace delegates, and tasks need not dictate suspension internals. Compare .NET's
existing span, Unicode, nullability, delegate, Task and TimeProvider contracts before
claiming an improvement. Develop APIs by concrete scenarios; distinguish representation,
metadata enforcement, compiler diagnostics and library policy. Treat all unselected
representations as research questions.

**Actions and outcome:** Added the [platform direction](platform-direction.md) and
[runtime API plan](runtime-api-plan.md); updated active policy summaries that still
prescribed value-by-default. Extended the text, nullability, delegate and date/time
research notes and linked the roadmap. This slice changes documentation only. No new
Span, Rune/codec, nullable signature, callable representation, Task or injectable-clock
implementation is claimed. Proposed slice ordering remains an assistant recommendation;
no reply selecting those detailed designs is available in this exchange.

**Follow-up on research scope:** The author requested reviews of other platforms,
comments on .NET runtime APIs, and independent projects that seek to improve the
.NET experience. The assistant proposed retaining .NET as the compatibility baseline
while examining concrete feedback, alternative designs and counterevidence. Updated
[design research](design-research.md#broader-api-review-scope-2026-09-13) to require
relevant comparisons for substantive API design, with attributed opinions, source
status and transfer costs. Added Noda Time, Rust text documentation and .NET API-review
process links as initial entry points. No full comparative review or adoption of
those projects is claimed; this establishes the workflow for subsequent API slices.

**Author's qualification:** Not everything warrants change. Larger platform decisions
need input and grounding, while some ideas can be prototyped immediately. The assistant
updated the research policy to scale evidence to consequences, keep prototypes
provisional and include retaining current behavior as an explicit alternative. This
is a decision-quality requirement, not a new permission or external-feedback gate.


## 2026-09-13 — A bounded view before a collection hierarchy

**Author:** Asked to continue the proposed read-only view/variance investigation,
then cautioned that .NET's accumulated collection interfaces should be reviewed later.
Questioned the need for readonly, immutable and frozen contracts. Suggested Iterable,
counted Collection, indexed/ordered Sequence, Map and Set with separate Mutable*
contracts as an illustration, explicitly not a selected design.

**Assistant response and action:** Kept the prototype application-local. Audited the
existing interfaces and found variance metadata/dispatch prerequisites and a legacy
Iterable byref-return mismatch with the Raven projection. Added an explicit Dog-array
adapter returning Animal through a getter-only interface, a C# comparison and direct
IL checks. Reviewed .NET variance and collection discussions, Java backed views and
alternative immutable .NET collections as different contracts.

**Outcome and limits:** The prototype demonstrates live aliases, shared element
identity, private backing storage and GC retention without copying the array. It
allocates an adapter and does not implement implicit covariance. No new standard
collection interface, general variance or readonly/immutable/frozen type was selected.
See the [experiment and validation](experiments/readonly-views/README.md). The future
collection hierarchy and variance representation remain open.

**Subsequent supplied proposal:** The author shared a ChatGPT hierarchy with covariant
Iterable/Collection/List/Set, Map<K,out V>, invariant Mutable* extensions, and concrete
Array/ArrayList/HashSet/HashMap plus Immutable* and Frozen* families; an earlier list
also included LinkedList. Recorded both proposals in the
[collection-contract review](collection-contracts.md). The assistant identified Set
membership inputs, Map's Option/byref lookup results, fixed-size array mutation and
linked-list indexing costs as questions to resolve before selecting that hierarchy.
No author decision adopting these particular interfaces is inferred.

**Explicit array decision:** The author then stated, “System.Array should be generic
in NeoCLR. System.Array<T>”. Recorded this as the selected generic array direction,
while keeping the rest of the hierarchy provisional. The assistant proposed preserving
ordinary T[] syntax and CLI array signatures/instructions, with no wrapper allocation,
and identified type identity, metadata, reflection, member lookup and fixed-size mutation
as mapping work still to perform. This slice does not implement the generic Array API.

**Variance follow-up:** The author observed that generic System.Array allows variance
to be declared on the type. The assistant clarified the proposed rule as invariant T
for mutable arrays, with covariance potentially on read interfaces. A generic declaration
still needs to agree with runtime array signatures, casts and element access; it does
not itself make mutable covariance safe or complete the missing generic-variance work.

**Shape clarification:** The author confirmed unchanged language treatment and
emphasized “The shape is known”. The assistant recorded the purpose as a known generic
array definition for the runtime and tools, linking element type, members, interfaces
and invariance to existing array signatures/instructions. This refines the intended
mapping; it does not claim that mapping has been implemented.

**API priority direction:** The author placed the generic array API, collection
interfaces and basic implementations on the immediate runtime-API priority list.
A complete library is unnecessary, but these fundamental building blocks should at
least have usable prototypes. The assistant updated the API plan and roadmap to put
generic array mapping first, then minimal contracts and alignment of arrays/ArrayList,
followed by bounded keyed collections. The text-first suggestion was superseded.
The detailed slice boundaries are the assistant's proposed execution plan, not an
adoption of every previously supplied collection type. No new API was implemented
in this documentation update.

## 2026-09-13 — Normalize array and native-memory roles with .NET

**Author:** Asked to continue with Array<T>. On learning that the current generic
System.Array is a native descriptor rather than the managed array object, observed
that this question has changed since returning to .NET-style type semantics and
said, “We should normalize with .NET”.

**Assistant finding and response:** Audited the native descriptor, managed ArrayRef
representation, reflection and Raven bridge. Initially proposed moving the native
API to an explicit interop name, then clarified that renaming alone would leave
the descriptor's mixed ownership and borrowing responsibilities intact. Proposed
the .NET separation of managed arrays, explicit native allocation/release, borrowed
memory views and distinct ownership when needed. Recorded the comparison and
migration order in [collection contracts](collection-contracts.md#managed-arrays-versus-native-buffers).

**Outcome and open work:** The ownership direction is recorded; the Array<T> mapping
and native API migration are not yet implemented. The prior generic-array identity
and invariance decisions are retained, rather than interpreting the latest direction
as an unspoken reversal. Exact native replacement APIs, safe borrowed views and
their lifetime enforcement remain implementation/design work. No new standard
NativeBuffer type was adopted.

## 2026-09-13 — Implement the array shape without a native compatibility alias

**Author:** Asked to continue, then clarified that compatibility with old neoCLR
iterations is unnecessary. Asked whether Array<T> should implement interfaces.
Explained that direct array loops should remain indexed, while interface support
allows arrays to be passed as iterables and used with extension methods.

**Assistant response and actions:** Implemented a generic managed array shape over
the existing array identity in the Raven profile. Initially staged a temporary
LegacyNativeArray rename; after the author's correction, removed that path and
replaced the native descriptor API with a bounded NativeMemory allocation/free
API. No legacy alias was committed. Added an explicit Iterable<T> declaration,
kept the current growable List contract off fixed-size arrays, and added a sample
covering indexed loops, an Iterable parameter and the ToList extension.

**Compiler finding:** Native method emission exposed a missing pointer case in
Raven's metadata-reference rewriter. Fixed it with a metadata regression on the
existing Raven experiment branch. Native-sized numeric and explicit pointer
conversions remain separate compiler gaps. The target adapter also accounts for
Raven's empty Unit pointer locals when importing native void-pointer calls.

**Outcome:** See [generic managed arrays](generic-managed-arrays.md) for implemented
behavior, migration and validation. The source profile changes do not refresh the
installed SDK/extension. The broader collection hierarchy, read interfaces, Span
and native owner contracts remain open. Historical Neo work remains outside this
Raven-profile migration, as previously directed.

**Consistency observation:** The author described the generic array signature as
a more consistent solution for neoCLR while retaining .NET familiarity, then
corrected a spoken project-name transcription to neoCLR. The assistant agreed
about a shared generic definition for members/interfaces/reflection, while noting
that ordinary CLI array signatures already encode their element type. This is a
regularization of the array contract, not a claim that CLR arrays were untyped.


### 2026-09-13 — Reading array contracts from target metadata

**Author direction:** Continued the Array<T> work after emphasizing that arrays
must participate in Iterable and extension methods while direct loops remain indexed.

**Assistant proposal and action:** Replaced the experiment's Boolean array-interface
projection with an optional generic shape mapping in Raven. The core reference now
declares System.Array<T> implementing Iterable<T>; Raven substitutes the element type
and reads the declared interface closure. The runtime support came from the preceding
slice. This keeps future interface additions primarily in target metadata rather than
adding a compiler rule for each contract. The normal .NET target remains unchanged.

**Boundary:** This does not adopt a broader collection hierarchy or make explicit
System.Array<T> source annotations interchangeable with T[]. Installed tools are not
refreshed by this source slice.

**Validation outcome:** The metadata projection passes 25 focused Raven tests,
58 saved-project cases and 28 query checks, including array extensions. The interface
probe checks the reference declaration. These are source-level integration checks;
a packaged SDK/editor refresh remains separate.


### 2026-09-13 — Unifying the array spellings and refreshing the tools

**Author direction:** Accepted the proposed consecutive slices: make Array<T> and
T[] interchangeable in Raven, verify the full array experience, then refresh the
local SDK and VS Code extension.

**Assistant action:** Implemented target-specific canonicalization in source type
binding and imported signatures. Both spellings use ordinary CLI vector signatures;
interface member calls use normal interface dispatch. The new sample demonstrates
alias mutation, nested arrays, indexed loops, Iterable parameters, GetIterator,
query extensions and reflection. Compiler regressions cover opt-in behavior and
invariant arrays. Allocation remains an array expression rather than a new generic
class constructor. Broader collection taxonomy and arbitrary class-member projection
remain separate. Local packaging/installation is the following slice.

**Source validation:** The new sample passed on neoCLR; the saved-project suite
passed 59 cases, and the new SDK server passed 52 completion/hover/diagnostic checks.
Raven's compiler slice is committed as `6072dcf4b` on its experiment branch.

**Local tool outcome:** Built and installed SDK/VSIX `0.1.12-neoclr.11` in a new
isolated profile with the unified-array demo. Seven packaged suites passed; the
installed VSIX server and exact configured Run task also passed. Preserved 1,074
existing Raven source files. [Local instructions](raven-generic-arrays-local-build.md)
and the linked manifest record artifact hashes and evidence. This completes the
authorized local refresh without publishing a release or replacing prior demos.


### 2026-09-13 — Collection contracts advertise supported operations

**Author direction and question:** Asked to continue the collection review/prototype,
then asked what the assistant would do. The assistant proposed Iterable, Collection,
Sequence, MutableSequence and the existing growable List as a minimal capability chain,
with invariant type parameters and provisional naming.

**Author response:** Observed that this is preferable to a mutable default followed
by a view that rejects Add. The assistant clarified that arrays should expose element
replacement without claiming growth; read interfaces restrict access but do not make
shared storage immutable or prevent mutation through other aliases.

**Action underway:** Recorded the [capability prototype](collection-contracts.md#2026-09-13-capability-prototype)
and compared it with .NET's array/read-wrapper contracts and the proposed read/mutable
interface inheritance change. Implementation exposed a Raven inherited-indexer lookup
bug, which is being addressed on the experiment branch rather than worked around in
samples. No implementation completion or tool refresh is claimed by this planning entry.


**Subsequent direction:** The author asked to focus next on Map and a default
implementation corresponding to .NET Dictionary. The assistant queued Map<K,V>
plus a dictionary-style implementation, proposed evaluating HashMap as the concrete
name, and identified equality/hashing, comparer ownership, duplicate-key behavior,
Option/Result lookup and read/mutation separation as decisions for that slice.


**Capability outcome:** Implemented the experimental hierarchy in the Raven profile
and used normal array member dispatch for Count/indexers. ArrayList inherits the
same read/replacement contracts through List. The generic array shape no longer
hardcodes a single interface, but still forbids incompatible record storage; ordinary
conformance validation rejects unimplemented operations. Validation passed 39 runtime
tests, 22 Raven tests, 60 saved projects, 28 query checks, six capability rejections
and 56 editor checks. The installed .11 tools remain unchanged; Map is the next
planned slice.


**Separate follow-up:** The author asked to consider Option or Result for LINQ
terminal methods such as First and explicitly said to keep the work in separate
slices. The assistant recorded collection capabilities, Map/dictionary work and
terminal-operator outcome design as separate steps. First returning Option is a
proposal; Single cardinality semantics and other operator outcomes remain to decide.
No terminal operators were added to the current capability implementation.

### 2026-09-13 — A first Map implementation, with comparer policy still open

**Author direction.** After separating collection read/replacement/growth capabilities,
the author asked to focus on a Map interface and a default implementation corresponding
to .NET Dictionary. The author also raised Option/Result outcomes for LINQ terminals
and explicitly said, “Keep this as separate slices.” The subsequent “Continue” starts
the Map work; it does not settle every proposed collection name or comparer policy.

**Assistant proposal.** Implement invariant Map<K,V>, MutableMap<K,V> and HashMap<K,V>
using existing managed storage and IL. Because the current library has typed equality
but no general hash-code/default-comparer contract, start with explicit equality and
hash callbacks. Use Option for lookup, Boolean TryAdd for duplicate rejection, Set
for insert/replace, and a snapshot of keys. The assistant identified the convenience
cost of mandatory callbacks and left default comparers, null-key policy, removal and
pair enumeration open. These are assistant-selected prototype boundaries, not a
reported author endorsement of the final API.

**Action and outcome.** Added the [IL implementation](../runtime/raven/Map.neoil),
metadata/importer adapters, a [Raven order-lookup sample](experiments/raven-target/samples/library-maps.rvn)
and [contract/research notes](map-contracts.md). The table stores collision chains in
managed arrays/ArrayLists and grows using saved hashes. The sample demonstrates
Option destructuring, reference keys and shared reference values. No Raven compiler
or runtime opcode change was needed. Direct IL checks cover collection pressure,
reference retention, collisions, snapshots, invalid interface operations and callback
reentrancy. The existing test fixture was also corrected to use the migrated indexer
owner and the target library when assembling.

Validation reported by the assistant: 13 collection/runtime tests, 61 saved-project
cases, 81 signature checks, 10 capability rejection cases and 59 editor checks passed.
The .NET comparison confirms the shared duplicate/replacement behavior and illustrates
its live key view and null-key rejection. neoCLR's key snapshot and current
callback-defined key domain are documented differences, not asserted improvements.
The installed .11 SDK/extension was not changed. LINQ terminal implementation remains
a separate slice, and this bounded Map is not described as a complete Dictionary API.

### 2026-09-13 — LINQ terminal outcomes and concrete collection guidance

**Context and assistant proposal.** Following the author's request to keep Map and
LINQ terminal outcomes in separate slices, the assistant proposed First/Last returning
Option<T>, and Single returning Result<T,SingleError> with Empty and Multiple cases.
Filtered selection would compose through Where; all normal outcomes would dispose
the acquired iterator. Runtime faults would remain faults, without implying fault
unwinding or introducing exception classes. These concrete contracts are an
assistant-selected prototype interpretation of the author's Option/Result direction.

**Author's next slice.** During implementation, the author asked for built-in filter
functions on ArrayList with appropriate Option/Result usage in an upcoming slice.
The assistant recorded a review of existing Find, FindIndex and Exists alongside
additional useful operations. The plan distinguishes absence, empty filtered results,
Boolean questions and actual recoverable errors rather than wrapping every return.
No ArrayList API or implementation was changed in this terminal slice.

**Guideline and correction.** The author said to prefer suitable built-in functions
when the concrete collection is available and use LINQ when querying through an
interface. The assistant initially qualified a blanket speed claim. The author
clarified: “It's not that they are faster necessarily,” explaining fewer allocations
from direct data-structure access instead of a chain of objects. The author then
explicitly allowed LINQ specialization and reiterated the general preference for
built-in operations. The recorded guideline now reflects allocation efficiency,
permits specialization, and does not claim universal faster execution.

**Action and evidence.** Added the terminal library methods, SingleError value union,
metadata adapters and a [Raven sample](experiments/raven-target/samples/library-query-terminals.rvn)
covering pattern matching and propagation. Direct IL tests check cardinality,
short-circuiting, disposal and fault boundaries. A measured equivalent lookup used
five managed allocations for ArrayList.Find versus nine for Where(...).First(),
including common setup. The assistant identified that the existing Find still creates
an ArrayIterator: direct-storage scanning is upcoming work, not a completed change.
Those counts measure this interpreter implementation, not host bytes or elapsed speed.
See the [query API](raven-query-api.md) and [upcoming API plan](runtime-api-plan.md).
The installed SDK/extension remains unchanged; no Raven compiler or opcode change
was needed for the terminal implementation.

**Validation outcome.** The assistant subsequently verified 62 saved-project cases,
29 query checks, 90 signature checks, 59 editor checks and 18 focused runtime tests,
plus Clippy, formatting and the API audit. A .NET 10 target comparison confirms the
existing default-sentinel ambiguity and exception-based cardinality behavior. The
ArrayList work remains planned separately; LINQ specialization remains permitted.

## 2026-09-13 — Concrete ArrayList filtering after the LINQ terminals

**Author's direction.** The author requested built-in ArrayList filtering as a
separate upcoming slice, with Option and Result used appropriately, then directed
continuation. The preceding discussion clarified the motivation: avoid unnecessary
query-chain allocations when a suitable concrete operation exists, while retaining
LINQ composition and possible specialization. It was not a universal speed claim.

**Assistant's selected contract.** Implement the familiar predicate method family:
Find/FindLast return Option<T>, FindIndex/FindLastIndex return Option<Int32>,
Exists/TrueForAll return Boolean, and FindAll returns a new shallow list. Changing
FindIndex from -1 absence to Option is the assistant's concrete interpretation of
the requested outcome review, not a separately quoted author decision. Empty
filtered lists and Boolean questions need no Result wrapper; callback faults remain
runtime faults. Range/removal and fallible-predicate overloads remain open.

**Action and evidence.** Added direct IL scans over the initial backing buffer and
extent, retaining the existing search policy under callback mutation. Updated Raven
metadata, samples, completion expectations and migration documentation. The
[filtering contract](arraylist-filtering.md) compares .NET's method family and index
sentinel with the selected Option contract. Scalar searches no longer create an
iterator. The equivalent Find lookup now measures four managed allocations versus
nine for Where(...).First(), including setup, compared with the earlier five versus
nine observation. This measures interpreter heap objects, not speed or host memory.

**Validation outcome.** Four direct filtering tests passed, including callbacks that
grow the source during GC. Source validation passed 63 saved-project cases, 12
application cases, 104 signature checks and 59 editor checks, plus Clippy and
formatting. A Raven sample initially used a qualified None pattern that did not
satisfy exhaustiveness checking; importing Option cases and destructuring Some
fixed the sample without a compiler change. The .NET comparison ran targeting
net10.0. No new SDK/extension was installed and no release was published in this
slice. Historical Neo behavior remains separate and unchanged.

## 2026-09-13 — Testing the collection slices together

**Context and proposal.** After the ArrayList filtering commit, the author directed
continuation. The assistant selected an integration slice: combine the new Map,
filtering and LINQ terminal APIs in an order application before expanding the API.
This follows the author's earlier request to evaluate the platform through realistic
programs and allocation-aware concrete collection use; the particular scenario is
an assistant proposal, not a newly specified author requirement.

**Action.** Added an Order class, duplicate-preserving registration, Option lookup
and propagation, direct FindAll filtering, array/interface query composition and
Result-based Single handling. The sample demonstrates that filtered membership is
fixed at creation while class instances remain shared with the map and source list.
No runtime or Raven compiler change was required. The documented current limits
include explicit hash/equality callbacks, no transactional registration and no
concurrency guarantee.

**Outcome and evidence.** The [collection workflow](raven-order-workflow.md#collection-integration-scenario-2026-09-13-source-slice)
runs unchanged under colliding hashes and with additional garbage-collection pressure.
Validation passed 15 application checks and 61 editor checks, including inferred
Order members through map lookup and filtered-list access. The assistant initially
wrote a None construction without the required constructor invocation; correcting
the sample to None() resolved the import failure. This does not establish improved
compiler diagnostics for that invalid expression. The source sample and release
procedure are ready for inclusion by the existing bundle builder; no package,
installation or publication occurred in this slice.

## 2026-09-13 — Predicate terminals and the next library-authoring direction

**Author's request and action.** The author asked for First and related overloads
that accept a function predicate. The assistant added First/Last/Single overloads,
retaining Option/Result outcomes, normal-outcome iterator disposal and terminal
fault boundaries. Direct loops avoid a Where wrapper. Verification required control
flow that establishes a first match before reading the saved generic value; the
implementation was corrected to express that without a default T or verifier change.

**Validation outcome.** The source changes passed 63 saved-project cases, 29 query
checks, 15 application checks, 116 signature checks and 64 editor checks, including
predicate signature help. Five runtime tests cover both overload families, matching
order, cardinality, cleanup, faults and allocation comparisons. Clippy passed.
The .NET comparison shows that its List Last(predicate) can traverse backwards,
whereas this prototype keeps forward query order. Installed .12 tools remain the
previous validated build, without these latest source additions.

**Author's proposal and decision.** While this work was underway, the author proposed
writing the runtime class library in Raven, with neoIL still preferred where useful,
to help develop neoCLR itself. The assistant supported a high-level implementation
path and proposed checking bootstrap and generic-import gaps first. The author then
clarified that neoIL remains important but that, now a high-level language targets
neoCLR, the class library should use it. The author requested a release or at least
a tag before proceeding, and emphasized that migration should start now because
library growth would make the translation larger and harder.

**Assistant's response and next action.** Use Raven as the default authoring language
for ordinary library code; retain neoIL for justified low-level, bootstrap and test
uses. Finish the predicate slice and take a source milestone tag rather than claim
that the older .12 binaries contain it. The [library direction](runtime-library.md#raven-as-the-library-source-language-2026-09-13)
records stable library identity, bootstrap metadata and generic-body importing as
migration groundwork. The current bridge admits known generic API calls but rejects
generic application bodies; the migration is not yet implemented by this checkpoint.

## 2026-09-14 — Pause library migration and evaluate Raven target support

- **Author:** Asked whether the Raven bridge should be evaluated before authoring
  System classes in Raven, then directed a pause. Prioritize reviewing Raven changes
  and integrating suitable bug fixes, followed by consistent target mapping rather
  than an attached bridge that compensates for compiler behavior.
- **Author:** Clarified that stable importing is crucial, but Raven itself should
  handle .NET and neoCLR targets. Earlier comments distinguished namespace functions
  from their CLI container and rejected NotImplementedException as a neoCLR API.
- **Assistant:** Proposed separating general compiler fixes, reusable target
  configuration and temporary neoCLR adapters. Retain shared CIL emission where it
  fits; the importer should validate and resolve the emitted contract.
- **Actions:** Paused and locally stashed the scalar Math migration, inventoried the
  37 Raven experiment commits and inspected metadata, iteration, propagation and
  application-import boundaries. Recorded [the assessment](raven-target-evaluation.md)
  with review batches, target-layer responsibilities and migration exit criteria.
- **Finding:** Reproduced the author's string-filter screenshot in a disposable
  project. Adding missing String equality operator metadata made the query run
  without Raven changes; collection-interface conversion was not the failing step.
- **Outcome/open work:** Migration is paused. No Raven fixes have been merged by this
  review; integration requires dependency review and tests on proposed branches.
  Namespace-function identity projection and removing the metadata-only exception
  placeholder remain explicit follow-up work. Installed demo files were preserved.

- **Follow-through after the author's “Continue through those issues”:** Extracted
  general Raven fixes into two reviewed batches on `codex/compiler-fixes-integration`,
  with before/after regressions and a passing 5,489-test baseline. Pushed the review
  branch; Raven main was not merged. Kept target-contract changes on a separate
  experimental branch.
- **Assistant's implementation and finding:** Made emission-core selection a project
  compiler option and separated compilation from neoCLR artifact import. Testing the
  ordinary driver exposed host-reference injection and host-derived defaults; these
  were corrected for explicitly configured metadata targets. The .NET 10/11 matrix,
  targeted compiler tests, five independent compile/import checks and 63 saved-project
  checks passed. [The workflow](raven-target-compilation.md) records how to reproduce it.
- **Remaining direction:** This is an incremental target-support improvement, not a
  completed target pack or general library importer. Migration stays paused while
  stable identities, generic bodies and temporary adapters are addressed. No installed
  SDK, extension or user demo was changed by this follow-through.


## 2026-09-14 — Integrate general Raven fixes into main

- **Author's directive:** “Keep the tracks separate”: experimental neoCLR support
  belongs on its own Raven branch, while general fixes should be integrated into
  Raven main. The author reinforced that anything benefiting Raven generally belongs
  on main.
- **Assistant's response:** Extract fixes by behavior and dependencies, validate with
  ordinary .NET inputs, and advance main without merging the experiment wholesale.
  A fix is not permanently experimental merely because neoCLR exposed it.
- **Action:** Reused the isolated main-based numeric/binding integration branch and
  extracted the namespace metadata/completion fixes. Reworked the namespace regression
  to use default compilation options and normal framework references. It reproduced
  the completion failure without neoCLR settings, then passed after the fix.
- **Workflow:** Recorded the separation in AGENTS.md. Further general cross-target
  emission candidates remain explicitly queued for dependency review; intentional
  Void/array/protocol differences remain on the experimental track. The System-library
  migration stays paused during this integration work.
- **Outcome:** The 47 focused namespace tests and full 5,490-test baseline passed
  without failures/skips. Fast-forwarded and pushed Raven main to `8fa59a967`, containing
  the reviewed general numeric, binding/dispatch and namespace fixes. Experimental
  target policies were not merged into main. The remaining emission candidates stay
  queued for independent review. No installed SDK or extension was refreshed.
- **Track synchronization:** Merged Raven main into `codex/neoclr-namespace-metadata`
  at `5d1022ced`, preserving the experiment's Void regression and main's independent
  namespace test. Compiler source was unchanged by this merge; 50 focused tests passed.
  The experimental branch now contains main as an ancestor. This merge direction does
  not bring experimental policy onto main.

## Release stabilization before library migration — 2026-09-14

**Author direction:** “Our goal is to fix these things before we make the next
release. After that we can worry about migrating code from neoil to Raven.”
This follows the directive to integrate general compiler fixes into Raven main
while retaining experimental neoCLR target behavior on a separate branch.

**Assistant response and action:** Continued independent review of closed-generic
metadata emission, using ordinary .NET reference assemblies and Raven's existing
EmitOptions contract. The adapted regression reproduces an emission failure on
main. The fix was subsequently integrated into Raven main at `031b9aaaa`: 18 focused
checks and the .NET 10/.NET 11 build/run matrix passed. Remaining fixes and release
validation are still pending; this entry does not report a release.

**Outcome/direction:** Stabilizing the compiler/importer integration is a prerequisite
for the next release. Authoring the runtime library in Raven is deferred until after
that release; the current neoIL library remains the implementation during this work.

## General Raven fixes across frameworks — 2026-09-14

**Author clarification:** “Everything that's benefiting Raven, even when targeting
the .NET Framework or the Nano Framework, should be on main. The experimental
integration with neoCLR will stay in its experimental branch.”

**Assistant response/action:** Confirmed that target-independent compiler and
metadata fixes belong on Raven main, updated repository instructions, and continued
independent review of generic-field emission. The boundary is general utility versus
neoCLR-specific policy, not whether a defect was discovered through this experiment.

**Validation boundary:** A .NET 10/.NET 11 matrix result is not evidence of executing
on .NET Framework or NanoFramework. Those targets remain relevant to design/review;
actual target validation must be reported separately when performed.

**Implementation outcome:** Integrated the generic-field fix into Raven main at
`ea6f3383b`; 22 focused checks and the modern .NET build/run matrix passed. The
completed feature branch was removed, preserving the active neoCLR experiment.

## A configurable target model, not scattered special cases — 2026-09-14

**Author direction:** Integrating neoCLR as a Raven target should require very little
special-case compiler handling. The author cited selecting `Iterable<T>` instead of
`IEnumerable<T>` as an example of a manageable framework-contract mapping.

**Assistant proposal:** Put reusable target configuration, compiler correctness and
metadata handling on Raven main. Keep neoCLR's experimental configuration and
unresolved semantic differences on its experimental branch. Classify each mismatch
as a compiler bug, a configurable framework contract or a genuine runtime difference.

**Action/outcome:** Continued independent metadata-emission fixes and recorded this
as the review criterion. This is not a claim that all existing neoCLR special cases
have been removed or that a finalized target-configuration API has been integrated.

**Subsequent author clarification:** Nothing specific to neoCLR should enter Raven
main yet, including tests; those can be considered when the integration is ready.
General fixes can enter now when useful across CLI-metadata-compatible frameworks,
even when their library API surfaces differ. The author raised a possible future
emission/backend rethink, explicitly as an open question beyond this work.

**Assistant response:** Keep current integration evidence independent of neoCLR,
record the narrower boundary in repository instructions, and treat backend changes
as future evaluation rather than part of stabilization. Current imported regression
fixtures use ordinary .NET references and no neoCLR-specific options or names.

**Further author clarification:** NeoCLR should currently remain sufficiently close
to the .NET Framework and other CLI derivatives that few special differences are
needed. More significant differences may arise later, with runtime nullability as
an example. The author clarified that such differences could affect Raven's
**semantic model**, not merely metadata mapping.

**Assistant response:** Distinguish configurable API names from differences in type
identity, conversions, flow analysis, diagnostics and emitted metadata. Continue
compatibility fixes against the shared CLI model now; keep runtime nullability and
its semantic-model consequences as a separate, unresolved design question.

**Immediate author priority:** Stabilize and commence a release so there is a stable
new preview to demonstrate. The assistant proposed finishing the remaining
compatibility fixes, validating demos and packaged tools, and preparing the release;
library migration and new runtime semantics remain outside that release work.
This records the intended sequence, not a completed release gate.

## Raven main stability audit — 2026-09-14

**Author concern:** Raven main should remain stable and integrated changes should not
break its samples; the work should demonstrably improve the compiler.

**Assistant action:** Set aside the pending delegate fix and audited exact main
`ea6f3383b`. The baseline passed 5,493 tests and the standalone corpus built/ran
173/172 samples successfully. The project build exposed four NanoFramework failures
in custom-attribute emission plus a separate MacCatalyst/Xcode prerequisite failure.
The earlier narrow checks had missed that compiler regression.

**Correction/outcome:** Added primitive and enum attribute regressions, both failing
before correction, and separated runtime-type attribute serialization from target
metadata signature resolution. Integrated the correction into Raven main at
`5a67d5d4c`: 39 focused tests, all four NanoFramework builds and the modern .NET
build/run matrix passed. All 38 eligible project executables also passed; the 13
build-only and 20 non-executable classifications stayed explicit. No sample source
or exclusion was changed. MacCatalyst still needs Xcode 26.6; this host has 26.2,
so the entire project-build gate is not claimed green. NanoFramework build success
is not hardware execution. The pending delegate fix was then restored for combined
validation. See Raven's `docs/compiler/main-stability-audit.md` for the audit scope.

**Follow-through:** The restored general delegate fix passed 55 combined focused
checks and was integrated into Raven main at `35a9df494`. Temporary audit/integration
branches were removed. Broader baseline and sample results retain their original
commit scope; release validation remains outstanding.

## Publishing the stabilized preview — 2026-09-14

**Author direction:** “Then its perhaps time for a new preview release of neoCLR”.
This followed the requirement to keep Raven main stable, integrate general compiler
fixes there, and retain neoCLR-specific integration on the experimental branch.

**Assistant action:** Synchronized the experimental Raven branch with the reviewed
main fixes, built a fresh SDK/VSIX and prepared Preview 6. Release validation found
Rust syntax newer than the declared minimum, preview samples being checked against
the historical library, and additional extension dependency notices. Corrected those
issues and reran validation against the selected source and packaged artifacts.

**Outcome:** Published [Preview 6](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.6)
from neoCLR `5c52b4bce2da2ef31f279d0f5f3fa924b82baf61`, with experimental Raven
`0.1.12-neoclr.13` from `246d697bf6c69ff8cc56ca4859879edd7c081e7d` on
`codex/neoclr-namespace-metadata`. All six Linux/macOS/Windows stable/minimum-Rust
source CI jobs passed, along with 15 extracted-package suites, the match/signature
checks, both packaged language servers and isolated VSIX installation. All eight
uploaded asset digests were verified. The release includes source, the macOS arm64
runtime bundle, separate Raven SDK/VSIX, notices and validation evidence.

**Still separate:** This is the neoCLR experiment, not a full Raven release gate.
The earlier main audit's MacCatalyst/Xcode limitation remains documented. Runtime
library migration to Raven and broader semantic changes remain deferred.


## Website: features, influences and open discussion — 2026-09-14

**Author direction:** Improve the website to better market neoCLR and its implemented
improvements, list the project's influences and features, and demonstrate them with
samples. The author then requested areas being explored for improvement and a general
invitation for feedback and discussion.

**Assistant proposal:** Organize the page around the CLR foundation, design reference
points, current feature contracts and source-backed examples. Distinguish implemented
preview behavior from research and explain benefits alongside .NET compatibility costs.

**Action:** Expanded [the homepage](../website/index.html) with feature summaries and
six generated excerpts covering Result, Option, generic Void, collection capabilities,
UTF-8 slicing and query cardinality. Added research questions on nullability, memory
views, callables, async, text, collections and injectable clocks. Corrected outdated
application-type/capture limitations against the current API coverage record. Linked
general feedback to GitHub Issues after the repository API reported Discussions disabled;
no repository settings were changed. Noda Time is described as a research reference,
not an adopted API; Rust Result is a design precedent, not an imported ownership model.

**Evidence and status:** Reused [platform direction](platform-direction.md),
[API coverage](raven-runtime-api-coverage.md), [collection contracts](collection-contracts.md),
[query contracts](raven-query-api.md), [text model](text-model.md), and
[array variance](array-variance.md). Checked the linked Rust Result, .NET array/Unicode,
and Noda Time primary references on 2026-09-14. The website build checks embedded
excerpts and local links. These are website edits, not new runtime capabilities or a
new release; deployment is separate from the local changes.

**Author clarification:** The focus is demonstrating the platform's style through
Raven code, with some neoIL as a supporting view. “Discussion” means an invitation
to exchange ideas, not a request to use GitHub Discussions.

**Assistant adjustment:** Put a source-backed Raven example in the hero, moved
features and examples before the influences section, and placed the neoIL example
after the five Raven examples. Kept the invitation broad, with GitHub Issues as an
available contact route rather than defining discussion as a GitHub feature.

**Publication direction and validation:** The author requested committing this work
on main and publishing the site. The local build and repository-link checks passed;
browser checks covered desktop and 390px mobile, including example navigation and
absence of page-wide horizontal overflow. The assistant is committing the website
changes with the changelog and using the existing GitHub Pages workflow. Deployment
completion must be verified from that run rather than inferred from a successful push.


**Further author direction:** Organize the message around familiar value/reference
semantics and metadata/assemblies; Void as a real generic argument and its implications
for Func APIs; Option, Result, modernized collections and date/time; evolving APIs and
feedback; and Raven as the first adaptable compiler target with .NET-hosted tooling
and a VS Code extension. Describe the next work as moving the class library from
neoIL to Raven and implementing missing library features.

**Assistant action:** Revised the website around that outline and added excerpts
from the existing Void callback and calendar samples. Distinguished .NET-hosted Raven
tooling from neoCLR execution, and a migration direction from unchanged-binary
compatibility. Listed library migration and API expansion as planned work; this website
request does not itself resume the paused library implementation or override its
compiler-stabilization prerequisites. Reused the Void, delegate, calendar, format and
runtime API design records. Continued the authorized main/Pages publication workflow.

**Runtime additions requested by the author:** Mention the interpreter, garbage
collection and initial debugger support. Present a possible future JIT and garbage
collector improvements as areas under consideration.

**Assistant action:** Added current runtime capabilities with links to execution,
GC and debugger documentation, distinguishing the terminal debugger from future
Raven source debugging in VS Code. Added separate JIT and collector research cards;
no JIT selection, implementation or performance improvement is claimed.

**Positioning clarification:** The author wants neoCLR marketed as a software
platform improving on .NET and evolving through feedback, rather than centering the
compiler integration alone. The assistant revised the hero and platform/library
introductions to connect type system, execution engine, class library and tools,
with Raven as the demonstration language. The invitation asks visitors to help shape
the platform; implemented differences and future investigations remain distinct.


**Further structure and audience direction:** The author requested a concise overview
organized as Familiar, Runtime, Type system, Runtime class library (Option, Result,
unions, modernized APIs, collections, date/time and LINQ), Migration, Tooling and next
implementation work. It should be relatable to .NET developers but stand on its own,
explaining features, familiar behavior and differences rather than abstract sales
copy. UTF-8 remains prominent. APIs are evolving and feedback is welcome.

**Examples and highlighting:** The author requested Raven union sugar and a note
about its multiple construction forms, plus syntax highlighting following the adjacent
Raven repository or MyServiceBus's website. The supplied MyServiceBus path did not
exist; the assistant found `$HOME/Projects/MyServiceBus` and inspected its
TextMate/Oniguruma integration. Reused Raven's grammar with its MIT license and
adapted tokenization to build-time HTML spans rather than shipping Monaco or a
browser tokenizer. Added a test for token scopes, multiline comments, escaping and
exact source-text preservation. Source examples now use qualified Result factories
and imported Some construction; detailed union patterns and research are optional.

**Validation scope:** The Result propagation and UTF-8 samples compiled, verified
and ran using the locally available stabilization toolchain. A focused imported
Some construction program also compiled and ran with result 42. The full current
query sample could not run on that older package because its metadata predates the
query-terminal APIs; no full query-suite pass is claimed. The original `.Some` trial
failed and was replaced with the validated imported `Some(value)` form. No Raven or
MyServiceBus source was edited. Continue the already authorized main/Pages workflow.

**Invitation clarified:** The author reiterated that visitors should be invited to
join the discussion and give feedback, specifically about the runtime and APIs.
The closing section now names that focus directly.

**Website checks:** HTML nesting, anchors and repository links passed. Desktop and
390px mobile review confirmed the six-card library grid, highlighted code and union
disclosure behavior without horizontal page overflow.


## Indexer ergonomics in Raven — recorded 2026-09-14

**Author's report and direction:** Supplied a VS Code screenshot showing
`typeof(int).GetProperties().Item.` offering `PropertyInfo` members and said,
“This should be aligned with the experience in C#.”

**Assistant's response:** Explained that Item is an indexer metadata name and that
element-member completion belongs after indexed access, such as
`GetProperties()[0].`. Proposed treating this as a general Raven fix, separate from
the imported-union emission work already in progress.

**Action and outcome:** Reproduced the behavior using ordinary .NET collection
classes/interfaces and a Raven-declared indexer. Fixed named lookup, symbol
referenceability and semantic-model fallback, with positive coverage for indexed
access and ordinary properties named Item. Integrated `ac4901f6b` into Raven main;
440 focused checks passed. The preceding union-emission fix was separately integrated
as `43f288b05`, with 291 focused checks and the .NET 10/.NET 11 matrix passing.

**Still open:** The experimental Raven branch and installed SDK/extension have not
been synchronized or rebuilt in these slices. The source fixes do not yet change
the author's installed editor. See the [target evaluation](raven-target-evaluation.md)
for validation scope and remaining stabilization reviews.


## Empty arrays on the generic array type — recorded 2026-09-14

**Author's direction:** Pointed out that neoCLR has `Array<T>` and proposed
`Array<T>.Empty`, with Empty probably exposed as a property.

**Assistant's response:** Agreed that the closed array type already supplies the
element type. Distinguished this proposed neoCLR API from .NET's
`System.Array.Empty<T>()` method, and from Raven's general ability to emit an empty
array when a target lacks that factory.

**Action and status:** Added this distinction to the ongoing array-factory review.
The property and its compiler mapping are proposals, not implemented in this entry.
Any neoCLR-specific mapping remains on Raven's experimental branch; general
metadata-capability handling belongs on Raven main after independent validation.


**Follow-up direction:** The author pointed out that ForEach no longer needs to be
static with `Array<T>`. The assistant proposed an instance `ForEach(action)` method,
with the receiver supplying both the array and its element type, retaining the
existing `Func<T, Void>` callback contract. This is recorded for the array API slice;
the static implementation has not yet been replaced.


**Implementation follow-through:** Added the static Empty getter and instance ForEach
to the Raven runtime profile, compiler reference declarations and importer catalog.
Raven's experimental array projection now exposes configured shape members. Empty
uses a zero-length allocation for now; no shared-identity guarantee is made. Four
Raven samples compiled, imported, verified and executed, and 121 signature checks
passed. Expression-bodied callbacks exposed a general void-call stack defect;
`8dbd96fb6` was independently reproduced/tested and integrated into Raven main, then
cherry-picked as `5c32d1d06` into the experiment. Nominal generic Void emission remains
experimental. Installed tools have not yet been refreshed.


## Target profiles, symbol implementations and future reach — recorded 2026-09-14

**Author's direction:** Suggested avoiding accumulated framework-specific exceptions
in Raven's semantic model and code generation, using separate symbol implementations
where helpful and configurable contracts such as Iterable versus IEnumerable. Proposed
.NET and neoCLR Target Profiles encompassing .NET versions, NanoFramework and neoCLR,
and eventually refactoring emission to support backends beyond Reflection.Emit.

**Assistant's response:** Proposed separating a framework profile from an emission
backend: several profiles could share one backend. Suggested dedicated/composed symbol
representations where behavior really differs, while keeping common CLI symbols and
language rules shared. The generic array shape is a candidate for evaluating that
boundary. Proposed preserving the current stabilization scope rather than starting a
backend rewrite inside the array slice.

**Author's additional future directions:** Asked to record that Raven might eventually
be bootstrapped, with its compiler written in Raven. If neoCLR becomes a project with
sustained investment, consider different architectures, Native AOT and potentially
microcontrollers. Explicitly described these as far-future considerations.

**Assistant's response and action:** Recorded the
[profile/symbol/backend proposal](raven-target-profiles.md), linked it from the contract
assessment and roadmap, and separated compiler bootstrapping, runtime portability,
ahead-of-time application compilation and constrained runtime profiles. These are
possible directions, not promises or selected implementations. No bootstrapping, new
backend, architecture port or microcontroller implementation was started.

**Concrete outcome of the ongoing slice:** The generic array API is committed in
neoCLR as `ff617c4` and its compiler projection in experimental Raven as `8823261b6`.
The general void-invocation fix was independently integrated on Raven main as
`8dbd96fb6`. The array implementation remains an incremental metadata-driven projection;
a full target-profile abstraction and specialized array-symbol design remain open.


**Author's clarification:** “I mean we might support microcontroller architectures as
a target.” Named the broader theme “targetability and portability.” The assistant
updated the architecture proposal and roadmap accordingly: architecture targeting and
implementation portability are the direction; a smaller runtime profile is only one
possible future consideration, not the meaning of the request.


## MSBuild support for Raven projects — recorded 2026-09-14

**Author's direction:** “We should have some proper MSBuild support targeting neoCLR
eventually so that we can use the Raven project files.” Explained that this is useful
for compiling projects.

**Assistant's response:** Proposed treating this as a future targetability milestone:
reuse Raven project compilation and supply neoCLR reference/build assets, with MSBuild
orchestrating compilation and the temporary import/verification stage. Suggested
single-project support first, followed by incremental builds, project references and
consistent editor resolution. Package/SDK names and a framework moniker remain open.

**Action and outcome:** Added the proposed scope, alternatives and validation criteria
to the [target-profile design](raven-target-profiles.md#msbuild-project-support--future-milestone-2026-09-14)
and roadmap. This records future work; no MSBuild integration was implemented in this
exchange, and the existing local .14 installation is unchanged.

**Author's clarification:** “I don't think we should have it all” — MSBuild is a useful
initial build system to support. Added: “It won't be a .NET SDK integration, but the
experience using the project files will be similar.”

**Assistant's response and action:** Narrowed the recorded initial scope to a small
MSBuild path for `.rvnproj` compilation using neoCLR references and build targets,
independent of Microsoft.NET.Sdk. Kept incremental builds, project references and
asset restore as optional later slices rather than requirements for initial support.
The familiar project-file experience is the goal; no SDK integration was selected.


**Implementation follow-through:** After the author asked to continue, the assistant
implemented standalone MSBuild props/targets and a `.rvnproj` template in neoCLR.
The existing installed Raven compiler compiles the project, followed by independent
import/verification; Build does not run the program. No Raven edits or .NET SDK
integration were needed. Added distribution/editor build-task configuration and
[documented the bounded build path](raven-msbuild.md), with larger build-system
features explicitly deferred.


**Author's observation:** A project targeting neoCLR should also improve the language
server experience by giving it context.

**Assistant's response:** Agreed that source items, references and target contracts
should come from the same project evaluation for build and editor. Raven already
reads `.rvnproj` files for language services; the standalone imports centralize that
configuration. Proposed checking completion against the actual new MSBuild project,
rather than assuming successful compilation proves editor behavior.


**Verified outcome:** The installed standalone MSBuild project passed all 68 editor
checks against its configured language server, confirming shared project/reference
context without a .NET SDK import. The fresh local bundle also passed 15 build
scenarios; its propagation demo built and ran separately. Instructions and exact
revisions are in the [MSBuild guide](raven-msbuild.md). This is a local installation,
not a new public release.


### 2026-09-14 — Plan reflection around needs before expanding descriptors

**Author's direction:** Requested `Type.IsValueType` and restrained additions to the
existing model. Then asked for a roadmap review of reflection versus introspection,
using .NET's `Type`/`TypeInfo` split as a concern: identify the needs before choosing
how to implement the functionality.

**Assistant's response:** Proposed examining metadata inspection, runtime execution
capabilities, portability and AOT separately, without preselecting separate public
classes. Kept the bounded `IsValueType` addition in the active preview scope.

**Action and outcome:** Added a [roadmap investigation](reflection-model-review.md)
with a .NET baseline, alternatives and validation criteria. This records planned
architecture work; it does not claim a new reflection implementation or a completed
preview release.


**Author's follow-up:** API planning can identify the core of the platform, its
standard contract, and features or APIs that differ by target.

**Assistant's response and action:** Added a common-core, optional-capability and
host-service classification to the runtime API plan and roadmap. Required availability
and conformance questions to be resolved before choosing profiles or capability
metadata. The classification is a planning framework, not a finalized standard or
an implemented target-capability system.


### 2026-09-14 — Combine the MSBuild workflow and library demonstration

**Author's direction:** Asked to continue with the three proposed slices, then said
“Or together.” The assistant proposed a combined preview: make standalone MSBuild
the primary workflow, add a bounded application-plus-library scenario, then validate
and publish the complete package. The author subsequently requested IsValueType and
future reflection/core-contract planning.

**Implementation so far:** Added one-level ProjectReference orchestration using
Raven's existing project evaluation, with exact reference-pack compatibility checks,
changed-library rebuilds and failed-output invalidation. The source MSBuild suite
passes 22 scenarios; the library project's editor scenario passes 69 checks before
building. Added separate Build/Run tasks and a readable two-project example. No Raven
compiler changes were needed. Packaged release validation and publication remain
pending; these source-check outcomes are not a release claim.


### 2026-09-14 — Task-based async with runtime support

**Author's direction:** Said that neoCLR need not retain all backward compatibility
and can choose the latest async model: “we should stick with task-based, the
task-based abstractions.” Intended execution support belongs in the runtime.
Compiler-generated async state machines are not necessarily required, but may be
used transitionally because Raven already supports emitting them (author-reported
capability, not validated for neoCLR in this exchange).

**Assistant's response:** Distinguished the selected public task abstraction from
runtime suspension and resumption. Clarified that removing compiler-generated state
machines does not remove the need to preserve execution state. Proposed treating
Raven lowering as an optional transition and retaining modern .NET as a comparison,
without assuming that newer machinery is inherently faster or copying legacy contracts.

**Action and outcome:** Updated the [platform direction](platform-direction.md#async-and-time-preserve-the-application-model),
backlog, roadmap and Raven application notes to reflect this decision. Consulted the
.NET TAP documentation and runtime-async draft; no runtime or compiler implementation
was changed. Earlier entries describing the model as open remain historical positions.

**Open:** Concrete task types, no-payload completion, Result/error and cancellation
semantics, scheduling, references across suspension, GC roots, cleanup, debugger
integration and metadata. A pinned implementation comparison and a bounded execution
probe remain needed; this exchange does not choose opcodes or promise a release.


**Author's clarification:** The runtime will eventually contain suspension logic,
while Task still represents the operation. Explicitly identified Task<void> as possible in neoCLR, using its ability to accept
Void as a generic argument.

**Assistant's response and action:** Recorded Task as the operation representation
and Task<Void> as completion without a result payload in the platform direction.
This sharpens the intended contract; task members, suspension machinery and error
semantics still require design and implementation.


**Author's library-design clarification:** Application projects may still use
callbacks. The runtime class library and System namespaces should be designed for
modern consumers and modern programming languages.

**Assistant's response and action:** Recorded this distinction in the
[API policy](api-policy.md#design-for-modern-language-consumers) and linked it from
the async direction. Applied task-based completion to public asynchronous operations;
explained that callbacks can still serve roles such as predicates, event handlers
and internal adapters. These examples are the assistant's interpretation of the
principle, not additional author-selected API contracts. No APIs were implemented
or removed; detailed contracts and adaptation needs remain open.


**Author's correction:** Clarified that some runtime APIs may themselves be better
expressed as callbacks. Tasks are the normal way to represent asynchronous execution,
not an exclusive requirement for all runtime APIs.

**Assistant's response and action:** Updated the API policy and platform direction
to state this explicitly, including public runtime APIs. The earlier callback
examples were illustrative, not an exhaustive set of exceptions. No implementation
changed.


**Author's feasibility question:** Asked whether neoCLR already has everything needed
for compiler-generated async/await, given Raven's existing state-machine emission,
and whether it could be made to work before runtime suspension.

**Assistant's assessment and action:** Inspected the local Raven lowering and neoCLR
contracts and recorded a [source-backed feasibility assessment](async-state-machine-assessment.md).
The approach appears feasible without suspended VM frames, but current task/builders,
error lowering, heap state identity and continuation/execution ownership need work.
Proposed a guest queue and an initially incomplete await as a bounded probe. No async
implementation or execution test was performed; no Raven source was modified.


**Author's error-model clarification:** Directed that errors should continue to
propagate through Result, giving Task<Result<T>> as the asynchronous shape.

**Assistant's response and action:** Updated the async direction, API policy and
feasibility assessment: Task carries completion and Result carries success or a
recoverable error. Await yields Result; an Err completes the task as a value rather
than entering a task-exception channel. Recorded the current explicit-error spelling
Task<Result<T,E>> without claiming a one-parameter Result alias is implemented.
Cancellation and terminal Fault handling remain separate open contracts. No code
was changed or async execution validated.


**Author's API-design questions:** Asked whether asynchronous APIs should also have
synchronous versions, whether methods need an async marker, and whether neoCLR
should use different conventions or behavior from .NET.

**Assistant's proposals and action:** Recorded an [API design discussion](async-api-design.md)
comparing .NET guidance, Rust activation and .NET ecosystem naming policy. Proposed
purposeful sync/async surfaces, explicit Task/await contracts, familiar names as a
starting point and further decisions on cancellation, ownership and scheduling.
Distinguished optional frontend async syntax from implicit suspension and required
runtime metadata. These recommendations have not been adopted by the author or
implemented; broader comparison and executable validation remain open.


**Author's naming clarification and decision:** Clarified that the question concerned
Async method-name suffixes. After the assistant recommended ordinary async names,
the author agreed: make async the ordinary operation, identified by return type,
with blocking/sync alternatives as the exception. Stated that existing synchronous
APIs were a major reason for Microsoft's .NET naming convention. Requested that the
decision be documented and committed. The author also explicitly corrected the error
shape to Task<Result<T,E>> earlier in this exchange.

**Assistant's revised proposal and action:** Initially proposed retaining Async in
the broader review, then recommended omitting the mandatory suffix after the naming
clarification. Updated the API policy, async comparison and platform direction to
record the selected convention and distinguish it from still-open behavior proposals.
Recorded the historical explanation as the author's rationale, supported by TAP's
coexistence guidance rather than claiming a complete history of Microsoft's decision.
Prepared this documentation and the preceding async discussion for commit; no runtime
or compiler implementation or existing API names changed.

**Release outcome (2026-09-14):** Published Preview 7 from `5da27a7` after six
stable/minimum-Rust Linux/macOS/Windows CI jobs and all 18 packaged validation suites
passed. The package includes MSBuild Build/Run tasks, the one-library demo and
Type.IsValueType; the SDK/VSIX remain the reviewed experimental Raven .14 build.
The core-contract and introspection/reflection reviews remain plans. Evidence and
checksums accompany the [release](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.7).


### 2026-09-14 — Namespaces rather than utility classes

**Author's direction:** Asked to resume library migration, then clarified that
System.Math should be a namespace containing functions. Directed that, at least
for now, this should follow Raven's existing .NET metadata contract. Explained that
utility classes usually substitute for namespaces of functions, and that neoCLR
need not copy that public structure. A real namespace should also accommodate
related types and other declarations, not only functions and constants.

**Assistant's response and action:** Agreed to separate the public namespace API
from its CLI representation. Recorded the [API guideline](api-policy.md#namespaces-for-utility-functions-2026-09-14)
and updated the Math migration plan. Inspected the existing namespace-member probe
and reran it successfully (eight checks): target-owned TopLevelAttribute metadata,
separate consumer imports and overloads, completion, accessibility and rejection of
unmarked lookalike containers. No new binary metadata format or Raven compiler
changes were introduced for this decision.

**Outcome and remaining work:** The guideline is recorded. The earlier uncommitted
static-class Math pilot needs to be adapted; the namespace Math reference surface,
qualified calls, coexistence with related types, and implementation import still
need validation together. The probe result establishes the existing namespace
contract, not completion of the Math migration. Cross-language consumption and
reflection of generated containers remain explicit design costs.


### 2026-09-14 — Shared System project and namespace lookup correction

**Author's follow-up:** Directed continuation of the namespace-function pilot, then
asked for a shared project for foundational runtime APIs, comparing it with the
historical mscorlib grouping. Subsequently explicitly asked to fix the Raven
resolution issue too. The assembly's historical name was context for the grouping
question, not a selected neoCLR assembly identity.

**Assistant's implementation:** Consolidated the pilot into
[System.rvnproj (now System.Runtime.rvnproj)](../runtime/raven/System.Runtime.rvnproj), with Math as its first source.
The source exposes namespace functions through Raven's existing CLI container and
TopLevelAttribute contract. Added fresh bootstrap reference generation, ordinary
Raven project compilation, signature-checked importing and checked-in executable
fragments. Retained the existing internal Math method owner for direct IL and the
archived Neo frontend, while exposing a namespace in Raven. Generated bodies replace
the five handwritten scalar implementations; native Double services remain.

**Problem found and compiler action:** A consumer adding a type to System.Math could
use wildcard-imported functions but could not call them with qualified names.
Reduced this to ordinary .NET reference metadata: source-only namespace lookup
omitted referenced functions and constants. A focused regression failed before the
fix; all 284 namespace/import tests passed after qualified lookup used the merged
namespace. Integrated only that general correction into Raven main as `3ec32c96e`,
then cherry-picked it onto the experiment as `008cb3245`. Removed the temporary,
fully merged correction branch. No neoCLR policy was added to Raven main.

**Validation and remaining scope:** The separate Raven consumer checks 69 outcomes,
including named arguments, wildcard/qualified calls, a related namespace type,
Int32 boundary values, Result payloads/errors and a Double function. Six invalid
library contracts are rejected without executable output. Fresh regeneration matches
the checked-in bodies, and existing direct runtime Math tests pass. The
[authoring document](raven-system-library.md) distinguishes the bootstrap reference,
implementation input and executable library. General generic-body importing and
migration of core type definitions remain later gates. The installed .14 SDK is
unchanged and predates the compiler correction; no new release is claimed here.

**Author's source-style and workflow follow-up:** Suggested importing
`System.Result.*` and `System.Option.*` to use `Ok`, `Error`, `Some` and `None`
directly, then expressed the hope of porting the API to Raven slice by slice without
interruptions. The assistant identified a repeatable build/validation path as the
objective, with generic bodies and shared type identities still requiring proof.
Updated Math and its consumer matches to imported Result cases. The qualified
carrier annotation remains explicit after the signature guard caught an unresolved
return-annotation issue; recorded that as a deferred general Raven candidate, not a
completed compiler fix. This slice establishes the first migration path, not proof
that every remaining API can already be translated without integration work.

### 2026-09-14 — Runtime Contracts and one unit value type per target

**Author's direction:** Asked for Raven targeting neoCLR to use Void wherever its
.NET target uses Unit, while keeping ordinary void calls free of a stack result.
Clarified that Unit is not another neoCLR platform type. Compared this selection
with Iterable/IEnumerable selection, endorsed the name “Runtime Contract”, and
explicitly directed integrating reusable target support into Raven. Also asked for
compiler changes and the neoCLR integration to be documented in both projects.

**Assistant's response and work:** Separated the unit value representation from the
no-result call convention and added an opt-in unit contract to compilation/project
options. The selected target type represents `()` in value contexts; intermediate
emitter Unit machinery is removed from target output. Built an end-to-end probe for
Void values, calls and Result<Void, E> propagation. Independently extracted metadata,
iteration, propagation and core-selection mechanisms onto a main-based Raven branch,
retaining default .NET exception behavior and excluding the experimental generic-array
and no-exception policies. General unit-contract tests use System.ValueTuple on .NET.

**Related compiler correction:** The earlier unresolved-return candidate was reduced
to an ordinary .NET diagnostic defect and fixed on Raven main as `0c66fbaa7`, with
`e20534894` in the experiment. Missing imports now prevent emission. Case imports and
carrier-type imports remain distinct. Generic Unit storage testing also exposed an
invalid generic void emission in the default target; the correction preserves a real
unit value type in generic arguments.

**Scope:** Runtime Contracts describe compiler-facing requirements; a target profile
selects their values and supplies references. They do not implicitly select exception,
array variance or cleanup policies. Generic implementation importing was deferred in
this turn to prioritize the author's unit-contract direction. No SDK installation or
release is implied by these source changes; final validation/integration is recorded
with the implementation commits and changelog.

**Completed outcome:** Raven main now includes the reusable metadata, iteration,
propagation, core-selection and unit contracts through `2d17199a1`; the temporary
integration branch was fast-forwarded and removed. The experimental branch adopts
unit projection as `fc4592663`, retaining its separate target policies. Raven's
compiler documentation and both repositories' workflow instructions now require
compiler-affecting changes to be recorded on both sides. The combined contract/project
suite passes 87 tests; Raven's CI gate passes 311 compiler, 73 core and 249 language-server
checks with three existing skips. The experimental unit suite passes 13 checks.
The neoCLR probe emits no System.Unit and executes six expected output lines,
including both propagation paths. All 69 Math consumer results and six rejected
contracts pass; regenerated Math bodies remain identical. These are local commits;
no new tools installation or publication was performed.


## 2026-09-14 — Larger library migration slices and an Operators container

**Author direction:** Asked for larger slices and to continue porting feasible runtime
library code, fixing Raven/compiler or neoCLR runtime problems encountered along the
way. The previous rule remains: general Raven fixes go to main independently;
neoCLR-specific target policies stay experimental.

**Assistant action:** Extended the bounded importer from direct generic parameters to
constructed collection, delegate and union signatures. Ported ToList, First, Last and
Single with predicate overloads into the shared Raven System project. Preserved
Option/Result outcomes and iterator cleanup, then checked the generated bodies with
existing query/runtime tests. General imported-signature, generic-member emission and
same-namespace type-lookup defects were reproduced with ordinary C# reference
contracts and corrected independently in Raven. Bare empty member-union returns also
exposed missing case construction; the independent .NET regression and correction
cover return expressions and explicitly typed local initializers.

**Author refinement:** “We should probably not call that type Enumerable but Operators
perhaps.” **Assistant response and action:** Adopted `System.Linq.Operators` for the
extension-method container, updating reference metadata, direct IL calls and docs;
receiver syntax remains `items.First()`/`items.ToList()`. Consumers need rebuilding.

**Outcome and remaining work:** The seven terminal overloads execute from Raven-generated
bootstrap bodies. Instance type definitions and deferred iterator classes still require
shared implementation/reference identity support before they can migrate faithfully.
The [authoring document](raven-system-library.md) records this boundary and validation;
compiler fixes and commit references are tracked in the
[integration evaluation](raven-target-evaluation.md). No release or local tools refresh
is implied by this source migration.

## 2026-09-15 — Injectable clocks and factories on collection interfaces

**Author proposal:** Suggested a `Clock` interface with a `SystemClock` implementation
for dependency injection and mocking. Considered `Clock.Instance`, but explicitly
said this might be bad design and emphasized the abstract time-provider purpose.
Separately proposed `List<string>.Create()` to request an ArrayList through its List
contract, observing that List is currently mutable.

**Assistant response:** Recommended selecting SystemClock at application setup and
passing Clock to time-dependent code. A possible SystemClock.Instance convenience
would keep that choice visible; a static default does not inherently rule out
injection. Considered the collection factory a reasonable convenience for fresh
mutable storage, with concrete construction retained when implementation-specific
behavior matters. Highlighted the dependency from a factory on the interface to its
default implementation and the need to validate static interface-member binding.

**Actions and outcome:** Checked the implemented List/MutableSequence/Sequence
hierarchy, confirming List supports Add and indexed replacement. Recorded the
[clock proposal and comparisons](date-time-design.md#clock-contract-and-default-implementation-proposal--2026-09-15)
and [collection factory tradeoffs](collection-contracts.md#interface-owned-factory-proposal--2026-09-15).
Neither factory/default accessor nor a new Clock interface was implemented. These
remain proposals; clock method shape and final factory placement are open.

**Ongoing migration:** Completed the separate nongeneric instance import gate in
`d073ee0`, with constructor/private-state/property execution and contract rejection
checks. This does not yet port generic collection implementations or establish
static members on interface implementations.

## 2026-09-15 — A namespace function for terminal faults

**User proposal:** While continuing the runtime-library migration, proposed a
`System.Fault` function “for causing the system to fault and terminate”, then clarified
“with a message”.

**Assistant response and action:** Selected the existing terminal guest-fault boundary,
with a computed String diagnostic, and implemented a namespace function using Raven's
existing CLI namespace-container contract. Distinguished guest termination from forcibly
aborting an embedding host and made no cleanup guarantee. The compiler's recognition
of non-returning calls is a separate open issue; no name-specific Raven policy was added.
See [the contract and validation](system-fault.md). This provides a library fault API;
checked generic storage and private query implementation dependencies remain migration work.


## 2026-09-15 — Confirming Raven fixes against .NET

**User question:** “You do verify bugs in Raven with its .NET target, do you?”

**Assistant response and action:** Confirmed that the imported-generic-method repro
uses an independent C# library and executes on .NET 11 under default and explicit
System.Runtime metadata options. Two explicit-metadata cases failed before the fix.
A separate generic-array repro subsequently crashed its isolated .NET test host;
typed element access repaired that general emission defect.

**Outcome:** Integrated the fixes separately into Raven main as `730adc7b0` and
`8dfb64a2b` after focused tests and the bounded integration gates passed, then retained
the experimental cherry-picks separately. Removed the temporary general branches.
No .NET Framework/NanoFramework execution was claimed. Checked storage and private
library-helper admission remain specific to the neoCLR experiment. See
[the evaluation and validation](raven-target-evaluation.md).

## 2026-09-15 — NeoCLR Time API v1 proposal

**Author proposal:** Define a layered time architecture with `Instant` as the real
timeline point, `Duration` for elapsed time, separate civil values (`Date`, `Time`,
`LocalDateTime`), explicit offset/zone projections, independent calendars, and
`Period` for calendar arithmetic. Keep `Clock` as a narrow injectable provider of
`Instant`, with system, fixed and manual implementations. Model timezone gaps and
overlaps as explicit union outcomes, and use Result for parsing and lookup failures.
Retain recognizable .NET naming and ergonomics without creating another combined
`DateTime` type. The supplied Raven-shaped examples are illustrative; their Raven
validity was not asserted.

**Assistant action and outcome:** Updated the [date/time design proposal](date-time-design.md#neoclr-time-api-v1-proposal--2026-09-15)
with the type responsibilities, .NET/Noda Time comparison boundary, tradeoffs and
validation requirements. No new time types, timezone implementation, calendar system,
parser, formatter or runtime contract were added by this documentation update. The
existing Date/Time core and static local-clock API remain implemented; the v1 layering,
method signatures, defaults and DST policies remain provisional.

## 2026-09-15 — Minimal Unicode-centred text model proposal

**Author proposal:** Refine the text direction to a small Unicode-centred model:
Unicode supplies the semantics, String uses canonical UTF-8 storage, UTF-8 and UTF-16
are explicit representation views, and ASCII is a constrained subset rather than a
separate text system. The proposal makes Char a Unicode scalar value, suggests
`AsciiChar`/`AsciiString` constrained representations and explicit Result-based ASCII
narrowing, and leaves other encodings as `Encoding` codecs. The supplied Raven-shaped
examples are illustrative; their validity in Raven was not asserted.

**Assistant action and outcome:** Recorded this as a substantive proposal in the
[text-model review](text-model.md#minimal-unicode-centred-model-proposal-2026-09-15),
including the conflict with the currently implemented UTF-16 code-unit Char contract,
the .NET-compatible alternative, migration costs and validation needs. No compiler,
runtime, metadata or String API behavior was changed. The choice between the existing
Char/Rune split, a scalar-oriented String with a compatibility Char view, and the
minimal scalar model remains open.

## 2026-09-15 — Consolidated String and encoding proposal

**Author clarification:** The intended follow-up was to ensure that the String proposal
captures the supplied detailed model: immutable Unicode String as a sequence of scalar
Char values, canonical UTF-8 storage, explicit `Utf8String`/`Utf16String`/`AsciiString`
representation types, a separate `Encoding` transformation abstraction, and the
existing Option/Result/Fault conventions. The examples are illustrative and are not
asserted to compile as Raven.

**Assistant action and outcome:** Expanded the [text-model proposal](text-model.md#consolidated-string-proposal-semantic-text-and-representation-views)
to make that model the preferred direction for further review. The current UTF-16
code-unit Char behavior, existing String slicing contract and runtime implementation
were not changed; migration, view lifetime, metadata and interop validation remain open.


### 2026-09-15 — Finish the port before revisiting API design

**User direction.** The author asked to “Finish the ongoing work” and clarified:
“We won't delve to much into the design of the API. We have design proposals, but
that is the focus after the port.”

**Assistant response and action.** The assistant scoped completion to the ongoing
Time port, preserving its existing public behavior and deferring Date and API
redesign. The implementation moves Time into Raven and extends the checked library
importer for its existing interfaces, static factories and readonly receivers.
A parser issue reproduced with Raven's .NET target is recorded as a deferred general
compiler candidate; no Raven compiler changes are included in this slice.

**Outcome.** Source and regression coverage are recorded in
[the library migration notes](raven-system-library.md#time-value-implementation--2026-09-15).
Design proposals remain proposals and will be considered separately after the port.


### 2026-09-15 — Broader implementation batches after Date

**User direction.** After Date, the author requested a “bigger and safer slice” covering
as many APIs/areas as possible, with separate commits.

**Assistant response.** Finish and commit Date first, then select existing APIs whose
contracts fit the checked importer. Preserve behavior and separate commits by area;
API redesign remains outside this migration work.


### 2026-09-15 — Namespace folders for library sources

**User direction.** The author specified that each runtime-library namespace should
have its own folder.

**Action.** Raven sources move under `src/System`, with subfolders matching declared
namespaces. Type files retain type names; namespace-function files use Functions.rvn.
Project inputs, generation source maps and documentation follow the new paths.
This is source organization only, without changing APIs or assembly partitioning.


### 2026-09-15 — Current folders do not settle future type ownership

**User observation.** The author noted that structure may change later and that
System.Char functions should probably belong to a ported System.Char struct.

**Assistant response.** Agreed to preserve that as a later structural change. Current
folders follow current declarations; the ongoing port does not redesign the public
Char type or make the function namespace permanent.


### 2026-09-15 — Port the fundamental structs as a family

**User direction.** The author requested as many fundamental types as possible,
noting that several have few or no members and should share the already ported logic.

**Assistant action.** Established a checked intrinsic-storage rule with Int64 and
extended it to ten numeric/Boolean structs. Kept existing APIs and comparison
behavior. Native-sized integer types retain IL while their operator/conversion
reference contracts are incomplete; the corresponding comparison compiles against
.NET with the same Raven compiler. No Raven compiler changes are included.

## 2026-09-15 — Minimal clock layer and public API preview

**Author direction.** Introduce Clock.Now returning Instant, a SystemClock backed by
system time, and Duration. Keep the initial layer small. The author then asked for
system-local rendering into LocalDateTime for the demo, and a website preview of
both this API and the Type/introspection versus TypeInfo/reflection boundary.
The author corrected the assistant's attempt to update an archived `.neo` sample:
current examples must use Raven.

**Proposal and refinement.** The author supplied the broader Time API v1 proposal
(Instant/Duration, civil time, separate offsets, zones, calendars and Period) and
asked that the implementation comply with its minimal subset. The assistant chose
signed 100 ns ticks, a narrow injectable clock and a provisional system-zone
conversion. To match the proposed civil-time boundary, LocalDateTime now contains
only Date and Time; the old static Clock.GetLocalNow and offset property are removed.
FixedClock is demonstrated as an application-defined test double, not a new library
class. The broader proposal remains future work.

**Actions.** Instant and Duration are authored in Raven; system-clock and local
conversion services use typed runtime calls. Raven examples demonstrate clock
substitution. Website excerpts come from executable Raven samples and distinguish
current development APIs from the published preview. See [the minimal contract](instant-clock.md)
and [the broader proposal](date-time-design.md). No Raven compiler or .NET target
changes are part of this slice.

### 2026-09-19 — Put TypeInfo in the member hierarchy, then address strings

- **Author:** explicitly selected “Make TypeInfo extend MemberInfo” after asking
  whether that relationship had already been implemented. This reopens the earlier
  closed Introspection story for that bounded change.
- **Assistant:** implemented the fourth sealed case and optional declaring-type
  ownership, preserving nested source ownership through metadata rather than names.
  Co-located the declarations to satisfy Raven's existing same-file rule.
- **Author:** directed the following order: “strings API - minimal API adjustments
  and behavior implementation”, then the feature page.
- **Assistant:** accepted separate commits in that order. String work and its page
  remain subsequent slices; this entry does not report them as completed.

- **Author, subsequent correction:** asked that APIs and samples favor union
  patterns/destructuring over GetSomeCase-style extraction and idiomatic case
  construction over explicit carrier wrapping. Requested a separate cleanup after
  this hierarchy slice and before strings.
- **Assistant:** updated this slice's owner access and accessor filtering to patterns,
  and used imported Some/None case constructors. Recorded the ongoing style rule in
  AGENTS.md; the wider cleanup remains the next commit.

### 2026-09-19 — Document idiomatic Raven and clean up before strings

- **Author:** requested a cleanup after the TypeInfo commit and before String work,
  then showed `val DeclaringType: Option<TypeInfo> => Some(StoredDeclaringType)` as
  the preferred property form. Asked to document idiomatic Raven conventions and
  look at the Raven repository for guidance.
- **Assistant:** read Raven's style guide, feature-meaning guide, property spec and
  pattern/Option/Result feature docs. Added a linked neoCLR conventions guide,
  distinguished target bootstrap ABI from ordinary user code, and applied the
  conventions to runtime properties and sample union usage.
- **Assistant:** asked whether the String slice should begin with strict UTF-8
  conversion or the larger proposed scalar-based Char redesign.
- **Author:** “Do what you believe benefits us right now”.
- **Assistant decision:** begin with strict UTF-8 conversion while preserving the
  current String/Char semantics; scalar Char remains deferred. This is a plan for
  the next slice, not a report of its implementation.

- **Validation finding:** the wider application checks exposed a pre-existing
  source-token mismatch for value-type constructors lowered to free functions.
  The assistant fixed the synthetic receiver token slot separately in `346678f`;
  the class-identity/value-copy sample now executes and the application suite passes.

- **Author:** clarified that `()` maps to void and that neoCLR-targeting Raven should
  use the `void` keyword rather than `unit`, at least for now.
- **Assistant:** verified that the runtime-unit mapping already selects System.Void,
  but the parser does not yet recognize lowercase void. Recorded the desired
  spelling and the implementation gap separately in the conventions guide.

- **Author, revised decision:** “Or we keep unit keyword as mapping to System.Void.
  For now.”
- **Assistant:** retained Raven's existing keyword and target configuration; no
  keyword compiler change was made. Updated the conventions while preserving the
  earlier suggestion and correction in this record. String work remains next.

## 2026-09-19 — Minimal String boundary API

The author asked to continue with strings after the introspection and idiomatic
Raven cleanup, leaving the feature page for the following slice. When asked whether
to start with strict UTF-8 conversion while retaining String/Char semantics, the
author replied, “Do what you believe benefits us right now”. The assistant chose
System.Text.Utf8 with Sequence<byte> and a typed invalid-input Result, preserving
Char's code-unit behavior and deferring Encoding and specialized Utf8String types.
The implementation and tradeoffs are recorded in [the String integration notes](raven-string-api.md).
The author's last source-keyword correction remains unit mapped to System.Void;
this slice makes no keyword change. A String feature page follows separately.

### Feature pages as implementation notes

The author emphasized that the API should stay minimal, demonstrate working
behavior and remain open to input. A feature page need not show everything and
should describe the current implementation, which may change. The assistant
shortened the String page to one executable conversion example, linked detailed
tests and documented provisional status. The author then proposed a separate
summary page for proposals, with feature-specific future-direction sections.
That overview is the following documentation slice, separate from shipped behavior.

The proposal overview is now implemented at `website/proposals/index.html`, with
links from the homepage and feature pages. It summarizes existing design records,
labels current foundations separately and records costs and unresolved questions.
String and Introspection pages now have short future-direction sections. This
changes documentation only and does not promote proposals into implemented APIs.

### Website upkeep and publication

The author asked to document the structure for later updates, keep the site aligned
with feature work and the product at release, and suggested manual publication
instead of publishing with pushes. The assistant documented the workflow in
[feature-page maintenance](design/feature-pages.md), linked it from contributor
instructions and AGENTS.md, and changed Pages deployment to manual dispatch on main.
Push and PR validation remain automatic. This is a local workflow change; no site
publication has been performed, and the remote policy changes only after integration.

### Native UTF-8 direction supersedes the compatibility carryover

The author clarified: “We don't want to use UTF-16 in NeoCLR”, directing work toward
native UTF-8 and following the String proposal. This supersedes the assistant's
earlier suggestion to preserve UTF-16 semantics. String storage was already UTF-8;
the assistant changed CompareOrdinal to native byte/scalar ordering and updated
samples and direction notes. Scalar Char is now the selected migration target,
not merely an optional proposal; its existing 16-bit representation remains an
explicit implementation gap requiring coordinated runtime, metadata and compiler
changes. This slice does not claim that migration has already happened.

### Further pages and the preview review

The author directed the next sequence: finish the String feature, then create the
other website feature pages, then review the whole preview to decide what it needs
to communicate the direction. The assistant added short notes for the existing
Option/Result, collection/query, date/clock and bounded-file APIs using tested source
excerpts. The release review distinguishes implemented paths, the remaining scalar
Char migration and packaging/validation gates from optional proposals. No new
release version, date or publication has been selected.

The assistant's [readiness review](preview-readiness-2026-09-19.md) identifies scalar
Char as the clearest remaining implementation gap before claiming the selected
native text model. It recommends keeping broad proposal families deferred and
prioritizing a pinned, validated evaluator bundle. A fresh local VS Code UTF-8
workspace runs and completes correctly. These are local findings, not a selected
release scope, version/date, cross-platform gate or publication.

### Scalar Char implementation begins

After the author asked to continue, the assistant began the outstanding scalar-Char
migration. The runtime now stores and validates 32-bit scalar values, uses four-byte
native storage and classifies supplementary characters from the pinned Unicode data.
The Neo frontend accepts supplementary literals. Raven's targeted lexer/literal
baseline passes (70 tests); the target-specific compiler and bridge integration is
next. This does not yet claim a complete Raven scalar-Char experience.

### Feature links and Raven language introduction

During the scalar-Char integration, the author asked for feature boxes to link to
feature pages instead of repository samples and requested a Raven language page.
The assistant updated homepage links, added the language introduction with existing
executable snippets, and recorded the navigation rule. Raven’s own README and style
guide supplied the language context; target limitations remain explicit.

The author then clarified that readers should find user content, including try-it
instructions, on the website and favored the .rvnproj approach. The assistant added
a self-contained Preview 7 installation and project guide, moved complete feature
sample downloads onto the site, and made repository references optional. Newer APIs
remain labeled as development work without a published matching evaluator bundle.

The author emphasized that neoCLR and its guest programs have no .NET dependency;
the dependency belongs to surrounding tooling, including Raven Language Server
used by VS Code. The assistant made this explicit on the homepage, Raven page and
setup guide, and recorded the distinction in the website maintenance rules.

### Raven scalar Char integration — 2026-09-19

On continuation, the assistant completed the target-specific compiler/bridge slice:
scalar literals, patterns, array storage and classification work in Raven programs.
Execution testing exposed 16-bit array admission and boxed supplementary patterns;
both were corrected. Invalid Int32 scalar conversions fault at runtime, and the
surrogate predicates were retired. Wide numeric casts still narrow before scalar
validation; scalar String access remains separate. General lexer candidates remain
unextracted pending independent main-based validation. This is local development
validation, not a published preview or a complete text proposal implementation.

### Characters as text units — 2026-09-19

While the assistant was implementing scalar String access, the author questioned
whether the abstraction had gone too far toward encoding and said a character
should mean what a developer sees as a character. The assistant distinguished
scalars from grapheme clusters and paused the scalar public API. Asked whether
ordinary length, indexing and iteration should use graphemes, the author answered
“Yes—graphemes by default”.

The author then emphasized encoding-independent String and character abstractions,
with UTF-8 as the canonical encoding. The assistant recommended String as Unicode
text, Char as one extended grapheme cluster, explicit scalar access and encoding at
byte boundaries. It revised its interim suggestion of returning String from indexing:
that is useful implementation scaffolding, not the intended Char abstraction.
Char representation, normalization/equality and scalar naming remain open.

A temporary grapheme implementation passed nine Rust String tests but was preserved
outside the checkout instead of integrating its provisional public API. Earlier
scalar implementation history is retained. The independent expression-bodied
indexer emission bug exposed during this work was reproduced on Raven main's
ordinary .NET target, fixed with class/struct execution tests (31 focused tests
passed), integrated into main and cherry-picked to neoclr. The temporary Raven fix
branch was removed. See [the revised text abstraction](design/text-abstraction.md).

### Implementing the revised character model — 2026-09-19

The author asked for a recommendation based on other languages and environments.
The assistant recommended Swift-inspired grapheme characters with explicit scalar
and encoding access, and refined its earlier indexing recommendation: provide
iteration first and defer integer indexing. The author directed “Make it so” and
then emphasized an improved .NET-like experience fitting modern computing.

The assistant implemented owned grapheme Char storage, String Length/iteration,
explicit uint scalar traversal and UnicodeScalar classification. A Raven program
now executes combining sequences, emoji, literal patterns, arrays and interface
iteration on neoCLR. Runtime testing exposed the need to count Char payload bytes
in array limits; end-to-end testing exposed String interface receiver adaptation.
Both were addressed in this work. The implementation keeps ordinal equality,
Unicode 16 segmentation and snapshot iteration explicit; normalization, cursors,
a scalar value type and host/target literal Unicode-version alignment remain open.
See [the contract and evidence](design/text-abstraction.md). This records local
implementation, not publication of a new preview or website deployment.

The author then explicitly affirmed that a character is not a number format and
that numeric casts should not exist merely because of its backing storage. The
assistant confirmed that the implementation rejects those casts and keeps Unicode
numeric access explicit through GetScalars. This is an API principle, not just an
implementation limitation or a temporary missing conversion.

The author subsequently suggested future UTF-8 and ASCII string types for
encoding-specific functionality, while preserving String as the default neutral
text container and Char as an encoding-independent character. The assistant
recorded Utf8String/AsciiString as possible future specialized types, not additions
to this minimal implementation. UTF-8 remains the canonical runtime storage choice.

### Next preview release request — 2026-09-19

After the grapheme implementation, the author directed that a release should follow.
The assistant selected Preview 8 and a fresh experimental Raven .15 toolchain,
continuing the previous release's source matrix and macOS arm64 binary scope.
The compiler, runtime and website text slices were committed separately. Release
preparation uses an isolated clean checkout so existing local edits are preserved.
At this point, publication was pending exact-candidate CI and extracted-package validation.

### Task contracts before dependent APIs — 2026-09-19

During Preview 8 preparation, the author selected Task and async state-machine
contracts as the first priority after release, with runtime suspension as a future
step. The author asked for a modern .NET-like developer experience without legacy
constraints, identifying ConfigureAwait as one area to reconsider. The author then
clarified: “The rationale is that we need the Task contract for upcoming APIs”.

The assistant proposed treating completion, continuation scheduling and logical
context flow as explicit contracts, aiming to avoid routine per-await boilerplate.
The assistant recorded the priority and rationale in the async design, assessment
and website proposals. This selects the order of work, not a final scheduling policy
or a shipped Task implementation. Runtime suspension and the behavior of cancellation,
UI affinity, cleanup and context propagation still require design and validation.

### Conventional query terms before async — 2026-09-19

The author subsequently directed that LINQ-style operators use conventional terms,
rather than inheriting .NET method names, and placed this work before the async model.
The assistant recorded that revised order and recommended Map/Filter for the current
Select/Where operations. FlatMap and Fold were examples for future semantic review,
not claims of newly implemented operators. The selected direction is recorded in
[API policy](api-policy.md#query-operator-naming-direction-2026-09-19); Preview 8 retains
its existing operator names until the separate migration slice is implemented and tested.

The author confirmed initial capitals and explicitly requested method renaming after
release. The author then supplied a ChatGPT formulation: “Prefer terminology that
has converged across modern languages for fundamental iterable operations; retain
.NET terminology where it is already broadly conventional or materially clearer.”
The author highlighted Where → Filter, Select → Map and SelectMany → FlatMap as
strong candidates, while rejecting an automatic preference for Fold over Reduce or
Drop over Skip. The assistant adopted that principle: the immediate implementation
scope is the existing Where/Select pair; other operators require separate semantics.

### Preview 8 publication — 2026-09-19

The assistant published [Preview 8](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.8)
at neoCLR `07ecdbe` with Raven `ddf10eca` (.15). All six source CI jobs passed
1,280 tests each, and all validated archives matched the published source files.
The extracted package, editor and notice checks passed; GitHub's asset digests
matched the eight local uploads. The isolated VS Code grapheme workspace is available
locally and its language client started successfully. See the
[validation record](preview-8-validation.md) for scope and limitations.

Release content and installation instructions were updated on the website for a
separate manual publication. The subsequent Filter/Map migration is a separate
post-release slice; it does not modify the published Preview 8 artifacts.

The Preview 8 website was published through the separate manual
[Pages run](https://github.com/marinasundstrom/neoCLR/actions/runs/35456186059),
and the live installation page was checked for the .8/.15 asset set.

### Query operator rename after publication — 2026-09-19

After Preview 8 was published, the assistant renamed Where to Filter and Select to
Map in the Raven-authored API, reference declarations and importer bindings.
The implementation retains System.Linq and the existing iterator bodies. First,
Last, Single and ToList keep their names; FlatMap is a future direction, not a new
operator in this slice. There are no legacy aliases, so callers must rebuild with
the matching development references and runtime library.

Samples and the website now distinguish the new development names from Preview 8.
The new on-site example is compiled by the query suite and prints 10 and 30.
Execution, stale-name rejection, signature admission and completion are covered;
see [the query validation record](raven-query-api.md#validation-after-the-naming-migration).
No Raven compiler change was needed. Task/async contracts remain the next foundation.

### A basic iterable set before Task work — 2026-09-19

The author asked, “Can we add additional operators before doing the task work?” and
clarified, “So we have the basic set”. The assistant proposed Any, All, Count, Take,
Skip, Concat and FlatMap, plus explicitly seeded accumulation. Asked to choose its
name, the author replied, “Fold for seeded accumulation”. This selects a bounded
library slice before Task/async contracts, not complete LINQ compatibility.

The assistant implemented the proposed operators in Raven, extended the neoCLR
bridge for the new signatures and nested open collection arguments, and added
execution/lifetime tests and a runnable sample. The [query contract](raven-query-api.md#basic-operators-before-task-contracts--2026-09-19)
records .NET/Rust comparisons, normal disposal guarantees and deferred work.
Validation results are recorded there when completed. No Raven compiler change or
new release artifact is implied by this development slice.

While this slice was being validated, the author requested a subsequent port of
Raven.Core's Option and Result operator methods, with documentation and website
updates, before Task work. The author also requested a .NET-to-neoCLR LINQ mapping
table and an operator list or demonstrations for Option/Result. The assistant added
the query mapping table and queued the outcome-operator port as the next separate
slice; this note does not claim that port is complete.

### Outcome operators, extension declarations and imported cases — 2026-09-19

The author questioned why work continued in the Preview 8 worktree after release
and directed development onto main. The assistant transferred the pending outcome
changes to the original neoCLR checkout, preserving existing user edits; the old
release worktree was left clean. Raven's neoCLR branch remains separate.

The author then asked whether extension lookup was accepting ordinary classes
without ExtensionAttribute or a this receiver, and requested Raven extension syntax.
The assistant found that bootstrap C# declarations supplied this while the runtime
implementations were static Raven classes. Iterable and outcome implementations
now use public extension containers and self. The bridge validates the emitted
extension markers rather than relying solely on the reference declaration.

The author explained that importing System.Option.* and System.Result.* allows
plain Some, None, Ok and Error patterns, then requested that this be recorded in
the conventions. The assistant updated the guide in commit d5b7409 and used those
patterns in the new operators and executable samples. The library slices and
branch/callback checks compile and execute with these patterns.

The [outcome contract](raven-outcome-operators.md) records the bounded Raven.Core
port, deliberate adaptations and deferred APIs. This is development after Preview 8;
Task/async contracts follow this operator work.

The author then questioned whether the explicit OrElse lambda signature was needed
or merely demonstration. The assistant compiled the shorter form successfully,
simplified both OrElse callbacks, and documented the convention to prefer inference.
The Then callback still needs explicit types in the tested SDK; the assistant
recorded that limitation and related compiler candidates rather than presenting
those annotations as a language or API design requirement.

The author clarified that website examples and samples should show the language
as it is, without unnecessary explanatory annotations, while retaining those
actually needed. The assistant recorded this in both the Raven conventions and
website maintenance guidance. The query and delegate examples were simplified
and checked against their existing outputs; needed generic callback signatures
remain. This is a presentation convention, not a change to Raven's type system.

The author further clarified that these conventions also apply to the runtime
library and other code. The assistant broadened the explicit scope to all
hand-authored Raven code and updated AGENTS.md so future work follows the same
rules. Existing documented bootstrap ABI and intentional test exceptions remain;
this clarification does not claim a repository-wide code cleanup.

### Starting the provisional async mechanism — 2026-09-19

After the iterable/outcome slices, the author asked to continue. The assistant
re-read the Task proposal and inspected Raven's current builder and exception
lowering, then proposed an executable pending-completion model before compiler
integration. The author clarified that state machines are provisional, their
supporting contracts need not last forever, and compiler adaptation may be needed.
The assistant agreed to separate public completion behavior from replaceable
builder, awaiter and saved-state machinery.

The assistant added the [Raven completion experiment](experiments/task-contract/README.md),
which uses a single explicit executor and manual heap-owned resume objects. It
keeps expected errors in Result and documents the next compiler-integration steps.
This is an experiment, not a public Task<T> implementation or generated async/await
support. Cancellation, host-driven I/O, logical context and executor affinity remain
open. The final execution results are recorded with the experiment.

The author then emphasized that neoCLR lacks exceptions and async must work without
them. The assistant inspected Raven's generated catch and found that removing
SetException alone leaves the catch body intact. The proposed adaptation explicitly
omits the async exception wrapper for neoCLR, preserves Result completion values
and terminal Faults, and audits ordinary cleanup separately. This requirement is
recorded in the async design; no compiler implementation is claimed by this update.

The author clarified: “Result<T,E> is just a normal type in this model,” including
when returned from a task, so Task and failure remain separate. The assistant
made lowering explicitly uniform in T: no Result recognition, case inspection or
special completion path. Ordinary Result operations and ? remain separate language
semantics. The manual probe's Result match was identified as application logic,
and future integration validation now includes unrelated payload types.

### Propagation through provisional async lowering — 2026-09-19

The author emphasized that propagation is important to the developer experience
and asked to continue. The assistant confirmed that Raven already lowers ? into
early-return control flow before await rewriting, then added an explicit opt-out
for async exception capture on Raven's neoclr branch. The transformation does not
inspect Result or introduce a task-specific failure channel. Tests compare both
policies around immediate and pending awaits and check skipped side effects after
propagation. The .NET default remains intact.

A separate attempt to exercise Task<unit> with an explicit unit return produced
RAV2705 under both policies. The assistant recorded it as an independent compiler
gap, not as completed coverage. Public Task/builder contracts and end-to-end neoCLR
async remain outstanding; the exception-capture option is provisional compiler API.

The compiler adaptation was committed as Raven b99025680 on neoclr, with 61
focused and 119 feature-selected tests passing (overlapping sets). The assistant
also changed the manual neoCLR probe to use ? in its application transformation,
keeping Result case inspection out of its continuation machinery.

### Generic async unit correction — 2026-09-19

After the author's continuation, the assistant reproduced the unit-task gap on
Raven main using ordinary .NET metadata. Investigation also found missing generic
unit completion and an awaitless expression-body path bypassing async lowering.
The assistant fixed these on an isolated main-based branch, verified 145 selected
async/resource tests on .NET 11, and integrated the general fix into main and
neoclr separately (fb8eb77bb and b67d1e11b). No neoCLR-only policy was merged into
main. This closes the general compiler gap, not the guest Task/builder work.

### Imported union cases and retiring the old Error wrapper — 2026-09-19

The author pointed out that with `import System.Result.*`, a typed binding can use
`Error("Unavailable")` instead of repeating `Result<int,string>.Error(...)`, with
the same principle applying to Option and other imported cases. The assistant
agreed and tried the simpler spelling. The current reference library exposed a
collision with System.Error; narrower imports also lost extension visibility.
The assistant initially proposed documenting a temporary qualified spelling.

The author then said “We should probably not keep System.Error” and explained that
it predates the union convention. The assistant selected removal of the legacy
message wrapper and its runtime support, while retaining the Result.Error union
case and domain-specific error types. Simple message examples migrate to string
payloads. This avoids adding a compiler name-resolution policy to preserve an
obsolete API. Matching rebuilt references accept the imported Error constructor;
all nine outcome-operator checks pass. The website and conventions now describe
ordinary payloads and the breaking development migration. Validation includes 70
focused Rust tests, 251 signature checks, nine outcome checks, six String admission
cases, 25 saved-project checks, the ten existing completion-model scenarios and the
VS Code demo build/run. The wider Rust suite is still running at commit time; its
stale removed-type/helper-count fixtures were corrected and the affected groups
passed on rerun. This does not change published Preview 8.

### Minimal Task infrastructure — 2026-09-19

The author directed continuing Task work after retiring System.Error, then narrowed
the immediate goal to Task<T> and TaskCompletionSource<T> as a PoC that can expand
later. The author clarified that necessary supporting concepts are allowed: these
are the foundation for upcoming APIs. The assistant proposed a consumer handle,
producer completion authority, shared state and continuation dispatch, keeping
Result ordinary and state machines provisional.

The assistant implemented the generic types in Raven with an explicit TaskQueue,
private storage and internal cross-type implementation calls. Fifteen source-level
scenarios and four direct-IL tests pass, including unit, ordinary Result failure,
first-completion behavior, queued callbacks and retention through 38/39 collections,
including completion after the producing API has returned. A saved .rvnproj demo
builds and runs with the matching development toolchain for VS Code.
Compiler-generated async is not part of this result. The callback spelling and
expression-getter limitations are recorded for independent compiler investigation.
The website now describes this as development work after Preview 8.

The earlier Error-removal full Rust run finished with 1,271 passing tests and seven
failures: four stale Error fixtures, plus three UTF-8 tests that overlapped the
incomplete Task profile edit. All five affected test targets passed on rerun (the
four fixture targets within the 70-test focused pass, and all three UTF-8 tests).
This records combined validation, not a claim of an uninterrupted green full run.

## 2026-09-19 — Finish async before introducing threads

The author asked to continue Task work and, if practical, add a small Thread and
ThreadPool API with a sample. The assistant identified a choice between isolated
workers and shared guest objects: shared objects would require synchronization and
a thread-safe completion implementation. Asked which scope to pursue, the author
selected “Defer threads; finish async first.”

The active implementation milestone is compiler-generated async methods awaiting
pending Tasks and resuming through the existing explicit queue. Task and failure
remain separate: Result is an ordinary payload, and state-machine machinery stays
provisional. Thread and ThreadPool APIs are deferred. Compiler integration is in
progress; the existing completion PoC does not yet establish generated async
execution on neoCLR.

## 2026-09-20 — Task belongs to completion, not threading

The author directed finishing the active async slice and placing Task-related APIs
in `System.Tasks`, explaining that they are not tied to threading specifically.
The assistant adopted that namespace for Task, TaskCompletionSource and TaskQueue.
The compiler-facing state-machine interfaces and builder remain provisional
CompilerServices machinery. Threads remain deferred; Result stays an ordinary Task
payload rather than a task failure state. End-to-end validation is in progress.

## 2026-09-21 — Async execution and next platform slices

The assistant completed ten end-to-end generated-async scenarios, including pending
and completed awaits, GC across two suspensions, unit, Result propagation before
and after await, composition and queue scope failures. Fifteen completion scenarios
and 47 focused runtime checks pass. Raven's neoclr branch contains the target builder
integration (56543aecf); 38 focused compiler checks pass. The namespace is System.Tasks.
The runtime constructor-storage correction was committed separately as d062a04.

The author then asked for a small Thread and ThreadPool API after the Task PoC,
renaming System.IO to System.Storage in anticipation of storage providers, and a
new locally installed build. This supersedes the earlier deferral of threads after
async completion. The assistant accepted separate implementation slices; worker
isolation versus shared-object semantics remains to be settled before implementation.

The author selected “Isolated workers for the first PoC”, then emphasized choosing
a modern, familiar API without copying .NET for its own sake. The assistant chose
Thread.Start and ThreadPool.Queue with static text-to-text callbacks and a common
Worker result handle. Heap sharing and Task integration are excluded from this
worker slice; the implementation uses actual OS threads and a reusable bounded pool.

The author clarified: “Our goal is to be able to start a task on a new thread.”
The assistant revised the API to return Task<string> directly from both Thread.Start
and ThreadPool.Queue. Workers send owned text back to the caller; its queue completes
the local Task. The assistant identified blocking completion waits while pumping the
queue as a current PoC limitation, leaving nonblocking dispatch for later work.

The assistant renamed the current System.IO API and implementation folders to
System.Storage, following the author's storage-provider direction. The existing
File, Path and error contracts retain their behavior, including ConsoleReadError
from the former namespace. Provider selection remains future work; no provider
interface was invented as part of this rename. Published history remains unchanged.

The author emphasized that the API will be shaped further, that having Task from
the start should improve its integration with Thread, and that synchronous versions
remain a future design question. The assistant retained Task completion for this
PoC and did not add a parallel public synchronous API.

The author clarified that File is intended to use Task later, when implementing the
Storage model: StorageItem, File, Directory and storage providers. The assistant
kept this slice to the namespace migration and recorded asynchronous storage as
future work, without changing today's synchronous File operations.

The author corrected the error placement: ConsoleReadError belongs in System.
The assistant moved it there, while file errors remain in System.Storage.

The assistant installed a separate development build at
`~/.neoclr/experiments/tasks-storage-20260921`, with matching Raven compiler, language
server, importer, core reference and runtime. Its async, worker and storage .rvnproj
samples build through MSBuild. The worker sample prints “Hello, thread” and
“Hello, pool”; the async sample prints “Suspended” then 42; the file sample verifies
UTF-8 round-trip and size limits. Ten async and four worker source scenarios pass
against the installed runtime. Storage/console migration passes 27 runtime checks
and 257 signature checks, and the actual language server exposes Storage members
and System.ConsoleReadError. Raven project/heap-policy validation passes eight
checks; website generation and its focused tests pass. This is a local development
installation, not a published release or website deployment.

## 2026-09-21 — Promise-style Task composition

The author said the Task API should be reminiscent of a Promise API, with methods
for composing and continuing tasks, and need not use .NET terminology. The assistant
recorded this as API direction, preserving initial-capital method names and the
existing separation between Task completion and ordinary Result payloads. Exact
operator names and scheduling semantics remain open. No composition methods were
implemented or installed in this documentation slice.


## 2026-09-21 — Cancellation belongs to Task; establish the model first

The author requested CancellationToken support. The assistant proposed cooperative
requests with ordinary value/Result completion and asked whether cancellation should
also be a distinct Task completion state. The author replied: “Cancellation state
belongs to Task. Failure is modelled via Result.Error. Otherwise Faults.” The
assistant began cancellation plumbing and considered adding a cancellation branch
to Raven's current await lowering.

The author then directed reading the updated Task model proposal, emphasized that
Raven integration may require a separate lowering, and said: “Our primary goal is
the Task model.” The assistant reviewed the proposal and current Raven await path,
prioritized public State/Outcome and composition before lowering, and recorded the
[implementation sequence and gaps](task-model-alignment.md). The review identified
that the draft worker code could discard a completed value merely because a token
had been requested; that must change to explicit operation-level acknowledgement.

This assessment updates design and website direction only. Cancellation code remains
in progress and unvalidated end to end; no Raven cancellation lowering or new local
installation is claimed. The proposal remains preserved as supplied by the author.


### Core outcome implementation and Promise naming

The assistant started the runtime-first core slice: normal TaskState enum,
TaskOutcome<T> union, State/Outcome observation and first-terminal-transition producer
completion. Unfinished token/worker cancellation changes were preserved separately
while this contract was established. The author suggested considering the name
Promise instead of TaskCompletionSource. The assistant adopted Promise<T> for the
producer, exposing Task, Complete(value) and Cancel(), with Task<T> remaining the
consumer handle. This naming does not introduce Promise rejection or Result-aware
completion. Token requests, Map/Then and automatic await cancellation propagation
remain later slices.

The core slice passed 24 source scenarios, ten generated-async regressions, four
worker scenarios, nine direct runtime/access checks and 263 signature checks.
Cancelled outcome retention was also confirmed across 38 garbage collections. The
explicit outcome sample prints Cancelled, and library snapshot/API audits and
website checks pass. No cancelled-await lowering or new installed SDK is included.
The compiler/bootstrap pattern limitations found during validation are recorded
in the current Task contracts as deferred Raven candidates.


## 2026-09-23 — Task cancellation lowering and a default queue

The author asked to finish the remaining Task/async-await slices so APIs can be
built on them. The assistant found that Map/Then had been committed in 606a597 and
started the target-specific cancelled-await protocol. Result remains an ordinary
payload. The implementation uses an explicit compiler policy, never the name of
a Task type or the payload's union identity, to select cancellation propagation.

The author then requested a Default TaskQueue so constructing a Promise does not
require creating and passing a queue. The assistant proposed invocation-local
default dispatch, with the active explicit queue taking precedence. Default queue
construction is a subsequent slice; no automatic scheduler is implied.

Testing exposed two issues: routing cancellation through successful completion
required an invalid default Result, so the assistant separated the terminal exits;
and await in for loops skipped iterator disposal. The latter is explicitly diagnosed
pending suspension-aware loop lowering. Current behavior and validation are recorded
in [Task contracts](task-contracts.md#cancelled-awaits--2026-09-23).


### Automatic progress and familiar concepts

The author then suggested that a scheduler may already be needed for the PoC:
“We don't want to have to drain the queue manually.” The assistant adopted a minimal
runtime dispatch loop over the invocation default, keeping explicit queues for
controlled dispatch and deferring the public scheduler shape. The author restated
the purpose: take familiar .NET concepts, improve them to fit the framework's style,
and align them with the strategy for runtime suspension. This is a design criterion,
not a requirement to copy .NET names or freeze state-machine APIs.

The default-dispatch slice adds parameterless Promise construction and automatic
progress before invocation return. It retains queue identity and pending callbacks
through GC, and workers have separate dispatch state. External completion waiting,
async entry-point signatures and public scheduling policies remain open. See
[the current contract and comparison](task-contracts.md#default-dispatch--2026-09-23).


The author emphasized explicit awaitable operations: creating and awaiting a thread
makes the operation clear, while Tasks are not inherently built on threads. The
assistant clarified that the current Thread.Start returns Task<string> for worker
completion, and recorded I/O and stream operations as future awaitable producers.
A first-class awaitable thread handle is not implemented by this exchange; the
current source spelling is await Thread.Start(...). Task completion and worker
execution remain distinct responsibilities.


The author corrected TaskQueue.Current() and RuntimeServices.CurrentTaskQueue():
context lookup should be expressed as a property. The assistant changed both
Raven-facing contracts, kept the native transport function behind the getter,
and recorded the convention for runtime code as well as examples.


### 2026-09-23 — Provisional API boundary, Result composition and editor use

- **Author:** clarified that the goal is a provisional contract good enough to
  build APIs on, "At least for the Task API surface". Requested that the finished
  work be testable in VS Code and suggested including MapResult extensions.
- **Assistant:** kept Task/Promise, completion and cancellation outcomes,
  composition and working async/await as the boundary; queue and state-machine
  implementation remain provisional. Added explicit Task<Result<T,E>>.MapResult
  with queued Ok mapping, unchanged Error and cancellation bypass. Planned a
  matching local development toolchain and runnable .rvnproj examples, with a
  Tasks feature page clearly distinguishing development from published Preview 8.
- **Open:** cancellation requests/tokens, host event progress, protected cleanup
  and broader suspension support; this exchange does not settle scheduler design.


### 2026-09-23 — Propagation and await precedence

- **Author:** asked whether propagation works in async methods, then asked how
  `await Foo()?` should behave. Suggested a special ergonomic interpretation
  equivalent to `(await Foo())?`, but explicitly said it is not the priority.
- **Assistant:** verified current postfix-first parsing and the existing passing
  separate-statement propagation tests. A combined `(await input)?` experiment
  exposed an uninitialized carrier field in the generated state machine. Fixed
  the temporary lifetime in general lowering and added immediate/resumed Ok,
  Error and cancellation regression cases. Kept parsing precedence unchanged.
- **Direction, not implemented:** revisit await/propagation shorthand as a language
  design question after the provisional Task surface and local editor experience.

- **Validated outcome:** six combined-expression cases and two separate-statement
  cases pass on neoCLR. Raven main received only the independently tested general
  fix (c51c69bad, 21 tests); neoclr retains its separate policies (e56fc1ddf,
  40 tests). The temporary main-fix branch was removed after fast-forward integration.


### 2026-09-23 — Task feature page and matching local development tools

- **Assistant action:** added the Tasks feature page with tested worker, Promise,
  cancellation, MapResult and awaited-propagation examples, complete website-hosted
  downloads, current limits and future direction. Marked it development after
  Preview 8; no publication or new public release is implied.
- **Local outcome:** prepared `~/.neoclr/experiments/tasks-20260923/Tasks.code-workspace`
  with five .rvnproj folders, matching compiler, reference library, importer,
  runtime and Raven Language Server. Each saved project builds and runs with the
  documented output. Stdio LSP checks expose Task/Promise/MapResult and the Default
  property. The server build allows existing nullable warnings; those unrelated
  warnings were not changed. The user's installed release selection is unchanged.
- **Validation:** website build/link checks cover 11 pages; three Python checks and
  the highlighting test pass. Desktop layout was visually inspected. GUI interaction
  was interrupted by the user's activity; build/run and editor protocol verification
  are completed, but an interactive VS Code Run Task session is not claimed.


### 2026-09-23 — HTTP applications as the first major milestone

- **Author:** asked to process the proposals, identify urgent needs (including
  exploration without commitment), and create/update the roadmap. Directed a minimal
  POC with HttpClient and HttpServe/HttpListener APIs, needed socket abstractions,
  streams, encoding and a small JSON API: “This will be what we use to justify the
  runtime.” Allowed supporting changes and requested gradual, smaller API cases.
- **Assistant proposal:** make two Raven apps on neoCLR exchange UTF-8 text and JSON;
  start with controlled loopback HTTP, and develop delayed I/O completion, memory
  streams, chunked text decoding, JSON, TCP echo and independent HTTP peers first.
  Prioritize external progress, buffer/handle ownership, cancellation and cleanup.
  Record conflicts between older stream notes, the revised Streams proposal and
  explicit-token Task cancellation. Keep server naming and backend choices open.
- **Action:** added the [HTTP POC roadmap](http-poc-roadmap.md), triaged all current
  proposal families, linked existing implementation evidence and primary comparisons,
  and aligned roadmap/API-plan entry points and the website proposals overview.
  Original proposal text, including the existing Streams working edit, is preserved.
- **Outcome and open work:** this is a planning change; no socket, stream, encoding
  or JSON implementation is claimed. The author selected the milestone; the detailed
  slice order, loopback envelope and API alternatives are assistant proposals pending
  executable evidence, not individually approved platform contracts. No release date
  or production networking commitment was selected.

- **Author clarification:** the proposals are not necessarily consistent and are
  “Not showing what it necessarily should look like in the end”.
- **Assistant adjustment:** explicitly treat proposals as inputs about needs and
  alternatives, not final specifications. Do not prefer a newer proposal merely for
  its recency or combine all proposed abstractions. Select eventual API shapes from
  small application experiments; retain, reshape or discard ideas based on evidence.


### 2026-09-23 — Unified platform roadmap through sample products

- **Author:** requested a unified `platform-roadmap.md` next to the HTTP plan,
  with milestone themes and concrete products/samples based on the proposals.
- **Assistant proposal:** retain the HTTP pair as M1, then consider a File Catalog,
  Download Queue, Activity Report, Assembly Explorer and Portable Sample Pack. Each
  has smaller runnable checkpoints, completion evidence and a .NET/CLR comparison.
  Put broader type-system, dynamic, metadata and execution ideas in bounded research
  products whose outcome may be to keep existing mechanisms.
- **Action:** created the [platform roadmap](platform-roadmap.md), retained the
  [HTTP roadmap](http-poc-roadmap.md) as its detailed first-milestone plan, and updated
  planning indexes and the website proposal overview to distinguish their roles.
- **Status:** documentation/planning only. The later themes, products and dependency
  order are assistant proposals, not individual author commitments or implemented
  samples. The earlier clarification that proposals do not prescribe final API
  shapes remains in force.

- **Author follow-up:** “Make the platform-roadmap authoritative for our work, unless
  we direct work in another way.”
- **Assistant action:** made the platform roadmap the default authority for priorities,
  sequencing and scope in both the roadmap and AGENTS.md. Explicit author directions
  override it. Detailed plans remain subordinate; a one-off task outside M1 does not
  automatically reorder the roadmap or turn candidate milestones into commitments.


### 2026-09-23 — Prefer progressive API cases before networking

- **Author:** asked whether sockets and HTTP client should be down-prioritized,
  allowing features and existing APIs to evolve progressively. Asked that the roadmap
  avoid starting with the most complex work unless it reveals useful runtime issues,
  such as garbage collection. Expressed openness to rearranging the list.
- **Assistant recommendation and action:** retain HTTP as the first major application
  destination but revise default delivery to Byte Copy, Text/JSON Transformer, Delayed
  Copy with actual guest GC, File Transformer, TCP Echo and then the HTTP pair. Move
  S1 memory copy ahead of further S0 integration; record the existing host probe as
  partial evidence rather than a reason to rush into sockets. Keep checkpoint APIs
  revisable as each new consumer tests their assumptions.
- **Status:** this is the assistant's revised working sequence in response to the
  author's question, not approval of each eventual contract. The isolated host probe
  has seven passing checks and is committed at `629cbf4`; guest GC/Task integration
  and the new sample checkpoints remain open. The roadmap was committed at `6634b79`.


### 2026-09-23 — Foundational primitives with explanatory samples

- **Author:** directed continued work on primitives and foundational runtime behavior
  first, while producing comprehensible samples that convey context and use. Later
  requested a dedicated Arrays website page comparing neoCLR arrays with .NET.
- **Assistant action:** built an application-local Byte Copy sample using ordinary
  managed arrays, checked ranges and concrete short-read/short-write endpoints. A
  greeting transfer explains why byte pipelines need partial-transfer handling.
  Added range/alias, failure and real synchronous GC-retention checks; retained
  runtime responsibility for bounds, element compatibility and tracing rather than
  inventing new instructions without evidence of a gap.
- **API limits:** this is a provisional library experiment, not public stream
  contracts or asynchronous/native buffer lifetime support. Whole-pipeline aliasing
  remains a stated precondition; the individual copy supports overlap. Existing
  array tests also cover direct IL checks below the frontend.
- **Website action:** added an Arrays page with a tested readings/snapshot example,
  expected output, .NET comparisons and current limits; linked it from the homepage
  and Collections. The site remains unpublished by this work.
- **Deferred finding:** an infinite-loop copy function without a fallback return
  compiled to an unsupported InvalidOperationException reference on the local
  target. An explicit EOF loop avoids it. A general Raven fix needs an isolated
  CLI/.NET reproduction; no Raven repository change was made.


### 2026-09-23 — Technical website and an open project overview

- **Author:** said the website sounded too much like sales copy. Directed it to
  explain the current project, features, relevant .NET differences and direction.
  Added that general information should live on the webpage, with GitHub docs used
  only when detail warrants them; the site should feel capable of becoming a
  community project. Requested an overview of the project's background and aims.
- **Assistant response and action:** revised homepage and feature-page framing,
  replaced the stale broad next-release pitch with the active foundations-to-HTTP
  roadmap, and added an on-site overview covering background, .NET/Raven relationships,
  current limits, goals and participation. Kept general navigation on the website
  and retained optional technical records. Described concrete ways to contribute
  without claiming an established community or settled API design.
- **Scope:** editorial and navigation changes; executable sample sources and runtime
  behavior are unchanged. The website maintenance guidance now records these author
  preferences. Local validation does not constitute publication.
- **Validation:** all 13 pages build with internal links checked; tokenizer and
  website tests pass. Inspected the homepage and new overview at desktop and narrow
  sizes. No site publication was performed.


### 2026-09-23 — API documentation alongside the website

- **Author:** requested API documentation generated from metadata and XML docs using
  DocFX, published alongside the website at `/docs/`, with real descriptions to make
  it useful. Subsequently clarified that this release needs “good enough
  documentation” for an API overview, not exhaustive coverage.
- **Assistant response and action:** added a broad on-site API index and an initial
  generated Task, Promise, TaskQueue and TaskState reference. Descriptions currently
  live in an authored XML sidecar; the signatures come from the compiler reference
  assembly. Added DocFX to the combined website build and Pages artifact, with
  navigation from all site pages. Documentation completeness does not delay the async
  preview beyond this overview and main-API scope.
- **Finding and tradeoff:** DocFX 2.80.1 cannot render the neoCLR-specific
  `Func<System.Void>` callback signatures. Three methods are documented on a linked
  Raven-notation guide instead. The generated declarations are clearly labelled
  metadata notation; no runtime types were changed to accommodate DocFX. A checked-in
  metadata snapshot lets website CI build without a Raven source checkout.
- **Outcome:** local DocFX build and existing website checks pass. Publication uses
  the existing manual main-branch workflow and has not occurred in this work.
  Broader member coverage and automatic XML emission from Raven source comments
  remain future work.


### 2026-09-23 — Keep the changelog after release

- **Author:** stated, “We are going to keep a Changelog after this release”.
- **Assistant:** confirmed that `CHANGELOG.md` remains a permanent record: published
  entries stay frozen and later work continues under Unreleased. Recorded this
  explicit continuation in the changelog workflow; existing per-commit requirements
  remain in effect. No release publication or changelog reset was performed.


### 2026-09-23 — Release completion and website navigation

- **Author:** directed continued release work. While validation ran, requested fewer
  website menu items after release, removal of “Experimental” from the API site's
  neoCLR branding, and showed a screenshot where the icon touched the name.
- **Assistant response and action:** kept Preview 9 fixed at `834028c` while preparing
  the website changes separately. Published the prerelease after all six exact-commit
  source jobs and the extracted package/editor gates passed. Preserved the source
  archive, checksums, manifest and supporting logs as release assets.
- **Website work:** prepared five common menu destinations (About, Features, Raven,
  API docs and Try it), plain neoCLR API branding and an eight-pixel logo gap at the
  default font size. The main site retains its experimental platform description.
  Rendering under the actual `/neoCLR/` base path exposed root-relative DocFX links;
  the builder now adjusts both HTML and navigation JSON, with a regression test.
- **Validation:** package scope remains macOS arm64; source validation covers Linux,
  macOS and Windows on stable and Rust 1.85. Website/DocFX builds and link/highlighting
  checks pass locally. Desktop and narrow navigation and API header were inspected.
  The separate [manual Pages deployment](https://github.com/marinasundstrom/neoCLR/actions/runs/35876887523)
  published website revision `5bffbca`. Public homepage and `/docs/` checks confirmed
  the Preview 9 status, five-item menu, plain API brand and logo spacing.
- **Next:** the release checkpoint is complete. Streams, Storage and Encoding precede
  networking; the selected post-release concurrency direction remains future work.


### 2026-09-23 — Make the next release's CI efficient

- **Author:** directed improving CI for the next release, explicitly “not now”. Said
  repeating every part and entire samples for every platform takes too long; isolate
  platform-specific behavior instead.
- **Assistant response:** proposed one comprehensive shared validation run, focused
  host-boundary tests, separate minimum-Rust compatibility and a small cross-platform
  smoke set. Recorded [a plan](ci-efficiency-plan.md) with measurement and coverage
  criteria rather than changing the active release or workflow.
- **Status:** planning only. Preview 9 passed the existing six-job matrix. Job
  classification, duplicate removal and the revised matrix remain future work.


### 2026-09-23 — Stop documentation pushes starting full runtime CI

- **Author:** observed that pushing the docs changes had started full runs and said
  this should be avoided. This is an immediate correction to triggers, separate from
  the broader next-release CI redesign previously deferred.
- **Assistant action:** cancelled the redundant docs-push and release-tag runs;
  added documentation/site path exclusions to runtime push/PR triggers and made
  automatic pushes branch-only. Explicit manual candidate validation remains
  available, and executable experiments still trigger the unchanged six-job matrix.
- **Validation:** actionlint passed for both workflows. The documentation closeout
  uses a normal commit, without a skip marker, to check the new path filtering.
  Published release validation and website deployment were not cancelled.


### 2026-09-23 — Working streams before Storage alignment

- **Author:** “Once we have working read write operations with streams then we can
  start aligning the Storage APIs. Even implement the Path value object”.
- **Assistant response:** agreed to finish and validate stream reads/writes first,
  then align Storage using that evidence and consider host paths separately from
  provider-specific addresses. No final Path contract was proposed or accepted.
- **Action:** implemented directional, blocking FileInputStream/FileOutputStream
  wrappers with typed errors, hidden native handles and explicit Close. Connected
  them to the experimental provider sample; the same byte workflow runs against
  disk and deliberately short-transfer memory streams. Added process-unique native
  IDs so retained streams cannot alias another invocation's resources.
- **Evidence:** [sample and verifier](experiments/storage-provider/README.md) and
  [on-site stream API guide](../api-docs/streams.md). The Raven MSBuild sample passed
  with real disk-byte assertions. Storage alignment and the Path value object are
  next; asynchronous I/O, buffer ownership across suspension and Task.Run scheduling
  remain separate open work. This is development work after Preview 9.


### 2026-09-23 — Runtime-owned terminal fault codes

- **Author:** asked whether known runtime faults have codes, and directed adding
  them if absent, giving StackOverflow as an example. Subsequently specified that
  faults triggered from user code need a distinct code and users must not set it.
- **Assistant finding/proposal:** Fault had only diagnostic text, location and stack
  trace. Proposed a machine-readable classification assigned at detection sites,
  independent of message text, with UserFault for explicit guest requests.
- **Action:** added host FaultCode/Fault.code, specific codes for selected known
  runtime failures and a RuntimeError fallback, plus debugger `fault_code` and CLI
  display. System.Fault(message) and the fault instruction always use UserFault;
  their guest signatures do not accept a code. Added on-site reference and tests.
- **Scope:** no guest exception hierarchy, catch/recovery semantics or HRESULT
  compatibility. More precise classification of remaining RuntimeError diagnostics
  can follow; Storage alignment and Path remain the next platform slice.


### 2026-09-23 — Validated Path values and string overloads

- **Author:** directed continuing directly into the next slice after fault codes.
  Clarified that static Path.Parse should return Result<Path, ...> and ensure valid
  syntax. Said further Path operations can come later and expressed uncertainty
  about requiring Path versus offering string overloads “where a path is expected”.
- **Assistant response:** selected Parse returning Result<Path, InvalidPathError>
  and private construction. Initially proposed a Path core contract with convenience
  overloads considered at application boundaries; after the author's clarification,
  recommended Path and string overloads on path-taking APIs, with strings parsed
  and forwarded to the typed implementation. This remains an assistant recommendation,
  not an author decision to implement an overload family now.
- **Action:** implemented a small application-owned immutable Path class, with
  validated spelling, absolute/relative indicators and explicit lexical Equals.
  Directory parses child paths; provider operations consume Path values and the host
  provider maps them under an explicitly configured native root. More Path operations
  and platform-wide API promotion are deferred.
- **Provisional tradeoff:** use a conservative slash-separated logical grammar rather
  than claim a universal native path parser. This revises the earlier experiment's
  provider-specific string spelling: syntax is now shared while resolution stays
  provider-owned. Constructor privacy protects ordinary Raven use, not arbitrary
  forged IL; provider root mapping is not a symlink-safe sandbox.
- **Evidence:** [sample and verifier](experiments/storage-provider/README.md),
  [complete sample API reference](../api-docs/storage-experiment.md#path-value-object).
  Explicit parsing remains visible in the sample so overload ergonomics can be judged
  before finalizing the Storage contract.


### 2026-09-23 — Path is a Storage value, not a system-wide requirement

- **Author:** said it is wise to keep the path primitive as something Storage uses,
  rather than imposing it everywhere. Callers may use it to verify/parse a path but
  still pass a string to certain APIs.
- **Assistant:** agreed to distinguish the Storage value from other APIs' string
  parameters. Suggested optional parsing followed by passing path.Text; this is not
  a decision to add implicit conversions or provider string overloads.
- **Action:** preserved string parameters on file streams and the new host metadata
  wrapper while adding Path-taking GetFile to the application-owned Storage provider
  experiment. Updated the roadmap and on-site reference with this boundary.
- **Open:** exact grammar, convenience overloads within Storage, provider identity,
  directory resolution and asynchronous lookup remain under exploration.


### 2026-09-23 — Prioritize integrating the working Storage slice

- **Author:** directed, “We want to integrate this as soon as possible”.
- **Assistant:** interpreted this as integrating the working Storage APIs into the
  platform library, with the disk/memory program becoming a consumer rather than
  continuing to expand an isolated experiment. Deferred the newly proposed memory
  hierarchy slice in favor of integration.
- **Action:** moved the tested Path implementation into System.Storage.Path,
  preserving static Combine/GetFileName string helpers. Changed the sample to import
  the platform value and added generated member reference coverage. No Raven
  compiler or Runtime Contract policy changes are needed; the existing reference
  catalog and library authoring mechanism are extended.
- **Remaining:** provider/File/Directory and stream capability integration follow in
  bounded slices. Current hierarchy and replacement limitations stay explicit;
  this does not select a final API or require every remaining question to be settled.


### 2026-09-23 — A future Uri value can follow the same boundary

- **Author:** said a similar situation will arise with a future Uri class.
- **Assistant:** agreed that a structured validated value can be useful without
  forcing every URI-taking API to require it. Parsing, relative references and
  URI-specific operations should be evaluated near networking work.
- **Action:** recorded this as a future design comparison, not an implemented Uri
  API or a reprioritization away from current Storage integration.


### 2026-09-23 — Future Unix and Windows Path parsing

- **Author:** directed that the future parser accept Unix and Windows formats and
  normalize them through the Path object.
- **Assistant:** recorded this as the target direction rather than treating today's
  narrow slash grammar as final. Identified format selection, drive/UNC roots,
  separator handling and equality under normalization as contracts to define.
- **Action:** updated the roadmap and on-site Path direction. No new parsing format
  or normalization behavior was implemented in this integration slice. Existing
  Parse preserves accepted spelling; native string helpers retain host rules.
- **Open:** no format-selector signature or automatic host-format detection was
  selected. Normalization is not filesystem lookup, authority or stable identity.


### 2026-09-23 — Keep Path helper expansion outside the minimal Storage POC

- **Author:** suggested future Combine and other Path helper overloads that accept
  Path objects, while explicitly asking to keep the current work minimal. Identified
  the Storage API POC as the goal.
- **Assistant:** retained the current string helpers and recorded Path overloads as
  direction only. Kept this integration focused on the existing validated value and
  working read/write/lookup consumer, not additional formats, helpers or Uri APIs.
- **Remaining:** integrate the existing provider surface in bounded slices; no
  overload signature or normalization behavior was selected by this exchange.


### 2026-09-23 — Storage POC remains the product checkpoint

- **Author:** reaffirmed: “The goal is to have a POC Storage API in place that can
  demonstrate the API and file access.”
- **Assistant:** scoped the checkpoint around a runnable app obtaining a directory
  from a provider, resolving or creating a file, and reading/writing through streams.
  Richer metadata and broader storage features remain later work.
- **Action:** made that completion evidence explicit in the roadmap. Provider
  GetDirectory now returns the Directory interface and the sample obtains its root
  through lookup. The disk/memory product and directory contract checks pass.
- **Remaining:** finish the minimal integrated provider surface; concrete disk/memory
  providers are still sample-owned. This is not a claim that the full proposal is done.


### 2026-09-23 — StreamReader for a comprehensible Storage sample

- **Author:** suggested that the platform may also need a StreamReader class.
- **Author clarification:** there will also be a TextReader interface.
- **Assistant response:** consumers use TextReader; StreamReader implements it over
  InputStream, allowing future text sources without a byte-stream dependency.
- **Assistant proposal:** a minimal UTF-8 reader over InputStream with bounded
  reads, explicit ownership and typed errors, shared by disk and memory consumers.
- **Action:** recorded the .NET comparison and concrete contract questions in the
  Streams proposal and linked it from the POC roadmap. No StreamReader API has been
  implemented in the directory-lookup slice; namespace, signatures, BOM handling
  and exact lifetime/error policies remain to be selected and tested.


### 2026-09-23 — Seekability as a separate stream capability

- **Author:** asked whether to implement a provisional ReadOnlyAccessStream
  interface to allow seeking.
- **Assistant recommendation:** use the proposal's SeekableStream capability
  alongside InputStream; consider a combined SeekableInputStream only if needed.
  Keep text readers usable with sequential streams and define buffer/decoder reset
  behavior before adding text-reader repositioning.
- **Action:** recorded the alternatives, .NET comparison and seek-and-reread
  validation scenario in the Streams proposal. No seek API was implemented and no
  author decision on the recommended name or signature is assumed.


### 2026-09-23 — Complete the Storage POC across the specified slices

- **Author:** suggested System.IO for readers and moving streams there too; then
  reaffirmed that presenting a POC is the first objective and all specified work is
  part of it. Asked to continue through the slices until done.
- **Assistant:** recommended System.IO for byte streams and text readers while
  retaining System.Storage for providers/items/paths. Committed to completing
  provider integration and enumeration, namespace alignment, text readers and a
  small seekability case in a coherent sample, validating each slice.
- **Current action:** consolidating provider resolution on StorageProvider with
  GetItem/GetFile/GetDirectory. Concrete implementations remain behind interfaces.
- **Remaining:** the roadmap now explicitly includes those pieces in the POC exit
  evidence. The exact seek signature and text-reader bounds/ownership policies still
  need implementation decisions and tests; completion has not yet been claimed.


### 2026-09-23 — Future asynchronous Storage contracts

- **Author:** asked whether some future Storage methods should be asynchronous and
  return Task.
- **Assistant recommendation:** consider Task<Result<…>> for lookup, listing and
  stream opening/creation, retaining synchronous parsing and represented properties.
  Scheduling, cancellation, ownership and closing races must be defined first.
- **Action:** recorded this as future direction. The active POC remains synchronous;
  no completed-task wrapper or automatic thread spawning is being presented as async I/O.


### 2026-09-23 — Storage POC implementation outcome

- **Assistant action:** integrated System.IO byte streams and the TextReader interface,
  a bounded UTF-8 StreamReader with explicit input ownership, and a separate
  SeekableStream capability on file input. Kept System.Storage for providers/items/paths.
- **Observed outcome:** the standalone platform-only Storage POC compiles, verifies
  and runs, producing checked file bytes and output for write/read, seek/reread,
  mixed enumeration, traversal and expected errors. Source, walkthrough and API
  reference are on-site build inputs; no website publication is claimed.
- **Implementation findings:** the bridge needed reader local admission and Boolean
  constructor coercion. The current compiler requires an object view for checked
  discovery of an unrelated stream interface. No Raven repository changes were made.
- **Still open:** async Storage signatures, scheduling/cancellation, metadata query
  shape, incremental readers/enumeration and broader providers. The author's async
  question is recorded as exploration, not a decision to change POC signatures.
- **Evidence:** [product](experiments/storage-poc/README.md),
  [contracts](experiments/storage-provider/ReaderContracts.rvn),
  [API guide](../api-docs/streams.md), [roadmap](platform-roadmap.md).


### 2026-09-23 — Scheduling and provisional TaskQueue

- **Author:** asked whether a scheduler concept is needed; explained that TaskQueue
  was a temporary way to build the Task interfaces/contracts and that the model
  must adapt to future needs, including runtime suspension.
- **Assistant proposal:** distinguish completion, work scheduling and continuation
  dispatch, initially as internal contracts rather than a public Scheduler API.
  Proposed checking the worker adapter's queue limitations.
- **Author clarification:** “I don't say that we should keep it, we will update the
  model when there is a need.” This leaves both retention and replacement open.
- **Action:** documented current Promise/builder ownership and added an executable
  cross-queue characterization fixture. Updated the roadmap and on-site explanation;
  no runtime or public API model is changed by this slice.
- **Remaining:** select ownership/progress rules when a concrete consumer needs them;
  runtime suspension, explicit-queue worker submission and public scheduler shape
  remain open. The assistant's initial early-rejection approach was not implemented:
  Raven queue-reference comparison encountered an unsupported importer comparison,
  and this slice instead characterizes behavior without expanding the provisional API.
- **Observed outcome:** the cross-queue fixture passes with the expected four lines;
  existing cancellation/sibling, GC, UserFault and bootstrap-boundary checks pass.
  API snapshot and combined website checks pass; no publication is claimed.
- **Evidence:** [contracts](task-contracts.md#scheduling-and-suspension-exploration--2026-09-23),
  [fixture](experiments/worker-task-cancellation/Affinity.rvn).


### 2026-09-23 — Console and standard streams

- **Author:** directed focus to common Console methods and stream support through
  Console.In, Console.Out and Console.Error. Clarified: “Console should stay a class”.
- **Assistant action:** implemented System.Console as a static class, with text
  reader/writer properties and separate byte-stream factories. Added bounded line
  reading and a reusable UTF-8 StreamWriter, plus a greeting/diagnostic sample.
- **Design choices:** keep calls synchronous; distinguish EOF from an empty line
  through Option, preserve stdout/stderr separately and close only the wrapper for
  process channels. Fresh wrappers, fixed UTF-8 and LF/CRLF line parsing are explicit
  provisional limits; no encoding, redirection or scheduler API is added.
- **Remaining:** runtime suspension, broader terminal controls, lone-CR line endings,
  host-specific byte output for workers/debugger and more efficient input buffering.
- **Evidence:** [Console guide](../api-docs/console.md),
  [sample and checks](experiments/console-streams/README.md).


### 2026-09-23 — Showing Result/Option ergonomics through Console

- **Author:** proposed Console.ReadLine followed by `?` and an optional binding,
  so errors propagate without an explicit Error branch. Confirmed that the else
  branch must return before input is used afterward. Requested a website section
  on Raven error/Result handling and samples that prefer these forms over match
  unless matching demonstrates a useful case distinction.
- **Assistant action:** added guard and if-let examples to the Console sample and
  an error/optional-result section to the existing outcomes feature page. The current
  compiler supports `let Some(input) = ... else` for the linear guard and
  `if let Some(input) = ... { ... } else { ... }` for a branch. The initially
  suggested `if let input = ... else` spelling was rejected by this compiler;
  examples use the supported explicit Some pattern instead.
- **Behavior to demonstrate:** `?` propagates decoding/I/O failure through a compatible
  Result return type; the optional binding handles EOF, not failure. An empty input
  line remains Some(""). The application boundary reports the propagated error once.
- **Evidence:** [guard sample](experiments/console-streams/Propagation.rvn),
  [if-let sample](experiments/console-streams/IfLet.rvn),
  [on-site guide](../api-docs/console.md#propagation-and-optional-input).

- **Observed outcome:** both pattern samples passed all four input cases. Native
  byte-output tests (three), greeting cases and reader/writer contract checks passed.
  The dedicated Console feature page links these tested examples and the on-site
  API guide; samples are bundled for download. The generated API snapshot covers
  371 items. The older raw-System Console tests are blocked by that library's
  unresolved StorageItem dependency, independently of the composed Raven library
  used by the new checks.

### 2026-09-23 — Consistency review and Object/Value follow-up

- **Author:** requested continued useful work with reassessment of what should be
  adapted from .NET, redesigned or removed. Then suggested focusing on Object's
  missing behavior and the role of System.Value now that Object exists. Confirmed
  unfinished work should be completed before tackling that review.
- **Assistant finding:** the roadmap still ended with already-completed release
  gates. A Raven-only StorageProvider contract in the legacy manifest also caused
  unresolved StorageItem failures in otherwise unrelated Console tests.
- **Action/outcome:** moved that include to Raven library composition, restored all
  11 legacy Console tests and corrected the roadmap's immediate checkpoint.
- **Next:** review Object and Value independently of this repair. Initial source
  inspection shows Value is an erased payload facility, not the Object base class;
  no removal or universal Object equality policy has been selected.

### 2026-09-23 — Harmonize reference and value semantics with .NET

- **Author:** “We should harmonize what we can with .NET. The reference type and
  value type semantics is imporant.” This follows the request to assess Object's
  missing behavior and System.Value's role, while completing unfinished work first.
- **Assistant action:** completed and committed the composition repair first. Then
  compared Object, Value, boxing and nominal class/value assignment, added a pinned
  .NET 10 baseline probe and documented actual public API coverage. Object.GetType
  is executable; Equals/GetHashCode/ToString remain reference scaffolding.
- **Revised direction:** supersede older value-like Object inheritance and universal
  structural Object equality proposals. Use class reference sharing, shallow value
  copying and explicit boxing as the baseline. Recommend overridable Object.ToString
  first, then identity/equality/hashing with their .NET contracts considered together.
- **Kept open:** Value removal requires carrier/native/host migration; Object does
  not automatically supply its copy/extraction semantics. No new Object runtime
  methods, hash policy or storage replacement were implemented in this review.
- **Evidence:** [review](object-model-review.md),
  [.NET baseline](experiments/object-baseline/README.md),
  [on-site reference guide](../api-docs/objects.md).

### 2026-09-23 — Object display and an abstract root

- **Author:** asked to continue the reviewed slices, then proposed making Object
  abstract: “If we ever want to declare a lock object we will find another way.”
- **Assistant action:** implemented the class ToString fallback/override slice and
  adopted an abstract Object root. The source/reference constructor is protected;
  derived construction chains to an empty runtime entry. Direct construction is
  rejected. This deliberately differs from .NET's instantiable Object.
- **Implementation finding:** preserving Object ancestry also requires preserving
  its base-constructor call, previously omitted for the rootless projection. Virtual
  calls and explicit base calls must select different adapters. Abstract Object must
  not cause its concrete default methods to disappear from reachability analysis.
- **Limits:** boxed-value/string virtual formatting, equality/hash and Value migration
  remain separate follow-ups; no lock primitive or generic synchronization object
  was added. [The display sample](experiments/object-display/README.md) and raw tests
  record the executable scope, including negative construction/dispatch cases.

- **Validation:** 21 dispatch/construction tests, the source sample and negative
  abstract-construction check, three existing reflection/inheritance samples,
  generated library/API checks and the combined website build passed.

### 2026-09-23 — Identity prerequisites after Object display

- **Author:** asked to continue with the next slice after the abstract Object work.
- **Assistant action:** committed the bounded display slice, then compared identity
  prerequisites with the pinned .NET 10 baseline. Added raw-artifact scenarios for
  casts/aliases, mutation and GC, arrays, boxes, typed nulls and heap identity scope.
- **Finding:** String-to-Object conversion creates a wrapper; repeating the conversion
  or roundtripping through String loses its identity. This differs from the .NET
  baseline. Diagnostic allocation IDs also restart per execution and must not be
  treated as globally unique object identities.
- **Proposed next implementation:** keep ReferenceEquals separate from overridable
  equality, implement class Equals/GetHashCode together, and explicitly resolve the
  String representation/scope before claiming general Object identity. This is an
  assistant recommendation; no further author decision is recorded here.
- **Outcome:** characterization and documentation only; no equality API, hashing
  algorithm, String storage replacement or System.Value migration in this follow-up.
  See [the review](object-model-review.md#identity-prerequisites--2026-09-23).

- **Validation:** all six new runtime identity cases, four existing reference-identity
  tests, 14 .NET assertions and the combined website build passed.


### 2026-09-23 — Possible networking/web namespace separation

- **Author:** suggested considering `System.Networking.Sockets` and
  `System.Web.Http` later as a separation between networking and web concerns.
- **Assistant response/action:** recorded the suggestion as provisional in the
  platform roadmap and linked it from the HTTP POC plan. Compared .NET namespace
  organization and noted the historical System.Web association. Proposed keeping
  HTTP consumers independent of explicit socket manipulation.
- **Open:** final names, package/provider boundaries, URI placement and the exact
  client/server surface. No API rename or implementation occurred; current Object
  work and the existing networking milestone sequence remain in place.


### 2026-09-23 — Object equality and record-syntax acceptance

- **Author:** asked to resume Object work after recording the possible networking
  namespaces, then said: “The ultimate test would be record semantics support -
  with the record syntax.”
- **Assistant action:** implemented a bounded class identity/equality/hash slice
  with a handwritten Cell/Key sample, native identity services and API documentation.
  String identity calls explicitly reject the current wrapper representation.
- **Validation finding:** a raw direct instance Equals call with null could initially
  behave like static ReferenceEquals(null, null). Added a separate receiver check
  for instance equality; static null identity remains valid.
- **Record action/outcome:** added a concrete Raven record-class acceptance source
  and a matching .NET assertion. The Raven probe reaches emission then fails on
  missing EqualityComparer<T>. Compiler source also requires System.HashCode for
  generated hashes. This is not completed record support.
- **Next:** compare typed comparer/hash contracts and Runtime Contract adaptation;
  use record syntax to validate generated members, ordinary dispatch and reference/
  value semantics together. Broader record forms and String/boxed behavior remain
  explicit follow-ups. See [the Object review](object-model-review.md#record-syntax-as-the-end-to-end-acceptance-case).


### 2026-09-23 — Raven examples as a pre-release compatibility pass

- **Author:** proposed porting selected Raven/.NET examples near release to discover
  crashes and missing functionality, then solving the findings. Specified that shared
  changes must first be verified on Raven main and released, even when they also
  exist on the neoCLR branch, so shared issues can be excluded from the comparison.
- **Assistant action:** added a later roadmap/release-validation gate with pinned
  compiler/release provenance, minimal port diffs, failure classification, fixes and
  retests. Shared fixes remain independently integrated; neoCLR policy stays on its
  feature branch. Platform-neutral cases should not be repeated across every host.
- **Open:** example selection, Raven release/version, concrete fixes and execution of
  the pass. No Raven release or compatibility-pass completion is claimed now.


### 2026-09-23 — Implement HashCode and adapt record generation

- **Author:** clarified: “we should make Raven support the neoCLR semantics for
  records. And implement System.HashCode”.
- **Assistant response:** accepted this as the next implementation direction after
  completing the current Object checks. Plan coordinated library/compiler work,
  isolated neoCLR integration, and regression checks for normal .NET record behavior.
- **Status:** direction recorded; HashCode and record generation changes follow the
  bounded Object commit. No completion is implied by this entry.

Object-slice validation completed: nine equality/service checks, six identity cases,
six display regressions, both Raven samples, 15 .NET assertions, full library
regeneration and combined API/website validation passed. The record probe's emission
failure remains the next acceptance target.


### 2026-09-24 — First target-aware record implementation

- **Author direction carried forward:** make Raven support neoCLR record semantics
  and implement System.HashCode. Keep .NET reference/value behavior central and use
  actual record syntax as the acceptance case.
- **Assistant action:** implemented a bounded mutable HashCode value accumulator,
  an opt-in compiler RuntimeRecordContract and a record-class sample with integer
  components. Existing .NET synthesis is retained; other target record shapes report
  an explicit diagnostic. The compiler work stays on the experimental feature branch.
- **Integration finding:** record emission exposed readonly backing fields,
  IsExternalInit return modifiers and Deconstruct integer outputs. Added checked
  importer handling rather than changing records into ordinary mutable classes.
  Initialization-only assignment is a compiler rule; runtime field flags and property
  reflection remain outside this slice.
- **Observed outcome:** the neoCLR record source passes identity, equality through
  typed/Object views, operators, hashes, display and deconstruction. HashCode tests
  cover signed boundaries, independent copies and UTF-8 string content. Compiler
  baseline and configured tests pass (40 checks); full artifact regeneration and
  combined website/API validation pass (385 documented items).
- **Open:** additional component/null contracts, record structs/generics/inheritance,
  generic hashing and randomized/adversarial-input policy. A source-defined generic
  provider fixture exposed a shared lookup-cache candidate, deferred for independent
  main validation; the accepted contract is tested across a referenced metadata boundary.
  See [the sample](experiments/records/README.md) and [design comparison](hash-code-design.md).


### 2026-09-24 — Complete the first struct and record-struct slices

- **Author:** “Continue with the slices until struct support snd record structs have
  been implemented”, following “Skip VS Code build, continue”. This explicitly
  selects the next Object work and excludes further editor-build work in this slice.
- **Assistant proposal:** implement named-value Object overrides first, then configured
  Raven record-struct generation and end-to-end samples, retaining reference sharing,
  value copying and boxing semantics.
- **Actions:** added boxed-payload override dispatch, value type testing and copied
  unboxing; extended the experimental Raven contract and importer. The checked sample
  now includes ordinary Counter and Coordinate/NamedCoordinate/OwnedCoordinate record
  structs. Updated API reference, feature page and integration/design records.
- **Outcome and bounds:** the first non-generic implementation has source execution
  evidence; see the [Object review](object-model-review.md#struct-object-slots-and-record-structs--2026-09-24).
  Generic/nested-struct components, nullable-value boxing and automatic ValueType
  equality remain explicit follow-ups. No VS Code build, release or publication was
  performed. No subsequent author approval of individual implementation details is
  inferred from the continuation request.


### 2026-09-24 — Provisional reference nullability for Raven compatibility

- **Author:** nullable annotations may not be the final design, but should be kept
  at least for reference types to obtain compatible Raven behavior; the metadata
  format may be reconsidered later.
- **Assistant response:** retain reference annotations now and align Object.Equals
  and ReferenceEquals with their existing runtime null behavior, separately from
  any future metadata redesign.
- **Action:** update Raven source, compiler-reference/bootstrap declarations and
  core Object typed-null importing; extend the checked Object sample and API/site
  documentation. No runtime type, virtual slot or null representation change.
- **Open:** final nullability policy and metadata representation, nullable values,
  and generated record-specific equality annotations. This does not infer a permanent
  commitment beyond the author's stated reference-type compatibility direction.


### 2026-09-24 — Prefer Option for modeled absence

- **Author:** neoCLR should favor the Option union to model absence for value types
  and reference types. Current nullable annotations are mainly for reference-type
  compatibility; nullable structs might be added later but should not be added now.
- **Assistant response/action:** record Option<T> as the preferred absence model,
  keep reference annotations for existing null-based contracts and Raven compatibility,
  and explicitly defer Nullable<T>/nullable-value boxing in the roadmap and on-site
  guidance. Preserve the earlier metadata-format question as open.
- **Outcome:** documentation and priorities change; no runtime/compiler behavior is
  changed or Option-record component support claimed. Earlier nullable-value
  follow-up suggestions are superseded for the current scope.

### 2026-09-24 — Apply Object contracts across the class library

- **Author:** confirms the direction and asks that the established Object hierarchy,
  equality, hashing and ToString behavior work across the platform and be used
  wherever the runtime class library needs those operations. The available message
  ends mid-sentence; no additional design choice is inferred.
- **Assistant response:** finish the active boxed Boolean slice, then audit library
  consumers and value-like types against the common contracts. Distinguish bounded
  existing implementations from complete cross-library support.
- **Initial findings:** Path has typed equality and a non-override ToString but no
  matching Object equality/hash; HashMap requires explicit equality/hash callbacks;
  HashCode has only integer/string inputs; several time/value/error types have typed
  methods without a complete Object contract. These are inspection findings, not
  repaired behavior. See the Object review for the proposed implementation order.

- **Author follow-up:** also ensure correct garbage-collector behavior for boxing.
- **Assistant response/action:** include GC roots, aliasing and reference-bearing boxed
  payloads in the consistency gate. Add a focused regression that clears the source
  struct, forces collection and invokes the boxed override through its retained child.
  Existing box/alias/heap-limit checks remain part of validation.

### 2026-09-24 — Value-type async state machines

- **Author:** asks how true value-type support can improve async state-machine
  efficiency in Release, whether Raven supports that representation, and directs
  focusing on making it work if so.
- **Assistant findings:** Raven defaults to struct states; neoCLR opts into classes.
  Existing Raven tests pass, but the target Release struct probe is rejected during
  import. Emitted code boxes at startup and continuation registration because the
  current builder takes an interface by value. The scratch closure failure shared
  by both modes was isolated, not fixed.
- **Action:** retain a reproducible probe and prioritize byref startup, one suspended
  state owner, GC/completion checks and measured allocation comparison ahead of Path.
  Keep the working heap default. Record the .NET builder comparison and implementation
  gates in the async assessment.
- **Open:** target value-state execution and allocation benefit are not established;
  no new public scheduling model, default switch or optimized SDK release is claimed.

### 2026-09-24 — Transitional state-machine implementation

- **Author:** directs continuing through the value-state slices, then clarifies that
  runtime-owned async suspension remains the future direction. Generated state
  machines are a shortcut to building the platform; their support APIs may later
  be deprecated or removed.
- **Assistant action:** implement a ref compiler/importer protocol over the existing
  reference builder, with one retained owner on first suspension. Keep application
  Task/await behavior separate from the temporary builder ABI. Add managed class
  field addressing and no-result byref value methods exposed by the real generated
  program. Preserve the working heap policy while comparing both representations.
- **Validation/outcome:** see the [retained experiment](experiments/value-async/README.md)
  for the final ready/pending/GC/cancellation measurements and bounded limitations.
  No runtime-owned suspension, general custom-builder contract or stable builder API
  is implied by this implementation.

### 2026-09-24 — Library Object semantics and explicit equality operands

- **Author:** requests applying Object semantics to the runtime library, particularly
  Path, and investigating other classes.
- **Assistant:** implements Path typed/interface/Object consistency and exercises it
  through existing HashMap callbacks. Audits value-like wrappers separately from
  mutable collections, resources and provider-bound objects.
- **Author correction:** warns that equality interfaces must not casually impose
  nullable operands when value types cannot be nullable; asks for explicit contracts.
- **Assistant action:** removes its uncommitted Path? typed parameter. Preserves
  Equatable<T>.Equals(T) and Equals(Path); keeps the existing Object.Equals(Object?)
  boundary distinct. Documents and tests reference and value equality contracts.
- **Outcome:** the Path fixture passes. The TypeInfo probe reports typed equality
  but different Object equality/hashes for wrappers of the same type. The
  [audit](object-model-review.md#path-integration-and-wider-library-audit--2026-09-24)
  records this next repair and unresolved provider/descriptor identity decisions.

### 2026-09-24 — Introspection after Path

- **Author:** directs continuing with introspection types after the current slice.
- **Assistant action:** commits Path as 840eceeb, then fixes the demonstrated
  RuntimeTypeInfo Object mismatch using represented-type equality, consistent hash
  and represented-type display. Keeps typed operands non-nullable and wrapper
  allocation identity separate. Adds generated TypeInfo/MemberInfo reference coverage.
- **Evidence:** the [integration fixture](experiments/introspection-object/README.md)
  covers repeated/constructed/array type descriptors and a map under GC pressure.
- **Open:** assembly/module scoped identity is the next candidate; member equality
  needs kind and closed-owner checks. Parameter snapshots lack declaring-member
  identity, so their equality is deferred rather than inferred from names or positions.

### 2026-09-24 — Await propagation documentation and landing theme refinement

- **Author:** corrects the earlier await precedence discussion: `await input?`
  now awaits first, then propagates Result Error or Option None. Parentheses are
  unnecessary. Asks for a clearer code inset and a hero adapted to the selected
  theme, preserving syntax highlighting; confirms the dark palette already works.
- **Assistant action:** updates current migration/feature documentation and the
  executable sample, and exercises the shorthand in the async propagation probes.
  Keeps the earlier discussion above as history and published release notes intact.
  Gives the light hero a pale slate-blue background and the sample a tinted inset,
  preserving the dark hero palette and shared Raven token highlighting.
- **Validation:** all six immediate/resumed Result success/error/cancellation
  probes and four Option Some/None probes pass. The combined site builds and
  checks 491 pages; all ten website contract tests pass. Both theme palettes and
  highlighted code were inspected in the browser.
- **Author follow-up:** requests publishing only the site, without a full repository
  build or runtime release. The assistant selects a `[skip ci]` push followed by
  manual dispatch of `pages.yml`; that workflow has only site build/deploy jobs.
  Publication outcome is recorded by the GitHub Actions run.


### 2026-09-24 — Generic math as a later exploration

- **Author:** “We might also add support for generic math interfaces later”.
- **Assistant response:** proposes recording this as exploration after consistent
  numeric contracts, comparing .NET generic math rather than expanding the current
  floating-point Object slice.
- **Action:** adds a roadmap exploration with a generic sum sample, compiler/runtime
  static-member questions and conversion/overflow/special-value validation needs.
- **Open:** interface names, breadth, language/runtime support and scheduling are
  undecided. No generic math API has been implemented or committed for a release.


### 2026-09-24 — Continue from Char into String

- **Author:** requests not stopping after the current slice and continuing with the
  next one, assumed to be Strings.
- **Assistant response/action:** completes and commits Char contracts as cf65c1aa,
  then reproduces String Object dispatch failure and implements bounded content
  equality/hash/display. Keeps String allocation identity as separate unresolved work.
- **Evidence:** [String Object sample](experiments/string-object/README.md) and
  [Object review](object-model-review.md) track the behavior and validation.
- **Open:** stable String identity, interning and a shared heap-text representation
  are not implied by the content-contract change.


### 2026-09-24 — Next focus: String storage and identity

- **Author:** asks what to focus on next, then directs continuing.
- **Assistant recommendation:** investigate shared immutable String storage and
  reference identity, ahead of remaining value contracts and default comparers;
  consolidate stale roadmap status. Generic math stays later exploration.
- **Action:** maps copy/ownership boundaries, adds a measured Rust prototype and
  .NET identity baseline, and records alternatives and migration gates in the
  [String storage investigation](string-storage-design.md). Updates the roadmap's
  current summary to reflect completed Object/record/content-contract slices.
- **Open:** the production representation, text-byte accounting and host/worker
  lifetime rules remain to be implemented and validated. The prototype does not
  enable String ReferenceEquals or change guest APIs.


## 2026-09-24 — Future System.Text and comparer infrastructure

**Author:** Requested a more consistent System.Text API in the future and, sooner or
later, general comparers, especially string comparers similar to .NET.

**Assistant:** Recorded this as future roadmap/design work while completing immutable
String storage. Proposed separating ownership from equality/hashing/ordering policies,
with ordinal, case and culture behavior evaluated explicitly against .NET. No new
comparer API or System.Text migration is claimed. The [String design](string-storage-design.md)
records tradeoffs and the remaining identity work.


**Author follow-up:** Added ToUpper/ToLower-style functions and comparison methods to
the plan, and clarified that the API need not copy .NET directly: improvements are
welcome where appropriate. **Assistant:** Recorded casing, culture/Unicode behavior,
comparison/hash consistency and compatibility tradeoffs as design questions; no
implementation commitment beyond future text work was inferred.

## 2026-09-24 — String construction and explicit Count

**Author:** Asked for `String(['F', 'o', 'o'])`, suggested Sequence<char> as the
parameter and a contract String would naturally implement, then proposed explicitly
implementing Count so it appears through Sequence<char>.

**Assistant action:** Added sequence construction, public grapheme indexing and
explicit Collection.Count while retaining public Length. The
[sample](experiments/string-sequence/README.md) compiles and runs, verifies immutable
copies and Unicode boundaries, and rejects direct Count/index mutation. Recorded the
installed compiler's direct empty-literal emission limitation; a typed empty char
array works. API reference and integration documentation accompany the change.

**Author follow-up:** Asked whether an Iterable<char> overload should take count.
**Assistant proposal:** Count is useful for allocation but not essential; a growable
buffer could consume an iterator. Keep Sequence in this slice and revisit Iterable
with buffering/limits. An explicit count needs a precise meaning. No author decision
on that future overload is recorded, and no such overload was implemented.

**Author decision:** Avoid defining contracts prematurely. After the semantics slices,
implement APIs and behaviors as needed for real cases, considering the case first;
some choices are inexpensive to change. **Assistant action:** Recorded this development
policy in the authoritative roadmap. Iterable construction remains undecided, rather
than a selected future overload.


## 2026-09-24 — Completing String reference semantics

**Author:** Asked what was next, then accepted the proposed String reference-identity
and identity-hash slice with “Let’s do it!”

**Assistant proposal/action:** Use the shared text owner across aliases and conversions,
keep content equality/hashes separate, and validate distinct equal constructions,
GC and host lifetime. Implemented owner-based comparison and address-independent
identity hashes through existing Object operations; no interning, compiler metadata
change or new helper API. Expanded the Raven String Object sample and .NET baseline.
The [design record](string-storage-design.md#shared-owner-identity--development-2026-09-24)
explains hash collisions, cost and remaining boundaries. Follow with a bounded semantics
review before choosing API additions from a concrete case.


## 2026-09-24 — Explicit interning exploration

**Author:** Suggested that some interning functionality should be in place, then asked
to continue after the assistant proposed explicit interning and a repeated-identifier
case without prematurely selecting permanent pool lifetime or literal interning.

**Assistant action:** Inspected immutable LoadedProgram versus per-execution mutable
state; built a test-only owned pool with entry and UTF-8 payload limits. The sample's
4,000 field-name occurrences reuse four owners/28 payload bytes. Four checks pass for
identity, quotas, separate pools and GC/host/pool lifetime; the .NET baseline also passes.
This is retained-payload evidence, not an allocation benchmark or a production API.

**Open:** compare execution/session ownership using repeated host invocations, then
choose exhaustion behavior and public exposure from the case. String.Intern remains a
candidate spelling; no automatic literal interning or lookup method is selected.
See the [experiment](experiments/string-interning/README.md).


## 2026-09-24 — Integrating interning with execution lifetime

**Author:** Asked to continue after the bounded interning experiment and proposed
execution/session ownership comparison.

**Assistant decision/action:** Selected execution ownership for the first implementation:
LoadedProgram is immutable and existing invocations/workers own fresh state. A persistent
runtime session would add an abstraction without a current case requiring it. Promoted
the pool into runtime state and added String.Intern, configurable entry/payload quotas
and InternPoolLimitExceeded. Added host-invocation, worker, terminal cleanup and Raven
sample checks; these distinguish independently created text from an already shared
host input. The author did not separately specify the lifetime or quota defaults;
these are documented assistant implementation choices, subject to future case evidence.

**Open:** shared-session requirements, future suspension ownership, automatic literal
interning and lookup helpers remain separate questions. No SDK release or website
publication is part of this slice.


## 2026-09-24 — Meaningful String parameter names

**Author follow-up:** While interning integration was underway, requested better names
for String method parameters. **Assistant action:** Aligned constructor `characters`,
Intern `text`, Concat/CompareOrdinal `left`/`right`, Equals `other`, search `substring`,
`prefix`, `suffix`, and SliceUtf8 `byteStart`/`byteLength` across source, reference,
introspection and docs. Added named-argument sample checks and a rejection check for
old generic names. Positional signatures remain unchanged; named callers must migrate.


## 2026-09-24 — Socket API immediately after the ownership slice

**Author:** directed the assistant to continue immediately toward the Socket API after
finishing the current slice. **Assistant action:** completed and committed the private
receive registry (`cff9d0df`), then implemented the first public Connect/Receive/Close
client and Task/Result bridge. A host greeting server supplies the first application
case; two-sided echo still requires send and accept/listen. Validation evidence is
maintained in the [sample](experiments/socket-client/README.md).

**Author follow-up:** “I didn’t realize we need UdpClient and TcpClient classes on top
of sockets later”. **Assistant response:** these can be useful convenience layers;
record them for later evaluation without making them prerequisites for Socket.
TcpClient/stream adaptation and UdpClient/datagram behavior need separate cases.
This is a proposed layering direction, not a commitment to exact .NET names or APIs.


**Further author clarification:** the immediate goal is a provisional Socket API
sufficient to eventually receive and send HTTP messages for the web-app demo; the
whole chain need not be fully developed. **Assistant response/action:** retain the
client slice, select send and listener/accept next, then the minimum stream/text and
HTTP behavior required by the demo. Do not make TcpClient/UdpClient or a complete
network stack prerequisites. The demo purpose and this sequencing are recorded in
the socket design and roadmap.


## 2026-09-24 — HTTP directly over Socket and near-term DNS

**Author:** suggested direct use of Socket for the HTTP client's TCP flow, requested
networking/HTTP essentials including headers, requests and responses, and proposed
adding DNS now. **Assistant assessment:** direct Socket use is appropriate; .NET's
SocketsHttpHandler also establishes sockets without requiring TcpClient. Propose
host-backed async name resolution early, keeping blocking resolver work off the VM
thread, and a bounded HTTP client with explicit framing and ownership.

**Action:** updated the roadmap and socket design with hostname exchange, GET/POST
and web-app products, essential protocol behavior, validation and the local-HTTP/TLS
boundary. Exact public resolver/address/HTTP signatures remain provisional. This
exchange adds implementation direction; it does not report DNS or HTTP as implemented.


## 2026-09-24 — Encoding boundaries and propagation syntax

**Author:** the socket case also shows the need to handle character and string
encoding later. Clarified that the Raven compiler fix makes `await Foo()?` and
`try Foo()?` sufficient: propagation surrounds the complete expression.
**Assistant action:** record explicit text conversion and fragmented multibyte
validation in the HTTP direction; simplify the active socket sample and documentation
and refresh its local compiler bundle to verify the syntax. No new encoding API
is implemented in this slice.


## 2026-09-24 — Separate Networking and Web feature entry points

**Author:** requested feature pages and landing-page boxes for both Networking and
Web once their POCs are in place. **Assistant response:** introduce Networking with
the hostname/TCP echo POC; give Web its separate page and box when HTTP works.
**Action:** add the Networking overview, tested-source lookup example, API links and
homepage/guide navigation. Record the Web publication gate in the roadmap. No HTTP
implementation or website deployment is implied.


## 2026-09-24 — TypeInfo flags and the closed interface direction

**Author:** requested IsAbstract, IsOpen (positive inheritance terminology rather
than IsSealed), IsClosedHierarchy, nominal IsUnion, IsEnum and IsValueType; directed
future work toward the proposal's more extensive closed TypeInfo interface hierarchy.
**Assistant initially proposed:** recording the set as target direction before
implementation. **Author correction:** “The flags is what we should make sure are there”.
**Assistant action:** implement the three missing flags, retain the existing three,
preserve descriptive inheritance metadata in the importer, and add runtime and Raven
source validation. The broader interface hierarchy remains an incremental direction.
**Assistant contract choice:** IsOpen excludes closed families; IsClosedHierarchy
refers to the declared direct family, not necessarily all transitive descendants.
See the [contract and evidence](introspection-design.md#type-classification-flags--development-2026-09-24).


## 2026-09-24 — Future IPAddress and possible HostEntry

**Author:** suggested creating IPAddress when ready, and perhaps HostEntry, following
the theme of value objects. **Assistant response/action:** record IPAddress as future
direction and HostEntry as a candidate, while retaining the current string-address
POC. Proposed immutable address semantics, Result-based parsing and explicit equality/
hashing for later evaluation; class versus struct and exact signatures remain open.
The [design comparison](socket-api-design.md#address-value-objects--future-direction-2026-09-24)
records the .NET baseline, benefits, migration costs and validation questions.
No implementation or change to immediate milestone order is implied.

**Outcome:** the compiled flag sample passes with seven collections and zero live
objects; matching library/API snapshots and the combined 575-page website validate.
No interface-family redesign or website publication is included.


## 2026-09-24 — Complete public API reference coverage

**Author:** reported missing collection classes and other library types on the
website, clarified that coverage means public types, and emphasized the types used
in samples to understand the platform. Further clarified that every public type
should participate in generation even without documentation content, preferably
with type and member descriptions.

**Assistant action:** audit the public reference metadata, replace selection by
automatic public-type generation, add descriptions for the missing API families
and improve sample-oriented navigation. Preserve explicit manual references for
namespace containers and the readonly marker where the pinned renderer omits a
type page. Missing prose is reported without hiding public APIs; missing pages
remain a validation failure. Publication is separate from the local site build.

**Author follow-up:** RavenDoc changes must be on both Raven main and neoclr.
**Verification:** the general flat-namespace navigation change is on main as
`9775eca0f` and neoclr as `9e196cc4b`; RavenDoc sources and docs match across those
branches. This task's public inventory and documentation additions are neoCLR site
configuration/content, not target-specific code added to Raven main.

### 2026-09-24 — Symbol-derived member pages enhanced by authored content

- **Author:** points out missing navigable property/return/parameter types on
  RuntimeContext.Current, suggests comparing language documentation conventions,
  and establishes that XML/Markdown enhance a structure generated even without
  comments. The longer-term goal is richer usage content on every type/member page.
- **Assistant:** compares Microsoft Learn, Rust and Kotlin references; chooses
  Parameters, Property value and Return value with linked types and a distinct
  Declaring type label. Implements shared RavenDoc sections from symbols, merges
  authored descriptions, and retains remarks/examples and overload grouping.
- **Evidence:** 26 focused RavenDoc tests pass, including source and metadata
  cases with undocumented properties, nullable/array/generic types and overloads;
  sample site rebuilt. Shared change is on Raven main (0017c5165) and its neoCLR
  branch (ce8d1068c); the generator documentation records the comparison sources.
- **Future direction:** deepen type/member usage examples progressively. No blanket
  claim is made that all APIs already have complete examples.
- **neoCLR validation:** the pinned publisher builds and checks 955 pages; all 15
  website tests pass. RuntimeContext.Current has a linked Property value, and
  InputStream.Read displays linked parameter/return types with existing XML prose.

### 2026-09-24 — Hide importer/exporter scaffolds from the API browser

- **Author:** identifies duplicate non-generic Array, Option and Result entries and
  asks to exclude importer/exporter artifacts without confusing readers.
- **Assistant:** identifies the analogous TaskOutcome container and their six CLR
  case-carrier types. Adds exact, reasoned exclusions using RavenDoc's existing
  configuration; keeps all generic application types and the complete metadata
  inventory. These exclusions are neoCLR policy, not a universal RavenDoc rule.
- **Validation:** 17 website tests pass, including real publisher checks that
  generic APIs remain and scaffold entries disappear. Full-site validation uses
  an isolated copy of committed compiler inputs because parallel compiler work
  makes the working checkout's reference fingerprints stale.
- **Outcome:** all 930 remaining pages build and pass link/anchor checks. The local
  preview is refreshed; publication remains separate.

### 2026-09-24 — Provisional HTTP client and handler pipeline

- **Author:** asks that the rough HttpClient stay close to the envisioned proposal,
  adapted from .NET, while remaining open to revision as encoding expands. Suggests
  handlers for mocking, then clarifies: “Handlers allow us to attach behavior to the
  client request-response pipeline.”
- **Assistant:** implements application-local request/response/content contracts and
  a small handler interface. Forwarding handlers observe before/after outcomes; a
  fake substitutes transport, while the terminal handler uses DNS and sockets.
- **Integration:** nested generated states exposed a private containing-field access
  rejection. The importer now admits that valid access while retaining unrelated
  private-field and readonly-write rejection checks. Other compiler observations
  remain explicitly unresolved in the [sample](experiments/http-client/README.md).
- **Open:** public API promotion, cancellation/deadlines, reusable handler ownership,
  broader HTTP and the neoCLR responder. No retry/authentication implementation or
  permanent handler base-class hierarchy is inferred from this direction.

- **Subsequent author direction:** continue until a sample app and APIs are integrated
  into neoCLR. Finish HttpClient first, then add HttpServer and test both together,
  also using an independent server. Prefer lower-level/native facilities where useful;
  the author subsequently confirms that implementing HTTP over the Socket API is OK.
- **Assistant action:** promotes the bounded contracts into System.Web.Http and adds a
  private continuation adapter over existing DNS/socket operations. No public scheduler
  or state-machine API is introduced. HttpServer follows client validation; broader
  protocol support and native-provider selection remain open.


- **Validated outcome:** the integrated app passes 18 controlled-peer cases and an
  independent Python HTTP server, with zero retained managed objects in every run.
  The .NET 10 comparison passes. Seven public HTTP types have generated API coverage;
  the combined website builds 961 pages and 17 website tests pass. HttpServer remains
  the next author-selected slice; no release or publication is performed.

### 2026-09-24 — Uri and IPAddress family values

- **Author:** says Uri and IPAddress value objects will be needed; proposes that
  IPAddress might be a union of IPv4Address and IPv6Address.
- **Assistant:** records the union as a candidate alongside the earlier immutable
  address direction, comparing .NET and identifying parsing, identity, mapped addresses
  and scope handling as open questions. Keeps HTTP client/server work active and the
  existing string inputs unchanged. No new value-object implementation is claimed.
- **Evidence:** [address model notes](socket-api-design.md#uri-and-address-family-values--author-direction-2026-09-24).

### 2026-09-24 — HTTP errors as nested union values

- **Author:** asks what error union HttpClient.Get should return; suggests nested
  union payloads and cautions against importing .NET's exact exception hierarchy.
- **Assistant proposal:** distinguish invalid requests, transport causes, invalid
  responses, unsupported features and resource limits. Consider a nested transport
  union preserving resolver/socket causes and failure stage, while keeping HTTP
  statuses as responses and text decoding at Content.ReadText. Compare provider-neutral
  alternatives before requiring socket-shaped causes from all transports.
- **Status:** [candidate recorded](http-client-design.md#error-unions--candidate-2026-09-24),
  not implemented or author-approved as an exact case list. The integrated POC retains
  explicitly provisional string errors. Timeout/cancellation alignment remains open.

### 2026-09-24 — HttpClient conveniences after Uri

- **Author:** asks to later match basic .NET HttpClient behavior, including a
  parameterless constructor and BaseAddress; explicitly identifies Uri as a prerequisite.
- **Assistant:** notes that HttpClient() already selects HttpSocketHandler in the
  integrated POC. Records BaseAddress and relative-address resolution after a defined
  Uri contract. BaseAddress is not implemented, and string concatenation is not selected
  as a substitute. This does not interrupt the next minimal HttpServer slice.

### 2026-09-24 — Bounded HTTP server and independent peers

- **Author direction:** after the HttpClient POC, add HttpServer and test both together
  and against independent peers. Continue with subsequent application slices.
- **Assistant choice:** implements a provisional ServeOne callback with buffered
  request/response values. This avoids selecting a permanent hosting loop before
  cancellation, concurrency and runtime suspension are defined. This exact method
  shape is an assistant implementation choice, not a quoted author specification.
- **Action:** adds Listen/GetLocalPort/ServeOne/Close and received request headers,
  internal bounded parsing/encoding, an executable UTF-8 greeting server and matching
  importer/reference/library contracts. The server computes response framing and
  closes its accepted connection after sending or a Result error.
- **Evidence:** [design](http-server-design.md) and [verifier](experiments/http-server/README.md)
  cover a neoCLR pair, .NET client, fragmented requests and malformed input/application
  responses with collection and zero final live objects. No runtime/compiler semantics
  change, release or publication is part of this slice.
- **Open:** request deadlines/cancellation, general HTTP status/body support, structured
  errors and a permanent hosting contract. Next use the existing JSON experiment in
  the network application; Uri/BaseAddress remain later as directed.

### 2026-09-24 — JSON over the HTTP POC

- **Author context:** the application goal includes a small JSON API and the author
  asks to continue subsequent slices after the HTTP integration. The user favors
  deciding contracts from concrete cases, rather than prematurely completing layers.
- **Assistant action:** reuses the existing JSON document experiment in a report server
  and client. The server constructs a report; the client fetches and parses it, reads
  fields and prints a locally constructed acknowledgement. No POST or public JSON API
  is implied. The shared JSON codec is unchanged.
- **Outcome:** [the verifier](experiments/http-json/README.md) passes a neoCLR pair and
  independent Python HTTP peers. Server/client executions collect six or seven times
  with a 256-object heap and finish with zero live objects. The website includes the
  tested async consumer and a downloadable sample. No compiler changes or release.
- **Next gate:** bounded HTTP request/transfer lifetime, then use this concrete consumer
  to evaluate public JSON contracts. The full web-application milestone remains open.

### 2026-09-24 — First HTTP lifetime checkpoint

- **Context:** after the client/server and JSON application cases, the roadmap selects
  request/transfer lifetime; the author asks to continue.
- **Assistant choice:** adds a provisional five-second bound at the existing native
  Send/Receive owner, matching the style of the earlier fixed connect budget. This is
  an implementation choice for the demo, not an author-selected general timeout API.
- **Behavior:** expiry commits SocketError.TimedOut, releases transfer storage and
  preserves the socket. Existing HTTP adapters close their owned connection on an
  error. No managed timer race, public Scheduler or new compiler contract is added.
- **Evidence:** [design and comparisons](socket-api-design.md#pending-transfer-deadline--2026-09-24)
  record .NET async cancellation versus synchronous socket timeouts and Rust's native
  options. Deterministic owner-clock checks and stalled HTTP peers exercise cleanup.
- **Open:** configurable budgets, one total request deadline, cancellation of accept
  and application handler work. This changes previously unbounded development
  transfers and can reject legitimately slow peers; repeated short progress can
  still exceed the per-transfer bound in aggregate.

### 2026-09-24 — Shared deadline beneath the HTTP bridge

- **Context:** the author continues work after the pending-transfer timeout slice;
  the roadmap's remaining gate is one request lifetime across phases.
- **Assistant action:** adds private absolute-deadline paths to resolver and socket
  owners. The shorter phase bound remains active, and a shared budget is not renewed
  by fallback or short reads. A combined numeric-loopback probe exercises lookup,
  connect, initial body progress and expiry of the remaining receive.
- **Outcome:** 32 socket tests and nine resolver tests pass. Expired native admission,
  buffer cleanup and detached late resolver results are covered. The public managed
  HTTP behavior is unchanged; no service ABI or compiler work has happened yet.
- **Next:** carry the shared deadline through the private Raven bridge, then define
  handler/server ownership. No public timeout/token type is selected in this slice.
- **Evidence:** [probe](experiments/request-budget/README.md) and
  [contract comparisons](http-client-design.md#shared-native-deadline-checkpoint--2026-09-24).

### 2026-09-24 — Shared budget reaches the managed HTTP client

- **Assistant continuation:** follows the native checkpoint immediately with the
  private Raven bridge. Selects a provisional 15-second socket-handler exchange budget,
  retaining shorter phase bounds. The author has not selected that duration as a
  permanent or configurable public timeout policy.
- **Action:** passes a checked monotonic stamp through lookup/connect/transfers,
  consumes pending outcomes before finalization, closes late successful connections
  and checks expiry before returning buffered success. Public signatures stay unchanged.
- **Integration finding:** separate library slices could not call internal core helpers.
  The bootstrap reference temporarily exposes four Until methods for compilation;
  normal references and importer validation retain their internal contract. Explicit
  library-mode signature admission and negative application checks preserve the boundary.
  No Raven compiler change or general compiler fix is claimed.
- **Validation:** the focused HTTP verifier exercises regular header/body trickles,
  ordinary fragmentation and a stalled response. Native ownership and service-analysis
  tests cover the underlying paths; matching artifacts and API documentation are refreshed.
- **Remaining scope:** custom pipeline work outside transport, server accept/application
  callbacks, cancellation tokens and configurable timeouts. The HTTP design records the
  distinction from a general HttpClient.Timeout contract.

### 2026-09-24 — Uri, typed HTTP errors and BaseUri

- **Author:** “Our goal is to have Uri, HttpError, and BaseUri ready.” Requests both
  Uri and string overloads, rather than requiring callers to construct Uri values.
- **Assistant:** prioritizes this over the next cancellation exploration. Separates
  general reference parsing/resolution from the HTTP provider's transport restrictions.
  Uses BaseUri as the current spelling, superseding earlier BaseAddress discussion.
- **Action:** implements an initial managed Uri and UriError, with two Resolve
  overloads, exact text equality/hash behavior and reference-resolution validation.
  Adds RFC cases, negative grammar cases, Object/GC checks and a .NET 10 comparison.
- **Provisional choices:** strict escaped ASCII, 4096-byte bound and lexical identity;
  IP literals and broader normalization remain open. These are assistant choices for
  the POC, not permanent author requirements. See [URI design](uri-design.md).
- **Integration finding:** the first boxed-Object probe exposed a missing bridge
  reference-category entry, now added alongside interface/collection conversions.
  A boundary run exceeded its initial timeout; a local process sample showed array
  resource scans dominating while the parser held encoded input. Replaced that
  temporary byte array with bounded string slices; no GC policy change is claimed.
- **Outcome:** the final managed probe passes both resolution overloads, grammar,
  length limits, Object/interface/collection behavior and zero-live-object cleanup.
  Bootstrap/API validation, the 986-page site and 17 website tests pass.
- **Remaining:** HttpError and BaseUri integration with string/Uri HTTP overloads.
  No new scheduler or runtime-suspension requirement is introduced.

### 2026-09-24 — Later URI/URL encoding utilities

- **Author:** “We will have to add utilities for URI/URL encode later.”
- **Assistant response:** records this as future work, with path-segment, query-value
  and form-encoding distinctions to investigate. No method names or final encoding
  contract are selected, and the current escaped-input parser contract is unchanged.
- **Evidence:** [URI design](uri-design.md#later-encoding-utilities--author-direction-2026-09-24)
  and the platform roadmap. No encoding utility is implemented by this slice.

### 2026-09-24 — Standard union syntax as the class-library default

- **Author question:** asks whether custom union implementations have the desired
  form, explicitly distinguishing that question from .NET binary compatibility.
- **Author correction:** points out that standard union syntax supports members and
  directs its use across the neoCLR class library, with manual contracts only rarely
  when actually needed.
- **Assistant action:** removes the uncommitted HttpError custom-carrier/payload
  extension instead of making it the new template. Updates Raven conventions and
  roadmap sequencing, and creates a direct union probe with payloads, a computed
  property and an authored ToString. No runtime public API is changed by this probe.
- **Findings:** current Raven compiles the source. The inspected CLI carrier has a
  byte tag and typed case fields, plus IUnion/Value/HasValue/TryGetValue members;
  it is not the same private storage as the existing System.Value-backed carriers.
  Import first rejects a SocketError byref. A temporary narrow byref admission then
  reaches a non-local initobj rejection; that diagnostic-only edit was reverted.
- **Decision boundary:** prefer normal source syntax while evaluating the target
  representation independently. The observed CLI layout is evidence, not approval
  of a permanent ABI. Default/inactive state, copying, boxing and GC remain validation
  requirements before normal union forms are used for the public HTTP contract.
- **Evidence:** [direct union probe](experiments/http-error-unions/README.md).
  HttpError/BaseUri integration remains pending. Existing custom carriers are not
  migrated wholesale during this investigation.

- **Continuation outcome:** the bridge now imports and executes a reduced nested
  standard-syntax union. Constructor receiver initialization is bounded to its own
  value type, and recognized TryGetValue methods use conditional outputs. The probe
  exercises empty defaults, matching/nonmatching cases, copies and boxed display
  under GC pressure. A mixed legacy SocketError now reaches runtime verification,
  which rejects its non-defaultable System.Value field. No public HttpError or
  runtime-library carrier migration is claimed; those remain subsequent work.
- **Validation:** the probe reports 101 allocations, two collections and zero live
  objects. Three malformed-contract checks and the mixed-default rejection pass,
  as does the constructor-argument regression. The broader record suite fails before
  import on Equatable/Equals diagnostics with the installed SDK; this remains an
  open validation gap, not a passing check or a claimed compiler regression fix.


### 2026-09-24 — Private Raven storage declarations

- **Author clarification:** private storage `var` and `val` declarations are emitted
  as fields rather than properties. Explicit `field` is primarily for compatibility
  or when a field declaration is specifically intended, public or private.
- **Assistant action:** records this in the Raven conventions and repository guidance.
  Bootstrap contract checks should follow emitted metadata rather than insist on
  explicit source `field` spelling. The existing instance-library regression uses
  private `var` storage and passes; no wholesale storage rewrite is claimed.
- **Scope:** this clarification accompanies the ongoing standard-union bootstrap
  work and does not replace the HttpError/BaseUri objective.


### 2026-09-24 — Raven union metadata boundary

- **Author observation:** points out `RavenUnionCompanionAttribute` during the union
  work. Reiterates that the objective is normal union syntax in the class library,
  followed by returning to the main work.
- **Assistant findings/actions:** identifies Raven's case-name/ordinal metadata and
  generic companion-to-carrier association. Preserves selected case metadata in the
  bootstrap reference fixture and validates it before importing native bodies. Adds
  a generic producer/consumer metadata probe, without claiming generic runtime support.
- **Author direction:** avoid a runtime dependency on Raven metadata. The companion
  object belongs to Raven's convention; support it during development, but defer
  standardizing a platform case map and reconsider the mapping later.
- **Boundary:** the adapter lives in the Raven bridge; imported runtime types and
  methods do not depend on Raven attributes. This is not a new union ABI. Production
  reference integration and mixed legacy carrier defaults still block HttpError;
  generic companion execution is not a new prerequisite for that HTTP objective.


### 2026-09-24 — No mandatory per-case union predicates

- **Author direction:** rejects a hard convention requiring an `Is*` property for
  every union case, explicitly including Result and Option. Directs migration of
  existing unions to normal union syntax where applicable after the current slice.
- **Assistant response/action:** removes the proposed compatibility predicates and
  accessors from the SocketError source migration. Uses generated case matching and
  updates the convention docs so the older manual-carrier contract is historical,
  not a requirement imposed on new unions.
- **Sequence:** finish SocketError SDK/runtime integration, then migrate applicable
  existing unions in bounded slices. Generic Option/Result/TaskOutcome and other
  remaining manual carriers need their own target validation; no completed migration
  of those families is implied by this direction.

- **Author follow-up:** suggests a match expression for SocketError.ToString and
  notes that cases need not be qualified inside their containing union.
- **Assistant action/validation:** uses unqualified match arms; verifies all 13
  names directly and through Object, inactive defaults, nested unions and copies
  under GC pressure (127 allocations, three collections, zero live objects).
  Raven warns that the final catch-all is unreachable for declared cases; the
  runtime test confirms it still handles the inactive default as Empty.

### 2026-09-24 — Empty cases, data-bearing variants and enums

- **Author clarification:** SocketError can be recorded as a basic union. Unions
  shine when grouped variants hold their own data; enums should remain an option
  when the model is a set of named constants. Keeping SocketError as a union is
  acceptable, without implying that every named set should become one.
- **Assistant action:** records the distinction in the modeling and Raven
  conventions. Retains SocketError and points to the tested nested HttpError probe
  as the data-bearing example; the public HttpError API is still planned.

### 2026-09-24 — EntryKind as an enum

- **Author observation:** identifies EntryKind as an enum candidate while asking
  the assistant to continue through the migration slices.
- **Assistant decision/action:** removes EntryKind from the union migration and
  implements it through the existing enum lowering, with File = 1 and Directory = 2.
  These preserve the host kind values. Zero is unnamed; unsupported/missing entries
  continue to use StorageLookupError. Provider callers use enum comparisons.
- **Tradeoff:** this gives named constants without empty case-carrier machinery,
  like .NET enums, but allows unnamed underlying values. No new Unknown result is
  invented and enum values do not themselves prove a successful storage lookup.
  Matching SDK/library/application rebuilds are required. The rebuilt error probe
  validates all 76 empty-case error variants and EntryKind box/unbox round trips.
  Storage provider and reader checks pass with an explicit larger fixture budget.

### 2026-09-24 — Future Error interface and diagnostics

- **Author proposal:** errors expressed as unions could later implement an Error
  interface providing integration points for stack traces and other diagnostics.
  Notes the lack of macros for generating supporting implementations.
- **Assistant response:** records this as future exploration, not a current API or
  a requirement on every error union. An interface supplies access points but does
  not itself capture a trace. Compare compiler-generated support, explicit library
  helpers and runtime/API-boundary capture, including cost and propagation behavior.
- **Open:** interface members, who owns/captures diagnostic context, when a trace
  is captured, and whether support is compiler-generated remain undecided. The
  current migrations neither add Error nor claim stack-trace capture for results.

- **Author follow-up:** adds composability and decorating errors with more
  information as future benefits of an Error interface. The assistant records
  wrappers retaining an underlying error as an example, without selecting an API.

### 2026-09-25 — Helpers on System.Enum

- **Author direction:** asks for enum names/values helpers on Enum itself, and
  clarifies that static methods should have overloads taking an enum type parameter
  while retaining the overloads accepting TypeInfo.
- **Assistant proposed contract:** GetNames(TypeInfo) and GetNames<TEnum>(), plus
  GetValues(TypeInfo) and GetValues<TEnum>(). The generic value form preserves the
  enum type and requires an enum constraint. All forms share metadata and ordering;
  enum formatting should use that same metadata. Implementation is the next slice,
  not a shipped API in the union migration.

### 2026-09-25 — Built-in range and Boolean text fields

- **Author proposal:** add MinValue and MaxValue fields to built-in types where
  applicable; points to .NET Boolean.FalseString and TrueString.
- **Assistant response:** schedules an audit after the Enum helper slice, covering
  numeric and Char limits and Boolean text fields. The supplied
  [Microsoft reference](https://learn.microsoft.com/en-us/dotnet/api/system.boolean.falsestring?view=net-10.0)
  defines FalseString as a static read-only field containing `"False"`. Field shape,
  pointer-sized limits and existing Boolean formatting need validation. No fields
  are claimed implemented by this planning entry.

### 2026-09-25 — Enum flags helpers

- **Author proposal:** consider methods for enum flags on Enum.
- **Assistant response/investigation:** identifies HasFlag as the first candidate,
  with `(value & flag) == flag` semantics including zero. Existing nominal runtime
  enums already emit typed HasFlag and bitwise helpers, but the current Raven
  reference does not expose HasFlag on System.Enum. A public surface and validation
  are still needed; no new helper is claimed shipped. Broader has-any/set/remove
  convenience APIs remain use-case-driven candidates.
- **Related correction:** Char is a grapheme in neoCLR, so the earlier suggestion
  to copy UTF-16 Char bounds does not fit. Restrict the upcoming numeric field audit
  accordingly; pointer ranges must follow the target contract.

### 2026-09-25 — Finish the current slice and return to HTTP

- **Author direction:** “When done with this, put other changes on hold, finish
  union migration, continue with work on Http.”
- **Assistant response/actions:** completes the Enum helper slice first and places
  numeric/Boolean fields and further flags APIs on hold. The SDK probe validates
  names/typed values/TypeInfo discovery, formatting, compiler rejection of non-enum
  arguments and runtime rejection of non-enum TypeInfo; GC run reports 501 allocations,
  ten collections and zero live objects. Broader checks expose the historical Neo
  manifest accidentally using Raven-only generated union bodies. Explicit legacy
  snapshots restore that profile boundary; Raven still uses normal union declarations.
- **Next:** close remaining applicable migrations, keeping generic Option/Result/
  TaskOutcome projection blockers explicit, then resume the HTTP error/URI pipeline.
  Those generic carriers are not claimed migrated and do not automatically become a
  prerequisite for the author-directed HTTP work.


- **HTTP follow-through:** the assistant extends the bounded library projector to
  a normal nongeneric sequential HttpError prototype with URI, DNS, socket and text
  payloads. A separate Raven consumer compiles; imported library methods execute
  extraction/default/copy/boxing checks (101 allocations, four collections, no live
  objects). It fixes a private-helper conversion wrapper without relaxing visibility.
  Payload mismatches and overlapping layouts are rejected. This completes a bridge
  prerequisite; public HttpClient/handler/server signatures still need integration.

### 2026-09-25 — HttpClient's fundamental operation

- **Author direction:** specifies Send(HttpRequest, CancellationToken) returning
  Task<Result<HttpResponse, ...>> as the basic operation, with Get, other methods
  and GetString as variants.
- **Assistant response:** adopts this as the integration target, using the planned
  HttpError union and routing convenience methods through the handler pipeline.
  Inspection finds task cancellation outcomes but no public CancellationToken.
  Records cancellation forwarding/cleanup and convenience-method validation in the
  [HTTP design](http-client-design.md#core-request-contract--author-direction-2026-09-25).
- **Still open:** cancellation outcome versus HttpError representation, GetString
  non-success status policy and implementation of the token-aware overloads. The
  payload-union bridge checks pass; the public HTTP surface is still tokenless and
  string-error-based at this checkpoint.

- **Author base-address clarification:** BaseUrl/BaseUri should be Optional<string>.
  Verb methods construct URLs from it when set and expect relative URLs in that
  mode; without a base, callers use absolute URIs.
- **Assistant response/action:** records Option<string>, the existing library type,
  retaining BaseUri as the working name. Updates the HTTP/URI design and roadmap;
  replaces the earlier planned absolute-override test with the stated two-mode
  contract. Uri remains useful internally for parsing/resolution and as an address
  overload. Implementation and exact error cases remain pending.

### 2026-09-25 — Networking and web release scope

- **Author direction/question:** work toward a release with networking and web that
  feels reasonably complete, without being 100% done; HttpClient and cancellation
  tokens are on the list, and asks what else belongs.
- **Assistant assessment:** the current GET/200, bounded buffered bodies and
  ServeOne prove integration but need request-body/status breadth and lifecycle
  behavior for a coherent release. Proposes headers/text/byte content, typed errors,
  deadlines and cleanup, basic interoperable framing, a repeatable stoppable server,
  and a packaged storage-backed notes sample with API documentation.
- **Assistant scope recommendation:** investigate HTTPS through a maintained backend
  early; require it for an external-service client claim or explicitly retain a
  plain-HTTP experimental scope. Defer pooling, HTTP/2/3, retries, broad hosting,
  convenience socket wrappers and runtime suspension. Keep validation shared except
  where host behavior differs. Records candidate gates in the
  [roadmap](platform-roadmap.md#networking-and-web-release-target--discussion-2026-09-25).
- **Status:** this is scope planning; no new networking APIs, release version/date
  or author approval of the additional recommended gates is claimed.

### 2026-09-25 — IPAddress union selected for the release

- **Author direction:** adds IPAddress to release scope, showing a union containing
  IPv4Address(...) and IPv6Address(...) cases and optional additional members.
- **Assistant response/action:** removes IPAddress from the deferred release list
  and records the standard-union shape in the roadmap and socket design. Proposes
  parsing, formatting, value semantics and DNS/socket integration as the minimal
  useful slice, keeping payload details open and IPv6 transport support distinct.
- **Status:** the release inclusion and case names are author-selected; payload
  representation, scope/mapped-address policies and implementation remain pending.

### 2026-09-25 — Execute the networking/web slices in order

- **Author direction:** “Let's start working. Finish all slices one by one.” This
  follows the proposed nine-slice sequence and the IPAddress addition.
- **Assistant action:** starts public typed HTTP errors, preserving native causes
  through normal HttpError union cases. Adds projected payload-family consumer
  bindings and fixes constructor instruction emission. No Raven compiler emission
  or Runtime Contract option changes. Public signatures now require matching
  rebuilt library/reference/application artifacts; cancellation and BaseUri are next
  contracts, not silently implemented by this change.
- **Integration observation:** storing the intermediate awaited Result as an async
  local exposes the already tracked hoisted-field initialization issue. Direct
  MapError composition works in the client sample. This is not claimed as a compiler
  fix; a reduced independent compiler investigation remains necessary.
- **Progress:** client cases and the JSON application pass with independent peers
  and zero live managed objects after GC. Server error paths also pass, including transfer timeout and invalid application
  responses. Matching API reference and clean generated-library checks pass; the
  site build completed before the subsequent validation correction.

- **Author validation correction:** asks to select only the most relevant tests for
  each slice and skip website builds. The assistant adopts focused validation,
  keeps docs/reference snapshots synchronized and reserves broader checks for
  release stabilization or demonstrated risk. The typed-error website build had
  already completed; no further website build is required for this slice sequence.


### 2026-09-25 — Select a closed IPAddress class hierarchy

- **Author:** raises a closed hierarchy as an alternative to the earlier IPAddress
  union, then explicitly chooses “Use a closed class hierarchy.”
- **Assistant:** compares separate family reference types with the single value-union
  representation; proposes immutable address data and value equality. Notes that
  importer layout limitations should not decide the public model.
- **Action:** updates the release roadmap, records the comparison in the
  [address hierarchy probe](experiments/ip-address-hierarchy/README.md), and starts
  with an executable inheritance/equality/GC check. The probe exposed a protected
  base-constructor importer gap; a bounded direct-base allowance addresses it.
- **Open:** public reference projection, parsing/formatting, scope identifiers,
  DNS/socket integration and API documentation. This checkpoint does not complete
  the address slice or imply IPv6 transport support.


### 2026-09-25 — Implement the selected address hierarchy

- **Author:** asks to continue after selecting the closed class hierarchy.
- **Assistant action:** implements public IPAddress/IPv4Address/IPv6Address with
  immutable data, value equality, parsing and canonical formatting. Adds typed
  IPAddressError, changes DNS to return address values, retains string Socket
  overloads and adds typed ones. IPv6 values are accepted independently of transport;
  the IPv4 backend returns UnsupportedAddressFamily for IPv6 socket operations.
- **Tradeoffs:** strict decimal IPv4 intentionally differs from legacy forms accepted
  by .NET. Scope identifiers, public byte construction and IPv6 transport remain
  open. These bounded choices are assistant implementation decisions, not separately
  quoted author approvals. See the [design and evidence](ip-address-design.md).
- **Validation:** focused parser/Object/GC and closure checks, .NET parsing comparison,
  typed DNS/loopback echo, independent HTTP-server interoperability, signature and
  artifact/API snapshot checks. Website source is updated; the author-requested
  website build skip remains in force. Runtime cancellation is the next release slice.

### 2026-09-25 — Cancellation request and acknowledgement foundations

- **Context:** the author includes cancellation tokens in the networking/web release,
  asks to finish slices one by one, and retains runtime suspension as future work.
- **Assistant decision and action:** implements an invocation-local source/token pair
  and disposable registrations in System.Concurrency. Cancel requests synchronously;
  the operation acknowledges only after cleanup. Keeps this independent of a public
  scheduler and of the current compiler state-machine representation.
- **Tradeoffs:** follows .NET's authority/observation split and callback ordering, but
  does not claim .NET's thread-safety or exception aggregation. Registration is a
  reference handle. These are bounded implementation choices, not newly quoted
  author approvals. See [the contract](cancellation-design.md).
- **Evidence:** callback reentrancy/disposal, copied/boxed/array tokens under GC,
  cleanup before Task cancellation, access rejection and a .NET 10 behavior baseline
  passed. The test ends with zero live allocations. The library and reference
  snapshots are refreshed; the website build remains skipped by author direction.
- **Open:** HTTP/socket token wiring, timers, linked sources and cross-invocation
  cancellation. A nested callback/outer-array compiler emission observation is recorded
  for independent reduction; no compiler fix or runtime suspension is claimed.


### 2026-09-25 — Integrate optional HTTP base addresses

- **Prior author direction:** BaseUri should be optional string configuration; use
  relative verb-method URLs when it is set, absolute URLs otherwise, with both
  string and Uri overloads. The author also requests relevant per-slice tests and
  no website build.
- **Assistant action:** integrates that rule with the existing Uri parser/resolver
  and handler pipeline. Rejects authority-replacing network-path references with
  a configured base. Validates text at request construction and allows configuration
  changes between calls; Send consumes an already constructed request.
- **Evidence:** both overloads pass the base-resolution/error matrix and the .NET
  comparison. The updated sample passes against an independent Python HTTP server;
  both runtime fixtures finish with zero live allocations. API docs and website
  source are updated, with the website build skipped.
- **Open:** release slice 4 still needs token-aware Send/Get, native cancellation and
  GetString. The assistant reviewed HTTPS feasibility and records a possible
  host-side TLS adapter, not an implementation or author-approved dependency choice.
  See [the design](http-client-design.md#baseuri-and-address-overloads--implemented-2026-09-25).

## 2026-09-25 — Propagated error conversions and HTTP status names

- **Author clarification:** Raven supports converting a propagated error through
  object conversion or an implicit converter, and “Conversions can be defined as
  extension methods.”
- **Assistant action:** add an application-owned HttpError-to-AppError extension
  example alongside propagation to Object in the [status fixture](experiments/http-status/Main.rvn).
  Keep the HTTP library's own typed error contract. Verification is recorded in the
  fixture README; no new Raven conversion semantics are claimed.
- **Author question:** “Should HttpStatusCode get its own enum?”
- **Assistant proposal:** yes, use named integer constants in System.Web.Http for
  response status and UnsuccessfulStatus payloads, retaining unnamed valid extension
  codes and numeric success classification. Finish status behavior checks first,
  then verify enum conversion/formatting before changing the signatures. This is a
  follow-up design proposal, not an implemented enum or an explicit author decision.

- **Author follow-up:** response patterns could query/deconstruct HttpResponse when
  it has a Deconstruct contract; HttpRequest could offer the same on the server side.
  **Assistant response:** retain this as a sample-driven follow-up in the request/content
  slice, with named properties primary and no positional signature selected yet.

- **Author design direction:** consider record-like deconstruction for ordinary API
  objects from the outset; value-object semantics are not required. The author sees
  room to design APIs around Raven capabilities rather than inherit .NET API shapes.
  **Assistant interpretation:** assess useful pattern/extraction contracts during API
  design independently of equality, copying and immutability. .NET also supports
  deconstruction; the opportunity is consistent Raven-oriented library design, not
  a claim that the language mechanism is unavailable on .NET. Request/response
  positional signatures remain open until demonstrated by a real sample.

## 2026-09-25 — Response inspection is an optional developer experience

- **Author clarification:** the motivation for request/response deconstruction is
  developer experience, including nominal/positional inspection and property patterns.
  The author sketches matching Ok(HttpResponse { StatusCode: HttpStatus.NotFound,
  Headers: let headers }) and explicitly says it need not be preferred or correct code.
- **Assistant response:** separate readable-property patterns from a positional
  Deconstruct contract. Validate syntax against the target before publishing examples;
  do not make either style mandatory or imply value equality. The chosen enum name
  for the current slice is HttpStatusCode, consistent with the prior discussion.
- **Action:** add a bounded property-pattern probe to the status fixture. An initial
  compile reports RAV1613 for combining outer `if let` with inner `let headers`;
  the probe uses `Headers: headers`. Execution evidence is recorded with the fixture.

- **Author further clarification:** an outer `let Ok(HttpResponse { ... Headers:
  headers }) = result` already supplies binding; an `is` pattern instead captures
  explicitly with `Headers: let headers`. **Assistant action:** add both forms to
  the target fixture and distinguish this from the rejected combination of both
  binding styles in one pattern. No Raven syntax or compiler change is requested.

## 2026-09-25 — Propagation in application examples

- **Author clarification:** prefer error and Option propagation in samples and code
  demonstrating user code, because it becomes clearer once propagation is understood.
  This does not exclude handling the union when that is useful.
- **Assistant action:** record the preference in Raven conventions and separate the
  buffered POST application example from explicit union assertions in its fixtures.
  The example propagates Send failures, applies a deliberate non-success status policy,
  and adapts the provisional ReadText string error at the boundary. No language change.
- **Evidence:** [POST fixture](experiments/http-post/README.md); normal echo, send
  failure, invalid UTF-8 and cancellation are exercised. Broader example migration
  remains incremental as each area is edited.

- **Author follow-up:** keep converters near methods to project individual error types
  into a meaningful common application error returned by Result. A future System.Error
  interface (the author points to Raven.Core) could wrap errors as causes and add context;
  the author suggests the interface itself would define that conversion.
- **Assistant response:** record that as a design candidate, distinguish generic cause
  wrapping from application-specific domain mapping, and leave interface conversion
  lookup and representation to investigation. No System.Error implementation or automatic
  wrapping is claimed for this HTTP slice.

## 2026-09-25 — HTTP content will support streams

- **Author direction:** “We will support streams also for HttpContent. But continue.”
- **Assistant action:** record stream-backed content as planned capability, with
  ownership, cancellation, unknown lengths and repeatability still requiring design.
  Continue the bounded HTTP slice with case-insensitive, duplicate-preserving header
  lookup. Byte buffering is an intermediate implementation, not the final constraint.
- **Scope:** no stream overload or transport streaming is implemented by this header
  checkpoint. Linked design compares .NET StreamContent and identifies lifetime/framing
  questions rather than assuming its ownership policy.

## 2026-09-25 — With methods and Raven with-expressions

- **Author direction:** “When implementing the With* methods we can eventually
  support Raven with-expression”.
- **Assistant response:** preserve copy-producing WithHeader as an API building block
  and record eventual language integration. Determine how with-expressions would handle
  Result-returning updates and validation errors before selecting a compiler convention.
- **Scope:** no with-expression lowering change or automatic method-discovery contract
  is implemented in this HTTP slice. Shared content still prevents deep immutability;
  update syntax alone does not imply records or value equality.

- **Author correction:** “Perhaps not in this case though since WithHeader is not
  complying with that expectation”. **Assistant response:** agree that WithHeader
  selects a collection entry by name and can fail validation, rather than directly
  setting a property in a copy. Keep it outside the prospective with-expression mapping;
  the broader future direction for suitable With* methods remains open.

## 2026-09-25 — Explicit HTTP accept as a callback alternative

- **Author proposal:** HttpServer could expose asynchronous Accept/AcceptRequest,
  returning a Result containing a request/response pair such as HttpContext, alongside
  the callback-focused API. The exact method/argument spelling remains exploratory.
- **Assistant investigation:** inspect the existing exchange, which owns accept,
  parsing, handler invocation, sending and connection cleanup; compare .NET
  HttpListener.GetContextAsync and HttpListenerContext using primary documentation.
- **Assistant proposal:** evaluate Task<Result<HttpContext, HttpError>>, with request
  access and an explicit response completion capability. For the buffered stage,
  Respond(HttpResponse) could reuse the existing message type; a live Response writer
  remains an alternative for stream-backed output. Make context cleanup, cancellation,
  one-response state and server shutdown explicit; keep ServeOne as a possible adapter.
- **Outcome:** record the design candidate in the server-lifecycle slice. No Accept,
  HttpContext, response writer or options type is implemented or finalized here.

## 2026-09-25 — HttpContext as the HTTP application foundation

- **Author clarification:** the application could close or dispose the context when
  ready; “The http context would be a foundational concept for building http web apps”.
- **Assistant response:** treat HttpContext as the lifetime scope of one HTTP exchange,
  shared by explicit acceptance and callback-based hosting. Update the roadmap from an
  optional return-container exploration to this architectural direction, keeping the
  current slice order and exact API signatures open.
- **Correction to earlier assistant proposal:** close/dispose is a normal end-of-scope
  operation, not merely abandonment. Async completion and error reporting, cleanup on
  early exit, repeated disposal and future stream/connection ownership still need rules.
- **Action:** update server design and roadmap. No public API or runtime implementation
  is introduced by this documentation change.

### 2026-09-25 — complete framing, then context ownership

- **Author:** “Continue with both slices,” referring to response framing/HEAD and
  HttpContext/server lifecycle.
- **Assistant action:** added bounded chunked and close-delimited reception and HEAD
  request/client overloads plus server body suppression. Focused raw-peer and .NET
  HEAD checks, framing negatives, signature admission and API/bootstrap snapshots pass.
  Website content was updated; its build was skipped as directed.
- **Remaining:** the context lifetime slice follows this checkpoint. No release or
  deployment has been performed.

### 2026-09-25 — response configuration versus context convenience

- **Author:** HttpContext should offer shortcuts such as responding with text, on top
  of managing the combined request/response lifetime.
- **Assistant proposal:** initially suggested RespondText forwarding to an asynchronous
  context Respond operation.
- **Author correction:** “HttpResponse of course have the real Respond methods,” with
  status-code and optional-content overloads, and convenience indirection on HttpContext.
  Clarified: “Respond on the HttpResponse sets the status code and content.”
- **Assistant response:** accepted `context.Response` as the response API and asked
  whether Respond only configures it, with separate asynchronous context completion,
  or also sends. This completion boundary is pending clarification; the earlier
  context-owned Respond suggestion is not treated as the selected contract.

- **Author follow-up:** considers request/response interfaces with separate inbound and
  outbound implementations as a direction, while saying that setting status/content may
  be enough now. Also requests context Respond forwards matching HttpResponse.Respond.
- **Assistant implementation choice:** setter-style Respond methods, an explicit asynchronous
  context Complete boundary, context forwards/text helpers, and shared accept/callback
  lifetime management. The interface split is recorded for later; no new hierarchy is added.


### 2026-09-25 — provisional JSON release demonstration

- **Author:** says it is soon time to consider release and suggests a provisional JSON
  DOM in System.Data.Json, plus minimal reflection-based serialization/deserialization
  to demonstrate the platform, without claiming the API is finished.
- **Assistant proposal:** after the two active HTTP slices, evaluate a bounded DOM and
  one concrete model round trip. Inspection finds public member metadata but no public
  reflective value access, assignment or construction/invocation in current descriptors.
- **Action:** record that dependency and scope on the authoritative roadmap. No JSON or
  reflective access implementation is claimed; the broader serializer remains future work.

- **Author clarification:** System.Runtime.Reflection will extend the
  System.Introspection model; its operations work for models backed by loaded runtime
  types. **Assistant response:** keep metadata description separate from runtime
  capabilities and evaluate an explicit unsupported-operation result for metadata-only
  models. Namespace/layering direction is recorded; exact extension signatures and
  failure shape remain to be designed for the JSON POC.

- **Author sketch:** internal ReflectionHelpers and a ReflectionExtensions extension
  over TypeInfo, illustrated with a SetValue operation returning Result<void,
  ReflectionError>. The author labels it “the concept.” **Assistant response:** public
  extensions delegate to internal helpers; settle receiver/parameter shapes around
  the JSON round trip. The sketch is not recorded as compilable Raven or a final API.

- **Author correction:** “I don't say you should create that API. Or name types that
  way.” **Assistant response:** treat the sketch as illustrative separation of
  responsibilities only. Exact APIs and type/helper/error names remain open; the
  roadmap was revised to avoid turning the sketch into an implementation requirement.

- **Author scope:** the minimum should allow creating objects and setting properties,
  or fields, in a way familiar to .NET users while remaining neoCLR. **Assistant
  response:** scope construction and value reads/writes around the serialization round
  trip, with neoCLR type/failure contracts; retain API/name decisions for that work.

- **Author correction:** “the extension should be on PropertyInfo,” correcting the
  earlier TypeInfo sketch. **Assistant response:** property operations extend
  PropertyInfo; corresponding field operations would extend FieldInfo. This updates
  the receiver direction without finalizing illustrative method names/signatures.


### 2026-09-25 — HTTP lifecycle checkpoint outcome

- **Assistant-reported implementation:** Accept/HttpContext, synchronous response
  configuration and matching context conveniences, asynchronous Complete and explicit
  close/dispose now share the callback path. Server.Close ends owned exchanges;
  cancellation and outstanding scope/read/send bounds are documented.
- **Validation:** direct context/sample and two cancellation peers pass, as does the
  independent .NET verb/HEAD callback regression. Each finishes with zero live objects.
  API/signature/bootstrap checks pass; website build was skipped as directed.
- **Open:** the frozen compiler/bridge has captured-callback and discarded-unit
  propagation failures; the passing fixtures use explicit state/result assertions.
  A [captured-callback reproduction](experiments/http-context/repros/README.md) is saved
  for reduction and independent Raven validation. No compiler fix is claimed. The
  JSON/runtime-reflection discussion is roadmap direction, not an implemented API.

- **Author extension:** a JSON serializer POC should exercise StreamReader,
  StreamWriter and Stream APIs; JsonSerializer should read/write streams and support
  string input/output. **Assistant response:** use the same model round trip through
  both paths, checking partial transfers, malformed input, encoding and ownership as
  release-demo acceptance criteria. This updates planned scope; no serializer API is
  implemented by the HTTP lifecycle checkpoint.

### 2026-09-25 — JSON string/stream exploration

- **Author direction:** continue the serializer POC; eventually offer GetJson<T> and
  PostJson extension methods for HttpClient.
- **Assistant action:** add a bounded, experiment-local JsonSerializer over the
  existing JSON tree codec, exercising StreamReader/StreamWriter with borrowed
  streams. Keep public extraction and runtime-reflection object mapping distinct
  from this I/O probe.
- **Open:** public structured errors, reflective member access/construction, async
  stream behavior and the exact HTTP convenience contracts. The HTTP extensions
  remain planned, not implemented. Evidence: [JSON streams](experiments/json-streams/README.md).

- **Author additions:** provide provisional System.IO.MemoryStream; handle the JSON
  DOM first, before considering object deserialization. **Assistant action:** implement
  the three existing byte-stream capabilities with bounded managed storage and use
  write/rewind/read in the DOM fixture. Reflection remains a later milestone.

- **Validation outcome:** the focused target DOM/memory/short-transfer fixture and
  .NET memory/JSON baseline pass. The target ends with zero live objects after 13
  collections. This establishes synchronous DOM I/O evidence, not a public JSON
  API or reflection implementation.

### 2026-09-25 — choosing the public JSON DOM shape

- **Author question:** whether the DOM should be a union or a closed hierarchy.
- **Assistant proposal:** compare a six-case union against kind-specific reference
  types; both can have mutable containers and exhaustive matching. Recommend the
  hierarchy for container APIs, retaining unions for errors. The first extraction
  was provisional and based on the earlier experiment, not an author decision.
- **Author decision:** “Closed JsonValue hierarchy with kind-specific APIs (recommended)”.
- **Assistant action:** put object/array operations on those node types, scalar
  payloads on typed properties, and preserve explicit JsonNull vs MissingField.
  Extract a DOM-only JsonSerializer and structured JsonError into System.Data.Json.
- **Validation so far:** the public consumer and acknowledgement sample pass, including
  nested I/O causes and memory/short-stream paths, with zero final live objects.
  Parser corpus and artifact checks are recorded with the [fixture](experiments/json-dom/README.md).
- **Open:** larger application-driven limits, replacement/removal/query conveniences,
  and later reflective object mapping. No inferred decision on those APIs, deep
  equality or HttpClient JSON extension signatures.

- **Author clarification:** asks whether serialization should be Result-based.
  **Assistant response:** all four implemented string/stream Serialize/Deserialize
  overloads already return Result with JsonError; nested I/O causes are preserved
  and callers can propagate with `?`. This confirms the API direction rather than
  introducing exception-based failure handling.

- **Author correction:** meant Task/asynchronous stream reading rather than Result.
  **Assistant response:** proposed checking pending-I/O contracts before exposing an
  asynchronous serializer path. **Author decision:** “No keep it sync”; asynchronous
  reads are a possible later optimization track. Continue the current work.
  **Action:** retain synchronous Result-returning overloads; no Task wrapper, async
  stream contract or scheduler change was added.
- **Final validation:** all 54 JSON document cases and 12 checked integer conversions
  pass through the public API against the .NET baseline; signature and matching
  API/bootstrap snapshot checks pass. Website build remains skipped.

### 2026-09-25 — Command-line entry injection

- **Author question:** “Can't neoCLR inject commandline args as parameter to Main?”
- **Assistant finding:** the Raven bridge rejected parameters; runtime startup was
  parameterless. Proposed a generated adapter preserving that ABI.
- **Action and outcome:** implement collection-profile Main(string[]) with a fresh
  argument array excluding the executable, matching .NET. Keep the full vector in
  Environment.GetCommandLineArgs and migrate the JSON corpus. Three argument/GC
  cases and signature checks pass. No compiler configuration change.

### 2026-09-25 — Console display fallback and scalar overloads

- **Author direction:** add Console.WriteLine(object) as a fallback, then suggests
  overloads for known built-in types to avoid boxing.
- **Assistant response/action:** add virtual ToString dispatch and null-as-empty-line
  handling, plus direct Boolean, Char and integral overloads. Reuse Int32/Char
  formatting and add private Int64/UInt64 text services.
- **Bounded choice:** keep floating-point formatting separate because its Object
  numeric formatting contract is not yet implemented. No culture/provider API.

### 2026-09-25 — Fix the JSON application's async blocker

- **Author:** directed the Console follow-up into an integrated JSON Web sample,
  with object mapping/minimal reflection only after an end-to-end example. When
  integration exposed an issue, explicitly said “This issue must be fixed”.
- **Assistant investigation:** isolated the startup fault to a hoisted Result field
  in a reference async state machine. Its System.Value payload has no readable
  default; assignment happens in MoveNext, after the constructor check. Both
  HttpError and application-owned AppError reproduced it, ruling out the earlier
  custom-error-only suspicion.
- **Action:** added explicit checked deferred class-field storage and selected it
  in the bridge through the core IAsyncStateMachine contract. Restored structured
  application errors in the JSON client/server. This does not fabricate default
  union cases or change normal constructor validation. The focused storage and
  pending-await/GC regressions pass. See [the decision and evidence](value-storage.md#deferred-class-fields--development).
- **Open:** other observed frozen-compiler expression/emission issues are recorded
  separately in the [sample limitations](experiments/http-json/limitations.md).
  Runtime suspension remains future work. The station-only managed Web round trip
  now passes; the earlier larger report still reaches transport timeouts and is not
  claimed fixed. The assistant began a [minimal reflection investigation](json-dom-design.md#next-investigation-one-mapped-report--2026-09-25),
  finding metadata accessors but no runtime creation/property execution path.
  No public reflection or object-mapping API was implemented.


### 2026-09-25 — JSON conveniences on both HTTP peers

- **Author clarification:** future GetJson/PostJson-style HttpClient extensions are
  one side of the story. The server should directly deserialize HttpRequest content,
  serialize response models, and the client should deserialize received responses.
- **Assistant proposal:** compose shared serializer/content conversion with verb,
  request, response and context conveniences. Preserve distinct send/read operations,
  structured errors and the existing response-configuration/completion boundary.
- **Action:** recorded all four directions in the roadmap and JSON design, with
  .NET client/content and ASP.NET Core request/response comparisons. No API was
  implemented or signature finalized; this is later work and does not reprioritize
  the active mapping/runtime investigation.
- **Open:** helper placement, empty/null content, media types, status/error handling,
  cancellation and future stream semantics. The current serializer remains synchronous.

### 2026-09-25 — Directional interface names

- **Author:** requested Equatable<T> → EquatableTo<T>, Comparable<T> → ComparableTo<T>
  and Convertible<T> → ConvertibleInto<T>, adding missing interfaces. Subsequently
  directed that full tests and website building be skipped, and requested a commit.
- **Assistant action:** renamed the equality/ordering identities, implementations,
  interface views, constraints, record configuration and current documentation.
  Convertible existed only in a proposal; added invariant ConvertibleInto<T> with
  its proposed Convert() -> T method and ordinary interface dispatch. No implicit
  conversion or return-type-directed overload feature was added.
- **Validation:** 47 focused Rust tests and bridge signature checks pass; the Raven
  consumer compiles, imports and runs with output 42, 0, True. API snapshot
  validation passes. Generated-record
  assignment could not be validated with the installed compiler: both the original
  Equatable<T> and renamed contract fail assignment, so this is a pre-existing
  compiler limitation. The runtime library/API artifacts are regenerated in an
  isolated copy to exclude concurrent JSON overload work. Website/full tests were
  not run, following the author instruction.
- **Scope:** this author-directed naming change does not reprioritize the active
  roadmap milestone. See [contracts and migration](common-interfaces.md#directional-interface-names-development-2026-09-25).

### 2026-09-26 — Publication-ready website and local testing tools

- **Author:** requests a buildable website and installation of the latest working
  SDK/VS Code extension, allowing uncommitted changes to be omitted.
- **Assistant:** isolates committed neoCLR revision 0606af5c from staged JSON work.
  Repairs missing JSON payload-member documentation with explicit manual routes
  and fixes two web-feature guide links. All 1,016 pages and 18 website checks pass.
- **Tool installation:** building matching local Raven/neoCLR tools; validation and
  installation outcomes will be recorded separately. No website publication or
  release is implied by these local builds.

### 2026-09-25 — JSON node naming and target-directed overload selection

- **Author:** suggested a `Node` suffix for JSON DOM serialization/deserialization
  methods if they conflict with object mapping. Separately asked whether the observed
  overload failure is inconsistent with C#/.NET and proposed exploring return-type
  selection for multiple `Into()` implementations of `ConvertibleTo<T>`, using the
  expected result type, with Rust as inspiration.
- **Assistant investigation:** the current compiler rejects methods with identical
  parameter types but different generic arity. That is a separate declaration bug,
  not evidence that return-type overloads are needed for generic JSON deserialization.
  The independently developed main-based Raven fix was subsequently integrated as
  `9c6860e1d` and cherry-picked onto neoCLR as `9da3c7cb7` (2026-09-26); 145 focused
  overload/generic tests passed.
- **Assistant proposal:** evaluate `DeserializeNode` as explicit DOM naming separately;
  keep return-directed overload selection exploratory. Specify no-target calls,
  ambiguity, generic inference and interface dispatch before committing a language
  rule. Existing target typing does not by itself settle those questions.
- **Status:** no node-method rename or return-type-overload feature implemented in
  this conversation entry. The immediate work remains typed JSON mapping.


### 2026-09-26 — Typed JSON reads expose a generic reference-cast gap

- **Assistant action following continuation:** adds typed string/stream deserialization
  wrappers over the existing flat mapper and uses them in the HTTP application.
- **Investigation:** execution revealed that `unbox.any T` extracted a class payload
  for reference T. The runtime now shares the reference-cast path, preserving identity,
  null behavior and GC roots while retaining exact value unboxing.
- **Validation:** five focused runtime tests and the public JSON mapper pass; the
  latter finishes with zero live objects. The typed managed HTTP pair and DOM/stream
  regression also pass with zero final live objects; 551 bridge signature checks
  and API/library snapshot checks pass. Website build skipped by author direction.
  See the [Object review](object-model-review.md#reference-targets-of-generic-unboxing--2026-09-26)
  and [compiler integration](raven-system-library.md#typed-json-wrappers-and-generic-arity--2026-09-26).
- **Open:** explicit Node names and return-directed overload selection remain design
  questions. The separately added ConvertibleInto interface does not establish such
  a compiler selection rule.

### 2026-09-26 — Configured HTTP/JSON testing workspace

- **Author:** asks to commit the work, remove legacy source-file instructions from
  active build guidance, document the website build and explain launching VS Code
  with a specific sample. Selects the HTTP/JSON web application as the testing flow.
- **Assistant:** installs SDK and extension 0.1.12-neoclr.20260926 from Raven
  54ec1c718, with neoCLR bundle cbf7bb70. Typed JSON edits staged when packaging began
  are excluded. Later author commits preserve the build-guide cleanup separately.
- **Validation:** eight async samples and collection/union/string editor protocol
  checks pass. Independent HTTP routes pass, and the managed pair passes in
  isolation. A combined sequence hits the existing transport deadline. The ordinary
  CLI budget is insufficient for this fixture; tasks use the existing measured
  runner with explicit limits. Full MSBuild acceptance fails its unresolved-call
  negative test; that compiler defect remains open.
- **Follow-up:** later installed-workspace managed-pair retries also time out. The
  default verifier therefore uses an independent Python client for server routes;
  `--managed-pair` preserves the failing reproduction. No reliable managed-pair
  claim is made. Personal home paths in documentation are normalized to `$HOME`.
- **Outcome:** the installation and sample-selection commands are documented in
  [the local snapshot guide](local-sdk-snapshot.md). Website build on the later
  committed tree also passes all 1,016 pages. No publication or release performed.


### 2026-09-26 — Explicit JSON DOM method names

- **Author:** “Continue. *Node naming is probably what we want.”
- **Assistant action:** adopts DeserializeNode/SerializeNode for both string and
  stream DOM overloads, retains object-mapping method names and existing Object
  node passthrough, and updates public callers/reference documentation.
- **Decision boundary:** no compatibility aliases in this development API. This
  naming change does not implement return-type-directed overload selection.
  Shared HTTP JSON helpers remain the next convenience slice.
- **Outcome:** focused DOM/stream, typed mapper and default HTTP pair pass with
  zero final live objects. All 551 signature checks and API/library snapshot checks
  pass. Website build skipped by author direction; no publication performed.


### 2026-09-26 — Shared JSON content conversion

- **Author:** requests continuation and asks not to run the full suite unless needed.
- **Assistant:** implements the next roadmap layer as System.Web.Http.Json.JsonContent
  with buffered model/node creation and reads, reusing serializer errors and limits.
  Both sides of the mapped HTTP sample use the same conversions. This is a
  provisional static helper surface; extension syntax and client verb wrappers remain
  later work. Header validation, HTTP status and context completion stay explicit.
- **Validation scope:** public content/mapping checks, bridge signature checks,
  matching API/library snapshots and the mapped HTTP pair; no full suite or website
  build. The public consumer already passes with zero final live objects.
- **Outcome:** the managed pair and independent Python server check pass with zero
  final live objects, as do all 559 signature checks and API/library snapshots.
- **Author follow-up:** selects GetJson/PostJson helpers as the next step and proposes
  HttpResponse.Request of type Option<HttpRequest> to associate a response with its
  request. Assistant proposes client association through the handler pipeline and
  None for standalone/server-created responses; exact policy remains to validate.
- **Author naming refinement:** asks whether .NET uses GetAsJson/GetFromJson and
  specifies generic overloads only for now. Assistant verifies the .NET names
  GetFromJsonAsync<T>/PostAsJsonAsync<T> and proposes neoCLR GetFromJson<T>/
  PostAsJson<T>, with POST returning HttpResponse and no implicit response mapping.
  These names and scope are recorded for the next slice; they are not yet shipped.


### 2026-09-26 — Request association, client defaults and JSON-client investigation

- **Author:** asks to continue, then explicitly prioritizes HttpResponse.Request.
  Subsequently postpones per-call header configuration on the JSON helpers and asks
  for client-wide default request headers.
- **Assistant implementation:** adds the optional response association at Send,
  preserving already-completed tasks and attaching pending results with a private
  continuation. Adds copied DefaultRequestHeaders with explicit fields taking
  precedence; defaults derive a request without mutating the caller's request.
  Response.Request exposes the effective request delivered to the handler.
- **Validation:** immediate/delayed handler, allocation-pressure, failure/cancellation
  and default-header tests pass with zero final live objects. No full-suite run.
- **Separate investigation:** generic GetFromJson/PostAsJson source compiled, but the
  converter method group inside Map emitted a default task without a diagnostic;
  an explicitly typed delegate instead reported RAV2203. The prototype is preserved
  and removed from active compilation. No generic client API is claimed complete.
  See the [investigation record](experiments/http-json-client-prototype/README.md).

- **Author goal reminder:** “We shouldn't forget our goal to have a sample demonstrating Http end-to-end using neoCLR”. The assistant confirms the managed client/server JSON sample remains the acceptance goal; request association and defaults support it, with generic helper integration still pending the compiler investigation.
- **Validated outcome:** independent GET/POST header checks pass; the mapped JSON neoCLR client/server pair also passes after this change (425 client and 369 server allocations, zero final live objects). All 569 bridge signature checks and API/library snapshots pass. Website build remains skipped by direction.


### 2026-09-26 — Generic JSON clients reach the managed HTTP sample

- **Author direction:** continue toward the end-to-end neoCLR HTTP application,
  retaining client-wide headers and deferring per-call JSON-header customization.
- **Assistant investigation:** reduces the method-group failure independently on
  Raven main; explicitly supplied enclosing type arguments were rejected for typed
  delegates, while inline higher-order calls emitted default results. Four positive
  regressions fail before and pass after; the uninferred negative case stays rejected.
  All 399 relevant overload-resolution tests pass before/after, and the five new
  checks pass on the neoCLR branch. General fix integrated on main as 13b9105d8,
  cherry-picked to neoCLR as 56083626e; no runtime-specific policy moved to main.
- **Implementation:** adds generic GetFromJson/PostAsJson, nested HttpJsonError causes,
  bounded private callback admission, and reference-preserving generic box T. The
  mapped sample now fetches a typed report, posts it and reads the acknowledgement.
  API reference, website and snapshot dependency coverage are updated.
- **Validated outcome:** public helper/cancellation fixture passes (863 allocations,
  zero live objects); five focused box/unbox tests and 609 bridge checks pass. The
  managed pair passes with 761 client / 626 server allocations; the independent
  client check passes with 789. All end with zero final live objects. A first pair
  run during regeneration times out; isolated checks pass with unchanged deadlines.
  The slower debug object test run was stopped after passing relevant cases and
  replaced by the five optimized focused checks; no full suite or website build.
- **Still open:** repeatability under load, the separate unresolved-call diagnostic
  issue, and deferred API expansion. See the [sample evidence](experiments/json-object-mapping/README.md#generic-json-verbs--2026-09-26).


### 2026-09-27 — Thematic tracking and current-state feature pages

- **Author direction:** asks to consolidate tracking documents by theme, then
  continues that work. Subsequently permits purging irrelevant website history:
  “It should reflect the state as is.” Routine preview breakage does not need a
  migration narrative; brief possible directions per feature may remain. The
  author reiterates: “Make sure features pages are up to date”.
- **Assistant action:** retains one authoritative roadmap and four current status
  owners for HTTP/networking, runtime/language, library/data and tooling/release.
  Archives the six earlier plans and preserves their entry-point anchors; design
  notes and experiments remain supporting records rather than independent priorities.
- **Website action:** reviews all feature pages, corrects stale claims about
  cancellation, streams, HTTP uploads/JSON, string indexing/identity and reflection,
  removes routine migration and experiment chronology, and updates overview and
  proposal summaries. The Tasks page/download use the existing development sample.
  Records the editorial policy in [website maintenance](design/feature-pages.md).
- **Scope:** no runtime/API behavior changes, release or website deployment. The
  [HTTP tracker](http-capabilities.md#finish-this-poc) still owns the outstanding
  packaged-SDK verification and explicit POC completion checkpoint. This direction
  changes documentation presentation, not the frozen POC feature scope.
- **Validation:** all 18 website tests and the 1,038-page site/link build pass.
  Rendered content from all 11 feature pages and the current Task download are
  checked. Six archived plans preserve their original body after link relocation;
  all 22 dated neoCLR issues have theme assignments. Existing RavenDoc missing-summary
  warnings remain. No runtime/API behavior changed or full runtime suite ran.


### 2026-09-27 — Release the POC with proportionate CI

- **Author clarification:** “The intention is to release this POC.” Reiterates that
  running all tests on every target in Actions takes over an hour and the release
  procedure needs optimization.
- **Assistant response/actions:** keeps the finite POC scope and exact-package checks,
  adds an optional HTTP package component, and separates one complete canonical CI
  run from focused host execution and per-OS minimum-Rust compilation. Preserves a
  manual full stable matrix. Checks the historical run: its slowest job took 68
  minutes, with approximately 225 total runner-minutes.
- **Release boundary:** this author direction selects release preparation, not a
  version/date or an assertion that known compiler defects are fixed. Package results
  and hosted acceptance must be recorded before release readiness is claimed.
- **POC outcome:** fresh SDK/runtime archives pass the typed JSON independent-peer
  and managed-pair checks, all 11 upload cases, and the repaired website download.
  Records bounded POC completion on macOS arm64 with known limitations preserved.
  Runtime package notices pass; the SDK audit identifies YamlDotNet 16.3.0 and its
  missing notice is added to the companion attribution inventory. Release preparation
  remains active; neither the final release nor website has been published.
- **Publication authorization:** the author then directs “Continue directly to release”.
  The assistant selects the next unused neoCLR preview version after checking existing
  releases, retains exact-candidate CI/package gates and proceeds toward publication.

- **Release qualification findings/actions:** packaged MSBuild reproduced accepted
  unresolved calls. The assistant first investigated a general compiler defect;
  comparison showed Raven main already rejects it. The defect belongs to neoCLR's
  terminal-Fault flow policy. Raven `d48bf14ba` fixes recursive binding/diagnostic
  loss with 55 focused tests. Windows host CI also exposed premature TCP connect
  success; neoCLR now waits for Winsock readiness, and the backpressure test checks
  portable byte/ownership behavior. Both findings remain release gates until the
  matching hosted/package checks pass. New SDK/VSIX packaging is in progress.

### 2026-09-27 — Standard Option/Result unions before publication

- **Author correction:** “Result and Option aren't implementing the union contract
  correctly.” When asked about the migration boundary, explicitly selects
  “Migrate to standard Raven unions”, then directs the assistant to use Raven.Core's
  implementations as the reference.
- **Assistant findings:** current handwritten runtime carriers lack the generated
  IUnion/Value/HasValue surface; compiler reference stubs advertise Value/TryGetValue
  while import adapters translate only the case extraction path. Earlier generic
  projection exceptions explain this gap but do not satisfy the requested contract.
- **Action in progress:** publication is held. The migration starts from Raven.Core's
  Some/None and Ok/Error declarations and propagation methods, with neoCLR's unit
  and interface identities. Generic companion projection, actual runtime behavior,
  matching reference/API documentation and fresh package validation remain required.
  Prior candidate package checks are evidence for that candidate only.
- **Author confirmation and test scope:** after noting the earlier port may have been
  correct, the author directs “Continue. Then continue with release.” Subsequently
  limits validation to enough tests to establish the contract and points out new
  fixes on Raven's neoCLR branch. The assistant includes `0524da35c` (contextual
  pattern symbols and generic completion), retains the successful prior canonical
  CI evidence, and targets migration/packaged contract checks instead of repeating
  unrelated suites.

- **TaskOutcome correction and audit:** the author asks whether TaskOutcome is a
  Raven union, then directs “Turn it into a Raven union” and “And look for other
  types that need such migration”. The assistant replaces its manual carrier with
  Completed/Cancelled cases and generated projection. A Raven runtime source audit
  finds no remaining manual union carriers; standalone error structs and the
  JsonValue class hierarchy retain their different contracts. Focused contract and
  completion/cancellation validation passes: the combined contract, 24 pipeline
  checks and seven runtime task tests. API documentation and the website are refreshed.
  A boxed Completed pattern binding limitation is documented; an explicit case
  cast validates the actual boxed contract. Release package qualification follows.

- **Package finding:** the first mapped HTTP response failed after migration. A
  focused property-accessor test reproduced native reflection snapshots using the
  old Option layout. The assistant updates that native interop boundary for the
  selected Raven and Neo profiles and keeps release publication held for fresh
  accessor and HTTP checks. Both profile regressions and all mapped HTTP peer/pair
  checks pass with the rebuilt runner and zero live objects. The Raven-source-only
  audit had not covered this
  native construction path; it is now included in the migration review.

- **Final qualification:** runtime revision `59f9f4a7` with Raven `a108df82a` passes
  extracted union/editor, mapped HTTP peer/pair and all 11 upload checks, with zero
  live objects. The author-directed reduced validation scope is explicit: full CI
  evidence at `1404454e` precedes migration; subsequent changes have focused checks.
  The assistant prepares Preview 10 assets, notes and [evidence](preview-10-validation.json)
  for publication and leaves later HTTP features tracked rather than starting them.

- **Publication outcome:** [Preview 10](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.10)
  is published as a prerelease at tag `v0.1.0-preview.10` (`2879df22`). All eight
  remote asset digests match the reviewed local files. The source archive matches
  all 2,620 tracked files and compiles after extraction. The roadmap now records the
  POC as released and done for now; client/server HTTP directions remain tracked.
  Website publication uses its separate manual workflow.

## 2026-09-27 — Useful APIs after the released POC

- **Author direction:** “Next is filling the gap of useful APIs, some that I have
  mentioned”. This selects library coverage after Preview 10 publication.
- **Assistant review:** recovers the earlier requests for consistent System.Text,
  general/string comparers, casing/comparison methods, StringBuilder and better time
  APIs. Current Raven HashMap still takes separate equality/hash callbacks; String
  already exposes exact equality and ordinal operations.
- **Assistant recommendation:** start with paired equality/hash policies and an
  ordinal string comparer consumed by HashMap, followed by text construction,
  explicit casing and scenario-driven parsing/time coverage. Keep at most one small
  independent companion task and retain feature work ahead of optional optimization.
  This ordering is the assistant’s proposal, not a quoted author selection of an API.
- **Action/outcome:** updates the authoritative roadmap and existing library/runtime
  trackers, correcting the stale M1 release-preparation status. The
  [library sequence](tracking/library-data.md#active-direction--useful-api-gaps)
  records the next bounded contract and validation requirements. No new API is
  implemented by this planning change; exact signatures remain open. HTTP stays
  done for now, with future client/server capabilities retained in its own tracker.


## 2026-09-27 — Implement comparers and reduce routine validation

- **Author:** “Implement comparers”.
- **Assistant action:** implements separate equality/hash and ordering interfaces,
  callback adapters, StringComparer.Ordinal and a HashMap policy constructor in
  Raven. Preserves callback construction and native UTF-8 ordering. Documents explicit
  policy domains and defers automatic defaults/culture policies. No Raven compiler
  or native-runtime change is made.
- **Outcome:** six focused source scenarios, six comparer metadata checks and five
  map/GC regressions pass, along with editor discovery and API snapshot validation.
  Full library regeneration matches the tested artifact. The website build and its
  18 tests completed before the subsequent author correction. No publication occurs.
  See [evidence](experiments/raven-target/comparer-validation.json).
- **Tooling finding:** the broad metadata probe retained old union expectations.
  Correcting its zero-interface TaskOutcome assertion exposes a removed Result case
  setter assumption. That remaining maintenance is tracked separately; the assistant
  does not claim the broad probe passed.
- **Author correction:** “Skip website build and in future tasks just run the tests
  that are necessary to verify that the work is actually passing. Of course include
  any performance test if necessary.”
- **Assistant response/action:** stops additional website builds and records the
  standing preference in AGENTS.md, API maintenance guidance and the active roadmap.
  Content and snapshot maintenance continue; performance tests depend on a relevant
  performance question.
- **Author clarification:** “The point is that we shouldn't run the full test suite
  unless actually needed to.” **Assistant:** refines the standing rule to targeted
  validation by default, with broader suites/builds when the change or unresolved
  uncertainty justifies them. The current request to skip another website build
  still applies; there is no blanket prohibition on necessary future validation.


## 2026-09-27 — Explicit comparison and a pause to rethink strings

- **Author:** asks for String comparison shortcuts by ordinality and case.
- **Assistant proposal:** String.Compare with explicit Ordinal/OrdinalIgnoreCase
  modes, keeping CompareOrdinal and adding CompareOrdinalIgnoreCase.
- **Author decision:** “Yes, explicit comparison modes”. Then clarifies that .NET
  ergonomics and familiarity matter, but APIs need not look exactly like .NET.
- **Assistant implementation choice:** Unicode default simple folding shared by
  comparison, equality and hashing, retaining native UTF-8/scalar ordering. This is
  the assistant's bounded design choice, not a claim the author selected every
  Unicode detail or that .NET behaves identically.
- **Author follow-up:** directs learning from other frameworks, accounting for UTF-8
  and reconsidering the whole String design: “So implement this, and then we have
  to stop and re-evaluate: what can we do differently, and what can we improve on
  .NET?”
- **Assistant response/action:** implements the requested slice and records a stop
  before additional string feature implementation. The existing library tracker
  owns a keep/change/defer review with concrete examples and explicit tradeoffs.
  StringBuilder and broader casing are candidates after that review, not automatic
  next tasks. .NET remains the ergonomic target; differences require justification.
- **Evidence:** the [comparison contract](ordinal-text.md#explicit-comparison-modes-development)
  documents simple-fold differences and matching artifacts. Two native folding tests, three Raven scenarios, eleven metadata checks and an
  archived ordinal regression pass; library/API snapshots match. See the
  [focused evidence](experiments/raven-target/string-comparison-validation.json).
  Website/full-suite runs are skipped. No complete redesign, new release or website
  publication is claimed.


## 2026-09-27 — Reviewing String as the foundation for selected System.Text work

- **Author:** “Continue with review”. Subsequently stresses that upcoming System.Text
  work depends on getting String's crucial interfaces right.
- **Assistant:** broadens the review from convenience methods to byte/scalar/grapheme
  units, range ownership, conversion progress and errors, Unicode versions and the
  dependencies of builders, codecs and readers/writers.
- **Author clarification:** “We shouldn't copy everything from System.Text, but we
  should at least know what we want to bring over to NeoCLR.” This refines the earlier
  “entire System.Text API” wording; the recorded goal is capability selection, not
  complete .NET namespace or overload parity.
- **Author emphasis:** “We are building a modern api”, and asks to know the impact
  of the current String model, including whether choosing it is a problem.
- **Assistant assessment:** separates UTF-8 storage from grapheme semantics and
  implementation costs. Recommends retaining the model with explicit lower-level
  contracts, records real interop/streaming/indexing limitations and conditions for
  reconsidering scalar Char or UTF-16 compatibility. This is an assessment, not
  evidence of performance superiority or author approval of a redesign.
- **Assistant findings:** current String already has integer grapheme indexing;
  earlier design prose was stale. Exact prefix/search and byte slicing can operate
  inside a grapheme. Joining Chars can change grapheme count, so codec buffer lengths
  cannot inherit Char's unit. Five small language probes make these differences
  concrete; Swift canonical equality is deliberately not neoCLR's exact equality.
- **Assistant recommendations:** retain the native text model; prototype the scalar,
  traversal/range and incremental-conversion boundaries; align Unicode versions;
  reconsider the simple-fold policy's name before dependent expansion. Select a
  small builder, scalar support, improved UTF-8 conversion and incremental codec roles.
  Defer general encoding registries and broad legacy coverage; do not copy fallback
  hierarchies or automatically include regex/JSON work.
- **Actions/outcome:** updates the [existing text design](design/text-abstraction.md#systemtext-foundation-review--2026-09-27),
  current String guide, authoritative roadmap and owning library tracker. Keeps
  [probe sources/results](experiments/text-review/README.md). Recommendations are not
  implemented public APIs or author approval of exact signatures. No runtime or
  compiler change, full suite, website build or publication occurs.


## 2026-09-27 — Separate logical text from serialization and API encoding defaults

- **Author:** distinguishes strings encoded in metadata from class-library preferences
  for UTF-8, and says a character should represent a Unicode character in the system
  without bearing the actual encoding. Users select encoding unless an API supplies
  a default such as UTF-8.
- **Assistant response:** separates artifact serialization, logical text, runtime
  storage, API boundary encoding, foreign representation and interpretation policies.
  Corrects the review's overly broad wording about building System.Text around Char:
  high-level String/Char input is appropriate; low-level buffer progress must name
  its units. UTF-8 storage does not force UTF-8 into every public text signature.
- **Action:** updates the existing text design and owning tracker. Retains the current
  grapheme Char, noting that “Unicode character” does not by itself choose between
  scalars and graphemes. No metadata format, runtime representation or public API is
  changed. Byte-oriented projections remain explicitly encoding-specific contracts.

## 2026-09-27 — Swift-like text and a bounded System.Text foundation

- **Author:** clarifies that Char should hold a visible/user-perceived character,
  not an encoding unit. Subsequently says the intent is the basics of System.Text,
  supporting encoding and perhaps StringBuilder so later APIs can be built on them.
  “We will be more like Swift, I believe, so our API should reflect that.”
- **Assistant response:** retains grapheme Char, distinguishing graphemes from glyphs
  and display width. Makes Swift the primary text-surface comparison, preserving
  .NET familiarity where useful. Narrows implementation recommendations to UTF-8
  conversion progress and evaluating minimal text construction. Public scalar/range
  machinery and the broader capability inventory are not blanket prerequisites.
- **Author input:** provides a ChatGPT proposal for encoding-neutral String, scalar
  and encoding views, opaque positions, strict decoding and explicit normalization;
  that proposal also removes Char and Length and places graphemes in System.Text.
- **Assistant assessment:** treats the pasted proposal as input, not a user decision.
  Supports separating text and encoding, but identifies removing Char as conflicting
  with the author's current Swift-like direction. Canonical-equivalence comparison
  and storage normalization are separate choices. No approval to remove Length,
  indexing or exact equality is inferred.
- **Actions/outcome:** updates existing design/roadmap/theme tracking and website
  future-direction prose. Adds an [application-only boundary experiment](experiments/text-boundaries/README.md)
  with scalar/range and bounded decoder checks. Compilation, import verification and
  three focused executions pass, including all 14 byte splits of the teaching text.
  Records application bridge/fixture limitations; no compiler or System API changes.
  No full suite or website build. Public view names, position semantics and builder
  shape remain open; the supplied proposal is not implemented wholesale.

## 2026-09-27 — Text versus String naming proposal

- **Author:** supplies additional design discussion favoring Text for an abstract
  Unicode value, explicit views, Text/Rune/Bytes/Encoding vocabulary and possible
  namespace reorganization. Asks to add it to the discussion, not implement a rename.
- **Assistant assessment:** records the semantic-readability argument and the
  familiar String alternative. Distinguishes source aliases, public type identity,
  namespace placement and character-sequence semantics. Notes that Swift retains
  String naming, and that the proposal's absence of a default element type differs
  from neoCLR's current grapheme Sequence contract. No change to Char is inferred.
- **Action/outcome:** adds the [open naming comparison](design/text-abstraction.md#open-naming-discussion-text-versus-string--2026-09-27)
  and links it from the existing library tracker. Records concrete bridge coupling,
  migration costs and validation needed if the proposal advances. No code, public API,
  compiler, namespace or milestone sequence changes; documentation consistency and
  whitespace checks only. The preferred name and any alias remain undecided.

## 2026-09-27 — High-level text handling without excessive API layering

- **Author:** clarifies that the objective is a high-level text API whose naming
  challenges developer assumptions. Suggests that C# string/char could still be
  part of lower APIs, while cautioning not to overdo the design.
- **Assistant response:** treats Text as a candidate application-facing abstraction,
  not merely an alias proposal. Recommends explicit lower-level views where needed
  and avoiding redundant text types. Distinguishes C# UTF-16 char from existing
  neoCLR grapheme Char; does not infer approval to repurpose either concept.
- **Action/outcome:** records the [intent and scope guard](design/text-abstraction.md#high-level-text-api-intent--author-clarification)
  in the existing design and theme tracker. Exact names, any compatibility surface
  and migration remain open. Documentation only; no runtime/API changes.

## 2026-09-27 — Correcting the comparison and testing a small text API sketch

- **Author correction:** C# string and char are not simply handling fixed bytes.
- **Assistant response:** acknowledges conflating bytes with encoding units. C# string
  is variable-length UTF-16 text and char is a code unit; the design question is which
  operations and units are prominent in everyday APIs, not whether .NET handles text.
- **Author:** asks for the next safe step and then says “Continue”.
- **Assistant proposal/action:** compares String and Text vocabulary under identical
  contracts for split decoding, construction and extraction. Adds one executable
  application-only consumer to the existing experiment, reusing unchanged boundary
  evidence. A hypothetical Text alias is not compiled or presented as implemented.
- **Outcome:** construction preserves snapshots through append/clear/reuse; joining
  decoded chunks resegments graphemes; optional delimiter extraction hides position
  arithmetic. Exact matching inside a grapheme remains a distinct search policy,
  not a generic high-level default. The [sketch](design/text-abstraction.md#consumer-api-sketch-identical-behavior-two-vocabularies)
  records the smallest foundation recommendation and open naming/codec questions.
  Compilation, import verification and the focused consumer execution pass; unchanged
  boundary evidence is reused. No production API or runtime change; no usability or
  performance superiority claim. No full suite or website build.

## 2026-09-27 — Incremental reader integration and a future text-oriented Encoder

- **Author:** directs continuation after the consumer sketch. During implementation,
  explicitly says an Encoder abstraction will be needed later and must fit neoCLR's
  string/text direction rather than mirror .NET.
- **Assistant decision:** chooses existing StreamReader.ReadToEnd as the first
  library consumer, with a private UTF-8 decoder. Keeps public codec signatures
  provisional; this does not implement the entire experimental carrier surface.
- **Action:** implements bounded conversion and carry, retries unconsumed input
  before reading again, and retains the existing byte bound, BOM and ownership
  contracts. ReadLine is unchanged. Malformed input can fail before EOF, so docs
  record changed failure ordering/cursor behavior and no partial text return.
- **Integration finding:** the rich prototype's readonly carrier and private union
  dependencies exceed current private-library admission. Uses an internal helper
  with existing Result and progress properties; no compiler/bridge change or public
  workaround types. Explicit branch control also avoids the previously observed
  negated-property assignment problem. These remain integration limitations, not
  a selected public API shape.
- **Future direction:** records [Encoder requirements](design/text-abstraction.md#later-encoder-abstraction--author-direction)
  around valid text input, explicit encoded bytes and precise progress. UTF-16 code
  units are not imposed as the input model. Exact type names, error policy and
  resumable input boundaries remain open.
- **Validation scope:** focused reader contracts and buffer/maximum-bound consumers,
  generated implementation and API snapshot checks; unchanged tests are reused.
  A bounded old/new host-cost comparison checks for material regression. No full
  suite, website build, publication or public Encoder/Decoder API is included.

- **Subsequent author correction:** StreamReader and StreamWriter are supposed to
  use specific encoders, with UTF-8 as default.
- **Assistant response/action:** distinguishes the intended selectable-encoding
  contract from today's UTF-8-only constructors. Corrects the next bounded direction
  to establish selection for both adapters, independent conversion state and explicit
  limits/counts. The internal decoder is default-path work, not completion of the
  stream abstraction. No selected-encoding constructor is claimed as implemented.
- **Observed outcomes:** three current reader runs pass, including the maximum
  bound under an expanded instruction allowance. Snapshot checks pass. The small
  diagnostic is slower than the previous implementation; the old maximum fixture
  faults on its whole-input array budget. Records these tradeoffs rather than claiming
  a speedup. See [reader evidence](experiments/text-boundaries/reader-results.json).

## 2026-09-27 — Calendars, language and independent localization

- **Author:** asks to develop the date/time proposal into useful APIs, with an invariant
  culture, Hebrew comparison calendar and two named cultures; requires a separate
  worktree and a commit. Clarifies that the design is provisional and .NET-inspired,
  favoring modernity, with rendering a date to a string as the immediate goal.
- **Author:** asks whether Swedish and Hebrew need separate calendars and formatters,
  names HebrewDateTimeFormat, and requests Hebrew alphabet and invariant/Latin output.
  Points out that each culture has a language and explicitly chooses to include
  system culture discovery now. Corrects “used” to “uses”: Israel uses both Gregorian
  and Hebrew calendars.
- **Assistant:** separates Gregorian/Hebrew arithmetic from Swedish/Hebrew cultural
  presentation. Implements Gregorian as he-IL's default with an explicit Hebrew
  override, separate internal formatter policies, Language, fixed/system providers,
  and Hebrew numeral/month rendering. Date remains an absolute day with Gregorian
  convenience properties. Records narrower complete-year limits and fixed profiles.
- **Author:** clarifies that localization is separate from culture APIs; interfaces
  should be shared across areas, while providers/sources can differ (JSON or resource
  files). **Assistant:** records that boundary; built-in formatting data is the current
  slice, with no resource loader or area-specific localization interface added.
- **Outcome:** implementation and focused validation are recorded in the
  [calendar guide](calendar-globalization.md), executable samples and .NET fixture.
  The Rust comparison passes all 656 Hebrew year starts/month lengths; the Raven
  consumer passes Hebrew/Latin formatting and provider checks. Unified localization
  contracts, full locale resolution and broader formatting remain open.

- **Author follow-up:** asks whether Hebrew dates can convert to Gregorian and vice
  versa. **Assistant:** confirms both projections use the same absolute Date, with
  1 Tishri 5784 equal to 2023-09-16 and range errors outside the Hebrew policy.

- **Author:** requests globalization be featured on the website. **Assistant:** adds a dedicated feature page, homepage entry and guide/API navigation, with tested Hebrew/Latin output and a distinct future-localization section. Publication is a separate operation.

- **Author:** directs a separate DateTime feature page and selects the Time API and time-zone handling as the next slice. **Assistant:** keeps the existing time route as the DateTime page, separates globalization and records the next work without implementing zones in this commit.

## 2026-09-27 — Time offsets, named zones and parenthesized DateTime

- **Author:** says “Continue” after selecting Time and time-zone handling next.
  **Assistant:** implements civil/elapsed arithmetic, fixed offsets, named IANA rules,
  a resolved ZonedDateTime and explicit local mapping instead of guessing through DST.
- **Author:** asks “Isn't TimeOffset a better name?” **Assistant:** adopts TimeOffset
  for the fixed UTC displacement, separate from TimeZone's changing rules.
- **Author:** proposes a nominal DateTime union, initially describing local or offset
  date-time, then specifies “Parenthesized union”. **Assistant:** first spells the
  proposed `DateTime(LocalDateTime | OffsetDateTime)` shape without wrapper cases.
- **Author:** clarifies that the intended second value may be “zone-id time”, leaves
  terminology to the assistant and says offset date-time was not necessarily intended.
  **Assistant:** chooses `DateTime(LocalDateTime | ZonedDateTime)`, retaining named
  zone identity; leaves OffsetDateTime out. This changes the earlier proposal's
  no-DateTime direction, without introducing .NET's Kind flag.
- **Assistant actions:** compiles the parenthesized declaration through Raven and
  extends the neoCLR bridge's exact supported union shape and conditional extraction
  checks. A consumer exposed failed-match output handling; the bridge was corrected.
  Native transition tests and a .NET 10 comparison passed. Final executable and
  artifact validation is tracked in [the Time/zone guide](time-zones.md).
- **Author:** directs committing the completed slice, merging into main and cleaning
  up branches/worktrees. **Assistant:** checks main is clean and plans integration
  with the independently completed text/encoding work; no unrelated work is removed.

- **Validation outcome:** both DateTime variants, union arrays/defaults and the final
  time/zone consumers pass. The expanded signature probe passes 122 checks; guest IL
  cannot mutate TimeOffset storage or construct a private TimeZone. Focused reference
  rendering confirms the pages and records the parenthesized-heading/payload-page
  limitations explicitly. Integration and cleanup occur after these feature checks.
## 2026-09-27 — UTF-8 and strict ASCII as a shared encoding foundation

- **Author:** directs continuation of reader/writer encoding selection. In response
  to an optional choice between UTF-16 interchange and ASCII, selects “UTF-8 + strict
  ASCII”. Stresses that the foundation will affect many APIs, including HTTP.
- **Assistant response:** moves from considering built-in modes toward a shared
  codec interface; avoids prescribing an enum that every consumer must switch on.
  Distinguishes text conversion from HTTP charset policy, framing and field validation.
- **Action:** adds an application-only selected encoding/decoder contract and both
  reader/writer consumers. Default and explicit UTF-8, strict ASCII, decoder isolation,
  owned carry, byte limits/counts, finalization, partial writes and pre-write rejection
  pass in two focused executions. Public System APIs are not modified.
- **Findings and remaining work:** whole-input acceptance with returned String is a
  viable high-level alternative to output-buffer progress. ReadLine, leaveOpen, public
  metadata and typed-error migration remain necessary before production integration;
  future stateful Encoder finalization remains open. Records the shared foundation
  and HTTP dependency in existing trackers without reopening HTTP POC scope.
- **Evidence:** [selection source and results](experiments/text-boundaries/README.md#shared-encoding-selection-application-contract-probe).
  No full suite, website build, runtime/compiler changes or performance claims.

### 2026-09-27 — Shared encoding contract reaches the library

- **Author:** directed continuation after choosing UTF-8 and strict ASCII, and
  emphasized that encoding is foundational infrastructure for HTTP and other APIs.
- **Assistant action:** promoted the tested roles to Encoding, Decoder, Encodings
  and a standard Raven EncodingError union. Added selected reader/writer constructors
  alongside UTF-8 defaults, independent decoder ownership, decoded line boundaries,
  strict ASCII rejection before output and preserved leaveOpen/partial transfers.
- **Outcome:** focused library consumers pass for the built-ins and a deliberately
  expanding application codec; reader regression checks include split scalars and
  the 65536-byte bound. Public reference and implementation snapshots are refreshed.
  This is development work after Preview 10, not a newly published release.
- **Tradeoffs/open work:** owned text results and whole-value byte snapshots keep
  lifetimes clear but allocate. Source quotas and UTF-8 text quotas remain distinct;
  broader codecs need further quota design. Stateful Encoder, minimal construction
  and HTTP charset policy remain separate follow-ups. No String/Text rename or
  widening of the HTTP POC. See [contract](design/text-abstraction.md#shared-encoding-apis-development)
  and [focused evidence](experiments/text-boundaries/encoding-validation.json).

### 2026-09-27 — Report construction candidate evaluated

- **Author context:** asked to continue after the shared encoding integration.
  Earlier direction requested text foundations and possibly a builder, with useful
  features ahead of optimization work. No new builder design was explicitly approved.
- **Assistant action:** selected the roadmap's bounded report-construction evaluation.
  Added an application-only accumulator and a concrete report consumer, checked hard
  UTF-8 quotas and retained snapshots, and compared balanced deferred construction
  with ordinary concatenation. A .NET semantic fixture separates UTF-16 units,
  UTF-8 bytes and grapheme counts for the same report.
- **Observed outcome:** semantic checks pass; the managed prototype is slower at
  both measured sizes. At 1024 pieces/16 KiB, concatenation took 51–58 ms and the
  prototype 561–574 ms. These are local diagnostics, not cross-runtime benchmarks
  or native allocation measurements. [Evidence](experiments/text-boundaries/builder-validation.json).
- **Assistant decision/recommendation:** retain the useful quota/snapshot contract
  as a candidate and defer public builder promotion; do not start optimizing the
  probe. Recommend the already requested Encoder progress contract with a writer
  consumer as the next bounded task. This is not a new author naming decision or
  a permanent priority change. No runtime/public API, website or release change.

### 2026-09-27 — Encoder progress tested with a writer

- **Author context:** continued the foundation work after the builder evaluation;
  the earlier request for an Encoder adapted to neoCLR text remains the direction.
  No new public API spelling or lifecycle policy was explicitly selected by the author.
- **Assistant proposal/action:** test whole-text Accept followed by bounded Drain,
  returning bytes written and a Ready/NeedsOutput/Finished state. The application
  provider retains valid immutable input and converts scalar-safe chunks for UTF-8
  or strict ASCII. A real short-write OutputStream consumer tests explicit Finish,
  stream-only Flush and ownership-only Close.
- **Outcome:** three focused runs and a .NET semantic comparison pass. Checks cover
  one-byte buffers, source ownership, retryable preflight errors, chunk boundaries,
  partial writes and final-output failure. A synthetic trailer proves completion
  must wait for final bytes; it is not a new supported encoding. No runtime/public
  API or compiler change, benchmark, full suite or website build.
- **Assistant recommendation/open work:** promote the factory/progress roles next,
  integrate explicit StreamWriter Finish and document migration for custom Encoding
  implementations. Keep general TextWriter completion, names and exact bounds under
  review. HTTP policy and public builder promotion remain separate. See the
  [contract](design/text-abstraction.md#encoder-progress-and-writer-evaluation--2026-09-27)
  and [evidence](experiments/text-boundaries/encoder-validation.json).

### 2026-09-27 — Encoder contract promoted into the library

- **Author:** asked to continue after the Encoder acceptance/drain evaluation.
- **Assistant action:** implemented Encoding.CreateEncoder, Encoder and immutable
  public progress properties with a standard Raven EncoderState union. Connected
  both built-ins to StreamWriter and added explicit Finish; Flush and Close retain
  their separate stream/ownership roles. Updated the compiler bridge reference and
  API documentation alongside the implementation.
- **Outcome:** four focused public Encoder/writer runs and three selection/line
  runs pass, including independent sessions, strict line rejection, custom final
  output and the maximum writer bound. Evidence is [recorded](experiments/text-boundaries/public-encoder-validation.json).
  No full suite, website build, runtime-default change or performance claim.
- **Compatibility/costs:** custom development Encoding implementations need the new
  factory. Output/drain failures make subsequent writes unusable; custom output
  overflow may follow partial output. Accepted text remains retained until drained.
  Bounded UTF-8/ASCII foundations are complete; broader codecs, general TextWriter
  completion, HTTP policy and public builder promotion remain separately scoped.

## 2026-09-27 — select casing and Int64 API slices

- **Author:** asked to list the next couple of API improvements after Encoder completion.
- **Assistant proposal:** explicit Unicode casing, then one Int64 parsing/decimal
  formatting slice for file sizes/reports; fill missing numeric bounds as companion
  work. Keep builder/HTTP deferred and avoid treating recommendations as commitments.
- **Author decision:** “Continue with those slices”.
- **Action/outcome:** implemented whole-String Unicode 17 full default upper/lower
  casing and strict Int64 Result parsing, decimal ToString and read-only bounds.
  The [design](design/text-casing-integer.md) records .NET differences, existing
  Unicode-version split, naming and costs. [Focused evidence](experiments/casing-integer/README.md)
  covers a runnable report and exact runtime/metadata contracts. No general culture,
  normalization, builder optimization or next milestone was inferred from this direction.

## 2026-09-27 — extend Reflection beyond properties

- **Author:** selected the next issue after date/time integration: constructor discovery,
  MethodInfo.Invoke, FieldInfo.GetValue/SetValue and argument-taking typed/untyped
  TypeInfo.CreateInstance. The author left placement in Reflection open for discussion.
- **Assistant:** proposed GetConstructors as descriptive introspection, with invocation,
  member access and construction as System.Runtime.Reflection extensions. Existing
  parameterless construction/property operations already returned Result.
- **Author clarification:** “Keep Result returns”. Later requested that Reflection have
  its own feature page. The initially sketched direct Object/T returns are not adopted.
- **Action:** develop the [bounded contracts](reflection-members.md), public/native
  consumer checks, source field access metadata and a separate Reflection feature page.
  ConstructorInfo extends the closed MemberInfo family; typed activation checks result
  compatibility before constructor execution. Validation evidence and remaining limits
  are maintained with the feature and its executable consumer.

- **Next author-selected slice:** after Reflection, add System.Runtime.GC with basic
  information queries and collection influence. The assistant proposed starting from
  actual collector capabilities; no GC API is implemented by this Reflection change.

## 2026-09-27 — Number and Parsable interfaces selected

- **Author:** “Now add Number and Parsable interfaces and implement them for the
  common set of types”.
- **Assistant scope proposal:** Number<T> for basic arithmetic, ordering, Zero/One;
  Parsable<T,TError> for static Parse returning Result; eight fixed-width integer
  types plus Single/Double, and Boolean for parsing only.
- **Author confirmation:** “Yes, that scope”.
- **Investigation:** authored static abstract/operator interface contracts and
  neoCLR generic-call import need support before the APIs can be genuinely useful.
  General Raven work is isolated on a main-based feature branch. Implementation is
  in progress; [numeric contract notes](design/numeric-contracts.md) distinguish
  selected scope from completed behavior.

- **Subsequent author correction:** “Put the parsable interface on hold. Focus on
  Number interface and Parse per concrete type”. The author then suggested a
  numeric-specific parsing interface rather than .NET-style broad Parsable. The
  assistant keeps concrete parsing in scope and records ParsableNumber as a future
  candidate, separate from arithmetic; no parsing interface is being implemented.

- **Further author direction:** static interface members, default implementations
  and accessibility within interfaces (including public/private) are important to
  the intended platform. The author expects Raven to supply these language features
  and wants neoCLR to make broader use of interfaces.
- **Assistant response/action:** distinguish Raven semantics, emitted CLI metadata,
  importer admission and runtime dispatch/access checks. General authored static
  contract fixes were tested independently (13 focused cases) and integrated as
  Raven main `d7d69fe69`, then neoCLR branch `82774c30d`. Number remains the first
  selected consumer, not the intended ceiling on interface support. Existing native
  default-body/explicit-mapping support does not establish complete Raven import or
  arbitrary private interface helper support. Broader scope is recorded in the
  [runtime/language tracker](tracking/runtime-language.md#interfaces-as-a-platform-capability).

- **Numeric error correction:** the author directed use of NumberParseError for all
  numbers unless separate errors are needed. The assistant is migrating Int32 and
  Int64 and their current consumers to that shared union, retaining Boolean's
  format-only error because Boolean is not numeric. Earlier plans to preserve the
  separate numeric error types are superseded; archived Neo compatibility remains
  isolated from the current Raven API.

- **Implemented outcome:** Number/ordering/arithmetic for ten primitive numeric
  types, concrete parsing and shared NumberParseError, with a bounded generic
  application consumer. General Raven fixes were integrated independently; the
  [numeric evidence](experiments/numeric-contracts/README.md) records focused
  compiler/native/metadata/consumer checks. Parsable remains on hold. General
  static defaults and arbitrary interface helpers/accessibility remain broader
  follow-up work, not a claimed completion of the interface direction.

- **Parsing capability clarification:** the author questioned the ergonomics of
  Parsable<T, NumberParseError>, preferring ParsableNumber<T>, and then clarified:
  “What we really want is to signal that a number type T has a Parse method in a
  generic method”. The assistant initially emphasized waiting for a consumer;
  the clarification establishes the intended generic capability itself: a
  constraint permitting T.Parse with Result<T, NumberParseError>. The assistant
  recorded that proposed contract separately from Number's arithmetic capability
  in the [numeric design](design/numeric-contracts.md#proposed-numeric-parsing-capability).
  General typed parsing does not inherently require exceptions, but associating
  error types introduces an ergonomic choice. Implementation remains on hold under
  the earlier explicit direction; compiler/importer admission still needs validation.


## 2026-09-27 — Bounded general interface consumer

- **Author direction:** leave the numeric parsing interface discussion and move on.
  The assistant proposed checking a static helper, a default calling a private
  helper and rejected external access; the author replied “Continue”.
- **Assistant investigation:** Raven compiled the first example, but the neoCLR
  importer admitted only abstract public instance contracts. Native nominal defaults
  and static helper admission also needed extension. A private instance helper probe
  emitted Private, Virtual, NewSlot; this is recorded as a general compiler candidate
  requiring independent validation, not silently normalized by the target importer.
- **Action:** implement the bounded public default/private static helper path,
  preserving receiver identity, class precedence and access. Keep private instance
  helpers and wider interface accessibility/virtual-static work deferred. Record
  focused native and Raven validation in the [consumer evidence](experiments/interface-helpers/README.md).
  ParsableNumber remains on hold; its intended inheritance from Number is unchanged.

- **Author follow-up:** asked whether everything around interfaces works, including
  explicit implementations. The assistant distinguished passing native explicit/default
  tests from Raven importer admission: explicit implementation import is still rejected,
  alongside the other deferred forms. Proposed explicit implementations as the next
  focused boundary; did not claim general interface completion.

- **Author decision:** “Record those limitations”. The assistant added an explicit
  capability matrix to the runtime tracker and linked it from the platform roadmap,
  separating native evidence, Raven importer restrictions and next validation needs.

- **Outcome:** 16 focused native tests and the final Raven consumer pass, including
  static-helper reflection and negative access/admission cases; API snapshot checks
  pass. The bounded implementation is complete in development. The limitations
  matrix remains open work, and no full suite or website build was run.


## 2026-09-27 — Explicit interface implementation follow-up

- **Author direction:** “Continue” after the bounded helper slice and recorded
  interface limitations. The assistant selected explicit implementations through Raven,
  including two same-named contracts and ordinary private-access rejection.
- **Investigation/action:** Raven already emitted private/final/virtual/newslot bodies
  and MethodImpl declarations. neoCLR's importer rejected them, and runtime validation
  limited explicit bodies to managed-reference receivers. Add checked mapping import
  and nominal class receivers without changing compiler semantics or target settings.
- **Scope:** ordinary methods on non-generic application classes/interfaces; include
  shared state, void results and private reflection. Preserve the documented IsVirtual
  divergence from CLI. Broader explicit accessors, value/generic types, core-library
  contracts and derived defaults remain recorded limitations, not inferred completion.
  Commands and outcomes belong to the [focused experiment](experiments/explicit-interface-implementations/README.md).

- **Coordination:** the author asked to coordinate reflection changes. The assistant
  contacted the reflection task and the overlapping entry-point task, preserved
  unrelated Raven and neoCLR edits, and arranged to commit this bounded interface
  slice before their main-branch integration. Shared files are staged by owned hunks;
  the API snapshot is checked from staged sources rather than unrelated in-progress
  bridge edits. This does not claim that reflection has already been integrated.

- **Outcome:** 15 focused native tests pass. The Raven consumer exposed encoded
  implementation names in reflection; the assistant preserved source-qualified
  names for explicit bodies without changing dispatch identities. Final execution
  and both negative source cases pass. The staged-source API snapshot check passes;
  no full suite or website build was run. Reflection integration remains coordinated
  separately rather than included in this bounded implementation.


## 2026-09-27 — Result and Task entry points

- **Author request:** “We need to support Main returning the Raven-supported Result
  and Task overloads.” The assistant found the target importer restricted entries
  to no-result methods, Raven's default bridges depended on CLR facilities, and the
  CLI discarded integer exit statuses.
- **Assistant decision/action:** retain target Main signatures in Raven's heap-async
  intermediate metadata and adapt startup in neoCLR's existing importer. Reuse
  managed Result extraction/display and the default task dispatcher. Keep .NET
  bridges and ordinary nonblocking GetResult unchanged. Details, limits and evidence
  are in [entry results](experiments/entry-results/README.md).
- **Author follow-up:** after the fix is ready, update samples and experiments to
  use async directly in Main. The assistant accepted that sequence, preserving
  fixtures whose purpose is callback/queue behavior. This does not authorize new
  HTTP capabilities or imply a permanent roadmap reprioritization.

- **Author delivery direction:** commit the completed changes and document supported
  entry points on the website. The assistant added a development-only signature
  table and exit/cancellation behavior to the Raven page, linked from Tasks.

- **Validation/outcome:** the author explicitly required updated samples to build
  and run. Both async samples and the HTTP client do so; 25 entry cases and focused
  compiler/native tests pass. Raven's target fix is committed as `d6f04b751` on
  `neoclr`. The affected website pages and API snapshot pass checks; a full website
  build encounters an existing unrelated LocalTimeMapping API coverage mismatch.
  Two independently reproduced async compiler gaps are recorded for separate work.
  No website publication or general interface/reflection completion is implied.

## 2026-09-27 — Basic GC information and control

- **Author:** requested the slice after Reflection: “Add System.Runtime.GC class with
  basic capabilities for querying information from GC and influencing the GC”.
- **Assistant proposal:** reuse actual object counters, synchronous collection and
  KeepAlive; explain execution/worker isolation and avoid implied byte or generation
  accounting. Put the facade in the requested namespace.
- **Action:** implemented the [provisional contract](runtime-gc.md), shared automatic
  and explicit collection root tracing, private exact services, Raven facade, focused
  tests and [public consumer](experiments/runtime-gc/Main.rvn). Broader GC tuning and
  memory diagnostics remain open; the request does not reprioritize the full roadmap.

- **Author follow-up:** requested garbage collection documentation on the website.
  **Assistant action:** add a dedicated feature page with tested examples, counter
  meanings, lifetime semantics and limits, alongside the API reference.

- **Delivery outcome:** Reflection was committed as `3b927cb6`; GC and the completed
  numeric/interface integration were committed as `8acef987`. The final entry-point
  integration preserves GC roots through queued callbacks and async Main resumption.
  Focused native, source-consumer, API/library snapshot and website checks pass.
  The website was built locally, not published. Main integration and task worktree
  cleanup are performed after these checks.

## 2026-09-27 — Task.Run as canonical shared-context work submission

- **Author:** asks to add the documented Task.Run API, leaving execution policy to
  the runtime and suggesting a thread as a possible initial mechanism.
- **Assistant:** asks whether to start with noncapturing transferable values or
  include shared captured objects, explaining that the latter requires wider
  heap/thread-safety work.
- **Author:** selects “Include shared captured objects now”, then clarifies:
  “Task.Run will be our canonical API for creating tasks that can capture context.”
  Leaves Thread's public future uncertain and suggests it as a backend primitive.
- **Assistant:** distinguishes lexical capture/shared identity from ambient context
  flow, identifies the existing single-owner heap and isolated worker mismatch,
  and asks whether native threads are required now or cooperative execution is
  acceptable.
- **Author:** asks for the assistant's recommendation and adds, “We should align
  with the behavior of Task.Run in .NET I thin”.
- **Assistant recommendation:** use native-thread execution initially so blocking
  work permits caller progress, preserve shared object identity, and include async
  callback unwrapping. Keep pooling/dedicated-thread policy internal and retain
  neoCLR's Result/cancellation/terminal-Fault model. Exact implementation and
  overload evidence remain pending.
- **Action:** records the direction, alternatives, costs and focused acceptance in
  the [concurrency design](concurrency-direction.md#taskrun-with-shared-captures--author-direction-2026-09-27)
  and [runtime tracker](tracking/runtime-language.md#author-selected-taskrun-work--2026-09-27).
  Runtime implementation and validation remain pending; no Task.Run API is claimed
  complete by this design record.

- **Later author direction:** “In the future we might have something similar to
  green threads in the runtime.” The assistant records this as a possible backend
  that must preserve the selected sharing/progress contract, not a replacement of
  the current native-thread implementation direction.
- **Implementation follow-up:** integrates the Reflection/GC baseline and implements
  synchronized managed storage as the first prerequisite. This preserves weak heap
  handles and identity, and avoids holding storage locks during debugger reference
  traversal. Public Task.Run, shared GC coordination and task publication remain
  unfinished; native storage checks alone do not establish that full contract.
  109 focused native/integration checks pass. A small release-build class-field
  probe records about 4% overhead; results and limitations are retained with the
  [storage evidence](experiments/task-run/README.md). No full suite or website build
  was run for this prerequisite.

## 2026-09-27 — Reusable attributed route union prototype

- **Author direction:** proposed RoutePattern attributes on union cases, clarified
  the possible unmatched marker as UnmatchedRoutePattern, and requested preparation
  once for the server lifetime. The author then delegated whether Result should
  signal unmatched routes and asked to continue.
- **Assistant choice:** use Result<AppRoutes, RouteMappingError>, separating NoMatch,
  malformed target and failed conversion; omit an unmatched attribute initially.
  Select emitted-metadata generation above the existing primitives. Schema binding
  is checked at build time and patterns compile once in Create; this adds a build
  step while avoiding runtime attribute discovery. This is an experiment rather
  than a claim of completed SDK integration.
- **Actions:** a reduced probe exposed silently omitted Raven case attributes.
  Fix and test this independently on Raven main (`3179cd21e`, 16 attribute tests,
  189 union tests), then integrate only that fix onto neoclr (`2f62361ef`). Add
  neoCLR's bounded Int32 carrier projection without arbitrary explicit-layout
  aliasing. Implement the generator, typed Result errors and item HTTP case.
- **Scope:** String/Int32 and empty cases, exact parameter names, rejected structural
  overlap, reusable patterns and application-owned dispatch. Enum/Uuid/Option JSON,
  automatic SDK integration and broader attribute reflection remain open.
  [Executable recipe and evidence](experiments/route-union-mapper/README.md).

- **Author correction:** “Perhaps we should add runtime attribute reflection,”
  followed by the explicit preference to use reflection at application startup and
  cache the mapped information. Source generation should be considered in future.
  **Assistant response:** adopt startup reflection as the immediate next direction;
  retain the generated prototype as evidence, not as the intended permanent workflow.

## 2026-09-27 — Member and parameter attribute data

- **Author:** asked to add attribute retrieval to the introspection model for members
  and parameters, similar to .NET, then asked whether multiple attribute declarations
  are handled. This refines the startup-reflection direction for route mapping.
- **Assistant choice/action:** add GetCustomAttributesData on MemberInfo and
  ParameterInfo, immutable type/constructor/typed-argument descriptions and source
  token retention. Read data without executing attribute constructors; preserve
  multiple declarations and repeated allowed attributes in metadata order. The
  initial application attribute constants are String/Int32/Boolean, with unsupported
  constants and named arguments rejected explicitly rather than silently omitted.
- **Validation:** the public consumer checks every available member category,
  parameters, union cases, separate/grouped/repeated attributes, repeated reads and
  snapshot copies. Attribute constructors deliberately Fault. Native tests cover
  serialization, malformed data and token ownership. A real catalog-route declaration
  is inspected once at startup for the website example.
- **Remaining:** cached dynamic union construction is the next mapper slice. Broad
  framework/CLI attribute coverage and named/type/enum/array arguments are not claimed.
  A nullable-string attribute constructor triggered Raven's CustomAttributeBuilder
  emission exception and is recorded for independent investigation. Source generation
  remains a future option; enum/Uuid/Option JSON requests remain pending.


## 2026-09-27 — retained constructor invocation for startup route mapping

- **Author:** asked to continue after the multiple-attribute introspection slice.
  Earlier direction selects runtime reflection at startup and cached mapping, with
  source generation reserved as a future alternative.
- **Assistant:** identified class-only reflective construction and unavailable
  application value-constructor descriptors as the next bounded prerequisites.
  Added exact ConstructorInfo invocation, value-case/carrier boxing, and bridge
  constructor wrappers retaining source identity. The sample prepares descriptors
  from a route attribute once and reuses them to construct ordinary Raven unions.
- **Scope:** descriptor selection is retained; arguments and access are checked per
  call. There is no claim of a compiled native plan or completed general route
  mapper. General schema validation/capture binding is next; enum/Uuid/Option JSON
  requests remain open. Evidence and validation are recorded with the
  [construction case](experiments/union-construction/README.md).


## 2026-09-27 — reusable startup route mapping

- **Author:** asked to continue the attributed route and introspection work. Earlier
  direction selects startup reflection with cached mappings and leaves unmatched
  handling to assistant judgment.
- **Assistant action:** composed attribute data, route primitives and retained
  constructors into a reusable application mapper, with a typed AppRoutes facade.
  Added capture-name snapshots, conservative structural overlap checks, dynamic
  constructor argument collections and effective type visibility for startup validation.
  The catalog server prepares once, then matches ordinary union cases; its client
  lists items and fetches item 42. No generation step or hosting framework is needed.
- **Outcome:** focused consumer, schema-rejection and HTTP evidence is recorded with
  the [runtime mapper case](experiments/runtime-route-mapper/README.md). Discovery is
  cached; target decoding, allocations and checked construction still occur per request.
- **Open:** SDK packaging and source generation are future options. Enum, Uuid and
  Option JSON support remains requested; this slice does not implement those types.


## 2026-09-27 — Integrating completed Task.Run work

**Author:** Asked to integrate completed changes into main, then clarified that this
means neoCLR main while keeping compiler work separate in Raven. Subsequently asked
for applicable Raven work on both main and neoclr, qualifying that with “If it applies
to both”.

**Assistant:** Prepared the neoCLR merge in the Task.Run worktree while the website
chat completed its changes in the main checkout. Preserved main's routing, typed JSON
arrays and reflection-constructor work alongside shared-task submission. The direct
unit-await compiler correction is target-specific: the ordinary CLR control using
configured System.ValueTuple passes unchanged, so no general main-branch fix is
claimed. The independently validated shared-capture fix is already on both Raven
branches. The author’s conditional integration direction preserves independent
validation of general compiler fixes; it does not request a wholesale target merge.

**Validation:** The direct-await slice passes 24 Raven neighbors and three neoCLR
compile/verify/run consumers. The combined neoCLR merge passes 15 VM suspension and
nine reflection checks, eleven Task.Run signature checks and the compiled
capture/identity/unwrapping consumer against the combined artifacts. Its combined
source/generated-library fingerprints and API snapshot are refreshed. See the [Task.Run checkpoint](experiments/task-run/README.md#direct-unit-await-checkpoint)
for scope and remaining lookup/inference work.

## 2026-09-27 — semantic API member browsing in RavenDoc

- **Author:** requested Raven-semantic reference pages with inherited/implemented
  members, linked interface origins and default implementation information, union
  case pages, companion merging, nominal generic/delegate labels, authored content
  overlays and closed hierarchy relationships. Selected ArrayList as the review
  model and requested a continuously served preview.
- **Refinements:** initially requested separate extension sections, then selected
  inline Methods/Properties grouping with a distinct E icon overlay. Grouping by
  declaring type uses the extension's declaring container. Requested an inherited
  toggle, red-ish S badges, no expandable unions/enums in the sidebar, and static
  members only on their declaring type. These later directions supersede the
  earlier extension-section proposal.
- **Assistant actions:** implemented the shared generator behavior, sample and docs
  in Raven, with separate general compiler symbol corrections. Integrated the shared
  changes into main and neoclr. Browser testing found and corrected stale group
  counts and generic interface labels. Confirmed ReferenceEquals is absent from
  the refreshed InvalidFormat and ArrayList pages, retaining inherited instances.
  The author identified the release notice as a scrolling culprit and JsonNumber
  as an overflow example. Measured sidebar space below the actual notice/header
  position and wrapped long declaring-type headings; verified menu-end access
  without article scrolling. Also constrained the matching page-outline links;
  verified JsonNumber without horizontal overflow at 390px and 1280px.
- **Evidence:** [integration and validation](ravendoc-integration.md),
  [pinned publisher](../tools/ravendoc/version.json). The local preview is not a
  public deployment. Separate runtime/reflection development remains independent.

### 2026-09-27 — Compact documentation typography

- Author: the fonts feel too large on a 13-inch screen, especially code blocks;
  asked for a balanced technical-documentation scale across the site. Shared
  defaults belong in RavenDoc and must remain overridable.
- Assistant: set relative-unit defaults equivalent to 15px prose and 13px code,
  remove neoCLR’s oversized article-title override, and retain a distinct landing
  hero scale. Document the CSS variables and demonstrate defaults in the sample.
- Author feedback on the preview: “It’s better now”.
- Review: light/dark desktop and narrow-screen checks preserved highlighting;
  a long inline expression exposed mobile overflow, addressed by wrapping inline
  code while keeping code-block scrolling.

## 2026-09-27 — Raven contracts and platform compatibility

**Author:** Reaffirmed that general fixes belong to Raven, with shared fixes on both
main and neoclr and platform-specific differences isolated on neoclr. Asked that we
also determine whether neoCLR lacks compatibility, then clarified that the baseline
is “the contract that Raven normally follow”. Observed that porting to neoCLR has
uncovered many bugs in Raven proper.

**Assistant:** Distinguished invalid Raven output, missing neoCLR support for valid
Raven contracts, and intentional platform differences. Proposed independently
validating general fixes on ordinary .NET, then checking neoCLR consumers without
weakening import or verification to conceal runtime gaps. The current generic
capture reproduction fails on ordinary .NET before the fix; it is compiler-owned.
A separate async generic-containing-type failure also reproduces without captures.
See the [Task.Run checkpoint](experiments/task-run/README.md) for implementation
and validation outcomes; broader .NET library parity is not inferred from this direction.

### 2026-09-27 — Reverse type relationships and extension visibility

- Author requested links to types inheriting or implementing the current type,
  clarifying that interfaces inherit interfaces whereas classes implement them.
- Assistant implemented separate Derived interfaces and Implementing types
  sections, plus Derived types for classes, scoped to the documented surface.
  Generic and indirect relationships are included; indirect entries are marked.
- Author then requested a Show extension members toggle. The assistant added
  a persisted, initially enabled filter independent of inherited-member visibility.
  Browser checks on ArrayList confirmed both grouping modes, counts, persistence
  and independence; source/imported documentation tests cover type relationships.

- Follow-up: the author asked for the toggles to sit next to one another when
  space permits, and for the grouping and both toggle values to be shareable in
  the URL. The assistant implemented wrapping controls and URL precedence over
  saved preferences, preserving unrelated query parameters and page anchors.

## 2026-09-27 — Release-readiness review before further expansion

**Author:** Asked to run more tests in anticipation of a release, first identifying
features needing more work and general bugs. Clarified that main is now the working
branch.

**Assistant:** Selected main a79d0b34 as the assessment baseline, broadened native
and integration checks, rechecked old compiler observations and separated feature
scope decisions from confirmed gate failures. Tests initially ran in the clean
Task.Run checkout at the same commit; the assessment is maintained on main. Proposed
fixing canonical gate/probe failures and silent wrong behavior before adding optional
features, retaining one canonical full-suite run and focused host/package checks.
See the [release tracker](tracking/toolchain-release.md#pre-release-assessment--2026-09-27)
for findings and evidence. No new release version or publication was selected.

**Follow-up author decision:** “We should fix the generic async support.” Generic
async import is now a pre-release requirement, rather than a recommended optional
limitation. The assistant will repair neoCLR’s state-machine/closure import path
against Raven’s normal contract and keep the independent Raven generic-containing-
type emission issue separate unless needed by that implementation.


## 2026-09-27 — Async/await as a release requirement

**Author:** “Having a functioning async await support is crucial for this release.”
This follows the explicit request to fix generic async support.

**Assistant:** Agreed to treat observable async behavior as a release gate and
prioritize it over optional APIs. Implemented bounded generic application-type and
callback import, preserving Raven's normal metadata. The first consumer returns
`42` and `after`; a stronger consumer forces two suspensions and checks values,
shared array/object identity and cancellation. Existing Task.Run regression checks
cover unit/typed work, mutable captures, unwrapping and failures.

The stronger consumer exposed a distinct Raven signature-emission defect for
constructed source types inside imported generic types. The assistant reduced it
against ordinary .NET, fixed it on an isolated main-based branch, verified 16 focused
compiler checks, and integrated general commit `4c8d60176` into main and individually
as `4cfc75b4e` on neoclr. No target-specific branch was merged wholesale.

**Remaining:** This is a generic-method compatibility checkpoint, not a release
certification. Generic-containing-type async arity and silent async interpolation
remain open in [release tracking](tracking/toolchain-release.md). A separate
System.Runtime target-metadata async-attribute lookup failure discovered during
reduction is recorded in Raven's compiler documentation. See
[the implementation/evidence](experiments/task-run/README.md#generic-async-application-import).

**Follow-up action:** Fixed Raven's enclosing generic state arguments and implicit
field receiver after await, with 35 focused ordinary .NET checks; integrated general
`70cc9dfae` into main and individually as `7f35ba31c` on neoclr. The neoCLR
GenericOwner consumer now suspends and updates original int/string receivers.
Generic methods on generic owners remain an importer boundary. A distinct generic
private-var explicit assignment diagnostic is recorded in Raven.

**Author delivery request:** After finishing, build and locally install the matching
SDK and VS Code extension, then launch an HTTP client/server application. The
assistant will use fresh versioned local artifacts and validate the installed pair.
This requests local installation; it does not certify or publish a release.

**Packaging finding:** The first fresh SDK build failed compiling Raven.Core
WithContext. The new enclosing-argument logic included a semantic extension owner
that emits as a nongeneric CLI container. The assistant reproduced this in two
existing ordinary .NET tests, corrected the owner boundary and passed 29 focused
checks. General `768fdcd7a` is integrated into Raven main and individually as
`b7bc6838d` on neoclr. The first failed artifact attempt is not installed.

**Delivery outcome:** Built and installed SDK/VSIX 0.1.12-neoclr.20260927.async2,
with all 440 SDK files verified against build output and the extension version
confirmed in both VS Code installations. Installed completion, three generic async
consumers, independent HTTP requests and the Raven pair pass. Opened the HTTP
workspace and separate server/client terminals; the client returned accepted:true
and the server was left listening on loopback port 61515. See the
[local snapshot](local-sdk-snapshot.md) for paths and evidence. No release was
published; silent interpolation and canonical gate failures remain tracked.

## 2026-09-27 — Investigating sample server CPU

**Author:** Asked to run the sample server, investigate high CPU and determine
whether optimization is needed. This follows the preference to optimize only when
it materially matters.

**Assistant actions:** Ran fresh installed servers, measured idle and request CPU,
collected native sampling profiles, repeated small GET/POST requests and a header
size sweep. Identified two older Raven language servers near one core each; did not
terminate them or infer a managed root cause. HTTP steady idle used 0.4–0.9% over
ten-second windows; requests used about 1–1.7 CPU seconds and 1 KB headers over
three CPU seconds. A 2,049-byte header reproduced the sample's fatal limit error.

**Assistant recommendation:** Repair per-request error handling, then profile-guided
loaded metadata lookup caching/indexing; retain quota, identity and access contracts.
Reassess synchronization after measuring that bounded change. This is an investigation
and recommendation, not an implemented optimization or approval of a broad redesign.
See [evidence and limitations](experiments/http-json/cpu-investigation-20260927.md).

## 2026-09-27 — A maintained neoCLR and .NET comparison

- **Author direction:** asked for a comparison page, or to keep an existing one
  current; clarified that a short summary and feature table should highlight both
  similarities and differences and link to articles explaining changed features.
- **Assistant action:** expanded the former About-page summary into
  [neoCLR and .NET](../website/content/comparison/index.md), separating shared
  foundations, deliberate contract differences, and implementation/coverage gaps.
  Added links to feature articles and primary .NET documentation, with a review
  date and development-versus-release labels. About and Guides point to the page.
- **Maintenance:** the [website workflow](design/feature-pages.md) now requires
  reviewing this comparison when a relevant contract, limit or release changes.
  This is documentation maintenance, not a change to platform priorities or a
  claim of full .NET compatibility or superior performance.


## 2026-09-27 — Proceed with measured HTTP CPU slices

- **Author:** “Process with these slices.” This followed the request to run the
  sample server, investigate high CPU and decide whether optimization was needed.
- **Assistant actions:** Implemented rejected-connection recovery and an
  execution-scoped metadata candidate index. The repeated local comparison measures
  roughly 24% lower median GET/POST CPU, with low steady idle and unchanged limits.
  Focused contracts and thirteen Task.Run consumers pass.
- **Separate Raven outcome:** Managed stacks exposed repeated workspace discovery;
  a failing regression was repaired by excluding generated output trees. The general
  fix was integrated independently into main and neoclr. Two old hot language servers
  were stopped after stack capture; a separate input-loop diagnosis remains open.
- **Assistant assessment, not another author decision:** Fresh profiles justify
  considering incremental quota aggregation next, but not weakening quotas or
  removing locks. Startup and the remaining old editor memory/input issue are separate.
  See [implementation, measurements and boundaries](experiments/http-json/lookup-comparison-20260927.md).

- **Installation outcome:** SDK/VSIX cpu1 and the fresh indexed-runtime bundle were
  installed alongside prior bundles. Editor completion, four independent HTTP cases
  and the Raven client/server pair pass. The updated workspace was opened and the
  visible pair launched on port 63796; the client returned accepted:true and the
  server remained running. [Exact provenance](experiments/http-json/local-cpu-toolchain-validation.json)
  distinguishes the SDK revision from a concurrent editor-only completion-order
  change included in VSIX packaging. This is not a published release.


## 2026-09-27 — Release continuation and Windows candidate

- **Author:** “Continue with release”, then “We might want to release it for windows also”.
- **Assistant plan:** Freeze the current bounded capability scope, repair confirmed
  release gates and run canonical source validation once, with focused host checks.
  Prepare Windows x64 native-runtime packaging and extracted sample execution; full
  Raven SDK/bridge packaging on Windows is a separate, unqualified target.
- **Actions so far:** Formatting/strict lint pass. The stale signature probe was
  updated for standard unions; 639 checks pass. The silent interpolation issue was
  split into a missing neoCLR object Concat overload and a general Raven diagnostic
  bug. Focused tests pass; the Raven repair is integrated independently into both
  branches. Native package extraction exposed a stale Result sample, now corrected.
- **New qualification finding:** Regenerating the full library with the current
  compiler exposed a delegate importer that discarded generic Void results before
  the caller's own pop. Correct the CLI contract rather than altering source callbacks.
  Full regeneration and package checks pass. The hosted Linux suite passes 1,659
  tests; macOS/Windows host checks and minimum-Rust checks pass. Later sample
  validation exposed a stale zero-exit assumption, corrected and verified against
  all 25 extracted source/artifact samples without repeating unaffected full tests.
- **Release outcome prepared:** Preview 11 carries macOS arm64 Raven tools and a
  Windows x64 native-runtime ZIP. Windows execution and extracted hashes pass;
  PE imports require the Microsoft Visual C++ x64 Redistributable. Windows Raven
  SDK/bridge qualification remains open. See the [release evidence](preview-11-validation.json).
  Assets are staged in an unpublished GitHub draft pending this final documentation
  commit and publication; this entry does not claim publication already happened.

## 2026-09-27 — HttpServer as an application-owned accept loop

- **Author:** illustrated listening once and repeatedly awaiting Accept, leaving
  the application model inside the loop, then asked to incorporate this framing.
- **Assistant:** clarified that Accept yields an HTTP request context rather than
  a raw socket, and that sequential or concurrent handling needs an explicit
  application policy. The current implementation closes each exchange connection.
- **Action:** added the model to the Web guide and extracted the existing bounded
  report-server accept loop into its case page. The explanation covers rejection,
  completion, cleanup and shutdown without proposing a new HttpServer API.


## 2026-09-28 — Tuple value types and isolated Raven integration

- **Author:** “We should add tuple support via System.Tuple (which will be the name
  corresponding System.ValueTuple in .NET)”. This selects the public name and value
  representation; it does not specify every ValueTuple API or tuple arity.
- **Assistant proposal/action:** traced Raven's tuple binding/emission and neoCLR's
  generic value admission, then selected an initial one-through-seven-component
  family with public Item fields, constructors, tuple syntax, labels, copying and
  nesting. Kept empty parentheses on the existing System.Void unit contract.
  Wider flat tuples and additional interfaces remain an assistant-scoped boundary,
  not an author rejection of those capabilities.
- **Author correction:** “You might need a separate worktree for work in Raven repo”.
  The assistant created Raven-neoclr-tuples on codex/neoclr-tuples, based on neoclr,
  preserving the original checkout and its untracked investigation file.
- **Assistant finding/action:** nested tuple metadata exposed a general Raven failure.
  Created Raven-tuple-metadata from main, reproduced it with standard .NET ValueTuple,
  and independently validated the fix with 31 focused tests including execution.
  Commit 90b996b1b was fast-forwarded into local Raven main and cherry-picked into
  the isolated target worktree as ee3a23d15. Target naming policy remains separate.
- **Author reaffirmation:** “Make sure that what should be backported to main/.NET is
  cherry picked over - fixes and such”. The assistant confirmed the general fix was
  already on main and committed to keeping further general fixes independent.
- **Outcome/evidence:** see the [tuple experiment](experiments/tuples/README.md) for
  actual consumer, metadata and importer validation. The .NET baseline comparison
  and costs are in the [contract](tuples.md). This is development work after Preview
  11, not a release, SDK installation or website publication. Target identity policy
  is isolated in Raven commit 79a5d7ffc; the original Raven checkout remains unchanged.
  The Web API direction remains unchanged.
  The broader source-ownership audit has two reproduced pre-existing failures;
  an eight-element tuple probe fails at emission rather than with a specific arity
  diagnostic. Both are recorded as open limitations, not passing checks.

## 2026-09-30 — Separate Self integration from structural types

The author directed integration of Self into Raven and neoCLR while retaining
structural/function-type work on feature branches in both repositories, then
requested branch cleanup and remote synchronization. The assistant extracted the
Self delta onto neoCLR main rather than merging Function ancestry, and added an
explicit neoCLR-only compiler gate. Rebuilding main's nominal callbacks showed
that inhabited unit-result transport is independently required by the nominal
Func ABI; that compatibility rule stays shared. Structural metadata and complete
compiler support remain deferred. Validation evidence accompanies the integration;
remote synchronization does not publish a runtime bundle or website.


## 2026-09-30 — .NET metadata API and potential Raven foundation

- **Author:** “We should have a good enough API for .NET so we can later integrate it
  with the Raven compiler.” The author also said the pending Metadata Introspection
  API would potentially be built on a Raven port of the Metadata Reader and Writer API.
- **Assistant proposal/action:** continued the .NET library with a typed reference-profile
  facade: Read/Create/Write, owned decoded documents, local section/use validation and
  explicit catalog resolution. Recorded compiler-readiness criteria and the potential
  Raven port/projection boundary in the [design](design/extended-cli-metadata.md#net-api-direction-and-potential-raven-port-2026-09-30).
- **Outcome:** the focused .NET consumer passes 36 shared profile cases, including 29
  rejections, and independent emission/ownership checks. This implements an experimental
  one-root metadata profile, not complete assembly IO or Raven compiler integration.
  [Evidence](experiments/extended-cli-metadata/dotnet-profiles-validation.json).
- **Open:** physical CLI binding, compiler adapter validation, complete assembly writer
  coverage and the Raven port/Metadata Introspection API remain pending. “Potentially”
  is retained as an option; no port language/API implementation is inferred as complete.


## 2026-09-30 — Cecil as a metadata API reference

- **Author:** “We perhaps should model the metadata API:s on Cecil.” This was a
  suggestion while the .NET artifact reader slice was underway, not an explicit
  requirement to fork Cecil or reproduce every API.
- **Assistant response/action:** reviewed pinned Cecil 0.11.6 assembly/module/type
  reference sources, proposed a Cecil-inspired object-model layer over the existing
  codecs, and recorded alternatives, structural-type differences, costs and open
  mutation/lifetime choices in the [design](design/extended-cli-metadata.md#cecil-inspired-object-model-direction-2026-09-30).
- **Outcome:** completed the separate bounded .NET artifact reader with 50 shared
  cases, including 44 rejections. The higher-level Cecil-inspired model remains a
  proposed next prototype; no new dependency, compiler integration or Raven port is
  claimed. The earlier potential Metadata Introspection direction is preserved.


## 2026-09-30 — Cecil-like API selected as the primary compiler abstraction

- **Author:** “The Cecil-like API will be our primary abstraction for manipulating
  metadata and especially PE assemblies - for the compiler.” The author allows
  modifications as needed and explicitly asks for tests in C#.
- **Assistant action:** recorded this as the selected direction, superseding the earlier
  provisional orientation while preserving that history. Continued the object model
  with AssemblyIdentity, physical AssemblyReference and explicit IAssemblyResolver;
  added a standalone C# contract-test executable generating real PE metadata fixtures.
- **Outcome:** 11 C# tests pass and the existing CLI model consumer passes. The model
  now resolves assembly dependencies by exact metadata identity through host policy.
  [Design and validation](design/extended-cli-metadata.md#primary-compiler-abstraction-and-c-contracts-2026-09-30).
- **Open:** physical type-reference resolution, import, mutation/builders, PE writing,
  full Raven compiler integration and a possible Raven port remain pending. Selecting
  the primary API does not imply these operations are already implemented.


## 2026-09-30 — API-produced assembly must execute in neoCLR

- **Author direction:** “Continue implementing until you have something more or less
  complete.” The author then clarified: “It's not complete until you can load something
  that has been produced by the API - something that conforms to the assembly and
  metadata format - into neoCLR.” They identify that as the proof for Raven integration.
- **Assistant response/action:** made actual runtime acceptance the gate rather than
  another .NET-only codec round-trip. Added physical nominal references, controlled PE
  assembly/type/method builders, imported calls and C# producer/validation tests.
  Clarified that today's runtime accepts native artifacts and the supported Raven route
  uses the CLI bridge; direct #Neo runtime loading is still missing.
- **Observed outcome:** an API-produced PE application calling an API-produced PE library
  was imported through the matching bridge, verified by neoCLR and executed with
  `=> Int32(42)` and exit code 42. A C# runner automates PE production, bridge import,
  native serialization and runtime load/verification/execution. Initial stale-bundle
  attempts failed on entry/core contracts; matching bridge/core/System inputs resolved
  them. The macOS build required an explicit compatible SDK path.
- **Scope:** this proves a bounded static Int32 compiler producer route; it is not full
  Raven codegen integration, arbitrary PE rewriting or native structural execution.
  [Design and scope](design/extended-cli-metadata.md#end-to-end-compiler-producer-baseline-2026-09-30).

## 2026-09-30 — Native functions and metadata-first compiler sequencing

- **Author correction:** “NeoCLR metadata will support functions outside of types.”
  The author then directed a working metadata format first, integrations in the
  refactored compiler second, and improvements such as structural types afterward.
- **Assistant response/action:** made top-level functions first-class assembly-owned
  declarations with nullable type ownership. Added direct native format-5 emission from
  the same model, alongside PE output. Local globals have a CLI representation; native
  emission also supports cross-assembly top-level calls. Preserved earlier structural
  experiments as future-facing evidence rather than a gate for compiler integration.
- **Observed outcome:** the C# API produced two native assemblies that neoCLR loaded,
  verified and executed directly, without the CLI importer, returning 42. The same
  runner confirms missing/wrong-revision dependency rejection. Fifteen C# contract
  groups pass. [Design and evidence](design/extended-cli-metadata.md#top-level-functions-and-direct-native-emission-2026-09-30).
- **Open:** expand ordinary metadata coverage to the refactored compiler's actual
  requirements and then implement its adapters. Structural runtime integration,
  direct PE/#Neo loading and the Raven port remain separate later work. This result
  establishes the bounded acceptance baseline, not a complete general compiler backend.

## 2026-09-30 — Use the refactored compiler as the metadata integration consumer

- **Author direction:** integrate with activity `01a0f154-2448-7df3-8536-c837097b46c2`
  once the metadata work is far enough. The author explained that this supplies an
  end-to-end case for testing and implementing the rest of the format's support.
- **Assistant response/action:** read **Refactor Raven for platform targets** and
  inspected its current semantic-loader, imported-assembly and emitter boundaries.
  Recorded a small compiler-to-native-runtime consumer as the next integration
  experiment, rather than making complete metadata coverage a prerequisite.
- **Current outcome:** member-reference resolution passes 21 C# contract groups, and
  the earlier native producer/load proof remains available. Raven's composition is
  still .NET-owned; native adapters are not yet installed. Inspection was read-only.
- **Next:** use the named compiler activity for a bounded import/emission/runtime case
  and let its unsupported cases drive subsequent ordinary metadata work; structural
  extensions follow that integration. [Boundary inventory and proposed case](design/extended-cli-metadata.md#refactored-compiler-as-the-next-end-to-end-consumer-2026-09-30).


## 2026-09-30 — Keep the metadata API independent of Raven

- **Author clarification:** the Cecil-style metadata API stays a separate project;
  Raven consumes it for the neoCLR target.
- **Assistant response/action:** recorded a one-way dependency from Raven's target
  adapters to the library. Format/model code remains library-owned; compiler symbol
  projection, lowering and target diagnostics remain Raven-owned. Updated the roadmap
  and library documentation to make this boundary explicit.
- **Outcome:** the existing independent .NET project already matches this ownership.
  No code relocation or compiler change was needed. Package distribution, any repository
  split and actual adapter implementation remain open. [Ownership decision](design/extended-cli-metadata.md#independent-metadata-project-raven-target-consumer-2026-09-30).

## 2026-09-30 — First incremental Raven-to-metadata integration

- **Author direction:** continue work; complete integration in multiple steps rather
  than expecting all support immediately, and continue Raven refactoring when needed.
- **Assistant action:** created Raven `codex/metadata-consumer` from shared main in an
  isolated worktree. Added a compiler-side executable consuming the independent
  metadata project through public semantic operations. The frontend still uses its
  .NET provider for an API-produced PE dependency, with that limitation explicit.
- **Observed compiler gaps and fixes:** added resolved binary operator facts; reproduced
  and corrected callee-as-receiver projection (two failing cases); reproduced and
  corrected a required signature-only parameter incorrectly acquiring a null default
  (one failure, two passing controls). These shared fixes were committed separately
  as `1ea0ca263` and `d7040e21d` and fast-forwarded onto local Raven main.
- **Observed outcome:** the Raven program emitted through the metadata API loads,
  verifies and runs in neoCLR with result/exit 42. Unsupported division and a missing
  method fail as expected. All 87 focused Raven tests pass; compiler builds pass for
  .NET 10/11. The experimental consumer is `7e18edb66`, with paired integration docs
  and [recorded hashes](experiments/extended-cli-metadata/raven-compiler-validation.json).
- **Open:** native metadata provider, production emitter/target configuration and wider
  ordinary metadata support. The prototype is not advertised as full target support.
  No code or messages were sent to another thread, and no remote push was performed.

## 2026-09-30 — Runtime loading remains the metadata acceptance gate

- **Author reminder:** "Don't forget neoCLR runtime support for loading the assemblies with the metadata format."
- **Assistant response:** retained actual native loading, verification and execution
  as the acceptance gate. Distinguished the format-5 runtime artifact from the
  temporary reference-only PE used for Raven binding, and stated that direct PE/#Neo
  loading remains unimplemented.
- **Action:** extended the source-to-native case to a Raven application and two Raven
  libraries; added an owned direct-reference list to the native metadata reader.
  The application binds only the outer library; the host supplies both native runtime
  dependencies explicitly. No inference of a permanent change in roadmap priorities.
- **Observed outcome:** all three application variants execute to 42, as does reversed
  runtime module order. Native verification rejects missing and wrong-revision direct
  and transitive dependencies. All 24 C# metadata contract groups pass. The existing
  runtime already handles this graph, so no runtime code change was needed.
- **Open:** direct PE/#Neo loading, structural metadata semantics, native compiler
  symbol loading and production target composition. [Scope and evidence](raven-cli-bridge.md#transitive-native-runtime-acceptance--2026-09-30).

## 2026-09-30 — Initial direct runtime container and parsing cost

- **Author:** “Continue until we have an initial end-to-end integration with a neoclr
  runtime that can load metadata.” This continued the runtime-loading acceptance
  requirement; JSON-only compiler output was not treated as the final container gate.
- **Assistant action:** Added an explicit PE/#Neo execution profile to the independent
  .NET metadata API and the Rust runtime. Raven's opt-in adapter now writes this
  container. The same library files are read for compiler symbols through CLI reference
  declarations and loaded by neoCLR through the authoritative native section. The
  two-library program and both source/module orders execute to 42; rejection checks
  cover changed containers, unsupported required schemas and missing/wrong dependencies.
- **Author:** Explained that avoiding JSON parsing is also about building an effective
  runtime and reducing execution overhead.
- **Assistant response:** Clarified that this initial profile still contains format-5
  JSON, so direct PE loading does not remove text parsing or establish a speedup.
  Proposed a versioned binary native encoding and separate load/parse, link, verify
  and execution measurements as the next evaluation. This is proposed follow-up,
  not an implemented binary decoder or measured performance result.
- **Outcome/open work:** Initial direct loading works on the feature branches. The
  metadata library remains separate; the .NET semantic bootstrap remains temporary.
  Native compiler symbol loading, production target registration, guest Introspection
  loading/emission, wider signatures and structural runtime semantics remain open.
  See the [format/tradeoffs](design/extended-cli-metadata.md#direct-runtime-container-checkpoint--2026-09-30)
  and [tested compiler/runtime hashes](experiments/extended-cli-metadata/raven-compiler-validation.json).

### First acceptance programs

- **Author:** “I believe. Hello World is good first target, then calling another
  function from the entry point.”
- **Assistant:** Kept the existing arithmetic/dependency regression case and added
  the requested simpler sequence: direct console output, then Main calling Greet
  to print. Added a bounded, explicitly authorized Console-reference mapping.
- **Outcome:** Both Raven programs compile into PE/#Neo, verify/load/run in neoCLR,
  print exactly Hello World and exit zero. Validation exposed the native System
  Console method's Void-valued result; the emitted statement now discards it.
  String signatures, other Console overloads, a native symbol provider and binary
  payload performance work remain open. See the runtime/compiler validation report.

## 2026-09-30 — Binary payloads and class-library bootstrapping

- **Author:** Continued implementation after the Hello World/runtime-loading gate.
  Earlier highlighted JSON parsing overhead as a reason to build an effective runtime.
- **Assistant action:** Added a bounded binary execution schema and direct runtime
  decoding, retaining schema-1 compatibility. Hello World/function-call and the
  transitive Raven library cases pass with binary containers. The experiment uses
  CBOR of existing native fields rather than claiming a final indexed metadata layout.
  A focused release-mode load comparison separates container decoding, legacy load
  validation/linking, preparation, verification and execution.
- **Author direction:** Emphasized compiling the runtime class library and using its
  metadata to load symbols into Raven. Present similarity to .NET metadata can be
  used now, while allowing native metadata to diverge later.
- **Author proposal:** “One option would be to translate the current JSON into neoCLR
  assemblies.”
- **Assistant response:** Proposed that translation as a bootstrap path: preserve
  identities, declarations, signatures and bodies, reject unsupported information,
  and temporarily supply .NET-compatible reference projections for Raven. Native
  metadata remains authoritative. This does not require the full compiler backend
  to emit the new representation before class-library symbol-loading tests can begin.
- **Status/open work:** Binary runtime decoding is implemented; the general class-library
  JSON translator and native Raven symbol provider are not. The current C# declaration
  model only admits the bounded writer subset and cannot represent the complete
  runtime/raven/System.Runtime.rvnproj surface yet. The author's proposed translation
  path should drive the next concrete class-library slice and reveal required fields,
  generics, signatures, attributes and body encodings rather than silently dropping them.
  See [binary design](design/extended-cli-metadata.md#binary-native-execution-profile--2026-09-30)
  and [measurements](experiments/extended-cli-metadata/binary-loading.md).


### 2026-09-30 — Continue Raven after the metadata size benchmark

- **Author direction:** “Next step should be continuing work on Raven.” This follows
  acceptance of smaller assembly files and deferral of runtime optimization.
- **Assistant action:** Continued on `codex/metadata-consumer`, extending the native
  compiler adapter to Unit-returning helpers and static library methods. The separate
  metadata project already supports no-result methods; no format change was needed.
- **Validation:** C# checks compile/reimport a Raven library and execute Unit and Int32
  overloads from PE/#Neo in neoCLR. Explicit/implicit returns and rejected calls are
  covered. See [the contract](raven-cli-bridge.md#unit-returning-native-helpers-and-library-methods--2026-09-30).
- **Open:** Wider compiler metadata contracts, the native symbol provider and runtime
  class-library source compilation. This bounded slice does not complete those goals.


### 2026-09-30 — Native Raven output is the directive

- **Author clarification:** “Our directive is to make Raven emit code for the new
  assemblies” and format adaptations such as assembly-level functions should happen
  now when possible, otherwise later.
- **Assistant response:** Prioritize a usable native assembly emission path rather
  than waiting for complete format adaptation. Existing assembly-owned functions
  remain supported. Namespace-owned function semantics still need a metadata contract.
- **Completed slice:** Namespaced static library emission/reimport and runtime calls
  pass in both source orders at Raven `0c13e890b`; no format changes were required.
- **Next action:** Move native emission beyond the hard-coded probe. Broader source
  coverage and production target composition remain open.


### 2026-09-30 — Translated System as compiler symbol input

- **Author clarification:** “We of course also want to load symbol into the compiler
  from neoclr assemblies. That is where the translated System (.Runtime) assembly plays in”.
- **Assistant response:** Treat native emission and native symbol loading as paired
  integration tracks. The translated System assembly is the next symbol-loader
  acceptance input; host .NET primitive binding does not satisfy that requirement.
- **Action:** Add an opt-in compiler command for actual source files and current
  writer-produced PE/#Neo references. Keep the separate metadata project ownership.
- **Open:** General native declarations, fields/generics/signatures, and a faithful
  projection or native symbol provider sufficient to bind against translated System.
  The command does not claim that support; see the [loader boundary](raven-cli-bridge.md#opt-in-native-compiler-command--2026-09-30).


### 2026-09-30 — Reuse import/codegen boundaries and plan instruction editing

- **Author question/direction:** Asked whether Raven's .NET semantic importer can be
  adapted for neoCLR until a separate one exists, and described removing Reflection.Emit
  as a future goal for the .NET target too.
- **Assistant inspection:** Found ISemanticDataLoader/ICompilationEmitter boundaries;
  DotNetSemanticDataLoader and PE symbols still use MetadataLoadContext and reflection
  handles. The existing .NET code generator directly uses ILGenerator/builders. Proposed
  reusing semantic import via a native reference view now and extracting common metadata
  inputs and instruction/backend contracts incrementally.
- **Action:** Added a native library inventory, explicit static callable projections,
  compiler bindings and native System calls. This is a partial import, not the complete
  core library. Runtime checking exposed native private visibility and generic-arity
  name collisions; the reader accounts for both. Host facades also affected resolution,
  so the selected native mode retains only the host primitive core bootstrap.
- **Author question:** Asked whether the Cecil-like library supports raw opcode/operand
  emission in addition to Call/LoadArgument helpers.
- **Assistant answer:** It currently does not; public helpers cover a bounded internal
  instruction set. Proposed a shared instruction/typed-operand model beneath helpers.
- **Author follow-up:** “Cecil even has an ILProcessor that allows you to insert
  instructions before and after another instruction. That might be a future direction.”
- **Assistant proposal:** Future body editing with instruction objects, insertion,
  replacement/removal and instruction references for branch targets. No public raw Emit
  or ILProcessor-like editing has been implemented in this slice.


### 2026-09-30 — Opcode API follow-up

- **Earlier author question:** Asked for raw opcode/operand emission alongside typed
  metadata helpers and suggested Cecil ILProcessor-style insertion as future work.
- **Implemented follow-up:** A bounded OpCode enum and typed Emit overloads now share
  the helper validation/emission path. Raven's native operation emitter consumes them.
- **Remaining scope:** The instruction representation is internal; no public body
  collection, insertion, branches or exception-region editing has been implemented.
  This advances metadata/target integration without treating the future editing API
  as complete. See the [emission contract](raven-cli-bridge.md#opcode-based-metadata-emission--2026-09-30).

## 2026-09-30 — Shared compiler pipeline and builder abstractions

- **Author:** Asked to continue until integration works, authorizing codegen refactoring
  or redesign so .NET and neoCLR do not require entirely separate implementations.
  Clarified that common traits should be reused and differences abstracted, specifically
  Reflection.Emit type builders versus the independent neoCLR metadata builders.
- **Assistant:** Inspected the existing emitter and builder dependencies. Proposed first
  sharing compilation setup/validation through explicit backend selection, then extracting
  type/method handles and common lowering incrementally. Reflection.Emit's `Type`,
  `MemberInfo` and builders currently reach deep into general codegen; renaming the
  emitter interface alone would not remove that coupling.
- **Action:** Added an explicit artifact backend contract to Raven's `Compilation.Emit`;
  the native adapter and rvnc now use it. The independent metadata project and native
  format remain unchanged. Backend selection does not alter semantic target contracts.
- **Open:** The type/method builder boundary and broader common lowering still require
  extraction. Full native System import, class-library compilation and general signature/
  instruction support are not implied by the initial integration. See the
  [implemented boundary and focused validation](raven-cli-bridge.md#shared-emission-pipeline--2026-09-30).

### Follow-up — actual body-lowering reuse

- **Author:** Asked to continue broader abstraction of .NET codegen and implement neoCLR equivalents.
- **Assistant action:** Extracted the existing native linear-body lowering into an immutable
  compiler model, then connected both the existing .NET method generator and the native
  metadata builder through internal adapters. Native call identity and explicit Console
  permissions remain adapter-owned. Unsupported .NET bodies and debug/PDB output keep
  the existing general codegen path. No independent replacement of the whole .NET backend
  was introduced.
- **Scope:** Method-body reuse is implemented; type/signature builder unification is still
  open. The same-compilation execution probe and C# fallback/PDB checks are recorded in
  [the integration documentation](raven-cli-bridge.md#shared-linear-body-lowering-and-backend-method-builders--2026-09-30).

- **Author clarification:** Defer metadata loading and its common .NET/neoCLR abstraction
  to a future slice; avoid big workarounds and fix what is necessary for the end-to-end case.
- **Observed outcome:** The same compilation prints Shared Hello and returns 42 on .NET
  and neoCLR; the native compiler-command Hello/helper case also passes. An additional
  translated-System probe passes API binding/execution but the driver rejects a host-bound
  Math call with NEOMETA001. No importer change or assembly-identity workaround was made.
  That issue remains explicit for the author-directed metadata-loading follow-up.

## 2026-10-01 — codegen architecture review and first migration slice

- **Author:** Asked to pause and investigate abstractions required for the neoCLR target,
  then asked for a plan and directed continuation. These exchanges preceded this entry;
  no separate dates are assigned to the individual messages.
- **Assistant assessment:** Small callable/type wrappers leave Reflection types and
  members throughout general codegen. The native source-operation traversal risks
  duplicating the existing lowered-bound pipeline. Proposed incremental typed references,
  declarations, shared body emission, target policies and coherent target composition.
- **Action:** Deferred the uncommitted static-type prototype and replaced the shared
  source-operation traversal with compiler-lowered body consumption. Added observable
  implicit-return regression coverage. This is an initial bounded migration, not complete
  portability of the general generator.
- **Open:** Reference/declaration contracts, broader instructions and eventual target
  composition remain. The independent metadata library stays separate. Metadata loading
  and the known translated-System driver collision remain deferred as previously directed.
  See [the implemented scope](raven-cli-bridge.md#compiler-lowered-native-bodies--2026-10-01).

## 2026-10-01 — extended CLI compatibility baseline reaffirmed

- **Author question:** “You do still try to make sure our metadata format is a basically
  compatible with .NET metadata, except for the extensions we have already implemented?”
- **Assistant response:** Reaffirmed standard CLI metadata as the baseline with explicit
  neoCLR extensions. Explained that String signatures/ldstr and argument stores/starg
  use ordinary CLI constructs rather than new extensions.
- **Assistant qualification:** Current native PE artifacts carry conventional CLI reference
  declarations but execute a separate #Neo payload. Their CLI bodies are placeholders;
  these artifacts are not interchangeable executable .NET/neoCLR assemblies. This is
  a compatibility gap in the prototype, not evidence that extended CLI compatibility
  has been achieved.
- **Action:** Made this baseline and gap explicit in the metadata design, roadmap and
  both compiler integration documents. Continued the bounded argument-store slice
  using existing CLI/native instructions without adding a format extension.
- **Open:** Reconcile the native execution payload/reference projection bridge with
  compatible standard CLI metadata/body representation plus intentional extensions.
  No native CLI body loader or lossless general assembly roundtrip is claimed.

## 2026-10-01 — codegen performance consideration

- **Author:** “Also think about performance in the code gen, we might revisit that later.”
- **Assistant proposal:** Keep performance in view and record a later measurement of
  declaration collection, body planning, reference resolution and writing, including
  allocation costs. Avoid claiming that abstractions themselves make emission faster.
- **Action:** Recorded the follow-up in Raven's migration plan and neoCLR integration
  documentation while continuing the current correctness slice. No new benchmark was
  run; the earlier JSON-versus-assembly runtime benchmark does not measure compiler
  codegen and is not evidence for its performance.

## 2026-10-01 — shared abstraction and selective target capabilities

- **Author:** Emphasized a shared codegen abstraction fitting .NET and neoCLR without
  locking the compiler to either, with extra instructions and metadata categories
  selectively exposed by target.
- **Assistant response/action:** Recorded compiler-owned references, declarations and
  logical operations with backend-owned encoding/handles and explicit capability
  admission in Raven's migration plan. The existing bounded plan is partial progress,
  not completed general type/field or capability composition.
- **Investigation correction:** The assistant initially proposed source var-parameter
  stores based on an outdated parameter-spec paragraph. Focused tests exposed that
  the current binder and newer spec section reject var/val outside primary-constructor
  promotion. The source-emitter prototype was removed before commit, the stale text
  corrected, and typed stores were scoped to the independent metadata API with direct
  CLI/native execution tests. No language change was made to create a consumer.

## 2026-10-01 — real class-library compilation and broad consumer

- **Author:** identified compilation of the runtime/class library written in Raven as
  the ultimate test; asked to try parts to expose emission/metadata gaps before
  metadata loading. Then requested the broadest sample covering needed target features.
- **Assistant proposal:** start with unchanged source and a precise member selection,
  then use a broad consumer to drive shared codegen and native metadata together.
- **Performed:** added and ran a C# source inventory. Whole Math, UnicodeScalar and GC
  fail binding with missing native library dependencies under the host-core bootstrap.
  Original Int32 Min/Max/Sign declarations retain their namespace, bind, and fail native
  emission on namespace-function metadata; no assembly or runtime success is claimed.
- **Assistant selection:** order-collections is the broadest practical inspected seed:
  classes, constructors, properties, generics, interfaces, arrays, iteration, delegates,
  lambdas, unions/patterns and aliasing. JSON adds UTF-8/inheritance but more initial
  dependencies. This comparison is an inspection, not a complete feature census.
- **Open:** namespace-function identity/projection first, then incrementally compile
  the consumer's real library source closure. Broader metadata importing stays deferred.
  [Evidence](experiments/extended-cli-metadata/class-library-validation.json) and
  [scope](raven-cli-bridge.md#class-library-emission-acceptance--2026-10-01).


### 2026-10-01 — Fill emission gaps against executable cases

- **Author:** “We are going to fill the gaps so we can complete the emission story.”
- **Assistant response:** keep the actual Order declaration as the next acceptance
  case, then construction/mutation and the broader consumer's generics, delegates and
  unions. Metadata additions should serve executable compiler cases.
- **Actions/results:** the independent API's property/accessor producer and owned
  snapshots were implemented and committed, with binary runtime execution and C#
  tests. Raven shared root/instance declaration contracts and receiver argument slots
  were implemented and tested on .NET. Native source class admission remains gated;
  no full Order or broad consumer emission is claimed.
- **Open:** shared field/property/constructor bodies, object locals and the broader
  consumer gaps. Broad symbol importing remains deferred. See the
  [current integration record](raven-cli-bridge.md#shared-root-and-instance-declaration-contracts--2026-10-01).


### 2026-10-01 — Subsequent Order execution results

- **Author:** continued the request for subsequent emission slices.
- **Assistant actions/results:** the unchanged Order declaration now compiles with
  constructors and auto-properties, using compiler-synthesized accessor bodies. A
  separate Main executes boundary checks and then local aliasing/property mutation
  on .NET and binary neoCLR in both file orders, returning 42. This advances the
  earlier entry's recorded native class gate; it does not mean the full consumer runs.
- **Open:** nominal signatures and generic/delegate/union coverage; the complete
  order-collections sample still has native dependency binding gaps. See
  [recorded executable evidence](experiments/extended-cli-metadata/order-runtime-validation.json).


## 2026-10-01 — complete the current emission slices

The author directed: “Continue will those slices. Don’t stop until done”, referring to
nominal signatures, implicit constructors and field/property initialization. The assistant
proposed sharing canonical constructor initialization first, then extending the separate
metadata signature model and Raven adapters. Implemented work now includes shared bound
field-initializer assignments, default root constructors and owned nominal parameters/
results. C# contracts and .NET/binary neoCLR consumers passed; the author has not yet
responded to these outcomes. These are bounded producer/codegen slices, not a claim that
the broad class-library/collection consumer or metadata importing is complete. See
[the integration evidence](raven-cli-bridge.md#raven-owned-nominal-signature-integration--2026-10-01).

## 2026-10-01 — a connected array emission milestone

- **Author directive:** “Continue with a big set of slices. Don’t stop until you reach
  a reasonable point”. Earlier directions retained independent metadata ownership,
  shared .NET/neoCLR codegen and executable class-library/consumer-driven acceptance.
- **Assistant proposal:** complete a connected vector milestone: metadata declarations,
  typed/raw instruction production, Raven adapters, then shared array iteration using
  the Order declaration and batch expression from the broad consumer.
- **Actions/results:** committed each bounded slice. CLI SZARRAY and native ArrayRef
  signatures now cover calls, locals and storage. C# API-produced binary tests execute
  arrays on neoCLR, including alias mutation and length/bounds failures; 58 API test
  groups pass. Raven shares literal/index/Length/storage emission and array for lowering.
  The expanded separate consumer returns 42 on .NET and binary neoCLR in both source
  orders; 39 focused loop/array tests pass, including mixed general enumerator nesting.
  [Evidence](experiments/extended-cli-metadata/array-runtime-validation.json).
- **Open:** full collections/System source compilation, generic/delegate/union contracts,
  broader constructors/static storage and native metadata symbol imports. Executable
  native payload plus CLI projection remains a temporary bridge. No complete emission
  story or measured performance improvement is claimed; no author reply to these
  results is recorded yet.

## 2026-10-01 — indexed collections, then generics

- **Author:** requested another large set of slices to a reasonable stopping point.
- **Assistant:** selected indexed properties to build on vector emission and support
  collection element access, then implemented independent metadata associations and
  shared Raven accessor planning. C# API binaries verify/run on neoCLR; the expanded
  concrete OrderBuffer consumer runs on both targets/source orders, with alias and
  evaluation-order checks, bounds faults and rejection of unsupported signatures.
- **Author steering during work:** “After all this: Continue with another big set of
  slices. I assume generics. Don’t stop until you reach a reasonable point”.
- **Assistant response:** finish the indexer evidence/commits, then pursue a connected
  generic signature/call milestone through metadata, Raven and runtime execution.
  That generic work is a plan at this entry, not a completed result. Full generic
  collections/System compilation and native symbol importing remain open.

- **Assistant follow-through:** completed generic declaration, instantiation and Raven
  adapter slices. Ordinary .NET GenericParam/MVAR/MethodSpec encodings and native
  method parameters/call arguments retain the same bounded source semantics. A loader
  restriction incorrectly grouped generic static class methods with receiver-dependent
  methods; static generic methods now load while generic instance admission remains
  unchanged. Expanded Order tests pass forwarding, inference, recursion, generic vector
  construction/iteration, overloads, multi-parameter methods and conditional values on
  both runtimes in both source orders. Five unsupported native contracts produce
  source diagnostics without output. See the [generic evidence](experiments/extended-cli-metadata/generic-runtime-validation.json).
  This completes the unconstrained owned static-generic milestone, not generic types,
  constrained dispatch, native symbol imports or full System collection compilation.

## 2026-10-01 — generic receivers and typed defaults

- **Author:** requested another large set of slices to a reasonable stopping point,
  then reiterated continuation after the active slice.
- **Assistant:** selected ordinary generic class instance methods as the next connected
  metadata/runtime/compiler boundary. Extended owned API calls and native verifier
  admission, keeping virtual/byref and generic-constructor restrictions. Raven shares
  receiver-aware planning through an explicit instance-generic capability.
- **Performed validation:** generic forwarding and mutation, no-result copy/reverse,
  recursion, receiver/argument order and independent receivers execute from binary
  assemblies on .NET and neoCLR. A native test forces collection inside a generic call
  to exercise receiver/argument rooting. Wrong receivers and generic constructor
  projections reject.
- **Assistant follow-through on continuation:** added typed local address/initobj/default
  producer support, then shared Raven default(T) emission and generic array clearing.
  Primitive/reference defaults execute on both runtimes; cleared object elements fault
  when dereferenced. The [updated evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
  records source/runtime hashes and source-order coverage.
- **Still open:** generic type owners, constraints/virtual dispatch, imported generics,
  native symbol loading and compiling the full runtime class library. These bounded
  slices do not establish complete System/collection support.


### 2026-10-01 — Static generic owner emission milestone

- **Author direction:** continue with another substantial set of slices and commit
  each slice, preserving the Raven-to-neoCLR emission objective. Earlier direction
  identified generics as a likely next area; no new full-library completion was claimed.
- **Assistant action:** implemented bounded static generic type owners in the separate
  metadata API, preserving independent CLI VAR/MVAR and native owner/method scopes.
  Integrated shared Raven type/call capabilities and both adapters. The .NET executable
  exposed an open declaring-type call bug; the source-method resolver now constructs
  the owner before the generic method.
- **Validation/outcome:** 64 C# metadata contract groups pass; API-produced native
  binary verifies/runs 42. Focused Raven Release/Debug tests and the expanded Order
  consumer pass both targets/source orders, including reordered owner arguments and
  cross-scope forwarding. See [recorded binary evidence](experiments/extended-cli-metadata/generic-runtime-validation.json),
  neoCLR producer commit `0da5a3b0`, and Raven integration commit `339587142`.
- **Open:** generic instance layouts/fields, constraints, imported generics and full
  class-library compilation remain unimplemented boundaries. The native binary payload
  plus CLI projection is still a bridge, not the final native compiler backend.


### 2026-10-01 — Generic instance storage follows static owners

- **Author direction:** continue the existing emission integration, committing bounded
  slices. The prior stopping point identified generic instance layouts and fields next.
- **Assistant implementation:** added generic reference-class construction and exact
  constructed receiver identity to the independent metadata producer. Constructors,
  fields using declaring-type VAR, nested class signatures and generic instance methods
  use ordinary CLI representations and existing native Constructed metadata. No new
  runtime encoding was needed. Raven now shares these value and call paths through
  separate generic-class and constructed-field capabilities.
- **Outcome:** producer binary verification/execution returns 42; all 65 C# metadata
  contract groups pass. Raven's Order consumer executes primitive, object and nested
  generic storage on .NET and native in both source orders. Focused Release/Debug
  regressions exercise shared planning and mutation through aliases. See producer
  commit `62bf5931` and [binary evidence](experiments/extended-cli-metadata/generic-runtime-validation.json).
- **Open:** generic properties/indexers, direct external constructed field handles,
  constraints, imported generic owners and full class-library compilation. Native
  rejection is explicit at these boundaries; ordinary .NET support remains available.


## 2026-10-01 — Constraints followed by an integration assessment

The author directed “After this, continue with generic type constraints directly,”
then “Continue with the rest of the constraints after this slice,” and finally
“After this we should assess the state.” The assistant completed constructed-field
integration, owned nominal type bounds, and distinct class/struct/new type requirements
across the producer, Raven adapter and runtime. A .NET struct-flag inconsistency found
by C# tests was also corrected. These are feature-branch results, not release claims.

The assistant's [assessment](experiments/extended-cli-metadata/state-assessment-2026-10-01.md)
separates verified binary execution from declaration-only support and remaining work.
Full runtime-class-library emission, open constrained operations, broader bounds,
collection dependencies and native symbol loading remain open. The proposed next
sequence is recorded as a recommendation; no subsequent author selection is available.


## 2026-10-01 — Whole-source acceptance and .NET alignment clarification

Following the assessment, the author said “Continue.” The assistant selected the
complete, unchanged System.Globalization.Language source as a small acceptance unit
and found static computed properties blocked native emission. Shared accessor-call
lowering and native declaration admission now cover that unit without changing the
metadata API, runtime or instruction set. Both runtimes print und/sv/he and return 42;
collection-interface inventory identifies the next declaration gap.

During the work the author clarified: “We are supposed to align neoCLR behavior with
.NET, unless we make other choices. The instruction set is the same, if we ignore
what we don't support - like exceptions”. The assistant accepted .NET behavior and
CLI instruction semantics as the default and recorded that unsupported coverage is
not a semantic divergence. The temporary #Neo storage/loading bridge does not imply
a separate instruction-set design. No new divergence was proposed or implemented.

### 2026-10-01 — Interface dispatch as the next acceptance gate

The author directed: “After this, implement dispatch for interfaces”. The assistant
will follow interface-valued signature validation with concrete class implementations
and calls through interface references. This supersedes generic interface inheritance
as the immediate next slice; it does not claim dispatch is implemented yet.


### 2026-10-01 — Reassess against the library and existing applications

The author asked, after interface dispatch, whether larger steps could take the project
toward compiling/running the full Raven runtime library and applications, suggesting
running the existing samples and investigating gaps. The assistant first completed
and committed nongeneric interface dispatch in the metadata API and shared Raven
backend, with .NET/native execution and C# tests. It then performed 31 source inventory
attempts and four legacy-bridge runtime controls using current compiler/runtime builds.

All twelve selected applications bound/emitted ordinary CLI, while direct emission
rejected the actual neoCLR target profile. Three application controls ran with expected
output. Order-collections failed import because emitted CLI constructs None then calls
a Some carrier constructor; the exact compiler root cause remains open. The 167-file
library attempt used a consumer snapshot and exposed missing implementation services
and runtime-contract identity conflicts, not a valid full-library bootstrap. The
assistant proposes target-profile/bootstrap integration first, then collection/library
boundaries and shared union/callback emission. These are proposed next milestones;
the author has not yet responded to that sequence. [Assessment and evidence](experiments/extended-cli-metadata/readiness-assessment-2026-10-01.md).

## 2026-10-02 — Reassess samples after output metadata, then reconnect Raven

- **Author:** asked, “Asses samples after this”, following ref/out metadata work,
  then directed, “Continue solving this afterwards”.
- **Assistant proposal/action:** finished the output-contract slice, rebuilt the Raven
  probe, and assessed unchanged samples before selecting the next codegen task. The
  31-case inventory showed four library emit/verify successes; all twelve selected
  applications still stopped before native execution despite successful CLI controls.
  The four small native execution controls passed. The assistant proposed connecting
  Raven's ref/out path next instead of adding unrelated metadata features.
- **Performed outcome:** Raven `e6912a285` adds the explicit shared managed-reference
  capability and connects signatures, local addresses and indirect operations to both
  adapters. Five native controls now pass, including output forwarding/ref mutation;
  64 focused C# tests pass. Collections moves past output-local admission to imported
  value-receiver TryGetOutput invocation admission. The full application is not complete.
- **Open:** imported value receivers, propagation failure/Unit handling, exact native
  System identities and implementation-bootstrap dependencies. The author did not
  independently approve each detailed API choice; continuation directed further work.

Evidence: [sample assessment](experiments/extended-cli-metadata/readiness-assessment-2026-10-02.md),
[native controls](experiments/extended-cli-metadata/raven-ref-out-validation-2026-10-02.json),
[unchanged collections](experiments/extended-cli-metadata/collections-after-raven-ref-out-2026-10-02.json).


## 2026-10-02 — Continue through sample execution

**Author:** “Keep working on this problem until you've solved it and the samples you
are attempting to get working actually work. That they compile and they run successfully.”

**Assistant response/action:** acknowledged that intermediate commits are checkpoints,
not the stopping criterion. Committed imported constructor support (neoCLR a7569ca5,
Raven 5bc0d567e), then implemented nested declaration ownership with CLR/native execution
returning 42. The unchanged collections application still requires nested imports and
further native codegen/dependency support; its successful execution is not yet reported.
See the [metadata design checkpoints](design/extended-cli-metadata.md).

The author subsequently requested that neoCLR also have a `codex` structural-types
branch on top of main. Preserve the existing Function implementation/history on
`codex/structural-types`, merge the Self-integrated main, validate its structural
ABI, and retire the old branch name after remote synchronization. The author's
uncommitted structural-types proposal remains outside these integration commits.


## 2026-10-02 — Reuse the Function feature branches

**Author:** “We should perhaps also think whether we should try to implement function
types for the neoCLR target. We already have branches for that” and “We can integrate
compiler and runtime work.”

**Assistant proposal/action:** inspected neoCLR codex/structural-types (a081c6e3) and
Raven codex/neoclr-structural-types (b92a0d4a7). The runtime branch already implements
structural Function shapes, checked binding, retained captures and a migrated library;
the current Raven metadata branch already contains the target's inhabited-result policy.
Integrated the runtime feature work into codex/extended-cli-metadata, retaining System.Fail
and the metadata branch's PE loader and constructor/byref changes. Regenerated the library
and reference bundle and ran focused runtime/metadata checks. This is feature-branch
integration, not a main merge or a declaration that direct native Raven callback emission
is complete. Next: expose Function signatures and binding through the metadata API and
shared compiler emission plan, then run the unchanged broad collections sample directly.


**Assistant outcome (same date):** completed metadata Function signatures/static binding,
Raven static method groups and noncapturing lambdas, case patterns and propagation,
reference conversions, portable iteration and explicit translated-library linkage. The
unchanged collections application now emits native PE/#Neo, verifies and runs with exact
expected output and exit 0. A runtime access regression was reproduced: generic Function
invocation incorrectly rejected the caller's internal type; the fix preserves open-context
access checks and keeps explicit foreign internal-type use rejected. See the
[bundle-hashed acceptance record](experiments/extended-cli-metadata/collections-end-to-end-2026-10-02.json).
This meets the requested broad application execution checkpoint. Complete native runtime
library source emission and native semantic metadata loading remain open; System is still
the explicitly matched translated implementation, and the input symbols use CLI metadata.

## 2026-10-02 — Direct native semantic importing takes priority

**Author:** preferred reading neoCLR metadata into Raven's semantic model to translating
it into .NET metadata; said integration must develop the metadata-library foundation.
The author then clarified that the architecture is already documented and further work
should fulfill that vision, rather than establish a new direction.

**Assistant:** inspected the native readers, shared definition model, Raven semantic
loader interface and reflection-owned target setup. Initially discussed declaration
model needs, then explicitly anchored the work in the existing builders → definitions
→ metadata → PE direction. Recorded concrete gaps and an incremental sequence under
the existing design, beginning with native definition materialization and a native
function-library semantic import, then cross-assembly execution and broader System types.

**Outcome:** priority and implementation alignment documented in both repositories.
No native semantic importer is claimed implemented by this audit. Existing source
collection/reference tests remain evidence; expanding the mixed CLI bootstrap is no
longer the immediate task. See [the alignment](design/extended-cli-metadata.md#direct-native-semantic-import-implementation-alignment-2026-10-02).

**Author clarification:** “We don't want to recreate the .NET reflection API,” while
requiring dependency resolution during symbol loading. **Assistant audit:** found the
existing exact-identity IAssemblyResolver and reference identity checks. Recorded use
of that contract with native definitions and explicit compiler import-session dependency
policy, rather than reflection emulation. Native cycle/session work remains planned.

**Further author direction (2026-10-02):** after implementing the neoCLR target, consider
investigating Cecil as a replacement for .NET target reflection usage. The author then
clarified the current scope: support both targets and how their assemblies load and emit.
**Assistant response:** keep Cecil evaluation deferred, preserve current .NET behavior,
and continue native read/import/emission integration under the existing architecture.
**Performed outcome:** first native primitive-function definition materialization passes
96 metadata groups; Raven semantic importer remains pending.

## 2026-10-02 — Native integration as evidence for a future .NET backend

**Author:** emphasized that neoCLR import/emission is important because it will help
determine whether replacing Reflection and Reflection.Emit for the .NET target with
Cecil or another mechanism is feasible. This follows the earlier decision to defer
that migration while supporting both targets.

**Assistant response:** treat native loading/emission as a test of provider-neutral
semantic and codegen contracts. Keep general fixes isolated, native mappings behind
target contracts, and .NET execution as a control. Do not interpret successful native
slices as a completed .NET migration or a choice of Cecil.

**Performed evidence:** native property/indexer import and execution, including
setter-only access, already pass; this slice adds interface definitions and inherited
dispatch. The six-consumer native harness and 103 metadata groups pass. The next
planned work will check interface-valued storage across native assemblies. The CLI
primitive core still uses the existing bootstrap loader, so the entire compilation
pipeline is not yet independent of reflection. Broader native types and a dedicated
.NET provider/backend evaluation remain open.

**Follow-on outcome (2026-10-02):** the planned interface-storage check now passes.
A three-assembly Raven consumer preserves interface field/array aliases and executes
dispatch after replacement (42); C# tests run equivalent imports on CLR. No additional
reader/compiler/runtime code was required after interface loading, providing a concrete
example of reusing the contracts. This does not resolve the bootstrap and broader
.NET replacement questions recorded above.


## 2026-10-02 — Dependency-aware pure metadata views

**Author:** proposed a projection layer above the metadata reader/writer infrastructure
for dependency resolution and inspection of imported/external declarations, explicitly
saying it should be “pure metadata” rather than a reflection layer.

**Assistant audit:** found exact-identity AssemblyReference/TypeReference resolution,
loaded constructed signatures and generic interface relationships already implemented.
The missing uniform session and substituted-member views should build on those contracts.

**Assistant proposal:** keep raw definitions, a context with explicit dependency policy,
and read-only constructed views separate. Raven's importer consumes these views and
copies semantic information into symbols; the emitter remains independent. Start with
a fixed snapshot catalog before adding lazy loading or wider member decoding.

**Performed outcome:** recorded the [architecture and staged acceptance tests](design/metadata-resolution-views.md),
linked it from the roadmap and Raven integration documentation. No new public API,
compiler behavior or runtime capability is claimed. Context implementation, view caching,
consumer migration and external-interface emission remain open.


**Author clarification:** the same library will be needed in NeoCLR itself for
metadata-only introspection and as input to System.Runtime.Reflection.Emit, independently
of loading assemblies into the execution runtime.

**Assistant response and performed action:** extended the design with three consumers
(Raven importer, metadata-only Introspection and Emit), an explicit input-to-output
import boundary, and host-independent semantics for a future Raven/NeoCLR port. Existing
runtime Introspection must be audited before reuse; no port or execution-independent
replacement has been implemented by this documentation slice.


**Subsequent author correction:** this need not be the exact future library. The current
work is a prototype, potentially only for the .NET-based Raven compiler, which may
inform later work after bootstrapping Raven to compile for NeoCLR on NeoCLR.

**Assistant response and action:** clarified the design's immediate .NET-hosted scope
and removed any implied requirement for identical future APIs or a direct port. Reuse
and eventual library shape remain exploratory; metadata/execution separation is retained.


## 2026-10-02 — Implementing the C# Introspection facade

**Author:** clarified that the C# facade should follow the shape of System.Introspection
above readers, alongside builders, then directed implementation and co-evolution with
the runtime model and an owned metadata load context. The author noted that reusable
complexity could move out of the symbol/semantic loader into this introspection layer.

**Assistant response:** separate metadata identity/dependency resolution/substitution from
Raven language binding. Inspect runtime AssemblyInfo/ModuleInfo/TypeInfo/NominalTypeInfo
before implementing the initial facade; do not invent runtime-backed capabilities.

**Performed outcome:** added the immutable-catalog C# load context and nominal facade,
with CLI/native, cycle, diamond, conflict, version, concurrency and isolation tests.
Raven now consumes it for nominal resolution and removes its private assembly resolver.
A compilation-owned weak-key adapter supplies lifetime without exposing caches publicly.
Constructed/member views and further loader simplification remain next; the runtime
model and guest implementation are unchanged. 109 C# groups pass; native integration
execution evidence is recorded with this slice. No future identical port/API is promised.


## 2026-10-03 — Dual-target execution gate

The author requested a plan for end-to-end .NET and NeoCLR compilation. The assistant
proposed a broad existing application plus a separately source-built library subset as
the first gate. Asked to choose, the author selected that gate over the entire runtime
library and accepted the bounded primitive CLI bootstrap/runtime seed. The author then
explicitly requested implementation of the seven-slice plan. Work begins with paired
ordinary driver commands and direct native import, retaining the .NET backend. The broad
sample/full-library outcomes are still open; see the [implementation plan](design/dual-target-compilation.md).


## 2026-10-03 — Continue the dual-target application gate

The author asked to plan work toward the end-to-end case, then explicitly requested
implementation of the resulting plan. The assistant proposed completing external
interfaces first, followed by canonical bootstrap/source ownership, independently built
collection libraries, the unchanged broad application and a refreshed inventory.

The external-interface slice now passes the separate contracts/implementation/consumer
case on both targets, including a generic diamond and alias mutation. Metadata authoring
requires explicit complete external contracts; native reading uses the metadata context.
Authored PE emission retains the validated graph rather than reloading dependencies.
Existing native encoding and runtime dispatch required no changes for this case.
The broader source-library gate remains open; see the [implementation tracker](design/dual-target-compilation.md).


### Follow-up: source ownership and native Self (2026-10-03)

The author directed continuation, then added: “We also need to support the Self “type” signature in neoclr metadata” The assistant agreed to preserve native Self across metadata
signatures, introspection, Raven symbols and emission, rather than relying on the CLI
marker. Existing runtime SelfType and structural-description encoding were found; the
Cecil-like assembly signature path is not yet implemented. This remains open work.

The ownership slice independently reached execution: seven unchanged iteration/collection
source units compile as a library and a separate consumer executes on both targets.
The explicit ownership manifest rejects competing CoreProbe declarations and configures
only the runtime contracts supplied by the selected bootstrap. No full-System or broad
application completion is claimed; see the dual-target tracker and matching evidence.


The author then asked whether Self is a special type parameter available without an
explicit generic declaration. The assistant explained it as an implicit implementing-type
parameter scoped to an interface, distinct from declared generic parameter slots. Class
Self denotes the declaring constructed type. The proposed metadata API should preserve
a dedicated symbolic signature and contract-scoped view; implementation remains open.

Expanded API-reference regeneration was attempted after the bootstrap command change;
it failed in source-union projection with `None` not in scope. The last verified snapshot
was preserved and the stale fingerprint recorded in API maintenance notes. This does not
invalidate the independently executed source-ownership fixture, but it is an unresolved
validation item rather than a successful API refresh.

The subsequent authorized continuation implements the first bounded contract slice:
Self signature nodes, native encoding/decoding and canonical interface-scoped introspection
views. C# tests cover definition/builder parity, generic-scope separation, accessors,
round trips and invalid contexts. NeoCLR loads/verifies a generated assembly containing
these declarations and executes its ordinary entry point (42); this is loading evidence,
not Self dispatch evidence. Raven integration and implementation substitution remain open.

## 2026-10-03 — Builder and IL generator boundary reaffirmed

- Author, during union support: “separating the neoclr methodBuilers from IL emission”
  and “methodBuilder.GetILGenerator”.
- Assistant inspected the feature branch: MethodBuilder.GetILGenerator already returns
  the metadata library's IILGenerator; direct builder emission methods forward to it.
  Raven's target adapter already consumes that generator behind its own boundary.
- Action: use GetILGenerator explicitly in the new tag/payload contract test. Preserve
  existing forwarding compatibility for now. A wider cleanup/removal of convenience
  forwards remains open, not implemented or inferred as an immediate breaking change.
- Evidence: tools/metadata/NeoCLR.Metadata.Experimental/Model/MethodBodyGenerator.cs,
  MethodILGenerator.cs and TaggedValueLayoutChecks.cs; Raven Int32Emitter adapter.

## 2026-10-03 — Share CLR instruction and metadata information

- **Author:** emphasized that targeting NeoCLR should not require complicated new
  machinery when the writer can consume the same information: “same instructions,
  same instruction set as the .NET platform, the CLR.” Mapping differences are the
  exception that requires target handling.
- **Assistant:** agreed to carry existing lowered operations and metadata facts through
  the target adapter, and identified missing preservation of union attributes, case
  identities and generic companion relationships as the next writer/reader work.
  Proposed reusing Raven's attribute-based contract rather than a separate union encoding.
- **Actions/evidence:** Object display, contextual nulls, case tests and configured unit
  storage now reuse existing CLI/native operations. Core display and null drivers execute
  on both targets; the configured native unit driver executes with its explicit seed.
  Unchanged Option now reaches metadata preservation after its bodies pass preflight.
  See [case/unit evidence](experiments/extended-cli-metadata/union-case-unit-2026-10-03.json).
- **Open:** custom-attribute authoring/reader/introspection coverage, native union/case
  semantic reconstruction, then separately compiled union consumer execution. This is
  a work direction and partial implementation report, not completed union support.

## 2026-10-03 — CLI baseline with Raven integration first

- **Author:** “We should follow CLI Metadata format unless we diverge.” Ordinary metadata
  should use the same database format; structural types and module-level functions are
  examples of differences to account for explicitly.
- **Assistant:** identified the current native-authoritative PE/#Neo transport and proposed
  moving ordinary metadata authority to CLI tables instead of extending parallel encoding.
- **Author clarification:** “As it is now we want Raven to target neoCLR.” New semantics
  should not dominate current work; the format can be adjusted during later experiments.
- **Assistant response/action:** retained CLI compatibility as the architectural baseline,
  but deferred wholesale container migration as a prerequisite. Updated the roadmap,
  metadata direction and dual-target plan to prioritize demonstrated union/import/emission/
  execution gaps. Existing uncommitted attribute implementation work remains in progress.
- **Open:** union metadata preservation, native semantic reconstruction and executable
  separate-library consumers. This entry records scope; no format migration or new
  runtime acceptance result was completed by this documentation change.


## 2026-10-03 — Shared format structure and NeoCLR categories

- **Author:** noted that a future fork of the format remains possible, while the current
  structure can encode NeoCLR semantics. Module functions and function types remain
  NeoCLR concepts distinct from ordinary CLI categories; the message ended open-ended.
- **Assistant:** retained CLI-derived structure and explicit extensions, without inferring
  a decision to fork now. Continued unions using ordinary nominal declarations and the
  existing Raven attribute contract. Structural Function integration remains on its
  feature branches; no additional instruction was inferred from the unfinished thought.
- **Outcome:** plain/generic local union cases now compile/run on both targets with
  case metadata and generated display. Native imported union semantics remain open.


### 2026-10-03 — Unions as a class-library prerequisite

The author emphasized that making Option and Result work unlocks compiling the class
library that depends on them. The assistant agreed with the dependency order while
separating union support from unrelated remaining library/runtime gaps. Implementation
now executes separately compiled plain/generic union consumers on .NET and neoCLR,
using native metadata import and symbol-authored emission. The unchanged source Option
probe passes union declaration admission but rejects an unregistered System.Void residual
type dependency. Option/Result and the broad library/application gate remain open.
See the separate-union evidence and dual-target integration tracker. This continues the
existing Raven integration priority; it does not authorize a format fork or new semantics.


### 2026-10-03 — Inhabited unit versus no-result returns

The author clarified that neoCLR's void/unit type may have a value and be passed as an
ordinary type. The compiler may materialize that value when needed and omit it otherwise.
The assistant accepted that direction: parameters, storage and generic arguments require
an inhabited value contract; a no-result return is a distinct callable property. Existing
runtime Type.Void/Value.Void and function no_result already express this distinction.
The compiler must preserve side effects when discarding a result. This is a target contract,
not permission to admit CLI System.Void as an ordinary .NET generic argument.

The existing native unit-storage driver passes against the current union integration
(exit 42), exercising storage, out assignment and ordinary arguments. An additional generic union with a System.Void payload currently rejects its
case constructor during binding (RAV1501), while the source Option ownership probe rejects
an unregistered System.Void dependency. These are separate compiler/bootstrap gaps; neither
justifies prohibiting inhabited unit in neoCLR. Full Option/Result execution remains open.


2026-10-03 follow-up outcome: the generic System.Void union constructor rejection was
an unconditional special-void check in overload resolution. Raven now honors the exact
selected unit contract; the negative ordinary-CLI control still rejects. Generic union
payload execution returns 42 and all 16 focused tests pass on both the integration and
isolated main-based fix branches. Source Option's bootstrap ownership gap remains open.


### 2026-10-03 — Source-owned Option/Result pass the native execution gate

Following the author's continuation request, the assistant split a bounded executable
System seed from the translated full library. The seed contains real primitive services;
Object display calls the existing type-handle query directly to avoid a dependency cycle
through guest introspection and Option/Result. Unchanged iteration, Propagatable, Option
and Result sources compile into a separate library, then a native-import consumer runs
with exact output and exit 42. No source union declarations were rewritten or replaced.

Two implementation gaps were fixed: metadata authoring now records value-interface edges,
and Raven reconstructs imported concrete value calls and unit value parameters from symbols.
Missing-library and duplicate-seed ownership reject before publication. The .NET class-library
adapter and application-order-collections gate remain open. A newly written test also exposed
an unrelated possible pattern-variable shadowing issue (reusing an outer `error` name inside
an Error case makes the outer name unavailable afterward); it is recorded as an unisolated
binder investigation, not fixed or claimed validated for main in this slice.

### 2026-10-03 — Retain nominal Array<T> backing during native integration

- Author recalled that arrays used to map to Array<T>, and that the Function experiment
  may have changed them to structural types. This was a recollection, not a confirmed
  account of the exact introducing change.
- Assistant inspected the current integration: compiler arrays retain IArrayTypeSymbol
  storage and the explicit RuntimeIterationContract.ArrayShapeTypeName projection;
  the Function experiment documents structural array introspection. No structural
  array change was made in response.
- Author directed: “We should keep it backed with Array<T> and decide what to do later
  when we work on structural types”. Assistant accepted nominal backing for this gate
  and deferred structural identity changes.
- Investigation: enabling the existing array iteration selection gets the broad sample
  past Filter lookup. Selecting the source Propagatable contract avoids the default
  CLR exception-catching propagation rewrite; the next broad binding/emission gap is
  the bounded seed's missing Console.WriteLine(Int32). These exploratory settings did
  not establish execution support and were not substituted for the existing query gate.
- Performed assessment: unchanged Array.rvn compiles with the cumulative library and a
  separate array-query consumer compiles. Execution fails to match the vector's
  source-owned interface implementation. Nominal descriptor/storage linking and
  iterator dispatch remain open. See [recorded evidence](experiments/extended-cli-metadata/nominal-array-assessment-2026-10-03.json).

### 2026-10-03 — Nominal array direction reaches executable evidence

Following the author's instruction to keep arrays backed by Array<T> and defer structural
array decisions, the assistant implemented an explicit native descriptor selection and
vector/interface dispatch. The separate consumer mutates through MutableSequence<int>,
observes shared storage through the original array, and runs source iterator/query code
(return 42). This is an assistant-verified outcome, not full broad-application completion.
The unchanged broad sample next exposes the retained seed's missing WriteLine(Int32).
See [execution evidence](experiments/extended-cli-metadata/nominal-array-execution-2026-10-03.json)
and the [integration record](raven-cli-bridge.md).

### 2026-10-03 — Unchanged broad native application executes

Under the author's continuing end-to-end directive, the assistant closed the explicit
seed's integer console overload and corrected imported value override identity through
metadata slot facts and Raven symbols. The unchanged application-order-collections now
compiles against the independently emitted native source-library subset, with library
sources absent from consumer compilation, and executes with exact expected stdout/exit 0.
This is assistant-verified native evidence, not completion of the paired .NET gate or the
entire runtime library. Next is the .NET source-library execution adapter gate.
[Evidence](experiments/extended-cli-metadata/broad-native-execution-2026-10-03.json).

### 2026-10-03 — Paired .NET assessment and independent binding fix

Continuing the author's dual-target goal, the assistant added executable .NET allocation
and process-failure adapters, then compiled the unchanged library source set. The compiler's
existing exact-nullability override rule rejected SingleError.ToString; a bounded safe
reference-return strengthening fix was isolated on the author-requested main-based Raven
fix branch (54fc1e0aa) and tested independently (13 cases), then integrated as 1836ca9ef.
No main merge was performed. The native broad application still executes successfully.

The .NET assessment does not pass: consumer compilation exposes CLR void used as a generic
argument in the emitted Option/Propagatable metadata. The assistant identified explicit
unit-value mapping as the next task and retained the sources unchanged. Both executable
service consumers pass; emitting the library alone is not counted as success. The failed
consumer leaves no Application.dll. [Evidence](experiments/extended-cli-metadata/dotnet-source-assessment-2026-10-03.json).

### 2026-10-03 — Explicit .NET unit storage and the next array boundary

Following the continuing paired-target work, the assistant implemented an opt-in .NET
source-void alias backed by a fieldless service-assembly unit value. This preserves ordinary
CLR no-result signatures without replacing the core or rewriting class-library sources.
The common compiler changes are isolated on the main-based fix branch (eb5df24b1) and
integrated as 97d07b901; 22 focused tests pass on both lines. No main merge was performed.

Assistant-verified results: separate .NET union, ArrayList, HashMap and query consumers
execute successfully; native broad execution remains passing. The .NET broad application
compiles and prints the expected prefix, then terminates at custom array-interface use.
The minimal array consumer reproduces signal 10. This is recorded as failure, with explicit
.NET array conversion/adaptation as the next bounded task. [Evidence](experiments/extended-cli-metadata/dotnet-unit-execution-2026-10-03.json).


### 2026-10-03 — Reassess direction against working .NET compilation

The author asked to inspect Raven main before further work, noting that the .NET target
worked before refactoring, then clarified: “I don’t want you to create new problems. So we
need to re-assess our direction”. The assistant stopped the proposed array-adapter work
and compared main `46491585e` with integration `97d07b901`.

Assistant-verified findings: array-interface projection, RuntimeIterationContract and the
expression generator are identical on those revisions. Both pass 36 focused array and
iteration tests. A normal compiler invocation using CLR IList<int>, shared array mutation
and LINQ compiles and executes with expected exit 42 on both. The custom-contract tests
include binding/emission assertions and do not establish execution of custom NeoCLR
interfaces on CLR vectors. The failing paired bootstrap manifest explicitly enables those
custom interfaces; that experiment is not the ordinary .NET baseline.

The assistant acknowledged that proposing an adapter as the immediate next step was
premature. Proposed direction: preserve the normal .NET dependency/runtime path, verify
shared-code changes against it, and assess native NeoCLR with its own library/runtime.
Cross-runtime source-library execution should be evaluated separately before it drives
more compiler changes. The author has not yet approved a replacement completion gate.
No compiler/runtime code changed; no adapters were implemented or prior commits reverted.
The earlier paired gate remains incomplete. This bounded audit does not prove the entire
compiler regression-free. [Evidence](experiments/extended-cli-metadata/dotnet-main-audit-2026-10-03.json).


### 2026-10-03 — Retain metadata library, reassess shared compiler paths

The author clarified that Raven runs on .NET while .NET and NeoCLR are distinct targets,
questioned whether the symbol-loader/codegen refactoring had gone too far, and asked to
preserve working .NET behavior while retaining the NeoCLR metadata library and finding
better shared paths. The assistant proposed an architectural audit rather than further
adapter work, then performed additional paired checks on main `46491585e` and integration
`97d07b901`.

Assistant-verified results: 71 invocation/receiver/constructor/field/loop/collection tests
pass on both. Metadata/import/propagation checks pass 44 on main and 45 on integration,
with the same single struct-constraint round-trip failure on both. The test expects
ValueType, while the imported symbol reports ValueType | Constructor after a previously
integrated implied-constructor flag change. That discrepancy needs separate reconciliation;
it is not evidence of a new integration-only regression. Earlier 36-test array evidence
remains applicable, but full compiler parity and captured-loop behavior remain unproven.

The audit recommends keeping native metadata/introspection/builders, the narrow semantic
reference provider, explicit emission backend and symbol-authored native references.
Its first simplification candidate is the additional release-only .NET portable body path:
it coexists with the established emitter and creates a parity obligation. Other candidates
are shared array lowering and legacy bridge branches, each requiring independent evidence
before changes. No production path was removed and no compiler/runtime implementation
changed. The CLR array adapter remains suspended. The author has not separately approved
each candidate; these are assistant recommendations under the authorized reassessment.

Detailed findings, reproduction filters and bounded next steps are recorded in Raven
`docs/compiler/architecture/neoclr-refactor-parity.md`, commit `10f5f0089`.


### 2026-10-03 — First stabilization and simplification slices implemented

Following the author's “Implement” instruction, the assistant reconciled the shared
constraint test without changing compiler behavior (`9bbcbb2b0`, independently isolated
as `e46c0a9c9`): source struct flags describe the written constraint while imported CLI
flags include the implied constructor. Four integration and seven isolated checks pass.

The next implemented slice, Raven `c2a66d82a`, removes automatic release-only portable
.NET body generation and its adapter; MethodGenerator always uses the established
MethodBodyGenerator. The native planner, metadata library and GetILGenerator adapter
remain. All 94 selected .NET tests pass before and after; the rebuilt compiler still
compiles and executes the unchanged native broad application against its separate library.

The loop-capture reproducer remains wrong: class-method callbacks yield 0 on main and
333 on the pre-removal integration branch instead of 123. This is not fixed by selecting
the established body emitter and is not a passing acceptance case. Shared lowering and
closure lifetime need independent work; the rest of the planned cleanup remains pending.
No new CLR array adapter, metadata format change or backend rewrite was introduced.
[Execution evidence](experiments/extended-cli-metadata/dotnet-emitter-simplification-2026-10-03.json).


### 2026-10-03 — Keep array expansion out of ordinary .NET lowering

Continuing the authorized simplification, the assistant investigated capture storage:
the established compiler allocates one method-wide shared closure, but for-loop iteration
values are written to locals while callbacks read hoisted fields. Integration's global
array expansion instead writes the shared field, so all callbacks observe its last value.
Correctness requires lexical closure lifetimes; copying every capture would lose outer
variable sharing and was not implemented.

The assistant scoped array expansion to explicit portable planning in Raven `19a3e84c0`,
retaining the established .NET loop route. All 86 focused checks pass. The unchanged
native broad application executes against its separate library; the existing labeled/
nested-loop C# fixture's Raven program also verifies and executes natively with exit 9.
The capture repro now prints main's 0 rather than 333; expected 123 remains unresolved.
This is a target-boundary cleanup, not a capture fix. Native introspection, builders,
ILGenerator and nominal Array<T> remain intact. The next cleanup is an inventory of
remaining native/legacy reference consumers; no speculative adapter was introduced.
[Evidence](experiments/extended-cli-metadata/portable-array-boundary-2026-10-03.json).


## 2026-10-04 — Prioritize full-library compilation by shared blockers

**Author direction.** The author asked to investigate the large blockers for compiling
the entire neoCLR System library and prioritize fixes by how much additional code
they enable, rather than implementing individual APIs for their own sake. The author
then emphasized: “We need a strategy”.

**Assistant investigation.** Ran a fresh full 166-file source compilation, twelve
source-family attempts and six minimal capability probes through the native driver.
Recorded source/artifact hashes, compiler/runtime revisions and complete diagnostics.
Full compilation stopped during binding. Missing runtime-service declarations dominate;
minimal probes independently expose unit, enum, floating arithmetic, inheritance,
mutable-capture and ordinary-driver Self limitations. An explicit-unit-contract
experiment still rejects the MemoryStream Result<unit, E> signature.

**Assistant proposal and recorded direction.** Prioritize capability batches:
inhabited unit and erased values; a checked service ABI catalog and ownership plan;
reusable enum/numeric/inheritance shapes; core identity/Self; callbacks/storage/generics;
typeof/introspection. API samples become acceptance cases. Time-zone-specific continuation
is superseded by this author-requested high-leverage strategy. No new compiler/runtime
capability was implemented in this assessment, and no subsequent author acceptance of
every proposed ordering detail is implied.

**Evidence and open work.** See the
[full-System strategy](experiments/extended-cli-metadata/system-compilation-strategy.md)
and its linked audit. Binding failures mask later phases. Preserve the existing native
gate and .NET controls; re-rank after each capability batch. Legacy translated snapshot
regeneration remains a separate recorded failure.

## 2026-10-04 — System.Value across nominal and structural values

**Author direction.** After requesting implementation of the capability batches, the
user clarified the intended carrier: “System.Value as a type than can hold any value
both structural and nominal types”, including types outside the System.Object hierarchy,
for passing values to introspection/reflection APIs.

**Assistant response.** Agreed to keep System.Value independent of the Object hierarchy
and preserve payload type identity. Proposed first connecting the already implemented
runtime erased carrier to native metadata using exact core identity, with scalar outcome
tests as the first integration proof, not a claim that every value category is supported.

**Actions and limits.** The inhabited-unit stream gate was committed first. Work is in
progress on native System.Value signature import/export and generic IsValue/UnpackValue
seed methods. Existing runtime erasure already validates payload storage and depth,
rejecting frame-backed references whose lifetime cannot escape. Broad nominal/structural
payloads, pointer/reference lifetimes and introspection call contracts still need explicit
coverage; this direction does not silently remove those safety requirements or turn
System.Value into System.Object boxing.


## 2026-10-04 — Native function types and function values

**Author direction.** The author clarified that function types are native in neoCLR,
that introspection should represent each signature as its own type, and that function
objects/references serve the role of delegates without nominal delegate identity.

**Assistant response and evidence.** Confirmed the distinction between a native function
shape and a bound target/receiver. Inspected the existing `FunctionTypeInfo` facade and
load-context interning, then extended its C# regression assertions for independently
authored equal shapes and different result types. Native metadata already carries
`Function { parameters, returns, no_result }`; no new format was introduced for this
clarification. Func/Action remain temporary Raven/CLI transport only.

**Actions and limits.** The source Tasks/Concurrency integration now binds callbacks on
constructed generic classes and interfaces. Unit-return callbacks use explicit generated
adapters where a no-result method needs a value result; this preserves distinct native
signatures and has an allocation cost. Structural Function language experiments remain
on their feature branches; metadata-only introspection does not invoke function objects.


## 2026-10-04 — Number means the entire numeric feature

The author directed: “Continue solving the rest of the issues with Number”, then
clarified: “I mean the entire feature around number classes/structs and Number interface”.
The assistant adopted the complete numeric feature as the active story: actual source
numeric structs, Number's static identities/operators and inherited ordering, concrete
parsing and generic consumers. Floating arithmetic alone is not the completion gate.
This follows the recorded numeric design (ten fixed-width integer/floating types;
Decimal, native-sized arithmetic contracts and a new Parsable interface are not inferred).

Actions so far: floating library/consumer execution now passes on both targets;
static interface metadata and scoped Self inheritance unblock the unchanged Number
source, followed by a separate ordinary struct and consumer. The independently useful
floating unary binding fix was verified on a main-based Raven branch and integrated
into local main as 87ba62572. No push is reported. The primitive source ownership/storage
mismatch is reproduced with actual Single.rvn: binding resolves Self to the canonical
primitive while the writer still treats the source owner as a nominal wrapper. This
requires an explicit representation contract; the ordinary Scalar acceptance fixture
is a static-contract test, not a substitute for primitive numeric implementations.
Full primitive-family and generic algorithm execution remain open.


A negative Number implementation then exposed a shared conformance bug: imported
static abstract properties were skipped during binding. A .NET-only, main-based
regression reproduced three invalid variants accepted before the fix; all 27 focused
interface/Self tests pass afterwards. The isolated fix b88a8d19d was fast-forwarded
into local main and integrated separately as 90c02af96. Missing/instance/wrong-result
identity properties now fail with RAV0330, independently of writer validation.


### 2026-10-04 — Numeric declaration storage and ownership

- **Author direction:** continue until the entire feature around numeric structs/classes
  and Number is solved, rather than stopping at floating arithmetic.
- **Assistant work:** completed integer-width/unsigned metadata and Raven emission,
  verified separately compiled consumers on both targets, then added explicit numeric
  scalar designation to the metadata definitions/builders and introspection. The native
  runtime already provides Runtime representation and scalar managed receivers; an
  API-authored primitive library and separate consumer now execute those operations.
- **Boundary decision implemented:** a System name alone does not claim intrinsic
  storage. Explicitly designated numeric declarations reject record fields/constructors
  and executable CLI output. Native snapshots and imported callable references retain
  the designation. No automatic source-field removal or bootstrap fallback was added.
- **Open:** Raven's canonical primitive-provider ownership, checked intrinsic `m_value`
  emission, and generic Number bounds/callself. This is not a claim of completing the
  author's requested full numeric family. See the
  [strategy checkpoint](experiments/extended-cli-metadata/system-compilation-strategy.md)
  and [API contract](../api-docs/experimental-metadata.md).

### 2026-10-04 — Future function-kind generic constraint

- **Author:** asked to define a future constraint admitting a function type, suggesting
  Raven's `func` keyword as the constraint syntax.
- **Assistant response:** proposed recording `where T: func` as a future function-kind
  constraint without nominal delegate identity, separate from current Number interface
  bounds. Any-function versus signature-specific constraints still need specification.
- **Existing context:** the structural-types proposal already sketches `where F : func`
  for Callable<F>. This exchange reinforces that direction; it does not implement the
  syntax, establish a metadata encoding, or promote structural Function experiments
  from their feature branches. See [Callable<F>](proposals/structural-types.md#19-callablef).
- **Action:** recorded the direction while continuing the owned-interface method-bound
  metadata slice. Open decisions include signature compatibility, generic inference,
  metadata representation and how the ordinary .NET target would reject or map it.


### 2026-10-04 — Number generic native integration gate

- **Author direction:** finish the Number feature, including the numeric classes/structs,
  and continue until the feature works rather than stopping at each prerequisite.
- **Assistant implementation:** completed method interface bounds, native semantic import,
  external static and constructed instance constrained calls, and generic forwarding.
  Explicit numeric ownership remains in the host/emitter contract. No importer handles
  enter shared code generation; default .NET behavior and structural Function branches
  are unchanged.
- **Observed outcome:** ordinary Raven commands rebuild all ten unchanged numeric source
  types and the cumulative library subset, compile a separate generic algorithms library,
  then compile and run a consumer using only their artifacts. Every Number member,
  inherited CompareTo and forwarding pass with exit 42 and empty stdout; the concrete
  parsing/boundary consumer returns 99. Invalid string arguments diagnose RAV0320 without
  publishing output. 140 C# metadata groups, 122 focused .NET tests and seven existing
  native consumers pass. See the [evidence](experiments/extended-cli-metadata/number-generic-end-to-end-2026-10-04.json).
- **Remaining scope:** this is completion of the bounded Number story, not the entire
  runtime library or all generic constraint forms. The previously stale guest API
  snapshot remains recorded; the experimental host API manual is updated.


## 2026-10-04 — Reassess the highest-unlock work after Number

- Author asked: “Re-assess what is left and what is next. Following the same objectives. The fixes that unlocks the most.”
- Assistant reran the source frontier using the proven numeric ownership profile. The
  70-source baseline compiles, while adding an empty first source file reproducibly
  causes ArrayList<byte>/List<byte> binding failures. The owning compiler defect is
  not yet identified or fixed.
- Assistant recommends repairing this composition stability issue first, then source-owned
  String/Char and shared text services, cross-assembly inheritance and introspection.
  The full library remains blocked in binding; previous Number execution stays valid.
- Roadmap and strategy now distinguish completed numeric prerequisites from open work.
  See [assessment and evidence](experiments/extended-cli-metadata/system-compilation-strategy.md#current-reassessment-after-number-2026-10-04).


### Follow-up: fix authorized and verified (2026-10-04)

- Author directed: “Go ahead fix”.
- Assistant traced the order-dependent failure to cached conversion answers obtained
  before source interface relationships completed, added a failing C# regression and
  fixed the shared cache boundary. Raven integration 459856a71 passes 139 focused tests.
- The regression also fails independently on main. Minimal prerequisite interface-list
  guards plus the conversion guard pass the same 139 tests on a main-based fix branch;
  f0c3b75a0 was fast-forwarded into main without the experimental backend.
- Empty-first/reversed native source builds and separate numeric, broad application and
  combined task consumers succeed. No metadata/runtime code change was needed. The
  next proposed feature batch is source-owned String/Char and shared text services.
- [Recorded execution evidence](experiments/extended-cli-metadata/source-order-conversions-2026-10-04.json).


## 2026-10-04 — Unicode text model versus native encoding

- Author asked whether string is a sequence of graphemes, char one grapheme, rune one
  Unicode scalar and byte one byte. Assistant confirmed the conceptual distinctions,
  identifying that the implementation currently uses uint plus UnicodeScalar helpers,
  not a dedicated rune type.
- Author clarified: “The runtime's native representation is UTF-8, but the runtime's
  native *text model* is Unicode text”. Assistant accepted this correction: graphemes
  and scalars are views of Unicode text; bytes describe its encoded representation.
- Actions: clarify the text API guide and preserve this distinction in native String
  integration. No rune keyword/type was introduced. String source ownership and instance
  execution are developed separately from bootstrap-owned grapheme Char.

### Follow-up: source text integration (2026-10-04)

- Author directed: “Don’t stop until you have solved Char and String”.
- Assistant extended source-owned Char through native symbols, metadata references and
  execution. The broader sample exposed missing grapheme literal/pattern/equality lowering
  and interface boxing/dispatch, which were implemented at their respective layers.
- The older sequence-construction sample exposed a bridge-only constructor. Assistant
  added the real constructor to String source and ordinary native constructor metadata
  and execution, instead of rewriting the sample or retaining competing seed declarations.
- Separate consumers now execute the unchanged grapheme, comparison, UTF-8 slicing and
  sequence-construction cases. Unicode remains the text model and UTF-8 the representation;
  no dedicated rune type or numeric Char category was introduced.
- [Integration details and reproducible evidence](experiments/extended-cli-metadata/source-text-2026-10-04.md).

## 2026-10-04 — Next milestone after Char/String

- Author asked “What is next?” and then directed “Proceed”.
- Assistant proposed refreshing the full-System audit, prioritizing shared blockers,
  and extending source-built text into streams and JSON with native artifact-only
  consumers and ordinary .NET regression checks.
- Performed: refreshed the 166-source audit; kept the 75-source baseline passing and
  verified the 81-source task combination compiles. A separate seven-source encoding
  library exposed a field-receiver stack defect, fixed in Raven's portable emitter.
  Native UTF-8/ASCII consumers and field-order/failure regressions execute.
- Open: text-stream required-result lowering, cumulative bootstrap String binding,
  JSON, class inheritance, metadata-handle services and full dual-target library parity.
- Evidence: [encoding milestone](experiments/extended-cli-metadata/source-encoding-2026-10-04.md).

## 2026-10-04 — Native inheritance exposes a general Raven constructor defect

- Author directed continued work on the immediate blockers and welcomed discovering
  general Raven issues while implementing the neoCLR target.
- Assistant extended the host metadata writer from reader-only class bases to local
  base authoring, constructor calls and inherited field layout, then connected Raven's
  semantic base/constructor operations to that writer.
- The paired test exposed an ordinary .NET omission of base initialization. Assistant
  reproduced it on Raven main, moved explicit initializer resolution after member
  registration, and retained canonical constructor symbols without skipping declaration
  diagnostics. A test caught and prevented a transient lost-static-initializer diagnostic.
- Performed: isolated the compiler fix as `2416a1646` on a branch based on main, with
  12 focused tests. Integration `f38dbfb75` passes 34 focused tests and paired driver
  execution. Metadata has 145 passing groups; text-stream consumers remain passing.
  Main has not been merged. [Evidence and remaining scope](experiments/extended-cli-metadata/class-hierarchy-foundation-2026-10-04.md).
- Open: JSON still fails before output at its closed-family signature. Closed/protected
  declarations, broader inheritance and the full class-library gate remain unfinished.

## 2026-10-04 — Integrate compiler fixes, retire branches, then closed families

- **Author:** asked to ensure valid Raven fixes reach main, then continue closed-hierarchy
  and protected-constructor support; subsequently asked to clean up integrated fix branches.
- **Assistant action:** fast-forwarded Raven main through the isolated constructor fix and
  seven remaining independently tested commits. Main is `4f95db536`; 148 focused .NET 11
  checks passed. Deleted the four integrated local fix branches, retaining detached clean
  worktrees and leaving native integration/structural experiments separate. No push performed.
- **Continuation:** implemented runtime-only protected constructor family checks and closed
  class direct-child ownership checks, with binary cross-assembly execution and denial tests.
  General protected members, writer/facade/Raven admission and JSON execution remain open.
- **Evidence:** [hierarchy integration record](experiments/extended-cli-metadata/class-hierarchy-foundation-2026-10-04.md),
  `tests/protected_constructors.rs`, Raven `docs/compiler/general-fixes-from-neoclr.md`.

### Follow-up: revisit closed-family metadata encoding

- **Author:** asked to continue directly with the next slice, then clarified that a later
  design pass should investigate encoding closed hierarchies directly in metadata
  (correcting the word "IL").
- **Assistant interpretation/action:** retain the existing native closed-family flag and
  CLI-shaped Abstract declaration for this bounded integration. Do not conflate closure
  with CLI Sealed or invent an instruction. Richer native family/permits encoding remains
  a later design decision, not a prerequisite or a claimed completed format redesign.

- **Author follow-up:** asked how closed hierarchies are encoded now.
- **Assistant explanation:** native type rows carry is_closed_hierarchy/Abstract/nonsealed
  flags; direct children are inferred from ordinary base relationships in the defining
  assembly, without a separate permits list. The PE native payload retains this contract;
  the CLI reference projection retains Abstract only, and ordinary .NET emission keeps
  its attribute-based representation. A richer explicit family/permits representation
  remains future work.
- **Verified outcome:** paired direct and separately compiled library/consumer cases run
  on both targets and reject external direct children. Raven integration `d618c9667`;
  146 C# metadata groups and 35 Raven regressions pass. JSON progresses to propagation
  lowering. [Milestone and evidence](experiments/extended-cli-metadata/closed-family-2026-10-04.md).


## 2026-10-05 — Reflection/object mapping as the bridge to networking

- **Author:** "Once we have reflection and object mapping ready, that proves internal
  calls snd such work, then the socket snd http stack is not far away".
- **Assistant response:** reflection/object mapping supplies reusable evidence for calls
  carrying source-owned types into the runtime. Socket/HTTP error, lifetime and async
  behavior still require their own tests. Keep JSON mapping as the current gate and
  the network stack next; do not treat those later capabilities as complete.
- **Current implementation evidence:** the handle/reflection service gate executes
  constructor state and identity through native assemblies. The metadata API now
  authors bodyless runtime service declarations; its C# API consumer executes against
  the real runtime with an explicit test seed. Raven source admission, production
  descriptor factories and JSON object mapping remain open.
- **Links:** [runtime-service authoring](experiments/extended-cli-metadata/internal-call-authoring-2026-10-05.md),
  [handle/reflection gate](experiments/extended-cli-metadata/native-handle-reflection-2026-10-04.md).

- **Author follow-up:** "Implement the necessary types in NeoCLR runtime library if necessary".
  **Action:** the tested native service declarations now live in the runtime library's
  native source directory. Existing Object/RuntimeTypeHandle identities are reused;
  no additional descriptor types were needed for this bounded gate. The compiler-facing
  marker remains bootstrap metadata until source-built core ownership is integrated.
  [Source declaration evidence](experiments/extended-cli-metadata/source-internal-calls-2026-10-05.md).

### 2026-10-05 — Source-built native JSON mapping gate

The author directed: “Continue work until you have solved JSON object mapping” and
previously authorized necessary runtime library types. The assistant continued through
metadata finality/Boolean ownership, portable conversion and accessor emission, then
scoped runtime descriptor materialization and internal snapshot/service adapters.
Production mapper/serializer/introspection sources and the earlier mapping sample were
kept unchanged. Two artifact-only consumers now verify and execute with exact output;
nested/jagged-array mapping, mutation/shared identity and rejection before model side
effects are checked. [Commands, hashes, dependencies and scope](experiments/extended-cli-metadata/source-json-mapping-2026-10-05.md).

This is an implemented native integration checkpoint, not a claim of full reflection,
full-System, .NET library parity or HTTP completion. The assistant recorded the explicit
bootstrap and instruction budget and proposed inventory/network reassessment next;
no new author decision about that subsequent work is recorded here.

### 2026-10-05 — Compile the library to reassess the next blockers

The author asked, “Re-asses by compiling the library,” then directed, “Continue fixing
it.” The assistant compiled the full 166-file source inventory, cumulative groups and
artifact-referenced additions. The 75-file baseline passed; the cumulative build exposed
String.SliceUtf8 source/import lookup differences, while storage and network groups
showed missing compiler-facing service contracts. A minimal probe failed with source
String and passed with the emitted type. The assistant prioritized that shared blocker.

Explicit source primitive member selection fixed the binding discrepancy. The cumulative
109-file build then emitted and ran JSON consumers. Adding Tasks exposed the metadata
writer's 256-type cap; aligning authoring/native limits with the existing bounded CLI
reader admitted the 115-input library. Both JSON consumers and the task consumer execute
against it. [Recorded evidence and remaining scope](experiments/extended-cli-metadata/cumulative-library-type-budget-2026-10-05.md).
Storage/network contract families and source/bootstrap ownership remain open; this is
not recorded as full-System or dual-target class-library completion.

### 2026-10-05 — Assembly inspection for integration debugging

During storage integration the author suggested, “Perhaps we should build a
command-line decompiler for neoCLR assemblies. If that speeds up debugging.”
The assistant proposed first investigating a read-only disassembler showing signatures,
scoped references and instruction bodies. Inspection found that `emit-il` lowers source
and `verify` checks loaded assemblies, but neither displays an assembly's decoded bodies.
A bounded metadata/body dump is the proposed first tool; source reconstruction is a
larger follow-up. No decompiler implementation or author selection between those scopes
is recorded. Storage work continued and its unchanged sample executed successfully.

### 2026-10-05 — Benchmark against .NET after full class-library compilation

The author directed that, once bootstrapping allows the entire class library to
compile, equivalent neoCLR and .NET programs should be benchmarked, with benchmarking
included on the website when this work is released. The assistant accepted this as
a post-bootstrap release gate and recorded reproducibility, equivalent behavior,
startup versus warmed execution and transparent limitations in the platform roadmap.
No benchmark measurements or website publication have been performed for this request;
ongoing HTTP propagation fixes retain their current priority.

### 2026-10-05 — Complete the native HTTP integration gate

The author directed, “Continue until you have solved Http API.” The assistant worked
through generic callbacks, propagation in call arguments, captured self/private access,
property patterns, boxed enum behavior and managed entry arguments. The complete HTTP
source group now emits and separate artifact-only consumers and loopback harnesses pass
all 25 gate commands, including eleven stream-upload cases. Production HTTP sources
were preserved; two stale sample expectations were corrected with explicit checks.
The generally useful propagation fix was independently tested and integrated into local
Raven main. No remote push is recorded.

The assistant records callback HTTP as working and native async state-machine emission,
full-System ownership and a preexisting Array/Object generic dispatch failure as open.
This does not complete the later benchmark/release milestone. See the
[HTTP evidence and limitations](experiments/extended-cli-metadata/source-http-2026-10-05.md).

### 2026-10-05 — Editor experience becomes a release requirement

The author said, “The goal for the release is to also have the editor experience working.
Language server support consuming and emitting neoClr metadata. Perhaps also having the
disassembler.” The assistant accepted native metadata-backed editor/LSP support as a
release gate and proposed using the shared semantic importer and ordinary compiler emitter
for editor builds, preserving .NET behavior. The disassembler remains a candidate because
the author expressed it provisionally. The roadmap records acceptance expectations and
keeps current Object ownership work active. No editor or disassembler implementation is
claimed by this documentation action.

### 2026-10-05 — End-to-end includes VS Code

The author clarified, “Our goal is to get the end to end scenarip working with lang
server and VS Code support.” The assistant accepted editing against native metadata,
then compiling and running from the same project configuration, as the end-to-end gate.
The current root-authoring prerequisite remains active; the editor must share the
compiler importer/emitter and dependency catalog. Declaration authoring now passes
156 C# metadata groups, but root-slot/compiler wiring and editor execution remain open.
This clarification does not claim that a VS Code workflow has been implemented.

### 2026-10-05 — Samples require Tasks and await for release

The author said, “Also for the release, samples must work, so tasks and await must work.
In the future we might implement runtime suspension, and green threads, but not now.”
The assistant accepted native Tasks/await sample execution as a release gate, using the
current execution model, and kept runtime suspension/green threads deferred. Existing
callback Tasks and translated async evidence must not be mistaken for native compiler
async completion. The roadmap now directs a sample inventory and native async gap work
after the active root-ownership prerequisite. No async implementation is claimed here.

### 2026-10-05 — Disassembler follows source-root execution

The author said, “If this is successful, then we should start building a disassembler
for neoclr metadata files. We will need that. Then we can resume the end-to-end work.”
The assistant accepted that sequencing. Source-root compiler emission now executes in
NeoCLR, with 47 focused compiler and 15 runtime checks passing. The metadata disassembler
is now the next bounded task, rather than merely a release candidate. Broader integration
resumes afterward; production System, editor support and Tasks/await remain open.

The assistant subsequently implemented `neoclr disassemble` using the existing native
container decoders. It displays metadata facts and indexed instructions without linking
or executing; it is explicitly a diagnostic listing, not reassemblable neoIL. Six new
checks and nine existing CLI/source-emission checks pass, including the Raven-emitted
source-root PE. [Usage and scope](il-inspection.md). Broader end-to-end work resumes with
source-root driver/consumer ownership and constructed local bases still open.

### 2026-10-05 — POC scope and required sample compilation

The author said, “Our goal is end-to-end POC for neoCLR. Not everything has to be solved
for the release. Samples however need to be compiled.” The assistant accepted this
scope correction: sample compilation is required; full class-library/platform feature
completion must not become a prerequisite for the POC. The assistant proposed a normal-
compiler sample inventory after committing the active driver slice, prioritizing shared
blockers rather than API completeness. That inventory is pending, not a reported result.

The active slice now compiles a root library through `rvnc neoclr` and executes it with
explicit root selection through the NeoCLR CLI. The caller is neoIL, so a separately
compiled Raven root consumer remains open. Thirty runtime/CLI checks and the driver
acceptance controls pass. See [the reproduction](experiments/extended-cli-metadata/source-object-driver-2026-10-05.md).

The assistant then ran the selected ten-sample native driver inventory: three compile
and execute with exact output, five stop at native Task async recognition, and two expose
constructor/inheritance gaps. The [recorded evidence](experiments/extended-cli-metadata/poc-sample-inventory-2026-10-05.md)
changes the next bounded priority to native async identity/lowering. Full Object-root
replacement is not required by the already-executing retained-seed samples. This is
assistant-observed evidence and a proposed implementation sequence, not a new author
requirement or a claim that all release samples are covered.


### 2026-10-05 — Website is part of release readiness

The author said, “For a release we also have to update the website.” The assistant
accepted this as part of the release gate and updated the homepage and Raven page to
distinguish the published bridge from current native development. Superseded native
progress notes were consolidated into current capabilities and limits: nine of ten
original samples have execution evidence; inheritance and native editor/LSP acceptance
remain open. Matching downloads, setup instructions and API reference must be reviewed
against the eventual release artifacts. This is documentation preparation, not release
creation or website publication.

The assistant regenerated the CLI API reference using Raven main `08f34891b`, corrected
stale Fail links, generic-signature markup and metadata-document routes, then verified
all 1,803 generated pages and 18 website tests. Browser inspection confirmed the changed
homepage and native-target content. Reference generation against integration `9a4f74884`
exposed an unresolved Option/None binding regression; the same current bridge succeeds
on main. This difference is recorded for compiler repair, not treated as solved by the
website refresh.

The subsequent compiler investigation resolved the Option/None regression at lexical
case lookup. It also reproduced a cold semantic-query defect on main, so the assistant
validated and integrated the general fix there independently. The repaired integration
compiler produces the same reference bytes as main; 190 main and 196 integration checks
pass, along with separate union-library consumers on both targets.
[Compiler and runtime evidence](experiments/extended-cli-metadata/union-lexical-cases-2026-10-05.md).


### 2026-10-05 — Continue through actual VS Code acceptance

The author directed: “Continue untill you have VS Code acceptance.” The assistant
proposed closing native reference refresh, declaration navigation and shared project
build/run, then testing the real extension host instead of counting a stdio-only check.
Implementation reused evaluated project configuration and the explicit metadata catalog;
source-built library references remained native. The assistant observed 14 passing
VS Code checks, including actual broad-sample and Tasks/await execution, artifact
replacement, missing-dependency recovery and a .NET editor control.
[Evidence](experiments/extended-cli-metadata/native-vscode-acceptance-2026-10-05.md).

During validation the author reported an inconsistent union hover, specifying
`Option<T>(Some<T> | None)`, then clarified: “It's technically correct though”.
The assistant corrected its initial suggestion that this necessarily exposed a carrier
constructor: expanded union case notation is valid. No display-format change was made.
Native type-position hovers were checked for union kind and constructed arguments;
runtime-specific interface signatures remain visible. The first VS Code test timeout
was traced to the test serializing MarkdownString as an empty object, not a product
hover failure; the test now reads its value property.

This completes the bounded native editor POC on the tested development toolchain,
not release packaging, publication, full decompilation or the entire source-library
bootstrap. Those distinctions remain explicit in the roadmap and website.

## 2026-10-05 — API help in the editor and release readiness

**Author:** Asked whether the POC was ready for release, requested an updated website
if so, and asked to enable documentation in the IDE. Clarified that XML or Raven
Markdown can serve API descriptions, while website guides need not all be inline
comments: “What I basically meant was that the user should get some documentation
displayed in VS Code”.

**Assistant action:** Kept release publication pending matched-package/install
qualification, reused Raven's existing Markdown-first/XML-fallback documentation
model for native references, added native project sidecar generation and checked
documentation in the actual VS Code extension host. Updated development website
instructions without changing published Preview 11 claims. This does not assert
that every runtime API already has bundled documentation.

**Author follow-up:** Explicitly included the website and API rendering through
RavenDoc in the release gate. **Assistant action:** Tested the extracted SDK with
an isolated installed VSIX (19 checks), discovered missing union-case summaries on
the rendered site, and integrated the independently proven general documentation-ID
fix into Raven main. Kept CLI carrier details in website support documentation while
sharing useful authored API prose with native hover. The website build and 18 tests
pass. The release remains unpublished: clean-source validation rejects stale bootstrap
snapshots, and attempted regeneration exposed field-address initialization in the
legacy DateTime bridge. No snapshot hashes were forged or output fragments published.
[Evidence and next gate](experiments/extended-cli-metadata/native-release-gate-2026-10-05.md).

The full extracted-source optimized runtime test run subsequently completed with
1,782 passes and two stale test failures. The assistant corrected only those
expectations and verified all ten affected tests. Runtime behavior was unchanged;
the canonical source snapshot audit remains failed.

## 2026-10-05 — Native metadata is the release path

**Author:** Emphasized that the APIs must work, then corrected the assistant's focus:
“The aim is to get rid of the bridge and use neoclr metadata sources for neoclr.”
**Assistant:** Stopped the legacy bootstrap regeneration, discarded the uncommitted
DateTime bridge changes, and replaced that work with a reproducible native library
build and executable artifact-only consumers. Rebuilt 115-source Numbers and 19-source
Http with the extracted SDK; 15 sample compilations, 13 non-network executions and
both HTTP/JSON loopback rounds passed. The primitive core and retained seed are still
explicit bootstrap inputs; full removal is not claimed. The failed legacy source audit
remains recorded but is not a rationale for further bridge feature expansion.
[Evidence](experiments/extended-cli-metadata/native-source-release-2026-10-05.md).

## 2026-10-05 — Callback signatures in API lists and tables

**Author:** Supplied screenshots of `Run(callback: Func<Void>) -> Task<void>`, the
same representation in its parameter table, and `FlatMap` showing `Func<Char,
Iterable<U>>`. Suggested `ToDisplayString` and clarified that the member list was
affected. Requested function signatures such as `() -> Task<void>`.

**Assistant investigation/action:** Found that member lists explicitly selected
nominal delegate names and parameter tables separately rebuilt named generic types,
bypassing normal function display. Corrected both RavenDoc paths, preserving nominal
delegate declarations and links inside generic return types. Also prevented parameter
names splitting across lines. Twenty generation tests cover source/imported metadata;
the actual 1,803-page site and 18 checks pass. Browser inspection confirmed Task.Run
and String.FlatMap. The screenshot's completion-only overload correctly displays
`() -> void`; the task-producing overload displays `() -> Task<T>`. No callback or
runtime API contract changed. The shared fix is on Raven main `8ddd73c31`, and the
publisher is pinned to it; no website deployment was performed.

## 2026-10-05 — Native bundle installation gate

**Author direction carried forward:** Continue toward release, using native metadata
sources and ensuring APIs execute correctly.

**Assistant action/outcome:** Added explicit-input packaging and an extracted-bundle
verifier. Five artifact-only projects compile; collections, Tasks, JSON and live HTTP
execute. Installed VSIX acceptance passes 19 checks. Website development status now
reflects this local gate. Primitive core/retained seed provenance and publication
remain open; neither a released download nor complete System bootstrap is claimed.
See [evidence](experiments/extended-cli-metadata/native-bundle-2026-10-05.md).


## 2026-10-05 — Preview 12 and main integration authorized

**Author:** Asked to continue release and merge the NeoCLR repository work into main
when done; selected Preview 12 with macOS arm64 as the qualified native toolchain.

**Assistant action:** Reproduced the primitive core and retained runtime seed with
identical hashes, pinned seed source for release reproduction, and prepared the
version update. Merge/publication outcomes are recorded separately after execution.


**Release qualification follow-up:** Both bootstrap bytes reproduce from source.
The clean candidate archive passes extracted execution; all 1,784 local runtime tests
pass across 236 targets. Website build/checks pass (1,803 pages, 18 tests). Hosted
minimum-Rust and OS boundary checks pass; canonical validation is still running.
The native source profile retains full runtime tests and source smoke checks while
the legacy bridge snapshot audit remains explicitly separate. A stale deleted
sample path and newer-Rust deprecation diagnostics were corrected. Main integration
and publication await the final hosted outcome.


**Hosted outcome:** Run [37355774315](https://github.com/marinasundstrom/neoCLR/actions/runs/37355774315)
passed canonical full source/runtime validation, both OS boundary jobs and all three
minimum-Rust jobs. This clears the candidate gate for the authorized main integration
and Preview 12 publication. The packaged runtime's executable sources differ only by
subsequent compatibility annotations; later release commits document evidence.


**Publication outcome:** Fast-forwarded NeoCLR main to `476d61af`, pushed main and
published [Preview 12](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.12)
on 2026-10-05. Downloaded the published macOS arm64 archive and verified its SHA256
against the accepted candidate. The manual Pages deployment passed; public homepage,
installation instructions, Task.Run member/parameter signatures and String.FlatMap
function signatures were checked. [Publication evidence](preview-12-publication.json).
Full native System bootstrap and broader platform qualification remain future work.

## 2026-10-07 — Shared RavenDoc website improvements

The author requested migrating Raven's full website from DocFX to RavenDoc,
retaining its structure/design and a visible API Reference for Raven.Core and
Raven.Macros. They requested separate slices for configurable site search, code
copying, nested-type listings and namespace documentation, and asked that the
RavenDoc changes also reach neoCLR's site. They suggested using neoCLR's site as
a design reference and specified an icon-triggered search field that can retain
a query. The assistant implemented the shared changes on Raven main and applied
the publisher/configuration changes to neoCLR main. The author requested local
site review when complete; publication was not requested. Validation results are
recorded in the integration notes and changelog.

During site review, the author clarified that library APIs must share Raven's main
website shell and namespaces should be listed flat, as on neoCLR. They accepted
copy buttons on signatures but required them to float without adding top spacing.
The updated shared publisher carries that layout correction to neoCLR as well.

The author next requested a clearer reading sidebar, responsive landing-page
spacing and source display names (“Core extensions” and “Macros”) distinct from
assembly filenames. Raven's authored content and spacing were updated, while
the shared publisher's navigation/display-name support and compact mobile header
were propagated here. A missing macro-only API sidebar was fixed at the same time.

The author asked for a click-through review of Raven's page organization and for
RavenDoc features to remain general site-generator capabilities cross-checked
against neoCLR. The publisher now offers site-wide or section-local navigation;
Raven uses one composed article hierarchy, while neoCLR explicitly keeps its
existing section/API boundaries.

The author emphasized that navigation should follow simple content conventions,
with configuration for deliberate overrides; repository-specific organization
belongs in content/TOCs rather than hard-coded RavenDoc behavior.

The author refined the organization: Getting started (renamed from Docs) and
Language reference should keep distinct hierarchies because they are separate
navbar destinations. RavenDoc gained explicit directory/TOC boundaries and labels
without changing neoCLR's existing navigation policy.


## 2026-10-07 — Shared RavenDoc assembly sources and future processors

The author requested that general RavenDoc fixes also reach the neoCLR branch
and site, and that libraries split across System.Runtime, System.Data and
System.Networking share one namespace tree while retaining assembly identity.
The author also proposed future customization of Markdown and code-source
handling through processor extensions, and later clarified a loader-independent
member model for Raven and C# inputs.

The assistant implemented a configurable assembly-input list for the site,
using its current combined CoreProbe snapshot without inventing separate native
assembly identities. The shared RavenDoc implementation supports grouped
assembly namespaces, cross-source links, C# source lookup, static-type metadata
and nested types on their owner's page. Loader/model/renderer extensibility is
documented as future direction; no plugin API is introduced in this release.
Validation and the pinned generator revision are recorded in the changelog and
`tools/ravendoc/version.json`. These local changes do not publish the neoCLR site.


### Follow-up: opt-in dynamic navigation

After the Raven compiler API reference exposed substantial repeated sidebar
markup, the author requested dynamic loading without a SPA framework, then
specified that it must be opt-in for large sites. The assistant implemented
`sharedApiNavigation`, defaulting to false, and enabled it in both Raven and
neoCLR site configurations. Normal page navigation and namespace fallback links
remain static. This is a site-generator capability rather than a neoCLR overlay.


### Follow-up: full-height mobile navigation

The author reported that the shared API drawer stopped above the bottom of a
mobile viewport and directed that it be fixed after the Raven release through
an incremental site publish. The shared RavenDoc fix excludes modal drawers
from the desktop height reservation. neoCLR adopts the same upstream generator;
no target-specific stylesheet workaround or neoCLR deployment is introduced.

## 2026-10-06 — Full bootstrap inventory and first source-core fix

**Author:** Asked what full bootstrap requires, then directed continuation.

**Assistant proposal/action:** Distinguished source-built System from later compiler
self-hosting; prioritized a fresh inventory and source core ownership. The audit found
that existing Storage adapters and six omitted contract files already compile. The
all-source mode exposed a PE-only reflection-loader cast; isolated Raven fix
`6b31997a9` removes it and unifies imported Object signatures with the selected source
root. All 41 focused tests pass; released library compilation controls remain green.

**Remaining:** Union ToString synthesis currently requests source Object members too
early and aborts the full build. Declaration ordering is the next priority before
interpreting wider core/service diagnostics. No full-bootstrap completion or new
execution support is claimed. See [frontier](experiments/extended-cli-metadata/native-bootstrap-frontier-2026-10-06.md).


**Union follow-up outcome:** Raven `d29179810` completes the source Object.ToString
signature through the existing lazy declaration path. Four ordering regressions pass,
206 existing tests remain green, and both full-library layouts now reject normally
without output. Keeping bootstrap RuntimeTypeHandle removes the typeof configuration
error and seven conversion errors; both layouts expose 45 missing service-member
diagnostics. The next bounded work is canonical handle ownership and coherent runtime
service inputs, followed by reduced residual failures. No full-bootstrap success or
change to published Preview 12 is claimed.


**Ownership follow-up correction:** The source-handle support had already been implemented
in the October 5 gate. The new audit omitted its explicit primitive ownership selection.
Reusing that contract and removing the competing seed handle reaches 74 diagnostics with
all 178 inputs, matching the retained-handle control. This is an audit correction, not a
new compiler fix. Missing service families now have priority.

**Author question on main:** Asked whether general fixes are committed to Raven main.
The assistant confirmed that independently validated general fixes belong there and
clarified that the two latest source-Object fixes remain isolated because their regression
contract is absent on main. Inspection confirms main has the earlier RavenDoc, union-doc,
lexical-union and reference-assignment fixes; no new main integration is claimed here.


**GC continuation:** On the author's continuation, the assistant added source adapters
for the existing GC services. Executing a separate consumer exposed the no-result
service mismatch, now fixed with nine GC tests and a focused admission test. The native
retention consumer executes, but the valid null-call consumer still rejects because
nullable parameter facts are lost on import. The assistant proposes making nullable
metadata preservation the next shared blocker rather than expanding isolated API
facades. The null fixture remains recorded; this is not full GC API or bootstrap
completion. See [partial gate](experiments/extended-cli-metadata/source-heap-2026-10-06.md).

## 2026-10-06 — Raven use cleanup without exception regions

The author proposed `use` support on Raven `codex/metadata-consumer`, requiring
cleanup on block exit, return and error/None propagation because neoCLR has no
exception unwinding. The assistant proposed shared scope-exit lowering with explicit
target contract selection and an initial synchronous slice. The author directed
implementation, then clarified that general improvements may be added to main.

Implementation stays on the requested Raven branch with general lowering/tests
separate from neoCLR's protocol selection; no main merge is claimed. Six native
protocol fixtures verify and execute, while async/iterator use remains diagnosed.
See [the integration record](raven-cli-bridge.md#raven-synchronous-use-cleanup-2026-10-06-development)
for contracts, limitations and validation. This entry records the scoped compiler
work and does not reprioritize the remaining platform roadmap.


**Nullability direction (2026-10-06):** The author asked, “Can we add nullability another
way? Or should we defer?” The assistant proposed preserving declaration annotations
using the existing .NET nullable-attribute convention while deferring new native runtime
semantics. On continuation, the first slice implements explicit callable annotations in
the host metadata API and CLI PE path, with .NET interpretation and execution tests.
Native transport and Raven reconstruction remain open; KeepAlive(null) is not fixed by
this first slice. See [scope, alternatives and next gate](design/callable-nullability.md).


**Raven branch consolidation (2026-10-06):** The author requested inclusion of the
latest `codex/metadata-consumer` work to enable synchronous use cleanup, suggested
retiring that branch, and explicitly selected `codex/source-object-metadata-resolution`
for future work. The assistant merged the cleanup commit there as `7bfc6ad27`, retaining
both source-root fixes. Only documentation conflicts needed resolution. Validation:
54 baseline checks, 92 post-merge checks, and six native consumers verifying and exiting
42. The old clean checkout remains available; it is superseded for new work, not deleted.
Native async cleanup and nullable metadata integration remain open.


**Nullable API issue resolved (2026-10-06):** The author directed continued work until
resolution. The assistant extended native callable declaration metadata and then Raven
import/emission, preserving explicit nullable facts without new runtime null semantics.
The original KeepAlive(null) fixture now compiles from emitted GC references alone and
executes successfully. Array/generic symbol checks, 17 .NET checks and seven native
consumers pass. A shared nullable-generic storage correction is isolated (`7ac1fec4a`);
its owning portable layer is absent on main, so no main backport is claimed. A stale
sealed-class expectation was corrected separately (`d1efc68f6`). Context/field metadata
and full bootstrap remain open. See [design and gate](design/callable-nullability.md).


**Managed array direction (2026-10-06):** During source Environment integration, the
assistant found that the old runtime service returned a temporary value-array buffer
where Raven emits a managed reference. The author clarified that managed arrays are the
supported model and inline value arrays have no current story, apart from possible
future interop. The assistant retains Array<T> backing and materializes fresh managed
argument arrays at the native boundary; legacy transport compatibility is an internal
detail, not a public feature direction. The Environment artifact-only consumer passes,
including snapshot mutation, and four runtime boundary tests pass. Full-System compilation
still rejects with 48 diagnostics. See [gate](experiments/extended-cli-metadata/source-environment-2026-10-06.md).

## 2026-10-06: Prefer specific temporal types

- **Author:** Clarified that DateTime should not lead the documentation; it is an
  option when representing either a local or a zoned date. “Specific date types are preferred”.
- **Assistant:** Agreed and moved the optional union explanation below the primary
  time-zone workflow, updating feature navigation and its manual API supplement.
- **Implementation context:** The calendar bootstrap exposed constructor-union import
  support as a compiler gap. Testing that representation does not change the public
  recommendation to use specific temporal types.

## 2026-10-07 — Aggregate bootstrap coverage versus library packaging

- **Author:** “We might split the System.Runtime because not everything is part of
  core. System.Data. System.Networking. System.Web.”
- **Assistant response:** Agreed to treat the all-source build as coverage rather
  than a shipped layout and to map dependencies before choosing assembly boundaries.
- **Performed work:** The larger-envelope prototype lets all 197 diagnostic inputs
  emit; runtime admission still rejects the source Array backing contract. A separate
  API-authored large-library consumer executes. These are distinct gates.
- **Open:** Exact core membership, optional-library dependency edges and native service
  ownership; separate-library execution is not yet established. The proposal does
  not imply a final assembly naming/ownership decision.
- [Implementation evidence and tradeoffs](experiments/extended-cli-metadata/expanded-library-2026-10-07.md).

### Follow-up evidence for the 2026-10-07 packaging discussion

- **Assistant investigation:** Counted emitted local references with nested union
  cases assigned to their declaring owners. Observed Data → Runtime, Networking →
  Runtime, and Web → Runtime/Data/Networking; no local reverse edge from the Runtime
  remainder. Seed dependencies and native adapter ownership still need review.
- **Performed action:** Removed the duplicate retained Object in the diagnostic
  audit; the full-source artifact still emits. Runtime admission now identifies a
  retained/source WriteLine result-contract mismatch. No assembly split is implemented.
- [Inventory and ordered proposed gates](experiments/extended-cli-metadata/library-boundaries-2026-10-07.md).

## 2026-10-07 — Void is the NeoCLR unit type

- **Author clarification:** Raven's separate .NET unit representation exists because
  CLR void cannot occupy value/generic-argument positions. NeoCLR does not have to
  inherit that restriction: “we only needed the void type, because we have given it
  the meaning of the unit type.”
- **Assistant response:** Treat NeoCLR void as the canonical inhabited unit, including
  generic arguments and callbacks. Preserve .NET carrier lowering inside the .NET
  target. A callable's no-result stack convention remains an encoding distinction,
  not a second language type.
- **Performed work:** Explicitly designated the selected System.Void declaration as
  native unit storage, including output-owned external references. Metadata and native
  callback/generic checks pass; a separately compiled Raven NativeMemory consumer
  executes with unit parameters and generic interfaces. .NET unit regressions pass.
- **Open:** Full-System admission reaches retained-seed dependency ownership. This is
  not full bootstrap or a completed optional-library split.
- [Evidence](experiments/extended-cli-metadata/canonical-unit-2026-10-07.md).

## 2026-10-07 — Source-owned broad application acceptance

- **Author's earlier objective:** Compile the runtime-library sources, consume their
  emitted metadata without including those sources in the application, and execute
  unchanged application-order-collections. Keep bootstrap dependencies explicit and
  preserve ordinary .NET behavior. Later suggested separating optional Data,
  Networking and Web libraries from core.
- **Assistant implementation follow-up:** Added an explicitly selected imported Object
  owner to Raven's semantic import contract and compiler driver. This resolved the
  competing bootstrap/source Object identities without relaxing inheritance checks.
- **Observed outcome:** The native consumer compiles, verifies and runs with exact
  expected stdout, no stderr and exit 0 against the separately emitted 197-input
  aggregate. The C# root probe and 47 focused compiler regressions pass. Reproducible
  commands and hashes are checked in; the website labels this as development work.
- **Still open:** Production assembly boundaries, project/editor ownership catalogs,
  wider API execution coverage and removal of permitted bootstrap inputs. This outcome
  does not establish a new release or imply every API has been executed.
- [Acceptance evidence](experiments/extended-cli-metadata/source-owned-orders-2026-10-07.md).

## 2026-10-07: Platform distribution packages versus assembly ownership

- **Author:** “Platform packages perhaps need to be their own package and potentially
  more pacakages”.
- **Assistant proposal:** Separate managed foundation/optional libraries, OS/architecture
  runtime payloads and developer tools; packages may contain multiple assemblies and
  native assets. Keep further splits evidence-driven.
- **Author follow-up:** Asked to continue. This is not recorded as approval of specific
  package IDs, formats, versioning or a native plugin architecture.
- **Action:** Recorded the [candidate package boundaries](experiments/extended-cli-metadata/package-boundaries-2026-10-07.md),
  their compatibility/qualification gates and .NET asset-selection comparison. Inspected
  imported Object override validation across metadata authoring, method references and
  reader reconstruction; documented the coordinated change needed rather than weakening
  one check.
- **Outcome/open:** Documentation only in this slice. Runtime/orders acceptance remains
  the prior passing evidence; optional-library execution, package implementation and
  CLI/LSP catalog propagation remain open.

## 2026-10-07: Runtime as the dependency foundation

- **Author:** Directed that the system.runtime package be well defined around core
  runtime functionality, with all other packages depending on those fundamentals,
  “and not the other way around.” Said this direction need not copy .NET exactly.
- **Assistant response:** Agreed to enforce dependency direction and distinguish
  fundamental mechanisms from optional policy. Proposed Runtime-owned managed array
  reflection consumed by Data-owned JSON mapping as the current concrete boundary.
- **Action/outcome:** Added ArrayReflection and migrated ObjectMapper off internal
  cross-assembly services. Separate Runtime/Data consumers now execute nested JSON,
  setters, arrays, mutation and identity checks. Corrected imported Object call names
  and native root type-syntax ownership exposed by that execution.
- **Open:** Exact minimal Runtime membership, separate Web acceptance and distribution/
  editor catalogs. No package IDs, registry or version scheme was selected here.
- [Dependency rule](experiments/extended-cli-metadata/package-boundaries-2026-10-07.md),
  [execution evidence](experiments/extended-cli-metadata/separate-data-2026-10-07.md).

## 2026-10-07: A unified class-library API reference

- **Author:** Asked to continue and proposed presenting the split class library “as one”
  in the RavenDoc-generated site, without separate sections, while retaining where each
  type/function is declared (assembly/file provenance).
- **Assistant response:** Proposed one namespace/type navigation tree with assembly
  provenance on declaration pages. Distinguished logical assembly identity from a PE
  container, package and namespace.
- **Action:** Reviewed RavenDoc's existing per-API grouping and the aggregate docs bridge.
  Recorded a unified native-input migration gate; did not claim that multi-assembly
  rendering is already implemented or change the existing site into per-DLL sections.
  Continued the Web split audit against independently compiled dependencies.
- **Observed outcome:** Web is blocked by private shared-deadline operations crossing
  Networking's boundary. No Web artifact is published; Runtime's dependency direction
  remains unchanged. A typed supported deadline contract is the next design/implementation
  slice; raw runtime-service integers are not promoted to public APIs by this audit.
- [Reference direction](../api-docs/README.md#one-class-library-reference-across-assemblies),
  [Web boundary evidence](experiments/extended-cli-metadata/web-boundary-2026-10-07.md).

## 2026-10-07: Project artifacts, documentation bundles and precise source links

- **Author:** Directed that Networking and other libraries be their own projects,
  referencing Runtime and Platform integration projects, with their outputs shipped.
  Clarified that RavenDoc should define a bundle of assemblies shown as one structure.
  Asked for universal GitHub source links to the specific type/member file and location,
  comparable to .NET support; described the present solution as incomplete.
- **Assistant proposal:** Preserve declaration origins independently of documentation
  grouping, target representation and where inherited/extension members are displayed.
  Use repository revision, source path and declaration span, not namespace guesses.
- **Action:** Recorded the project gate and bundle/source-link requirements, compared
  Source Link and Portable PDB inputs, and proposed investigating a declaration map
  where standard debug information lacks exact declaration spans.
- **Outcome/open:** Requirements documented; no project graph, bundle implementation,
  new metadata encoding or complete source-link support is claimed. The ongoing Web
  deadline slice remains unverified: separate Web compilation succeeded after an import
  correction, but the new async consumer currently rejects its Task<()> entry signature.
- [API requirements](../api-docs/README.md#planned-bundles-and-declaration-source-links),
  [project direction](experiments/extended-cli-metadata/package-boundaries-2026-10-07.md#real-project-graph-author-direction-2026-10-07).

## 2026-10-07: Bootstrap and release scope reaffirmed

- **Author:** Reaffirmed bootstrapping, full NeoCLR support and an acceptable developer
  experience for the next release, explicitly including the website.
- **Assistant response:** Kept executable library and project blockers first, with
  documentation bundles/source links tied to release usability rather than a separate
  reprioritization. No declaration that the release is ready.
- **Action/outcome:** Typed shared network deadlines remove Web's private dependency on
  Networking internals. Five consumers compile against emitted libraries only and execute;
  header-phase loopback cancellation closes cancelled sockets while independent work
  completes. API reference and website development content are updated in this slice.
- **Open:** Real native project references (currently explicitly rejected by workspace
  tests), Platform integration projects, build/LSP ownership catalogs, final artifacts
  and release qualification. A new async fixture exposed a Task<()> return-conversion
  gap; its explicit Task<int> form passes. This gap remains recorded, not fixed here.

## 2026-10-07: Bootstrap branch merge gate

- **Author:** Directed that codex/native-system-bootstrap be merged into neoCLR main
  when bootstrap is ready, with continued development from there.
- **Assistant response/action:** Recorded this as authorization conditional on bootstrap
  qualification. Both HTTP cancellation cases now pass, but a fresh Web build exposed
  intermittent System.Void metadata encoding failure; an identical retry succeeds.
  Preserve both outcomes and investigate before claiming a reliable build.
- **Open:** No merge performed. Reliable build, project/editor and artifact qualification
  remain necessary; successful selected consumers alone do not close that gate.

## 2026-10-07: Release-task ownership of general Raven fixes

- **Author:** Directed that the general Raven issue be handled by task
  01a11579-77d7-7500-8561-950c20b1f6bc, which is working on a release.
- **Assistant action:** Sent that task the main-based codex/fix-async-unit-return branch,
  complete patch, baseline failures and focused test results. Left its remaining main
  validation, changelog/commit and release integration with that task; did not merge main.
- **Separate bootstrap outcome:** Raven e141006f3 fixes exact bootstrap Void namespace
  lookup. Both forced-lookup regressions failed before and all 29 focused tests pass
  after; six fresh Web compilations succeed. Native consumer evidence is recorded in
  the [bootstrap unit report](experiments/extended-cli-metadata/unit-bootstrap-2026-10-07.md).
- **Open:** The handoff does not establish general-fix completion or a release. NeoCLR
  continues native execution qualification and project/editor/artifact bootstrapping.


## 2026-10-07: Investigate repeated editor hover delays

- **Author:** Reported VS Code hovers repeatedly remaining on “Loading...” and
  identified one of the Main.rvn files opened in the previous session.
- **Assistant:** Extended the real extension-host acceptance to record provider
  request latency, repeated broad-sample hovers and requests after unsaved edits.
  Kept this investigation alongside split class-library editor qualification.
- **Observed:** The initial split run passed 24 checks. Warm Main.rvn requests took
  3–13 ms, while some startup/reload requests took up to 2.749 seconds; the server
  hover handler's maximum was 86.2 ms. This suggests waiting outside that handler,
  but does not identify the cause of the author's persistent delay.
- **Open:** Reproduce the original persistent workload and isolate any scheduling,
  reload or client delay before claiming a fix. [Evidence and scope](experiments/extended-cli-metadata/native-split-editor-2026-10-07.md).


## 2026-10-07: Native RavenDoc provenance and providers

- **Author:** Pointed out the API reference's “Assembly: NeoCLR.CoreProbe.dll” label
  and requested the actual Runtime/Data/Networking owners. Proposed native metadata
  support through a provider structure and standardized member-presentation models,
  allowing the compiler API as an intermediary.
- **Assistant inspection:** RavenDoc currently loads CLI references and adjacent/framework
  DLLs, while the renderer already accepts compiler symbols and multiple assemblies.
- **Assistant proposal:** Add an explicit native input provider over the existing catalog
  and symbol adapter first; retain actual declaration identities and extract a stable
  documentation model incrementally. Do not relabel bridge ownership or add a resolver.
- **Status:** Direction recorded; provider implementation and website migration remain
  open. The active bootstrap-input validation continues independently.
  [Plan and evidence](experiments/extended-cli-metadata/native-documentation-provider-2026-10-07.md).


## 2026-10-07: Defer RavenDoc architecture for release focus

- **Author:** Asked to delay fixing RavenDoc's bridge assembly display, considering
  rewriting RavenDoc in Raven in the future. Directed attention to the actual release
  and changes that add value to it.
- **Assistant:** Deferred native documentation providers and standardized-model work,
  keeping the previous proposal as future context. Proposed focusing on reproducible
  builds, working samples, installation and editor usability while leaving bridge
  provenance explicit.
- **Action:** Updated the authoritative roadmap and API maintenance direction. The
  possible rewrite is recorded as exploration, not an approved implementation project.
  No runtime/compiler behavior or release qualification result changes in this update.


## 2026-10-07: Native pattern-binding samples

- **Author:** While the next AOT Result slice was underway, asked: “Can you also test
  samples using the let <pattern> [else] and if let <pattern>”.
- **Assistant action:** Added an ordinary Some/None pattern app covering both outcomes
  of let-else and if-let, and used both forms in a local generic Result app. Both run
  through Raven native metadata, the interpreter and isolated ARM64 executables.
- **Observed limit:** Plain positional let deconstruction reaches an unsupported
  BoundPatternAssignmentExpression in the pinned Raven native emitter. Retained a
  negative source probe and checked its diagnostic/no-output behavior, rather than
  claiming this form works. No shared Raven compiler change was made in this slice.
- **Evidence/open work:** [Sources, reports and specialization limits](experiments/aot-values/README.md#generic-result-and-pattern-bindings-2026-10-07).
  Raven native deconstruction, actual library Result dependencies, multiple closed
  instantiations and native input/lifetimes remain open.

## 2026-10-07: Consistent interpreter and native faults

- **Author:** Clarified that faults should render a message and stack trace like .NET
  exceptions; distinguished user faults from runtime faults by code, with standardized
  runtime messages. Suggested a host-visible fault object and printing utility, then
  emphasized the same experience under interpretation and native compilation.
- **Assistant:** Replaced the in-progress status-only failure binding with a shared host
  presentation contract and caller-owned native diagnostic capture. Proposed exit code 1
  for standalone unhandled faults, preserving ordinary program exit codes and allowing
  embedding hosts to decide their own policy.
- **Author:** Replied “Ok. Sounds right” to that exit-code convention and directed further
  work after the current slice.
- **Actions/evidence:** Added a shared code-message catalog and borrowed interpreter
  diagnostic view, native message/frame capture, a C printing utility, and explicit
  failure-service bindings. Original managed wrappers remain compiled. Focused parity
  tests cover code/message/frame order, native-service frame omission, first-fault rules
  and a common 64-frame truncation limit. See [implementation and evidence](experiments/aot-fault-details/README.md).
- **Open:** Legacy raw diagnostic fields/display remain compatible; CLI presentation
  migration follows separately. Native trace source locations and richer identities,
  input services and a stable native hosting ABI remain future work. No catch/unwinding
  or guest exception hierarchy was requested or introduced.

- **Follow-through:** The next committed slice routes interpreter CLI execution faults
  through the common presentation, with exit 1 and unchanged successful exit values.
  Loader/verifier detail and legacy host/debugger Display remain compatible. The Raven
  failure reproduction now checks exact interpreter/native diagnostic text when run
  against the updated CLI, in addition to message and frame parity.

## 2026-10-08 — Continue toward native Console support

**Author direction:** “Continue towards the immediate goal of supporting Console class
and don’t stop”. This follows the AOT union, input and shared fault diagnostic slices.
**Assistant action:** continued on main with explicit native byte-input binding,
retaining the ordinary Raven wrapper and committing bounded slices. The native input
[experiment](experiments/aot-console/README.md) records byte/EOF/error tests and the
remaining Console dependencies. The following slice adds value-profile WriteLine
and an interactive input/output consumer. The assistant then selected a bounded,
host-owned invocation text region for Int32 formatting: preserve strings across calls
and faults, reclaim together at the next invocation, and reject escaping reference
storage. This is an experimental lifetime foundation, not an author decision to replace
tracing with regions or reference counting. A subsequent explicit reference-arena slice uses the same bounded lifetime for
nongeneric class objects, preserving aliases and reclaiming cycles at reset. The
assistant tested reference copies, null faults and a Raven class/Console consumer.
General collection, interface dispatch, arrays and stream support remain open. This Console direction does
not establish a new permanent roadmap priority outside the author-directed AOT work.

## 2026-10-08 — Let the HTTP API expose native execution requirements

- **Author:** “The idea is that by working towards the HTTP API we discover what we
  need to run it. Even if it will require integrating a garbage collector”. This
  clarifies the purpose of the progression through Console, request-line parsing
  and the existing RoutePattern library.
- **Assistant response:** Follow the actual HTTP API's dependencies, including GC
  if required for correct execution. The bounded invocation arena is a temporary
  sample mechanism, not a choice of memory management for a long-running server.
- **Actions:** Compiled a direct RoutePattern.Parse/Match/GetInt32 consumer against
  the supplied Runtime/Web/Networking/Data libraries. Its first AOT admission failure
  was `ArrayRef(String)` during specialization. Implemented a bounded String-array
  storage foundation and added a repeatable interpreter/admission probe. See the
  [native Console experiment](experiments/aot-console/README.md).
- **Open:** Full native RoutePattern/HTTP execution, native roots and tracing,
  reclamation policy and network ownership remain work to discover and implement.
  The author did not select a collector algorithm or request replacing the existing
  interpreter collector. Small consumers remain validation steps, not substitutes
  for getting the existing HTTP API running.

- **Observed outcome:** the storage slice was enough for the small real-library
  Parse/Match/GetInt32 consumer to run standalone with matching output/faults and only
  libSystem dynamically linked ([evidence](experiments/aot-console/route-pattern-validation.json)).
  This qualifies that path, not the complete HTTP API; broader route outcomes and
  repeated-request lifetime checks remain next steps.

- **Follow-through:** The assistant expanded the real-library consumer to eight route
  outcomes and a capture retained across requests. Native arena usage reaches 117,507
  bytes at 128 requests and faults under a fixed 64 KiB budget; interpreter diagnostics
  show eight pressure collections preserving the retained references. This establishes
  a concrete reclamation requirement, not a throughput comparison. The assistant records
  typed roots/allocation descriptors followed by nonmoving tracing as provisional next
  slices; no native collector is implemented here. See the
  [lifetime evidence](experiments/aot-console/route-lifetime-validation.json) and
  [integration plan](native-execution-investigation.md#http-driven-native-reclamation-requirement-2026-10-08).


## 2026-10-08 — Continue through basic native GC

- **Author:** “Continue until you have basic GC support even for native compilation.”
- **Assistant response:** Continue across implementation slices rather than stop at
  another diagnostic root checkpoint. Use the repeated routing workload within a fixed
  budget, retained patterns/captures and clean live-set exhaustion as acceptance evidence.
- **Provisional implementation choice:** nonmoving mark-and-sweep with collection only
  at compiler-published operation boundaries. Conservative object-word candidates trade
  possible extra retention for a smaller first implementation; native allocation services
  do not collect while internal temporary data is live. This does not replace interpreter
  GC or select the eventual production collector.
- **Actions so far:** add an in-buffer descriptor/mark-sweep kernel and sanitized contract
  test. Generated-code integration and fixed-budget acceptance remain open in this slice.
  See [design and evidence](experiments/aot-console/native-gc.md).

- **Follow-through:** The assistant committed the collector kernel, then connected an
  explicit native-GC mode to compiler-published pre-operation boundaries. The generated
  ARM64 lifetime fixture passes with a 2 KiB heap, including interior-only ownership,
  repeated discarded cycles, erased returns and fault retention. The Raven route sample
  completes 1,024 requests in 64 KiB and preserves interpreter output/fault behavior.
  [Recorded evidence](experiments/aot-console/route-native-gc-validation.json) distinguishes
  this completed basic GC from remaining precise maps, pressure scheduling, host handles
  and full HTTP server work. AddressSanitizer was unavailable on this host due to startup
  deadlock; undefined-behavior/bounds sanitizers and executable guards passed.


## 2026-10-08 — Evolve the native POC with measurements and interpreter feedback

- **Author:** Continue toward an improvable POC, benchmark when relevant during native
  compilation implementation, and examine improvements portable to interpreter mode.
- **Assistant proposal and action:** Compare existing traversal/scheduling policies, then
  replace native repeated heap passes with an intrusive worklist. The interpreter already
  uses a worklist, so this is a transfer from interpreter to native. No interpreter or
  Raven compiler change is required by this slice.
- **Measured outcome:** Seven paired samples after warm-ups show about 2.9× faster
  collection of an adverse reverse-linked graph; real Raven routing is effectively
  unchanged. Counts/output/budget checks agree. Focused kernel, compiled lifetime/fault
  and interface-dispatch tests pass. See [method and evidence](experiments/aot-console/native-gc.md#measured-worklist-slice--2026-10-08).
- **Open:** Safe pressure scheduling, precise object tracing and full HTTP-server
  qualification. Interpreter scratch-buffer reuse is a measurement candidate, not an
  implemented improvement. Benchmark relevant performance changes, not every routine API
  change; eventual .NET comparisons need matched workloads and explicit timing scopes.


## 2026-10-08 — Benchmarkable HTTP app and Native compilation feature page

- **Author:** Work toward a web server app to benchmark against other platforms and
  present results on a “Native compilation” feature page with a homepage box.
- **Clarifications:** Compare interpreted and native modes first, then equivalents on
  other platforms; check in samples. List development caveats, and mark even a POC
  included in the next release as work in progress. Continue toward the HTTP test app
  after this slice.
- **Assistant action:** Add a same-artifact routing comparison harness, checked-in Raven
  workload and ASP.NET Core greeting comparison candidate. Compile the existing Raven
  server as an admission driver; its native selection rejects unsupported generic
  arguments. This is a component benchmark and server dependency checkpoint, not a
  native server result. Add the requested feature page, homepage box and navigation.
- **Comparison contract:** qualify the same Raven server in both modes, then match other
  platforms' requests/responses and protocol settings. Separate startup, steady-state
  latency/throughput, errors, memory and deployment size. Preserve raw samples, versions
  and caveats. A C-hosted Raven handler would require a separate mixed-stack label.
- **Validation:** Five measured routing pairs produced 29.447 s interpreted / 0.392 s
  native median process times with matching output, not server throughput. The .NET
  candidate passes a successful greeting exchange check; it has no performance result.
  [Recorded inputs and samples](../benchmarks/native-web/routing-validation.json).
- **Open:** Native server support and full HTTP measurements; no cross-platform speed
  ranking or production readiness is claimed. See the [workbench](../benchmarks/native-web/README.md).


- **HTTP follow-through:** Isolating listener creation exposed the missing native
  UnpackValue<long> path. Add exact Int64/UInt64 erased transport, generic scalar
  specialization and equality, with ten focused tests and a Raven wide-value consumer.
  The listener passes interpreted execution and now reaches unbound socket services;
  the full server independently reaches function-valued generic arguments. The author
  directs “Continue and don't stop”; continue implementation slices toward the HTTP app.

## 2026-10-08 — StringBuilder question during native HTTP work

- **Author question:** “Does neoCLR also need a StringBuilder class?”
- **Assistant assessment:** A small UTF-8 builder is a useful library candidate for
  incremental text construction, including the existing HTTP header concatenation loop.
  It is not a prerequisite for the current native-execution foundation. Response-body
  streaming remains a separate need; a builder should not require buffering entire bodies.
- **Existing evidence reviewed:** The [ReportTextBuilder experiment](design/text-abstraction.md)
  has explicit byte quotas and immutable snapshots but was slower than concatenation in
  its interpreter report workload. The existing decision to defer public promotion remains.
  No claim is made that its earlier timings predict the native HTTP workload.
- **Assistant proposal:** Revisit a minimal Append/AppendLine/Clear/materialization API
  with explicit UTF-8 byte units and snapshot guarantees when the HTTP consumer justifies
  it. Compare an append buffer or one-pass join with existing concatenation, measuring both
  execution modes. ToString naming was suggested, not adopted; the prototype uses Build.
- **Action and open status:** Inspect current source and research; record this exchange.
  No public builder was implemented or reprioritized by this question, and no author
  decision approving its API or implementation is available. Continue the authorized AOT work.

## 2026-10-08 — Author-selected builder side quest and separator joins

- **Author direction:** “We may implement it now as a side quest as it is related.”
  This explicitly revisits the prior deferral while retaining the native HTTP goal.
- **Assistant proposal:** A small UTF-8 builder with append, line append, clear/reuse,
  immutable snapshots and native/interpreter validation, followed by measurement.
- **Author API choice:** “Append returns the builder for chaining, with a fault when
  its limit is exceeded”. The author then asked whether Result<StringBuilder, ...>
  with propagation was worthwhile. The assistant proposed retaining fluent Append
  and considering TryAppend only for a consumer needing recoverable quota errors.
  The author directed continuation; no TryAppend API was requested or implemented.
- **Additional author direction:** “We also need a String.Join that can handle
  separators”. The assistant included the string-array overload and a shared joining
  service, preserving separators around empty elements.
- **Actions:** Implement ordinary Raven storage and snapshot caching, a shared
  interpreter/native joining service, and sealed-owner native call admission needed
  for ToString. Keep development API/reference and integration documentation aligned.
  See [design and validation](design/string-building.md) for contracts and evidence.
- **Open:** Broad formatter/collection overloads, mutable native-buffer ownership,
  larger quotas and replacing actual HTTP construction remain later work. This side
  quest does not establish that the HTTP server runs natively or change its priority.

## 2026-10-08 — Future HTTP method abstraction

- **Author:** Suggests that neoCLR might need an equivalent to .NET HttpMethod later,
  linking its API documentation.
- **Assistant proposal:** Consider an immutable token with standard instances and
  validated extension methods. Keep representation, construction errors and migration
  open; explicitly compare .NET's case-insensitive equality with HTTP's case-sensitive
  tokens. Method representation must not imply unsupported protocol capabilities.
- **Action:** Inspect current string Method storage and record the
  [proposal and primary sources](http-client-design.md#future-httpmethod-type--proposal-2026-10-08).
  No API is implemented or approved by this exchange. Native HTTP execution remains
  the active goal; host GC root work is still in progress.

## 2026-10-08 — Reassess runtime foundations during native HTTP work

- **Author direction:** Re-evaluate approaches across the runtime, including Function
  types and delegate-like objects, because runtime async suspension and green threads
  will probably be introduced later. Continue toward the HttpServer POC.
- **Assistant response:** Keep callable contracts separate from suspended activations,
  I/O completion and scheduling. Treat native descriptor layout, thread-local roots and
  quiescent callback dispatch as provisional implementation choices.
- **Action:** Inspect current FunctionObject, AOT descriptors, TaskQueue and prior
  scheduling design; record a [cross-runtime review](runtime-scheduling-design.md#cross-runtime-reassessment--author-direction-2026-10-08)
  with .NET and Java comparisons, migration boundaries and validation questions.
- **Open:** No green-thread stack model, public scheduler API, callable semantic change
  or implementation schedule is selected. Native callback dispatch is being developed;
  this discussion is not evidence of completed runtime suspension.

- **Author follow-up:** “We might change from a TaskQueue to a Scheduler” and “the
  Runtime eventually schedules green threads”, recalling earlier scheduler discussion.
- **Assistant response:** Agree that a runtime-owned Scheduler is the direction to
  investigate; today's native TaskQueue/Drain integration is a compatibility layer.
  Keep callable identity distinct from runnable/suspended activations, roots and
  resumption. Do not infer that TaskQueue will remain the future scheduling contract.
- **Action/outcome:** Record the [clarification](runtime-scheduling-design.md#scheduler-clarification--author-follow-up-2026-10-08).
  Guarded native recursion and unit entry adaptation are committed; host queue-drain
  integration is under validation. Green threads remain unimplemented by this work.

- **Author clarification:** Work motivated by native compilation should benefit
  interpreted mode and eventually JIT: “So cross-cutting runtime services”.
- **Assistant response/action:** Record shared service contracts with backend adapters
  as the design direction. Scheduling/activation lifecycle, I/O cancellation/completion,
  roots and logical fault reporting are the cross-cutting boundaries; native stack
  probes and interpreter frame storage remain implementation mechanisms. Require
  applicable fixes and cross-mode tests to inform interpreter work, without claiming
  an unmeasured performance improvement or an implemented JIT.


## 2026-10-08 — First matched native HTTP execution

- **Context:** The author's continuing objective is an AOT HttpServer app, followed by
  useful interpreter/native and external-platform benchmarks, with WIP caveats.
- **Assistant implementation outcome:** The existing Raven Server source now compiles
  through neoCLR metadata/CIL into native ARM64 and serves a real loopback request.
  The same artifact subsequently passes four cases in interpreter, sanitized native
  and standalone native modes: greeting, fragmented input, duplicate Content-Length
  and handler rejection. Response bytes/application output match. Native scope/frame
  cleanup checks pass and the standalone image links only libSystem.
- **Evidence:** [Reproduction and recorded checks](../benchmarks/native-web/README.md#first-native-http-execution).
- **Next proposed driver:** Repeated requests in one process, GC/lifetime pressure and
  matched load measurement. These one-request results are correctness evidence, not
  throughput results or a production-readiness claim. Runtime Scheduler work remains
  guided by the author's cross-cutting-services clarification above.


## 2026-10-08 — Native compilation as the next-release teaser

- **Author direction:** Have something to show in the next release as a teaser:
  “Native compilation is what brings value to the platform we are building”.
- **Assistant response:** Focus on the checked-in Raven HTTP app producing a standalone
  ARM64 executable, a reproducible demo and explicit WIP limits. Build on the first
  working request with repeated requests, cleanup under sustained use and an honest
  interpreter/native comparison for the Native compilation page. Keep future Scheduler
  and cross-cutting service design as foundations, not prerequisites for the teaser.
- **Current evidence/open work:** The four-case same-artifact HTTP consumer passes;
  repeated-request validation and load measurements are being developed. No release
  version/date or production guarantee is inferred from this direction.

- **Author release clarification:** The target release has “full Raven compiler-target
  bootstrap (no .NETbridge)” through neoCLR metadata, interpreted or compiled natively
  by neoCLR.
- **Assistant response/action:** Record that as a release requirement, distinguishing
  the current HTTP execution proof from a qualified release build path. Existing
  preparation documentation still permits a generated CLI Core bootstrap and current
  HTTP compiler commands use Core.dll; this is remaining target qualification work.
  Treat compiler hosting/self-hosting as separate from the stated compiler-target
  requirement, without assuming an additional compiler-host migration.


## 2026-10-08 — Sample-driven native baseline and release priorities

- **Author request:** Run the samples in interpreted and native modes, assess missing
  native coverage and identify release priorities.
- **Assistant response/action:** Inventory 98 Raven API samples and six additional
  async/JSON consumers through emission, interpreter execution, AOT, linking and native
  execution. Record first blockers separately from runtime failures and reuse explicit
  HTTP/GC evidence. The assessment and raw results are linked from the roadmap.
- **Author clarification:** “Our goal is having a stable baseline” does not mean every
  sample works in all modes; neoCLR will continue changing.
- **Assistant response:** Define supported, tested scenarios and explicit exclusions.
  A survey failure is a prioritization signal, not automatically a release blocker.
  Preserve the explicit no-.NET-bridge target bootstrap requirement and existing
  tooling/platform gates; do not silently expand these to universal AOT parity.
- **Author proposal:** Future AOT reflection could use an IL-free metadata file beside
  the binary, converging with metadata as an interop mechanism.
- **Assistant response/action:** Record that [future exploration](native-execution-investigation.md#il-free-metadata-for-aot-reflection-and-interop-author-proposal-2026-10-08).
  Distinguish metadata inspection from retained native invocation, with pairing,
  stable ABI and trimming questions still open. No implementation is claimed.
- **Author subsequent direction:** Fix the easiest and urgent native compilation
  issues. **Assistant proposal:** Assess primitive arrays first because they recur
  as the first AOT blocker in otherwise-interpretable collections/async consumers;
  confirm layout and lifetime requirements before admission. Broader reflection and
  green-thread changes remain future work.

## 2026-10-08 — Unprefixed runtime interfaces

**Author direction:** ensure none of neoCLR's interfaces use the `I` naming
convention, including Raven's mappings for the neoCLR target profile.
**Assistant audit:** source interfaces already use names such as `Iterable`,
`Disposable` and `HttpHandler`; `IAsyncStateMachine` and `ITaskAwaiter` remained.
**Implemented async slice:** coordinate `AsyncStateMachine` / `TaskAwaiter`
across runtime source, compiler target mapping and the temporary CLI bridge.
Preserve .NET target names and internal compiler special-type identifiers.
This changes metadata identities and requires rebuilding matched applications,
runtime libraries and compiler references. The native callback dispatch fix remains
a separate slice; naming does not itself add async scheduling support.

**Validation/outcome:** Raven mapping commit `185d32f06` passes 37 profile/async
tests. A fresh native library bundle and three real async consumers pass. Native
interface callbacks pass sanitized receiver retention, null and fault parity tests;
entry-task draining remains unsupported. Legacy Tasks regeneration exposed a general
source/metadata cache bug; withholding provisional declaration-time results fixes
the reproduction and 16 lookup tests pass. The Tasks slice and API snapshot were
refreshed. The wider neoIL audit then found `IUnion`; its migration remains next.
The full legacy snapshot check separately reports an existing missing StringBuilder
input in Tuple, not an async naming failure.

**Follow-up:** the general lookup fix also passes all 16 focused tests on Raven
main and is committed there as `d0a115dcf`. The assistant selects `UnionValue` for
the legacy boxed-active-case protocol, avoiding the `Union` attribute shorthand.
This is an assistant implementation choice within the author's naming direction,
not a change to native union storage or an author-selected reflection API.

**Union outcome:** all 27 affected union slices were regenerated. The constructor
field-initialization and typed-null-output gaps in the legacy importer were fixed.
The executable Option/Result/TaskOutcome and time/timezone samples pass. The old
union sample's preservation assertions were stale: Raven commit `0d261d8be`
explicitly initialized failed TryGetValue outputs, documented and tested since
2026-10-03. The assistant aligned the legacy fragments, assertions and current API
docs with that shared contract, recording the compatibility change. This does not
introduce a new target-specific output policy. A reverse control on Raven main
fails both new source-lookup regressions when the cache fix is removed; restoring
it passes all 16 lookup tests.

**Final protocol validation:** Raven `be58723fb` retains renamed async interfaces
in target metadata during CLI emission, including the explicit legacy probe-core
profile. All 16 focused profile/emission/.NET controls pass and the CLI async
sample executes suspension and result `42`. The native-enabled compiler rebuild
succeeds. No runtime Raven/neoIL interface declaration retains the `I` prefix.


## 2026-10-08 — Cross-mode parity and CLI instruction semantics

**Author:** Reaffirmed that interpreted mode and native compilation should yield the
same behavior. After directing continued work on array views, asked for an assessment
of the next fixes. Then clarified: “Another requirement is that we should follow .NET
semantics in IL unless we diverge. So most if not all instructions right now should
work the same way as in .NET.”

**Assistant investigation:** Native class-reference array views exposed an existing
interpreter behavior: `isinst` with a different mutable-array element type faulted.
The first implementation preserved that behavior for parity. On the author's CLI
clarification, the assistant checked .NET's instruction documentation and emitted a
DynamicMethod control. The recorded September array decision requires invariance,
not a fault for a failed type test. The initial parity-only approach was revised.

**Action:** Correct the shared verifier/interpreter and native slice so incompatible
array type tests return null while explicit casts remain subject to the documented
invariance rule. The [contract and validation](array-variance.md#type-tests-retain-cli-semantics-2026-10-08)
distinguish this correction from a new divergence. Keep the existing array-view work
and sample-driven release assessment in scope; no full IL audit or universal parity
claim follows from these focused checks.

## 2026-10-08 — Math constants alongside native lookup

- **Author:** asks whether the Math namespace provides constants such as Pi, and
  requests adding them while the native lookup work continues.
- **Assistant investigation:** no Math constants are currently declared. Proposes
  Pi, E and Tau as Double constants with .NET-equivalent rounded values. A native
  source build exposes missing namespace-constant declaration support; this is a
  separate metadata/emitter requirement, not merely adding literals to a source file.
- **Action/outcome:** isolates the lookup collision to neoCLR's primitive bootstrap
  including a competing System.Math type. Rebuilds the explicit source-runtime
  bootstrap without that placeholder; the unchanged string consumer passes VM,
  sanitized native and standalone runs. No Raven lookup semantics were changed.
- **Open:** implement and validate namespace constants through separate compilation,
  both execution modes and the API reference. The proposed constants are not yet
  shipped by this lookup slice. [Lookup evidence](../benchmarks/native-web/math-lookup-validation.json).

- **Follow-up implementation:** adds Pi/E/Tau as finite Double namespace constants,
  with native metadata snapshots, compiler inlining and matching API documentation.
  The separate consumer first exposed that qualified namespace lookup collected only
  direct native methods, not constants; Raven now includes direct constant symbols
  and permits their receiverless binding. Ordinary CLI namespace carriers remain.
  Native metadata/consumer evidence is linked from [Math](math.md#assembly-level-constants-development-2026-10-08).

- **Validation boundary:** the constant consumer passes the interpreter, but AOT
  rejects its first Double literal. The assistant records this gap explicitly and
  identifies Double instruction lowering as the next native task, rather than
  claiming parity or disguising constants behind a different API. The current
  constant slice does not add native floating execution or floating Math services.

- **Recorded outcome:** Raven implementation committed as `c3dcf4f5f` on its native
  integration branch. Six Raven controls, ten runtime metadata checks, metadata
  round-trip/rejection checks, six emission/visibility controls and the .NET literal-bit
  reference check pass. The Math constant sample is deliberately recorded as
  interpreter-passed/AOT-blocked, not universal execution parity.

## 2026-10-08 — Assembly-level members include qualified namespaces

- **Author correction:** “We shouldn't call them namespace functions or namespace
  constants. They are assembly-level members”. Clarifies that types, functions and
  constants can be assembly-level members, and that their names include namespaces.
- **Assistant response/action:** adopts assembly ownership with namespace-qualified
  names, updates current terminology and the new constant API/metadata names, and
  keeps namespace lookup/import behavior. Historical conversation entries and
  completed validation artifacts retain their original wording as evidence.
- **Implementation:** the unreleased host API is AssemblyConstantDefinition,
  AssemblyBuilder.AddConstant and ModuleDefinition.Constants; metadata uses
  assemblies[].constants. Matching compiler/runtime bundles must be regenerated.
  [Naming contract](design/extended-cli-metadata.md#assembly-level-members-and-qualified-names-2026-10-08).

- **Author follow-up:** metadata itself and the libraries for reading/writing it must
  reflect this structure.
- **Assistant implementation:** adds AssemblyDefinition.GetMembers for writer graphs
  and reader snapshots, and AssemblyInfo.GetMembers for the metadata facade. The
  typed member views preserve kind, declaring assembly/module and qualified name;
  constants attach to one graph. Round-trip controls cover types, overloaded functions
  and constants together, equal names in distinct assemblies, rejected reattachment,
  and exclusion of nested/type-owned declarations from the assembly-level view.
  This is metadata tooling, not new guest reflection or native Double execution.

- **Validation:** authored and native-container round trips, reader facade ownership,
  overloads and rejected reattachment pass. Six compiler emission/visibility controls
  pass with the renamed contract; the Math consumer remains interpreter-passed and
  AOT-blocked on Double instructions. The API snapshot is current.
  [Compiler evidence](experiments/extended-cli-metadata/assembly-constants-validation.json).
- **Integration revision:** Raven adapter and compiler documentation committed as
  `bc3c500e6` on `codex/source-object-metadata-resolution`; neoCLR owns the metadata
  model and renamed manifest contract on main.


## 2026-10-08 — Exploring modules as namespaces

- **Author request:** first record the supplied “First-Class Module System”, Version
  0.2 proposal. The author cautioned that artifact names do not reflect current
  direction, found the structure interesting, and asked about building around modules
  rather than namespaces. Follow-up: “The module becomes the namespace, technically”.
- **Further author direction:** “RuntimeContext will have to be adapted around modules
  too. Since Assembly is not prominent anymore.” This expands the proposed runtime
  API focus; exact loading, identity and packaging contracts remain unresolved.
- **Assistant interpretation:** explore a named metadata declaration container whose
  path supplies qualification and whose identity supplies ownership and visibility.
  Distinguish this from physical CLI modules; assess same-name modules across
  assemblies, access checks, native metadata and migration of current member APIs.
  Evaluate direct module discovery through RuntimeContext, with assembly packaging
  potentially retained underneath rather than required for ordinary discovery.
- **Actions:** preserved the supplied text verbatim in the
  [proposal archive](proposals/first-class-module-system.md), with its supplied ending
  at section 17 Phase 1 documented in the index. Added a separate
  [assessment](design/module-system-assessment.md) comparing the CLI baseline, Rust
  module privacy and alternatives; linked the discussion from the website proposals.
- **Status and open questions:** the earlier assembly-level-member direction remains
  the implemented model. This discussion explores evolving it; no module encoding,
  Raven syntax, artifact names, implementation experiment or release reprioritization
  has been adopted. The assistant's bounded experiment is a suggestion, not performed
  work. Cross-assembly imports, identity migration and runtime enforcement remain open.


## 2026-10-08 — Native core bootstrap and RavenDoc provenance

- **Author direction:** after recording the module proposal, resume main development;
  subsequently continue. During the native-core work, the author observed: “I imagine
  getting rid of the importer will also affect RavenDoc so it doesn't mention Probe.dll.”
- **Assistant response:** retire the temporary CLI projection input path while retaining
  native metadata reading. RavenDoc should use actual library ownership; changing
  labels alone would misrepresent its inputs. Its separate loader needs migration.
- **Work:** fixed self-owned native core production in `19cc3558`, then introduced an
  explicit Raven compiler API native-only semantic mode. A bounded integer consumer
  emits and runs interpreted and ARM64 native with result 42, using native metadata
  inputs and an empty native System seed. [Evidence](experiments/native-core-bootstrap/README.md).
- **Remaining:** native catalog/driver/project integration, a complete source-built core,
  Windows/editor qualification and RavenDoc's native provider. The prior RavenDoc
  release deferral remains history; the new observation is recorded as the intended
  migration outcome, not assumed approval for a rewrite or changed release scope.


## 2026-10-08 — Explicit RavenDoc loader priority and API experience

- **Author direction:** “Continue until you have solved the RavenDoc loader issue”,
  followed by “We still want the full experience in RavenDoc. It shouldn't have
  degraded when it comes to API reference.” This explicitly brings loader work
  forward from the earlier deferral; it does not request a RavenDoc rewrite.
- **Assistant response:** finish the bounded native-core driver slice, then reuse the
  native catalog in RavenDoc's existing rendering pipeline. Check documentation,
  ownership, cross-library links and grouped navigation, then broaden member coverage
  rather than treating successful loading alone as API-reference parity.
- **Actions:** driver selection committed in Raven `47cf49a9b`, with interpreted/AOT
  execution evidence in neoCLR `ea2de81d`. Native-loader implementation and expanded
  qualification now pass, including a fix for missing native namespace comments.
  The existing 63 .NET generation/site controls pass. Raven implementation: `962f178a0`;
  [native reference evidence](experiments/native-core-bootstrap/ravendoc-validation.json).
  The aggregate website snapshot still uses its CLI
  bridge and must not have ownership labels rewritten without changing inputs.


## 2026-10-09 — Replace aggregate documentation ownership

- **Author:** “We want to move away from: Assembly: NeoCLR.CoreProbe.dll”, while
  viewing BooleanParseError in the complete local API reference.
- **Assistant response:** use actual declaring-library symbols instead of relabeling
  aggregate input; preserve the full API experience during migration.
- **Actions/outcome:** the historical split native bundle exposed a null declaring-type
  assumption for assembly-level functions. Raven fixes now render these members and
  preserve their comments and constant comments. The preview contains 1,632 pages;
  BooleanParseError and CancellationTokenSource identify System.Runtime.dll. All 68
  focused .NET controls and reduced native documentation checks pass.
- **Remaining:** this audit explicitly uses the bundle's CLI primitive bootstrap.
  Native core production and semantic member/comment/route parity remain open;
  the complete website has not been replaced by the smaller audit.
  [Evidence](experiments/native-library-documentation-audit/README.md).


## 2026-10-09 — Native reference belongs in the neoCLR website

- **Author correction:** the standalone RavenDoc preview did not look like the
  neoCLR website. The author requested the changes on the neoCLR page and clarified:
  “The updated website is part of the future release”.
- **Assistant action:** added an integrated local preview using neoCLR's site theme,
  navigation and search, linked from existing reference pages. Preserve the complete
  reference while qualifying native metadata coverage. A private copy normalizes
  historical XML comments mislabeled as Markdown; producer correction remains open.
- **Scope:** this is local development visibility for the future release, not a
  publication or completion of the no-bridge/documentation-coverage gates.

- **Further author corrections:** verify broken Object links, restore XML doc text,
  emit correct IDs for neoCLR assembly-level members, include these members in the
  sidebar, remove CLR-container notes from native pages, use normal API URLs, and
  check the website before committing.
- **Actions:** fixed target-specific ID generation, selected-library enumeration
  (bootstrap deduplication intermittently hid Object), native member navigation,
  and assembly-function overload grouping. Recovered 1,482 reviewed XML entries
  against real native declarations for the historical bundle preview. Updated
  canonical local routes with compatibility redirects for the earlier preview.
- **Remaining release work:** historical bundle lacks some current declarations,
  including Math constants; 139 legacy pages are retained with explicit notices.
  Regenerate production native metadata/sidecars and qualify coverage before release.

- **Author follow-up:** also fix constants not showing up. A fresh Runtime artifact
  contains Pi/E/Tau and canonical XML IDs; rebuilding all four native libraries
  brings their pages and navigation into the website. This closes the constants
  gap above. The final integrated validation covers 3,483 pages, 136 retained
  legacy pages, 43 focused compiler/renderer tests and native fixture checks.
- **Build limitation:** the first fresh bundle attempt hit NEOMETA003 in
  HttpContext's constructor. A direct Web retry and a subsequent full bundle build
  passed; the intermittent metadata-type comparison failure is not claimed fixed.

## 2026-10-09 — Group time APIs and isolate the HttpContext fix

- **Author:** requested moving time-related constructs from System into System.Time
  and reconsidering the Time struct's name.
- **Assistant:** selected TimeOfDay because the type represents a time within a day,
  without date or timezone. Proposed moving dates, clocks, durations, instants,
  calendars, zone mappings and their errors together, keeping formatting in
  System.Globalization and keeping LocalDateTime.Time as a property.
- **Actions:** migrated Raven sources, explicit native ownership and temporary CLI
  catalogs, generated affected implementations, updated consumers and API documents.
  [Naming and compatibility](date-time-design.md#systemtime-namespace-2026-10-09-development)
  records the .NET TimeOnly comparison and rebuild requirement.
- **Author follow-up:** “Fix the `HttpContext` metdata error also, but commit separately”.
  The assistant traced it to nested Void arguments with different declaring owners,
  not a verifier tolerance issue. Raven `92a593ff7` normalizes the selected core unit
  to RuntimeUnitContract; neoCLR `5007c251` records that separate integration fix.
  A reduced constructor fails without the fix and both explicit Void/unit spellings
  compile with it; the complete native bundle builds.
- **Validation/outstanding:** the calendar sample matches interpreted/native output.
  Timezone samples compile and execute in the interpreter, while existing native
  timezone service and nonempty record-boxing limits remain. The native API preview
  was checked in the browser for System.Time navigation, TimeOfDay summaries and
  System.Runtime ownership. This does not qualify every time API for AOT or publish
  the website.

### 2026-10-09 — Bound native String work to the release POC

- **Author:** “Keep in mind that we're looking to release a proof of concept. It
  does not have to solve all the problems, just the most urgent ones before we
  release it.” This clarified the ongoing instruction to finish native String.
- **Assistant:** proposed closing the current source-String qualification and
  validation rather than expanding into Object display, intern optimization or
  full core bootstrap. Native String consumers now pass interpreted and ARM64
  execution; remaining limitations are recorded in the
  [POC scope](experiments/native-core-bootstrap/README.md#production-string-poc-2026-10-09).
- **Author:** earlier directed that RavenDoc branch work be integrated after native
  String is done, then explicitly asked to clean up the branches afterwards.
- **Outcome:** native String POC committed as `c12a533d` after its prerequisite
  slices. RavenDoc's five remaining commits were merged into Raven's
  `codex/source-object-metadata-resolution` as `c6d2137e3`; the focused suite passed
  36 tests (baseline 29). Both temporary RavenDoc branches and the clean temporary
  worktree were removed after verifying their work was retained. The pinned
  RavenDoc archive checksum matches its recorded revision, now an ancestor of the
  integration branch. Nothing was pushed or published.

### 2026-10-09 — Publish Raven's shared documentation changes first

- **Author:** requested a Raven website release before starting neoCLR's release
  procedure, including the shared RavenDoc Type extension presentation and checks
  that it works well for Raven targeting .NET.
- **Assistant:** built Raven main in an isolated checkout, checked the actual .NET
  Raven.Core extension pages in desktop/mobile and light/dark modes, and added
  browser regressions. Fixed a stale static-class assertion and theme-test mock,
  plus Copy-button overlap on long mobile API declarations (`c7e6248ee`).
- **Author:** suggested mentioning neoCLR and linking its website while keeping
  .NET as Raven's official target.
- **Assistant:** added a related-project note to Raven's homepage and overview,
  explicitly describing neoCLR integration as experimental (`e56c81084`). Checked
  the link and responsive rendering. The initial publication run was cancelled
  so the note could ship with the documentation update.
- **Publication:** final website workflow
  [37915415706](https://github.com/marinasundstrom/raven/actions/runs/37915415706)
  passed its build, WebAssembly playground, navigation and documentation browser
  checks and deployed successfully. The live [Raven site](https://marinasundstrom.github.io/raven/)
  reports source commit `e56c810845cbdb4bf3940ddfe06edd38fa750197`. Verified the
  published neoCLR link and .NET target wording, plus the extension reference's
  receiver, assembly and desktop/mobile layout. No neoCLR package release was started.

### 2026-10-09 — Windows and macOS release qualification

- **Author:** requested continuation with the neoCLR release, calling this the
  "Last chance to fix things" and specifying Windows and macOS.
- **Assistant:** selected the next sequential preview candidate (Preview 13),
  Windows x64 and macOS ARM64, and started fresh runtime, compiler, package and
  hosted checks. The author previously clarified the bounded POC scope; complete
  platform support is not inferred from this release request.
- **Findings/actions:** fixed Rust formatting and strict-lint issues, and refreshed
  the stale Preview 12 seed checksum for the added String.Replace service. The
  website build passed. Initial local AOT link failures identified a mismatched
  macOS SDK; checks were restarted with the matching Xcode SDK.
- **Open:** source/host gates, extracted package and editor qualification, exact
  release assets and publication. ARM64 AOT evidence does not establish Windows
  x64 native-code generation. Full source-owned native core bootstrap remains
  explicitly outside the completed String subset.
- **Qualification follow-through:** fresh macOS SDK/libraries and an extracted
  bundle passed collections, Tasks, JSON and live HTTP after repairing relative
  verifier report paths. The isolated packaged VS Code extension passed 26 checks
  without a development language-server override. Native website publication now
  uses a pinned reviewed rendering; clean builds check 3,491 pages and preserve
  136 explicitly marked compatibility gaps. Desktop/mobile review also corrected
  a Copy-button overlap.
- **Windows findings:** Git Bash supplied an invalid MSBuild compiler-host path;
  the packaging correction was placed on Raven main (`53e414bca`) and integrated
  into its neoCLR line (`71cafd353`, including an integration correction). A host
  refusal test also observed an unexpected successful connection after releasing
  its port; the candidate retains the Windows reservation to exclude parallel reuse.
  Windows validation remains required before treating that test repair as proven.

### 2026-10-09 — Separate Guides navigation before publication

- **Author:** asked to follow Raven's Language reference section: move the guides
  currently mixed into the API tree to Guides, with a separate visible sidebar
  and consistent navigation.
- **Assistant:** separated the authored guide menu from API navigation while
  retaining existing published article URLs. Identified RavenDoc's API-path
  restriction hiding explicit article sections and prepared a shared renderer
  correction with focused coverage (35 site-builder tests passed). The correction
  is on Raven main `6fae3c4e4` and the integration line `24a1a3716`. The neoCLR
  build checks all 3,491 pages; desktop/mobile review confirmed the visible sidebar,
  current-page links and separate API tree. Package qualification continues independently.

- **Author correction:** structural types should remain in API reference. Restore
  the structural overview and Array, Function, Tuple, Union and proposed Intersection
  contracts to the API menu; retain the separate feature/usage guides in Guides.


### 2026-10-09 — Bound long reverse-type relationship lists

- **Author:** noted that Derived types and Implementing types can grow very long,
  proposing initially collapsed sections or a few visible entries with Show more.
  The author explicitly added Derived interfaces to the scope.
- **Assistant proposal:** retain five visible entries and use a native disclosure
  for the remainder, with Show more/Show less, on shared RavenDoc output. Preserve
  the full alphabetically ordered links and indirect-relationship labels, including
  Derived interfaces. Four focused source/.NET metadata rendering tests passed;
  browser checks confirmed initial collapse, expansion and keyboard operation.
  The shared fix is on Raven main `0308f7b04` and integration `68a027962`.
- **Performed validation:** refreshed the native API snapshot and checked real
  Object and EquatableTo pages on desktop and mobile. Both keep five entries
  visible; expansion and keyboard collapse preserve every remaining link.
  Structural contracts remain in the API tree and Guides retains its own sidebar.

### 2026-10-09 — Preview 13 qualification complete

- **Author direction:** release the bounded POC for Windows and macOS, retaining
  the native String milestone and explicit remaining limits.
- **Assistant outcome:** Windows x64 and macOS ARM64 extracted bundles pass
  collections, Tasks, JSON and live HTTP consumers; each installed VS Code
  extension passes 26 checks. Canonical source workflow `37924446703` passes
  after five stale expectation repairs; the local source suite passes 1,816 tests.
  Windows GUI wait and command-capture repairs were acceptance-harness fixes,
  validated against the original bundle. Runtime code remains at `3ff1a66e`.
- **Documentation follow-ups:** separate Guides navigation, structural API contracts,
  bounded relationship lists, all 24 module introductions and a compiled Task.Run
  lead example have been reviewed in the browser.
- **Remaining limits:** the primitive CLI core/native seed remain explicit inputs;
  full native-only core bootstrap, general AOT coverage and x64 AOT are unfinished.
  ARM64 AOT remains a separate source experiment. See the
  [release qualification record](preview-13-validation.json).

### 2026-10-09 — Current documentation rather than release chronology

- **Author:** reported historical change notes while browsing the API reference,
  specifically pointed to “Preview 11” labels, and asked that those documents use
  the same style as the other pages.
- **Assistant response:** rewrite current guides around purpose, usage, contracts
  and limitations. Keep release chronology in the changelog and development records.
  The pass removes obsolete version labels and migration instructions, reconciles
  superseded Fault and TypeInfo notes, and preserves old anchors for incoming links.
  Benchmark configurations and evidence remain dated where the date is meaningful.


## 2026-10-09 — Render extensions through Raven semantics on both targets

- **Author:** requested equivalent RavenDoc presentation for .NET and neoCLR
  metadata. Type extensions must not appear as static carrier types; static and
  extension badges represent independent properties. Asked to check both the
  extension page and receiver member lists, review Raven’s selected extension
  namespaces, and explicitly verify `Option<T>` and `Result<T,E>`.
- **Assistant:** found that Raven’s website did not opt into extension namespaces
  and that .NET member declarations exposed synthetic receivers and `static`.
  Proposed E for instance extensions, S for ordinary static members and SE for
  static extensions, retaining semantic declarations and definition links.
- **Action:** implemented shared renderer changes and source/.NET metadata
  regressions, including generic constraints, specialized receivers, static
  methods/properties, default parameters and static methods with an explicit
  receiver-typed argument. Generic definition pages expose specialized and
  constrained extensions with their conditions visible, rather than claiming
  universal applicability. Raven’s configured namespaces cover its documented
  core and compiler extension libraries.
- **Author correction:** extensions must participate in the same member listing
  and grouping mechanism, not separate sections. The author also reiterated that
  the goal remains the neoCLR release.
- **Assistant response:** replaced the separate specialized sections with entries
  in the ordinary member groups, retaining receiver/constraint details on the rows.
  The author then required the signatures to match the other member rows. Compact
  labels now omit `func` and receiver-bound generic arguments; the applicability
  note preserves concrete constraints. Browser checks confirm those labels,
  grouping by declaring type and the extension toggle work for Option.Flatten and
  Result.WithContext. Stopped expanding the scope and
  returned to the already-qualified release assets.
- **Validation so far:** focused tests pass; native metadata rendering passes.
  Raven’s rebuilt Result page contains `WithContext` with `E: IError`; neoCLR’s
  Result page includes ten ResultOperators members. Browser review, renderer
  pinning completed at Raven integration `6a678bdeb` / main `55ccc9f47`.
  neoCLR’s pinned site rebuild checks 3,491 pages, retaining 136 explicitly marked
  legacy migration gaps; no generated page contains “Preview 11”. Browser checks
  verify compact labels and member grouping/toggles on both targets. Publication
  remains pending the release workflow; runtime qualification is unchanged.


## 2026-10-09 — Preview 13 publication

- **Author:** reiterated that the goal remained the neoCLR release while directing
  the final shared RavenDoc presentation corrections.
- **Assistant action:** published [Preview 13](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.13) from tag
  `v0.1.0-preview.13`, commit `0790b641`. Windows x64 and macOS ARM64 bundles,
  the exact qualified source archive, validation record and SHA256SUMS are attached.
  All five GitHub asset digests match the local qualified artifacts; both public
  package links return HTTP 200, and downloaded evidence/checksums match byte-for-byte.
- **Website:** neoCLR publication workflow
  [37932391592](https://github.com/marinasundstrom/neoCLR/actions/runs/37932391592)
  passed. Live browser checks cover the homepage release link, Guides, modules,
  native compilation, Result extensions and instance-extension declarations.
  Raven publication workflow
  [37931914316](https://github.com/marinasundstrom/raven/actions/runs/37931914316)
  also passed at main `55ccc9f47`. Live provenance and browser checks verify
  Option.Flatten and Result.WithContext compact signatures in ordinary member
  groups, extension toggles, SE static markers and instance declarations without
  CLR carrier details. Both website deployments and the runtime release are complete.
- **Scope:** this is the qualified POC, not full native core bootstrap. Bundled
  run tasks use the interpreter; ARM64 AOT remains a separate source experiment.
  The compiler/editor require the documented .NET SDKs; the interpreter does not.
- **Post-publication mobile check:** the author reported an expanded header and a
  misplaced Copy button. A fresh phone-sized browser load rendered both correctly;
  the assets advertised a ten-minute cache lifetime. The assistant proposed asset
  versioning, but the author chose to wait and then confirmed “Works om safari”.
  No speculative CSS or cache-versioning change was made.

## 2026-10-09 — Next priority after Preview 13 (discussion)

- **Author question:** asks which next priority to bet on: continued AOT/native
  compilation, including Windows; hot reload; JIT; API design; structural types;
  RavenDoc improvement/refactoring or a Raven-on-.NET rewrite; runtime hosting.
- **Assistant assessment:** the qualified release uses the interpreter, while
  ARM64 native HTTP execution has source-experiment evidence. Recommend making
  native compilation usable through an ordinary project workflow first, with
  explicit supported scope, diagnostics and standalone deployment evidence.
  Close native-only core/bootstrap gaps required by those consumers; track full
  bootstrap independently rather than imply the production String POC completes it.
  See the [roadmap](platform-roadmap.md), [release evidence](preview-13-validation.json)
  and [native web workbench](../benchmarks/native-web/README.md).
- **Assistant proposal:** begin with a packaged macOS ARM64 native consumer;
  follow with a bounded Windows x64 scalar/console portability proof before
  extending Windows services to the same HTTP workload. Then exercise an
  experimental persistent native host with explicit roots, callbacks, faults
  and shutdown; use that lifecycle experience for compatible-body reload.
  Retain API design and targeted RavenDoc corrections as supporting work.
  Defer broad structural type expansion, JIT and a wholesale RavenDoc rewrite
  until a concrete consumer or measured bottleneck justifies them. A Raven-on-.NET
  tooling slice could independently exercise Raven, but would not validate neoCLR AOT.
- **Comparison:** [.NET Native AOT](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
  already provides self-contained native deployment; neoCLR must demonstrate its
  own useful UTF-8/Raven workflow rather than claim AOT itself is unique.
  [.NET hosting](https://learn.microsoft.com/en-us/dotnet/core/tutorials/netcore-hosting)
  provides a native-host baseline, distinct from AOT executable deployment.
  Sources reviewed 2026-10-09; these are existing .NET capabilities, not proof of
  neoCLR performance or hosting equivalence. Reload proposals reuse the
  [existing lifecycle investigation](native-execution-investigation.md#hot-reload-across-execution-modes).
- **Status:** recommendation only; no subsequent author selection is available.
  Recorded this discussion and changelog entry. No compiler/runtime/tooling
  implementation, roadmap reprioritization or commit was performed.

- **Subsequent author decision:** “OK. Document that direction. And start working.
  Commit each slice.” The assistant records the selected native compilation,
  Windows portability, hosting and reload sequence at the top of the roadmap,
  preserving the earlier recommendation as the preceding discussion. Begin with
  a bounded ordinary-project native build workflow using existing backend contracts.
  Implementation outcomes follow in the linked roadmap/evidence and changelog;
  this decision alone does not establish packaging or native platform qualification.

- **First implementation outcome:** committed direction separately, then added
  a source-checkout ordinary-project console build command with the bundled Raven
  compiler, current AOT backend and bounded native GC host. Focused acceptance
  passes three standalone consumers (UTF-8, interpolation and a guest fault),
  interpreter parity, unsupported recursion rejection and stale/existing-output
  protection. A mismatched inherited SDK exposed a local link failure; explicitly
  selecting Apple's macOS SDK/toolchain resolved it. See
  [recorded evidence](experiments/native-project-build-validation.json).
  The helper is not yet a packaged AOT release; matched tool/adapters packaging
  and the HTTP project path are next.

## 2026-10-09 — Native kit and co-evolving runtime suspension

- **Author continuation:** directs continued work on the selected native compilation
  priority. The assistant stages a relocatable development console kit containing
  the compiler/library bundle, freshly built AOT tool, adapters and sample project.
  Extracted acceptance passes outside the checkout with Cargo absent from PATH,
  isolated UTF-8 execution and pre-build rejection of three modified tool inputs.
  [Evidence](experiments/native-build-kit-validation.json) records the candidate;
  no release publication or Windows/HTTP qualification is claimed.
- **Author clarification:** “Another thing to consider is runtime suspension,
  runtime scheduling, and green threads when we have a the native compilation
  foundations ready and let that co-evolve”.
- **Assistant response/action:** records a cross-cutting track alongside subsequent
  native/HTTP/hosting work, rather than waiting for complete AOT coverage. Retains
  the current hosts, queue adapters and stack/TLS mechanisms as provisional. Adds
  proposed suspend/resume, GC-root retention, cancellation/shutdown and code-generation
  lifetime checks to the [scheduling design](runtime-scheduling-design.md#co-evolution-with-native-foundations--2026-10-09).
- **Open:** stackful/stackless choice, carrier migration, preemption, public Scheduler
  shape and implementation scheduling remain unselected. The author clarification
  does not establish implemented green threads or expand the console kit's scope.

## 2026-10-09 — Dedicated Windows AOT GitHub Action

- **Author:** directs continued work and requests special actions for Windows
  compilation, clarifying “GitHub action”.
- **Assistant action:** expands the local Windows scalar workflow into an
  exact-revision, manually dispatchable Windows Server 2022/MSVC gate with retained
  compilation/link/execution artifacts and a native completion check. Pushes the
  committed native slices and workflow so GitHub can execute the gate.
- **Outcome:** [run 37947583061](https://github.com/marinasundstrom/neoCLR/actions/runs/37947583061)
  at `7cfe27222d875686f6ce5b0bfa052b8a64d05e88` passes four tests, none skipped,
  including 32 MSVC-linked native/interpreter comparisons. All 47 downloaded
  evidence file hashes match the report. See the
  [retained execution evidence](windows-aot-execution-validation.json).
- **Boundary:** this closes the scalar/literal-console Windows execution gate.
  Raven Hello World has Windows object emission evidence; its executable run,
  Windows managed services/project kits and unwinding remain separate work.
