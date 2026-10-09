# Changelog

Notable changes to neoCLR, grouped by release and development date. Every commit
updates the Unreleased section; related changes on the same date share an entry.
Published sections are frozen. See the [maintenance workflow](docs/changelog.md).

## Unreleased

### 2026-10-09

- Prioritize native JSON through the existing introspection/reflection-backed mapper.
  Add an interpreter-validated typed round-trip probe and a staged native plan.
  Implement closed native RuntimeTypeHandle storage and ldtoken identity across
  generic specialization, calls and fields; retain token-only type shapes, exclude
  handles from GC tracing and reject numeric casts. Identity is image-local and
  does not provide TypeInfo queries or reflection invocation. Improve unsupported
  boxing diagnostics for the next JSON admission gap; native JSON remains unfinished.
  Refine the native reflection plan around semantic metadata, separate discovery/
  invocation retention, checked adapters and an independent reflection consumer.
  Explicit build-time roots remain a proposed first mechanism, not a public API.
  Qualify native type identity on macOS ARM64 and Windows x64 in action
  `37978562111` at `3a8c315c`, verifying both artifacts and their source inputs.
  Preserve selected nominal source declarations, closed property signatures and
  member identities/access in a private build-time catalogue before native lowering.
  Keep accessor bodies excluded unless independently reachable; runtime metadata
  tables, explicit retention roots and reflection invocation remain unfinished.
  Record the author-directed interpreter/native JSON benchmark case after correctness
  parity, with a proposed measurement protocol; no benchmark results are claimed.
  Lower the exact TypeEquals runtime service to native image-local type identity,
  with interpreter parity and malformed-contract rejection tests. Extend the
  macOS/Windows type-token action with an executable runtime-equality consumer;
  general TypeInfo queries and reflection invocation remain unfinished.
  Execute native TypeName/TypeArgumentCount queries over primitive and closed nominal
  token producers using source semantic names and generic arity. Add a standalone
  interpreter-parity consumer, GC/bounds checks and explicit missing-metadata/shape/
  contract rejection tests. This uses bounded generated dispatch; general runtime
  descriptor tables, property queries, invocation and JSON remain unfinished.
  Qualify the descriptor consumer on macOS ARM64 and Windows x64 at `ba871e0d`
  in action `37980244735`, verifying all six direct-token/equality/descriptor reports
  and their source/artifact hashes. Add runnable consumer instructions.
  Add native TypeShape inspection from source metadata, preserving all existing
  selectors within the primitive/closed nominal subset, including generic/source
  visibility. Invalid selectors preserve RuntimeError and caller frames. Validate
  160 interpreter/native cases, closed-union/imported-visibility facts and unchanged
  ordinary user-fault behavior; extend the portable descriptor consumer.
  Qualify portable shape checks on macOS/Windows in action `37981420430` and
  preserve a Raven NominalTypeInfo probe whose name/nominal queries run natively
  with interpreter parity on macOS. Full reflection/JSON remain unfinished.

- Add a private retained native session with explicit root ownership, an entry-reset
  anchor, bounded callback retention and fail-closed dispatch after guest faults.
  Reject copied/wrong-thread sessions, reentry and shutdown with foreign service
  roots; preserve diagnostics until explicit whole-session discard, including intern
  pool teardown. Add portable lifecycle and generated-callback gates. This is an
  ownership experiment, not a public hosting API. Add an explicit opt-in AOT
  `fn<Void>` bootstrap export and atomic session adoption, replacing fixture heap
  scanning with an owned handle. Preserve output and existing session diagnostics
  on rejected handoff; check collection before adoption, bootstrap faults, foreign
  roots and root-quota exhaustion. Ordinary entry ABI and Raven contracts remain
  unchanged. Add a retained HTTP consumer using one listener and counter across
  three dispatches, collecting between exchanges and discarding the session after
  service teardown on success or faults. Add an explicit private project bootstrap
  option, interpreter/native parity checks and a Windows HTTP session action.
  Allow five seconds for the post-exit refusal probe so Windows can report the
  closed listener after its SYN retry interval; a timeout still fails the gate.
  Windows action `37976713202` at `84b6e425` passes all three cases with complete
  results matching macOS; verify 1,435 artifact hashes and 37 tracked source inputs.
  Exchanges use await; only the documented unit callback adapter starts asynchronous
  work for the host to drain. General async export completion remains unfinished.
  Record a separate pinned-compiler async pattern-local hoisting issue; the final
  interpreter consumer uses a normal lexical binding, not a compiler fix.
  Handoff Action
  `37974336464` at `201f567b` passes on macOS ARM64 and Windows x64 with matching
  faults; verify 21/25 artifact hashes and 15/16 source inputs respectively.
  Run `37972822432` at `c77e2b07` passes the same lifecycle and generated-callback
  gates on macOS ARM64 and Windows x64; verify 21/25 artifact hashes and 15/16
  source inputs respectively. Compiled callback faults also match the interpreter
  in the focused Rust regression.

- Add a private serial-reuse HTTP host gate: preserve the first guest failure,
  render then clear fault roots, release task/socket scopes and require an empty
  collected heap before another entry. Compare success/fault-or-cancellation/
  success on one native context with fresh interpreter invocations. This is
  stateless reuse qualification, not a public persistent-host API. All 14 macOS
  client/factory/recovery cases pass. Windows client run `37970610749` passes the
  same 14 cases and server run `37970610944` passes all five cases at `15ebc8b5`.
  Verify downloaded binary/library/compiler outputs and tracked source hashes;
  record the artifact service's omission of ten bundled editor-setting files per
  run and retain hidden files in future uploads. Ignore local factory-project build
  outputs. Record the
  previous await-based Windows client/server successes at `fbd73677`, verifying
  42 client and 53 server artifact hashes, and advance the roadmap to hosting.

- Add development `Task.CompletedTask: Task<()>` and inferred
  `Task.FromResult<T>(value): Task<T>` with immediate successful completion,
  preserved result identity and normal deferred callback dispatch. Replace
  completed-Promise boilerplate and pending fallback tasks in HTTP samples.
  Project unit property types in the temporary CLI reference and refresh public
  API documentation. Windows HTTP gates rebuild source libraries with the pinned
  compiler and primitive bootstrap so development APIs are tested together.

- Record requested Windows ARM64 qualification and its ABI/stack/toolchain gates;
  keep Windows HTTP project integration as the immediate showcase priority.
  ARM64 native support remains planned.

- Begin the author-directed macOS/Windows native HTTP parity work. Share socket
  lifecycle, accept, transfer, cancellation and deadline policy behind a private
  Winsock/POSIX boundary, preserving native-width Windows handles and balanced
  Winsock ownership. Add identical listener/accept/transfer contract tests to a
  two-platform GitHub Actions matrix. Run `37960722734` at `da8ca1f2` passes all
  three consumers on macOS ARM64 and Windows x64; verify 45 artifact hashes and
  14 source inputs per platform. Native HttpClient DNS/connect qualification
  remains planned. Add an opt-in Windows HTTP project profile
  with guarded heap/stack admission, shared HTTP callback loop, binary output and
  system-only KERNEL32/WS2_32 imports. Add paired interpreter/standalone request
  and fault validation. Windows run `37961490213` at `353188bf` passes all
  five scenarios with exact interpreter parity, executable-only deployment and
  overwrite/stale-output rejection; the same macOS cases pass. Verify 53 Windows
  and 29 macOS artifact hashes, matching source and bundle inputs. Add the
  validated HTTP project template and standalone showcase instructions. Add bounded
  native DNS workers and ordered nonblocking IPv4 connect attempts, sharing
  cancellation, phase/shared deadlines and owner-thread callback cleanup. Local
  sanitizer contracts pass for DNS snapshots, held-worker cancellation/timeout,
  worker limits, scope teardown, address fallback and refused connections;
  Windows adapter run `37963921861` passes all four socket consumers on Windows
  x64 and macOS ARM64. Bind the existing DNS/connect services in the native backend,
  preserve inherited reference fields and admit proven monomorphic inherited display.
  Raise coordinated type/clone bounds to 512, retain the 1024-function cap and reject
  dependency nesting beyond 128 before host-stack exhaustion. Match collector typed
  array validation to the new type bound. Add an await-based HttpClient showcase and
  explicit native entry I/O pumping at a saved published-root boundary, with guard
  cleanup on faults. Record await as the default application convention. Add a
  standalone Windows client/server qualification workflow; end-to-end execution is
  under validation. Focused unit, inheritance and sanitized heap/scope tests pass.
  Windows run `37966271634` at `87ce8a71` passes all ten native client scenarios,
  including a separate Raven server process; verify 42 retained artifact hashes and
  KERNEL32/WS2_32-only imports. Convert the server showcase to async Main/await and
  omit redundant trailing unit returns. All five server request/fault cases pass on
  macOS; six queue-entry regressions preserve interpreter/native parity. Keep
  synchronous listener setup separate from awaiting the exchange while a pinned
  compiler pattern-hoisting issue is tracked independently.

- Update website HTTP and Task samples to await completion from async Main,
  including DOM, mapped JSON and routed client/server pairs. Omit redundant trailing
  unit returns; document the cancellation observer's intentional callback. Extract
  complete tested entry/serving examples on the Web and native compilation pages.
  Refresh native platforms, build profiles, DNS/connect and async-entry support;
  preserve Windows ARM64, general scheduler and deployment limitations. All ten
  focused website sample checks pass, with artifact/source hashes retained in
  `docs/website-await-validation.json`.

- Add explicit `x86_64-pc-windows-msvc` AOT target selection for the bounded scalar
  and literal-console profile, emitting x64 COFF with the target calling convention.
  Preserve default macOS ARM64 output; reject Windows managed/closed-world and
  inspection profiles. Cross-target object checks and macOS regressions pass.
  Add a Windows C consumer and focused CI job for calls, branches, faults and UTF-8
  byte output. The first dedicated Windows Server 2022 run now passes all four
  tests with no skips, including 32 linked native/interpreter comparisons at
  `7cfe2722`; downloaded artifact hashes are verified. This qualifies only the
  bounded scalar/literal-console profile, not Windows GC/HTTP or project kits.
  Extend the dedicated GitHub Actions gate with exact-revision manual dispatch,
  explicit MSVC setup, retained objects/executables and compiler/linker/test logs,
  and a report that cannot pass when native execution is skipped or incomplete.
  Add a required standalone Raven Hello World execution gate for retained PE/#Neo
  and NEOX inputs, linking a static-CRT startup adapter and deploying only the EXE;
  compare UTF-8 stdout/stderr with the interpreter. Windows run `37948836623`
  at `ee0dac80` passes all five tests without skips, including both standalone
  containers and the 32 scalar comparisons; all 66 artifact hashes are verified.
  That run qualifies the retained compiler fixture. Extend the gate to build pinned
  Raven `71cafd35` with the current metadata writer and compile Hello World source
  on Windows, requiring separate PE/#Neo and NEOX execution records. Retain
  compiler revision/hashes, build diagnostics, source and emitted metadata. Windows
  run `37949899691` at `d80dd878` passes six tests with no skips: four standalone
  executions (retained/fresh PE/#Neo and NEOX) and 32 scalar comparisons. Verify
  all 91 downloaded artifact hashes. Select AnyCPU explicitly for the
  managed compiler build so MSVC’s Platform=x64 environment cannot redirect its
  output layout; native code still targets Windows x64. Extend fresh-source
  qualification with calls, branches, empty/repeated loops and byte-exact UTF-8,
  NUL and combining-character output. Add the scalar Boolean constants and
  comparison-result lowering emitted by Raven, preserving typed stack checks and
  Int32-only signatures/locals. Local native boundary/rejection checks pass.
  Windows run `37951030197` at `d7ae97ab` passes seven tests without skips,
  including six standalone executions and 32 scalar comparisons; all 113
  downloaded artifact hashes are verified.
  Define Windows managed-host allocation, stack and suspension ownership
  prerequisites. Add a private bounded VirtualAlloc heap with reserved guard
  pages and a focused MSVC lifecycle/guard workflow. Windows run `37952512363`
  at `9c3c586e` passes 12 allocation lifecycles and both guard-boundary checks;
  verify nine artifact hashes and three source hashes with checkout line endings.
  Add a Windows collector host over that heap, rejecting foreign-thread or
  live-root/frame teardown. Extend the focused gate with cyclic-graph retention,
  host-to-frame root handoff, reclamation, exhaustion/output preservation and
  cleanup, plus the existing collector contract consumer. Scope MSVC C4200
  suppression to the shared flexible-array text declaration and assert its
  unchanged 8-byte header/byte offset. Windows run `37953377713` at `8ac8040b`
  passes the integrated collector consumer, existing collector contracts and
  12 guarded-heap lifecycles. Verify 24 artifacts and 13 source hashes; local
  collector/host-root regressions also pass.
  Add a private Windows remaining-stack probe and executable checks for small
  stack rejection, bounded deep calls, collection at the limit, GC-frame cleanup
  and fiber rejection. Use fresh system bounds and reservation-based headroom.
  Windows run `37955003321` at `78f62757` passes 128 KiB admission rejection
  before allocation and safe return at depths 15/46 on 512 KiB/1 MiB stacks;
  heap/collector regressions pass. Verify 31 artifact and 16 source hashes.
  Add an explicit integer-only Windows generated-stack experiment, with inline
  4 KiB page probes, final machine-frame bounds including shadow space, a bounded
  entry prologue and existing diagnostic/GC-frame cleanup. Extend the Windows host
  Action with a linked recursive guest, small-stack rejection and repeated host
  reuse. Windows run `37956371244` at `29670d3b` passes the generated guest,
  small-stack rejection, recursive fault return and successful context reuse,
  plus COFF/admission/final-frame tests and existing host regressions. Verify all
  49 artifact and 37 source hashes. Local macOS stack regression also passes.
  Add a separately gated Windows Int32-array heap experiment with generated
  local/argument roots, allocation churn, bounds/exhaustion faults and host reuse.
  Windows run `37958065076` at `cb12c7de` passes 408 allocations across four
  invocations in a 2 KiB heap, generated live-root retention, bounds/exhaustion
  faults, full reclamation and context reuse. Verify 60 artifact and 40 source hashes.
  Reuse canonical collector kind constants in the text/array adapter, avoiding
  MSVC's cross-enum static-assert warning without disabling warnings.
  These remain private probes; the project integration below adds the console profile.

- Add an explicit development `windows-console` profile to the project build command:
  Raven project compilation, guarded GC/stack host, x64 COFF/MSVC linking, static CRT,
  byte-preserving standard I/O and standalone dependency checks. Add a source sample
  and dedicated Windows project Action with interpreter parity, standalone deployment,
  existing-output protection and stale-build rejection. Windows run `37958647006`
  at `a0b50a96` passes all five projects: managed arrays/text, interpolation, binary
  input/output and divide-by-zero diagnostics, with exact interpreter parity and
  executable-only deployment. All EXEs import only KERNEL32.dll; verify all 126
  artifact hashes, 25 source inputs, 32 pinned-bundle inputs and five project inputs.
  Local COFF compilation, profile rejection and macOS project regression pass.
  Keep HTTP, file/path, scheduler and extended text services outside this profile;
  packaged development kits still advertise only their macOS profiles.

- Add a development macOS ARM64 project-to-executable console command using the
  selected bundle's Raven project build, existing AOT backend and explicit native
  runtime adapters. Record build inputs/diagnostics and publish only after linking
  and OS-dependency checks; preserve existing output directories. This synchronous
  1 MiB GC profile is also available as a separately staged development build kit,
  with a matched AOT build, catalogued compiler/library bundle, native adapters,
  sample project, input hashes and dependency license texts. Application builds
  from the extracted kit require neither a source checkout nor Cargo; kit integrity
  is checked before tool invocation. This does not change released bundles or add Windows.
  Focused acceptance passes UTF-8/interpolation and fault parity, isolated execution,
  unsupported recursion rejection and stale/existing-output protection. Select a
  matching Apple Clang/macOS SDK explicitly through xcrun.
  Extracted-kit acceptance also passes an application build outside the checkout
  without Cargo and rejects modified adapter, backend and compiler configuration
  before invoking tools.
  Add an explicit HTTP project/kit profile with the existing Raven server sample,
  private socket/task host, native GC and guarded stack. Console remains the default;
  HTTP uses a bounded cooperative host, not a production Scheduler or green threads.
  Extracted acceptance matches the interpreter for greeting/fragmented requests,
  malformed-request and handler rejection, and guest callback faults with host
  cleanup checks and standalone execution.

- Record the post-Preview 13 priority discussion and subsequent author selection:
  prioritize usable project-level native compilation, bounded Windows portability,
  then hosting and reload; implement and validate in individually committed slices.
  These are development priorities, not newly shipped capabilities.
  Record the author's follow-up that suspension, runtime-owned scheduling and green
  threads should co-evolve with the native foundations, with shared cross-mode
  lifecycle contracts and provisional host/stack adapters.

- Correct API extension visibility: internal runtime-service containers no longer
  contribute public-looking members such as ConsoleFlush and ConsoleWriteBytes
  to Boolean. Refresh the shared RavenDoc renderer and native reference snapshot;
  retain a regression check and document the separate native extension metadata
  classification limitation. Released runtime packages are unchanged.

- Record Preview 13 publication for Windows x64 and macOS ARM64, public asset
  checksum verification and website deployment evidence. Published release notes
  and qualification records remain unchanged.

## 0.1.0-preview.13 — 2026-10-09

Windows x64 and macOS ARM64 toolchain bundles pass extracted consumer and installed
editor qualification. Canonical source validation and focused platform checks pass;
ARM64 AOT remains a separate source experiment, and full native core bootstrap is
unfinished. See [release notes](docs/preview-13-release-notes.md) and
[exact evidence](docs/preview-13-validation.json).

### 2026-10-09

- Align shared RavenDoc extension presentation across .NET and native metadata:
  instance declarations omit carrier details, static extensions use an SE badge,
  and constrained/specialized generic extensions remain discoverable with their
  conditions in the ordinary member lists, with consistent compact signatures
  and shared grouping/toggles. Pin the reviewed renderer and native snapshot after checking both
  websites, including Option, Result and collection receiver member lists.

- Rewrite current API and feature guides around usage and supported behavior,
  removing stale Preview 11 labels, upgrade instructions and superseded checkpoints.
  Preserve compatibility anchors, explicit POC limits and research/benchmark evidence.

- Lead the Tasks guide with the compiled Task.Run/await example; keep explicit
  isolated Thread usage in its own later section. Adapt module documentation IDs
  only in the temporary legacy-reference website pass.

- Document all 24 rendered API modules with purpose, current capabilities,
  boundaries and guide links; refresh the native reference snapshot.

- Keep five entries visible in long derived-type, implementing-type and
  derived-interface lists, with a shared RavenDoc Show more/Show less disclosure.
  Preserve all links and short lists; document the tested .NET 11 RC1 toolchain.

- Refresh source regressions for the current contracts: reject an unassigned
  metadata schema, recognize conflicting-signature diagnostics, preserve String
  owner identity through Object views, and check shared fault stack rendering.
  The five focused test targets pass; runtime behavior is unchanged.

- Capture VS Code acceptance output through a process task instead of Unix shell
  quoting, preserving argument boundaries on Windows and macOS. Normalize only
  CRLF/LF in exact-output comparisons; keep exit-code and stderr checks.

- Wait for Windows VS Code acceptance to finish after its asynchronous launch.
  Allow editor-only qualification of a previously built bundle, preserving the
  original package and consumer evidence instead of rebuilding unaffected tools.

- Separate Guides from the API reference tree, using an independently visible
  article sidebar with current-page navigation while preserving existing URLs.
  Keep structural type contracts (Array, Function, Tuple, Union and proposed
  Intersection) in the API reference; usage articles remain Guides.
  Pin the shared RavenDoc section-navigation fix; check the complete site and
  desktop/mobile navigation before publication.

- Select Preview 13 for Windows x64/macOS ARM64 qualification. Pin the matching
  Raven packaging revision, version the Windows editor asset consistently with its
  SDK, and add installed-extension acceptance without a development-server override.
  Draft installation, migration and POC limitations in the candidate release notes.
  Publication and final platform qualification remain pending.

- Resolve native-bundle verification report paths before invoking tools from the
  extracted bundle, preserving relative report destinations. Reserve the Windows
  refusal-test port for the test lifetime to prevent parallel listener reuse;
  retain the macOS-specific released-port setup.

- Make the reviewed native API rendering a checksum-pinned website publication
  input, so clean builds retain declaring assemblies, module members and extension
  presentation. Keep local preview selection and explicitly marked legacy coverage
  gaps; reject corrupt snapshot inputs before replacing the visible site. Reserve
  room for the Copy button in long mobile API declarations.

- Align the native String by-reference rejection regression with the current
  explicit-interface diagnostic. The test still requires failure before object
  publication; no by-reference receiver support is added. Exercise a reachable
  missing array capability in inspection tests instead of an unreachable instruction
  that the compiler correctly discards, and check the current 256-type budget.

- Refresh the release validator's native seed pin for the added String.Replace
  service. Keep Preview 12's published bootstrap evidence unchanged; candidate
  source and package execution must still pass before publication.

- Prepare Windows x64 and macOS ARM64 release qualification. Apply Rust formatting
  and replace an equivalent metadata-version pattern to satisfy strict Clippy on
  the current toolchain; remove an unnecessary cloned test expectation as well.
  No metadata admission behavior changes.

- Record the completed author-directed Raven website publication: validate shared
  RavenDoc Type extension presentation for .NET and introduce neoCLR as a related
  experimental project while retaining Raven's current .NET target. Verify the
  deployed source revision and live pages.

- Record completed RavenDoc integration and temporary-branch cleanup after the
  native String POC. Verify the existing pinned publisher checksum; no archive
  replacement or website publication is needed for this integration.

- Complete the bounded native String POC from unchanged Raven sources and native-only
  metadata. Add verified character/scalar array views, explicit String interface
  dispatch and receiver-slot construction lowering. Check interpreted/ARM64 output,
  exact fault rendering, repeated GC entries and missing-binding rejection; retain
  documented Object-display, bootstrap and intern-lookup limitations for later work.

- Share Unicode casing/folding kernels with statically linked native String services,
  including grapheme/scalar snapshots and character construction. Add a GC-rooted,
  per-entry intern pool with explicit quotas; record its provisional linear lookup
  cost. Preserve exact managed-vector service types and allocation limits in the
  interpreter. Validate native ownership, quota and failure behavior with UBSan.

- Add explicit memberless native primitive references for source-library bootstrap.
  Preserve them through metadata readers and require exactly one executable owner
  during loading; reject standalone execution and malformed storage declarations.
  Document Raven’s native array/callback/source-String integration contract.

- Give extension receivers a dedicated API section with separately linked type
  components and type-parameter navigation. Document Operators' element type in
  the same Type parameters table used by generic types. Module pages list Type
  extensions separately and show their receivers. Use Type extension in page labels
  and icon tooltips; place Receiver before Type parameters and Remarks. Share the
  rendering with Raven.

- Add native metadata `ValueIs` and `ValueUnpack` operands for typed inspection of
  runtime erased results, including generic extraction. Reject incompatible stacks
  and executable CLI emission; validate matching and mismatched UTF-8 slice results.

- Present extension containers with a dedicated icon, an extension declaration and
  linked receiver types in the API reference. Share the rendering with Raven main;
  validate native Operators as `extension Operators for Iterable<T>`.

- Correct API sidebar icons for module functions and constants using RavenDoc's
  existing member glyphs. Group module constants under Constants rather than
  Members, preserve reference URLs, and validate the native Math module's rendering.
  Rework the API landing page around browsing modules and finding APIs by task,
  moving detailed contracts and development history out of the introduction.

- Qualify the six unchanged production collection/equality interfaces needed by
  String through native-only compilation and imported generic dispatch in project,
  interpreter and ARM64 execution. Missing contract references reject before output;
  record the remaining source String dependency boundary. This is bootstrap
  qualification, not complete native String support or a new public API. Add seven
  real text-service forwarding methods to the native fixture RuntimeServices type;
  direct facade calls pass both execution modes and source String now resolves
  that type. Remaining member/storage dependencies still reject before output.

- Refocus the website on understanding and using neoCLR: explain the runtime,
  library and Raven toolchain on the homepage, group guide entry points, replace
  development chronology with current capabilities/directions, and separate native
  benchmark reports. Distinguish website guidance from GitHub implementation docs;
  remove stale release commentary while preserving versioned sample requirements.

- Add the private ordinal UTF-8 replacement service to interpreter and ARM64 native
  text bindings, with non-overlapping replacement, deletion, unchanged-result
  identity and checked allocation. Focused parity and failure-path checks pass.
  Add the source-owned String.Replace facade, exact bridge/native service mappings,
  generated implementation and matching API reference. Production Option/Result
  quote/backslash escaping now passes project/interpreter/ARM64 checks over the
  native core fixture. Full native source-core bootstrap remains open.

- Qualify Raven's immediate constructed-value receiver fix with native-only union
  display and imported ordinary/generic/nested receiver checks. Add an explicitly
  failing-contract escaping audit: both modes still quote payloads without escaping
  quotes/backslashes because String.Replace is absent. This records the shared gap,
  not a successful escaping implementation.

- Add architecture and metadata-format documentation and website guides, with
  compilation, layered runtime-service and artifact-format diagrams. Document
  current PE/NEOX encoding, ownership, admission boundaries and native execution
  limits; link both guides through website and repository navigation. Clarify that
  modules forming namespaces are the primary organization of class-library APIs,
  with the Modules feature page providing the detailed model.

- Admit nonmatching Char box tests in the closed native profile, preserving null
  Object results while rejecting Char box producers. Production Option/Result
  string-payload descriptions now match project/interpreter/ARM64 execution,
  including Unicode/NUL text. Extend qualification to Int32 zero/negative/min/max
  and integer Result payloads, with rejection before output when the native integer
  formatter binding is missing. Matching Char boxes, escaping and complete core
  bootstrap remain unsupported or unqualified.

- Lower generic String boxing to the existing native Object reference view and
  exclude unrelated unboxed value construction from Object display dispatch.
  Native-only Raven display/alias/distinct-string checks now match the interpreter;
  focused UTF-8/NUL fault and stack-trace regression passes. Unsupported value-box
  display guards remain. Complete union formatting still reaches unsupported isinst Char.

- Preserve System module ownership when AOT projects verified primitive methods to
  private functions; native-only static and String instance wrappers now pass
  interpreter/ARM64 UTF-8 checks. Missing declaration-owner diagnostics identify
  the module and assembly. The optional native core fixture advances the unchanged
  four-source union build to imported Attribute inheritance; production core and
  complete union formatting remain unqualified. A follow-up includes production
  Attribute with UnionAttribute and now qualifies the unchanged five-source union
  subset through project/interpreter/ARM64 pattern and propagation-protocol checks;
  imported-base rejection remains a separate control.

- Qualify native-only copied-struct execution after Raven's core ValueType identity
  correction. Extend the minimal core fixture with Byte and retain the unchanged
  production Option/Result frontier: missing String.Concat now reports a compiler
  diagnostic without publishing output. Real native String/core completeness and
  the general diagnostic repair's independent Raven-main port remain open.

- Qualify Raven's native-only project core selection through a module consumer,
  project build/run and interpreter/ARM64 native parity. Reject mixed bridge/core
  configuration without replacing published output. The shared project provider
  watches native core metadata; complete production core bootstrap and installed
  editor/package qualification remain open.

- Add the native declaration-module foundation: a versioned manifest table,
  scoped authoring and reader/discovery APIs, explicit assembly ownership, empty
  modules and validation in the runtime. Older metadata exposes marked namespace
  projections; older readers reject the new field. Physical metadata modules and
  guest RuntimeContext remain unchanged; module-private access is not implemented.
  Integrate Raven file/block module syntax, qualified imports and target-aware editor
  labels, with a checked sample returning 42 in interpreted and ARM64 native modes,
  including a separate library assembly. Add a development Modules feature page and
  module navigation/labels while retaining real Assembly owners in native API pages.
  The vendored RavenDoc gains explicit module terminology; existing routes remain valid.

- Preserve the selected native API preview across ordinary local website rebuilds.
  Apply and validate it before replacing the served website; fail without replacing
  the visible site when the selected audit is missing or fails validation. Selection
  is local and explicit; publication inputs and marked legacy coverage gaps remain
  unchanged.

- Move date/time values, clocks, timezone mappings/errors and calendar policies to
  System.Time; rename the civil Time struct to TimeOfDay. LocalDateTime.Time keeps
  its property name. Update source/native ownership, CLI bridge catalogs, generated
  implementations, consumers and API reference together. This is a breaking
  development identity change: update imports and rebuild matching libraries/apps.
  Formatting policies stay in System.Globalization; representation and arithmetic
  contracts are unchanged. Calendar output matches interpreted/native execution;
  timezone service and nonempty record-boxing AOT gaps remain explicit.

- Fix the native HttpContext constructor metadata mismatch through Raven's explicit
  unit-owner mapping (92a593ff7). Nested Promise<Result<Void, HttpError>> signatures
  now consistently select System.Runtime.Void instead of mixing in the temporary
  primitive core's Void. A reduced regression fails before the fix and both unit
  spellings pass after it; the complete native library bundle rebuilds successfully.

- Integrate the native API migration preview into the local neoCLR website shell,
  at the normal API URLs, including branding, navigation, authored API additions
  and search. Recover matching reviewed XML comments using canonical native IDs;
  preserve unmatched old pages with explicit migration-gap notices. Normalize
  mislabeled XML comments in a private input copy and rebuild native libraries so Math Pi/E/Tau appear with canonical IDs and values.
  Validate Object links, function overload groups and sidebar entries before commit;
  remaining coverage migration is required for the upcoming release.

- Audit real Runtime/Data/Networking/Web metadata through RavenDoc after fixing
  assembly-level function/constant documentation in Raven. The migration preview
  renders 1,632 pages with actual declaring libraries and no CoreProbe owner label.
  Preserve the complete website while public member/comment/route coverage is
  qualified. This audit explicitly retains the historical bundle's CLI primitive
  bootstrap; it does not satisfy the no-bridge release gate. Extend the reduced
  native core with Double for constant documentation checks; native execution still
  returns 42 in both modes.

### 2026-10-08

- Qualify RavenDoc's explicit native library loader using the full existing renderer:
  grouped navigation, real declaring identities, XML/Markdown and namespace comments,
  member signatures, generics, inheritance/extensions, source/API content, search and
  cross-library links. Preserve the current complete website reference until the
  production bundle has matching coverage; no CoreProbe relabeling or reduced replacement.
  Audit the existing split bundle: its CLI core correctly rejects on the native-only
  loader. Update the native-compilation page with driver and documentation progress.
  Fix metadata-reference links to repository design notes and experiments so the
  complete website passes local-link validation and can be previewed (1,828 pages;
  18 website tests).

- Exercise Raven's explicit native-only metadata mode with an authored native core:
  compile an integer consumer without CLI semantic references and run it interpreted
  and ARM64 native with result 42. Extend the probe through Raven’s explicit native
  reference catalog to call a separate native library with the same result. Check
  immutable catalog snapshots, CLI-input rejection, missing cores, exact identity mismatch and
  CLI emission refusal; standalone output links only libSystem. Full source-runtime
  and project/editor bootstrap remain pending. The explicit rvnc --native-core-reference
  driver path also passes the separate-library consumer and conflicting-option/output
  guards. Record RavenDoc's separate native-input
  migration so eventual ownership labels come from real library metadata.

- Allow native core PE/#Neo producers to reference their own core declarations with
  local TypeDef handles instead of failing on an external self-dependency. Require
  an authored Object root and reject missing local projection declarations; retain
  executable CLI and foreign-identity guards. Add writer checks and a native-only
  Raven core probe, which exposed the initialization failure closed by the explicit
  compiler API mode above. Full no-bridge target qualification remains pending.

- Record the supplied first-class module proposal verbatim and assess the author's
  clarifications that modules become namespaces and RuntimeContext should center on
  modules rather than assemblies. Separate exploratory ownership,
  visibility and compatibility choices from current metadata APIs and release scope;
  preserve illustrative artifact names without adopting them. No runtime change.

- Bind exact native ReadAllText/WriteAllText with explicit --bind-file-input and
  --bind-file-output on macOS ARM64. Preserve UTF-8 bytes, preflight limits/path checks,
  regular-file checks before truncation, error statuses and handle cleanup. Reads probe
  actual bytes before strict decoding and close handles before managed allocation;
  terminal allocation failure leaves outputs unchanged. Eighteen read and eleven write
  comparisons, existing UTF-8 controls, a let-pattern consumer and the unchanged file
  sample pass interpreted, sanitized native and standalone execution. Blocking writes
  are not atomic/durable saves and may leave partial output after I/O failure.

- Correct interpreted ExecutingAssembly for source-built runtime facades: skip the
  scoped query wrappers and report the nearest real source caller, including callers
  in dependency/runtime assemblies. The two previously failing assembly/introspection
  samples now complete. Legacy discovery and missing-reference faults remain; AOT
  introspection still rejects RuntimeTypeHandle and is not added by this fix.

- Lower a bounded Double instruction profile in ARM64 AOT: literals, typed storage,
  calls, add/sub/mul/div and ordered/unordered comparisons and branches. Preserve
  binary64 lanes, signed zero and NaN behavior; Double values are not GC roots.
  Math constants now pass interpreted, sanitized native and standalone execution;
  HTTP admission remains accepted. Single, conversions, remainder/negation and
  floating Math services remain unsupported; no complete floating IL claim.

- Adopt assembly-level member terminology for types, functions and constants, whose
  qualified names include namespaces. Rename the unreleased constant APIs to
  AssemblyConstantDefinition / AddConstant / Constants and the manifest member to
  assemblies[].constants. Rebuild same-day prototype consumers and artifacts; no
  compatibility aliases for the previous names. Add a common assembly member view
  to authored/read metadata and the introspection facade, preserving kinds, assembly
  ownership, qualified names and overload signatures. Constants attach to one graph;
  nested and type-owned declarations retain their owners. Values and lookup remain.

- Add development System.Math assembly-level constants Pi, E and Tau with .NET-equivalent
  Double bits. Native assembly-level constants retain exact values across separate
  compilation and interpreted execution without runtime storage. AOT admission still
  rejects Double instructions; native execution is pending. The initial
  public/internal finite-Double metadata contract rejects malformed/duplicate values;
  older readers require matching updated compiler/runtime bundles. Constants inline
  into consumers, which must be recompiled if values change. Refresh API coverage;
  standalone CLI projection remains unsupported and rejects explicitly.

- Remove the competing System.Math placeholder from the explicit source-runtime
  bootstrap. Rebuilt native bundles resolve qualified/wildcard Math calls without
  changing Raven/.NET lookup precedence. The unchanged string sample passes
  interpreted, sanitized native and standalone execution with exact expected output.
  Older Core.dll inputs must be regenerated; the temporary CLI bootstrap remains.

- Bind exact native PathCombine and PathGetFileName services with explicit
  --bind-paths opt-in on macOS ARM64. Preserve Unix lexical UTF-8 behavior, immutable
  owned results, GC roots and unchanged outputs on failure; no filesystem access or
  public API changes. Add 33 interpreter/native comparisons, contract and sanitized
  allocation/ownership checks. The unchanged Raven path sample now passes interpreted,
  sanitized native and standalone execution; HTTP compiler admission remains accepted.

- Fix Raven portable emission of discarded awaits by retaining statement-boundary
  control flow. Add a native success consumer and remove the named-local workaround
  from the async callback-fault sample. Nested expressions retain their operand guard;
  runtime contracts and metadata formats are unchanged. Raven `2c8c1f9de` passes
  67 focused compiler controls and three native/interpreter consumer comparisons.
  Four .NET controls also pass unchanged on Raven main; its portable backend is absent.
  Strengthen the consumer verifier to compare standalone output/fault text with the
  interpreter as well.

- Add bounded native `DrainEntryTasks` support for queue-only async entry points.
  Keep startup roots live through callbacks, release the entry-drain guard on faults,
  and reject entry pumping with host socket completions. The separate quiescent host
  pump and callback-driven HTTP path retain their existing contracts. Add focused
  lifecycle/contract checks and five executable comparisons, including success,
  pending/cancelled entry faults and callback faults with GC cleanup. This is development support,
  not native host waiting, runtime suspension or a general Scheduler.

- Replace the legacy union interface `IUnion` with `UnionValue`, coordinated with
  Raven `be58723fb` and completing the
  unprefixed runtime interface audit across Raven source and neoIL. Regenerate 27
  union slices and matching API references. The CLI importer now translates exact
  union-constructor field initialization and typed null output stores. Refreshed
  `TryGetValue` failures clear outputs to defaults, matching the existing native/Raven
  contract; legacy callers that expected preservation must migrate. Union and
  time/timezone consumers pass. Raven's general source lookup correction is also
  validated and committed on Raven main as `d0a115dcf`.

- Rename the Raven-source async interfaces to `AsyncStateMachine` and `TaskAwaiter`,
  coordinated with Raven target mapping `185d32f06`; .NET names remain unchanged.
  Rebuild matched compiler/reference/runtime/application bundles because metadata
  identities change. Refresh the async bridge fragments and API snapshot, and guard
  all runtime source interfaces against the `I` prefix. Native closed class-interface
  callbacks retain receivers through GC and preserve fault frames; state, cancellation
  and task-result samples now match interpreted execution. Host-backed entry task draining remains
  unsupported in AOT. Validation and limitations are recorded in the sample assessment.

- Admit native 64-bit division/remainder with operand-width overflow guards and
  unchanged zero/overflow fault behavior. Verify 72 native/interpreter cases and
  calendar sample parity; record the remaining time-contract/time-zone boundaries.

- Extend verified native primitive wrappers to Boolean, Int64 and UInt64 with exact
  borrowed receivers, and admit existing 64-bit ordering/branch lowering. Boolean and
  generic-collection Raven samples now match interpreted execution; 56 comparison
  cases, four receiver-mutation checks and a standalone primitive consumer pass.
  General virtual dispatch and wider arithmetic remain separate work.

- Compile bounded scalar arrays for integer, Boolean and Void elements, preserving
  default/reserved initialization, exact element identities, checked access and atomic
  GC payloads. Verify 77 native/interpreter mode/type comparisons and real Raven array
  aliasing/mutation; nine selected consumers advance to separately recorded blockers.
  Element borrows, jagged arrays and nominal/interface views remain unsupported.

- Assess 104 Raven samples with same-artifact interpreted/native execution, recording
  emission and AOT blockers separately. Reassess seven consumers after the array-view
  work: async propagation now matches, path/file blockers map to exact native services,
  and Math lookup, jagged arrays and interpreter introspection remain separate work.
  Record a final-runtime recheck of both introspection faults and prioritize lexical
  path bindings without changing the no-bridge release gate. Preserve a focused callback/HTTP baseline;
  the initial primitive-array priority is now implemented. Keep universal sample parity
  out of the release gate. Record no-bridge bootstrap probes and future IL-free reflection/interop metadata.

- Index native GC allocation ranges per collection, preserving interior roots and
  payload bounds without changing the heap ABI or collection policy. Ten kernel
  tests and HTTP checks pass; same-object native HTTP measurements improve from
  20.08 to 96.05 requests/s in short local runs. Record profiling/stack evidence,
  add hash-checked adapter comparison, and fix stdout read-ahead in the HTTP verifier.

- Add a Raven HTTP consumer serving 32 sequential requests per process and a matched
  interpreter/native validation and measurement mode. Record three short pairs, raw
  request/startup samples and limitations; all responses and native cleanup checks pass.
  Compact oversized command output in HTTP evidence to its hash. Reaffirm the next-release
  native teaser and full compiler-target bootstrap without the .NET bridge as distinct
  requirements; current development Core.dll remains an explicit qualification gap.

- Run the existing Raven HttpServer app natively through a private queue/socket host.
  Check in a same-artifact interpreter/native verifier: greeting, fragmented input,
  duplicate Content-Length and handler rejection match across interpreter, sanitized
  native and standalone ARM64. Validate native cleanup and libSystem-only linkage.
  Document this first HTTP correctness POC as WIP, with no throughput claim.

- Retain verified TaskQueue.Drain as an explicit host compilation root and export a
  private quiescent drain adapter. Validate real Raven nested posting and post-entry
  faults in interpreter/native modes, with root cleanup and standalone linkage.
  Record the author's future Scheduler/green-thread direction and cross-cutting runtime
  services across interpreter, AOT and eventual JIT; this adapter remains provisional.

- Adapt native value-profile no-result and inhabited-Void entry points to result
  zero on success, preserving caller storage on faults. Validate both entry shapes
  with zero/one Int32 argument. With the explicit stack guard, full Raven Server
  now passes compiler admission; native host queue pumping is still required.

- Add a private macOS ARM64 remaining-stack probe and sanitized worker-stack/unwind
  checks at two optimization levels. Record the required compiler/frame/host-entry
  contract and comparison with .NET sufficient-stack checks. Add opt-in guarded
  recursion with final machine/linkage/call-argument bounds and standardized
  StackOverflow faults; validate direct/Function recursion, host callback roots,
  truncated traces, small-stack rejection and reuse after unwind. Default recursion
  rejection remains. Full Server next reaches its unsupported no-result entry.

- Add a private native TaskQueue scope and exact closed service bindings. Retain the
  default queue and preserve active explicit Run/Drain receiver lookup; validate a real
  Raven Promise/queue consumer in both modes plus scope/contract checks. Align native
  wrapper root reporting with the recent socket/text bindings. Full Server now reaches
  the recursive-call guard; automatic queue pumping remains unfinished. Include an
  original owner/member and compiled-index cycle witness in recursion diagnostics,
  exposing the Server close/cancellation/callback cycle without weakening admission.

- Extend the private native socket kernel with bounded send snapshots and rooted receive
  copy-back, partial/EOF results, five-second deadlines and terminal scratch cleanup.
  Share operation slots/polling with accept; rename the private poll helper to
  neoclr_socket_poll_v1 and require the matching text adapter. Sanitized kernel and
  compiled-accept regressions pass. Bind exact CIL transfer services and validate a
  callback-driven native echo with GC between completions; full Raven Server admission
  next reaches SocketDeadlineAfter. Add exact shared-deadline bindings with monotonic
  stamps, expiry-before-submission and no budget refresh across transfers; validate
  native kernel and compiled echo paths. Admission now reaches StringCompareOrdinal.
  Bind StringCompareOrdinal with UTF-8/scalar ordering and validate 88 native/interpreter
  text/null cases; full Server now reaches specialized RegisterTaskQueue.
  Public HTTP execution remains incomplete.

- Add a private nonblocking native accept kernel with retained completion roots,
  deferred single delivery, cancellation/result consumption and scope cleanup. Validate
  real loopback TCP and listener compatibility. GC socket builds now link the GC/root
  helpers. Bind exact Accept/ConnectResult/Cancel CIL services behind --bind-socket-accept;
  validate compiled callback-driven completion and cancellation. Full Raven Server
  admission now reaches SocketReceive; task pumping and native HTTP remain incomplete.

- Add experimental native host dispatch for rooted zero-argument Void callbacks.
  Preserve receivers, heap contents and guest fault traces; reject invalid handles,
  wrong shapes and active-context reentry. This private image-local adapter does not
  establish a stable callable ABI or implement async sockets/runtime suspension.

- Record the author-directed cross-runtime reassessment of Function objects, scheduling
  and GC ownership for future async suspension/green threads. Compare alternatives and
  identify migration tests; no callable semantics or suspension implementation changes.

- Add private bounded native GC host root handles with explicit replacement/release,
  context/thread lookup and entry-reset guards. Validate retention, reclamation, stale
  handles, quotas and compiled entry behavior. GC-enabled native objects require the
  matching entry-check helper; asynchronous callback/socket execution remains WIP.
  Align published-frame validation with the compiler's 1,024-function bound; validate
  the highest supported ID and rejection beyond it in GC and diagnostic modes.

- Record a future HttpMethod abstraction proposal, including custom-token validation,
  case semantics and migration questions. No public API or HTTP capability changes.

- Admit verified Int32 enums in the bounded native value profile, retaining nominal
  identity through calls and reserved array snapshots. Add explicit enum storage
  conversion and Int32 bitwise operations; validate signed extremes and real Raven
  HTTP status/flags collections in both modes. The full server remains blocked by
  the specialization budget. Fix explicit-core inhabited Void imports without requiring
  a nominal seed declaration; preserve value/no-result distinctions and reject wrong
  core bindings. The nested HTTP-result probe now compiles and runs interpreted,
  and now runs natively after extending the checked value/array/GC layout bound to
  64 lanes and sizing call scratch for selected layouts. Validate nested HTTP error
  unions, status payloads, let/if-let matching, copies and GC under replacement.
  Coordinate native selection/boxing/codegen bounds at 256 types, 1,024 functions and
  256 clones, with counted diagnostics and boundary tests. The full server now clears
  selection (208 shapes/570 functions/213 clones). Add checked reference-class array
  slots with nominal typing, strong GC retention, shared-copy identity and distinct
  unwritten/null faults. The real Raven ArrayList<Counter> consumer passes both modes.
  Extend this storage to ordinary null-initialized class arrays; six focused tests and
  two Raven consumers cover defaults, identity, GC, limits and fault parity. The order
  sample advances to its array-to-Iterable cast; value-record defaults, element borrows
  and covariance remain unsupported. Correct byte-array backing selection when another
  element specialization is discovered first; 30 native/interpreter comparisons cover
  mutation, identity, initialization and fault parity, including wrong-element interface
  casts. Reject ordinary class construction for all closed array backings before
  erasure. Extend verified nominal/interface views to closed class-reference arrays
  with exact private type tags and GC validation; the order-collections sample now
  matches interpreted execution. Preserve explicit wrong-element mutable-array cast
  faults and field borrow/replacement restrictions. Correct isinst in the shared
  verifier/interpreter to return null on incompatible array tests, matching CLI semantics
  without admitting covariance. Previously these tests could fail verification or fault.
  Add 15 differential cases, malformed-tag/retention controls and Raven standalone
  checks. No public API or metadata format changes.
  Bind the exact ObjectReferenceEquals service to native identity comparison, including
  nulls and distinct equal-content strings. The full server next reaches the
  StreamError.ToString value-member contract; admit verified direct byref value
  ToString overrides in the private native projection and validate real StreamError
  descriptions in both modes. Project ordinary Int32 instance wrappers with verified
  explicit borrowed receivers; keep arbitrary input borrows unsupported. Validate
  signed-extreme formatting, equality and nested HTTP diagnostics. The full server
  now reaches asynchronous socket-result binding.

- Add development System.Text.StringBuilder with fluent append, atomic byte-limit
  faults, LF line append, clear/reuse and cached immutable snapshots. Add separator-aware
  String.Join for initialized non-null string arrays. Share checked one-allocation
  materialization between interpreter and native C services; retain existing GC owners.
  Admit verified sealed-owner native virtual calls while preserving null checks.
  Also reject that projection when the complete loaded set contains a derived type;
  current sealing metadata is descriptive rather than runtime-enforced.
  Refresh CLI library/reference projections, String constructor/operator admission,
  API documentation and executable samples. Compare construction workloads in both
  modes; the current builder is not a demonstrated performance improvement over Concat.
  This author-selected side quest revisits the earlier deferral; native HTTP remains WIP.

- Add bounded AOT execution of existing Function metadata: static/bound heap callbacks,
  typed stored values, closed generic owners, GC receiver retention and result handoff.
  Preserve interpreter fault diagnostics, including null binding versus null invocation.
  Reject recursive/borrowed/output callback paths and excessive dispatch candidates.
  Add native/interpretive Raven callback samples and sanitized GC/fault tests. The HTTP
  app advances to Function-array admission; task/socket completion and server timings
  remain work in progress. Follow with typed callback-array storage and Function method
  specialization: preserve checked initialization, receiver retention, replacement and
  ArrayList growth/copy behavior. Native/interpreter container and fault checks pass;
  the HTTP app next requires arrays of task-result values. Add checked reserved value-array
  snapshots, initialized-element GC tracing and closed nominal method specialization.
  Validate real ArrayList<Result<int,string>> growth/copy/replacement and union patterns
  in both modes. The full server now reaches the 128-type specialization budget; HTTP
  enum storage and task/socket completion remain open. No public API, Raven compiler
  or bridge contract changes. Record the StringBuilder discussion and existing deferred
  prototype evidence; the subsequent author-selected builder side quest is described above.

- Extend AOT erased transport and closed specialization with Int64/UInt64, preserving
  high bits and exact type tags through generic calls. Admit full-width equality;
  mismatched unpack retains RuntimeError/no-result-publication behavior. Add Raven
  socket-handle and listener probes: the former runs in both modes, while the listener
  advances from UnpackValue<long> rejection to unbound native socket services. Report
  the unsupported generic shape explicitly for the full HTTP app. No interpreter or
  public library/bridge contract changes. Follow with explicit `--bind-socket-listener`
  support for the ordinary Raven Listen/GetLocalPort/Close path. A bounded host scope
  owns opaque handles and closes outstanding listeners after guest success or faults;
  validate typed errors, stale/foreign handles, resource limits and exact fault parity.
  Standalone macOS ARM64 listener samples pass; async sockets/tasks and HTTP remain open.

- Add a native-web benchmark workbench with a checked-in Raven routing workload,
  same-artifact interpreted/native process measurements, provenance and HTTP server
  admission reporting. Add an ASP.NET Core greeting comparison candidate; matched
  persistent-server performance comparisons remain future work. Add a “Native
  compilation” feature page, homepage box and guide navigation, explicitly marking
  the POC as work in progress, including in a future release, with benchmark and
  native API/GC/deployment caveats. Correct existing proposal type formatting and
  fault-guide experiment links exposed by website validation.

- Add a bounded nonmoving native mark-and-sweep kernel with in-buffer allocation
  descriptors, free-block reuse/coalescing, interior-owner retention, conservative
  object scanning, atomic text/bytes and initialized String-array tracing. Sanitized
  contract coverage exercises cycles, aliases, fault roots and live-set exhaustion.
  Enable it in generated ARM64 code with explicit `--native-gc --reference-arena`,
  distinct GC hooks and matching statically linked adapters. Collection occurs at complete
  pre-operation boundaries, never inside allocation services. Validate an interior-only
  owner, erased return handoff, 512 discarded cycles, fault retention/reentry and live-set
  exhaustion. The real Raven routing workload now completes 1,024 requests in a fixed
  64 KiB buffer (arena-only previously failed at 128), retaining patterns/captures and
  matching interpreter output/faults. Default arena/diagnostic modes remain unchanged.
  This is conservative, nonmoving, synchronous experimental GC with collection at every
  boundary; precise object maps, pressure scheduling and general host handles remain open.
  Replace repeated heap marking passes with an in-header worklist, preserving the memory
  budget and clearing temporary links on descriptor failure. Add a reproducible paired
  benchmark: reverse-chain marking improves about 2.9× locally; Raven routing is effectively
  unchanged. Interpreter tracing already uses a worklist; no interpreter change is needed.

- Add private typed tracing-layout diagnostics to AOT inspection for admitted value
  profiles: distinguish managed references from native-width integers, preserve nested
  field offsets, identify conditional erased String payloads and owner-dependent borrows,
  and describe reference-object payloads. Validate cycles and nominal byte-array aliases.
  These are storage recipes only; emitted descriptors, safepoint roots and native
  reclamation remain future work. Follow up by clearing traceable local storage and
  erased discriminators in ordinary native function prologues, while retaining verifier
  rejection of unassigned reads. Inspection reports the selected local seed lanes.
  This adds initialization stores, not root registration or collection; runtime ABI
  and scalar-only local initialization are unchanged. Add reachable pre-operation root
  plans for calls, constructors, arrays and text materialization, distinguishing retained
  stack values from pending operands and selecting spill lanes including erased tags.
  Report native adapter/dispatch bodies as explicit coverage gaps. Add opt-in
  `--probe-stack-roots` emission of initialized stack snapshots and private JSON plans
  to a synchronous linked diagnostic callback. Validate actual ARM64 values across
  nested calls, faults and reentry; default emission has no probe dependency. This is
  read-only observation. Extend it with stack-owned diagnostic frames linked per thread,
  host-context identity, readable suspended caller snapshots and cleanup before every
  generated return, including propagated faults. The private probe callback advances to
  v2 and requires matching enter/leave adapters; entry ABI v3/v4 is unchanged. Root
  coverage remains incomplete and collection stays disabled. Publish typed argument/local
  lane addresses in diagnostic frames, including seeded unassigned local roots; observe
  later writes without following borrowed pointees. Read discriminators as 32 bits to
  avoid uninitialized padding. Frames grow to 72 bytes and the private enter adapter
  advances to v2; diagnostic hosts/images must be relinked together. Extend diagnostics
  to initialized constructor storage and successful call/constructor results, using live
  slot addresses and explicit activation phases. Faulted calls publish no result roots.
  Frames now grow to 104 bytes with enter-v3 and a transient callback; next-operation
  snapshots retire prior transient roots. Collection and adapter-internal root coverage
  remain disabled/incomplete. Close the ordinary-call diagnostic result handoff gap by
  seeding and registering caller-owned traceable result lanes before invocation, keeping
  the callee's output writes visible during frame removal. Pending scratch is distinct
  from successful guest-result publication; faults never activate the success phase.
  The private transient callback advances to v2 for the new pending phase. Extend
  diagnostic frames to native service and dispatch wrappers, publishing typed argument
  copies and unlinking on every generated return, including early faults. Synthetic
  frames stay out of guest fault traces. Service-internal temporaries remain uncovered;
  frame/callback ABI and default uninstrumented emission are unchanged. Add bounded
  read-only enumeration of fault-context message/frame-name slots during diagnostic
  unwinding and after host return, including arena-backed user messages. Code-zero
  contexts expose no stale frame slots; malformed counts and short output tables are
  rejected without partial publication. Validate independent contexts, reentry and
  interpreter fault-rendering parity. This is not host root registration or collection.

- Add a real RoutePattern workload covering eight repeated routing outcomes, pattern
  errors and capture retention. Measure native arena growth from 2,433 bytes before
  requests to 117,507 bytes at 128 requests; a fixed 64 KiB invocation faults without
  publishing a result. Interpreter diagnostics demonstrate reclamation during the same
  source workload. Record typed native roots/descriptors and nonmoving tracing as the
  next integration work; no native collector or public API is added by this slice.

- Add experimental ARM64 AOT String-array storage, discovered by compiling the real
  RoutePattern library consumer. Preserve aliases, immutable String owners, default
  nulls, checked reservation state and source faults; allocation failure publishes no
  result. Validate 54 Console and 45 value tests, a standalone Raven array consumer,
  and the real RoutePattern.Parse/Match/GetInt32 path with exact output/fault parity.
  Executables retain only libSystem as a dynamic dependency.
  Keep element borrows, value arrays and nominal String-array views excluded. Record
  the author's HTTP-driven dependency direction, including GC integration if needed;
  the invocation arena is not a final server memory policy. No compiler, public API,
  metadata or native context ABI change.

- Extend experimental ARM64 AOT UTF-8 bindings with exact String ContainsOrdinal,
  StartsWithOrdinal and EndsWithOrdinal predicates. Preserve empty-pattern, Unicode,
  embedded-NUL and null-fault behavior without allocation. Add a bounded Console
  request-line consumer with an application Route union and Int32 item IDs; this is
  an HTTP-oriented parsing experiment, not a server or general protocol parser.
  Validate 53 Console tests, including 60 native/interpreter predicate comparisons,
  and 18 standalone input streams with exact output/fault parity and only libSystem
  dynamically linked.
  Public APIs, Raven contracts, metadata and native context ABI are unchanged.

- Extend experimental ARM64 AOT integer-text binding with exact Int32 parsing and
  static Int32 wrapper projection. Preserve whole-text ASCII grammar, malformed-before-
  overflow precedence, allocation-free parsing and null-fault diagnostics. Add a bounded
  repeated Console input sample using Parse/Result/Option patterns; validate 52 Console
  tests, including 154 parser comparisons, and 11 standalone input streams with exact
  output/fault parity and no shared managed framework. Correct the latest roadmap/proposal notes:
  line-oriented native Console input already had standalone coverage. Public APIs,
  compiler contracts, metadata and native context ABI are unchanged.

- Compile typed String ceq in experimental ARM64 AOT with exact UTF-8 content and
  null comparison, distinct from reference identity. Equality allocates no storage;
  ordinary Raven ==/!= wrappers compile unchanged. Compile verified String interface
  implementations, including EquatableTo<string>.Equals, through an explicit raw
  receiver projection. Keep class dispatch and Object display separate, preserve
  callee/null fault traces, and include implicit targets in recursion admission.
  Validate 50 Console and 45 value tests plus standalone Unicode/NUL/dynamic-text
  equality and interface equality with exact output-fault parity. Borrowed receivers
  remain unsupported. No new service binding, compiler contract or ABI change.

- Admit verified String interface views in experimental ARM64 AOT. Retain original
  closed conformance before private projection, including inherited generic interfaces;
  preserve String/Object identity and null/cast behavior. Validate 48 Console AOT tests
  and a standalone Raven EquatableTo<string> round-trip with exact output/fault parity.
  Ordinary String interface method dispatch is covered by the follow-up above. No compiler,
  public API, metadata format or context ABI change.

- Preserve fresh Char-to-String identity in experimental ARM64 AOT when String
  identity is observable. Reuse bounded arena copying while keeping unobservable
  conversions allocation-free. Validate 47 Console AOT tests, exact allocation/cast/null
  fault sites, and standalone Raven character output with integer interpolation across
  seven Unicode/NUL cases and broken-pipe fault parity. String interfaces remain
  unsupported; compiler, target configuration, public APIs and ABI layout are unchanged.

- Add private String Object views to experimental ARM64 AOT. Preserve aliases,
  distinct literal evaluations, Object display, String round trips, nulls and invalid
  cast diagnostics. Materialize literals in the invocation arena when text identity
  is observable; upcasts reuse storage. Validate 46 Console AOT tests and fresh Raven
  mixed interpolation with exact standalone output and broken-pipe fault parity.
  Keep String interface casts and CharText identity producers explicitly unsupported;
  context ABI, adapter signatures, Raven configuration and public APIs are unchanged.

- Align interpreted String `ref.eq` with the existing shared-owner identity contract:
  accept direct Strings and preserve identity through separate Object wrappers.
  Distinct equal literal evaluations remain distinct; null comparisons stay false
  against non-null text. Add a failing-before regression and run six String ownership,
  conversion and GC tests. No interning, metadata or public API changes.

- Extend experimental ARM64 AOT with distinct copied Int32 boxes and verified
  Object.ToString dispatch through the explicit Int32 formatter binding. Preserve
  Object views, identity, result publication and exact managed fault sites under
  boxing/formatting exhaustion. Validate 44 Console AOT tests and a fresh standalone
  Raven Console.WriteLine(Object) consumer across six Int32 values. Mixed interpolation
  now reaches the separate unsupported String-to-Object view boundary; wider boxing,
  unboxing and general Object metadata remain outside this profile. No public ABI,
  Raven compiler, Runtime Contract or runtime-library API change.

- Qualify Raven's shared synthesized-concat conversion fix on main and the native
  integration branch: fresh native CIL preserves integer endpoints and null text;
  text-only interpolation runs as a standalone ARM64 executable with UTF-8/NUL parity
  and only libSystem linked dynamically. Record compiler/bundle hashes and executable
  isolation. Keep boxed Object display explicitly rejected by AOT without publishing
  an object file; this remains a native profile gap, not a Raven emission failure.

- Extend experimental ARM64 AOT with explicit native Console.ReadByte binding while
  compiling the ordinary Raven wrapper and its Result/Option branches. Link stdin
  support into the executable; preserve all bytes, EOF, unavailable/read-failed
  outcomes and managed faults for invalid adapter statuses. Validate exact service
  admission, all 256 bytes and the standalone Raven input consumer. This remains
  a bounded development profile; dynamic text and the rest of Console are pending.
  Preserve caller-module InternalCall identities before flattening AOT load sets,
  matching interpreter linking when the seed and source library both declare the
  same service. Original load-set verification still controls access and signatures;
  managed duplicate names gain no new local preference.
  Add explicit value-profile WriteLine binding: compile ordinary string, Boolean
  and empty-line Console wrappers, preserve UTF-8/NUL output, and propagate output
  failures with shared diagnostics. The standalone host converts broken pipes into
  I/O faults. Validate a fresh Raven interactive union app and exact interpreter/
  native output, exit and broken-pipe fault parity. Add Int32 formatting through an
  explicit caller-owned text arena (experimental ABI v4), enabling ordinary integer/
  byte WriteLine wrappers. Preserve text across nested calls and dynamic fault
  messages; bounded exhaustion reports NativeMemoryLimitExceeded. Arena text expires
  on the next invocation or buffer release; render faults first. This is not general
  managed memory/GC support. Validate endpoint formatting, allocation bounds, context
  reuse and a standalone numeric Raven input/output app. Extend copied value records
  and closed type-generic payloads to carry String pointers with mixed I32/I64 lanes,
  aligned private storage, null defaults and exact String pattern tests. Guard null
  native text arguments with interpreter-compatible fault diagnostics. Validate nested
  copies/interior output borrows and fresh Raven Some/None<string> patterns. General
  reference objects initially remained unsupported; a following explicit
  `--reference-arena` slice admits bounded nongeneric classes, constructors, shared
  fields, output copies and identity in ABI v4. Preserve aliases/self-cycles until
  region reset and match interpreter null-field fault traces; validate exhaustion,
  repeated reuse and a fresh Raven class/Console consumer. There is no per-object GC,
  inheritance, virtual/interface dispatch or array support yet. Support CIL callvirt
  to exact nonvirtual class members under the reference-arena opt-in, including its
  call-site null check. Preserve the distinct direct-call fault location and reject
  genuine virtual/interface dispatch, including bodyless interface declarations.
  Add zero-initialized packed byte arrays to the explicit invocation-arena profile,
  with aliasing, length, indexed loads/stores and one-byte interior borrows. Match
  interpreter null/index/negative-length faults; report bounded array/native-memory
  exhaustion without publishing a result. Validate empty arrays, byte-store canaries
  and a fresh standalone Raven array/Console consumer. Other array element types
  and stream/interface dispatch remain pending. Add explicit character text bindings
  and immutable grapheme transport through locals, calls and output borrows, preserving
  NUL defaults and exact invalid-input fault diagnostics. Statically link the same
  pinned Unicode segmentation implementation as the interpreter; validate fresh
  Raven Console.WriteLine(char) output for ASCII, accents, combining marks and emoji.
  Original primitive-owner methods remain verified before private static-wrapper
  lowering. Character fields, erasure and general text operations remain unsupported.
  Add SByte/Int16/UInt16 storage and unchecked narrowing with signed/unsigned loads,
  retaining Int32 stack arithmetic and existing numeric text formatting. Validate
  48 boundary cases across fields, borrows, parameters and returns, plus a fresh
  standalone Raven small-integer Console sample. Checked narrowing and wider
  integer formatting initially remained outside that slice. Add explicit signed and
  unsigned 64-bit formatting/native-width conversion bindings, UInt32/64-bit/native
  storage, unchecked integer conversions and wrapping Int64 add/subtract/multiply.
  Preserve full-width bits, typed copies and shared arena lifetime/exhaustion rules.
  Validate numeric endpoints, wrapping, signed versus unsigned widening and a fresh
  Raven Console app, including native-width defaults. Wide checked arithmetic,
  division and general native-pointer operations remain unsupported. Specialize
  multiple closed value shapes with distinct layouts, member bodies and private
  metadata tokens, retaining original identities/type arguments in diagnostics and
  reports. Preserve access/readonly verification and enforce 32 selected shapes,
  32 function clones and 128 selected functions. Validate cloned members/faults,
  discovery bounds and a standalone Raven app combining several Option/Result
  shapes, Error/None patterns and Console.ReadByte. Add identity-preserving nongeneric
  interface views and checked casts to the reference arena, retaining interpreter
  null/mismatch behavior and fault locations. Privately project the verified empty
  Object base while compiling its ordinary constructor; validate fresh standalone
  Console.OpenStandardInput/Output/Error factories. General inheritance, Object
  virtual slots and interface method dispatch initially remained pending. Add bounded
  implicit interface dispatch for reachable constructed classes, forwarding results
  and faults without synthetic managed frames. Keep unused implementations excluded
  and preserve original conformance/access checks. Extend the private budget to
  64 types and sixteen value lanes for StreamError, with matching call-result storage
  and boundary tests. Compile ordinary standard-input Read/Close methods and validate
  byte, EOF, zero-count, invalid-range and closed-stream outcomes. Add explicit raw
  stdout/stderr Write/Flush service bindings, preserving byte counts, ranges, limits,
  NULs, recoverable I/O results and null fault frames. Match line/stderr flush policy
  and reject invalid host counts. Admit inhabited unit storage for Result<unit,...>,
  distinct from no-result methods. Validate ordinary output-stream interfaces,
  handled broken-pipe errors, adapter boundaries and unit/adjacent-field copies.
  Specialize closed generic reference classes/interfaces with distinct object tags
  and exact closed member signatures. Preserve inherited interface dispatch and
  identity-preserving upcasts; verify original contracts before generic projection.
  Validate two generic class shapes, mutations and exact null/cast fault parity.
  Add checked reserved byte capacity with shared publication markers, interpreter-
  compatible unreadable-slot faults and native output initialization checks. Keep
  ordinary array defaults and range/zero-count service behavior; reject reserved-
  array element borrows until addresses carry initialization state. Validate ordinary
  ArrayList<byte> growth and inherited Sequence views in a Console consumer. Privately
  rename verified static/empty seed Console owners to avoid backend seed collisions,
  preserving their shapes and source diagnostics. Align the object allocator with
  sixteen-field admission and test exact-fit/exhaustion/canaries. Extend private
  erased-value transport to I64 payloads for String pointers while preserving
  primitive normalization, exact tags and mismatch faults. Admit String static
  generic helpers; validate UTF-8/NUL, null/empty and arena-backed payloads through
  output copies and fault messages. Add exact opt-in UTF-8 byte-count/slice services,
  preserving range/boundary results, null faults, embedded NUL and invocation-owned
  slice lifetimes. Validate 135 interpreter/native cases and allocation bounds,
  failure publication and canaries. Bind UTF-8 encoding to immutable byte-value
  snapshots with indexed reads, ordinary managed-array copying and fixed-extent
  replacement checks. Validate snapshots, null/empty input, limits and exhaustion;
  reject mutable element borrows/defaults. General object erasure and managed-array
  interface views remain pending; internal native bodies must be rebuilt together.
  Compile verified ordinary String instance wrappers via an explicit-receiver private
  projection, retaining bodies and fault identities. Validate a standalone Raven
  byte-count/slice Result consumer and null argument behavior. Extend measured
  selection budgets to 256 functions/128 shapes/64 clones with boundary rejection
  tests. Add bounded empty-record boxing with distinct reference identity, Object
  views and original allocation-fault sites; generated helpers count toward budgets.
  Validate ordinary EncoderState.HasValue through a fresh standalone Raven Console
  consumer, exact broken-pipe diagnostics and focused identity/resource checks.
  Payload boxing/unboxing remain outside the profile. Preserve verified nominal
  byte-array backing identities through specialization and interface dispatch;
  ordinary Count/indexer/iterator bodies keep aliasing and reserved initialization.
  Compile Console.Write and standard-error text writers with UTF-8/NUL and ordinary
  encoder loops; validate standalone output and broken-pipe user-fault parity.
  Reject corrupt backing metadata and class-style array allocation.
  Compile Console.ReadLine with strict UTF-8 decoding through its existing reader,
  decoder and Result/Option CIL. Preserve malformed-input outcomes, initialized-slot
  fault precedence, NUL text and allocation-failure publication. Extend measured
  private bounds to 512 functions/128 clones and 32 flattened value lanes, with
  matching 256-byte call results and 264-byte object allocation; retain 128 types
  and 16 direct fields. Validate decoder parity and storage/selection boundaries,
  plus ten standalone reader inputs and exact broken-pipe fault/exit parity.
  Validate default and consecutive Console line reads, typed range/closed errors
  and non-owning reader close behavior with six standalone session inputs. Preserve
  successful producer command/hash evidence and report unresolved source-path
  missing-member diagnostics separately.
  Add bounded virtual Object.ToString dispatch for Console.WriteLine(object?)
  on classes with verified explicit overrides. Preserve receiver identity, null
  checks and original override fault frames; expose exact dispatch targets in the
  compilation report. Keep default/base display, deeper inheritance, arrays and
  boxed receivers outside this slice and reject unsupported combinations. Reject
  native reference constructors without a leading base initializer and direct
  constructor calls outside that initializer, preventing an interpreter/native
  construction-fault mismatch. Validate standalone override/null Console output,
  exact output-fault parity, rootless/declared-base dispatch and source-name
  independence. Retain the producer lookup limitation in the evidence.
  Extend the explicit UTF-8 binding with immutable String concatenation and compile
  ordinary static String wrappers. Preserve NUL/UTF-8, null and overflow faults,
  arena limits and failure publication; test copied/aliased inputs and boundaries.
  Add a Console.In line/remainder consumer for ReadToEnd, including EOF, typed
  errors and byte limits. Record opt-in isolated producer staging and verify the
  preserved compilation hash when reusing that artifact. Verify nine standalone
  reader cases, exact output-fault parity, 43 Console tests and 18 linking tests;
  the executable runs alone with only the macOS system-library dependency.
  Investigate Raven producer failures on codex/source-object-metadata-resolution:
  qualify shared assembly/namespace/import lookup fixes also applied to Raven main.
  Exercise imported Console names in the reader sample and hash adjacent compiler
  implementations/configuration before accepting compilation reuse. Fresh Raven-to-AOT
  compilation passes all nine reader cases and exact fault/exit parity, without staging
  or reuse and with only the macOS system-library dependency.

### 2026-10-07

- Investigate future ARM64-first JIT/AOT execution, retaining interpretation and
  treating hot reload as a separate capability. Record primary-source comparisons,
  native runtime gaps, backend alternatives and proposed AOT/reload experiments;
  align architecture, roadmap and website proposal guidance. Record an AOT web app
  as the motivating POC, starting with scalar/Hello World steps before HTTP, and
  plan later reproducible benchmarks against .NET and selected other platforms.
  The research does not implement general compilation or hot reload or permanently
  replace Raven priorities. Following the author's instruction to continue AOT,
  add an isolated Cranelift 0.121.2 scalar compiler emitting macOS ARM64 objects
  with a C-callable Int32 export. Validate wrapping arithmetic/direct calls against
  the interpreter, unsupported-input rejection, object format and no runtime imports.
  Extend the next slice with Int32 locals, loops, early returns, signed/unsigned
  branches and operand-stack joins. Seven focused tests on main cover 79 native/
  interpreter comparisons and 17 rejected programs, including uninitialized locals
  and invalid control flow. Add a separate arithmetic-Fault slice: checked signed/
  unsigned arithmetic and division/remainder return explicit status through compiled
  calls, preserving first-fault behavior and leaving output untouched on failure.
  The experimental C export becomes neoclr_entry_v2 (status plus result pointer);
  rebuild probe objects and C hosts together. Nine tests cover 174 native/interpreter
  comparisons. Complete the next executable slice as ARM64 Hello World: lower UTF-8
  literals through an explicit console capability, generate position-independent
  code and link native startup/output services into the executable. Check exact
  bytes, OS-only dynamic dependencies, execution without companion files and service
  failures. Record the author's early self-contained CIL-to-native foundation and
  Hello World-to-HTTP Server sequence, with trimming recorded as a later step.
  Clarify the input as neoCLR CIL: compile Raven Hello World to native PE/#Neo
  metadata/IL, decode it directly, select its declared entry, and compile/link it
  into the standalone ARM64 executable. Support standalone NEOX too, retaining
  neoIL conformance inputs. Add a parameterless Int32 entry adapter and checked
  bundled Console.WriteLine(String) lowering. Record the pinned compiler bootstrap,
  fixture and reproducible pipeline; fifteen focused tests pass, rejecting corrupt
  containers and unsupported IL before emission. Record types/members and value-type
  union consumers as the author-selected next milestone. Inventory emitted Counter
  and Some/None types/members with interpreter validation and explicit AOT rejection;
  record a reference-counting versus tracing experiment proposal without changing
  managed lifetime policy. Implement the first flat-value AOT slice: compile Raven
  Counter and Int32/Boolean record-copy samples with constructors, fields, property
  accessors and borrowed instance receivers. Preserve copied arguments/results and
  stack joins, default initialization and Fault output rules; keep the v2 C entry.
  Record private aggregate ABI limits, native/interpreter checks and independent
  executable deployment. Extend the following slice with bounded nested record layouts,
  aggregate field copies and deep interior borrows, rejecting inline cycles and oversized
  payloads. Validate a Raven Envelope consumer as an independent ARM64 executable and
  nested copy/alias operations against the interpreter. Union-based console input is
  the next sample direction. Add ordinary output-parameter calls with conservative
  whole-slot definite-assignment checks, preserving forwarding, nested aliases and
  Fault propagation. Validate Raven output members and record the pinned producer
  forwarding workaround; conditional output contracts remain deferred. Add Byte
  storage and unchecked conv.u1/conv.i4 lowering for union tags, preserving truncation
  at storage/call boundaries and Int32 stack semantics. Validate a standalone Raven
  tag sample and native/interpreter boundary inputs; the aggregate ABI remains private.
  Resolve overloaded value members by exact signature and optional definition identity,
  rejecting mismatches and ambiguous roots. Validate Raven overloads, constructors and
  nominal output payloads. Record existing private native symbol names and future stable
  mangling requirements without introducing a linking ABI. Add read-only AOT metadata/
  call/opcode inspection with actual compiler admission and no output image; retain the
  generated union fixture to expose remaining requirements. Record future metadata-next-
  to-image ABI exploration and the author-directed union-app completion milestone.
  Add explicit --closed-world direct-call selection with reported exclusions and original
  identity mappings. Preserve selected-body verification and access checks; ordinary
  whole-module admission remains the default. This limited code-selection step is not
  a general metadata/reflection-aware trimmer. Preserve source access/readonly facts
  and reject foreign assembly origins in the selected verification projection. Complete
  a standalone Raven Some/None app with construction, both patterns, non-matches and
  copied payloads; record fresh producer/interpreter/native evidence and isolated
  deployment. Recommend a Result-driven interactive integer reader as the next consumer.
  Add bounded closed-world specialization for one closed instantiation per local value
  type definition, preserving nominal case identities and semantic origin checks. Compile
  a Raven ParseResult<int, byte> app with success/error payloads and retain empty static
  lexical companions as metadata only. Validate let-pattern-else and if-let success/miss
  paths in interpreter and standalone ARM64 samples. Record the pinned Raven emitter's
  plain positional let-deconstruction rejection with a reproducible negative probe.
  Generic methods, constraints, multiple instantiations and reference payloads remain
  rejected; 46 focused AOT tests pass. Add read-only closed-world admission inspection
  using the same selection/specialization preparation as emission, with phase-specific
  failures and exact rejected call references. Validate the real System.Result<int, byte>
  consumer through Raven and the interpreter, recording expected external-dependency
  rejection before object creation and its propagation-interface contract. Runtime-library Result AOT
  remains unsupported; an explicit assembly-aware value-library load set is the next
  proposed implementation slice. Five inspection and 28 value-profile tests pass.
  Implement the first explicit nongeneric value-library load set with trailing --module
  inputs. Verify original module scopes with the runtime binder/verifier before canonical
  projection; preserve original identities in selection reports and reject cross-module
  access violations. Compile a separate Raven Counter library/application into one ARM64
  executable with no input DLLs at execution time. Add PE/NEOX, interpreter parity and
  invalid dependency/access tests. Extend the verified load-set path with one closed
  generic value instantiation per type definition, preserving library identities in
  specialization reports. Compile a separate Pair<int, byte> library consumer with
  copied values, accessors and default initialization; validate both native containers
  and reject additional shapes, reference payloads and cross-module generic access.
  Add explicit --system/--object-root validation contexts for closed-world emission and
  inspection, preserving the runtime's original identity, root-slot and access checks.
  Permit verified unselected generic methods while retaining selected-method rejection.
  Validate a standalone generic value app against the pinned runtime-owned context; the
  real library Result initially stops at an explicit implemented-interface diagnostic.
  The next slice verifies original interface conformance, reports source identities and
  specialized relationships, then omits metadata-only relationships from the private
  direct-call projection. The actual System.Result<int, byte> consumer runs standalone
  on ARM64, covering success/error `if let` and `let ... else` branches. Reject malformed
  implementations, interface storage and dead dispatch. A follow-up moves verified
  metadata-only relationships ahead of executable specialization, reporting full closed
  interface arguments without consuming native shape limits. Compile the real nested
  Result<Option<byte>, ConsoleReadError> outcome model, covering zero/high bytes, EOF
  and both errors. Record the actual Console.ReadByte interpreter probe and its current
  initial static-owner AOT boundary. Admit nongeneric empty abstract sealed owners for
  direct static calls while rejecting their use as values/instance receivers and retaining
  access checks. Validate a Raven static factory returning the nested input result.
  ReadByte now reaches its erased Value payload; native input services remain unimplemented.
  Record an intermittent pinned Raven ReadByte binding diagnostic; reuse the unchanged
  producer fixture for backend validation rather than claiming a compiler fix.
  Add bounded primitive System.Value transport with exact Int32/Byte/Boolean/Void tags,
  copied locals/calls/results/output slots and pack/test/unpack lowering. Incorrect unpack
  returns RuntimeError without publishing a result; defaults, reference/record payloads,
  nested erasure and Value fields remain rejected. Validate standalone CIL transport and
  advance the unchanged ReadByte probe to its selected generic-helper boundary. Add
  bounded primitive static generic-method specialization, exact call binding and distinct
  private bodies with original identities/origins reported. Validate Raven forwarding in
  PE/NEOX, cross-library method shapes and clone/identity/access rejection. ReadByte now
  exposes managed helper bodies in the System seed. Add explicit `--compile-system`
  selection of managed seed bodies, preserving original verification and identity reports;
  the default remains validation only. Validate actual primitive seed helpers in an isolated
  native executable and reject duplicate/missing context, nonpublic access and native
  InternalCall services. ReadByte now reaches its String-based failure path. Lower explicit
  fault instructions in scalar/value code to experimental ABI status 4 (UserFault),
  preserving first-fault propagation and untouched export results. Fault paths may end
  before output assignment; normal returns retain assignment checks. Validate interpreter
  parity and an isolated native consumer. Hosts naming statuses should recognize 4;
  fault messages/stack traces and String-based System.Fail remain unsupported in AOT.
  Carry immutable UTF-8 literals through value-profile arguments, results, locals, output
  slots and control-flow joins using private read-only image data. Validate a fresh Raven
  producer in PE/NEOX and CIL transport including empty/embedded-NUL text; defaults,
  fields, erasure, generic String arguments and dynamic text remain rejected. ReadByte
  now passes String signature selection and stops at its native ConsoleReadByte service.
  Add shared FaultCode runtime messages and a borrowed FaultDiagnostic host view;
  legacy diagnostics remain compatible. Add opt-in AOT ABI v3 caller-owned fault records,
  UTF-8 messages, managed traces with the interpreter's 64-frame bound/truncation, and
  a linked C renderer. Bind exact supplied native failure services explicitly while
  compiling ordinary System.Fail wrappers. Standalone unhandled faults exit 1; hosts
  receive code/message/trace and control their own policy. Validate cross-backend
  diagnostics and a standalone Raven failure app. Route interpreter CLI execution faults
  through the shared renderer, matching native code/message/trace formatting and exit 1;
  successful program exit values remain intact. Loader/verifier diagnostics and legacy
  Fault Display stay compatible. CLI text consumers must adopt the new execution format. No native
  reference services, interface execution or stable native hosting ABI are added.
  General unions, reference fields and general library dependency compilation remain
  unsupported. Loop budgets/cancellation,
  managed allocation and HTTP remain future work. Runtime
  dependencies and public library APIs are unchanged. Cherry-pick this
  isolated slice to main at the author's correction, retaining main's newer native
  bootstrap work and refreshing the tool lockfile for the current runtime.

- Merge native System bootstrap into main while retaining the newer shared RavenDoc
  publisher/navigation. Continue development on main; Windows native toolchain and
  installed-editor qualification remain open release gates. The merged tree passes
  165 metadata contracts, 21 website/readiness tests and the 1,815-page site build.

- Refresh RavenDoc to Raven main `a00fa5ee6` so shared API navigation fills the
  full mobile viewport when opened as a drawer. This follows the Raven 0.1.14
  release as a site-only fix; desktop navigation keeps its reserved height.
  All 18 website contract tests and the full 1,804-page build pass. Browser
  validation on ComparableTo<T> confirms full-height drawers at 844px and 650px.

- Opt into RavenDoc's shared API navigation to avoid repeating the full tree on
  every reference page. Normal static pages and no-JavaScript namespace links
  remain available; the shared generator keeps inline navigation as its default.
  Pin Raven main `d1fe391b9`, including versioned session caching and stable
  sidebar space to avoid loading flicker. All 18 contract tests and the full
  1,804-page build pass; browser review confirms active selection after loading.

- Configure RavenDoc assembly inputs as a list sharing one API namespace tree.
  Keep the current CoreProbe snapshot's real identity; future split snapshots
  can join the same tree. Refresh shared source links, static-type presentation,
  cross-source references and nested-type navigation with Raven main `bb93da2da`.
  All 18 website contract tests pass; the full build checks 1,803 pages. Browser
  review confirms static Console metadata, GitHub source links and member grouping.

- Refresh RavenDoc to Raven main `7ad0f5817` and record the .NET 11 runtime-async
  unit payload fix carried to the integration branch. Preserve generic unit
  results and discard awaited payloads in statement position; 45 focused tests
  pass on both branches. Native async qualification remains separate.

- Refresh RavenDoc to Raven main `8aa4cba6d` and record the shared sealed-case
  member emission fix on the integration branch. Constructed member owners use
  actual emitted generic arity; ordinary nested types retain enclosing arguments.
  This repairs existing CLI metadata without changing native contracts. Include
  the shared Task/ValueTask unit-return binder fix; 24 unit/await and 25 case/owner
  tests pass on main and the integration branch.

- Refresh the shared RavenDoc compiler to Raven main `86c8dfbfc` and document the
  generic callback inference fix carried to the neoCLR integration branch. Match
  arms determine unresolved callback result types while lexical generic targets
  remain authoritative. Runtime Contract and CLI bridge encodings are unchanged;
  validation covers modern .NET and documentation generation, not native execution.

- Add GitHub source-file links to API type/member pages through configurable shared
  RavenDoc repository settings and Raven declaration indexing. Preserve assembly
  names and omit links where no matching source declaration exists. Validate the
  full site, 18 website tests and mobile type/member navigation.

- Update RavenDoc with a mobile three-dot navbar menu beside theme/search controls
  and stationary copy buttons during horizontal code scrolling. Validate all 18
  website tests, generated pages and both themes at mobile/tablet/desktop widths.

- Record the shared Raven heap-async resumption fix on main and the source-object
  metadata integration branch. Seventeen focused .NET tests pass; native runtime
  behavior and Runtime Contract configuration are unchanged. See the target
  compilation guide for the release-gate evidence and validation limits.

- Refresh RavenDoc so wide tables scroll inside mobile articles without widening
  the page. Rebuild the site and validate all 18 website tests.

- Update RavenDoc with explicit navigation section boundaries and sidebar titles.
  This supports distinct Getting started and Language reference areas in Raven
  while preserving neoCLR's existing navigation configuration.

- Cross-check RavenDoc site-wide navigation support against neoCLR and explicitly
  retain section navigation here. Raven can share one documentation hierarchy
  without changing neoCLR's intentional API browser boundaries.

- Update RavenDoc with macro-partition navigation fixes, a dedicated article
  sidebar with collapsible sections, configurable API source display names and
  a compact mobile header. Retain library links when no authored menu is supplied.
  Preserve assembly filenames in symbol metadata and neoCLR API navigation.

- Refresh RavenDoc with integrated multi-library website support and floating
  copy controls that do not push signature or sample text downward. Keep neoCLR
  namespace navigation flat and preserve the existing single-API configuration.

- Update the pinned RavenDoc publisher and enable site-wide icon search and code
  copying. Include generated and manual API articles in search, list ordinary
  nested types separately from union cases, and add authored namespace guidance.
  Namespace comments now round-trip through Raven documentation sidecars; the
  neoCLR reference snapshot, Runtime Contract and native/CLI encoding are unchanged.
  Validate 1,803 pages and local links/anchors, 18 website tests, and mobile/desktop
  browser behavior in both themes.

- Require Windows qualification for the next native metadata release. Add a manual
  Windows x64 compiler/bootstrap/library/package execution workflow; qualification and
  installed-editor acceptance remain pending. Make HTTP pipe readiness portable and
  decode runtime verification output as UTF-8 rather than the host code page.
- Simplify installation to one matched native workflow, add Raven to the main navigation
  with website/playground links, and remove superseded setup and integration chronology.
  Preserve actual download availability and concrete limitations. Cover the protected
  Attribute constructor with a manual reference because the pinned renderer omits it,
  and repair stale API-guide and maintainer-evidence links.

- Record independent-checkout macOS arm64 bootstrap/distribution qualification from
  pinned NeoCLR/Raven sources. Document explicit compatible Apple SDK selection and
  sequential shared-project builds after preserving initial toolchain/reference-output
  failures. No runtime/compiler semantics changed; other platforms and final release
  publication are not implied by this local evidence. Nine extracted compile/run
  commands and all 26 installed-package editor checks pass.

- Prepare primitive Core and source-owned retained-seed inputs from checked-in sources
  without old Numbers/Http artifacts. Record generator commands/hashes and validate
  prepared bootstrap inputs before class-library builds. This removes temporary input
  prerequisites, not the permitted CLI primitive bootstrap or retained runtime seed.
- Record the native RavenDoc provider direction: preserve actual declaring assembly
  identities through the existing native compiler-symbol adapter, then extract a shared
  documentation model incrementally. Provider implementation and website migration
  were subsequently deferred by the author for release focus, with a possible Raven
  rewrite left as future exploration. CoreProbe ownership has not been relabelled;
  the provider/model migration is not an upcoming release gate.

- Teach native POC packaging to consume the split class-library bundle with its
  generated help, exact producer checks and relative SDK/project configuration.
  Read runtime modules from the archive manifest during extracted verification;
  retain explicit legacy aggregate-bundle support. This is development packaging,
  not a release publication or completion of Platform/API-documentation work.

- Ship generated XML and Markdown help with each source-built native class-library
  assembly, validate documentation identity before publishing the bundle, and hash
  nested sidecar files. Extend relocation/editor checks to verify native IPAddress
  help. This transports existing comments; missing API help and unified RavenDoc
  generation remain open, including the current empty Data documentation output.

- Extend installed VS Code acceptance to the split native class-library bundle,
  including optional-library navigation and imported configuration recovery. Record
  client-side hover timings and repeated/unsaved-edit Main.rvn checks; persistent
  hover latency reported by the author remains under investigation. No compiler
  performance fix or complete SDK qualification is claimed.

- Emit a relocatable NeoCLR.ClassLibrary.props in staged bundles, with native references
  and matching Core/seed/ownership selections shared by compiler and editor project
  loading (Raven e93fcfdc1). Test symbol owners, missing/conflicting dependencies and HTTP execution from
  a relocated bundle with spaces in its path. Record the still-internal networking
  service boundary; no new public Platform API is introduced.

- Add native Data, Networking and Web projects referencing the Runtime foundation.
  Build and stage the four libraries with explicit seed finalization and prebuilt
  dependency imports (Raven 8fbacaa9f); publish a hashed bundle manifest only after
  successful builds. Five source-free consumers and the project HTTP consumer execute.
  Preserve rejection of stale/missing bootstrap inputs. Platform projects, API-doc
  bundling and the complete editor/toolchain distribution remain separate release gates.

- Add the checked-in native System.Runtime project and its explicit ownership manifest
  for the 175-source Runtime foundation. Qualify project compilation, retained-seed
  finalization and unchanged source-free orders execution. Higher-level project layouts
  and Platform extraction remain open; the legacy bridge project is retained separately.

- Qualify Raven 3892b113a dependency-first native ProjectReference builds with a four-project diamond,
  transitive native imports, generic object mutation/identity, exact execution output,
  cycle rejection and failed-output preservation. Record the prebuilt workspace boundary
  and remaining class-library project/editor packaging gates.

- Qualify native project import/run against separate source-built libraries using Raven
  0f85f53b8's explicit Object-owner selection. Add a reproducible unchanged HTTP project
  consumer and invalid-owner output-preservation check. Native ProjectReference build
  orchestration remains the next slice; no release or VS Code gate is claimed complete.

- Record Raven e141006f3's canonical bootstrap Void binding correction: two deterministic
  lookup regressions, six fresh Web compilations and five source-free executable consumers
  pass without metadata fallbacks. Preserve commands, artifact hashes and prior failures.
  Hand the separate general async unit-return fix to the author-designated Raven release
  task; retain native project/editor/artifact qualification as bootstrap work ahead.

- Add development NetworkDeadline and five typed DNS/socket deadline overloads, keeping
  raw stamps internal and the default expired. Separate Web consumes these native APIs
  with its existing exchange budget. Add source-free executable acceptance, C# visibility
  checks, API reference and website documentation. New bridge declarations support docs
  only; legacy CLI-to-neoIL translation does not support the typed deadline contract.
  Record passing consumers and loopback cancellation alongside an unresolved intermittent
  System.Void clean-build failure; bootstrap/main merge remains gated on qualification.

- Record planned real library/Platform project outputs and RavenDoc assembly bundles.
  Specify shared declaration provenance and exact revision-pinned source links for
  .NET/NeoCLR API documentation, with Source Link/PDB comparison and validation gates.
  These are requirements, not implemented project or documentation capabilities.

- Extend optional-library auditing with a separate Web build against emitted Runtime,
  Data and Networking references. Record private deadline API failures and absence of
  output. Document the author-directed unified RavenDoc class-library reference with
  declaring-assembly provenance; native multi-assembly documentation migration remains
  planned, not implemented. Keep separate package ownership and one-way dependencies.

- Add development ArrayReflection.GetLength/GetValue/Create for supported managed
  vectors, with terminal checked faults and preserved reference identity. JSON mapping
  consumes this public boundary across separate Runtime/Data assemblies. Align API
  declarations, XML reference, snapshot and website limits. Preserve canonical external
  Object ownership for ordinary GetType references; add C# and source-free execution
  regressions. Default runtime instruction limits remain unchanged.

- Validate external-root boxing against the complete authored Object slot contract;
  reject incomplete contracts before encoding. Separate System.Networking now compiles
  and runs the unchanged cancellation consumer against System.Runtime (exit 0).
  Correct optional-library auditing to enable explicit bootstrap intrinsics: the earlier
  CheckedStorage failure was missing build configuration, not a required new mapping.
  Add reproducible compiler/verify/run acceptance and C# incomplete-contract coverage.

- Add explicit selected-root Object slot references with exact signature and virtual
  dispatch validation. Source-free Raven dispatch through an object receiver reaches
  derived ToString/GetHashCode/Equals overrides (42); the API fixture also executes
  imported Equals dispatch. All 165 metadata groups pass. Networking now reaches a
  CheckedStorage ownership mapping gap rather than unsupported Object calls.

- Preserve selected imported System.Value ownership when mapping bootstrap helper
  signatures, writing native storage aliases and matching retained runtime services.
  C# introspection round trips retain exact external identity; a Raven parse/type-test/
  unpack consumer verifies and returns 42. Networking now reaches imported virtual
  Object.ToString support; optional-library execution remains open.

- Add explicit external native Object authoring selection, shared by manual definitions,
  builders and method references. Preserve scoped identity through existing native aliases
  and CLI Object reference signatures; conflicting owners reject. API-authored and Raven
  Equals consumers verify/run against source-built Runtime with exit 42. All 165 metadata
  groups pass; Networking now reaches a separate System.Value encoding blocker.

- Record the proposed distinction between managed assembly ownership, optional library
  packages, platform runtime payloads and developer tooling. Keep package IDs/layouts
  provisional and bootstrap inputs explicit; no package resolver or release split is
  implemented. Detail imported-root authoring as the next end-to-end dependency.

- Integrate Raven eab5b3e7d: native flags-enum marker lookup remains on the explicit
  primitive bootstrap when Object belongs to an imported Runtime. Record separate
  Data/Networking compilation: Data now reaches internal array-reflection dependencies;
  Networking rejects imported-root overrides. Neither failed build publishes output.

- Build the 174-input System.Runtime candidate without Data, Networking, Web or
  network adapters. Its source-free orders consumer verifies and executes with exact
  output/exit 0, retaining explicit primitive and service bootstrap dependencies.
  Record reproducible packaging evidence; optional assembly builds remain next.

- Establish the native source-owned orders gate: all 197 aggregate library inputs
  emit, and unchanged application-order-collections imports that artifact, compiles,
  verifies and executes with exact output/exit 0. Raven explicitly selects the imported
  Object owner; primitive bootstrap and finalized retained seed remain required.
  Add reproducible acceptance tooling and update the development website status.

- Integrate Raven's native erased Value import fix: preserve nominal semantic identity
  instead of parsing a nonexistent CLR special type. The unchanged orders consumer
  now reaches ArrayList emission validation; reconcile imported source-root ownership
  next. Record focused symbol/ownership tests and compiler/artifact hashes.

- Finalize the full-source audit's runtime seed with an explicit revisioned dependency
  read from the emitted native artifact. The combined load set verifies 2,433 IL
  functions and executes a control application. Invalid/missing/duplicate inputs
  publish no translated seed; wrong runtime revisions reject. Ordinary application
  import next exposes Raven's native System.Value classification gap.

- Encode the explicitly selected source/imported System.Void as canonical native unit
  storage, including callbacks and generic arguments. Keep no-result calling conventions
  distinct without introducing another language type. Metadata round trips and separate
  Raven NativeMemory consumers execute; ordinary .NET unit tests remain green.

- Preserve separately scoped unit-value and no-result declarations of runtime
  services when the registry explicitly admits both. Native WriteLine consumers
  execute with their own stack contracts; ordinary and same-module conflicts still
  reject. Full-System loading next reaches source-unit callback signature admission.

- Remove the competing bootstrap Object from the full-source audit's retained native
  seed. All 197 inputs still emit; admission advances to the retained/source WriteLine
  result-contract conflict, now identified with declaration and return details.
  Add a read-only emitted-dependency inventory and staged Data/Networking/Web split
  plan; no production assembly split is claimed.

- Add required native library schema 4 with a 16 MiB envelope; smaller libraries
  retain schema 3 and its 8 MiB limit. JSON/node/depth and total PE bounds stay
  unchanged. All 197 System inputs now emit; runtime admission next rejects the
  source Array backing contract. Full bootstrap is not yet complete.

- Preserve intrinsic String inheritance from the selected source Object root in
  native metadata and runtime constructor execution, retaining UTF-8 text storage.
  Linked PE execution passes; full-System encoding next reaches the payload limit.

- Align array backing validation across authoring, reading and runtime linking:
  allow the explicitly selected fieldless source Object base, while rejecting
  other/unselected bases. Managed-array allocation and alias mutation execute in
  linked PE assemblies. Full-System emission next reaches String/Object inheritance.

- Support explicitly owned System.Value runtime storage in metadata definitions,
  native signatures and introspection while retaining nominal CLI signatures.
  Source Environment payload type tests/unpacking execute through retained helpers;
  malformed carrier storage rejects. Raven's full-System audit advances to native
  array backing-storage validation. Generic helpers remain an explicit seed dependency.

- Route parameterless reflection construction through source-owned native service
  declarations, retaining constructor execution and access checks. Preserve the
  temporary CLI bridge with validated facade bindings. The full-source audit omits
  the seed-only GetType extension and advances to System.Value ownership validation.

- Route source Object.GetType through an explicit native handle facade instead of
  the bootstrap RuntimeServices signature. Source-owned Object/RuntimeTypeHandle
  verification and identity/hash checks execute; the full-System audit advances
  to ReflectionConstruct dependency resolution. Public API signatures are unchanged.

- Add closed-class builder authoring over an owned local base, including source
  Object, with existing definition validation and native encoding. Raven preserves
  that base; protected constructor chaining and virtual dispatch execute successfully.
  The full-System audit advances to ObjectTypeHandle dependency-contract resolution.

- Verify early source-unit ownership through generic interface dispatch in a separately
  compiled native consumer, alongside NativeMemory success/fault checks and the legacy
  bootstrap control. Raven no longer caches a bootstrap Void before source declarations
  exist. The full System audit advances to direct-base constructor validation.

- Verify source/native unit ownership with production Void and NativeMemory sources:
  a separate native consumer executes allocation/free and an inhabited unit parameter;
  expected overflow/double-free faults and unsupported-pointer rejection pass. Retain
  the ordinary-bootstrap control. The full audit now selects source Void explicitly
  and omits the seed copy; encoding still exposes a residual bootstrap Void reference.

- Add the source-built abstract System.Attribute base with a protected constructor;
  retain UnionAttribute inheritance and reuse its constructor for union metadata.
  Source and embedded marker gates execute, metadata identity checks pass, and invalid
  markers publish no output. The full System frontier advances to NativeMemory's
  pointer-to-source-Void signature. API reference shape and summaries are refreshed.

- Verify bootstrap FlagsAttribute and MethodImpl(InternalCall) with a source-owned
  Object root after Raven's exact-core identity fix. Production BindingFlags and an
  attributed runtime call execute; same-named source attributes reject before output.
  Ordinary-bootstrap NativeMemory execution/fault checks still pass. Full System now
  stops at UnionAttribute's unsupported external System.Attribute base.

- Allow top-level generic reference classes to inherit the explicitly authored native
  Object root. Definition/builder validation, constructor flow and native reading retain
  the existing CLI/native base encoding; general generic inheritance stays unsupported.
  An API-authored case and a Raven source-root fixture execute generic storage and
  inherited dispatch. The full-System audit passes Array<T> and next stops on enum
  attribute ownership; full System and ordinary imported-root consumers remain pending.

- Connect source NativeMemory to exact native allocation services through Raven's
  native pointer import/emission path. Separately compiled consumers execute both
  Alloc overloads, Free, typed pointer pass-through, double-free and overflow faults;
  unsupported string pointers reject before output. Extend metadata module-function
  reference authoring to pointer signatures. Full System now clears binding across
  194 inputs, then stops at generic classes inheriting the source Object root.

- Add bounded unmanaged pointer signatures to the .NET metadata API, using ordinary
  CLI PTR encoding and the existing native Ptr category. Callable readers, import,
  canonical introspection and argument/local/call/return validation preserve pointer
  targets. Reject unsupported pointer targets, generic arguments and Function shapes.
  All 162 C# groups pass; an API-authored allocation/free consumer executes with exit
  42. Raven mapping and source NativeMemory compilation remain pending.

- Add exact native allocation InternalCalls for the pending source NativeAllocation
  adapter: UIntPtr byte allocation, checked size multiplication and no-result void-pointer
  release. They share the existing pointer heap, initialization tracking and limits;
  legacy heap instructions remain unchanged. Five native-container service tests and
  17 pointer regressions pass. C# pointer metadata/Raven authoring remains to be connected;
  this slice does not reduce the four remaining full-System binding errors.

- Select source/native System.Fail ownership through Raven's explicit failure contract.
  Local and separately imported let-else calls now compile and execute on both present
  and absent paths; wrong owners reject without publishing an assembly. The full-System
  audit drops from 12 to four binding errors, all NativeAllocation. No runtime or
  metadata format change; complete System emission has not yet been reached.

### 2026-10-06

- Compile unchanged System.Fail with a native RuntimeFailure adapter and execute it
  from a separate metadata consumer. The exact no-result neoCLR.Runtime.Fail service
  raises UserFault; legacy inhabited-unit Fault remains available for existing seeds.
  An explicit primitive profile omits the competing Fail declaration. Five fault tests,
  signature controls and the source gate pass. Full-System binding now has 12 errors;
  source-owned terminal-flow recognition and NativeAllocation remain open.

- Execute unchanged source-built Console from a separate native metadata consumer,
  using an explicit primitive core and seed without competing Console declarations.
  Preserve both legacy unit and source no-result WriteLine conventions; blocking host
  call completion now respects the callee's result convention. Verify UTF-8 input/output,
  EOF, numeric overloads, stderr and independent wrapper closure. Full-System binding
  diagnostics drop from 30 to 14; this is not a complete System bootstrap.

- Build unchanged IntPtr/UIntPtr sources with explicit native primitive ownership and
  execute their comparison methods from an artifact-only consumer. Add exact-signature
  signed/unsigned widening services without changing Raven cast rules. Raven a6ee91610
  connects import/emission; its general default-receiver correction is isolated in
  24c2c4d40. Nine runtime, 161 metadata and eight focused .NET/portable tests pass.
  Full-System diagnostics drop from 35 to 30; Console service integration remains open.

- Add host metadata IntPtr/UIntPtr signature categories and ILGenerator Conv_I/Conv_U
  using standard CLI encodings and existing native runtime types. Method/field/property
  readers and introspection retain them. All 161 C# groups pass; the generated native
  consumer verifies and exits 42. Raven mappings and source ownership remain pending;
  this prerequisite does not yet unblock Console or change the full-System baseline.

- Compile unchanged calendar/time-zone sources through five native service adapters.
  A separately compiled consumer executes DST mappings, offsets, errors and optional
  local/zoned union conversions with Raven befb8c1e1. TimeZoneMapLocal returns managed
  Int64 arrays with allocation limits; legacy transport remains compatible. Full-System
  diagnostics drop from 48 to 35. Prefer specific temporal types in feature/navigation
  and API guidance; DateTime is an optional local-or-zoned contract, not the main model.

- Compile unchanged Environment sources with three native adapters and execute an
  artifact-only consumer covering argument snapshots, Unicode, cwd and variable states.
  EnvironmentArguments now admits managed string-array results with array/heap limits;
  the old internal transport remains compatible. Four runtime tests pass. Full-System
  diagnostics drop from 51 to 48; Console/native-width and other bootstrap work remains.

- Record Raven merge `7bfc6ad27` bringing synchronous use cleanup onto the continuing
  `codex/source-object-metadata-resolution` branch. The merged line passes 92 focused
  .NET checks and six native cleanup executions; native async cleanup stays deferred.

- Preserve explicit callable nullable annotations in the host metadata definitions,
  builders, CLI writer/reader and introspection facade. C# tests check .NET interpretation,
  execution identity and malformed payloads. Native encoding rejects these annotations
  until its matching support exists; this first step did not fix native KeepAlive(null).
  Follow-up: native callable origins now preserve and validate the same explicit facts,
  including binary/introspection round trips and CLI projection. Older readers reject
  annotated artifacts; matching development tools are required. Raven `d19c6e4a3` now
  preserves these facts through symbols and emission. Both artifact-only GC consumers
  compile, verify and execute, including KeepAlive(null); 17 .NET checks and seven native
  consumers pass. Nullable context/field support remains out of scope.

- Document companion Raven synchronous `use` support through a shared disposal
  contract: reverse-order cleanup on block completion, return, propagation and loop
  exits, with no exception regions. Six authored-protocol native consumers verify
  and execute; async/iterator cleanup and bare bootstrap-interface implementation
  remain outside this slice. Runtime instructions and implementation are unchanged.

- Add native source GC adapters and admit no-result Collect/KeepAlive service calls
  without pushing a unit value. Existing unit callers remain supported. Nine GC
  tests and an artifact-only retention consumer pass. Preserve KeepAlive(null) as
  an explicit failing native-import regression: nullable parameter metadata remains
  unsupported, so the complete GC API gate is still open.

- Compile unchanged Math sources through native adapters for the existing 15 floating
  services. An artifact-only consumer executes all 20 public functions, including
  rounding and Result failures. Record the explicit namespace alias needed with the
  retained bootstrap Math type; full-System diagnostics drop from 74 to 59.

- Add a reproducible post-release System bootstrap inventory, distinguishing omitted
  build inputs from compiler failures. Existing Storage adapters and six omitted
  contract files compile. Isolate and fix Raven's PE-only metadata assumption for a
  source Object root (41 focused tests). Fix union source-Object signature completion
  with four declaration-order regressions; 206 existing tests remain green. Full-source
  compilation now reports diagnostics instead of crashing. Prioritize handle ownership
  and missing runtime-service declarations; no full System assembly is published.
  A follow-up reuses the already implemented source-handle contract in the audit,
  removing its configuration and conversion cascades without a new compiler fix.

### 2026-10-05

- Point repository entry guides to Preview 12 native installation and distinguish
  the preserved older bridge walkthroughs. Record the passing hosted Preview 12
  canonical, OS boundary and minimum-Rust gates. Record main integration, Preview 12
  publication, verified public archive checksum and the deployed website/API checks.

## 0.1.0-preview.12 — 2026-10-05

Native Raven metadata POC for macOS arm64. Selected source-built libraries and
artifact-only applications, Tasks, JSON/HTTP and installed VS Code API help are
qualified; this is not full System bootstrap. See [release notes](docs/preview-12-release-notes.md)
and [validation](docs/preview-12-validation.json). Historical development entries
below preserve their original scope and remaining limitations.

### 2026-10-05

- Prepare author-selected Preview 12 for macOS arm64, preserving the bounded native
  POC scope. Select an explicit native source-validation profile in CI, retaining
  the default legacy audit separately. Pin the retained bootstrap seed source and reproduce both bootstrap
  artifacts byte-for-byte. Apply Rust formatting and equivalent Clippy cleanups,
  and update the source audit to the renamed function-object sample. Retain the
  Rust 1.85-compatible atomic update API with narrowly scoped deprecation allowances
  for newer toolchains. Publication
  and main integration are tracked separately.

- Package the native POC tooling, source-built libraries and explicit bootstrap
  dependencies with relative consumer projects and hash verification. Extracted
  compilation/execution and installed VSIX acceptance pass on macOS arm64. Record
  local candidate setup and provenance limits; no release publication is claimed.

- Update RavenDoc so callback types use function signatures in member lists and
  parameter tables, including Task.Run and String's FlatMap extension. Preserve
  nested type links and keep parameter identifiers on one line. The pinned shared
  fix passes 20 RavenDoc tests and 18 website checks; publication remains separate.

- Rebuild the POC class-library subset directly from Raven into native metadata,
  with checked-in ownership and explicit bootstrap inputs. Qualify artifact-only
  consumers using the extracted SDK: 15 compile, 13 non-network executions and
  both live HTTP/JSON rounds pass. Keep legacy bridge snapshot failures separate;
  full native bootstrap and release packaging remain open.

- Correct two stale release-test expectations for the already-supported no-result
  reflection setter and current native-service rejection diagnostic. The full runtime
  run exposed these assertions; all ten affected tests pass after correction.

- Label generated website/API pages as a development snapshot while retaining links
  to published Preview 11. Record release qualification of documentation rendering;
  no new release or website deployment is claimed. Update pinned RavenDoc to the
  independently tested Raven-main union-case documentation fix, restoring authored
  case summaries. Extend editor preparation to extracted SDKs and adjacent sidecars;
  qualify the matched SDK and installed VSIX with 19 checks, including Option/Result
  API help. Record the remaining source-bootstrap/distribution blockers explicitly.

- Repair the source release notice inventory for fifteen locked metadata/CBOR and
  hashing dependencies, preserving their shipped license texts and checksums.
  This clears the first extracted-source release audit blocker; it is not release
  publication or full candidate qualification.

- Extend native editor acceptance to imported API documentation: generated Raven
  Markdown/XML sidecars, hover prose, documentation-only refresh and XML fallback.
  Document the distinction between concise IDE API help and website-only guides;
  matched release packaging and installation qualification remain pending.

- Qualify the native development workflow in real VS Code on macOS arm64: project
  import, completion/hover, read-only declarations, artifact refresh, missing-dependency
  recovery, build failure safety and native execution. The unchanged collections and
  Tasks/await applications run against source-built libraries, with a .NET editor
  control. Add reproducible workspace preparation and extension-host acceptance tools.
  This is POC acceptance; packaging, matched downloads and publication remain pending.

- Connect explicit native Raven project references to adapter-enabled language-server
  builds. Verify imported-method completion, hover and semantic diagnostics over stdio,
  with C# project-loading checks and twelve ordinary project regressions. Native
  reference refresh, declaration navigation and VS Code build/run remain pending.

- Prepare native editor integration by moving Raven's explicit dependency loading
  into a reusable reference catalog. Validate immutable snapshots across artifact
  replacement, matching importer/emitter identities and failure-before-publication.
  Native inheritance/collections and ordinary .NET controls remain green. This does
  not yet connect native project files, the language server or VS Code build/run.

- Complete the unchanged native inheritance sample with ordinary abstract/virtual
  class slots, exact local overrides, direct base calls and inherited interface
  conformance. Preserve CLI slot flags and resolve encoded interface contracts to
  runtime virtual members through validated origins. Definition/builder C# tests and
  CLR/native execution pass; all ten original POC samples now have execution evidence.
  General external/generic virtual hierarchies remain unsupported. Editor release
  qualification and publication remain pending.

- Add ordinary abstract class authoring through definitions and builders. Preserve
  CLI Abstract independently of Sealed/static and native closed-family metadata in
  native readers and introspection. Validate concrete subclass construction and
  inherited fields on CLR and neoCLR; reject direct abstract instantiation. This is
  an inheritance prerequisite, not yet general abstract/virtual method support.

- Close the release reference-producer Option binding regression through Raven's lexical
  union case lookup repair, independently integrated on main. Validate cold semantic
  queries and separately compiled union factories on both targets. The repaired native
  compiler line regenerates the website reference byte-for-byte; no metadata/runtime
  changes or snapshot refresh are needed. Record matching dependency evidence.

- Refresh the website's native Raven target status and homepage development milestone.
  Replace superseded integration notes with current sample evidence and explicit native
  limits; retain published bridge entry-point guidance. Record website, setup and sample
  download review as release requirements. Regenerate the API snapshot with Raven main,
  document the parameter-array marker, repair the Fail link, generic-signature markup
  and metadata-reference routes. Record the integration branch's Option projection
  regression separately. No release or site publication is implied.

- Record the independently reproduced and repaired Raven main field-return regression
  discovered during native HTTP work. Preserve receiver order and original object
  identity across RHS control flow; integrate the compiler fix and retire its temporary
  branch. Native inheritance and editor/LSP acceptance remain release gates.

- Validate native value auto-property construction through the unchanged types sample:
  reference identity, struct mutation and collection copies produce the expected output.
  Keep constructor receiver checks intact; Raven initializes only owned auto-property
  backing fields directly. Nine of ten original POC samples now compile and execute;
  inheritance remains the sample blocker. Record dual-target source-value evidence.

- Record Raven's explicit native Task/builder/state-machine provider and first native
  async emission gate. Add a driver/runtime regression for completed/pending awaits,
  hoisted local preservation and cancellation; the unchanged cancellation sample also
  compiles and runs with exact output. Top-level nongeneric async functions
  execute, including nongeneric class methods with preserved nested ownership and
  private receiver mutation. Generic methods and async entry completion remain pending. Reuse
  existing runtime and metadata encodings, preserving the default .NET backend. Prove
  immutable hoisted Promise capture identity; the async sample now reaches the entry
  signature blocker instead of failing local storage emission. Native async Result
  propagation now preserves control flow and receiver storage across suspension; unchanged
  HTTP JSON client/server samples compile and execute together over localhost. Add an
  executable propagation regression and native HTTP runner with hashed evidence. Native
  Task<unit>/Task<int> entry completion now forwards arguments and preserves exit status
  and faults. Eight of ten original POC samples compile; all eleven non-network execution
  controls pass. Remaining sample blockers are value constructors and inheritance.

- Inventory ten unchanged POC samples through the native compiler driver. Order
  collections, interfaces and JSON object mapping compile and execute with exact output;
  five samples fail native Task recognition, with constructor and inheritance gaps in
  the other two. Add a reproducible inventory runner and hashed evidence; prioritize
  native async integration over full Object-root replacement for the retained-seed POC.

- Expose explicit Object-root loading through `--object-root <module-input>` with an
  explicit `--system` seed. Reuse exact module/revision/type-row validation, reject
  unregistered or incompatible roots before assembly publication, and keep default
  loading unchanged. An ordinary Raven driver-produced root library executes through
  the runtime CLI; 30 focused runtime/CLI checks pass. Raven consumer import is pending.
  Clarify release scope: an end-to-end NeoCLR POC with sample compilation, not exhaustive
  platform or class-library completion.

- Add development `neoclr disassemble <metadata-input> [output]` for PE/#Neo,
  NEOX and format-5 JSON. Show declaration metadata and indexed instructions without
  resolving dependencies or executing. Preserve existing files and reject malformed
  input before publication. The diagnostic listing is not reassemblable neoIL and
  does not inspect ordinary .NET assemblies. Six new checks and nine CLI regressions pass.

- Execute a Raven-emitted source Object root and derived override in NeoCLR using
  explicit host selection. Record Raven `246e8a5db`, bootstrap/artifact hashes, 47 focused
  compiler checks and 15 runtime identity checks. Driver/consumer ownership and generic
  reference bases remain pending. The author selects metadata disassembly as the next
  task before resuming broader end-to-end integration.

- Support Object overrides against an explicitly authored local root, preserving
  Virtual/reused-slot flags and exact Equals identity through native introspection.
  Reject bootstrap/foreign root arguments and stale signatures after root selection.
  API-produced constructors and all three overrides execute in NeoCLR; 158 C# groups
  and 14 runtime identity tests pass. Raven root declaration/mapping guards remain.

- Select explicitly authored Object roots for metadata boxing/value-test stack results
  through `AssemblyBuilder.ObjectType`, preserving `CoreObjectType` bootstrap semantics.
  API-authored boxing and virtual dispatch execute without a legacy System binding;
  incomplete roots and mixed bootstrap return signatures reject. 157 C# groups pass.
  Raven source-root emission remains guarded; no runtime format or .NET default change.

- Add concrete native Object virtual-slot authoring with definition/builder parity,
  exact Equals root identity and preserved CLI Virtual/NewSlot flags. API-produced
  metadata loads and executes all three Object slots plus boxed display in NeoCLR.
  157 C# metadata groups and 13 runtime root-identity tests pass. Production source-root
  compiler wiring and the CLI/VS Code gate remain pending; no native format change.
  Record working Tasks/await samples as a release requirement; runtime suspension and
  green threads remain deferred.

- Add explicit native Object-root declaration authoring through definitions and builders,
  with a baseless CLI reference projection and standard Object signature encoding.
  Native readers preserve canonical declaration facts without granting runtime admission.
  All 156 C# metadata groups pass; root slots, compiler wiring and production ownership
  remain pending. Clarify that the end-to-end gate includes language server and VS Code.

- Record Raven's opt-in source Object semantic binding: one baseless root supplies
  named/keyword signatures, implicit bases and overrides. 75 focused compiler tests
  and the native emission-boundary probe pass. Root metadata authoring remains open;
  both current emitters reject the option before publication. No runtime behavior or
  production seed changes in this integration record.

- Add explicit host selection of an Object root by library module/revision/type row,
  with slot validation, binary round trips and checked retained-seed dependencies.
  Default loading still rejects application lookalikes. Selection is transient;
  no metadata format changes. Rust Module literals must migrate to readers because
  Module now has private context. Source Object binding/writer/CLI integration remains
  pending; 86 runtime regressions and all-target compilation pass.

- Preserve assembly identities for matching runtime-service InternalCall declarations,
  binding symbolic calls locally while retaining signature and access validation.
  Add native source Object equality/identity adapters and an artifact-only executable
  contract test. Source Object no longer lacks those services; canonical root ownership
  remains unresolved and is not claimed complete.

- Record the author-directed release requirement for native metadata-backed editor/LSP
  support and compiler emission from editor builds, with .NET regression coverage.
  A read-only disassembler remains a release candidate; neither tooling feature is
  claimed implemented by this planning update.

- Support an explicitly source-owned, fieldless RuntimeTypeHandle declaration through
  metadata definitions/builders, native reading and introspection. Raven preserves
  canonical handle signatures and rejects duplicate seed ownership. The combined
  140-production-source library compiles and separate JSON/Tasks consumers execute;
  source Object ownership and remaining core service contracts are still open.

- Preserve System.Array<T> element arguments during inheritance and Object member
  dispatch, fixing default array display, identity equality and stable hashing.
  Reassess the post-HTTP compilation frontier: 139 production sources plus nine
  adapters compile together; separate JSON/Tasks consumers execute against the artifact.
  The full 166-source build still rejects core ownership/service gaps before publication.

- Establish the native source HTTP execution gate: compile the complete HTTP source
  group and run seven artifact-only consumers plus cancellation, status/server and
  eleven stream-upload cases. Add native modes to existing harnesses and record
  command/hash evidence. Correct stale scalar-JSON and EOF-body fixture expectations.
  Native async emission and full-System bootstrap remain open; no performance or
  release completion is claimed.

- Accept a single String vector on metadata entry points and materialize managed
  user arguments at native startup, excluding argv[0] while Environment retains it.
  Enforce array/heap limits and reject ambiguous/unsupported entry signatures.
  C# CLI/native round trips and runtime argument tests pass; all eleven unchanged
  stream-upload cases execute through native artifacts.

- Dispatch boxed Int32-backed enums through Object formatting, equality and hashing.
  Formatting uses existing metadata names/flags/numeric fallback; equality retains
  nominal enum identity. Native HTTP status checks and focused enum regressions pass.

- Preserve lexical nesting for native closures capturing their enclosing reference
  object. Resolve private member access through enclosing type identities; unrelated
  types gain no access. The existing HTTP server serves eight valid status cases and
  rejects two invalid responses; a receiver-identity consumer and access tests pass.

- Bind instantiated generic methods as native function values and CLI delegates using
  existing generic call identities. Validate substituted signatures, scopes and
  constraints; C# tests cover CLI execution, native reading and incompatible targets.
  The complete HTTP source group now emits and artifact-only header/base consumers run.

- Advance HTTP source compilation past Result propagation by fixing shared Raven
  nested visitor dispatch and short-circuit condition lowering. A native artifact-only
  consumer executes eight skip/success/error cases; .NET regressions pass. HTTP next
  rejects callback emission; full-library execution is not claimed complete.

- Record the author-directed post-bootstrap release plan: benchmark equivalent
  neoCLR/.NET programs and publish reproducible methodology and results on the website
  with the release. No performance results or publication are claimed yet.

- Compile the unchanged DNS/socket sources into a native library using 21 explicit
  service declarations. Execute the existing loopback cancellation fixture through
  artifact-only references, including DNS, cancellation, transfer mutation and resource
  reuse. DnsAddresses now shares bounded managed-array snapshot materialization with
  StorageNames; existing legacy neoil helpers remain compatible. HTTP propagation is
  the next demonstrated emission blocker, not complete networking/web support.

- Compile the remaining eleven storage sources with explicit native adapters and
  execute the unchanged storage sample from emitted artifact references. Materialize
  directory-name snapshots into managed Array<String> storage with array/heap limits.
  Permit external top-level value types in authored module-function signatures,
  enabling imported FileText Result returns. Exact stdout, file bytes, bounded I/O
  and errors are checked; full-System and dual-target completion remain open.

- Align metadata authoring and native reading with the existing 4,096-row CLI
  TypeDef budget: permit 4,095 declared types, reserving the module row. Manual
  definitions and builders share the bound; existing byte/member/signature budgets
  remain. The 115-file cumulative library with Tasks/Concurrency now emits,
  and both JSON consumers plus the existing task consumer execute against it.

- Verify Raven's explicit source primitive member selection: cumulative encoding,
  streams and JSON now compile into one 109-file library, and both artifact-only JSON
  consumers execute against it. Record the next exposed 256-type metadata authoring
  limit when Tasks/Concurrency are added; storage/network service gaps remain open.

- Reassess full-System compilation after JSON mapping with a reproducible source/import
  audit. The 75-file baseline and imported Tasks compile; cumulative encoding/streams/
  JSON isolate a source String.SliceUtf8 lookup failure. Record storage/network service
  contract gaps and full-source ownership blockers; no compiler/runtime behavior changes.

- Complete the source-built native JSON object-mapping gate: scoped introspection
  descriptor materialization, managed snapshot arrays and real property setter calls
  now execute through separate native libraries. Unchanged production mapping sources
  and the existing sample pass, alongside nested objects, Boolean/int/string properties,
  jagged arrays, shared mutation and pre-construction validation. Bootstrap dependencies
  and the explicit 100-million-instruction acceptance budget are recorded; full-System
  and HTTP integration remain separate milestones.

- Preserve ordinary final reference classes with the CLI Sealed flag in manual
  definitions, builders, native readers and introspection; reject derived classes
  with sealed bases. Admit explicit canonical Boolean scalar ownership alongside
  numeric primitives. These contracts support source JSON descriptor validation.

- Resolve local inherited public methods as interface implementations on derived classes.
  Mark only selected implementation slots virtual for CLI interoperability, preserving
  unrelated method flags. CLR and native interface dispatch execute to 42; 152 metadata
  contract groups pass. Inherited explicit reimplementation remains unsupported.

- Add an executable separate-library guard consumer covering the mapper's null-return
  guard, boxed int/bool patterns and Result match-return control flow. Native verification
  and execution pass; production JSON object mapping remains in progress.

- Preserve final vector parameter arrays through definitions/builders, CLI ParamArrayAttribute,
  native parameter-target attributes and introspection. Native writing requires an explicit
  core marker binding; separate consumers execute empty, expanded and existing-array calls.

- Preserve flags-enum intent as the ordinary core FlagsAttribute in CLI projections
  and the existing native enum-info flag. Definitions/builders and introspection expose
  the classification; manual attributes share validation. The unchanged source
  BindingFlags enum now compiles and executes through an artifact-only consumer.

- Author and execute the bounded Object Equals/GetHashCode/ToString overrides on
  nongeneric reference classes, preserving CLI slot flags, native identity and inherited
  dispatch across separately compiled assemblies. Encode core Object signatures with
  the CLI Object element code so Equals actually overrides on .NET. The retained
  executable seed now supplies Object.Equals using reference identity.

- Preserve nongeneric closed interface families through definition/builder authoring,
  native readers and metadata introspection. Validate direct implementation ownership
  at runtime linking while leaving open branches extensible. Native Raven gates also
  exercise static extension methods and the configured core TypeHandle<T> intrinsic;
  production descriptor emission and JSON mapping remain in progress.

- Bind native TypeModule results to source-owned ModuleInfo interfaces and instantiate
  their RuntimeModuleInfo provider within the same metadata assembly/module. Validate
  the existing two-string layout, interface relationship and heap budget. Generated
  native PE executes interface dispatch; production descriptor-library compilation
  and JSON mapping are not yet complete.

- Compile the runtime library's explicit native handle-service declarations through
  Raven into bodyless InternalCall metadata. A separate artifact-only consumer
  verifies and returns 42. Reject unsupported service declarations before output;
  unknown runtime bindings still fail verification. The bootstrap marker is temporary
  compiler metadata; production reflection descriptors and JSON mapping remain open.

- Author bodyless nongeneric assembly-function runtime services through the metadata
  definitions/builders using InternalCall implementation flags. Preserve flags and
  exact runtime service names in native readers, reference projections and imports;
  reject incompatible bodies/flags. Generated native PE and an artifact-only C# API
  consumer verify and return 42 with an explicit test seed. Raven source declarations
  and production introspection ownership remain pending.

- Extend the explicit native handle-service catalog with identity, generic arguments,
  shape/display/token queries, object-type lookup and parameterless reflection
  construction. A separately compiled provider/consumer verifies and returns 42,
  checking real constructor state; invalid arguments and unsupported construction
  reject. Raven `80ebdc0a7` supplies core Object dependency signatures. Production
  descriptor services and JSON mapping remain open; record the refreshed source audit.

### 2026-10-04

- Add opaque RuntimeTypeHandle signatures and IILGenerator.LoadTypeToken/raw Ldtoken
  authoring to the experimental metadata API. CLI output uses the core value-type
  reference and standard instruction; native PE uses the existing RuntimeTypeHandle
  and ldtoken contracts. C# tests cover local/constructed/scoped identities and import;
  generated native PE verifies and executes. Descriptor services and JSON object
  mapping remain subsequent integration work. Recognize core-local handle definitions
  and expose the real TypeName service in the explicit bootstrap catalog. Raven
  `1c3e14bc7` now executes generic and external nominal typeof through a separately
  compiled test provider; no production descriptor is substituted.

- Record shared conditional propagation lowering, independently integrated into Raven
  main, unlocking the unchanged five-source native JSON library. Runtime verification
  and an artifact-only DOM consumer pass, including alias mutation and number parsing.
  Add a reproducible internal codec gate for Unicode round trips, mutation and
  invalid-input/cycle rejection, with separate consumer ownership manifests.
  Public serializer/object mapping still requires introspection dependencies;
  record the handle-first capability sequence from the candidate expansion.

- Record shared local-assignment propagation lowering in Raven,
  independently integrated into main, and native success/failure execution against
  source-built libraries. JSON still rejects remaining nested propagation before output.

- Author local nongeneric class bases through definitions or builders, emit CLI
  TypeDef.Extends and native base relationships, and call direct base constructors
  through IILGenerator. Validate initialization before publication and preserve
  inherited field offsets. C# coverage executes both CLI and native PE construction.
  Raven's paired driver now executes forward base declarations and mutation through
  a base-typed alias on both runtimes. Isolate and validate its general constructor
  binding fix on a branch based on main; preserve the source-built text-stream gate.
  Integrate the validated Raven fixes into main (`4f95db536`, 148 focused .NET tests)
  and remove four integrated local fix branches. Runtime protected constructors now
  enforce family access across assembly identities; closed class roots enforce local
  direct-family ownership. Other protected categories reject explicitly. The new
  visibility spelling requires an updated runtime reader; previously descriptive closed
  class flags now require abstract, nonsealed roots and reject external direct children.
  Metadata builders/manual definitions now author protected constructors using CLI Family;
  native reader/facade round trips preserve accessibility, and unrelated calls fail writer
  validation. All 145 C# groups and 55 runtime regressions pass; generated public/protected
  assemblies execute. Native closed-family authoring, facade direct-child views and Raven
  admission now pass direct and separate library/consumer execution on both targets.
  Output-owned class-base facts validate imported reference conversions without reopening
  readers. CLI reference projections retain Abstract but no closure marker; executable CLI
  metadata-library Write rejects native closed declarations. All 146 metadata groups and
  35 Raven regressions pass. JSON next rejects a lowered propagation expression before output.

- Materialize local nongeneric native class bases in metadata definitions and the
  introspection facade, including standalone NEOX assembly snapshots. Reject cyclic,
  missing and unsupported bases; reject reference projection rather than dropping
  inheritance. A three-binary-assembly runtime regression verifies constructor chaining,
  inherited mutation, virtual dispatch and identity. Writer/Raven hierarchy emission
  remains the next integration step.

- Execute five unchanged source-built text-stream types against native encoding and
  text libraries. Verify Unicode line/whole-stream I/O, byte counts, EOF, shared cursor,
  leave-open, limits and malformed-input errors. Raven now preserves match-initializer
  early returns; unsafe nested exits still reject before output. The focused JSON
  document frontier now reaches nominal hierarchy/signature admission.

- Refresh the full-System audit after native Char/String ownership and establish a
  separate source-built encoding-library gate. UTF-8 incremental encode/decode, ASCII
  rejection, malformed input, and field evaluation/failure consumers execute through
  native metadata imports. Record the portable receiver fix and the next text-stream
  lowered-expression blocker; full System and dual-target library parity remain open.

- Complete source-owned Char/String native import and emission through a retained seed
  without duplicate text declarations. Separate consumers execute grapheme literals,
  equality/patterns, interface dispatch, Unicode text operations, String sequence
  construction and mutation-independent copying. Encode ordinary String constructor
  metadata and execute it over private immutable-text storage; CLI Char stays unchanged.

- Add explicit owned grapheme Char metadata declarations through definitions/builders
  and expose the fact through native introspection. API-authored managed receiver methods
  preserve combining and ZWJ graphemes in native execution; ordinary .NET Char tests remain
  unchanged. Native snapshot imports and output-owned external grapheme references now
  preserve scoped identity; separately compiled forwarders execute with both authoring
  paths. Reject competing owners and late storage reinterpretation. Raven source ownership
  remains pending. No numeric Char category or format fork is added.

- Compile source-owned String with an explicit ownership manifest and retained seed;
  separate consumers execute Unicode casing, byte/grapheme counts, equality and interface
  iteration. Implement existing equality operators in Raven source. Runtime dispatch now
  passes String reference payloads through interface views and resolves canonical String
  members. Char remains bootstrap-owned. Clarify Unicode text model versus UTF-8 storage.

- Support explicitly owned System.String reference declarations in the experimental
  metadata definition/builder APIs, native snapshots and external method references.
  Local and separately encoded instance consumers execute on NeoCLR; numeric value
  categories remain unchanged. Raven ownership selection and grapheme Char are not yet
  integrated. No runtime instruction or metadata version change is required.

- Verify Raven's direct pattern branching in if-expressions: null/non-null cases execute
  on both targets. Unchanged String/Char join the cumulative emitted library, runtime
  verification passes and a separate static String consumer exits 42. Canonical primitive
  instance storage/ownership and imported nullable annotations remain open. The independent
  constructed-setter .NET fix is validated and integrated into Raven main at 210d891e0.

- Author and round-trip explicit interface mappings through definitions and builders;
  emit standard CLI MethodImpl and existing native scoped relationships. Raven explicit
  getters/setters execute across native assemblies; qualified property names now validate.
  Source String proceeds to a String.Concat definite-assignment blocker. CLI snapshot
  mapping materialization remains unsupported; native CLI projection rejects mapping loss.

- Extend the checked native bootstrap catalog with 20 existing text services, including
  graphemes, scalar vectors and UTF-8 slice outcomes. Separate native library/consumer
  tests execute UnicodeScalar and text operations, including character-array mutation;
  missing declarations reject without output. Canonical source String/Char ownership
  remains open; no runtime implementation or metadata-format change is implied.

- Verify the shared Raven conversion-cache fix (459856a71; independently validated
  main backport f0c3b75a0): empty-first and reverse-order numeric source builds pass.
  Native numeric, generic Number, broad application and combined Tasks consumers execute;
  139 focused .NET tests pass on each compiler line. No metadata/runtime format change.

- Reassess the full-System frontier after Number with explicit numeric ownership.
  Add an ownership-aware inventory baseline and record a reproducible empty-file/source-
  order binding failure. Prioritize its repair, then shared text/core services, inheritance
  and introspection; this is assessment evidence, not implementation of those capabilities.

- Complete the native generic Number integration gate: rebuild all ten numeric source
  implementations, compile a separate generic algorithms library and execute a consumer
  with no library sources. Arithmetic, Zero/One, inherited CompareTo and forwarding
  return the expected result across all ten types. Invalid string arguments reject with
  RAV0320 without output. This completes the Number slice, not the entire class library.

- Add IL-generator constrained static calls through method type parameters with local
  interface bounds, including inherited bounds and Self signature substitution.
  Typed and raw Emit overloads share validation. CLI constrained./call and native
  callself execute the generic fixtures with exit 42; no runtime encoding change.
  External static and constructed instance targets now retain scoped contracts;
  bounded method forwarding validates the caller, and Raven emits these operations.

- Extend method interface bounds to explicit external nongeneric interfaces, using
  CLI TypeRefs and existing native TypeBound identities. Introspection checks the
  resolved dependency category; .NET and NeoCLR execute a separate-contract fixture.
  The experimental constraint record now exposes TypeReference instead of TypeDefinition.
  Legacy CLI projection rejects external bounds explicitly; direct native Raven import
  preserves them.

- Preserve owned nongeneric interface bounds on method type parameters through
  definitions/builders, standard CLI constraints, native metadata and introspection.
  Validate concrete local generic arguments and recheck after graph edits. Raven
  imports these bounds for semantic checks. The direct snapshot ImportReference
  convenience API still rejects bounded methods; Raven authors calls from symbol facts.

- Record a future function-type constraint using Raven’s proposed `where F: func`
  spelling. Signature requirements, metadata encoding and .NET target behavior
  remain design questions; this does not add compiler or runtime support.

- Extend the metadata IL generator's constrained call operation to static interface
  contracts, using standard CLI constrained./call and native nonborrowed callself.
  Substitute Self in stack signatures and retain exact implementation/operand checks.
  C# tests execute .NET dispatch and native ordinary/Self consumers return 42.

- Rebuild the cumulative runtime-library subset with all ten unchanged numeric source
  implementations under one owner, replacing seed Int32/Int64 declarations. Select
  existing integer formatting services explicitly. Preserve canonical primitive member
  names and match encoded interface contracts by metadata declaration name, retaining
  exact signature and visibility checks. Readers continue accepting previous encoded
  primitive names. Rebuild earlier experimental primitive-provider artifacts before
  using Raven’s new canonical member authoring. The source-free numeric consumer
  verifies and runs with exit 99; the generic follow-through gate returns 42.

- Compile unchanged source Single/Double Number implementations and NumberParseError
  into a native library, then execute an artifact-only consumer covering parsing,
  ordering and arrays. Raven selects explicit native primitive providers and maps
  checked intrinsic storage to scalar receivers. Metadata member references can be
  authored from semantic contracts without reader handles; dependency records retain
  canonical scalar identity.

- Add explicit native numeric primitive designation to metadata definitions/builders
  and expose it through introspection. Preserve existing runtime scalar representation,
  Self substitution and imported managed receivers. Reject ordinary record fields and
  executable CLI emission for these declarations. A separate API-authored primitive
  library/consumer loads and executes. Imported scalar ownership can now also be
  designated explicitly through AssemblyBuilder.SetNativePrimitive for constraint checks.

- Complete fixed-width integer metadata signatures and expose standard unsigned
  arithmetic, comparisons, shifts and conversions through the IL generator. Preserve
  storage signedness while using CLI evaluation widths. Raven imports and emits the
  corresponding native operations; separate integer libraries and consumers run on
  both targets. Fix the primitive local overload to admit floating locals. Primitive
  source ownership and generic Number dispatch remain separate unfinished work.

- Author/read static interface contracts and inherited ComparableTo<Self> through the
  metadata definitions, builders and introspection views. Validate static/instance
  implementation identity and emit CLI MethodImpl rows. Actual Number source, a
  separate struct implementation and artifact-only consumer execute direct arithmetic,
  identities and ordering natively. Intrinsic numeric storage and generic callself
  compiler emission remain open; this is not completion of the numeric family.

- Add Single/Double signatures, exact-bit floating literals and numeric conversions
  to the experimental metadata IL generator. CLI and native output execute the same
  arithmetic, NaN and signed-zero checks. Graph-based binary emission selects existing
  schema 3 for high-bit Double literals; older schema-2-only readers reject those images.
  Raven now imports and emits those primitives through its native target; a separately
  compiled floating library and artifact-only consumers execute on both targets, plus
  native parser payloads. Expose existing unordered comparisons through the IL generator
  so NaN keeps correct <=/>= behavior. Primitive unary +/- binding is isolated as a
  general Raven fix. Source primitive declarations and static Number contracts remain open.

- Select all eleven existing numeric/Boolean parsing services in the checked native
  bootstrap catalog. Actual Boolean and BooleanParseError sources compile separately
  and execute through an artifact-only consumer; boundary/error cases also exercise
  the numeric service family. Raven's explicit primitive-core selection fix is validated
  independently and integrated into local main as f749c1a75. Wider payload emission,
  static Number contracts and complete primitive-source ownership remain open.

- Admit Self in authored external interface method contracts and substitute the concrete
  implementing owner when validating required methods, including generic owners. Preserve
  symbolic Self in native metadata without a format change. The actual Clonable source,
  separate implementation and artifact-only consumer compile and run with exit 42;
  this establishes conformance and concrete calls, not constrained generic Self dispatch.

- Execute all six Raven Tasks/Concurrency source files through a separate native
  library and artifact-only consumer, including cancellation, generic continuations,
  interface callbacks, worker results and implicit entry draining. Typed queue services
  preserve source-owned identities and reject wrong or duplicate registrations; legacy
  queue entry points remain supported. Metadata IL generation now binds methods on
  constructed class/interface owners. Function types retain structural signature identity.

- Support ordinary top-level Int32 enums in the experimental metadata definitions,
  builders and introspection facade. CLI output uses System.Enum, value__ and literal
  constants; native output retains the existing enum representation. Raven compiles
  unchanged TaskState into a separate library consumed and executed on both targets.
  Native conv.i4 reads enum values; non-enum records remain rejected. Flags, other
  underlying widths and enum-owned methods remain outside this authoring profile.

- Admit explicit no-result callbacks for native ScheduleTask while retaining the
  existing inhabited-Void convention. The checked bootstrap catalog now includes
  scheduling and explicit entry draining. A separate native library/consumer verifies
  retained receiver identity and mutation; source-owned task queues remain pending.

- Validate the combined 57-source native library, including UTF-8/file streams,
  after correcting Raven source-array interface lookup caches. Separate consumers
  verify file mutation, exit 42 and unchanged broad application output. No runtime
  or metadata-format change; full System compilation remains open.

- Add explicit schema-3 library PE writing and native reading/loading, bounded to
  16 MiB PE, 8 MiB envelope and 32 MiB host JSON. Preserve schema-1/2 application
  bounds and declaration/storage limits. Raven libraries select this profile; 53
  unchanged sources compile together and separate stream/broad consumers execute.
  A >4 MiB API library also reimports and executes through both API and Raven consumers.
  Older readers reject required schema 3; use matching compiler/runtime bundles.

- Generate native bootstrap declarations and executable service wrappers from the
  existing checked signature catalog: 20 selected services plus generic erased-value
  helpers. Validate all 22 against the seed and reject unsupported selections. Native
  separate-library UTF-8, real file I/O and worker callback consumers execute; preserve
  exact ownership and core/seed rebuild requirements. Record the remaining queue,
  completion-callback and whole-library transport blockers in the updated audit.

- Map the explicitly bound core System.Value to the existing native erased carrier
  across signature writing, reading and reimport; reject malformed aliases and wrong
  projection cores. Seed generic IsValue/UnpackValue helpers execute through ordinary
  calls. Separate-library tests cover Int64 success, Byte parse statuses and wrong-kind
  failure. This preserves the independent-of-Object direction, not exhaustive payload
  support. Fix unit callback/discard regressions exposed by the cumulative native gate.

- Complete explicit inhabited-unit signature/local admission in Raven's portable
  planner. Add separate native stream and generic-unit library/consumer execution:
  unchanged MemoryStream, Flush outcomes, interface identity, unit arrays and generic
  returns pass. Preserve no-result calls and ordinary .NET behavior (38 focused tests).
  The cumulative single PE still exceeds schema 2's 1 MiB limit; this is recorded,
  not relaxed implicitly. No metadata/runtime representation changes.

- Reprioritize full-System compilation by shared capability blockers at the author's
  direction. Add a reproducible 166-source/family/probe audit and evidence-backed
  strategy: unit/erased values and common service ABI first, then reusable metadata,
  core identity and execution categories. Defer isolated time-zone wiring. This is an
  assessment and plan, not newly implemented compiler/runtime support.

- Add unchanged TimeOffset to explicit native source ownership (48 sources), with
  separate-consumer coverage for signed tick round trips, offset limits and civil-range
  rejection. Fix portable admission for imported value-type static properties such as
  TimeOffset.Zero in Raven; preserve separate reference-owner capabilities and the
  ordinary .NET backend. No metadata/runtime/bootstrap or public API change.

- Expand native source ownership to Instant, Clock, SystemClock and OverflowError
  (47 sources), binding the existing UnixTimeTicks service explicitly. Instant now
  calls its source-owned local-time factory directly instead of a bridge alias.
  Clock/overflow/interface consumers, full native gate, paired Duration and two runtime
  clock tests pass. Rebuild comparer core/seed together. Legacy snapshot refresh remains
  blocked by its union-reference build; no unverified generated hashes were updated.

- Expose the existing execution budget as run-only `--instructions <positive-count>`,
  rejecting malformed/repeated values before input loading. Keep the 100,000 default
  and other limits unchanged. The separately compiled native globalization sample
  completes with an explicit 1,000,000 budget; CLI tests cover exhaustion and overrides.
  No metadata/compiler change or performance improvement is claimed.

- Add explicit ownership and native acceptance for 43 unchanged collection/calendar
  sources. Raven 5c60425db closes terminal Fail control flow without weakening metadata
  checks. Separate Date, generic-collections and broad application consumers pass,
  alongside paired Duration and 60 focused .NET checks. Hebrew formatting matches
  expected output; larger globalization execution reaches the CLI instruction limit.
  Full .NET calendar coverage remains open; no metadata/runtime implementation change.

- Validate nested binary propagation in local initializers through separate native
  Result imports, preserving evaluation order, field snapshots and early failure.
  Expanded native/paired Duration acceptance passes. Raven cb0f48bd7 is independently
  backported to main (7db0f3dfe); 12 integration and 11 main checks pass. Date now reaches
  a local-before-store verifier error with no published output. No metadata/runtime change.

- Execute discarded Result propagation against a separately compiled native library.
  Raven a99152da0 preserves once-only evaluation and early failure returns; the broad
  native gate, paired Duration consumers and 18 focused .NET tests pass. The general
  lowering fix is backported to Raven main as 22539952c with six passing tests. Date reaches
  nested-expression propagation, reproduced with failure before output publication.
  No metadata/runtime/format or bootstrap change.


- Decode and emit canonical CLI Char signatures while preserving native grapheme
  storage and intrinsic call owners through explicit core bindings. Native character
  signatures round-trip through materialization, reimport and declaration projection.
  Separate-library grapheme execution, broad gate, paired Duration, seven consumers and
  130 C# groups pass. Date now reaches discard-assignment emission; array-element
  receiver addresses remain unsupported. No runtime/format change; guest API snapshot stale.

- Connect the explicit calendar bootstrap to existing host culture/local-time services
  and grapheme-count String.Length. Convert service value arrays into fresh nominal
  reference arrays with the established adapter. Native service/Unicode/fault checks,
  broad application acceptance and paired Duration consumers pass. Date inventory now
  reports only string-indexing binding errors. No compiler/runtime implementation change;
  correct earlier integration wording from scalar indexing to the shipped grapheme model.

- Add calendar-foundation acceptance with unchanged source-built Duration,
  ComparableTo and EquatableTo. The same reference-only consumer executes on .NET
  and NeoCLR; native ArrayList<Duration> storage/copy/iteration and the broad application
  pass. Compiler, metadata and runtime implementations are unchanged. Date dependency
  inventory records missing string indexing and two runtime services before publication.

- Validate explicit bootstrap Int64 member imports with exact-width managed receivers
  and add executable Int64.CompareTo to the comparer seed. Raven `9d06edf80` addresses
  value-returning receivers without write-back; native ArrayList<long> copy/indexer,
  evaluation-order and mutable-struct checks pass. Expanded broad gate, seven consumers,
  129 metadata groups and 29 focused C#/.NET checks pass. Full generic-collections still
  lacks Date. No runtime/format change; guest API snapshot remains stale.

- Extend the explicit comparer primitive bootstrap with Int32.Equals and direct
  ToString; unchanged library-integers now compiles and runs with exact output.
  The expanded native application/library gate passes without compiler or runtime
  changes. This does not compile Int32 from source or establish boxed virtual dispatch.

- Add IILGenerator.LoadArgumentAddress and raw Ldarga for by-value parameter storage
  in CLI/native output; reject receiver/byref/invalid slots and mismatched stores.
  Import CLI Object signatures with the explicit core identity. Raven `1fb1bbd45`
  adds bounded signed-range lowering and parameter receivers; unchanged library-comparers
  now compiles and runs against the source-built library. All 129 metadata groups,
  55 .NET tests, seven native consumers and the expanded broad gate pass. Runtime and
  format versions are unchanged; the separate guest API snapshot remains stale.

- Extend explicit bootstrap method imports for intrinsic String/Int32 receivers and
  the exact Object.GetHashCode virtual slot, with receiver/identity/signature rejection
  tests. Compile unchanged StringComparer into the native source library using a
  dedicated comparer ownership/core/seed profile. With Raven `7b3928239`, focused
  equality, Unicode ordering, folded hashes, map replacement and extreme Int32
  comparisons execute. The full comparer sample remains blocked on integer-range
  loops. No runtime or format-version change; guest API snapshot remains stale.

- Record Raven `1b15715bf` native primitive captures and promoted integer operand
  emission. Captured comparer policies, escaped/per-iteration values and mixed-width
  arithmetic execute; mutable captures reject before output. Broad native gate,
  seven consumers and 37 focused .NET/C# tests pass; metadata/runtime are unchanged.

- Expand native source-library acceptance with unchanged query-basics/query-names
  and observable lazy iteration/disposal checks through imported Filter/Map/Take.
  The broad application gate passes. Record missing numeric/comparer/introspection
  surfaces in a seven-sample assessment; no compiler, metadata or runtime change.

### 2026-10-03

- Record Raven `623cbc1d8` immutable reference capture lowering and expand native
  acceptance with unchanged list-filters plus escaped/shared-reference callbacks.
  Exact output, identity/mutation, separate invocation state and rejection without
  output pass alongside the broad application, seven consumers and 31 .NET tests.
  Metadata/runtime remain unchanged; general mutable/value captures remain open.

- Admit owned nongeneric nonvirtual reference-instance callback bindings in the
  metadata builders/IL generator. Consume the receiver and preserve shared mutation
  through CLI/native execution; invalid stack receivers reject on write. Raven
  `a53b6412a` uses this for source method groups against the separate class library.
  All 128 metadata groups, 31 focused .NET tests and native gates pass. Captured
  lambda lowering remains open; no runtime or format-version change.

- Support bounded nested vectors in metadata authoring/native materialization, using
  existing CLI SZARRAY and native ArrayRef encoding. Preserve exact nominal Array<T>
  backing casts and storage identity at runtime. With Raven `35464aabf`, unchanged
  array callbacks and nested mutation execute against the source-built library.
  All 127 metadata groups, 27 focused .NET tests, 19 runtime tests and seven native
  consumers pass. Older readers can reject nested vectors; no format version change.

- Expand native source-library acceptance with an array Count/mutation regression
  and unchanged Option, propagation and collection-capabilities samples. Raven
  `3367f3200` fixes projected array interface receivers; the broad native application,
  seven native consumers and 13 focused .NET tests pass. Record remaining callback,
  capture and library-coverage gaps without changing metadata/runtime semantics.

- Record Raven `42503ec23` diagnostic handling for native/bootstrap catalog identity
  conflicts, replacing an uncaught exception. C# rejection/unchanged-output coverage
  and seven native execution consumers pass; runtime and format remain unchanged.

- Record Raven `777499170` native type/union construction through introspection,
  removing obsolete reader-signature helpers. Source-union library and broad native
  application execution plus seven native consumers pass; no format/runtime changes.

- Expose native generic type-parameter names through introspection Name, preserving
  owner/ordinal identity and rejecting unmaterialized CLI names explicitly. Raven
  `003b9a38d` uses the retained type view for declaration facts. All 127 metadata groups
  and seven native consumers pass; the existing guest API snapshot remains stale.

- Route Raven native methods, constructors and module functions through introspection
  views (`015e6f66d`), removing the definition-based callable-symbol wrapper. Canonical
  constructor/accessor identity and all seven native consumers pass. Metadata encoding,
  public library APIs and ordinary .NET loading/emission are unchanged.

- Simplify Raven native field/property loading through existing introspection views
  (`12b545bc1`), removing duplicate definition/accessor lookup work. Native semantic
  contracts and seven executable consumers pass before/after. Record the explicit legacy
  callers; no metadata format, .NET backend or public metadata API changes.

- Scope Raven vector-loop expansion to portable planning (`19a3e84c0`), preserving
  ordinary .NET loop emission and native nominal-array behavior. All 86 focused checks,
  native broad execution and native labeled/nested-loop execution pass. Restore main's
  behavior for the known capture repro without claiming its lexical-lifetime bug fixed.

- Simplify Raven .NET body emission in `c2a66d82a` to its established generator while
  retaining NeoCLR metadata/planning. All 94 selected .NET checks pass before/after and
  unchanged native application-order-collections executes against its separately built
  library. Reconcile the stale constraint test separately (`9bbcbb2b0`, isolated
  `e46c0a9c9`); retain the reproduced loop-capture bug as unresolved.

- Record the paired compiler/target-boundary assessment in Raven `10f5f0089`: retain
  NeoCLR metadata/introspection/builders and review duplicated .NET body emission.
  Additional execution checks pass on both branches; a shared constraint-import test
  discrepancy remains unresolved. No compiler/runtime behavior is changed by the audit.

- Record the author-directed compiler direction reassessment and suspend the proposed
  .NET array adapter slice. Current Raven main and integration pass the same 36 focused
  array/iteration tests and ordinary .NET mutation/LINQ execution. Distinguish the invalid
  custom-interface bootstrap assumption from ordinary .NET behavior; no compiler or
  runtime implementation changes are part of this audit.

- Configure an explicit .NET inhabited-unit value without replacing the CLR core.
  Unchanged source unions/collections now separately compile, import and execute in four
  focused .NET consumers. Native broad execution is preserved. The .NET broad sample
  advances to custom array-interface execution and remains failing; no paired-gate success
  is claimed. Shared compiler changes are isolated and tested on the main-based fix branch.

- Add executable .NET CheckedStorage and terminal-failure bootstrap adapters and a
  paired source-library assessment driver. Isolate and validate Raven's stronger-reference
  override return fix on its main-based branch (13 tests); native broad execution still
  passes. The .NET consumer exposes invalid CLR void generic storage in Propagatable;
  library emission alone is explicitly not counted as successful execution.

- Execute unchanged application-order-collections against a separately compiled native
  source-library subset with exact stdout and exit 0. Add the retained seed's Int32 console
  overload using existing runtime services. Preserve inherited-slot facts through metadata
  introspection and symbol-only value-override references, fixing imported SingleError.ToString
  linking. The .NET source-library adapter gate remains open.

- Connect native vectors to an explicitly selected nominal Array<T> descriptor. Source
  array interfaces, mutation and query iteration execute against a separately compiled
  library. Add checked SetArrayBacking authoring and reader/runtime validation; legacy
  array execution is preserved. This optional native execution metadata requires a
  matching runtime and does not change CLI array encoding or .NET semantics.

- Record the author-directed nominal Array<T> backing policy; structural array changes
  remain deferred. Unchanged source Array and an independent consumer compile, but runtime
  vector/interface backing fails. Add reproducible assessment evidence without claiming
  execution support or changing the successful query gate.

- Add IL-generator/raw UnboxAny authoring with CLI/native encoding and typed stack checks.
  Exact value boxes and reference identity follow existing runtime semantics. Unchanged
  query sources now separately compile, import and execute (42); 126 metadata C# groups
  pass. The broad application next requires configured array extension receiver support.

- Add a separate native generic extension-library acceptance case alongside HashMap.
  Receiver-generic predicates and method-generic selectors execute (42) through Raven
  extension metadata discovery. Assess unchanged query sources: OfType object-to-generic
  conversion remains unsupported and fails before output publication. No format change.

- Extend the native source-library acceptance driver to unchanged comparers and HashMap.
  Separate import and execution verify collisions, growth, replacement, missing keys,
  callback dispatch, key snapshots and shared object mutation (exit 42). Record revisions
  and artifact hashes; the broad application and dual-target class-library gate remain open.

- Read native callback signatures and expose canonical FunctionTypeInfo metadata views,
  including owner/method substitution, external identity and explicit no-result facts.
  Symbol-authored method references accept bounded callbacks. Separately compiled unchanged
  ArrayList now imports and executes; 125 metadata groups and seven native consumers pass.
  Raven's explicit no-result callback import remains unsupported; no format version changed.

- Add a bounded collection bootstrap with explicit callback declarations and executable
  terminal failure. Unchanged ArrayList emits and runs with sources included, checking
  alias mutation, copying, iteration and Find; negative capacity faults correctly. Separate
  native import is explicitly blocked by callback signature materialization and is not
  counted as a completed library-consumer gate. The separate source-union gate still passes.

- Compile unchanged Option/Result, Propagatable and iteration sources with an explicit
  executable union seed and source ownership manifest. A separately compiled native
  consumer verifies copies, residuals and boxed display (exit 42); missing-library and
  duplicate-seed guards reject before publication. Authoring now retains top-level value
  interface edges. The .NET class-library adapter and broader application gate remain open.

- Validate inhabited unit through generic union construction, payload matching and an
  ordinary argument on neoCLR (42). Raven now respects the explicit unit contract in
  overload validation; the fix is isolated and tested on a main-based branch. No native
  metadata/runtime encoding change is needed; source Option ownership remains open.

- Record the author's inhabited unit direction: value-bearing parameters, storage and
  generic arguments remain distinct from no-result callables. Track generic union binding
  and source-library bootstrap ownership as remaining integration gaps.

- Preserve declared parameter names through CLI/native encoding and constructed
  introspection views. Author external value and nested type/method references from
  explicit identity/digest contracts, without reopening reader objects. C# metadata
  contracts pass 124 groups; separately compiled plain/generic union consumers run
  on CLR and neoCLR. Unchanged Option still requires a residual unit-value contract.

- Map explicitly bound core String static call owners and core Char type operands to
  canonical native primitive encodings. Keep ordinary nominal String import rejected;
  validate core/module identity and selected method signatures. C# runtime checks and
  122 metadata groups pass. Body validation errors now identify the declaring method.

- Add bounded type custom-attribute definitions/builders and metadata-only introspection.
  CLI rows/blobs and existing native attribute records preserve String/Int32/Boolean
  fixed arguments and explicit constructor owners. C# tests verify CLR decoding and
  separate native attribute dependency loading/execution without running constructors.
  Union compiler emission/import remains pending; native transport remains transitional.

- Clarify Raven-to-neoCLR execution as the immediate integration priority. Retain CLI
  metadata as the baseline while deferring wider format migration and new semantic
  experiments; explicitly retain the current PE/#Neo transport limitation.

- Record Raven case-branch/Boolean pattern lowering and configured unit storage.
  Native unit out/value argument execution returns 42. Unchanged Option passes
  source-body preflight; union/case attribute round trips remain the publication gate.

- Record Raven contextual reference-null lowering and dual-target execution. Plain/
  generic union bodies now reach metadata preservation; unchanged Option reaches
  its TryGetOutput case pattern. Native union output remains withheld.

- Support imported core Object.ToString virtual dispatch with exact CLI core and
  native System slot validation. API boxed overrides and Raven generic integer/string
  display execute correctly; other virtual class calls remain unsupported. Union
  preflight advances to its synthesized null literal, without native publication.

- Add IL-generator null identity/type tests and String checked casts using standard CLI
  instructions and existing native operations. Value/generic tests require explicit core
  binding. C# contracts (121 groups), API runtime tests and Raven ordinary-command null/
  type-test consumers pass. Union preflight now reaches Object.ToString dispatch; native
  union output remains blocked.

- Add owned mutable field-address operations to the IL generator, including constructed
  generic fields, using standard CLI/native ldflda. Reject readonly, foreign, temporary
  value and uninitialized receivers. API and Raven driver executions preserve nested
  value mutation and object aliases on both targets (42); 120 metadata groups pass.
  Unchanged Option now reaches its generated formatting comparison, with no native output.

- Add typed IL-generator boxing and an explicit core Object reference. CLI uses standard
  box tokens; native output requires the validated System core binding and uses the
  existing instruction format. Generic value dispatch, primitive display and reference
  identity execute; 119 C# metadata groups pass. Raven adopts an explicit boxing
  capability; full generated union display and native union metadata remain pending.
  Independently isolate Raven's arrow-method return-conversion fix (697a093d7 / main-based
  c96305e50), validated by ten tests on each line; main remains unchanged.

- Integrate bounded value Object overrides into Raven callable contracts and the
  NeoCLR adapter (`29268815d`), with explicit --runtime-seed host binding and seed/source
  ownership rejection. Ordinary/generic native driver cases execute; the existing CLR
  override return-nullability mismatch remains explicit. Unchanged Option now reaches
  generated display conversion lowering rather than override declaration rejection.
  Twelve focused Raven tests pass; full union metadata/emission remains pending.

- Add bounded value-type ToString override authoring through definitions and
  TypeBuilder.AddOverride. CLI output preserves the Object virtual slot, including
  when the method also implements an interface; ordinary and generic value dispatch
  execute in C# tests. Native writing now validates one explicit retained-System
  Object.ToString binding and preserves the slot name/flags through native reading and
  imported calls. An API-produced native library executes both separate direct calls and
  boxed ordinary/generic dispatch against the real System bundle; incompatible/missing
  bindings reject. No runtime or format-version change. Raven subsequently consumes the bounded override as recorded above. Host API
  XML/manual coverage is updated; the existing runtime API snapshot remains stale.

- Record Raven's generated union constructor/accessor planning fix: retained case and
  parameter syntax now reaches existing synthesized bodies with exact union-anchor
  validation. Shared core-body lowering and .NET union regressions pass; native union
  publication still requires override/display and union metadata contracts. No runtime
  or metadata API changes are included.

- Admit value-type interface relationships through metadata definitions/builders and
  native readers. Add bounded IILGenerator constrained interface calls and a typed raw
  Callvirt overload, encoding CLI constrained./callvirt and native borrowed callself.
  C# execution checks addressed mutation, independent copies and exact receiver types;
  implicit unboxed interface conversions still reject. Raven opts into value-interface
  declarations; Option now reaches its generated ToString blocker. No runtime/schema
  changes or boxed/generic constrained-call support are claimed. API snapshot remains stale.

- Record Raven's complete source-union declaration discovery and physical generic
  case ownership. Native preflight now identifies unsupported synthesized ToString
  overrides; unchanged Option also requires value-type interface implementation.
  .NET union controls execute, but native union publication remains explicitly blocked
  pending complete contracts. Metadata format and runtime are unchanged.

- Support Byte signatures/storage and `IILGenerator.Emit(OpCode.Conv_U1)` in the
  experimental metadata API, preserving Int32 stack values and exact by-reference
  identity. CLI/native readers and introspection retain Byte; CLR/native execution
  checks truncation and zero extension. Raven native emission/import now maps Byte
  explicitly. Generated union declarations remain pending; native format/runtime
  operations are unchanged. API snapshot regeneration remains blocked as documented.

- Advance source-union prerequisites with Raven native nested declaration emission:
  same-named payloads retain distinct owners; generic nested values, nongeneric classes
  and value copies execute on both targets. Generic enclosing capture remains unsupported.
  Metadata encoding/runtime are unchanged; generated union collection and byte tags
  are still pending.

- Preserve scoped nested class/value identities in direct native metadata snapshots and
  imports, including generic value cases below nongeneric owners. Same-named cases,
  external TypeRef scopes and visibility remain distinct; nested imported constructors
  execute on CLR and neoCLR (42). Raven retains enclosing symbol ownership. Source union
  emission and generic enclosing-type capture remain pending; native encoding is unchanged.

- Advance Raven's source-union prerequisites with native ordinary struct declarations:
  generic inline payloads, constructors, accessors, value copies and addressed mutation.
  Paired compiler-driver execution checks CLR-compatible copy/default behavior. Source
  union emission and separate native value consumption remain pending; metadata encoding
  and runtime behavior are unchanged.

- Extend union metadata prerequisites with nominal/constructed inline value fields,
  bounded recursive-layout rejection and direct native value declaration/import support.
  Tag/payload copies and native-imported value constructors execute on CLR and neoCLR
  (42); 114 C# contract groups pass. Raven preserves Struct identity from introspection.
  Source union emission, nested native case snapshots and byte-tag emission remain pending.

- Preserve writable ref/out parameter modes through native definitions, introspection
  and symbol-authored external method contracts, including generic interface substitution.
  Existing encodings are unchanged; readonly parameter metadata rejects explicitly.
  Separately compiled inherited interface dispatch runs on .NET and neoCLR. Unchanged
  Propagatable now emits natively; Option reaches a later source-declaration emission gap.

- Add a minimal checked-storage bootstrap and ordinary-driver acceptance for separately
  compiled generic reservation helpers. With explicit Raven bootstrap opt-in, native
  consumers preserve array alias mutation (42) and reject uninitialized reads. Source-owned
  iteration still executes on both targets. The next unchanged-source inventory identifies
  native propagation out-parameter contracts, Fail bindings and callback bootstrap gaps;
  ArrayList and the broad application remain incomplete.

- Preserve native Self signatures in the experimental metadata definitions/builders,
  PE/#Neo readers/writers and scoped introspection member views. Self remains distinct
  from generic parameter ordinals; invalid storage and free-function contexts reject.
  Executable CLI emission rejects Self; reference-only projection retains the explicit
  core marker. Contract loading is verified; Raven integration and typed dispatch
  authoring remain pending.

- Establish a source-owned iteration/collection contract bootstrap: seven unchanged
  Raven source units compile into a separate library consumed and executed on .NET
  and neoCLR. A checked-in ownership manifest selects the iteration contract; a
  minimal CLI core excludes competing collection declarations. Ownership failures
  reject before publication. The retained System seed and ArrayList remain later steps.
  Expanded API snapshot regeneration currently fails on source-union `None` resolution;
  preserve the last verified reference and record its stale source fingerprint explicitly.

- Support external generic interface inheritance and implementation declarations, with
  explicit complete contracts, definition/builder parity and metadata-only resolution.
  Authored PE emission preserves validated CLI implementation flags without dependency
  loading. C# contracts and Raven three-assembly diamond dispatch pass on both targets;
  native format/runtime behavior is unchanged. Source-library ownership remains next.

- Add metadata accessibility, declaration flags and constructor views with generic owner
  substitution. Raven consumes these facts; native byref/out profiles still reject explicitly.

- Integrate direct native metadata imports into rvnc with an explicit primitive-core
  argument. Both targets pass paired driver execution; native invalid-reference/output
  safety checks pass. Broad collections/source-library gate remains open.

- Record the author-approved dual-target execution plan and ordinary-driver baseline:
  .NET Hello/library and native Hello pass; native library field import remains a driver gap.

- Add canonical metadata-only generic method constructions, with copied arguments,
  owner-preserving definition navigation and simultaneous type/method substitution.
  C# contracts pass 109/109 and all seven native consumers execute (42). No invocation,
  emission or broader generic constraint support is implied.

- Add bounded inherited-interface metadata queries and consume them in Raven, preserving
  generic substitutions and diamond identity. Native encoding and .NET defaults are unchanged.

- Add metadata-only property and direct interface views with owner-scope substitution,
  canonical accessors and setter-only index signatures. Raven consumes these projections;
  inherited-interface traversal stays in the compiler. All 109 C# groups and seven native
  consumers pass (42). No runtime/encoding change.

### 2026-10-02

- Add metadata method/parameter views with distinct owner and method scopes.
  Raven consumes them instead of recursive signature projection and per-signature generic
  caches. All 109 C# groups and seven native consumers pass (42); no encoding change.

- Add constructed/array/primitive/owner-parameter metadata views and declared
  field projection. Raven consumes facade field types and closed signatures, preserving
  canonical array symbols. Method/parameter views and open method scopes remain pending.

- Add a C# metadata-only Introspection facade with a fixed MetadataLoadContext and
  canonical assembly/module/nominal views. Raven replaces its private nominal resolver
  with the shared context; constructed/member views remain pending. Validate exact
  identity, cycles, diamonds, conflicts and isolation; 109 C# groups pass.

- Record the proposed pure metadata resolution context and constructed-view layer over
  existing definitions/resolvers, including identity/lifetime rules, staged C# tests and future NeoCLR metadata-only
  Introspection/Emit use cases. This is a .NET-hosted Raven prototype direction,
  not a commitment to the same future implementation, API or port.
  This is architectural direction; no public API or runtime behavior is added.

- Import unconstrained generic native interfaces with constructed inheritance,
  parameter scopes and invariant argument checks; emit dispatch from semantic contracts.
  Reader imports also bind generic-owner fields. All seven Raven consumers execute (42),
  alongside 108 C# groups and .NET/both-container field/interface execution. External
  interface implementation declarations and constrained/variant profiles remain pending.

- Support imported fields on constructed generic root-class owners through
  scoped, output-owned field references. Preserve CLI open signatures and native slots.
  Seven Raven consumers and .NET/both-container field tests execute (42); 108 C# groups pass.

- Admit closed generic reference values and vectors in authored native field contracts
  on nongeneric owners. Preserve CLI signature/MemberRef and native ordinal encoding;
  reject open/bare-generic/foreign storage. C# metadata checks pass 108/108 groups and
  scalar/vector field execution passes on .NET and both native containers. Raven's
  cross-assembly holder consumer verifies replacement/aliasing; seven consumers run (42).

- Record Raven native host dependency binding without a separately supplied reader
  definition or image roundtrip. Legacy CLI/snapshot bindings remain available; the
  legacy Definition accessor throws for new native bindings. Seven consumers compile
  and execute (42), with invalid configuration rejected before output. No native format
  change; lazy semantic reader state and explicit core/bootstrap remain.

- Remove Raven native type/field reader fallbacks and its emitter-native resolver;
  all supported native references now require symbol contracts. Seven consumers execute
  (42). Library/runtime are unchanged; host input binding and lazy semantic loading
  still retain readers, and translated CLI compatibility lookup remains explicit.

- Enforce symbol-only native callable emission in Raven by removing its reader-definition
  fallback; incomplete contracts diagnose. Seven consumers execute (42), including direct
  concrete implementation calls. Clarify that native implementation flags differ from
  CLI projection flags; no metadata/runtime change or final-virtual extension is made.

- Validate authored static-container generic calls on CLR and both native containers.
  Raven now reconstructs these owners/methods from symbols while keeping static classes
  out of value signatures. No API or format changes; remaining reader-backed bindings
  are documented separately. All 108 C# groups and seven native consumers pass (42);
  API snapshot validation passes.

- Add the metadata library's IILGenerator with stable builder/attached-definition access,
  typed Emit overloads, helpers, locals and labels. Raven's NeoCLR adapter now uses it
  without changing shared compiler interfaces. Definition bodies remain canonical;
  body-authoring implementation now lives in the generator, with builder instruction
  methods forwarding as compatibility APIs. Writer-side validation remains unchanged. Loaded-body editing and instruction insertion are not added.
  All 108 C# groups and seven native consumers pass; generator-authored generic-owner
  code executes on CLR and both native containers (42). API snapshot check passes.

- Author nongeneric interface identities, direct conversion edges and abstract member
  references from semantic contracts. Derive transitive conversions and reject cycles
  or conflicting nominal classification. Raven now supplies interface relationships and
  dispatch flags from symbols; no metadata format change. Generic interfaces and richer
  class inheritance remain outside this slice. All 107 metadata C# groups and seven
  native consumers pass (42), including inherited dispatch and storage aliases; API
  snapshot validation passes.

- Author native instance-field references from explicit type/storage/readonly/slot
  contracts without reader definitions. Raven captures the ordinal during import in
  an optional compiler-owned layout interface and emits supported root-class fields
  from symbols. Exact artifact checks remain; ordinary .NET field addressing is unchanged.
  All 107 C# groups and seven native consumers pass, including alias writes (42);
  slot conflicts and readonly stores reject. API snapshot check passes.

- Author public nonvirtual root-class methods and constructors from output-owned type
  references and signatures without reader methods. Support generic-owner substitution;
  reject invalid constructor/scope/owner contracts and instance generic methods. Raven
  uses this path for the supported root-class members. Virtual/interface/value/nested
  profiles, fields and host setup remain reader-backed; the format is unchanged.
  All 107 C# groups and seven native consumers pass; authored generic-owner construction
  and member calls execute on CLR and both native containers (42). API snapshot passes.

- Extend authored native namespace-function references to output-owned external root-class
  signatures, including recursive generic constructions and vectors. Raven reconstructs
  these call signatures from symbols; member calls and richer type profiles retain the
  reader-backed route. Foreign output types and out-of-scope generic parameters reject.
  Graph validation now recognizes authored nominal call contracts. All 107 C# groups
  and seven native consumers pass (42); API snapshot validation passes.

- Author native top-level root-class references from resolved assembly/name/arity and
  artifact values without input definitions; generic construction keeps copied arguments
  and output ownership. Raven uses this path for public unconstrained root classes.
  Interfaces, inheritance, nested/value types and member references retain the prior
  path. No metadata format change; exact snapshot checks remain. All seven Raven
  consumers execute (42); 107 metadata groups pass, and the authored generic owner
  executes on CLR and both native containers (42). API snapshot check passes.

- Add native function-reference authoring from explicit identity, artifact digest,
  namespace/name and signature values, without reader definitions or resolvers.
  Primitive, method-generic and vector contracts retain existing name/signature linking;
  digest checks detect output-local snapshot conflicts, not runtime integrity.
  All 107 C# groups pass and the generic vector reference executes in both native
  containers (42). Raven now uses symbol-owned contracts on this path; all seven native
  consumers execute (42). Nominal signatures and reader-independent host setup remain
  pending; recorded exact revisions and artifact evidence.

- Record planned independent compiler import/emission contracts through Raven symbols
  and a separate metadata-library instruction-generator API. Inventory current loader
  coupling and scope the next symbol-only function-reference slice. This records
  direction; no API migration or runtime behavior change is claimed.

- Read native unconstrained static generic methods/functions into immutable definitions,
  preserving arity, names and method-parameter/vector signatures. Existing generic
  imports emit executable CLR and native calls (42 in both native containers; 104 C#
  groups pass). Nongeneric helpers reject unused generic parameters. Raven now imports
  this profile with method-owned parameters and shared inference/substitution; all seven
  native consumers execute (42). Unconstrained generic root classes now preserve owner
  parameter names and scoped signatures; imported construction and mutation execute on
  CLR and neoCLR (both containers), with 105 C# groups passing. Raven imports these owners
  through shared constructed-type substitution. Loaded local closed generic signatures
  now expose immutable ReferencedGenericType definitions/arguments and import recursively;
  factory/identity calls carrying Box<int> execute on CLR and neoCLR. Scoped local
  constructions such as Box<T> now retain method/owner parameters recursively; Raven
  inferred calls and vectors of open constructions execute using scope-owned caches.
  External generic constructions now retain exact dependency scope and arguments too;
  a three-assembly Raven consumer executes with both dependencies (42), and 106 C#
  metadata groups pass, including CLR forwarding and resolver rejection. Constraints
  remain pending. Shared Raven lookup now includes directly namespace-owned functions;
  qualified inferred/explicit generic calls execute (all seven consumers return 42),
  incompatible explicit arguments diagnose, and .NET controls retain their behavior.
  No format change.

- Close the shared Raven expression-bodied return diagnostic gap with an independently
  reproduced .NET fix (106 focused tests). Native unrelated interface returns now
  diagnose before emission; all six native import/emission consumers still execute (42).
  No metadata format or runtime changes; generic import and bootstrap gaps remain.

- Extend direct native reading to nongeneric top-level classes with primitive fields,
  methods and constructors. Preserve canonical declaration ownership, native origin
  tokens, visibility and readonly flags without inventing CLI signature blobs. Raven
  consumers execute overload calls, construction, stateful methods and direct public
  field loads/stores across native assemblies (42). Immutable imported field references
  emit CLI MemberRefs or validated native field ordinals; native field emission requires
  a native snapshot. Readonly stores and invalid receiver/operand contracts reject.
  Native method/function/constructor signatures now also retain local nominal class
  references in the immutable definition graph and import them into output-owned types.
  Raven factory, identity-call and nominal-constructor consumers execute (42).
  Primitive-only helpers reject nominal signatures. Local class-valued fields now also
  expose immutable signatures and import for CLI/native load/store emission; Raven
  replacement/mutation consumers preserve object identity (42). Native nominal method
  and field signatures now retain exact assembly-scoped references across dependencies.
  Resolver-taking method/field import overloads and Raven's explicit reference set resolve
  them without reflection or CLI projection; a three-assembly consumer executes (42).
  Missing/version/type/snapshot conflicts reject. Direct native method/constructor/field
  signatures now also support one-dimensional primitive and nominal class arrays,
  including externally resolved element types. Raven preserves array aliases and
  executes element replacement across native libraries (42). Generic/value/interface
  profiles remain outside direct reading. Non-indexed properties now load with canonical
  accessor definitions and logical primitive/nominal/vector signatures. Raven consumes
  instance/static properties across native libraries (42), rejecting read-only/private
  setter writes; a general setter-accessibility binder fix is independently tested on .NET.
  Indexed properties now retain ordered immutable parameter signatures through a new
  logical-signature overload. Raven binds overloaded native indexers and executes
  cross-library element replacement/read (42); inaccessible indexed setters diagnose.
  A separate shared Raven name-normalization fix closes a .NET emission crash exposed
  by the indexer regression tests. Raven now also assigns through setter-only native
  indexers using the shared property parameter contract, independently tested on .NET.
  Reads and compound assignments still require a getter; all native consumers pass (42).
  Direct reading now includes nongeneric interfaces, local inheritance and class
  implementations. Exact loaded relationships validate imported interface conversions;
  Raven binds inherited methods/properties and neoCLR dispatches across assemblies (42).
  Follow-on storage tests exercise interface-valued fields, constructor arguments and
  arrays in a second native library, preserving aliases and dispatch after replacement
  (42). The existing compiler/metadata paths require no additional implementation.
  Invalid interface returns reject emission with empty output; earlier expression-body
  diagnostics are tracked for independent .NET investigation.

- Raven imports bounded primitive namespace-function libraries directly from native
  definitions into compiler-owned symbols and emits their calls through the metadata
  builder's callable-reference API. Preserve exact identity, overloads and accessibility;
  reject conflicting native snapshots using cached image fingerprints instead of absent
  MVIDs. API-authored and Raven-authored library consumers execute in neoCLR (42)
  without a CLI dependency projection. C# metadata, target-emission and symbol checks
  pass. The explicit CLI primitive core remains a bootstrap; broader native importing
  is pending.

- Read native primitive namespace functions directly into the existing assembly/module/
  method definitions through ReadNativeAssembly, without a CLI projection round trip.
  Preserve exact references, namespace ownership, entry identity and original PE bytes;
  expose logical signatures with TryGetSignature. Unsupported nominal/generic declarations
  fail explicitly. Both container encodings and existing dependency resolution pass in
  96 C# metadata groups. Raven symbol integration and wider declarations remain pending.

- Record the author-directed priority of native metadata semantic import into Raven,
  fulfilling the existing builders/definitions/metadata/PE architecture. Audit native
  reader materialization and reflection-owned compiler setup; sequence definition
  reading, direct symbol import and cross-assembly execution. This is an implementation
  plan, not completed native importing. Reuse the existing exact-identity resolver;
  dependency loading must not recreate the .NET reflection API.

- Extend unchanged source collection execution to internal reference payloads, proving
  map/list/filter identity, mutation visibility, replacement independence and iteration
  after growth (42). Record broad application assessments with translated and source
  queries: the remaining blocker is mixed source/seed iteration identity. No full
  source bootstrap or query-operator emission is claimed.

- Compile unchanged HashMap, its collection interfaces, ArrayList and equality-policy
  implementation together into native PE and execute collision/growth, duplicate,
  update, key-snapshot and Option lookup checks (42). Shared interface planning now
  respects explicit imported signature capabilities; .NET defaults are unchanged.
  Translated System remains a dependency, so this is not full library bootstrap.

- Validate unchanged Raven callback comparer sources through native PE execution (42).
  Raven now applies target capabilities to explicit instance field declarations, using
  existing generic Function storage and interface dispatch. Capturing closures remain
  outside this bounded check; no runtime or metadata encoding change is needed.

- Bind explicitly core-marked CLI namespace containers to native assembly-level functions
  while preserving CLI reference identity. Reject unmarked or wrongly scoped containers
  and incompatible signatures. All 95 metadata contract groups pass. Raven now compiles
  unchanged ArrayList and its interfaces to native PE and executes growth, independent
  copies, iteration, callback searches and Option results, plus expected failure paths.
  Target-configured array length and an isolated shared required-result fix complete
  this bounded source checkpoint; full System source emission remains open.

- Add native-only MethodBuilder.ReserveArray and typed raw ReserveArray emission,
  preserving checked uninitialized slots through PE/#Neo loading. Executable CLI writing
  rejects the operation; reference projections remain supported. Explicit Raven bootstrap
  binding emits the operation, with stored-slot execution returning 42 and unread slots
  faulting. All 94 metadata groups pass. Unchanged ArrayList emission now reaches its
  System.Fail namespace dependency; the broader collections sample retains exact output.

- Admit generic root-class implementations of owned interfaces, including constructed
  owner arguments and transitive substitution. Metadata stack checks validate the
  actual generic receiver before interface dispatch; CLI/native C# execution returns 42
  and all 93 metadata groups pass. Raven generic providers and iterators execute against
  unchanged Sequence contracts. An explicit authoring seed binds unchanged ArrayList;
  native emission now reaches the missing CheckedStorage.Reserve intrinsic mapping.

- Support owned constructed interface inheritance and transitive positional argument
  substitution in the metadata producer/reader, plus constructed interface CallVirtual.
  Cycle, scope and implementation checks remain enforced. Unchanged Raven collection
  contracts through Sequence<T> now compile with a consumer whose inherited Count
  and indexer dispatch returns 42 on CLR/native and the actual neoCLR target profile,
  in both source orders. Raven admits interface indexers through a shared capability;
  separately isolated general fixes prevent incomplete source interface/member caches
  and infer abstract bodyless indexer accessors. The broad collections application
  retains exact output. Full library implementation bootstrap remains pending.

- Add explicit CLI declaration/native implementation bindings to the metadata API,
  preserving PE reference scopes while validating selected native signatures. Support
  local core TypeRefs/Function carriers and inhabited Void storage/result adaptation.
  Matching runtimes accept the new manifest mapping fields; older experimental bundles
  must be updated together. The unchanged Raven collections application now emits native
  PE, verifies and runs with exact expected output against translated System (exit 0).
  91 C# metadata contract groups and native binary fixture validation pass.

- Preserve access to caller-supplied internal types when generic library code invokes
  a structural Function. Open signatures still reject explicit foreign internal
  types; a cross-module regression reproduces the former execution fault and now
  returns 42. This aligns Function specialization with nominal generic owner checks.

- Add Raven's opt-in shared reference-iterator lowering. The unchanged collections
  sample now completes body planning and reaches explicit native System dependency
  linkage; no full native application execution is claimed yet.

- Connect Raven reference conversions to native castclass and preserve physical
  union-case identity across carrier views. The unchanged collections sample now
  passes these boundaries and stops at iterator for-loop lowering.

- Add typed `CastReference`/raw `Castclass` metadata emission for nominal references
  and vectors, with reference-only stack checks and standard CLI/native castclass
  encoding. C# consumers retain interface dispatch and return 42 on both runtimes;
  all 89 metadata contract groups pass.

- Extend Raven's opt-in shared plan with checked union-case branches and payload
  extraction, compiler-generated match failure and concrete value override calls.
  34 focused compiler tests pass; collections now reaches reference conversions.

- Admit concrete imported value-type overrides as direct managed-receiver calls,
  matching CLR value dispatch. Nonfinal reference overrides remain rejected.
  C# coverage executes a real CLR override and a matching metadata-produced native
  library (expected output and 42); all 88 metadata contract groups pass.

- Enable already-lowered static extension calls in Raven's native adapter under an
  explicit capability. The unchanged collections sample passes Single admission and
  now reports unsupported union-pattern emission; no native sample output is claimed.

- Connect Raven's native adapter to structural Function signatures, static binding
  and invocation. A directly emitted Raven PE passes a callback through a function
  and executes to 42. Noncapturing lambdas also compile and execute directly;
  five existing native profile controls continue to pass.

- Add structural Function signatures and checked static binding/invocation to the
  metadata API, including raw typed operands, exact shape identity, generic substitution,
  native Function encoding and CLI Func/Action transport. Imported callable signatures
  round-trip across the explicit core scope. C# consumers execute callbacks across a
  library boundary on CLR and neoCLR (42), including generic calls and no-result callbacks.

- Integrate structural Function runtime/library work from codex/structural-types into
  the metadata feature branch at the author's direction. Preserve System.Fail and
  binary PE loading; regenerate the matching class library/reference. Native Function
  signatures replace nominal delegates here; ordinary .NET remains delegate-based.
  65 focused native tests, 86 metadata groups, five direct native controls and the existing
  Function CLI-import consumer controls pass. Direct Raven metadata emission of Function
  values remains the next integration step.

- Import nested public metadata identities with enclosing TypeRef scopes, and author
  generic nested value types under nongeneric owners. Preserve scope in equality,
  substitution, native references and CLI projections. Separate-library metadata and
  Raven consumers execute to 42; 86 metadata groups and 30 compiler tests pass.
  Collections advances to the Single signature; application execution remains pending.

- Author nested nongeneric class/value definitions through definition collections and
  builder helpers, preserving CLI NestedClass rows and explicit native declaring owners.
  Validate ownership, visibility and native/CLI origin agreement; nested constructor
  execution returns 42 on CLR and neoCLR. Generic nesting and external nested imports
  remain subsequent work toward the unchanged collections sample.

- Support metadata-authored value constructors with field-assignment and construction
  receiver checks, plus imported public class/value constructors on generic owners.
  Raven opts native emission into imported constructors explicitly. Separate-library
  metadata and Raven consumers execute successfully (42); 84 metadata groups, 29 compiler
  tests and five native controls pass. Collections remains blocked by the nested
  System.Option.None signature; nested identities are not flattened or silently admitted.

- Add literal terminal failure to the metadata body API through `Fail(string)` and raw
  `Emit(OpCode.Fail, string)`. Native execution uses the existing UserFault instruction;
  CLI execution throws InvalidOperationException with the diagnostic. Validate empty-stack
  termination, unreachable code and output assignment only on normal returns. This does
  not add native exception handling or change the runtime System.Fail API. Raven marks
  generated propagation guards explicitly and preserves .NET null-throw behavior; the
  unchanged collections sample advances to imported carrier construction from None.
  All 83 metadata groups and 14 focused Raven tests pass.

- Rename the public terminal namespace function from `System.Fault(message)` to
  `System.Fail(message)`, distinguishing the action from the resulting host `Fault`.
  Migrate source consumers, Raven terminal-call recognition, CLI reference metadata and
  generated runtime implementations together. Callers must rebuild against the matching
  compiler/reference/runtime bundle; no public old-name alias is retained. Fault result
  types, codes, the low-level fault instruction and internal runtime binding are unchanged.
  Fresh library regeneration also updates one FileSystem helper's local layout from the
  current compiler; native enumeration/count/limit checks pass. Eight compiler tests,
  four admission cases, seven runtime tests and qualified/imported consumers pass.

- Integrate managed value receivers into Raven's shared emission plan and native adapter.
  A Raven consumer verifies and executes value mutation and generic output calls against
  a separate native library (42). The unchanged collections sample advances to a lowered
  throw guard; full propagation execution remains pending. Record the explicit receiver
  capability, unchanged .NET defaults and next terminal-failure contract requirement.

- Support initialized managed receivers on metadata-authored value instance methods and
  imported nongeneric methods on ordinary/generic value owners. Preserve receiver_byref
  through native projection and direct CLI calls; value constructors and constrained
  interface dispatch remain unsupported. C# library/consumer tests prove mutation and
  generic out calls on CLR/neoCLR (42), including receiver/output alias rejection.

- Connect Raven ref/out emission through the shared portable codegen plan and explicit
  adapter capabilities. Five native controls now pass, including source output forwarding
  and ref mutation; 64 focused C# tests pass. The unchanged collections sample advances
  from uninitialized-local admission to imported value-receiver TryGetOutput admission.
  Record the remaining propagation gap without changing Runtime Contracts or runtime IL.

- Reassess unchanged Raven samples after ref/out metadata support: all four small native
  execution controls pass, four library inventory attempts emit and verify, but all twelve
  selected applications still stop before native execution. Record the collections
  out-local admission gap, dependency configuration and implementation-bootstrap limits;
  the assessment does not claim new compiler support.

- Add explicit out-parameter contracts to experimental method signatures, CLI Param
  Out flags and native out_parameters metadata. Readers, imports, generic substitution
  and reference projections retain the contract. Producer flow validation requires
  assignment on every normal return and checks all ref inputs before publishing out
  assignments. Separate library/consumer binaries execute on CLR and neoCLR (42);
  conditional outputs, readonly references and Raven propagation admission remain open.

- Add writable managed-reference method parameters to the experimental metadata API:
  standard CLI BYREF signatures and native ByRef types survive generic substitution,
  imported calls and CLI projection. Initialized local addresses and forwarded ref
  arguments execute through separate library/consumer binaries on CLR and neoCLR (42).
  Out assignment contracts, readonly references, byref returns/locals and Raven admission
  remain unsupported; this adds no runtime opcode or wire-format extension.

- Add typed `ldobj`/`stobj` emission and `LoadObject`/`StoreObject` helpers to the
  experimental metadata API for owned local addresses, with exact-type and path-sensitive
  assignment checks. The C# consumer exercises generic copies and branch-merged updates
  on CLR and native binary loading/verification/execution (42). Byref signatures and
  Raven propagation calls remain pending; no runtime instruction or format change.

- Record the shared Raven concrete-case lowering fix and refreshed unchanged collections
  probe: normal CLI emission succeeds; native emission advances from masked propagation
  rejection to synthesized out-local admission. The general fix is isolated on Raven's
  main-based compiler-fixes branch and passes 25 focused tests on both compiler lines.
  Managed-reference/byref emission remains pending; no metadata/runtime expansion is
  claimed by this checkpoint.

- Emit imported constructed interface calls and final virtual class calls using standard
  CLI MemberRef/TypeSpec and callvirt contracts. Closed generic interface implementations
  share authored definitions with builders and survive native reference projection.
  Separate metadata library/consumer binaries return 42 on CLR and neoCLR; a Raven
  consumer also returns 42, with missing-dependency rejection and null receiver coverage.
  The unchanged collections sample advances past TryAdd/Add to propagation-expression
  lowering. General class virtual overrides, generic interface inheritance and union
  execution remain open; no new native opcode or runtime workaround is introduced.

- Preserve imported value categories through CLI signatures, generic substitution,
  native projection and runtime dependency validation. Separate library/consumer cases
  execute on CLR and neoCLR (42), and a Raven consumer passes. Native manifests may now
  include value_type_references; images using it require the updated runtime, while
  legacy images remain supported. The unchanged collections sample advances past
  Option<Order> declaration admission to an unsupported lowered invocation.

### 2026-10-01

- Record the author's return to Raven/native end-to-end work: defer broad metadata API
  migration unless integration requires it. Refresh the unchanged collections probe;
  binding/CLI control emission succeed, native emission still rejects Option<Order>.
  No new emission/runtime support is claimed by this investigation checkpoint.

- Advance definition-first metadata authoring: direct assemblies, types, fields,
  functions, static/instance methods, constructors and interface contracts share
  declarations with builders. Interface relationships and method-body instruction,
  local and label storage now belong to definitions. CLR/native execution covers
  object creation, readonly initialization, inherited dispatch and struct storage (42).
  Body clearing preserves local/label handles. Authored collections are append-only;
  Types, type Fields/Methods and module Functions expose IList (development API change).
  Properties and accessor associations now also share authored declarations with builders;
  CLR reflection and native execution return 42. Type.Properties changes to IList.
  Direct generic type construction now shares parameter names and constraint storage
  with builders; constrained constructed calls return 42 on CLR/neoCLR. Separate
  generic-parameter objects, arbitrary instruction editing and loaded editing remain pending;
  CLI/CIL encoding is unchanged. Record the author’s distinction: Cecil-like definitions
  with Reflection.Emit-style convenience builders, without drop-in compatibility claims.
  Record the clarified builders → definitions → metadata → PE pipeline and reverse
  reader boundaries as architecture work; encoding/packaging separation remains incomplete.

- Record the author's definition-first metadata architecture and planned refactor:
  directly editable definitions, optional builders over the same graph, and definition-
  driven writers. The current snapshot/producer split remains an acknowledged gap;
  prioritize unification before further builder-only features. Add a primary-source
  Cecil alignment map for lifecycle, definitions/references, ownership, imports and
  instruction editing; no drop-in compatibility or implementation completion is claimed.

- Add generic payload fields and initialized local-address field access for producer
  value types. CLR/native tests preserve value and array copies plus reference payload
  aliases. Invalid receiver types, uninitialized locals and stores into value copies
  reject; native lowering discards the legacy store's Void result to retain CLI semantics.

- Add bounded owned value-type declarations to the metadata producer, preserving
  VALUETYPE/GENERICINST categories through CLI output, native loading and projection.
  Defaults, primitive field reads, generic forwarding and arrays execute on CLR and
  neoCLR (42); special constraints distinguish values from references. Generic payload
  fields, addressed mutation, instance methods and value-type imports remain unsupported;
  this does not yet advance the collections Option<Order> gate.

- Import dependency-local reference types/constructions in static method signatures
  through the metadata producer and Raven target. CLR factory/reader and native Raven
  generic factory/payload-alias tests execute successfully. Bounded decoding rejects
  malformed/value-type/unsupported TypeRef signatures; collections Option<Order>,
  instance members and translated-System identity mapping remain open.

- Expand imported static generic calls to consumer-owned/imported reference arguments
  and caller generic parameters, checking ownership at construction and scope on emit.
  Standard MethodSpec shape remains unchanged. CLR and Raven-to-neoCLR probes execute
  nominal alias mutation and generic forwarding; collections union imports remain open.

- Integrate external reference signatures in Raven through an explicit target capability.
  Separate Raven-produced library/consumer binaries verify and return 42 in neoCLR.
  The unchanged collections sample advances past Register to PendingOrder’s unsupported
  Option<Order> signature; imported members and translated-System mappings remain open.

- Add immutable external class/interface references and generic constructions to the
  metadata producer. Preserve standard CLI TypeRef/GENERICINST/TypeSpec and native
  dependency-scoped signatures through fields, locals, defaults and generic substitution.
  Separate CLR/native library and consumer tests return 42. Imported member calls and
  translated-System identity mapping remain open; Raven collections emission is still
  blocked. Null-literal ImportReference calls now require an overload-selecting cast.

- Reaffirm unchanged sample execution as the main native-emission acceptance gate.
  Rerun order-collections after generic imports: binding/CLI control emission pass,
  but direct emission still rejects imported nominal/generic signatures in Register
  and writes no native image. Record that limit without treating focused probes or
  earlier legacy-bridge execution as completion of the direct target.

- Import bounded unconstrained static generic methods with primitive/vector and
  scoped method-parameter signatures. Add immutable imported instantiations and
  typed Call/raw Emit support; CLI uses MethodSpec/MemberRef and neoCLR uses its
  existing generic-call format. Reject constraints, malformed scope and unsupported
  arguments. Separate C# CLR and Raven native-profile library/application cases pass.
  Record the author's .NET baseline and full Cecil-like read/edit/create/write direction;
  general editing of loaded snapshots remains unimplemented.

- Extend the experimental read-only callable decoder, imports and member resolution
  to Int32/Int64/Boolean/String vectors using standard CLI SZARRAY signatures.
  Preserve scalar-only recognizers and reject malformed, nominal and generic imports.
  C# tests execute separate ordinary CLR images; Raven's native-profile library and
  application verify/run through binary loading, including array alias mutation.

- Integrate Raven's independent union-case contextual-typing correction into local
  compiler main (`46491585e`) and the target branch. The unchanged collections sample
  now imports, verifies and matches expected output; assembled binary App and System
  also load/verify/run. Preserve the distinction from direct metadata emission, which
  still rejects imported generic collection signatures. Record 339 passing C# cases.

- Record Raven's bounded .NET refactor-parity audit and validated neoCLR-profile
  binary emission: Hello World/function calls, Unit entry, arrays and owned interface
  dispatch verify/run without a host core reference. Exact primitive/Unit core
  identity is checked before output. Preserve open carrier-binding/capture issues;
  implementation bootstrap and native metadata symbol loading remain unimplemented.

- Record six independent Raven parser, binding and CLR-emission fixes extracted into
  a dedicated main-based branch and integrated into local Raven main (`e5607ca17`).
  All 23 regression and 174 surrounding cases pass; 16 regressions fail on the base.
  Native target work and broader lowering candidates remain separate; no push performed.

- Assess direct metadata-backend readiness with unchanged library/application sources
  and reproducible legacy-bridge controls. Record the native target-profile gate,
  missing implementation-bootstrap declarations and an emitted Option constructor
  mismatch in order-collections. Three existing application controls run successfully;
  the assessment proposes larger integration milestones without claiming a full build.

- Add owned nongeneric interface implementations and CallVirtual/raw Callvirt to the
  metadata API. Preserve standard CLI InterfaceImpl and virtual/final implementation
  flags; use existing native interface lookup and callvirt. Raven shared emission now
  executes interface methods/properties through two classes and array references in
  both file orders; missing implementations and invalid receivers reject, null faults.

- Extend owned nominal signatures and generic constructions to interfaces; preserve
  CLI/native projection and execute typed default-reference flow. Raven now compiles
  unchanged Iterable<T> alongside iterator contracts in both file orders. Interface
  implementation/dispatch remains separate work.

- Add invariant interface declarations and public abstract instance contracts to the
  metadata producer. Preserve standard CLI flags/bodyless methods and existing native
  Interface identity through reference projection. Raven now emits unchanged Comparer
  and EqualityComparer sources; binary loading/verification passes. The independent
  entry returns 42; interface implementation/dispatch is not yet exercised by this API.
  Extend declarations with owned nongeneric interface inheritance and abstract property
  associations, including unchanged Disposable/Iterator source. Reject cycles and
  malformed associations; accept canonical empty field origins on property-only types.

- Compile the whole unchanged Raven Language class through the metadata target after
  shared static-property accessor support. Both runtimes print und/sv/he and return
  42 in both source orders. Refresh library inventory: comparer interfaces bind but
  await interface declaration emission; ArrayList still needs native dependencies.
  Reaffirm .NET behavior and supported CLI instruction semantics as the default;
  the temporary payload bridge does not define a separate instruction set.

- Add owned nominal class bounds for declaring-type parameters to the metadata producer.
  CLI GenericParamConstraint and native TypeBound retain exact bounds in reference
  projections. Concrete invalid arguments reject, including runtime verification of
  corrupted call metadata. Earlier uses are revalidated when bounds change. This does
  not add interface/dependent bounds. Extend producer/reader and native enforcement with
  distinct reference/value/default-constructor requirements and CLI GenericParam flags.
  Raven class/struct/new type declarations execute on both targets; matching runtime
  required for new native constraint kinds. No symbolic new T() or constrained dispatch.
  Record the bounded integration assessment and remaining class-library gates.

- Add immutable constructed-field references and typed load/store/raw emits. CLI field
  MemberRefs preserve open signatures on constructed TypeSpecs; native field operations
  retain exact receiver identity. C# tests reject wrong receiver/owner scope and binaries
  execute 42. External assembly fields remain unsupported.

- Support properties and indexers on generic metadata owners, preserving declaring-type
  parameter scope in value/index signatures and accessor associations. Native reading
  rejects malformed owner instantiations and retains associations in CLI projections;
  API binaries verify/run 42 without a runtime format change. Raven now emits generic
  instance properties/indexers through the shared accessor paths; two-parameter key/value
  indexers and alias mutation execute on both runtimes/source orders. Matching reader required.

- Add generic reference-class construction to the metadata producer: typed fields,
  constructors, instance methods and nested constructed signatures preserve distinct
  owner/method scopes. CLI/native binaries verify and return 42; native reference
  projection retains generic field signatures. Exact receiver/field scope and 16-level
  nesting limits reject unsupported contracts. External constructed field handles,
  generic properties and constraints remain deferred. Raven now shares generic class
  values, storage and construction across adapters; primitive, object and nested
  Box values execute on both runtimes in both source orders.

- Add bounded static generic type definitions and constructed-owner method references
  to the metadata producer. Preserve independent VAR/MVAR scope, sorted GenericParam
  rows, TypeSpec/MemberRef/MethodSpec calls and native constructed owners through CLI
  reference projection. API binaries verify/run 42; generic object layouts remain out
  of scope. Raven now shares static generic owner planning and calls across targets;
  its consumer runs 42 in both source orders with owner arrays/defaults and independent
  method arguments. Cross-scope forwarding and reordered owner arguments also pass;
  malformed native owner scopes are rejected. No native runtime format change is required.

- Add typed local addresses and initialization to the metadata API (`Ldloca`, `Initobj`,
  `LoadDefault`). Validate local identity, exact initialization type and definite
  assignment; addresses cannot escape through the bounded value signatures. Ordinary
  CLI/native instructions initialize generic primitive, string, object and vector
  defaults; producer binaries verify/run 42 on neoCLR. Raven shares default-value
  planning and capability admission across adapters. Generic clearing executes on both
  runtimes; dereferencing a cleared reference element raises a null-reference fault.

- Extend generic metadata calls to ordinary instance methods on owned root classes.
  Preserve receiver identity and generic parameter scope in CLI MethodSpec and native
  call records/reference projections. Binary producer verification/execution returns 42;
  native admission retains nonvirtual IL receiver and nongeneric-constructor limits.
  Raven now opts into an explicit instance-generic capability through shared planning;
  its binary Order consumer passes receiver mutation/forwarding on both runtimes.
  Expanded acceptance covers generic no-result copy/reverse, recursive instance calls,
  receiver/argument order and independent receivers. Runtime tests force collection
  during a generic call; API checks reject wrong receivers and generic constructors.

- Add unconstrained generic function/static-method declarations with named method
  parameters, CLI GenericParam/MVAR signatures and native MethodTypeParameter records.
  Generic locals and vector element tokens preserve scope and reference projections;
  invalid scopes and open calls reject. Add immutable generic call instances, ordinary
  CLI MethodSpec tokens and native generic arguments, including forwarding through
  caller parameters and typed array factories. API-produced binaries verify/run 42;
  permit generic static class methods without relaxing instance receiver restrictions.
  Raven now shares generic signature/body planning for owned static calls; the Order
  generic consumer executes on both targets in both source orders (42). Expanded
  acceptance covers inference, recursion, multiple type parameters, generic vector
  creation/iteration and conditional values; unsupported native contracts reject
  without output. C# API checks cover raw calls, copied arguments and stack mismatches.

- Extend metadata property associations to indexed signatures, inferring copied index
  parameters from accessors and validating getter/setter agreement. Preserve ordinary
  CLI property parameters and native parameter lists, including overloaded indexers
  and read-only associations, through reference projections. API-produced binary
  assemblies now verify and execute overloaded getters and multi-index setter-only
  associations on neoCLR, returning 42. Existing runtime instructions suffice. Raven
  now consumes indexed properties through shared accessor planning and explicit
  capabilities; overloads and multi-index properties execute on both targets. Expanded
  Order collection acceptance covers nominal/array indices, alias mutation, evaluation
  order and bounds faults; correct the probe evidence labels to the actual checks.

- Add bounded vector signatures for primitive and owned root-class elements across
  parameters, results, locals, fields and properties. Preserve CLI SZARRAY and native
  ArrayRef identities in reference projections. LocalDefinition.SignatureType exposes
  the full slot type; primitive/class projections are null for array slots. Allocation
  and element instructions now expose typed helpers and raw Newarr/Ldelem/Stelem/Ldlen
  emits. Binary runtime tests cover primitive/nominal aliasing, empty arrays, negative
  lengths and bounds faults; raw ldlen retains native unsigned width before conv.i4.
  Raven now consumes vectors through shared signature/storage/body contracts; the
  Order-array consumer executes on both runtimes in both source orders, returning 42.
  Shared array-loop lowering now covers iteration, single collection evaluation and
  nested/labeled transfers. Fixed-length array contracts remain explicitly gated rather
  than losing their metadata; focused Raven tests and the expanded binary consumer pass.

- Support readonly primitive/nominal instance fields in the independent metadata API,
  CLI InitOnly flags and native reference projections. Enforce declaring-constructor
  direct stores and readonly managed addresses in verification and execution. Existing
  missing/false flags remain mutable; raw unmanaged memory is outside these guarantees.
  AddField gains optional isReadOnly and FieldBuilder exposes IsReadOnly: rebuild host
  consumers and use the updated runtime, since older runtimes do not enforce these flags.
  Raven now emits private val storage and stored val properties with preserved flags.

- Add owned nominal method/function parameters and results with exact stack identity,
  ordinary CLI CLASS signatures and existing native Named records. The native reader
  preserves them in reference projections; API-produced binary assemblies execute on
  neoCLR. Development API migration: builder methods and Signature properties now use
  MethodSignature/SignatureType; PrimitiveMethodSignature remains a compatible primitive
  construction helper. Rebuild consumers and inspect Primitive/ClassType explicitly.
  Raven now consumes the shared logical signature contract for factories, aliases,
  nominal overloads, self-return and constructor parameters on .NET and binary neoCLR.
  Extend the same identity model to mutable nominal instance fields, preserving CLI
  CLASS and native Named field signatures and exact store validation. FieldBuilder.FieldType
  now uses SignatureType; rebuild development consumers. API-produced binaries verify/run
  with stored-object alias mutation. Raven shares field load/store planning for explicit
  nominal fields and private storage, including initializers, validated on both runtimes
  in both source orders. Extend the same SignatureType model to property associations
  and native accessor references, preserving CLI property signatures and private setters.
  Raven emits nominal auto/computed/explicit properties and corrects stale provisional
  auto-property initializers while retaining canonical field identity. External nominal
  imports, nullability and generic signatures remain unsupported.

- Record the shared Raven fix for accessible explicit setters on `val` properties:
  owner writes invoke the setter while outside writes remain rejected. This corrects
  compiler binding without changing the runtime or metadata format.

- Add owned root-class metadata locals with exact nominal stack validation and
  aliasing on CLI and binary neoCLR. Development API migration: LocalDefinition.Type
  is nullable; ClassType identifies nominal slots. Primitive local behavior is unchanged.

- Record Raven's shared root-class and instance-method declaration contracts and
  receiver-aware primitive body planning. Raven now emits the unchanged Order class
  with explicit primitive constructors, mutable auto-properties and shared compiler-
  synthesized accessor bodies. Both source orders verify/run on binary neoCLR and
  .NET, returning 42. Raven now also emits owned root-class locals and validates
  property mutation through aliases on both runtimes. Ordinary nonvirtual instance
  calls now share receiver-first evaluation, including private nested calls and
  no-result mutation. Private mutable primitive storage now emits only a field,
  with qualified reads shared across backends. Computed properties and implemented
  get/set accessors now share body lowering, preserving private setter visibility and
  optional backing storage. Explicit root constructors also admit expression bodies,
  with overload and argument-order runtime validation. Explicit root `base()` now uses
  the existing backend initialization policy after checking its bound System.Object
  constructor. Preserve canonical private-storage fields during Raven rebinding so
  forward initializers reach emitted storage. Side-effecting initialization and both
  constructor body forms verify/run on .NET and binary neoCLR; user-defined base calls
  remain unsupported.
  Default root constructors and primitive field/property initializers now share
  canonical initialization with .NET. Explicit mutable primitive fields now preserve
  public/internal/private access through Raven emission, without property rows.
  Nullable/external locals and chaining remain gaps.

- Add non-indexed primitive property associations to the metadata producer and native
  reference projection. Preserve CLI Property/PropertyMap/MethodSemantics and accessor
  visibility; reject incompatible, reused or missing accessors. Add owned read-only
  Property snapshots with copied signatures and exact local accessor identity. Reject
  out-of-owner accessor associations and bound property/semantics counts. Property-bearing
  binary assemblies verify and execute in neoCLR. Raven source property emission is
  still pending; new property output requires the matching bounded metadata reader.

- Add root-class and primitive instance-field metadata production, native/reference
  roundtrip and owned Field snapshots. Preserve ordinary CLI type/field flags and
  tokens; validate binary runtime loading. Add primitive root constructors, nonvirtual
  instance calls, Dup/Newobj/Ldfld/Stfld and typed receiver validation. One API graph
  executes construction, field mutation and aliasing on .NET and binary neoCLR. CLI
  root constructors initialize System.Object before the declared body. Properties,
  inheritance and nominal signatures/locals remain unsupported; older bounded readers
  require updating.


- Preserve namespace identity on ownerless metadata functions and in runtime loading.
  Add the explicit namespace overload and namespaced import contract. The temporary
  CLI projection reserves `<NeoFunction>` names; existing global encodings are unchanged.
  New namespace-bearing artifacts require the matching runtime/metadata reader.
  Raven now emits namespace functions through shared admission and executes the
  original integer Math declarations on both .NET and binary neoCLR.

- Record actual Raven class-library emission probes and select order-collections as
  the broad acceptance target. Selected integer Math functions now emit and execute;
  whole-file dependency binding remains incomplete. The unchanged Order declaration
  now isolates the native object-emission gate. Its probe exposed a shared Raven bug:
  implicit property accessors/backing fields are now stable across repeated binding,
  validated independently on .NET with 49 focused tests.

- Add public/internal assembly-function visibility to the independent metadata API,
  preserving CLI/native access and ownerless definitions. Binary runtime checks prove
  internal calls/entries work and external calls require a public facade. Raven now
  preserves explicit public/internal and default internal access through shared
  capabilities; previously widened default functions become internal.

- Add shared Raven emission of primitive conditional values; verify binary execution
  with typed branch joins, nested selections and skipped faulting/side-effecting branches.
  Value blocks now permit initialized locals, assignments and calls before their result;
  internal if/loop prefixes now share statement emission, while returns, outgoing jumps
  and disposal remain explicit bounded limitations.

- Support exact Boolean operands for native and/or/xor in runtime execution and
  verification; retain mixed-type and Boolean arithmetic rejection. The independent
  metadata writer now accepts exact Boolean operands through raw emits and helpers.
  Raven now emits eager Boolean operators through the shared body plan, with paired
  truth-table and operand-evaluation-order checks on .NET and binary neoCLR.

- Enable Raven expression-bodied functions and static methods through shared compiler
  lowering on .NET and neoCLR. Verify binary loading/execution, separate-library
  references, Unit calls and precise rejection of unsupported expressions.

- Preserve public/internal/private static method visibility in the independent metadata
  API, CLI output and native projection. Binary runtime checks enforce assembly/type access.
  Integrate Raven static helper access through shared declaration capabilities; validate
  paired .NET/native execution and compiler/runtime rejection across assembly boundaries.

- Add typed Int32/Int64 AND, OR and XOR to the metadata writer and shared Raven
  emission on both targets, using existing CLI/native instructions. Add left and signed
  right shifts with Int32 counts; retain CLI-unspecified/native-masked out-of-range counts.

- Add typed signed division to the independent metadata API and Raven native emission.
  Reuse CLI/native div encodings and runtime zero/overflow faults for Int32 and Int64.
  Extend the shared emission path and writer API with signed remainder using rem;
  validate dividend signs, matching widths and execution faults.

- Add public/internal static type visibility to the independent metadata builder,
  CLI output and native reference projection. Raven emits internal helpers through
  its shared type plan; native runtime loading uses the existing visibility contract.
  Integration also fixes Raven qualified-type accessibility for ordinary .NET references.

- Raven backend profiles now admit logical declaration categories independently:
  assembly functions, static methods and static types. Preserve native ownership
  and the temporary CLI carrier representation without a metadata format change.

- Raven now uses immutable backend instruction/type capability profiles to admit
  shared body plans before native builders are allocated. .NET admits shared signed
  division; the native producer explicitly rejects it until its writer supports it.
  This changes no metadata encoding or runtime format.

- Add checked Starg/StoreArgument to the independent metadata writer, preserving
  by-value slot types and caller values. Raven source parameters stay immutable. Use the
  existing CLI/native instructions without a schema extension. Record the author’s
  CLI compatibility baseline and deferred codegen performance measurement.

- Add String signatures/locals and typed Ldstr emission to the experimental metadata
  API, including Unicode validation, native reference projection and imports. A native
  stack-consuming WriteConsoleLine overload enables Raven text helpers and computed
  console output, including cross-assembly calls. Reuse existing runtime String/ldstr
  support; nulls, equality and concatenation remain outside this writer subset.

- Raven native emission now coalesces partial static classes by semantic identity,
  preserving cross-part methods and rejecting unsupported members in any part.
  C# end-to-end checks verify both file orders on .NET and binary neoCLR assemblies;
  the independent metadata API and runtime format are unchanged.

- Raven now shares an explicit primitive value/no-result type contract between
  callable signatures and locals, with separate .NET/native mappers. Native local
  emission no longer depends on the callable builder's mapping helper. Preserve the
  selected .NET core and existing native assembly format; general type support is pending.

- Add operand-free Neg/Not to the experimental metadata API for signed Int32/Int64
  values, with wrapping negation and width-preserving complement. Reject Boolean and
  empty-stack operands before writing. Raven shares unary +, - and ~ across both
  backends; native emission reuses existing runtime instructions without a schema change.

- Add Int64 primitive signatures, locals and exact long constants to the experimental
  metadata API, plus unchecked signed Int32/Int64 conversions and matching-width
  arithmetic/comparisons. Preserve types through native projections and imports;
  replace Boolean stack tags with explicit primitive types. Raven shares this support
  across both backends. Int32-only convenience APIs and entrypoints stay unchanged;
  older experimental readers may reject Int64 declarations.

- Add checked operand-free Pop to the experimental metadata API. Raven's shared
  body path can discard Int32/Boolean call results in statement position while
  preserving call side effects and no-result Unit stack behavior. Native execution
  uses the existing pop instruction; no metadata schema change is required.

- Add immutable Int32/Boolean/no-result primitive signatures to the experimental
  metadata API, with typed declarations, imports, MemberRef resolution and native
  reference projections. Keep Int32-only convenience/recognition contracts and
  parameterless Int32/no-result entrypoints. Raven shares these signatures across
  both backends, including imported overloads; older experimental readers may reject
  Boolean declarations. Extend the same primitive type contract to local slots,
  including DeclareLocal/LocalDefinition.Type, CLI/native local signatures and typed
  store validation. Raven supports Boolean local initialization, assignment and equality.
  This consumer also exposed a general Raven assignment-RHS parser defect; the
  consumer branch fixes it with independent parser/assignment tests. Raven now
  emits short-circuit Boolean &&/|| through shared branches, with skipped-side-effect
  execution coverage on .NET and binary assemblies loaded by neoCLR.

- Add Int32 local declarations, method-owned local handles and raw Ldloc/Stloc operands
  to the experimental metadata API, with CLI local signatures and native local lists.
  Reject bad owners/indices, stack underflow and loads before stores. ClearBody retains
  locals. Older producer artifacts remain readable; older experimental host readers
  may reject the new local list. Lock backward reading with an explicit pre-locals
  fixture; optional local lists must not become required by exact object-shape checks.
  Raven now emits initialized locals and assignments
  through both bounded backends; the runtime's existing local support is reused.
  Add owned branch labels, signed comparisons and Boolean/branch Emit overloads with
  typed control-flow stack/initialization validation. Compute CLI/native destinations
  after layout; Raven if/else and lowered loops reuse the shared body path. Extend Ceq
  to matching Boolean operands so Raven negation and !=/<=/>= preserve native Boolean
  semantics; validate combined break/continue execution.

- Integrate Raven's compiler-lowered bodies into the bounded native metadata emitter,
  sharing implicit Int32 returns and simple named calls with .NET emission. Keep the
  independent metadata API, binary format and runtime unchanged. Record the author-led
  codegen abstraction plan and defer native symbol loading to its later slice.
  Add shared per-emission callable identity resolution in Raven with backend-owned
  handles and native definition registration before body emission; preserve overloads,
  owners and explicit dependency policies. Shared source callable plans now separate
  native source collection/validation from builder creation, preserving assembly-owned
  functions and empty static types while .NET retains its CLI carriers. Shared static
  source-type plans now drive typed builders in both backends, retaining symbol ownership
  and namespace identity; broader type/field references remain pending.

### 2026-09-30

- Add direct runtime PE/#Neo loading to the extended metadata experiment: required
  execution section 256 carries native format-5 metadata and bodies (schema 1 JSON,
  schema 2 binary). The separate .NET API writes containers and reads owned native bytes/CLI projections;
  Raven can use the same library files as compiler references and runtime modules.
  Reject missing/altered binding, unsupported required schemas and malformed containers;
  preserve runtime dependency and body checks. Validate 27 C# groups, 18 runtime
  process cases and 16 focused Rust/CLI tests. Structural runtime support and
  production target registration remain pending. This is feature-branch support,
  not a published format or a general CIL loader.
  Raven now selects the independent native backend through its shared Compilation.Emit
  pipeline; rvnc and compatibility APIs reuse compiler setup/contract validation.
  Default .NET emission remains unchanged. Native debug output and CLI core rewriting
  reject before output; deeper shared type/method builder abstractions remain pending.
  Extract shared Raven linear-body lowering consumed by .NET and native method-builder
  adapters. Eligible release .NET methods reuse it; general/debug/PDB paths are retained.
  A single compilation prints and returns 42 on both runtimes. Extend shared .NET lowering
  to assembly functions and Unit static methods with void CLI signatures; the same Unit
  Main/helper, explicit-return and empty-entry programs now exercise both backends. Type/signature builder,
  generic and control-flow abstractions remain open; the native source subset is unchanged.
  Share Raven's Int32/Unit callable signature and typed declaration-builder contract across
  Reflection.Emit and native adapters. Preserve .NET visibility, parameter names, generic
  fallback and selected-core identities; preserve native assembly-function ownership.
  Record a translated-System driver type-selection failure (host CoreLib collision);
  direct API execution passes. Metadata loading is an author-deferred follow-up.
  Add MethodBuilder.Emit overloads for supported logical opcodes, Int32 operands and
  builder/imported/native call references. Helpers delegate to the same validated path;
  invalid opcode/operand pairs reject before mutation. Raven's native emitter consumes
  this API. C# helper/Emit equivalence, CLI execution and native integration pass;
  broader opcodes and ILProcessor-style editing remain future work.
  Admit local parameterless no-result entry points in the metadata writer and native
  declaration reader; Raven Unit Main now emits, verifies/runs and exits zero through
  API and rvnc paths. Ordinary CLI output also preserves and executes void entry points.
  Existing no-result encoding is reused; foreign/parameterized entries and invalid stacks
  still fail. This removes the experimental Int32-only entry restriction.
  Add an owned native library/function inventory and explicit partial static Int32
  reference views. Raven reuses its existing semantic importer for selected translated
  System callables, then emits their native identities through a new MethodBuilder.Call
  overload. API/driver cases execute Math.Min against binary System to 42. Visibility,
  generic-arity collisions, invalid selections and unsupported Result signatures are
  checked. Full core loading and general raw instruction editing remain pending.
  Raven now offers an opt-in `rvnc neoclr` command for source files and API-produced
  native PE references, emitting directly through the independent metadata project.
  Process tests verify/run a separately compiled library/application with assembly-owned
  functions. Translated standalone System symbol loading remains the next loader goal;
  the command currently uses the documented host primitive bootstrap.
  Preserve namespaced Raven static library types through emission, reference reimport
  and native calls; same-name types in different namespaces execute correctly in both
  source orders. Namespace-owned functions and nested types remain rejected without
  writing output. This compiler-only extension does not change the native format.
  Extend the Raven feature-branch consumer to Unit-returning helpers and imported
  static library methods. A separately emitted library is reimported into Raven and
  both no-result/Int32 overloads execute from binary PE/#Neo; explicit/implicit returns
  and rejected calls are checked in C#. Entry points remain Int32; runtime encoding
  and ordinary .NET compiler behavior are unchanged.
  Add native-only bounded console literal emission and the author's Hello World
  acceptance cases: direct output and an entry-point function call both print one
  line and exit zero from Raven-produced PE/#Neo files. C# literal/Unicode/bounds
  checks pass; ordinary CLI emission rejects this native-only operation.
  Add required execution schema 2 using bounded CBOR, direct runtime deserialization
  without JSON parsing, and .NET WriteBinary/dual-schema inspection. Keep schema-1
  compatibility; unsupported binary forms, duplicate keys and resource-limit failures
  are rejected. In one local release comparison, the 65-function PE is about 31%
  smaller and binding/decoding about 14% faster; the tiny case is unchanged and
  shared System linking/preparation dominates. This is not a general startup or
  execution-speed claim. Binary Hello World, function-call and dependency cases pass.
  Add a release-mode phase comparison;
  record class-library JSON translation and Raven symbol loading as the next bootstrap
  direction, not completed general translation or native symbol-provider support.
  Add NativeModuleContainer and a no-overwrite .NET translator for existing format-5
  JSON, with standalone NEOX loading for root, dependency and System inputs. The full
  assembled System library preserves 117 types/743 functions; 641 IL functions verify.
  Both Hello examples and a translated generic module chain run using binary System.
  Validate 28 C# groups and 15 focused Rust/CLI tests. This transport has no CLI
  projection; direct Raven class-library compilation and compiler symbol import remain
  next steps, using translated artifacts as regression baselines.
  Add a reproducible translation experiment for existing Raven match/library samples,
  comparing neoil, JSON and binary application behavior with established expected output.
  Record the larger Raven collection-profile System's binary input/item-limit failures
  separately; application tests use the same matching JSON System. Correct obsolete
  Option carrier construction and the wrong-arity diagnostic expectation in existing
  match fixtures, without changing compiler semantics. Floating-math source/JSON runs
  expose an additional schema-2 gap: negative Double operands carry UInt64 bit patterns
  above Int64.MaxValue and translation rejects them. Fourteen of fifteen application
  cases pass binary execution with JSON System; six expected compiler rejections hold.
  Record the author's future neoil assembly-producer direction; direct binary assembly output is not implemented.
  Add standalone execution schema 3 for Int64/UInt64 operands and larger libraries:
  32 MiB JSON, 8 MiB envelopes, 2,097,152 items and depth 64. Preserve schema-2/PE limits.
  NativeModuleContainer.WriteLibraryBinary and the translator emit the new profile;
  old runtimes reject it explicitly. Full Raven collection System values roundtrip,
  and FloatingMath, OptionPositional and ValueCopy verify/run with binary System.
  Validate 29 C# groups and 18 focused Rust boundary/runtime tests, including exact
  unsigned floating operand bits. Refresh the API source fingerprint after regenerating
  the unchanged reference assembly with the corrected match fixture. Native compiler
  symbol import remains pending.
  Add direct neoil assembly with `assemble --format neox`, preserving default/explicit
  JSON output. The bounded Rust writer emits schema 3 without JSON and the CLI verifies
  the load set before creating output; dependencies remain separate and overwrites fail.
  Validate 21 focused Rust tests plus four complete-value C# reader comparisons; full
  Raven collection System and three applications assemble directly and run successfully.
  This is the existing neoil subset, not complete ILAsm parity or native Raven source emission.
  Add the requested JSON-versus-native-assembly release benchmark with nine raw samples,
  interleaved decoder/process paths, matching metadata/output and artifact hashes.
  On one Apple M1 run, System is 63% smaller and takes about 20% less time to decode than current
  pretty JSON, but full start/run time remains about 3.45 seconds with no meaningful
  improvement. Linking/admission dominates; direct-typed JSON is a faster diagnostic
  candidate. The author accepts the size improvement and defers runtime optimization.
  No production optimization, cold-disk or memory-performance claim is made.
  Earlier slices develop extended CLI metadata on a main-based feature branch, with an isolated
  NEOX 0.1 codec/inspector, versioned framing and structural signatures. Preserve
  structural-branch Function no-result/output contracts and owned/reference arrays.
  Add catalog-scoped nominal references and declaring-owner generic/Self contexts;
  differently numbered fixture references resolve to equal structural keys.
  Add synthesized array-length, tuple-element/deconstruction and Function-invocation
  references with derived contracts, checked owner shapes and stable resolved member
  keys within a catalog. All 30 focused tests pass, including golden byte vectors,
  malformed input, resource limits, identity and inspector process checks.
  Update the design, compatibility limits and staged Raven integration plan.
  Add a bounded unsigned PE32 #Neo embedding/extraction probe: .NET 10 metadata and
  Mono.Cecil 0.11.6 read unchanged conventional streams/signatures/bodies, while
  Cecil rewriting strips #Neo. The aware inspector rejects unknown required data
  that ordinary readers ignore; 15 malformed/unsupported container cases pass.
  This is inspection evidence, not an execution or semantics-preserving rewrite
  guarantee. Add provisional marked-artifact recognition: a metadata-root marker,
  stream digest and explicit expected-extended input reject stripped/changed metadata
  in 11 process-level cases, including Cecil's lossy rewrite. This is consistency
  checking, not authentication or an unaware-runtime execution guard. The 30 codec
  tests still pass. Production recognition/loading, real CLI dependency resolution,
  compiled cross-module evidence and Raven compiler changes remain pending.
  Record the author-directed plan for reader/writer libraries on both .NET and
  neoCLR, including native versus guest-accessible support and cross-platform
  conformance. Clarify their consumers: Raven symbol loading/code generation and
  neoCLR assembly loading into Introspection/assembly emission. Package/API names
  and implementation sharing remain undecided. Begin an experimental net10.0
  envelope reader/writer library with immutable owned sections and a separate
  conformance consumer. Four fixtures round-trip identically, independent .NET/Python
  emission agrees, and both readers reject 49 malformed vectors. Document all host
  types/members manually on the API site and in generated XML; guest snapshots stay
  unchanged. Add the .NET structural signature reader/writer with immutable syntax
  trees and local generic/Self contexts: 14 cross-reader vectors and 103 rejection
  cases pass, and independent .NET nested emission matches Python. Preserve Function
  modes/no-result and array distinctions with bounded decoding/encoding. Add .NET
  reference-table read/write and explicit-catalog structural identity: 95 shared
  vectors pass, including 22 equality/distinction cases and 69 rejections; independent
  UUID byte-order emission, writer validation and ownership checks pass. Document
  all six new host API types. Physical CLI binding and Raven/neoCLR consumers remain
  pending; host scope UUIDs are not production assembly identities. Add .NET
  synthesized-member table read/write and derived array, tuple and Function contracts:
  49 shared vectors pass (14 contracts, five identity comparisons, 30 rejections),
  plus independent golden emission, writer validation and reader ownership checks.
  Document four new host types; descriptors preserve native unsigned array length
  and Function modes/no-result without granting runtime invocation. Add a typed .NET
  reference-profile Read/Create/Write API with owned documents, local validation,
  opaque optional preservation and explicit catalog resolution. All 36 shared profile
  cases pass (29 rejections), plus independent construction/emission and ownership
  checks. Document both new host types and the author’s direction toward a Raven-ready
  .NET API and potential Raven port beneath Metadata Introspection; complete assembly
  IO, compiler integration and the port remain pending. Add bounded .NET unsigned
  IL-only PE32 recognition/extraction with expected-extended input by default, stream
  binding checks and typed profile validation: 50 shared cases pass (44 rejections),
  plus size/ownership checks. This is consistency checking, not authentication or
  conventional CLI verification. Document both host API types and the author’s Cecil
  suggestion, with a provisional object-model design above the existing codecs. Add
  the first read-only Cecil-inspired assembly/module/TypeDef model using real CLI
  declarations, with snapshot-scoped definition-backed references. Generated consumer
  checks cover Unicode, generics, nesting, ownership, local lookup and corrupt CLI
  tables under a valid digest, including decoded-name amplification rejection.
  Document four model types, their 4096-TypeDef and cumulative decoded-name limits;
  physical TypeRef resolution and assembly writing remain pending. Record the author’s
  selection of the Cecil-like model as the primary compiler metadata/PE abstraction.
  Add exact assembly identities, physical AssemblyRef rows and explicit host resolution
  with mismatch rejection and bounded key/reference decoding. Eleven standalone C#
  contract tests pass using generated PE fixtures; the prior model consumer also passes.
  Document three new model types and new identity/reference properties. Full Raven
  integration remains pending. Add physical nominal TypeRef resolution
  with bounded nesting and explicit dependencies, plus controlled assembly/type/method
  builders for static Int32 CLI PE production, body editing and imported calls.
  Fourteen C# contract groups pass. API-produced application/library PEs pass the
  existing CLI bridge and execute in neoCLR with result/exit code 42 using a matching
  System library. The C# runtime gate also serializes and reloads the native artifact.
  This is the existing Raven bridge route, not direct PE/#Neo loading; arbitrary
  rewriting and wider signature/IL coverage remain unsupported. Add first-class
  assembly-owned functions (nullable declaring type), CLI global-method emission and
  direct native format-5 emission from the same graph. API-produced native application
  and dependency load/verify/run without the CLI bridge and return 42; missing and
  wrong-revision dependencies are rejected. Preserve descriptive origins and no-result
  contracts. Fifteen C# contract groups pass. Cross-assembly globals currently require
  native emission; direct PE/#Neo runtime loading remains pending. Record the author's
  sequence: working metadata/APIs, refactored compiler integration, then structural
  improvements; the structural codec experiment is not a prerequisite for integration.
  Add owned callable declarations to the PE reader: top-level functions, type methods,
  entry-point/token lookup, CLI flags and copied signature blobs. Recognize the writer's
  static Int32/no-result signature subset without simplifying unsupported encodings.
  Bound method rows and repeated-signature decoding. Add owned physical MemberRef
  lookup and explicit nominal method resolution for the emitted static Int32/no-result
  subset, with overload/return matching and ambiguity rejection. Preserve unsupported
  reference signatures as opaque data; validate parent rows and share the decoded
  signature budget with MethodDefs. All 21 C# contract groups pass. Record the selected
  Raven refactoring activity and its inspected loader/emitter boundaries; the next
  planned consumer is a bounded compiler-to-runtime case that drives remaining format
  support, rather than waiting for exhaustive metadata coverage. Clarify that the
  Cecil-style API remains an independent library project consumed by Raven's neoCLR
  target, with compiler-owned adapters and no reverse compiler dependency. Add a
  first Raven source-to-native runtime consumer on Raven's codex/metadata-consumer:
  public operations feed the independent metadata API, and neoCLR verifies/runs the
  resulting application and dependency to 42. The input provider remains .NET-based;
  native target composition is still pending. The case drove general Raven fixes for
  binary-operation facts, invocation receivers and signature-only required parameters;
  87 focused compiler tests pass and those fixes are on local Raven main.
  Import bounded static Int32 callable references from read-only definitions into
  the independent metadata writer. Explicit core-contract assertions, consuming
  ownership and conflicting module/contract checks guard PE/native emission.
  The Raven probe no longer requires the dependency builder graph. C# contracts
  and native top-level/Raven consumers pass, returning 42; native symbol loading
  and production target installation remain pending. The Raven consumer now uses an
  optional compiler-owned native emitter API with explicit dependency bindings,
  source-located diagnostics and validation before output writes. C# contract checks
  and neoCLR execution to 42 cover the extracted adapter; the metadata project stays
  independent and ordinary .NET compilation remains unchanged. The adapter now emits
  multiple source files with cross-file calls; both input orders verify/run to 42,
  and rejected later-file operations retain their source location without output.
  Read bounded API-produced native declarations into an owned snapshot and project
  reference-only PE metadata for Raven's current symbol loader. Validate declaration
  consistency/bounds and reject unsupported shapes; bodies remain opaque to the
  metadata reader. Mark projected PEs as reference assemblies with throwing bodies.
  All 23 C# contract groups pass; Raven binds the native dependency through this
  projection and all three applications run to 42 using the original native artifact.
  This temporary input bridge does not implement a native semantic-data provider.
  Extend the case to a Raven-compiled library and separate Raven application,
  including overload selection and a library-local helper call. All three application
  variants return 42; unsupported library visibility and missing/wrong-version native
  dependencies are rejected. The native runtime format is unchanged. Extend runtime
  acceptance to a transitive chain of three Raven-compiled native assemblies, with
  both module orders, missing transitive modules and wrong revisions checked. Expose
  owned exact direct dependency identities through NativeAssemblyDefinition.References;
  implementation-only dependencies remain outside primitive PE projections. All 24
  C# metadata contract groups pass; the existing native loader/verifier/VM runs the
  chain to 42 without runtime implementation changes.


- Continue the structural Function experiment as `codex/structural-types` on top
  of shared main. Retain structural signatures/objects and Function introspection
  alongside main's integrated Self; the experiment remains outside main. Resolve
  nominal-versus-structural merge conflicts in favor of this branch's Function ABI
  and regenerate matching library and API artifacts. Validate 80 focused native
  tests, identical regenerated native bodies, metadata/audit checks and a structural
  unit callback executing through function.bind. Self cloning and all six negative
  consumer cases also pass on the structural bundle.
- Preserve the deferred Function/structural experiment as `codex/structural-types`
  on top of the Self-integrated main. Retire the previous branch name after
  synchronization; this organizational change does not enable structural types on main.

- Integrate native Self independently of structural Function types: nongeneric
  Number and Clonable contracts, conformance-owned inheritance, checked generic
  Self dispatch and borrowed receiver support. Preserve main's nominal delegates.
  Raven integration now selects TargetPlatform.NeoCLR explicitly and transports
  Self through a fieldless marker until a native metadata loader is available.
  Rebuild references and libraries together; the former generic contracts are
  incompatible. Structural types remain on feature/function-types. Validate 98
  native tests, rebuilt library/API snapshots, and class/struct cloning with six
  rejected programs; nominal unit callbacks still execute through delegate.bind.
  Numeric Self/parsing verification and both negative cases pass; update the stale
  importer diagnostic expectation and record matching artifact hashes.

- Backport Raven function type syntax for library callback annotations and lazy
  `Iterable<T>.OfType<U>()` filtering from the Function feature branch. OfType skips
  null/incompatible elements, preserves order, narrows matches without numeric
  coercion and disposes its source on completion or explicit disposal. Keep main's
  existing nominal Func/delegate runtime, artifact formats, comparer names and
  TypeInfo/MemberInfo contracts. Regenerate the matching library and public API
  reference; document and test the independent query contract. Also backport the
  source-coverage audit fix that includes generated method-body service callers
  and records generated slice provenance without treating it as execution evidence.
  Refresh stale query reflection expectations and union/fault fixtures, including
  the callback sample's Option pattern. Carry the Delegates evolved and Callable
  interface documents as open proposals, explicitly unimplemented on main; link
  them from the earlier exploration and website proposal overview. Record the
  author's subsequent direction to hold the Function experiment and focus next on
  Raven's neoCLR target support; compiler tasks remain to be selected.

### 2026-09-28

- Add development System.Tuple value types with one through seven components,
  mutable Item fields and positional constructors. Integrate Raven tuple syntax,
  names, deconstruction and nested values through the matching target compiler,
  checked importer and generated Raven library. Add focused metadata, malformed
  layout/signature and native consumer coverage, with matching API reference docs.
  This is neoCLR's name for the ValueTuple representation, not .NET's reference
  Tuple class. Unit remains System.Void; wider flat tuples/Rest and the full .NET
  equality/comparison/formatting API remain outside this initial slice. Matching
  development compiler/reference/importer/library artifacts are required.
  Independently validate Raven's general tuple metadata fix on .NET and integrate
  it into Raven main and the isolated target worktree.

### 2026-09-27

- Correct the website's shared banner, footer and release link to Preview 11;
  its site configuration still displayed Preview 10 after the release content update.
  Align API-guide availability labels with the published Preview 11 surface and
  link its release notes from setup, retaining prior-release compatibility notes.

- Explain HttpServer through an application-owned accept loop, separating request
  handling models from the listener. Show the existing report-server loop directly
  from its source, including rejection policy, completion and cleanup.

## 0.1.0-preview.11 — 2026-09-27

### 2026-09-27

- Qualify Preview 11 for macOS arm64 and Windows x64 native execution, with matching
  macOS Raven SDK/VSIX, source, notices and checksummed evidence. Record 1,659 passing
  Linux tests separately from the corrected archive validator's 25 passing samples;
  refresh feature/setup pages and document the Windows Visual C++ runtime prerequisite.

- Update RavenDoc extension list labels to omit type parameters supplied by the
  receiver: String shows Any() rather than Any<T>(), while caller-selected generic
  parameters remain and nested parameter types use the receiver substitution.
  Declaration pages retain their full signatures.

- Complete the locked dependency notice inventory for chrono-tz, sys-locale,
  phf/phf_shared and siphasher, including timezone-data licensing. Copy notices
  only after verifying upstream crate archives against Cargo.lock checksums.

- Prepare Preview 11 from the bounded current surface, with Windows x64 native
  runtime packaging and extracted-sample smoke checks alongside focused host CI.
  Keep CI reports/packages under target and reject dirty-source native packaging.
  Windows Raven SDK/bridge distribution remains unqualified. Include generic async,
  unit-delegate and interpolation consumers in the extracted toolchain bundle.
  Include routing/mapper client-server sources and separate-SDK verification, and
  refresh the packaged MSBuild reflection expectation for Number members.
- Repair numeric/object interpolation with String.Concat(Object?, Object?), using
  virtual ToString and empty text for null; integrate Raven's general missing-member
  diagnostic fix. Preserve generic unit delegate results until the caller discards
  them, regenerate all library slices, and restore current-union signature/sample
  checks. Update matching API documentation and keep old artifacts unchanged.
  Keep object concatenation out of the historical Neo projection, whose type
  system has no System.Object; verify its CLI and target-layout consumers.
  Align the non-Unicode environment test with the sample's explicit error exit
  code while retaining its typed EnvironmentUnavailable output check. Refresh
  interface-default, service-declaration, closed-dispatch graph and Raven calendar
  union fixtures to reflect implemented contracts and documented analysis limits.
  Check source/archive sample exit codes against their declared computed results,
  instead of treating intentional Int32 entry values as runtime failures.

- Restore canonical formatting and strict Clippy gates for release preparation.
  Rename the internal slot factory, name the completion observer type and simplify
  equivalent expressions; retain inline completion values to avoid new allocation.
  All-target lint and ten focused slot/alias/thread checks pass.

- Install local SDK/VSIX `0.1.12-neoclr.20260927.cpu1` and the indexed-runtime
  HTTP bundle; verify editor completion, independent HTTP cases and the Raven pair.
  Preserve prior installations and record per-component source revisions/hashes.
  This is a local development installation, not a published release.

- Index immutable execution metadata candidates while preserving type, signature,
  access and duplicate-resolution checks. Focused native and Task.Run consumers
  pass; local HTTP median CPU falls about 24%. Record remaining quota aggregation
  cost and general Raven workspace-discovery repair integrated into both branches.
  Quotas, locking, deadlines and public APIs are unchanged.

- Add a maintained neoCLR/.NET comparison covering contracts, practical tradeoffs,
  implementation and compatibility limits, with primary .NET references. Link it
  from About, Guides and section navigation; correct stale Preview 10 status and
  document when to review the comparison.

- Keep the DOM and typed JSON sample servers alive after protocol, limit and
  unsupported-request rejection. Count failed attempts toward the bounded sample
  lifetime and retain other error propagation. Verify an oversized connection is
  closed and a subsequent valid GET succeeds, with zero live objects at exit.

- Investigate installed HTTP server CPU with repeatable idle/request profiles and
  header-size measurements. Record low steady idle cost, substantial metadata/quota
  execution overhead, and fatal sample handling of over-limit requests. Separate
  old high-CPU Raven language servers from runtime findings; propose focused fixes
  without changing runtime behavior, limits or scheduling.

- Build and locally install matching SDK/VSIX `0.1.12-neoclr.20260927.async2`
  with the async compiler corrections. Verify installed generic async consumers,
  editor completion and HTTP client/server behavior; open the editable workspace
  and launch the live pair. Record revisions, hashes and remaining release gates.
  This is a local development build, not a published release.

- Align the editable local HTTP client with the current typed JSON sample. Its
  shared application contract uses HttpJsonError; prepare the matching generic
  GET/POST client and serve both requests in the local managed-pair verifier.

- Support bounded generic async methods on nongeneric application owners by
  importing their constructed state-machine and shared closure types. Preserve
  substituted members, callback receivers, type identity and access checks.
  Add positive two-suspension, capture, identity and cancellation consumers;
  retain malformed generic/ref protocol checks. Integrate Raven's independently
  tested general fix for constructed source types in target generic signatures.
  Support ordinary instance async methods inside bounded generic classes;
  integrate Raven owner-arity and implicit-field-receiver fixes with 35 focused
  CLR checks and a pending-await int/string receiver consumer. The reported
  interpolation defect remains a release follow-up; no public signatures or Runtime
  Contract options change. Integrate the SDK-bootstrap follow-up preserving
  nongeneric CLI ownership for generic extension closures, with 29 focused CLR checks.

- Record the author-selected pre-release assessment on main: feature finish lines,
  confirmed format/Clippy and stale union-probe gate failures, current async
  reproduction results and broader native/integration evidence. Separate supported
  behavior defects from explicit generic import limits and deferred capabilities;
  retain one canonical full candidate suite plus focused host/package checks.

- Include System and System.Tasks extension namespaces in the API reference so
  Result/Option and Task composition methods appear on their receiver type pages,
  alongside the existing LINQ and reflection extensions. Update RavenDoc to
  resolve open generic receivers, preserve extension-container declarations under
  the reader filter, and omit inherited members from static classes. Correct API
  navigation highlighting so namespace functions are selected individually.

- Update RavenDoc with linked derived types, derived interfaces and implementing
  types, marking indirect relationships within the documented API surface. Add an
  independent, persisted Show extension members toggle with accurate group counts.
  Place the toggles alongside one another when space permits and wrap on narrow
  screens. Share grouping and filter choices through URL query parameters, with
  valid URL settings taking precedence over saved preferences.
  Preserve the project dark palette when Auto follows a dark system theme.
  Include the publisher fix for literal generic names in XML documentation prose.

- Adopt RavenDoc’s overridable compact typography defaults (15px equivalent prose,
  13px highlighted code/signatures), remove oversized article-title overrides and
  balance the landing hero. Long inline code wraps within narrow-screen prose.
  Correct Task.Run guide links to the generated heading anchor.

- Update the pinned shared RavenDoc publisher with semantic member origins,
  inherited-member visibility and kind/declaring-type grouping. Keep static members
  on their declaring type; show selected LINQ/reflection extensions inline with E
  badges and linked origins. Include union case pages, companion merging, nominal
  generic/delegate names, closed hierarchy links, authored API content support,
  direct union/enum navigation and independent sidebar scrolling below the release
  notice. Wrap long group headings and page-outline entries to prevent horizontal
  overflow on narrow and desktop layouts. Hide empty
  namespaces and the compiler-owned companion marker by default. Document the general compiler symbol corrections; no
  Runtime Contract or runtime behavior changes are part of this update.

- Extend development JSON serialization/deserialization to nested nongeneric
  reference properties, with full input-tree validation before model construction
  or setters. Preserve scalar/name/null/error policies; expand the development JSON
  document and number-token cap from 128 to 1,024 UTF-8 bytes across string, stream
  and HTTP-content paths, matching buffered HTTP bodies. Count escaped output bytes,
  validate before stream writes, and add boundary checks plus a longer station report;
  support typed scalar/model arrays (including jagged arrays) and scalar roots,
  validate every array element before constructing models, and use checked runtime
  adapters for array access/construction. Integrate the independently tested Raven
  target-metadata array fix. Bound container depth to four including the root,
  reject polymorphic property values,
  and serialize shared children as independent subtrees. Add focused mapping checks
  and a nested station-report client/server case with on-site walkthrough and
  downloadable tested sources. Introduce a tested general HttpClient example and
  capability overview before “Case: Building a Http server app”, with the server
  followed by its connecting client. Give the case its own page and navigation,
  retain essential code on the feature page, and record contextual cases as a
  general website convention. Record the author-selected minimal Web API priority
  and roadmap acceptance scope. The latest author direction selects a route parser
  inside HttpServer with typed parameter extraction, deferring the earlier separate
  WebApplication plan; enum/Uuid/Option mapping and optional SQLite remain planned.
  Preview 10 remains flat-model-only.

- Add development RoutePattern.Parse/Match and RouteMatch.Get/GetInt32 for reusable
  literal and named-segment parsing inside HttpServer handlers. Separate no-match
  from malformed input and typed conversion failure; define bounded, case-sensitive
  paths with query separation and strict once-only UTF-8 segment decoding. Add a
  direct-use consumer and station client/server case with optional application union
  dispatch, API reference and tested website excerpts/download. Keep dispatch and
  lifecycle application-owned. Add an experimental emitted-metadata generator for
  attributed application union cases, with String/Int32 payload binding, build-time
  schema checks, startup-compiled reusable patterns and Result-based no-match/error
  handling. Include an item server/client case. Integrate Raven's independently
  tested case-attribute emission fix and admit bounded Int32-only standard union
  carriers through logical fields, without general CLR explicit-layout aliasing.
  Add a reusable runtime mapper sample that validates attributes once at startup,
  caches patterns/conversions/constructors and returns typed union results to the
  catalog HTTP handler. Reject repeated attributes, unsupported payloads and
  structurally overlapping routes before listening. Add RoutePattern.GetParameterNames
  snapshots and conservative Overlaps checks, TypeInfo.IsVisible (including source
  enclosing types and generic arguments), and ConstructorInfo.Invoke with dynamic
  Sequence<Object?> arguments. Update the server/client case, API reference and
  downloadable projects. SDK mapper packaging and source generation remain future work.

- Add development System.Runtime.GC with execution-local collection, allocation,
  retained/peak/reclaimed object counters and the host heap-object limit. Add
  synchronous full Collect and nullable KeepAlive; share automatic/explicit root
  tracing and expose ExplicitRequest in host collection history (exhaustive host
  CollectionReason matches must handle the new case). Preserve task,
  construction and interior-reference roots and existing limits. Counters describe
  objects rather than bytes; no generations, finalizers or tuning controls are
  implied. Add exact bridge contracts, a compiled consumer, API documentation and a dedicated
  garbage collection feature page. Supply manual reference entries for existing
  LocalTimeMapping deconstruction members omitted by RavenDoc and repair numeric
  guide links found by the combined website build. Integrate the concurrent Result/Task
  entry-point dispatcher while retaining GC roots and Reflection contracts.

- Add development ConstructorInfo/GetConstructors, Result-based method invocation,
  instance field access and argument-taking typed/untyped CreateInstance extensions.
  Validate typed result compatibility before construction; keep exact scalar binding
  and report ambiguous constructor matches. Preserve source field access/read-only
  restrictions and static method ownership; imported field execution requires new
  admission metadata. ConstructorInfo extends the closed MemberInfo family, requiring
  exhaustive-match updates; ReflectionError also gains five cases. Refresh library/API
  contracts and add a dedicated Reflection
  feature page. Generic classes, static fields, coercion and byref/out remain unsupported.
  Add metadata-only GetCustomAttributesData to MemberInfo and ParameterInfo, with
  CustomAttributeData/CustomAttributeTypedArgument snapshots for retained application
  attributes and String/Int32/Boolean constructor constants. Preserve scoped member
  and parameter targets; reject malformed targets, unsupported constants and named
  arguments. Inspection never executes constructors. Compiler-only/external framework
  annotations remain outside this bounded surface; nullable-string attribute emission
  has a recorded Raven limitation. Add ConstructorInfo.Invoke through reflection
  extensions for exact retained public constructors, including nongeneric value
  records and union case/carrier construction. Retain imported value-constructor
  metadata through checked wrappers; preserve initialized no-result value newobj
  results. Existing TypeInfo activation remains class-only. Add a startup union
  construction case and match updated runtime/library/reference artifacts.

- Record Task.Run as the author-selected canonical submission API with shared
  captures and runtime-selected execution. Document heap/scheduling prerequisites,
  backend alternatives and focused acceptance; implementation and Thread's future
  public role remain open. Add the first runtime prerequisite: identity-preserving
  synchronized managed slots, weak heap handles and debugger snapshots that release
  storage locks before following references. Cover native-thread alias mutation,
  coherent slot reads and payload-cache invalidation. Record green threads as a
  possible later backend. Add invocation-owned heap coordination with registered
  participant roots, atomic admission/publication under exclusive graph access,
  collection across parked participants and export after participant release.
  Route existing VM collection through one participant; concurrent guest execution
  and public Task.Run remain pending. Add focused coordinator/GC/startup checks.
  Implement the internal bounded native work owner with rooted capture/result
  handoff, invocation provenance, cancellable heap-gate waits and cancellation/join
  cleanup that preserves the host token. Validate blocking runtime-side callbacks;
  guest execution, shared services, Promise publication and Run overloads remain
  unconnected. Add retained VM instruction state and bounded execution intervals
  that publish roots before releasing heap access without resetting instruction
  fuel. Validate native guest capture/result identity and queue/entry draining with
  collection after every instruction; record a focused ordinary-execution cost
  comparison. Release graph access during scheduler completion waits, retaining the
  dispatch boundary and all roots without consuming guest instruction fuel; validate
  collection by another participant, exactly-once notification posting and cancellation.
  Add prepared blocking-call boundaries for file/console I/O, isolated-worker joins
  and native imports. Preserve rooted callers/destinations, snapshot byte writes and
  commit only transferred read bytes under graph access. Validate guest capture
  mutation during a host call and buffer alias/tail preservation. Shared invocation
  services and public Task.Run remain pending. Share instruction fuel and live-frame
  permits across VM contexts; paused frames retain capacity and returns/unwinding
  release it. Validate aggregate exhaustion/admission and record a focused cost probe.
  Share the invocation file table and intern pool across VM contexts, preserving
  handle position/close state, string identity and quotas. File-table locking stays
  outside graph access; file operations are initially serialized. Share scheduler and
  default-queue ownership with invocation service roots independent of submitting
  contexts. Serialize completion draining, retain cancellation while waiting, and
  detach worker joins before blocking so scheduler access remains available. Preserve
  final-GC diagnostics after service teardown. Share native-buffer identities, quotas
  and loaded library ownership across contexts. Retain direct tracked foreign-call
  arguments with exclusive leases; reject conflicting buffer access and omit busy
  debugger bytes while unrelated buffers remain usable. Validate shared guest
  buffers and native imports. Aggregate array budgets across active and parked guest
  contexts, retained results and prepared I/O buffers; count shared heap payload once
  and release private charges on participant exit. Keep selected System.Tasks
  mutations and state snapshots atomic across instruction quanta, including Promise
  completion/registration, queue bookkeeping and lazy Default creation. Yield outside
  those regions for queue callbacks; preserve cancellation and instruction limits.
  Test the generated library with forced interleaving. Submit actual captured values
  with provenance, roots and payload admission before native execution. Retain completed
  results and transfer their private charge without double-counting; release rejected
  work capacity. Include inline arrays in owned delegate receivers in array budgets,
  closing previously omitted payload accounting. Own native work at invocation scope,
  wake the scheduler on completion, retain worker faults/panics and cancel siblings.
  Close admission and join outside graph/service locks on root exit or unwinding,
  before releasing shared resources. Add internal native guest-delegate execution
  with shared captures/services/budgets, linked host/service cancellation and bounded
  captured-output forwarding. Keep callback return independent of the root pump;
  park entry draining for native work and redrain on completion without spending
  guest fuel while waiting. Add private ScheduleTask submission with bounded heap
  participants and validate generated Promise completion/continuation dispatch plus
  terminal guest faults. Add development Task.Run overloads for completion-only,
  typed and task-producing callbacks, with shared capture/result identity and async
  outcome transfer on the default dispatcher. Refresh matching library/reference
  artifacts and API documentation. Correct block-lambda return inference through
  Raven main 6cc4fed66 (neoclr a01fb6245), retaining parameter hints while inferring
  value returns independently of a completion-only overload. Remove the typed-local
  workaround from the compiled consumer. Correct short-name
  Task.Run lookup through Raven main f1a3792b8 (neoclr 09f584523); metadata order no
  longer selects Task<T> in place of its nongeneric owner. Promote the reduced
  lookup failure to a passing consumer and remove the example’s explicit alias.
  Correct direct completion-only await in the target
  compiler by recognizing the configured inhabited unit result; remove the integer-map
  workaround and promote the failing fixture to a positive consumer. Correct ordinary
  async mutable-local
  sharing through the independently validated Raven closure fix (main dc7b87eff,
  experimental integration 08815ceaf), retaining one closure per invocation across
  suspension. Promote the original failing consumer to a required 42-result regression
  and repair generic-method capture metadata through Raven main 586cc8d89 (neoclr
  b32459beb), with 11 focused CLR checks. Record the newly reached numeric-only
  generic application importer restriction as missing neoCLR support for Raven's
  normal contract; retain an explicit rejection fixture and a bounded import follow-up.
  Extend checked import to closed unconstrained static generic helpers, preserving
  nested substitutions, object/vector identity, generic library member signatures,
  exact Number constraints, visibility and source debug origins. Separate cache
  entries by assembly-qualified type arguments. Add executable and metadata checks.
  The async fixture now reaches unsupported constructed application state-machine
  import; retain that next compatibility gap explicitly.
  Keep the separately reproduced Raven generic-containing-type async arity failure
  distinct. Record the author's compiler/runtime compatibility ownership direction.
  Integrate the
  Task.Run work with main’s route, JSON-array and reflection-constructor additions,
  retaining native submission and refreshing the combined API reference. Include the
  completed RavenDoc publisher/navigation checkpoint in the integration. This is not
  a published release.

- Support Raven Main returning integer, Result and target Task combinations through
  target-owned startup adaptation, including optional string-array arguments and
  pending default-queue/host completion. Result errors print to stderr and return
  one; unit success returns zero. `neoclr run` now uses integer entry values as
  process status (including Neo/neoIL), so scripts must account for nonzero results.
  Preserve ordinary nonblocking Task.GetResult and .NET Raven entry behavior.
  Update the worker/async samples and HTTP client experiment to await directly in
  Main, document supported signatures on the website, and record independently
  reproduced async compiler observations for separate fixes.

- Import explicit interface methods on non-generic Raven application classes and
  application interfaces. Preserve private MethodImpl mappings, nominal object identity,
  void bodies and separate public methods; document qualified private reflection and
  the existing IsVirtual difference from CLI. Add focused source/native checks and
  narrow the recorded limitations; accessors, value/generic types and external core
  contracts remain outside this import slice.

- Admit Raven application interface defaults and public/private static helpers in
  development. Preserve nominal receiver identity, nested dispatch, class precedence
  and private access; exclude helpers from conformance obligations and virtual
  reflection flags. Add focused consumer/native evidence. Private instance helpers,
  static virtual defaults and broader accessibility remain deferred. Record explicit
  implementation and other Raven/native gaps in the authoritative interface tracker.

- Add development Number<T> for eight fixed-width integers, Single and Double:
  inherited ordering, static Zero/One and binary arithmetic. Add strict concrete
  Parse methods for the remaining numeric types and Boolean, using standard Raven
  NumberParseError/BooleanParseError unions. All numeric parsers, including Int32/Int64,
  share NumberParseError; migrate old numeric error patterns to the shared union.
  Parsing interfaces remain on hold; document the proposed ParsableNumber<T>
  constraint for generic T.Parse calls with the shared numeric error contract.
  Validate nominal static contracts and admit
  bounded closed static application numeric specialization, with explicit limits
  on custom types/additional constraints. Integrate independently tested general
  Raven static-interface/constraint fixes; broader static/default/interface
  accessibility is recorded as platform direction. Refresh matching API references,
  generated primitive slices and focused native/consumer/metadata evidence.

- Add provisional TimeOffset, named IANA TimeZone rules, immutable ZonedDateTime and
  explicit Unique/Ambiguous/Skipped local mapping. Introduce the author-directed
  nominal parenthesized DateTime(LocalDateTime | ZonedDateTime), with typed matching
  and inactive defaults; no .NET Kind flag or OffsetDateTime type. Extend the bridge's
  validated parenthesized union and conditional extraction contracts. Add Time
  wrapping/display, civil date carry and checked Instant arithmetic. Bundle IANA
  2025b via chrono-tz 0.10.4, bounded to 1900–2099; expose the version and system IANA
  discovery without silent UTC fallback. Fixed offsets retain seconds and ±18h bounds.
  Refresh matching artifacts/API docs and the separate DateTime feature page.
  Windows/Linux discovery, broader ranges, rule updates and scheduling remain open.
  Integrate concurrent encoding/encoder and casing/Int64 work from main; regenerate
  combined library/API artifacts and verify calendar, zone, encoder and casing/Int64
  consumers together.

- Add provisional Gregorian/Hebrew calendar policies, checked Date arithmetic,
  Date display and LocalDateTime construction. Add invariant, Swedish and Israeli
  cultures with Language, immutable calendar-selecting date/time formatters, Hebrew
  alphabet or Latin rendering, and fixed/system culture providers. Host preferred
  locale discovery uses sys-locale 0.3.2 through ProcessEnvironment; unsupported
  preferences fall back to invariant. Hebrew supports complete years 5344–5999.
  Refresh matching bridge/library/API artifacts; rebuild consumers together.
  Validate against .NET calendar fixtures and executable Raven consumers on macOS;
  Windows/Linux discovery remains unexecuted. Record the independent unified
  localization direction, with interchangeable JSON/resource sources still planned.
  Feature globalization separately from DateTime on the website; record Time and
  time zones as the author-selected next slice.

- Add development String.ToUpperInvariant/ToLowerInvariant with pinned Unicode 17
  full default casing, expansions and final-sigma context, without normalization or
  locale tailoring. Add strict ASCII Int64.Parse with standard Raven Int64ParseError,
  decimal ToString and static read-only MinValue/MaxValue bounds (not const fields).
  Refresh runtime/library/reference artifacts and feature/API docs; record .NET
  differences and focused report, native and metadata validation. New casing/parse
  native services require a matching development runtime; comparison/hash policy
  and released Preview 10 behavior remain unchanged.

- Record Preview 10 publication, verified remote asset digests and POC completion
  in the roadmap, HTTP and release trackers.
- Select useful library API coverage as the next author-directed work. Record a
  proposed comparer-first sequence, earlier text/casing/StringBuilder/time requests,
  bounded companion tasks and focused validation in the existing theme trackers.
  Later selected text and time slices are recorded above.
- Implement Raven EqualityComparer<T> and Comparer<T> interfaces, callback adapters
  and StringComparer.Ordinal. Add HashMap policy construction while preserving the
  callback constructor; both comparer methods retain the map's reentry protection.
  Ordinal uses exact content equality/hash and native UTF-8 ordering. No universal
  default or culture policy is added. Refresh library/reference
  artifacts, API docs, feature pages and focused source/metadata/GC/editor evidence.
  Correct the signature probe's TaskOutcome interface assertion; its obsolete
  Result payload-setter assertion remains tracked separately.
- Add explicit StringComparison.Ordinal/OrdinalIgnoreCase, String.Compare with a
  required mode, CompareOrdinalIgnoreCase and a matching StringComparer policy.
  Unicode 17 default simple folding drives equality, ordering and hashing; it does
  not normalize, expand mappings or promise .NET OrdinalIgnoreCase equivalence.
  Existing ordinal behavior is unchanged. Refresh matching native/library/reference
  artifacts and focused contract evidence; these APIs are not in Preview 10.
- Record .NET as the ergonomic target rather than an exact API template, learning
  from other frameworks and UTF-8 constraints. Finish the comparison slice, then
  stop further string expansion for an author-directed whole-surface design review.
  Complete that review with .NET/Swift/Rust/Go and alternative-.NET comparisons,
  focused behavioral probes, an assessment of String-model costs/limitations and
  a proposed System.Text capability portfolio.
  Correct stale String indexing/union documentation. Recommend scalar/range/codec
  boundary work before dependent expansion; no proposed API changes are implemented.
  Clarify the separate metadata, logical text, storage and API-boundary encoding
  contracts: String/Char APIs need not expose UTF-8 simply because storage/defaults use it.
- Narrow the text foundation direction to encoding/decoding and possibly a small
  builder, using Swift as the closer text API comparison and retaining grapheme Char.
  Record the supplied alternative design as a proposal, not an approved migration.
  Add an application-only scalar/range/bounded-decoder experiment and focused
  passing evidence; no proposed types are added to System. Track bridge limitations
  and distinguish byte progress from grapheme semantics. Add Text versus String
  as an open naming discussion, separating aliases, type identity, namespaces and
  sequence semantics. Clarify the high-level text API objective and small-surface
  constraint; no rename, new representation or namespace migration is implemented.
  Add a paired String/Text API sketch and focused executable construction, extraction
  and decoding consumer. Preserve snapshots and explicit ordinal search semantics;
  naming remains provisional and the accumulator makes no performance claim.
- Add development Encoding/Decoder contracts, Encodings.Utf8/Ascii and a standard
  Raven EncodingError union. StreamReader and StreamWriter accept selected encodings
  with UTF-8 defaults and leaveOpen overloads. Decode incrementally with owned carry;
  delimit lines after decoding and reject unrepresentable ASCII before output.
  Preserve partial transfers and typed stream errors; add InvalidEncoding cases.
  Malformed UTF-8 can fail before EOF, changing error ordering/cursor advancement
  from Preview 10. Limits distinguish source bytes, text UTF-8 bytes and encoded
  output; errors do not roll back streams. Refresh public metadata, library/API
  snapshots, feature pages and focused consumer evidence, including custom codecs,
  split input and the maximum reader bound. Stateful Encoder and future HTTP
  integration remain planned; no text rename, replacement fallback or new HTTP scope.
- Evaluate bounded report construction with an application-only builder: explicit
  UTF-8 quotas, atomic expected-limit failures, immutable snapshots, clear/reuse and
  decoded combining sequences. Focused contracts and a .NET semantic comparison
  pass. Identical-output diagnostics favor ordinary concatenation over this managed
  implementation; defer public builder promotion and record Encoder progress as the
  next bounded candidate. No runtime/API changes or builder optimization work.
- Add development Encoding.CreateEncoder, Encoder, EncoderProgress/EncoderState
  and StreamWriter.Finish after focused acceptance/drain evaluation. Standard Raven
  state/error unions express progress and Busy. Built-ins retain valid text and
  bounded encoded chunks; UTF-8 defaults and strict ASCII preflight remain. Finish
  drains final bytes separately from Flush/Close. Custom Encoding implementations
  must add the factory; drain/output failures stop later writes and Finish, while
  input preflight failures remain retryable. Custom output-limit failure may follow
  partial output. Refresh public metadata, API/library snapshots and feature docs;
  focused factory/progress, short-write/finalization, maximum writer and selection
  consumers pass. No new charset, native primitive or performance claim.
- Record the author's focused-validation policy: only necessary checks, performance
  tests when relevant, and a full suite only when needed. Skip unrelated website builds.

## 0.1.0-preview.10 — 2026-09-27

### 2026-09-27

- Migrate Option, Result and TaskOutcome to standard Raven unions, using Raven.Core
  as the Option/Result reference. Generated IUnion.Value boxes the active case;
  HasValue distinguishes inactive defaults and TryGetValue preserves unmatched outputs.
  Rebuild consumers: manual Is*/Get* and TryGet aliases and mutable case payloads
  are removed. Propagation output methods initialize their outputs on both branches.
  Support default delegate storage in generated carriers and Object dispatch on
  closed generic boxed values; open generic Object reachability remains rejected.
  Preserve the separate Neo bootstrap ABI with frozen legacy fragments. The Raven
  runtime audit finds no further manual union carriers. Combined union checks, 24
  task pipeline cases, seven runtime task tests and 94 affected/prerequisite tests
  pass, alongside ten focused Raven metadata tests. API snapshot, 18 website tests
  and the 1,025-page build pass. Boxed Completed pattern binding retains a documented
  compiler limitation; an explicit closed-case cast works. Packaged MSBuild and
  installed VS Code hover/build/run checks pass. Update completion verification for
  Raven’s generic labels (Option<T>, Result<T,E>) while retaining host-API rejection.
  Mapped HTTP qualification exposed native reflection snapshots still constructing
  the old Option layout. Validate and construct the selected Raven/Neo layout at
  that boundary; retain server fault output when a peer disconnects during a check.
  Both profile accessor regressions and all mapped HTTP peer/pair checks pass with
  the rebuilt runner, with zero live objects. Final extracted archives also pass
  union/editor, mapped HTTP peer/pair and all 11 upload checks. Record revisions,
  source-CI scope, checksums and remaining compiler observations in release evidence.

- Fix the socket test byte-array spelling flagged by Rust 1.98 Clippy in release CI.
  Wait for Winsock readiness before reporting a nonblocking connect as successful;
  retain portable TCP backpressure checks without assuming the kernel must short-write.
  Refresh full-suite expectations for the existing shared deadline bridge and standard
  SingleError union payloads; preserve cardinality, disposal and allocation assertions.
  Correct the bundled order sample to match FileWriteError.TooLarge for rejected writes.

- Select 0.1.0-preview.10 for the author-requested HTTP POC release. Prepare candidate
  notes and matching packages; publication is pending exact-candidate validation.
  Align website/API guides and installation instructions with the Preview 10 package
  set. Integrate Raven `d48bf14ba` to preserve missing-call diagnostics during terminal
  Fault analysis; no Runtime Contract or metadata shape changes.

- Implement the next-release CI split: one canonical full source/archive run,
  focused macOS/Windows OS and ABI checks, and compile-only minimum-Rust checks on
  all three hosts. Retain opt-in full stable matrix validation, record test profile
  and command timings, and cancel superseded automatic runs. Add optimized release
  validation while preserving the validator's debug default. Exact binary/SDK
  package checks remain separate release gates; no hosted speedup is claimed yet.
  Apply rustfmt to existing runtime formatting drift that blocked the CI format gate;
  these edits do not change runtime behavior. Resolve existing Clippy style
  findings and document narrow native-signature/inline-completion exceptions;
  preserve scheduler polling order and socket defaults.

- Add an optional HTTP POC component to the standard runtime packager, carrying
  the matching runner and complete JSON/client/server/upload checks. Verifiers can
  select a separately extracted SDK explicitly. This enables checkout-independent
  package validation. Fresh macOS arm64 archives pass typed JSON peer checks,
  all 11 upload cases and the repaired website download pair, with zero live objects.
  Record the bounded POC as complete and select release preparation; release
  readiness, a version/date and publication remain separate.
  Include the generic JSON client source in the website sample archive so its
  documented mapped-client check can build outside the checkout. Add the
  YamlDotNet 16.3.0 notice required by the rebuilt SDK dependency inventory.
  Restore source-archive notice coverage for socket2 and ten locked Windows support
  crates, preserving license bytes from checksum-verified Cargo archives.

- Consolidate active planning into the authoritative roadmap and four theme trackers:
  HTTP/networking, runtime/language, library/data and tooling/release. Archive six
  superseded plans with their evidence and preserve old section links. Retain the
  finite HTTP POC finish gates; no new feature, release or completion is claimed.
- Refresh all website feature pages around current behavior, limits and possible
  directions. Remove routine preview migration notes and superseded experiment
  chronology; correct stale HTTP, cancellation, storage, text and reflection claims.
  Use the current tested Thread.Run sample for the Tasks page and download. Preserve
  published release records and accurate setup/package availability. Website
  publication remains separate from this documentation change. All 18 website tests
  and the 1,038-page build/link checks pass. All 11 rendered feature pages are
  inspected; the Task download matches its executable source. Archived-plan text
  and all 22 dated issue assignments are checked. Existing RavenDoc missing-summary
  warnings remain; no runtime suite was run for this documentation-only change.

### 2026-09-26

- Add development known-length HTTP stream uploads through HttpContent.FromStream,
  with explicit source ownership, one-shot admission, 256-byte reads, source errors
  and cancellation cleanup. Buffered byte access remains available; TryGetBytes and
  IsBuffered expose representation, and Bytes faults for stream content. Client
  uploads allow 65,536 bytes; server bodies and responses retain their 1,024-byte
  buffered limits. Refresh bridge/library/API artifacts together for the new
  HttpError.Content case. Focused upload and buffered HTTP/JSON regression checks
  pass, along with 623 bridge signature checks, API/library snapshots, 18 website
  checks and the combined site build. Admit Disposable interface conversion for
  content. Unknown-length uploads, async sources and response streaming remain deferred.
- Set a finite HTTP POC finish line at the current upload increment and create a
  shared client/server capability tracker with completion gates and future HTTP/2/3
  direction. Record the author's next-step request to consolidate tracking by theme.
  No POC completion, protocol implementation, release or deployment is claimed.

- Reprioritize the next release toward feature delivery at the author's direction.
  Plan stream-backed HTTP content next, followed by bounded application-driven
  library additions; defer optimization unless it materially affects a feature,
  supported workflow or release requirement. Preserve known compiler/timeout debt
  and release validation gates. This updates plans, not implemented APIs.

- Reduce temporary host allocation in array-payload quota scans by walking borrowed
  sibling iterators instead of expanding every child into a work list. Preserve
  quota accounting, iterative deep traversal, GC pressure checks and deadlines.
  All 44 focused quota/array tests pass. Record the native-call trace, POST profile
  and identical-input HTTP comparison: successful candidate client runs take
  5.6–7.1 seconds versus baseline 16.0–18.2 seconds, with unchanged managed counts.
  Both runtimes also time out once in three runs; this is a cost reduction, not a
  completed timeout fix. Follow up with per-slot payload summaries, invalidated on
  successful writes/reset and rechecked against current aggregate limits. All 88
  focused quota/reference/GC checks pass. A new identical-input comparison passes
  three exchanges per runtime: candidate client execution 2.3–5.2 seconds versus
  the previous walker’s 5.9–8.1 seconds, with identical managed counts and zero live
  objects. Five diagnostic runs also pass; the earlier timeout remains unexplained.
  Numeric summaries add slot metadata but no GC roots or public API changes.
  Full suite and website build were not run.

- Add serial `--repeat` runs to the HTTP/JSON verifier, building inputs once and
  stopping on the first failure without changing deadlines or GC checks. Record
  the updated-toolchain managed-pair timeout, two successful independent client runs,
  all 12 independent server cases and a fresh unresolved-call/empty-body reproduction. Preserve artifact hashes
  and distinguish these open acceptance gaps from the integrated generic compiler
  fix and JSON helpers. No runtime fix or release-readiness claim is made.

- Integrate generic GetFromJson<T>/PostAsJson<T> extensions with string/Uri and
  cancellation overloads, standard HttpJsonError causes, BaseUri/default headers
  and response association. The mapped neoCLR demo now fetches a report, posts it
  back and reads an acknowledgement. Preserve reference identity for generic box T;
  keep value boxing copies. Integrate Raven's independently tested generic
  method-group fix (main 13b9105d8; neoCLR 56083626e). Track all JSON helper sources
  in library snapshots and reject omitted inputs. Update public API reference and
  website content. Focused helpers, five boxing checks, 609 bridge checks and the
  managed/independent HTTP peers pass with zero final live objects where measured.
  A concurrent-build run timed out; isolated runs pass without deadline changes.
  Full suite and website build skipped.

- Record a neoCLR-focused triage of all 22 open neoCLR and nine open Raven issues.
  Update roadmap navigation and recommend compiler correctness, the end-to-end
  managed HTTP/JSON application and bounded ownership work before broader API or
  architecture changes. Distinguish existing implementations from remaining issue
  scope, identify small companion tasks and record validation/release gates. This
  is a planning update; no fixes, API additions or GitHub issue closures are claimed.
  Reconcile the subsequent generic-method fix and JSON-helper integration before
  committing the plan; sample repeatability is the next bounded validation task.

- Add HttpResponse.Request as Option<HttpRequest>, associated by HttpClient.Send
  for immediate and pending responses through custom/socket handlers. Add copied
  DefaultRequestHeaders; explicit headers win case-insensitively, invalid defaults
  fail before handlers, and transport/content fields remain controlled by their
  existing APIs. Defaults derive an effective request without mutating the original;
  response association exposes that effective request. Preserve ready-task behavior,
  failed/cancelled outcomes and associations when changing response status/content.
  Record a deferred Raven generic method-group/diagnostic issue found while exploring
  GetFromJson/PostAsJson; those helpers are not implemented by this change.
  Focused handler and independent GET/POST wire checks pass with zero final live
  objects; the neoCLR mapped JSON client/server pair also passes (425/369
  allocations, zero final live objects). All 569 bridge signature checks and
  API/library snapshot checks pass.
  Full suite and website build skipped by direction.

- Add provisional System.Web.Http.Json.JsonContent helpers for model/node content
  creation and typed, TypeInfo or node reads. Reuse synchronous buffered JSON/UTF-8
  conversion with unchanged mapping errors and limits; reject oversized buffers
  before decoding. Creation sets the JSON UTF-8 media type; headers, status and
  exchange lifecycle remain caller policy. Update the mapped HTTP sample to share
  conversion across all four client/server boundaries and document the public API.
  Focused content checks, the managed pair and an independent Python server check
  pass with zero final live objects; 559 signature checks and API/library snapshots
  pass. Full suite and website build skipped. Record the next generic-only
  GetFromJson/PostAsJson direction and proposed optional response/request association
  as plans, not implemented capabilities.

- Rename the development JSON DOM entry points to DeserializeNode/SerializeNode
  for string and stream overloads. Object mapping keeps Deserialize<T>, TypeInfo
  reads and Serialize(Object), including its existing node passthrough. Update DOM
  consumers, API reference and website examples; rebuild consumers with matching
  references/library. This is a naming migration, not a codec or mapping change.
  Focused DOM/stream, typed-mapping and HTTP-pair checks pass with zero final live
  objects; 551 signature checks and API/library snapshots pass. Website build skipped.

- Install a committed local SDK/VS Code snapshot and prepare editable HTTP/JSON,
  calendar and async projects with a sample-selecting launcher. Add explicit
  HTTP test-runner budgets, peer verification and reproducible launch instructions.
  Clean up active build guidance and legacy source examples; document website builds.
  Normalize personal home-directory paths to $HOME in documentation and historical
  toolchain records (path spelling only; recorded revisions and checksums retained).
  Record the outstanding unresolved-call compiler diagnostic failure and HTTP
  timeout sensitivity rather than claiming full SDK acceptance.

- Add synchronous Result-based JsonSerializer.Deserialize<T> overloads for string
  and InputStream, preserving the existing flat object-mapping policy and errors.
  Use typed models in the opt-in HTTP sample and refresh API/library snapshots.
  Integrate Raven's independently tested generic-arity fix; rebuild consumers with
  the matching compiler, reference and library. Project these bounded wrappers
  through the existing generic-function ABI rather than broadening class imports.
  Fix unbox.any for reference targets to preserve casts, identity, nulls and GC
  roots, including String-to-Object allocation; exact value unboxing is unchanged.
  Validate five focused runtime tests, 551 bridge signature checks, the public
  mapper, DOM/stream regression and typed HTTP pair; all sample runs finish with
  zero live objects. API/library snapshot checks pass; skip website build as directed.
  Record Node naming and target-directed overload selection as exploration.

- Restore the committed website build with a linked reference for JSON union
  payload properties omitted by the pinned RavenDoc renderer. Validate manual
  member routes and anchors, retain API coverage checks, and correct JSON guide
  URLs on the web feature page. All 1,016 pages and 18 website tests pass.

### 2026-09-25

- Rename Equatable<T> to EquatableTo<T> and Comparable<T> to ComparableTo<T>
  across runtime contracts, Raven sources, bridge metadata, record configuration,
  samples and API documentation. Add invariant ConvertibleInto<T>.Convert() -> T
  with ordinary interface dispatch and implementation-defined conversion policy.
  This breaks source and metadata identity; rebuild consumers with matching
  reference/library artifacts. Equality and ordering behavior are unchanged.
  The local compiler still rejects generated-record interface assignment with
  either old or new names; explicit interface implementations are validated by
  the compiled/imported Raven consumer, 47 focused Rust tests and signature checks.
  Refresh matching library and API snapshots; skip full tests and website build
  by author direction.

- Add provisional JsonSerializer Object/TypeInfo overloads for flat String, Int32
  and Boolean properties, using checked constructors and accessors. Preserve DOM
  overloads and borrowed synchronous streams; use exact names, require writable
  properties on input and reject null/nested mappings. Retain ReflectionError inside
  JsonError and report unsupported mappings explicitly. Integrate request/response
  models in the opt-in HTTP demo and document the public contract. Enable the existing
  target typeof contract for the JSON source slice; no compiler code change. Update
  JSON admission checks and remove redundant post-Fault returns from the DOM fixture.
  Rebuild development consumers with matching references/library for the new union cases.

- Reduce method-resolution allocations by rejecting unrelated names, call forms
  and generic arities before copying definition identities. Preserve existing
  resolution checks and add separate load/verification/execution timings to the
  focused sample runner; HTTP deadline policy is unchanged.

- Record planned JSON HTTP extensions for all four request/response boundaries,
  with shared content conversion beneath client and server conveniences. Exact
  signatures remain open; no new API or asynchronous serializer is implemented.

- Add a private reflection construction checkpoint for public parameterless
  nongeneric reference classes. Resolve loaded type identities and reuse normal
  constructor frames, initialization, faults and GC roots. Report dynamic reflection
  in service analysis. Add checked instance property get/set execution through normal
  accessors and virtual dispatch, with exact scalar boxing, reference/null handling,
  receiver/value validation and GC coverage. Add public System.Runtime.Reflection
  CreateInstance/GetValue/SetValue extensions and a standard ReflectionError union,
  API reference coverage and a propagated application consumer. Project application
  instance properties and retain original source access for reflective admission;
  older imported origins without access information are denied for execution. New
  origin fields require a matching runtime. Preserve terminal System.Fault control
  flow and messages in the bridge. Regenerate the library with Raven's independently
  tested nullable-reference generic-signature fix. Add an application-owned reflected
  JSON report mapper with string/MemoryStream round trips, retained JSON/reflection
  causes and setter-effect checks. An opt-in HTTP variant passes independent peer
  checks and an isolated managed pair with zero final live objects. An earlier
  overlapping run hit the transport deadline; retain the default DOM demo and record
  that timing limitation. Public JsonSerializer object-mapping overloads remain
  under investigation.

- Fix constructors of imported reference async state machines containing hoisted
  nondefaultable Result/union fields. Explicit deferred field storage preserves
  checked reads before assignment and GC tracing afterward; ordinary constructors
  and value-type rules are unchanged. New artifacts require a matching runtime.
  Add focused storage and pending-await/GC regressions.
- Integrate the public JSON DOM into the HTTP report client/server sample: POST
  acknowledgement, structured application errors and 400/404 JSON responses, checked
  with independent Python peers. Keep the station-only report minimal; larger payloads
  still expose a transport/performance limit. Record frozen-compiler limitations
  and a bounded reflection/object-mapping investigation (not implemented APIs).

- Add Console.WriteLine(object?) with virtual ToString and empty-line null handling,
  plus unboxed Boolean, Char and integral overloads. Add private signed/unsigned
  64-bit decimal formatting services; floating-point formatting remains deferred.
  Update public API documentation and add scalar-boundary/object-dispatch checks.

- Support Main(arguments: string[]) in the Raven managed collection profile through
  a parameterless startup adapter. Supply a fresh array excluding the executable;
  preserve Environment.GetCommandLineArgs semantics. Include focused argument/GC
  and signature checks and migrate the JSON corpus entry.

- Add provisional System.Data.Json DOM APIs: a closed JsonValue hierarchy with
  kind-specific containers and scalar properties, exact number tokens, JsonError
  unions retaining I/O causes, and DOM-only JsonSerializer string/stream overloads.
  Keep small byte/depth/node bounds and borrowed stream ownership explicit. Include
  API/website documentation, public-consumer/corpus checks and closed-family bridge
  admission. Batch corpus inputs through the measured fixture runner with an explicit
  instruction budget; runtime/CLI defaults are unchanged. Application-local JSON
  experiments remain historical, not compatibility contracts; reflective object
  mapping and HTTP JSON extensions remain planned. Serialization stays synchronous
  and Result-based by author decision.

- Add provisional System.IO.MemoryStream with bounded managed storage, shared
  read/write position, absolute seeking, zero-filled gaps and explicit close semantics.
  Include bridge admission, API documentation and focused stream/DOM checks.
  Prioritize the JSON DOM before reflective object mapping.

- Add an experiment-local JsonSerializer string/stream adapter over the bounded JSON
  DOM codec. Exercise StreamReader/StreamWriter with short transfers, UTF-8, limits,
  I/O failures and borrowed ownership. Public System.Data.Json APIs and reflection
  mapping remain planned; record future HttpClient GetJson/PostJson extensions.

- Add development HttpServer.Accept and HttpContext ownership with Request/Response,
  explicit asynchronous Complete, Close/Dispose, Respond forwarding and UTF-8 RespondText.
  HttpResponse.Respond configures status and optional content without sending. ServeOne
  shares the context path and gains cancellation; handler errors/cancellation end the scope.
  Server.Close now closes active exchanges as well as the listener (a behavior change).
  Bound outstanding scopes and request/send deadlines; keep interface-based inbound/outbound
  message implementations as future direction. Update API docs, examples and lifecycle checks.
  Record the author’s provisional JSON DOM/reflection round-trip release exploration and
  its missing reflective access/construction prerequisites and string/stream serializer
  acceptance cases as planned work. Retain a failing captured-callback reproduction and document frozen-toolchain stabilization gaps.

- Add bounded chunked and close-delimited HTTP response reception and HEAD request/client
  overloads, with request-aware decoding and body suppression in server replies. Preserve
  body/header limits, reject ambiguous/truncated framing, and document unsupported chunk
  extensions/trailers. Expand focused peer/HEAD checks and API documentation. Increase only
  the bridge library import method bound to 256; application imports remain limited to 128.

- Add development Put/Patch/Delete client string/Uri/token helpers and request factories
  through the existing Send pipeline. Extend server parsing to PUT/PATCH buffered bodies
  and bodyless DELETE; reject nonempty GET/DELETE bodies before client I/O. Preserve
  BaseUri, cancellation and response-status behavior. Add focused overload and independent
  peer checks, API documentation and a propagated PUT example. HEAD/framing and the
  planned HttpContext lifecycle remain later slices; no native/compiler policy changes.

- Record an exploratory HttpServer accept/context API alongside callbacks for the
  server-lifecycle slice, including response completion, ownership and cancellation
  questions and the .NET comparison. Record the author's clarification that HttpContext
  is the foundational per-exchange application scope, closed/disposed when handling is
  done; distinguish completion/error reporting from cleanup. No runtime or public API change.

- Add development HttpRequest.WithHeader returning a new request with a replaced
  application field. Validate names/values and reserve transport/content controls;
  socket Send now serializes application headers for GET and POST, revalidates before
  DNS and enforces count/byte limits. Add propagation-based construction and focused
  handler/wire checks. Start the network deadline after request validation/snapshotting
  so preflight errors retain their type. Update API/reference snapshots; content remains
  buffered/shared. Record eventual with-expression exploration for suitable With*
  methods, excluding this collection-entry builder; no compiler convention is added.

- Raven neoCLR compiler analysis now treats `System.Fault(string)` as terminal,
  diagnosing following code as unreachable and accepting Fault-only return paths.
  Preserve runtime calls and recognize terminal statements in lowering/emission;
  no Runtime Contract configuration change. Validation is recorded in the Raven
  target integration documentation.

- Add request/response GetHeaderValues with ASCII case-insensitive matching and ordered,
  separate values for repeated fields. Missing or invalid names return an empty snapshot;
  no comma splitting, field parsing or synthesized transport headers. Update API reference
  and focused checks. Record stream-backed HttpContent as planned support, with lifetime
  and framing contracts still open; the current implementation remains buffered.

- Add development buffered POST through HttpClient string/Uri/token overloads and
  HttpRequest factories. Expose request content and UTF-8 content construction;
  validate/snapshot outbound bytes before I/O and read bounded Content-Length bodies
  before server callbacks. Keep framing provider-owned and GET bodies unsupported.
  Add a propagation-first text example plus independent peer, .NET and malformed-body
  checks. Update API/website sources; streaming, general headers and other verbs remain
  later slices. Record nearby application error converters and future System.Error
  cause/context exploration. Existing private request construction signatures require matching
  library/reference rebuilds; no native runtime or compiler policy changes.

- Add development HttpStatusCode names using the existing Int32 enum representation.
  HttpResponse.StatusCode and HttpError.UnsuccessfulStatus now carry that enum;
  rebuild consumers and cast to int for numeric formatting. Keep the integer response
  constructor and add a typed overload. Preserve unnamed codes and numeric error
  diagnostics; named constants do not imply transport support. Document optional
  property patterns separately from positional deconstruction. Remove an unnecessary
  peer ordering dependency in the cancellation fixture; headers and isolated-body
  checks pass, while competing-load timeout sensitivity remains documented.

- Expand development HTTP final statuses to 200–599. Send/Get preserve responses;
  IsSuccessStatusCode and GetString apply the 200–299 success range, with typed
  UnsuccessfulStatus errors. Handle bodyless 204/205/304 and serialize final statuses
  in HttpServer. Add independent client/server status checks and an application-owned
  extension error-conversion example; improve importer rejection context without
  widening admitted instructions. Record a supplementary cancellation fixture timeout
  that remains unresolved; do not count that regression check as passing. Request
  methods/content remain the next slice.

- Add token-aware HttpClient Send/Get and string/Uri GetString overloads. Forward
  cancellation through DNS/connect/transfers and close exchange-owned connections
  before task cancellation. GetString preserves HTTP errors and strictly decodes
  buffered UTF-8; malformed bytes produce Protocol, with status support still limited
  to 200 at that checkpoint (expanded above). HttpHandler implementations must migrate
  to Send(request, cancellationToken)
  and rebuild; client/concrete-handler tokenless overloads remain. Validate request
  isolation, cleanup, late cancellation and selected independent HTTP interoperability.

- Add private runtime-library DNS/socket operation cancellation hooks. Extend socket
  cancellation to pending connect/accept while preserving listeners, committed
  outcomes and callback/result acknowledgement. Retain cancelled operation slots
  until consumption and blocked DNS host-capacity permits until host work returns.
  Validate cleanup, completion races, stale/foreign handles and exact native service
  signatures. Add development CancellationToken overloads to DNS lookup and all socket
  connect/accept/send/receive forms, including private shared-deadline paths. Preserve
  native completion winners, dispose registrations before result consumption and
  acknowledge cancellation through Task. Tokenless overloads remain available; source
  disposal does not cancel I/O. Refresh API reference coverage.

- Add development HttpClient.BaseUri as Option<string>, plus Get(Uri) overloads
  on HttpClient and HttpRequest. Both address forms use the existing Uri parser
  and resolver before handler dispatch. A configured base requires relative URLs
  without authority; no base requires an absolute HTTP URL. Fix query-only absolute
  URLs, preserve UriError causes, and reject malformed percent escapes/control bytes
  as HttpError.InvalidUri (a change from the prior request-parser errors). Update
  the client sample, API reference and focused .NET/handler/interoperability checks.
  Record HTTPS adapter feasibility and validation needs without adding TLS support.
  Token-aware HTTP/native cancellation and GetString remain pending in slice 4.

- Add development System.Concurrency cancellation source, copyable token and
  disposable registration APIs. Requests invoke callbacks synchronously within
  one invocation; operations still own cleanup and Task cancellation acknowledgement.
  Cover default/copied/boxed tokens, callback ordering/reentrancy/disposal and GC.
  Keep timers, linked sources, cross-thread use and HTTP/socket token wiring pending.
  Admit the token's exact source-reference layout and captured receivers in the
  compiler bridge, with internal-helper guards and updated API documentation.

- Integrate development HttpError results into HTTP client, handler, request and
  server APIs using a standard payload-bearing union. Preserve resolver/socket
  causes and distinguish request, protocol, unsupported, limit, timeout and handler
  failures. Rebuild matching SDK/reference/library/application artifacts. Content
  decoding retains its separate provisional string-error contract. Add projected
  union consumer bindings and retain constructor instructions during import; public
  payloads, extraction and private access receive focused checks. Add selected-slice
  library regeneration for focused iteration. Follow author direction to use
  relevant per-slice tests and skip website builds while keeping docs current.
  Later entries cover cancellation foundations and BaseUri; HTTP token wiring remains pending.

- Record the networking/web release objective and a proposed scope: request/body
  and status support, cancellation/lifetimes, interoperable framing, server shutdown,
  a storage-backed sample and focused release validation. HTTPS remains an explicit
  scope decision. Add the author-selected IPAddress union with IPv4Address and
  IPv6Address cases to release scope, superseding its suggested deferral. The author
  subsequently selects a closed class hierarchy instead; public integration remains
  pending. These are plans, not implemented capabilities.

- Begin the address hierarchy slice with an isolated immutable-address probe. Admit
  protected direct-base constructor calls from derived constructors in the bridge,
  retaining private and unrelated-call restrictions. Check value equality, hashing,
  defensive copying and collection with live base-typed references. No public
  IPAddress API or IPv6 transport is introduced by that initial checkpoint.
  Follow with public IPAddress, IPv4Address and IPv6Address classes, strict parsing,
  canonical formatting, value equality and typed IPAddressError outcomes. DNS now
  returns Sequence<IPAddress>; update explicit string-result annotations and rebuild
  matching artifacts. Socket retains string overloads and adds address/value-sequence
  overloads, preserving bounded snapshots/deadlines. IPv6 transport reports the new
  UnsupportedAddressFamily case; scopes and IPv6 DNS/transport remain unsupported.
  Update HTTP's DNS stage, the loopback sample, on-site API reference and networking
  guide. Record reduced compiler/runtime observations without claiming compiler fixes.

- Extend the development union projector/importer to matched nongeneric sequential
  payload families. Verify an HttpError library prototype with URI/DNS/socket causes
  and text payloads, separate consumer compilation, copies/boxing/GC, and rejection
  of changed payloads and overlapping layouts. Keep private single-argument
  conversions at their validated call sites. Generic companions remain unsupported;
  public HTTP signatures are unchanged in this bridge checkpoint. Record the
  token-aware Send primitive and Get/GetString layering as the HTTP integration
  target, with cancellation behavior still to be implemented. Record BaseUri as
  planned optional string configuration: relative verb addresses with a base,
  absolute addresses without one.

- Add development System.Enum.GetNames/GetValues helpers with both TypeInfo and
  constrained generic overloads; typed values preserve enum identity. Add
  TypeInfo.GetEnumValues snapshots and shared unsigned ordering with aliases.
  Format admitted Raven enums through Object.ToString, including flags and unnamed
  numeric values. Document current Int32/three-enum admission and snapshot costs.
  Keep the historical Neo bootstrap on explicit legacy carrier/enum snapshots
  while Raven uses migrated unions. Refresh the stale bootstrap native-service
  count for existing string services. Record built-in constants and additional flags
  helpers as on hold; return to union closure and HTTP after this slice.

- Migrate the remaining empty-case storage, stream, text, console, parsing,
  division and LINQ error unions to standard Raven declarations. Replace their
  Is*/Get* helpers with case patterns, preserve producer outcomes, and document
  inactive defaults. Resolve value-case tokens used by isinst against the supplied
  core definition. Change EntryKind to a non-flags enum (File = 1, Directory = 2),
  leaving zero unnamed and lookup failures in StorageLookupError. Rebuild matching
  SDK, runtime library and applications. Update API references and legacy routes.
  Record generic/payload projection blockers explicitly. The large storage contract
  fixture needs a larger explicit test instruction budget; production limits stay
  unchanged.
- Record the proposed future Error interface for diagnostic integration,
  composition and decoration; no Error API or stack capture is implemented.

### 2026-09-24

- Clarify Raven storage conventions: prefer private `var`/`val`, which emit fields;
  reserve explicit `field` for intentional field declarations or compatibility.

- Make standard Raven union declarations the class-library default, with documented
  rare manual-contract exceptions. Add a direct HTTP-error union probe with members
  and a CLI shape report; record current importer gaps instead of extending the
  hand-authored carrier catalog. Admit value-constructor receiver initialization and
  recognized union conditional outputs in application imports; validate nested source
  unions, defaults, copies and boxing under GC pressure. Admit marked empty-case-only
  explicit-layout unions as managed field slots; reject nonempty or malformed layouts
  and exercise unions across a separately compiled dependency. Add a bounded bootstrap
  path that matches an empty-case union family against a separate core reference,
  imports generated bodies and conditional outputs, and preserves constructor checks
  through per-field default initialization. Encode generated static helper names.
  Preserve and validate Raven case metadata in the bootstrap reference fixture;
  check generic companion associations and separate consumer compilation. Keep
  Raven metadata in the development bridge, without a runtime dependency or a new
  platform case-map convention. Admit a validated core-owned IUnion protocol during
  library import, without duplicating its declaration; retain the source-owned
  bootstrap path and exercise shared-interface compilation and execution. Extract a
  reusable empty-case reference projector that replaces existing carrier/case definitions
  in place, reuses shared support definitions and rejects changed case identities.
  Check SocketError reference consumers and native import from standard union syntax;
  matched standard shapes override legacy erased-carrier initialization assumptions.
  Migrate SocketError to normal union syntax and generate its reference shape from
  embedded source in both SDK/core paths. Remove per-case Is*/Get* helpers; use case
  patterns. Add shared IUnion support, conditional case calls and inactive defaults;
  rebuild matching library and applications because storage and member contracts change.
  Nested source unions containing SocketError now execute with copying/boxing/GC checks.
  Use unqualified match arms in ToString; check all 13 cases and inactive defaults.
  Clarify empty-case unions as the basic form, data-bearing variants as the stronger
  modeling case, and enums as an alternative for named constants.
  Real TCP, managed listener/client and selected HTTP success/failure checks pass.
  Update generated library and API/website documentation. Per-case predicates are not
  a required convention for any union, including Option and Result. Batch remaining
  applicable union migrations before resuming HttpError/BaseUri (still unimplemented).
  Migrate DnsError and UriError next, projecting each source against the preceding
  core so compiler-support identities are shared. Remove their per-case Is*/Get*
  helpers in favor of patterns; inactive defaults expose HasValue false, Value null
  and ToString Empty. Matching SDK, library and applications must be rebuilt.

- Add managed System.Uri and UriError with strict escaped-ASCII parsing, string/Uri
  Resolve overloads and RFC 3986 relative resolution. Preserve lexical Text equality
  through Object and hashing; bracketed IP literals, IRI/IDNA and canonical resource
  equality remain unsupported. Add executable RFC/.NET comparison and GC checks.
  HttpError and HttpClient.BaseUri remain the next integration slices; URI/URL
  encoding utilities are recorded as later work.

- Add private absolute-deadline submission paths for resolver, address fallback and
  socket transfers. Preserve the shorter phase bound, reject expired work before
  native admission and prevent short progress from renewing a shared budget. Add a
  native loopback lookup/connect/body probe and deterministic ownership tests. Wire
  HttpSocketHandler to a provisional 15-second exchange budget, with late-success
  cleanup and trickling-peer checks. Public signatures remain unchanged; new private
  deadline services require matching runtime/reference/bridge/library artifacts.
  Keep Until helpers internal in application references, with bootstrap-only access
  and explicit library-import checks. Custom pipeline/server handler work is not bounded.

- Bound each pending nonempty socket Send/Receive to a provisional five seconds from
  native admission. Timeout releases transfer storage and returns TimedOut without
  closing the connection; committed outcomes and empty transfers retain their behavior.
  This changes previously unbounded development operations and is not configurable.
  HTTP closes its owned connection through existing error paths. Add deterministic
  owner-clock and stalled HTTP peer checks, and update API/website documentation.
  Whole-request, accept and handler deadlines remain open.

- Add a two-process HTTP JSON report sample using the integrated client/server APIs
  and the unchanged application-local JSON codec. Validate both applications against
  independent Python peers, structured JSON output and GC reclamation. Include the
  tested source as a website example/download; no public JSON API or POST is added.

- Add a provisional HttpServer with Listen/GetLocalPort/ServeOne/Close and received
  HttpRequest.Headers. Serve one bounded GET/200 exchange, validate response headers
  and supply framing; expected errors close the connection. Add a greeting app,
  neoCLR/.NET interoperability and malformed-request/response checks under GC, plus
  matching reference/library artifacts and on-site API/feature documentation. Add
  opt-in live console output to the trusted async fixture runner. No transfer deadline,
  general HTTP hosting loop, TLS or public cancellation is added.

- Add provisional System.Web.Http client/handler APIs and a sample with bounded byte framing,
  UTF-8 content, fake/forwarding handlers, a .NET comparison and 19 passing
  client cases including an independent Python HTTP server. Add generated API
  reference coverage and a Web feature page/homepage entry. Keep
  contracts provisional; transfer deadlines remain
  open. Admit nested callers accessing containing private fields in the importer,
  with negative access/readonly checks. Allow an explicit instruction budget in the
  trusted async fixture runner without changing runtime defaults. Record future Uri
  and IPAddress value-object direction, BaseAddress after Uri and a candidate nested HTTP error model;
  those types/unions are not implemented in this slice.

- Exclude non-generic Array, Option, Result and TaskOutcome importer/exporter
  scaffolds and their CLR case carriers from API pages and navigation; retain
  generic APIs and the full metadata inventory with explicit, reasoned coverage exclusions.

- Add symbol-derived member documentation structure with linked parameter and
  property/return/field/event types, even without authored prose. XML and Markdown
  enhance the generated contract; label the owner as Declaring type. Update the
  pinned RavenDoc build from the shared implementation on both Raven branches.

- Add Socket.Connect(Sequence<string>, port) with synchronous input snapshotting,
  full validation, duplicate removal and ordered IPv4 fallback. All attempts share
  five seconds; pending attempts get at most one second while alternatives remain.
  Extend the Raven echo POC to exercise fallback and caller-list reuse under GC.
  Refresh matching bridge/library/API artifacts. DNS remains separately bounded;
  fixed connection limits are provisional, with no configurable retry policy. Expiry
  closes the native socket and returns TimedOut; committed outcomes survive delayed
  delivery. The single-address overload shares the five-second bound. Accept and
  transfers retain their existing behavior.

- Restore Google Analytics measurement ID `G-SVXYRRCEEK` across authored and
  generated API pages through RavenDoc configuration, after the website migration
  dropped the previous tag. Update the pinned upstream publisher.

- Enable RavenDoc flat namespace navigation for the neoCLR API Browser, with
  full namespace names as peers and expandable type groups. Update the pinned
  upstream generator; hierarchical mode remains available and URLs are unchanged.
  Keep namespace rows expandable even when they only contain an overview, and
  display declared built-in type names in navigation and headings while preserving
  Raven aliases in code signatures. Ignore API-inventory build outputs.

- Include every public reference type in API generation, rather than maintaining a
  selected subset. Add collection classes/interfaces and the remaining public API
  families, with type/member descriptions and sample-oriented navigation. Check the
  full metadata inventory automatically; missing prose no longer suppresses pages.
  Document three pinned-renderer limitations through explicit manual type entries.
  Website publication remains a separate operation.

- Add Socket.Listen, asynchronous Accept and GetLocalPort for a two-process neoCLR
  echo POC. Reserve connection capacity for pending accepts; preserve accepted sockets
  after listener close, with that ownership rule documented on Close. Add
  AddressInUse and InvalidOperation outcomes, generated
  API docs and the downloadable server/client example. Validate 36 focused runtime
  tests, ten website tests, 582 API/site pages and both processes with zero retained
  objects. Accept deadlines, individual
  cancellation, overall connection deadlines and address fallback remain open.

- Record future IPAddress value-object direction and a possible HostEntry, with
  .NET comparisons and open parsing, equality and migration choices. This is planning;
  current DNS/socket signatures and immediate listener work remain unchanged.

- Add TypeInfo.IsOpen, IsClosedHierarchy and nominal IsUnion alongside IsAbstract,
  IsEnum and IsValueType. Preserve descriptive sealed/closed-family metadata in the
  Raven importer and neoIL; closed families are distinct from non-inheritable leaves.
  Rebuild matching bridge, reference and library artifacts. General raw-IL hierarchy
  enforcement and the richer TypeInfo interface family remain future work.
  Validate the compiled flag sample with seven collections and zero live objects,
  matching bootstrap/API snapshots and the combined 575-page website.

- Add development Dns.GetHostAddresses with Task/Result, read-only IPv4 address
  sequences, typed lookup errors and a provisional five-second deadline. Extend
  the socket echo client to resolve localhost, and keep lookup separate from
  connection. Refresh matching runtime/bridge/API artifacts and add the Networking
  feature page and homepage box; Web gets its own entry when its HTTP POC works.
  Document open array-generic metadata and nested callback capture integration cases;
  no compiler fix, automatic address fallback or overall connection deadline is claimed.
  Validate 25 focused runtime tests, the compiled hostname echo (31 collections,
  zero live objects), three visibility checks, ten website tests and 572 API/site pages.

- Add a private host-resolution scheduler source as the first DNS integration slice.
  Keep blocking lookups off the VM thread, bound process-wide host work and retained
  invocation results, and preserve capacity accounting after cancellation/teardown.
  Add deadline, late-result, failure and traced-callback lifecycle coverage. Public
  DNS APIs and the compiled hostname client remain pending; Socket.Connect is unchanged.
  Validate 20 focused resolver/scheduler/socket-VM tests and the 555-page website.

- Add Socket.Send with Task/Result byte-count completion and bounded source snapshots.
  Allow one pending send and one receive per connection, sharing transfer/operation
  budgets; preserve short writes, pending backpressure and committed close outcomes.
  Extend the Raven client to send and receive a real host echo, and refresh its API
  reference. Private completion/result services change; rebuild matching artifacts.
  DNS, listener/accept and HTTP remain pending. Record explicit character/string
  encoding in the later HTTP path and adopt the current Raven `await Foo()?`
  propagation syntax in the socket sample and conventions. Validate 35 focused
  runtime tests, the compiled echo with 30 collections and zero live objects,
  matching bootstrap/API snapshots and the combined 555-page website.

- Refine the networking plan toward the HTTP demo: use Socket directly beneath a
  private byte-I/O boundary, bring host-backed DNS into near-term client work, and
  define bounded request/header/response and framing milestones. DNS, HTTP and TLS
  remain planned, not implemented; TcpClient/UdpClient are not prerequisites.

- Add the first development System.Networking.Sockets client: Socket.Connect and
  Receive return Task<Result<...>>, with typed SocketError and idempotent Close.
  Numeric IPv4 TCP uses nonblocking native operations and the private scheduler;
  pending callbacks/buffers are traced and results consumed once. Add a Raven greeting
  sample and on-site API reference. Send, listener/accept, DNS, IPv6, per-operation
  cancellation and runtime suspension remain pending. Record TcpClient/UdpClient as
  later convenience-layer candidates, not prerequisites or implemented APIs.
  All 31 targeted runtime checks, the Raven client/visibility checks and 554-page
  website build pass. Keep the existing hoisted-Result-across-await limitation
  explicit; this slice does not relax initialization checks or implement suspension.

- Add a private reusable TCP receive registry behind the invocation scheduler.
  Separate connection/read lifetimes, preserve connections on read cancellation,
  retain completed results within admission quotas until consumed, and reject
  stale/foreign handles. Route the real-TCP VM fixture through this backend; host
  injection remains test-only, with no public Socket API or runtime suspension.
  All 35 targeted backend/VM/scheduler/worker checks and the combined website build pass.

- Retain ready callbacks and their default-queue destinations in a bounded, traced
  scheduler slot until active-frame installation succeeds. Failed installation keeps
  the roots; occupied slots do not consume more completions. Preserve current affinity.
  Clarify application, transitional compiler/TaskQueue and private runtime contract
  tiers: implement useful APIs now with generated state machines; defer suspension.
  All 28 targeted scheduler/TCP/worker checks and the combined website build pass.

- Plan an internal scheduling boundary before public socket integration, preserving
  runtime-owned async as the direction and TaskQueue as transitional machinery.
  Record completion/continuation ownership, wakeup and affinity requirements with
  .NET comparisons; update roadmap and Tasks documentation. No scheduler or runtime
  suspension API is added in this design checkpoint.
  Implement the initial private invocation scheduler: common rotating completion
  arbitration at idle and callback boundaries, consolidated GC roots, and durable
  worker wake hints. Preserve TaskQueue/affinity behavior and bounded polling for
  cancellation and the test-only socket source; runtime suspension remains pending.
  All 36 targeted scheduling/worker/TCP checks and the website build pass.

- Check pending socket-receive ownership with real loopback TCP in an isolated
  six-case probe: exact range delivery, retained destinations, terminal ordering,
  admission quotas and teardown. Guest Socket/Task integration remains pending.
  Extend the fixture to actual managed arrays and collector roots with five TCP
  cases; cancellation preserves the connection, and admission counts ready callbacks
  as well as pending reads. All 13 managed-heap ownership cases pass.
  Add a test-only TCP adapter to the actual VM collection and TaskQueue paths.
  Move bounded worker waits into invocation-level arbitration; the normal runtime
  still exposes no Socket API. Cover queue progress and terminal resource cleanup;
  five VM socket tests and all 26 worker tests pass.

- Select sockets as the next API work towards a web app running on neoCLR, keeping
  the networking proposal as the direction. Record provisional interfaces and
  lifecycle behavior. Add an isolated nonblocking TCP transport probe with eight
  checks and a .NET baseline; guest Socket APIs, completion/GC integration and
  cancellation remain pending. Update roadmap and website direction without
  changing production runtime or compiler APIs.

- Integrate explicit `String.Intern` with one bounded strong pool per execution;
  isolated workers and separate host invocations have independent pools. Retain
  canonical text until completion, Fault or cancellation while preserving returned
  owners. Add `Limits.intern_entries`/`intern_bytes` (4096/1 MiB defaults) and
  `InternPoolLimitExceeded`; hosts with exhaustive Limits initializers must add the
  fields. Update compiler reference/importer, API docs and samples together. No
  automatic literal interning, lookup API or shared runtime-session pool is added.
  Give String parameters meaningful names across source, reference metadata,
  runtime introspection and API docs. Positional calls are unchanged; named callers
  must replace value0/value1 with the documented operand/range names.

- Explore explicit String interning with a test-only bounded owner pool and repeated
  log-field names. Check canonical identity, insertion quotas, GC/host retention and
  pool teardown against a .NET baseline. Record execution/session lifetime questions;
  no production pool, guest API or automatic literal interning is introduced.

- Enable String ReferenceEquals and explicit Object base equality/identity hashing
  using the shared text owner across Object/interface wrappers. Preserve virtual
  content equality/hashing and ToString identity. Identity hashes survive GC and
  host retention without exposing addresses; collisions are allowed. No interning
  or literal-identity guarantee is added. Update runtime/Raven checks and API docs;
  compiler metadata and Runtime Contract settings are unchanged.

- Add `String(Sequence<char>)` with immutable snapshot construction, including char
  arrays. String implements Sequence with a read-only grapheme indexer and explicit
  Collection.Count; Length stays public. Preserve exact text without normalization;
  adjacent input characters can merge into a grapheme. Update target metadata,
  constructor lowering, API reference and tested Raven examples. Rebuild the matching
  bridge/reference and runtime together; Runtime Contract settings are unchanged.
  Record the author’s direction to let real cases drive APIs after the semantics
  slices; Iterable construction remains undecided.

- Investigate String storage and identity with an allocation-counting Release probe,
  a private shared-text prototype and a .NET alias/record/array baseline. Record
  migration, GC, host-lifetime and accounting gates, and consolidate stale roadmap
  status. Implement immutable shared UTF-8 String payloads: value copies and String
  Object display retain text; owned producers transfer their buffer. Keep String
  identity disabled, worker transfers owned and array byte quotas logical. Rust hosts
  migrate owned constructors to `Value::String(text.into())` and extraction to
  `into_owned()`. Cover GC/host lifetimes and record future System.Text and general/
  string comparers, casing and comparison methods without introducing those APIs.
  Use .NET as a design reference, allowing justified API differences. Add direct VM
  owner-retention checks across Object/interface conversions, display, fields, arrays,
  erasure and local byrefs, including GC pressure and host/cyclic/fault teardown.
  Keep guest identity disabled pending a complete comparison/hash contract.

- Support String Object content equality, matching UTF-8 hashes and unchanged text
  display through existing wrappers. Cover Object map keys, casts and GC; retain
  explicit rejection of String identity operations. Add generated reference for
  existing String methods/properties/operators and document the representation and
  provisional hash limitations. Collect before casts/type tests allocate String
  wrappers, retaining stack roots; default class/array equality rejects String
  operands without invoking identity. No compiler or managed layout change.

- Align boxed Char equality, hashing and display with its exact grapheme text,
  preserving copied values and distinct box identity without Unicode normalization.
  Add combining-text/emoji map and GC checks, and generated reference coverage for
  Char’s existing typed methods. Unlike .NET Char, the value remains a grapheme,
  not a UTF-16 code unit.

- Add boxed Single/Double Object equality and hashing with .NET-compatible NaN and
  signed-zero semantics. Preserve exact types, copied boxes and IEEE operators;
  floating boxed display remains unsupported. Add native and Raven map/GC checks,
  a .NET baseline and API documentation. Record generic math interfaces as a later
  exploration, not an implemented API or release commitment.

- Adapt the landing hero and inset code background to Light/Dark/Auto while
  preserving Raven syntax highlighting. Correct development async documentation
  and the executable sample to use `await input?`: await first, then propagate
  Result Error or Option None; explicit parentheses remain valid on older tools.
  Document manual website-only publication without a runtime build or release.

- Mark custom case carriers, including StreamError and TextReadError, with
  UnionAttribute in runtime source and reference metadata. Raven imports the
  typed constructor/IsCase/GetCase contract as IUnionSymbol and RavenDoc groups
  cases under their union. Standalone error structs remain ordinary structs.
  Use direct case construction in runtime implementations now that imported
  carriers have union semantics. Refresh bootstrap fragments and correct the
  carrier admission test's moved namespaces. This changes metadata classification,
  not storage or extraction lowering. Integrate Raven's existing configured-unit
  and structural-array identity fixes on its neoCLR branch for imported Flush
  and byte-stream interface signatures.

- Support Object.ToString for boxed Int32, Int64 and Boolean: culture-independent
  decimal integers and True/False text. Check integer limits and copied values
  after GC, and update API/website documentation. Other primitive formatting,
  format strings and culture providers remain unsupported.

- Migrate the website and API reference to one RavenDoc build. Adapt guides to
  Markdown and use an HTML landing page with a separate hero and feature cards.
  Add front-matter title/layout/outline controls, shared branding and a compact
  unreleased-documentation notice. Vendor the portable .NET 10 publisher with an
  immutable upstream revision, checksum and update script; remove DocFX and Node
  build dependencies. Generate Raven type/member pages from the checked reference
  assembly and authored documentation, including previously excluded callback and
  unit-result signatures. Preserve legacy reference routes, sample downloads and
  manual publication; retain explicit reference-coverage gaps. Simplify API lists
  to name-first signatures with parameter/property/field and return types plus a
  static icon marker, with full Raven list signatures available by opt-in.
  Distinguish classes (C), interfaces (I), enums (E), unions (U), delegates (D)
  and structs (S) on lists and detail pages. Add a shared
  RavenDoc API Browser with expandable namespace/type navigation, current-type
  highlighting and an off-canvas drawer on small screens. Compile authored
  nested `toc.yml` menus into section navigation, separate from main menus and
  page outlines. Add an N-logo favicon and a persistent Light/Dark/Auto theme
  menu. Share Raven website syntax highlighting for snippets and API
  signatures, with the engine bundled locally. General publishing changes are
  maintained upstream in Raven.

- Add boxed Int64 Object equality over the complete copied payload and a .NET-compatible
  lower/upper-half hash. Preserve exact-type comparisons and distinct box identity.
  Cover limits, deliberate hash collisions, Object-keyed maps and collection; boxed
  formatting and other primitive contracts remain separate work.

- Admit intrinsic Object in supported generic API signatures, matching ordinary
  import mapping. This enables HashMap<Object, Object> with explicit equality/hash
  callbacks and Option<Object> results. Check mixed Path/type/class/boxed keys,
  payload aliases, collisions, replacement, table growth and GC; all 955 objects
  are reclaimed across eleven collections. No default comparer, new primitive
  boxing behavior or runtime/library layout change is introduced.

- Give parameter snapshots owner-aware Object equality/hash and Name display,
  using closed declaring type, member kind/index and position even with absent
  parameter tokens or shared property accessors. Retain compact owner keys without
  member/parameter cycles; a public Member property remains future work. Document
  ParameterInfo and BindingFlags in the generated API reference. The internal
  parameter layout changes: rebuild the development runtime/library/SDK together;
  the archived value-descriptor profile retains its original layout. The parameter
  sample reclaims all 894 objects over 25 collections.

- Align field, method and property descriptor Object equality with kind, closed
  declaring type and definition index, with consistent hashes and Name display.
  Preserve Object overrides when projecting the shared descriptor base metadata.
  Add generated API coverage and document declaration-only queries; inherited
  traversal remains unimplemented. Add focused member
  identity and map/GC validation, reclaiming all 785 objects over ten collections;
  hashes and indexes are not persistent identities.

- Give AssemblyInfo and ModuleInfo wrappers scoped catalog equality, consistent
  Object hashing and assembly/module display names. Equal descriptors may remain
  distinct allocations; full assembly identity and module scope are required, not
  short names or tokens alone. Cover their public APIs in DocFX and explain the
  one-loaded-program limitation on the website. Extend descriptor map/GC validation
  and add colliding-name catalog regression coverage. The expanded Raven fixture
  reclaims all 476 managed objects across ten collections.

- Align RuntimeTypeInfo Object equality with represented type identity; hash its
  FullName consistently and display the represented type rather than the wrapper.
  Keep Equals(TypeInfo) non-nullable and Object.Equals(Object?) explicitly null-aware.
  Validate repeated, generic and array descriptors, typeof/GetType agreement and
  HashMap use under collection pressure. Add browsable TypeInfo/MemberInfo API
  reference and website guidance; hashes are neither unique nor persistent
  identifiers. Validation:
  the fixture reclaims all 422 managed objects across ten collections; three
  descriptor regressions and the Object reachability regression pass.

- Align Storage.Path typed and Object equality, hashing and display; implement
  Equatable<Path> and reuse the contract in HashMap lookup, replacement and rehashing.
  Keep typed equality operands non-nullable (Equatable<T>.Equals(T)); the existing
  Object.Equals(Object?) overload explicitly rejects null and unrelated objects.
  Preserve ordinal spelling and provider-independent semantics. Retain Object
  ancestry/constructor chaining for library class overrides and admit Path interface
  conversions. Preserve the archived Neo lexical Path surface in generated bootstrap
  fragments. Fix reachability analysis of nonvirtual class callvirt, exposed by
  Path becoming an Object subclass: retain its one static target and runtime null
  check. Update API reference and website. Audit other library classes and
  reproduce RuntimeTypeInfo's typed/Object inconsistency as the next bounded repair;
  provider/resource identity and general comparers remain separate decisions.

- Investigate value-type async state machines as the next author-directed priority.
  Add a reproducible Release heap/value probe: the initial heap baseline succeeds;
  struct emission exposes an import rejection and boxing at builder boundaries. Record
  by-reference startup, suspension ownership, GC and allocation measurement gates;
  keep the existing heap default. Implement the bounded ref builder projection,
  in-place startup and one retained state across suspensions, with completion cleanup.
  Add checked class field references and no-result byref value/interface methods.
  Validate ready/pending/cancelled states, unit/Result payloads and awaitless results
  under GC pressure; ready completion saves one managed object, pending counts match
  heap states. Keep broader performance claims open. Update API reference and website;
  builder APIs are transitional pending future runtime-owned suspension. Validation:
  37 runtime regressions, six Raven policy tests, twelve target matrix runs, metadata
  rejection checks, library/API snapshots and the combined website build. Raven
  target policy fix is 89a40051e; no change is integrated into Raven main.

- Add boxed Boolean Object equality and hashing alongside Int32: compare copied
  values only with the exact Boolean type, with hashes 1/0 for true/false. Preserve
  separate box identity and explicit Object base behavior. Extend the Raven sample,
  .NET comparison, runtime regressions and generated API reference; other primitive
  virtual equality/hash implementations remain outside this bounded slice. Check
  boxed reference-field tracing under allocation pressure and reclamation after
  scalar return. Record the author-directed library-wide Object/GC consistency audit
  and select Path contract alignment next. Validation: 44 runtime regressions,
  37 .NET assertions, the Raven Object sample, API reference and website build.

- Enable Raven's nullable-value declaration restriction for the development target:
  RAV0407 reports "Value types can't be declared as nullable" while nullable reference
  annotations remain valid. General Raven defaults stay compatible with .NET;
  Option remains the preferred absence model. Preserve inherited nullable metadata
  on generated record Object.Equals, with class/struct nullable Object comparisons
  and compiler rejection cases in the checked sample. Update integration, roadmap,
  on-site guidance and the development conversation record. Requires a matching
  development compiler; packaged SDKs are not refreshed by this change. The target
  record-struct generator narrows its comparison argument after an exact-type guard
  before unboxing. Validation: 117 integration compiler checks, record/Object samples,
  385 API entries and the combined website. Regeneration changes 102 input/compiler
  manifests only; runtime IL is unchanged. Follow up with typed record-class
  Equals(Record?) metadata and overload selection, preserving non-nullable record
  structs and explicit Equals declarations. Repair general Raven interface dispatch
  for top-level reference annotations and target nested-record component lookup.
  Extend the checked record sample and pinned .NET comparison; no library API changes.
  Typed-equality validation includes 61 target record checks, all three record programs,
  32 pinned .NET assertions, API/snapshot checks and the combined website build.
  Align generated record-class ==/!= nullable annotations with existing behavior,
  preserving value operands for structs and explicitly authored operators. Fix
  internal null guards to use identity, avoiding operator recursion and misleading
  custom operators in component equality/hash/display. Extend the sample and on-site
  guide. Operator validation: 66 target compiler checks, 34 pinned .NET assertions,
  all three record programs and rejection cases, API/snapshot and website checks.

- Implement the first non-generic struct/record-struct Object integration: explicit
  named-value overrides dispatch into boxed payloads with readonly protection;
  value isinst preserves exact boxes and unbox.any copies their payloads. Null
  unboxing uses NullReference and type mismatch uses the new InvalidCast Fault code.
  Extend Raven's configured record contract and importer for typed/Object/interface
  equality, hashing, display and deconstruction. Add a checked sample covering
  ordinary struct copies, box independence, default fields and string/reference
  components. Update the API reference, website, research and roadmap. General
  ValueType fallback equality, generic record components, nullable
  boxing and address-returning unbox remain unsupported. Validation: 55 compiler tests,
  47 focused runtime tests, 26 .NET baseline assertions, the compiled Raven sample,
  385 API items and the combined website. No VS Code build or release.

  Extend record components to same-compilation non-generic record structs, in both
  record classes and structs. Use typed value equality/hash without reference null
  guards and admit exact struct writes to declared instance output parameters for
  deconstruction. Add a separate Point/Rectangle/Drawing sample covering nested
  defaults, copy independence, equality, hashes and display. Keep the importer method
  limit unchanged and include both sample projects in the website download. Nullable
  struct components and arbitrary byref stores remain unsupported. Validation:
  58 focused compiler tests, both target samples, 28 pinned .NET assertions and
  combined website/API documentation checks.

  Repair default record structs with non-nullable string fields: generated methods
  guard null components, contribute zero to hashes and display empty component text,
  while preserving null in storage/deconstruction. Add a Defaults sample reproducing
  the former Utf8Encode fault, alongside null-versus-empty checks. No runtime layout,
  HashCode.Add(string) or nullable API contract changes. Validation: 59 compiler
  tests, all three target samples, 30 .NET assertions and website/API checks.

  Align Object.Equals(Object?) and ReferenceEquals(Object?, Object?) declarations
  with existing runtime null behavior. Annotate the source library and compiler/
  bootstrap references, admit literal-null core Object locals/call arguments through
  typed importer adapters, and regenerate matching library/API snapshots. Check
  null literals, nullable locals, overrides and boxed equality, while non-nullable
  Object assignments remain rejected. Runtime signatures/storage are unchanged.
  Record reference annotations as current Raven compatibility, with future metadata
  representation undecided; generated record-specific annotations remain separate.
  Validation: Object plus three record samples, non-nullable assignment rejection,
  regenerated bootstrap snapshot checks, 385 API items and the combined website.

  Clarify the author-directed absence model: prefer Option<T> for value and reference
  types; retain nullable reference annotations for Raven compatibility and existing
  null contracts. Defer nullable structs and nullable-value boxing rather than treating
  them as the next slice. Update roadmap, design record and on-site guidance; no
  runtime/compiler behavior or final metadata-format commitment changes.

- Implement bounded virtual Object equality/hash for boxed Int32: compare exact
  type and integer value, and return the stored integer hash. Preserve separate-box
  identity and explicit Object base behavior. Other boxed primitives and primitive boxed ToString
  remain unsupported; named struct overrides are covered separately. Extend the
  Raven Object sample, .NET comparison and API/website documentation. Validation:
  12 Object runtime tests, 22 pinned .NET assertions, the compiled Raven sample and
  combined website/API build. Include the sample in the local SDK workspace.

- Record the future minimal HTTP application namespace map, optional HTTPS
  dependencies, conceptual System.Web.WebApplication and longer-term time/String/
  StringBuilder needs. Extend the roadmap and HTTP plan with provisional layering,
  .NET comparisons and validation questions; no APIs or milestone priorities change.

- Extend experimental Raven record components to non-null strings and nested
  same-compilation record classes, including nullable record references.
  Preserve Equatable<Record>, use content/typed
  equality and matching hashes, and support string/reference deconstruction outputs.
  Typed importer call adapters preserve null argument positions; absent record
  components retain null through equality, hashes, display and deconstruction.
  Nullable strings/values and external record components remain unsupported;
  record structs are covered separately. The configured hash provider now requires Add(string) as well as
  Add(int). Expand the checked sample, design comparison, API guide and website. Validation:
  46 compiler tests, 32 reference-slot tests, Unicode/nested/nullable-record sample, editor
  completion and the combined site/API build. Prepare a separate matching local
  SDK snapshot, preserving the previous workspace.

- Prepare a matching local neoCLR/Raven development SDK workspace for VS Code,
  with records, Storage and Console projects and pinned compiler/server paths.
  Verify all three build/run paths and HashCode/Concurrency editor completions.
  Remove the Storage sample’s Console wildcard import to avoid the collision
  between Console.Error and the Result.Error case constructor. This is a local
  development snapshot, not a published SDK release.

- Add a bounded System.HashCode value accumulator (Add for integer/non-null string,
  ToHashCode and two-integer Combine) and opt-in Raven record-class integration.
  Integer records preserve reference identity while generating component equality,
  hashes, display and deconstruction. Other record shapes report RAVT004; generic
  hashing/comparers, boxed-value equality and runtime init-only field flags remain
  unsupported. Import recognized init-only setters and private readonly field writes
  with explicit context checks. Add a checked record sample and API/website coverage.
  Requires matching experimental Raven compiler, reference and runtime artifacts;
  default Raven/.NET record synthesis is unchanged. Validation: 40 compiler tests,
  three hash runtime tests, record/Equatable sample and shape rejection, readonly
  metadata checks, full library regeneration, 385 API items and the website build.

- Add a website namespace overview describing the current library and runtime
  support areas, with reference/guide links and explicit coverage gaps. Link it
  from the homepage and API navigation; keep Console under System and extend
  the inventory as implemented APIs introduce namespaces.

- Plan a pre-release Raven example portability pass after shared compiler fixes
  are independently verified on Raven main, released and integrated into the
  neoCLR branch. Track pinned baselines, minimal ports, failure classification and
  fixes/retests while retaining the efficient platform-specific CI split.

- Implement bounded Object.ReferenceEquals and virtual class Equals/GetHashCode,
  including default array identity, custom overrides, explicit base calls and a
  stable execution-local identity hash. Instance null receivers fault; static
  identity handles two nulls. Separate boxes retain distinct identity. String
  identity calls explicitly fault; boxed virtual equality/hash remain unsupported.
  Keep native identity imports exact and report ManagedHeap service requirements.
  Update importer static/virtual Object handling, generated library and API docs,
  and add a downloadable class sample. Record syntax is the author-selected next
  end-to-end gate: the initial acceptance probe exposed a missing comparer
  contract and HashCode dependency. The author then directed HashCode and adapted record synthesis,
  implemented in the follow-up entry above. Validation: nine equality/service tests, six identity cases, six
  display regressions, both Raven samples, 15 .NET assertions, full library
  regeneration, 378 API items and the combined website build.

### 2026-09-23

- Record System.Networking.Sockets and System.Web.Http as candidate future namespace
  boundaries, with a .NET comparison and open naming/package questions. This is a
  roadmap consideration, not an implemented API or a priority change.

- Characterize Object identity prerequisites before adding equality/hash APIs.
  Add artifact-roundtrip checks for aliases, mutation/GC, boxes, arrays, typed nulls
  and execution-local allocation IDs; expose the existing String wrapper identity
  gap in the review and website. Expand the pinned .NET baseline to 14 assertions,
  including custom equality/hash and String conversion behavior. No public equality
  method, hash algorithm or String representation change is implemented here.
  Validation: six new identity cases and four existing reference-identity tests,
  all 14 .NET baseline assertions and the combined website build passed.

- Make Object abstract with validated base construction, following author direction;
  direct Object allocation is rejected. This differs from .NET's concrete Object.
  Implement the first Object.ToString slice with a concrete-type-name fallback and
  ordinary class overrides. Preserve Object ancestry in imported application classes
  and distinguish virtual calls from explicit base calls. Rootless nominal classes
  and arrays use Object's default slot. Boxed-value and intrinsic-string virtual
  formatting remain unsupported; GetType support is unchanged. Add a checked Raven
  sample, raw dispatch regressions, refreshed API docs and an on-site source download.
  Application ToString declarations must use override; same-name hiding is rejected.
  No equality/hash implementation or erased Value migration is included.
  Validation: 21 dispatch/construction tests, the positive/negative Raven sample,
  three existing reflection/inheritance samples, matching library/API snapshots
  (375 API items) and the combined website build.

- Review Object and temporary Value storage against .NET reference/value semantics.
  Supersede older value-like Object inheritance and universal structural-equality
  proposals; record current implementation gaps and the next Object display/override
  slice. Add a pinned .NET 10 comparison program and generated reference coverage
  for Object.GetType and Value, with compiler-only Object members clearly separated.
  No Object runtime method or Value representation changes in this review.
  Validation: eight .NET baseline assertions, 32 focused runtime regressions,
  374 documented API items and the combined website build.

- Repair library composition: include StorageProvider only with the Raven
  StorageItem/File/Directory hierarchy, rather than in the legacy bootstrap library
  with static File helpers. Restore the legacy Console suite (11 tests); preserve
  the composed library's provider contract. Remove stale completed-release gates
  from the roadmap and record the upcoming Object/Value consistency review.
  Validation: 11 legacy Console tests, three composed-library Console stream tests,
  and matching generated-library/API snapshots.

- Keep Console a static class and add In/Out/Error, standard byte-stream factories,
  bounded UTF-8 ReadLine, Write and blank WriteLine. Add TextWriter and StreamWriter;
  extend TextReader with ReadLine (implementers must supply the new member).
  Calls remain synchronous; lines use LF/CRLF, with lone CR preserved as data.
  Add opt-in host byte-write/flush hooks and byte-exact Execution.stdout/stderr
  capture without a live host. Existing hosts retain WriteLine; new output hooks
  default to Unsupported until implemented. Refresh reference metadata, generated
  library and API docs. Adapt importer dispatch and Console's legacy Void returns.
  Show tested Result propagation and Option bindings on the Console feature page
  and dedicated Raven error-handling section; reserve match for useful case handling.
  Validation: three native boundary tests; greeting input cases; reader/writer
  ownership, partial-write and range contracts; both propagation/binding samples;
  dedicated/pooled worker output capture and budget regression;
  library/API snapshots and combined website build. The older raw-System Console
  suite remains blocked by an unresolved StorageItem dependency in that library.

- Characterize queue ownership in the isolated worker adapter with a pending await
  entered from an explicit caller queue: the body resumes through the producer's
  queue, while result observation waits for the caller queue to drain. Keep this as
  evidence of current behavior, not a future affinity guarantee. Record the author's
  direction to evolve TaskQueue and the scheduling model when concrete needs arise,
  including runtime suspension, without committing to retention or replacement.
  Update the roadmap, Task contracts, on-site explanation and source download;
  runtime and public APIs are unchanged. Record two unreduced callback-capture
  integration failures for independent investigation.
  Validation: exact cross-queue output, four worker cancellation/sibling outcomes
  with GC and zero final live objects, UserFault and bootstrap boundary checks,
  API snapshot, website tests/build and matching ten-file source archive.

- Connect acknowledged per-job cancellation to Promise.Cancel in an isolated Raven
  worker adapter. Admit the two existing services in bootstrap RuntimeServices only;
  normal Thread/Task APIs and installed worker implementation stay unchanged. Add
  dedicated/pooled awaiting consumers, sibling and GC checks, genuine UserFault and
  forbidden-service fixtures, and on-site adapter documentation/source download.
  Record the generated-field collision with an async parameter named state as a
  deferred integration issue; the tested sample uses destination. Refresh the matching
  normal-core API snapshot without adding managed public APIs.
  Validation: four cancellation/sibling outcomes with GC and zero final live objects,
  UserFault and normal-core boundary checks, original delayed-copy/busy-queue consumers,
  runtime/API snapshots, matching source archive and combined website checks.

- Give isolated jobs independent cooperative cancellation tokens. Add experimental
  raw RequestWorkerCancellation and JoinWorkerResult runtime services: requests retain
  pending roots, and acknowledged job cancellation returns an erased Void outcome for
  adapters. Cached success and unrelated Faults are preserved; host cancellation and
  legacy JoinWorker behavior remain terminal. Release pooled per-job state before
  result publication and retain all-job cancellation/join on invocation teardown.
  Ordinary Raven Thread/Task/Storage APIs are unchanged; document the raw services
  on-site and keep guest Task-adapter integration as the next slice.
  Validation: 10 worker-registry tests, 16 worker integration tests including both
  execution backends and cancellation/GC, service reachability, normal build and
  website tests/build.

- Connect the pending-read experiment to real isolated host workers and the existing
  notification adapter. Defer requested cancellation until joined producer completion,
  then discard the owned payload; preserve completed results against late requests.
  No public Storage/runtime API or worker interruption is added. Reuse the delayed-copy
  harness for the new consumer; record the private-field async bridge limitation and
  add on-site explanation and consumer-source download.
  Validation: five host-backed outcomes with actual GC and zero final live objects,
  original delayed-copy/busy-queue consumers, website tests/build and source archive.

- Add a pending-read contract experiment using real Task/Promise, Raven await,
  TaskQueue and managed GC. Six ordered scenarios distinguish cancellation requests
  from terminal acknowledgement, preserve bytes on failure/cancellation and reject
  duplicate terminal writes, releases and notifications. The buffer stays private
  while pending; producer callbacks and resource release are modeled, with no new
  public async Storage API. Add on-site documentation and a source download.
  Validation: all six guest scenarios, actual collections and zero final live objects;
  website tests/build and source archive verification.

- Add a bounded File Transformer sample connecting platform Storage and System.IO
  to the existing experimental JSON document consumer. Validate and serialize before
  exclusive output creation; preserve input and existing destinations, close streams
  and document partial-output risk after write/flush failure. Add an on-site walkthrough
  and downloadable source bundle. JSON remains sample code and I/O is synchronous.
  Validation: nine isolated file cases, unchanged source/existing-output checks,
  independent JSON output parsing, matching source archive, website tests and build.

- Complete the synchronous Storage POC with a platform-only file-access sample,
  downloadable on-site walkthrough and API reference. Move development byte streams
  from System.Streams to System.IO (recompile consumers and update imports). Add
  TextReader/StreamReader with bounded strict UTF-8, named errors and explicit input
  ownership; add optional SeekableStream with absolute byte positions on file input.
  Fix strict bridge admission of reader locals and constructor Boolean conversions.
  Keep Task-returning Storage as future exploration pending suspension, scheduling
  and cancellation contracts; no asynchronous I/O is claimed.
  Validation: standalone POC, disk/memory reader and provider contracts, nine native
  file-resource tests, strict interface probe, runtime/API snapshots, 332 documented
  API items and combined website build.

- Integrate FileSystem as the host StorageProvider with internal File/Directory
  implementations. Add relative Directory.GetItem/GetDirectory and bounded
  GetItems snapshots of mixed StorageItem interfaces. Bounds fail explicitly
  rather than truncating; StorageLookupError gains InvalidRange and LimitExceeded.
  Enumeration is synchronous and non-atomic with child lookup. Update the disk/memory
  sample to exercise platform host resolution and refresh API/runtime documentation.
  Validation: eight native file-resource tests, full SDK product/contracts and
  negative callers, runtime/API snapshots, 262 documented API items and the
  combined website pass.

- Consolidate StorageProvider around GetItem/GetFile/GetDirectory with interface
  results. Remove the temporary StorageLookup contract and provider-level byte
  methods; byte routing belongs to concrete File implementations. Development
  consumers must migrate lookup to StorageProvider and keep byte helper contracts
  inside implementations. Add generic disk/memory lookup and resolution-only provider
  coverage; refresh runtime and API reference artifacts. Record the complete POC
  scope, including System.IO alignment, text readers and a seekability case.
  Validation: full SDK sample/contract suite, strict interface import, bootstrap
  snapshot, 243 documented API items and combined website pass.

- Add synchronous StorageLookup.GetDirectory(Path), returning the public Directory
  interface with typed missing/wrong-kind errors. The disk/memory product obtains
  its root through provider lookup. Disk validates native kind; the bounded memory
  provider exposes only its root and does not infer directories from file keys.
  Development migration: StorageLookup implementers must add GetDirectory; byte-only
  providers are unchanged. Document the contract and refresh API/runtime snapshots.
  Provider consolidation, generic item lookup and enumeration remain follow-ups.
  Validation: SDK disk/memory product and contract/negative suite, strict interface
  import, runtime/API snapshots and combined website pass. Reaffirm the bounded POC
  checkpoint and record TextReader/StreamReader as planned follow-up work and
  seekability as an open design question, not implemented APIs.

- Implement StorageItem as a closed root over provider-implemented File and Directory
  interfaces. Provider result contracts expose interfaces; concrete classes remain
  implementation details. Move descriptor classes into the sample providers; test common
  Name/Path access over mixed items and reject unrelated branches in Raven and the
  strict importer. Raw neoIL does not enforce the closed-hierarchy metadata.
  Development migration: construct provider implementations rather than File/Directory;
  native static text helpers move to FileText because interface static methods are
  not supported. Legacy raw File aliases remain. Update API docs, generated runtime
  and reference snapshots, website and roadmap. Provider resolution and enumeration
  remain next; StorageLookup/FileAt/CreateNew are still transitional POC contracts.
  Validation: disk/memory product and contract/negative callers, mixed StorageItem
  arrays, Raven and raw-CIL closure rejection, strict interface/foundation and static
  file import checks, ten native file/Path regressions, runtime/API snapshots,
  244 documented API items and the combined website pass.

- Record the author's selected Storage model: StorageItem is a closed interface
  hierarchy over provider-implemented File and Directory interfaces; providers
  resolve paths and GetItems enumerates StorageItem values. Correct proposal class
  examples, update roadmap priority and distinguish the target from current concrete
  descriptors on the website/API guide. Interface migration and enumeration remain
  planned; no new runtime API is implemented by this direction update.
  Record queryable metadata as a future extension independent of native filesystem
  attributes, with System.IO/WinRT comparisons and query shape, availability and
  freshness left open. The interface/provider migration remains the next priority.

- Integrate development System.Storage.File and Directory descriptors and the optional
  StorageLookup provider capability. File retains provider/Path, derives Name and
  opens directional streams; Directory constructs child addresses and explicitly
  looks up files. Construction/properties perform no storage queries. Preserve the
  existing static string-based File.ReadAllText/WriteAllText helpers. The sample now
  imports platform descriptors; text conveniences stay on fixture providers.
  Directory.FileAt changes from the sample FileReadError to StorageLookupError.
  Add complete descriptor/lookup and legacy text-error API reference coverage, with
  an exact manual WriteAllText entry for DocFX's unit-result limitation. Regenerate
  runtime/reference snapshots and update the website and roadmap.
  Validation: disk/memory product and contract checks (including construction without
  provider calls), strict interface/foundation checks, ten native file/Path artifact
  regressions, runtime/API snapshots, 245 documented API items and the combined
  website pass. Concrete host-provider integration is next.

- Integrate development System.Streams.InputStream and OutputStream interfaces.
  File streams implement them directly; the disk/memory sample imports the platform
  contracts and drops forwarding disk adapters. Preserve blocking partial transfers,
  typed errors and explicit close. Add generated API coverage, a manual OutputStream.Flush
  entry for DocFX's unit-result limitation, and opposite-direction negative checks.
  Custom implementations require the matching core/runtime and a development Raven
  compiler with the imported-array and configured-unit identity corrections;
  Preview 9 is unchanged.
  Record the author's minimal WinRT-informed Storage direction: useful address/name
  properties and explicit operations first, richer metadata only when needed.
  Simplify the sample File constructor to (provider, path), deriving Name from Path;
  remove the redundant StorageProvider.FileAt factory. Directory.FileAt remains an
  address operation. Reject independent names; preserve existing lookup and I/O.
  Integrate the minimal System.Storage.StorageProvider byte contract with OpenRead
  and CreateNew. Sample lookup/text conveniences extend it; file byte operations
  dispatch through the platform interface for both disk and memory. Text encoding,
  descriptor lookup and richer metadata are not mandatory provider methods.
  File/Directory integration follows in the entry above. Validation:
  disk/memory SDK contracts and negative callers, interface import checks, runtime
  snapshot, 151 API summaries and the combined website pass.

- Add development System.Storage.Metadata.GetKind(string), EntryKind and
  StorageLookupError over the existing native metadata service, with generated API
  reference coverage. Add typed GetFile lookup to the disk/memory provider sample;
  distinguish missing and wrong-kind outcomes without opening file contents.
  Extend experimental Directory.GetFile with relative Path lookup, including nested
  paths. Absolute paths remain valid for providers but are rejected by a directory;
  its string overload still accepts one direct child. Document this provisional
  distinction, flat-memory-directory limits and tested .NET comparison.
  Lookup is an observation, not stable identity or a guarantee of later access.
  Record Path as Storage-specific rather than mandatory throughout the platform;
  host metadata and file-stream APIs retain string parameters. Matching regenerated
  development runtime/reference artifacts are required for these additions.

- Replace the experimental MemorySlotStorage with MemoryStorage: text helpers and
  streams now share byte contents per address. Support up to eight addresses with 64 KiB
  current payloads, strict UTF-8 reads and cross-API exclusive creation. Keep old
  payloads alive for existing streams after text replacement; this provisional
  memory policy is not a disk/portable identity guarantee. Extend the disk/memory
  sample and contract checks, and update the on-site API reference and roadmap.

- Add isolated host Storage lookup probes comparing .NET FileInfo caching with
  metadata observations, path replacement, open-handle identity and provider roots.
  Record typed metadata lookup as a provisional direction, not an implemented
  GetFile API. Make coherent memory-provider text/byte contents the next prerequisite.

- Integrate the tested immutable Path value into the development System.Storage
  library, with Parse returning Result<Path, InvalidPathError>, private construction,
  read-only text and lexical equality. Preserve existing Combine/GetFileName string
  helpers and add generated reference for all Path members. The Storage sample now
  imports the platform type; its Path.rvn retains only a fixture helper. Regenerate
  matching runtime/reference artifacts. Logical grammar and provider APIs remain
  provisional; Path is not required by unrelated string-taking APIs. Prioritize
  integrating the remaining working Storage slice over expanding isolated experiments.
  Record future Unix/Windows parsing and normalization, and an analogous Uri value
  direction and Path-taking helper overloads, as plans rather than implemented
  behavior; keep immediate integration scoped to a minimal Storage POC.

- Add host-visible FaultCode classifications, including StackOverflow for the
  interpreter frame limit, arithmetic/memory limits and host cancellation. Explicit
  guest faults always use UserFault; System.Fault still accepts only a message.
  Keep a RuntimeError fallback for diagnostics not yet classified. Preserve codes
  with stack traces and expose debugger fault_code; CLI messages include the code.
  Document all codes and the Rust host API on-site. Hosts constructing Fault with
  struct literals must provide code; formatted CLI diagnostics also change.

- Add development System.Streams FileInputStream/FileOutputStream APIs with typed
  StreamError results, bounded caller-buffer transfers, exclusive file creation,
  flush and explicit close. These calls block; they do not implement async I/O or
  durable flush. Connect the Raven provider sample to byte streams on disk and in
  memory, including deliberate short transfers and actual UTF-8 disk verification.
  Keep native IDs unique across invocations so retained wrappers cannot access a
  later invocation's files. Add browsable API reference and source downloads.
  Keep provider-owned child-address construction and the earlier bounded whole-text
  workflow as exploratory comparison cases. Following the author's clarification,
  align Storage and explore a Path value object next; application-owned provider contracts remain provisional.

- Require on-site reference coverage for public API changes, including namespace
  navigation and documented renderer exclusions. Add System.Concurrency Thread and
  ThreadPool reference pages, a namespace browsing index, and an explicit backlog
  for older APIs that still lack member reference coverage.

- Select the post-release concurrency, Storage and file-stream checkpoint. Keep
  Task/Promise in System.Tasks; move explicit thread APIs to System.Concurrency.
  Add retained Thread instances with one-shot Start, IsStarted and a stable Task,
  plus static Thread.Run; successful completion waits for native thread teardown.
  Update bindings, samples, generated library, API docs and the development website
  section. Preserve the website's tested Preview 9 worker example. This breaks the
  old namespace and static Thread.Start spelling; rebuild with matching artifacts.
  The first file Stream APIs and disk read/write acceptance app are now implemented
  as described above; Storage alignment remains pending.
  Task.Run will need completion-only and generic
  value-returning overloads; keep it pending suspension/scheduling design rather
  than publishing a string-only worker facade.

- Skip the runtime matrix for documentation, website and API-reference-only pushes
  and pull requests; keep executable experiments covered and provide manual runs.
  Release tags no longer automatically repeat the validated candidate matrix.
  Document the trigger rules and retain the broader CI redesign for the next release.

- Record publication of Preview 9 at `834028c` with six passing platform/toolchain
  jobs and extracted macOS arm64 package/editor evidence; mark the async checkpoint
  complete and resume foundational Streams, Storage and Encoding priorities.
- Align website, setup and API reference with Preview 9. Reduce the main menu to
  five consistent destinations, remove Experimental from the API header/title and
  add logo spacing. Make DocFX HTML and client-side navigation links work under a
  GitHub Pages project path, with regression coverage. Publish the combined site and
  `/docs/` from `5bffbca` through the manual Pages workflow; verify the public pages.
- Plan next-release CI efficiency: run shared validation once and isolate host-specific
  checks instead of repeating all samples per platform. Workflows are unchanged.


## 0.1.0-preview.9

Prepared candidate; publication date and final evidence belong to the release manifest.
The dated entries below record development, including explicitly labelled future plans.

### 2026-09-23

- Prepare version 0.1.0-preview.9 for the async/Tasks release, with matching Cargo
  metadata and release/migration guidance. Source and package validation of this
  versioned candidate must pass before publication; earlier candidate results remain
  separately attributed. Retain a fresh Unreleased section for subsequent work.

- Continue async release preparation with a fresh e9bb28a evaluator bundle, eight
  passing Workbench cases, all six packaged Task probes, dependency/hash checks, a runtime-only execution check
  and interactive VSIX build/run/type-hover evidence in a separate profile. Start exact-commit local source checks and six-job platform/toolchain
  CI, without claiming results before completion. Draft release notes and correct
  README's stale threading/async limitation while retaining published Preview 8 links.

- Make the low-level worker-cancellation sample self-contained by declaring its
  worker service imports in the neoIL source. The host example now uses that same
  source directly. This fixes the full-suite standalone sample assembly failure;
  verifier assembly diagnostics now identify the failing sample path. All nine
  verifier checks pass on stable Rust and Rust 1.85, and the host cancellation
  example passes. Preserve both earlier full-source failures separately; neither
  constitutes a passed release gate.

- Record the author-selected post-release System.Concurrency namespace direction
  and proposed Thread.Run versus retained Thread/Start/Task API shapes. Document
  optional thread packaging, platform limits, .NET comparisons and validation needs;
  update website future direction. Clarify abstraction-first concurrency, separating
  Task completion, possible portable worker execution and target-specific Thread
  capabilities. Record the subsequent Task.Run-style proposal for general work with
  platform-selected concurrent execution. Capture the author's confirmation of Task's
  general submission role and Thread's explicit, optional platform capability under
  System.Concurrency; Web Workers are a possible Task backend for WebAssembly.
  Exact contracts remain open. Current release APIs remain unchanged.

- Correct two stale release samples after the Error-wrapper removal and Storage
  namespace migration; preserve the case-payload and error-carrier regression intent.
  Validate all 84 saved-project outcomes against the rebuilt runtime and clean SDK payload.
  Document sidecar-free archive creation for the known macOS packaging issue,
  including staged-payload hash comparison and fresh extraction requirements.
  Preserve follow-up archive hashes, eight Workbench results, 84 saved-project
  outcomes and focused extracted-fixture checks in a separate readiness record.
  Extend migration review through the subsequent documentation changes, explicitly
  keeping proposed concurrency APIs outside this release's migration instructions.

- Preserve Rust 1.85 compatibility in worker notification dispatch by replacing
  unsupported let-chain syntax with equivalent nested conditions. The source-archive
  minimum-version check exposed the failure; the supported minimum remains unchanged.
  Validate all targets on Rust 1.85, strict stable Clippy and 14 worker regressions.
- Draft migration guidance from Preview 8 and intermediate Task builds. Reaffirm
  that this changelog continues permanently after the async release, with frozen
  published entries and a fresh Unreleased section for later development.

- Add a DocFX development API reference at `/docs/`, with an API overview, authored
  XML descriptions for the initial Task surface, and metadata-generated signatures.
  Document three unsupported callback signatures separately in Raven notation.
  Integrate the reference into website navigation, validation and the Pages artifact;
  publication remains manual. Keep this release’s documentation scope to a useful
  overview and main APIs, with exhaustive coverage deferred.

- Prepare async preview packaging: include the current Task probes in the evaluator
  bundle, make their helper/sample paths work after extraction, replace the obsolete
  explicit-worker-queue expectation and add an eight-sample saved-project Async
  Workbench check with a JSON report. Document the packaged commands; release
  certification remains separate from these focused checks. Fix three pre-existing
  rustfmt discrepancies and strict Clippy findings (redundant match guards and
  test idioms) found by the release-readiness pass; no runtime behavior changes.
  Persist toolchain roots in the Workbench project so Raven’s separate project load
  resolves the extracted bundle; record passing local package probes and remaining gates.
  Verify the repackaged eight-sample Workbench, all 22 MSBuild cases, Task editor
  completion, full library regeneration and bundle/SDK notices; preserve revisions,
  hashes and case results in a local-readiness record. Final release gates remain open.

- Make the author-approved async/Tasks preview checkpoint the immediate
  priority: stabilize the existing completion/await/composition and isolated-worker surface, require a
  fresh evaluator bundle and exact-candidate release gates, and leave HTTP and the
  notification adapter outside the supported release scope. Record the author’s
  release-timing question, update website direction and correct stale Task contract
  descriptions. Prioritize Streams, Storage and Encoding before networking afterward.
  Mark the website header Experimental across all pages and describe neoCLR as an
  experimental application platform, including APIs, language integration and
  development tools. No release version/date or publication is selected.

- Revise the website toward technical project documentation: current capabilities,
  limits, .NET tradeoffs and the active foundations-to-HTTP roadmap. Add a project
  overview with background, goals and contribution paths; keep general information
  on-site and repository design records optional. Preserve tested sample sources
  and distinguish published behavior, development experiments and proposals. Validate
  all 13 pages, site tests and responsive overview rendering.

- Add bounded JSON string-message and sensor-report document experiments using
  existing text primitives. Validate Unicode escapes, read/write all six value
  kinds, preserve number spellings, check Int32 conversion and reject duplicate
  decoded keys; bound bytes, nesting and value counts. Compare behavior with .NET
  10 and check construction, round trips and writer failures. Record importer
  limitations and advance the roadmap to delayed guest lifetimes. These are
  application-local experiments, not a public JSON library.

- Add an application-local UTF-8 chunk decoder experiment reusing Byte Copy's
  managed arrays and short reads. Demonstrate scalar boundaries versus graphemes,
  strict finalization, caller-buffer reuse and synchronous GC retention. Compare
  41 valid/malformed/split-input cases with strict .NET 10 decoding; keep public
  Encoding contracts, efficient output buffering and pending native I/O open.
  Record partial S2 evidence and link the experiment from the Strings feature page.

- Add a Raven Byte Copy experiment over existing managed arrays: validate ranges
  before mutation, preserve overlapping copies and demonstrate partial reads/writes
  with a greeting transfer. Validate 1,024 range/alias cases, extreme/empty ranges,
  output failure and synchronous array retention through guest GC. Keep public
  stream/buffer contracts and pending native I/O unselected; record the deferred
  compiler fallthrough-emission observation and partial S1 roadmap evidence.
- Add a dedicated Arrays feature page comparing neoCLR with .NET arrays, including
  generic shape, invariance, aliasing, bounds and collection capabilities. Include
  a tested downloadable readings/snapshot sample and exact output; link from the
  homepage and Collections. Validate 12 website pages, website tests, the new sample
  and 32 existing runtime array tests. No website publication is implied.

- Start M1/S0 with an isolated host-side external-completion experiment. Seven cases
  cover callback progress, cancellation acknowledgement, completion ordering and
  teardown. Add eight test-only delayed Byte Copy checks against the real managed
  heap, array slots and delegate receiver tracing: pending-to-ready root handoff,
  collection/reclamation, invalid-input rejection and terminal cleanup. Record
  owned-byte delivery as a candidate. Add experimental worker completion notification
  to the real VM, rooting callbacks at both GC paths and posting ready outcomes to
  the default TaskQueue. Validate a Raven await/copy consumer using an isolated
  worker-library adapter, failure/cancellation teardown and one-shot delivery. Poll
  ready notifications at default-queue callback returns so self-reposting guest work
  no longer requires queue quiescence; verify Raven and direct-IL consumers and
  explicit-queue isolation. Bound successful worker result text and captured output
  with a shared per-worker logical-byte quota (default 1 MiB); reject overflow before
  capturing a line or publishing success, preserving unavailable console input.
  Cover UTF-8, empty lines and notification failure delivery. Larger worker outputs
  now fault unless an embedder raises `Limits.worker_result_bytes`; exhaustive Rust
  Limits literals need the new field. This is not a total host-memory bound.
  Poll host cancellation after worker waits and between output writes; a request
  observed during delivery now skips remaining lines and stops before returning the
  joined value. Keep already written output. Add a runnable host/guest-IL sample,
  controlled notification orderings and teardown acknowledgement checks; guest
  operation cancellation remains unimplemented.
  Normal Thread/ThreadPool implementations retain queued
  joins; preemption, per-operation cancellation, bounded native I/O ownership and
  real I/O remain open.

- Prioritize an HTTP client/server application POC in the roadmap, with smaller
  stream, encoding, JSON and TCP cases, explicit acceptance criteria and complete
  proposal triage. Record external I/O progress, cancellation, buffer ownership and
  cleanup as urgent experiments; distinguish planned APIs from current behavior.
  Align planning entry points and the website proposal overview; no runtime APIs
  or release commitments are added by this documentation change. Add a unified
  platform roadmap with themed sample products: HTTP apps, a file catalog, a
  download queue, a time-aware report, an assembly explorer and a portable sample
  pack. Make that roadmap the default authority for work in AGENTS.md, subordinate
  to explicit author directions. Keep post-POC milestones and separate research
  products provisional. Refine M1 delivery to memory copy, text/JSON transformation,
  delayed guest-lifetime checks and a bounded file transformer before TCP/HTTP;
  retain HTTP as the first major application destination and allow earlier APIs
  to evolve as subsequent samples expose gaps.

- Add a development Tasks feature page with tested, downloadable worker, Promise,
  cancellation, MapResult and awaited-propagation examples. Link it from the
  homepage and proposals, distinguishing current behavior from future scheduling
  and cancellation-token work. Add a dedicated homepage Tasks box with a tested
  async worker excerpt and a direct feature-page link. Validate 12 pages and website
  tests; retain the five locally installed .rvnproj build/run checks and Task editor
  completion evidence. Prepare a
  matching local VS Code development workspace without changing release selection.

- Integrate Raven's propagation-temporary lifetime fix for `(await input)?`, with
  immediate and resumed Ok, Error and cancellation regressions. Preserve current
  postfix-first precedence; record ergonomic shorthand as future design work.
  Validate eight target propagation cases, 40 focused Raven target-branch tests
  and 21 independent Raven main tests; integrate only the general lowering fix.

- Add the explicit Task<Result<T,E>>.MapResult extension: queue Ok transformations,
  preserve Error payloads and propagate cancellation without invoking the mapper.
  Ordinary Task.Map still handles the complete value. Include a runnable sample
  and six passing outcome, dispatcher, GC and fault checks; validate 272 metadata
  signatures and the generated runtime snapshot.

- Add invocation-local TaskQueue.Default and Promise<T>() using the active queue
  or the default. Async functions and isolated workers no longer require queue setup.
  The runtime dispatches default-queue callbacks automatically before invocation
  return, preserving the entry result and retaining work through GC. Explicit queues
  remain caller-driven; external I/O completion and a public scheduler are future work.
  Validate 11 automatic-dispatch source scenarios, async regressions, ten runtime
  access/worker checks, metadata, snapshots and website checks.

- Add opt-in Raven cancelled-await propagation for named async functions, both
  immediate and resumed, through provisional IsCancelled/SetCancelled hooks.
  Nested calls cancel without fabricating a value; Result remains an ordinary
  payload. Await inside for loops is diagnosed pending suspension-aware iterator
  cleanup. Rebuild the compiler, reference library and callers together. Validate
  18 neoCLR scenarios, 31 focused Raven tests, metadata signatures, runtime
  regressions, bootstrap snapshots and website checks.
- Correct the development status after 606a597: Map and Then composition is
  implemented and tested; cancellation tokens remain outstanding.

### 2026-09-21

- Add the core development Task model: normal TaskState enum, TaskOutcome<T> union,
  and Task.State/Outcome. Rename TaskCompletionSource<T> to Promise<T> and
  TrySetResult to Complete; add Cancel with first-terminal-transition semantics and
  queued observers. Result.Error remains an ordinary completed payload. Rebuild
  references, library and callers together. Tokens, Map/Then and automatic await
  cancellation propagation remain outstanding; inspect cancellation through Outcome.
  Validate 24 source contract scenarios, 10 async and four worker regressions, nine
  direct runtime checks, 263 metadata checks and the website.

- Record Promise-style Task composition and continuation as API direction, with
  names chosen for neoCLR rather than .NET parity. Keep Result failure independent;
  the updated Task model proposes State/Outcome, distinct cancellation and Map/Then.
  Record runtime-first implementation slices and Raven lowering gaps; distinguish
  cancellation requests from terminal outcomes. Update the proposals overview
  without claiming implementation of cancellation or composition.

- Select provisional heap async states and exception-free lowering in the neoCLR
  project props, matching the development compiler and language server. Keep the
  installed development build separate from published Preview 8 artifacts. Verify
  the async, worker and storage projects through MSBuild and the language server.

- Rename the development System.IO namespace and source folders to System.Storage,
  including File, Path and file errors; move ConsoleReadError to System. Update
  references, samples and current documentation. Rebuild callers and library
  artifacts together; local synchronous
  file behavior is unchanged and storage providers remain future work. Validate
  27 runtime checks, 257 signature checks, website generation and LSP completions.

- Add an isolated-worker PoC in System.Threading: Thread.Start and ThreadPool.Queue
  exchange text through static callbacks and return Task<string> on the caller queue.
  Use separate interpreter heaps, a two-thread pool, bounded submissions and joined
  invocation cleanup. Queue pumping may block for results; guest objects and Task
  state are not shared across threads.

- Execute provisional compiler-generated async/await in the Raven profile using
  System.Tasks.Task<T>, TaskCompletionSource<T> and explicit TaskQueue.Run/Drain.
  Rebuild references and replace System.Threading.Tasks imports. Result remains an
  ordinary payload; exception capture and threading are not introduced. Validate
  ten async scenarios, fifteen completion scenarios and 47 focused runtime checks.
  This development behavior is not part of the published Preview 8 artifacts.

- Permit constructors to initialize fields containing erased values without inventing
  a default payload. Unassigned fields remain unreadable and constructor publication
  still requires initialization; explicit invalid defaults remain rejected.

### 2026-09-19

- Record the decision to defer Thread/ThreadPool work and finish compiler-generated
  async integration first. This is implementation direction, not shipped async
  support or a thread-safety guarantee. Independently reproduce and fix Raven's
  struct-field receiver addressing on main; add a provisional heap state-machine
  option only on Raven's neoCLR branch. Modern .NET compiler tests pass, while
  neoCLR builder/importer integration remains in progress.

- Add a provisional Raven-profile Task<T>/TaskCompletionSource<T> completion PoC
  with an explicit TaskQueue, ordinary generic payloads (including unit/Result),
  first-completion semantics and queued continuations. Preserve private storage
  and internal producer/consumer implementation access in guest metadata. Validate
  source scenarios, direct-IL access boundaries and GC retention; document the API,
  scheduling limits and outstanding compiler-generated async work. This development
  API is not in Preview 8. Update the proposals page and runtime API audit.

- Retire the legacy System.Error message wrapper, its native helpers, intrinsic
  runtime/host value and type, ErrorValues service and `error` instruction. Use
  ordinary strings or domain-specific Result payloads; typed error unions remain.
  This breaks old message-wrapper metadata/host APIs: rebuild references, library
  and callers together. Published Preview 8 artifacts are unchanged. Simplify the
  outcome samples to imported `Error(...)`, document the convention and migration,
  refresh the library inventory and update the outcomes feature page.

- Start the provisional async mechanism with an executable Raven completion model:
  validate pending heap-owned state across GC, multiple consumers, queued callback
  ordering, duplicate completion, Result/unit payloads and terminal fault boundaries
  in 10 scenarios. Record current Raven builder/exception-lowering gaps and the next
  compiler-integration steps. Distinguish public Task semantics from replaceable
  state-machine, builder and awaiter contracts. Update the proposals overview; this
  experiment does not introduce a public Task<T> or generated async/await support.
  Specify exception-free target lowering: Result errors complete tasks as values,
  Faults remain terminal, and Raven must omit its generated catch wrapper rather
  than merely omit SetException. Record cleanup and .NET regression requirements,
  with the current CreateMoveNextBody entry point identified for integration.
  Clarify that Task lowering is uniform in its payload T: Result is an ordinary
  type, with no special failure/completion path or Result-aware scheduling.
  Add Raven's provisional opt-out for generated async exception capture on its
  neoclr branch, with compiler execution tests for propagation before/after pending
  awaits and unrelated union payloads. Document the API-only configuration,
  unchanged .NET default and remaining target integration and generic-unit gaps.
  Validate 61 focused and 119 feature-selected compiler tests (overlapping sets),
  and use ordinary ? in the manual neoCLR continuation probe's application body.
  All 10 neoCLR probe scenarios pass, including retained state through 46 GC cycles.
  Refresh and validate the website proposals overview. Independently fix generic
  unit-task return binding and completion in Raven main, then cherry-pick that fix
  to neoclr; 145 async/resource tests pass on .NET. Guest builder/Task integration
  and the configured System.Void ABI remain separate work.

- Port 25 Option/Result operator overloads from Raven.Core to the development
  library, including transformations, recovery, branch actions, conversions and
  nested Option flattening. Use Filter and ToIterable for neoCLR's vocabulary;
  throwing/default/context-error helpers remain deferred. Implement both iterable
  and outcome operators with Raven extension declarations and validate their
  extension metadata against bootstrap references. Add branch/callback, signature,
  editor and saved-project coverage; document compatibility and allocation costs.
  Rebuild callers, references and System together; published Preview 8 is unchanged.
  Validate 9 outcome scenarios, 251 signatures, 86 editor sections, all 80 library
  slices, 58 iterable regression checks and the MSBuild example. Prefer inferred
  callback types in samples and record the remaining compiler limitations. Include
  the outcome verifier and the query
  verifier's helper in future bundles.
  Simplify redundant lambda signatures in query-terminal, deferred-query and
  delegate samples; retain the generic Map callback signature required by current
  inference. All three edited examples retain their checked execution output.
  Add the website's complete Option/Result operator table, a tested composition
  example and downloadable source/output. Link the .NET iterable mapping and
  distinguish development additions from Preview 8. Validate all 10 pages,
  cross-page links and highlighting, and review the operator table in the browser.
  Record natural, minimally annotated Raven as a project-wide convention for all
  hand-authored code, including runtime implementations, tooling, tests and examples.
  Make the scope explicit in AGENTS.md; retain annotations required for compilation
  or a clear contract and the documented bootstrap/test exceptions.

- Document imported union case patterns in the Raven conventions: use Some, None,
  Ok and Error without a leading dot when their case namespaces are imported.
  Update the convention examples, including nested patterns.

- Add the basic development Iterable operators before Task work: Any, All, Count,
  seeded Fold, Take, Skip, Concat and FlatMap. Preserve short-circuiting, empty-input
  outcomes, ordered lazy composition and normal iterator disposal. Count faults on
  Int32 overflow. Rebuild references and the System library together; Preview 8
  packages remain unchanged. Add executable samples, signature and editor coverage,
  and .NET/Rust contract comparisons. Support nested open collection arguments in
  the neoCLR library import bridge without changing Raven's compiler. Validate
  58 query outcomes, 174 signatures, 86 editor sections, 5 compiler-path checks,
  24 focused runtime tests and exact reproduction of all 77 library slices.
  Add a website guide with the tested basic sample and a .NET-to-neoCLR operator
  table, including outcome differences and unsupported operators. Keep development
  additions distinct from published Preview 8. Validate all 10 pages and review
  the mapping table in the browser.

- Record the general Raven SDK AppleDouble packaging candidate and the validated
  sidecar-free Preview 8 archive workaround; no upstream script fix is claimed.
- Rename the development query API after Preview 8: Where becomes Filter and
  Select becomes Map, with initial capitals and no legacy aliases. Rebuild callers
  and target references together; deferred execution and iterator behavior are retained.
  Update samples, metadata bindings, editor checks and the website migration guide.
  Validate 83 saved-project outcomes, 33 query outcomes, 142 signatures, 86 editor
  sections, 5 compiler-path checks, 15 application checks, 24 focused runtime tests
  and reproducible regeneration of all 77 library slices.
  Prefer converged terminology while retaining .NET terms where conventional or
  clearer; do not mechanically choose Fold/Reduce or Drop/Skip. Preview 8 still
  exposes the existing names.
- Record Task and async state-machine contracts as a post-Preview 8 foundation:
  upcoming APIs need the completion contract before runtime suspension arrives.
  Keep continuation scheduling and logical context flow as explicit open design work,
  including the goal of avoiding routine ConfigureAwait-style boilerplate.
- Select Preview 8 and Raven 0.1.12-neoclr.15 as the release candidate; prepare
  notes, migration guidance and the validation checklist. Repair the Windows
  calendar test's newline-dependent mutation and strict-Clippy test issues;
  document the independent parameter tables at the Introspection helper boundary.
  Select the release scope/date and keep actual publication evidence in the manifest;
  exact-candidate validation passed and all eight published assets were hash-verified.
  Refresh website feature status and installation instructions for Preview 8. Generate the advanced
  runner fixture from shared props with inline contracts, so packaged verification
  can copy it to a temporary directory without breaking relative imports. Update
  the full editor matrix to expect Sequence members from Introspection queries;
  align standalone array/match probes with current contracts. Keep the reduced
  probe core compilable by omitting collection-returning String members when its
  collection profile is not selected; the full runtime reference API is unchanged.
  Compare the clock sample’s six local components against the host time interval,
  including DST folds, rather than expecting the retired offset output.

## 0.1.0-preview.8 — 2026-09-19

Raven-authored System.Runtime, unified Introspection and grapheme text. See the
[release notes](docs/preview-8-release-notes.md) for the resulting API and migration.
The dated development history below preserves intermediate choices superseded by
later entries; it is not a list of simultaneously supported contracts. The release
manifest records the exact published candidate and its validation evidence.

### 2026-09-19

- Prepare Preview 8 packaging: use the current direct reference-core generator,
  share the target props across runner and MSBuild demo projects, include nested
  documentation and refresh text/introspection instructions. Format the existing
  ordinal/UTF-8 tests for the stable-Rust CI gate. Publication remains pending
  exact-candidate source and extracted-package validation.

- Implement the approved grapheme text direction: Char owns one validated Unicode
  16 extended grapheme cluster; String.Length and iteration use graphemes, with
  explicit GetScalars returning Sequence<uint> and UnicodeScalar classification.
  Preserve UTF-8 storage/conversion and ordinal equality. Numeric Char casts and
  native integer layout are removed; use the matching RavenGraphemeChar toolchain.
  Iteration currently copies snapshots and integer indexing remains deferred.
  Runtime character payloads count toward array limits. Record the .NET/Swift/Rust
  comparison and migration, regenerate 77 source slices, and validate the compiler,
  saved-project programs, signatures and editor contract. The earlier scalar-Char
  slice below is superseded.
  Independently fixed Raven expression-bodied indexer emission remains on main.
  Refresh the String feature/proposals pages with tested grapheme samples, costs and
  future encoding-specific types. Prepare a separate local VS Code snapshot; its
  expanded saved program and 20 editor completion sections pass.

- Route homepage sample boxes to feature pages and add a Raven language page with
  source-backed examples, explicit mutability and patterns, target distinctions and
  an on-site .rvnproj install/build/run guide, troubleshooting and sample downloads.
  Distinguish published Preview 7 setup from development API availability; record
  the website-first content rule for future updates. Clarify that neoCLR and its
  guest programs run without .NET; the Raven build tools and language server use
  .NET, with the VS Code extension connecting to that server.

- Migrate runtime Char values and native layout to validated Unicode scalars in
  four bytes. Reject surrogate/out-of-range values, preserve supplementary values,
  extend the pinned Unicode 16 classification table and support scalar literals in
  the Neo frontend. This breaks 16-bit Char layout; Raven integration uses
  a separate compiler contract. Complete Raven literal/pattern/array integration
  with `RavenUnicodeScalarChar`, remove surrogate-only predicates and regenerate
  the 76 library slices. Supplementary and invalid Int32 conversion programs run
  through the saved-project pipeline; compiler/runtime/editor checks pass. Wide
  numeric casts retain narrowing before validation; scalar String access remains
  a separate slice. Refresh the local VS Code workspace and website String status
  with the working scalar sample and remaining String-access limits.

- Record the preview-readiness review and fresh local VS Code UTF-8 snapshot.
  Identify scalar Char as the remaining text-model gap and retain full candidate,
  package and cross-platform validation gates. Correct old validation-page context
  so it does not select Preview 6 as the current candidate. No release is published.

- Add concise implementation pages for Option/Result, collections/queries,
  dates/clocks and bounded UTF-8 files. Reuse executable source excerpts, describe
  current limits and link future directions; add the feature index and collection
  proposal summary. All eight pages build with checked links and sample extraction.

- Adopt native UTF-8 as the text direction and change CompareOrdinal from UTF-16
  code-unit order to UTF-8/scalar order. U+10000 now sorts after U+E000; re-sort
  data that depends on the previous ordering. Keep the original proposal and
  decision history; mark scalar Char as migration work, not a compatibility goal. Thirteen ordinal/String runtime tests and the Raven
  boundary sample pass with the new ordering.

- Document the homepage/feature/proposal structure and website review with each
  feature and release. Separate Pages publication from code pushes: pushes and PRs
  still validate; only manual dispatch on main deploys. Isolate event concurrency
  so a push cannot cancel manual publication. No deployment is performed here.

- Add a separate proposals overview and feature-specific “Where we’re heading”
  sections. Distinguish working foundations, open designs and deferred work;
  link source proposals and .NET comparisons without release commitments.
  The site builder now checks all four pages, including proposal links.

- Add a concise String/UTF-8 implementation page with one executable example,
  downloadable checks and explicit open-design status. Keep detailed edge cases
  in technical docs and tests. Correct the Introspection page's stale hierarchy
  limitation. Three site pages build with checked local links and source excerpts.

- Add System.Text.Utf8.Encode(String) → Sequence<Byte> and strict
  Decode(Sequence<Byte>) → Result<String,InvalidUtf8Error>. Preserve BOM, NUL and
  normalization forms; reject malformed UTF-8 without replacement. Conversion
  snapshots bytes; no Encoding framework or Utf8String is introduced. Change
  String.IsEmpty() to the IsEmpty property; rebuild callers and target metadata.
  Retain Char semantics and UTF-16 ordinal ordering. Add executable boundary,
  allocation-limit, service-dependency, signature and editor checks. Four String
  sample programs, 23 edit/rejection checks and 16 runtime tests pass; clean
  bootstrap regeneration reproduces all 76 implementation slices.

- Clarify the final source-spelling decision: retain Raven's unit keyword and ()
  value mapped to neoCLR System.Void for now. Supersede the earlier suggested void
  keyword in the conventions guide; no compiler or runtime behavior changes.

- Document idiomatic Raven conventions using Raven's style and feature guides.
  Simplify getter-only runtime properties, prefer let for local bindings, use case
  constructors and destructuring in ordinary union callers, and expand compact
  sample control flow. Preserve carrier ABI accessors and copy-semantics fixtures;
  this cleanup does not remove the compiler/runtime union contract. Align the
  direct file probe with saved projects’ core/unit and managed-array profile for
  Ok(()); its verifier can select the matching System library. Record the desired
  void source spelling separately from the existing () → System.Void mapping.
  Regenerate the library; 81 saved-project checks, 15 application checks, 18 runtime
  reflection tests, descriptor admission and process/order workflows pass.

- Preserve source parameter-token alignment when the importer lowers an instance
  method to a free function. Reserve token zero for the synthetic receiver, which
  has no CLI Param row. This restores value-type constructor admission and the
  class-identity/value-copy sample after source metadata retention; Raven compiler
  behavior and Runtime Contract settings are unchanged.

- Make TypeInfo the fourth sealed MemberInfo case. Inherit Name, Module and
  MetadataToken; return Option<TypeInfo> from DeclaringType so top-level types have
  no fabricated owner. Preserve nested source ownership through module-scoped
  metadata and reject missing/cyclic owners. Consumers must rebuild, add the
  TypeInfo match arm and unwrap ordinary member owners. Author the sealed family
  together in Descriptors.rvn; use union patterns and case constructors in the
  changed APIs/samples and record that readability rule. Update the feature guide.
  Validate 25 runtime/metadata tests, 19 admission cases, 130 signature checks,
  38 editor checks and five saved samples plus 23 edit/rejection checks.

- Add the first in-depth Introspection feature guide with a runnable Raven tour,
  source-backed highlighted excerpts, shared expected output and sample downloads.
  Explain acquisition, discovery, scoped tokens, Sequence results and sealed member
  matching, with current limits and feedback questions. Update stale homepage API
  descriptions and validate nested pages, relative links and cross-page anchors.
  The tour and 22 shared saved-project checks pass, alongside highlighting and three
  cross-page validation tests. Desktop/narrow layouts were checked locally.
  This development guide is built locally; it does not publish a site or release.

- Plan dedicated next-preview feature pages with in-depth API walkthroughs, tested
  Raven samples, expected output, design comparisons, limits and feedback questions.
  Suggest Introspection first and strings after its later API/runtime story; no pages
  are built or published by this planning slice.

- Complete the Raven Introspection collection migration to Sequence<T>: generic
  arguments, implemented interfaces, enum names, fields, methods, properties and
  parameters now match assembly/module query capabilities. Use Count instead of
  Length and Sequence<Element> instead of array annotations; indexing, iteration and
  query extensions remain available, with no mutation members. Rebuild consumers.
  Preserve independent snapshots and private native array storage; no compiler or
  native-service behavior changes. Eighteen runtime tests, 19 admission cases, 130
  signature checks, nine saved samples plus rejection checks and editor checks pass.
  Close this Introspection story; String API/runtime behavior remains later work.

- Implement RuntimeContext.ExecutingAssembly and sealed AssemblyInfo/ModuleInfo
  interfaces with direct references and module/type discovery through Sequence<T>.
  Add module-scoped MetadataToken to Info interfaces; retain source rows and assign
  tokens to merged runtime definitions. System.Runtime appears as the foundation
  reference. Queries cover retained loaded metadata, never load files, and fault on
  unresolved references or unsupported open-generic member queries. Rebuild consumers
  for the new provider layouts. Add a
  runnable assembly-info sample and editor/interface, token and discovery checks.
  Record dynamic loading as future RuntimeContext work, without adding a loader.
  All 75 slices reproduce; 61 focused runtime tests, 27 admission cases, 24 selected
  saved-project checks and interface/editor checks pass.

- Preserve descriptive source assembly/module identity and definition tokens through
  Raven import, neoIL assembly, JSON artifacts and linking. Record the logical
  System.Runtime dependency instead of compiler bootstrap reference names; retain
  executable IDs separately and reject invalid/duplicate module-scoped tokens.
  Library slices do not copy colliding source rows into their merged module.
  Eighteen metadata/attribute/scope checks and a saved acquisition sample pass.
  This supplies the source identity for the discovery and token APIs above.

- Record AssemblyInfo.ReferencedAssemblies and module-scoped MetadataToken on the
  public Info interfaces as
  requirements for the next discovery slice. Require the System.Runtime reference
  mapping, distinguish reference identity from loading, and document source-token
  collisions when combining bootstrap slices. These requirements are implemented in the discovery slice above.

- Replace the Raven profile's public System.Type/Type.Info split with sealed
  TypeInfo identity, shape and query contracts. Both typeof(T) and Object.GetType()
  return TypeInfo; RuntimeContext.Current supplies the configured handle resolver.
  Preserve concrete types through base/interface, string, array and boxed views.
  Native descriptor results retain their public interface view. Rebuild consumers;
  only a hidden CLI attribute-token Type shell remains in the compiler reference,
  and the historical Neo profile retains its old contract. All 73 slices regenerate
  reproducibly; 62 focused runtime tests, 33 importer admission cases and 74 saved
  project checks pass (43 before a native-result fix, 31 resumed). Editor checks
  expose Info interfaces and RuntimeContext and hide Type/Info/private providers.
  Assembly/module discovery and ExecutingAssembly remain the next slice.

- Record string handling as the next focus after basic introspection. Limit the
  planned UTF-8 work to a minimal surface, leave encoding undecided, and defer
  specialized string classes such as Utf8String. Clarify that introspection followed
  by strings should provide a coherent API foundation for next-preview evaluation
  and feedback; reflection/invocation and emit remain future extensions. Record
  the selected Object.GetType spelling and ExecutingAssembly property as design
  directions; the acquisition/context migration is not reported complete.

- Add reference-only `isinst` and `ref.isnull` instructions for runtime type tests.
  Successful tests preserve object identity through interface views; failed/null
  tests return typed nulls. Interface tests also accept boxed implementations and
  preserve intrinsic string representation. Boxed values also match System.Object;
  boxed value-type targets remain unsupported.

- Migrate the Raven profile's six Info contracts to sealed interfaces with internal
  Runtime*Info providers. Preserve permitted-type metadata in the reference and
  reject external implementations in the target importer. Materialize native
  snapshots as implementing classes and emit interface dispatch; rebuild callers.
  MemberInfo matches exhaustively over FieldInfo, MethodInfo and PropertyInfo;
  import and execute Raven's reference type patterns, and reject missing cases.
  System.Type/Info acquisition and array-return signatures remain transitional.
  Record the subsequent proposal to add TypeInfo as a member case with optional
  declaring-type ownership; that extension is not implemented yet. The reflection
  sample now expects no class base for FieldInfo; saved-project checks can resume
  from a named sample after a fixture correction. All 71 saved-project checks,
  29 reflection checks and 21 reference/boxing checks pass.

- Record the author's selected sealed Info hierarchy, replacing the proposed open
  implementation model. Permitted runtime providers remain internal; collection
  return contracts and RuntimeContext implementation are separate work.

- Record the open collection-return question and propose capability-based interfaces,
  with Sequence<T> a candidate for materialized introspection results. Distinguish
  read-only views from immutable snapshots; no array-return policy is implemented.

- Preserve declaring-type metadata for Raven-authored Option/Result cases by emitting
  them inside their companion containers. This repairs a full-port regression without
  changing managed method bodies or case storage. Check all four cases against their
  resolved owner identities; regenerate all 73 slices and the API inventory. All 12
  generic-union admission cases, 34 focused runtime tests and source ownership/snapshot
  checks pass. Record the final source-port Rust baseline: 1246 passing tests across
  all 178 integration binaries plus unit tests, with corrected failures rerun. All 65
  saved-project cases pass against the corrected library, closing the execution gate.
  RuntimeContext and the interface-based API migration remain separate work.

- Establish System.Runtime.rvnproj and its System.Runtime managed implementation
  identity, replacing System.rvnproj/NeoCLR.System without duplicating source owners.
  All 73 slices compile/import under the new identity; generated executable bodies
  are unchanged and ownership/snapshot checks pass. The bootstrap reference and
  executable neoIL retain their explicit mapping. RuntimeContext/interface API
  migration remains the next slice; published release artifacts are unchanged.

- Record the next API-alignment direction: establish System.Runtime, implement the
  basic RuntimeContext/provider model, retire System.Type for TypeInfo and use
  Object.GetTypeInfo for instance acquisition. This is the selected plan, not an
  implemented API; it supersedes the tentative Object.GetType spelling. RuntimeContext
  owns context-dependent discovery formerly represented by .NET Assembly static
  APIs; the first selected surface is executing AssemblyInfo → modules → types.
  Exact member spelling is provisional; loading remains outside the initial scope.
  Record a possible optional collector capability on RuntimeContext for later design,
  including heap/lifetime questions; no GC API or execution-policy change is made.

- Complete the Raven branch review and prepare an isolated VS Code development
  workspace with reproducible setup tooling. General editor-reference and ref-struct
  deconstruction fixes are on Raven main (65 and 31 focused checks); neoclr retains
  target policies. Rename its integration branch to neoclr and remove eight merged
  fix branches. Project checks pass on main/neoclr (47/52). Fresh copied-toolchain
  workspaces run the saved sample and pass editor checks; the installed editor logs
  confirm successful startup. No release or global SDK installation is performed.

- Refresh two validation fixtures after the source port: retain the Int64 CompareTo
  parameter name in its expected signature and allow the Console reachability graph
  to include generated carrier dependencies. Primitive admission and all 11 Console
  tests pass; runtime graph limits and service assertions are unchanged.

- Remove the obsolete TypeOf<T>.Of helper from runtime and Raven reference metadata;
  use typeof(T), or ldtoken/GetTypeFromHandle in direct IL, and rebuild existing
  helper callers. Update active samples while retaining published preview notes.
  Thirteen type/reflection tests and the saved reflection program pass; old helper
  calls are rejected. Add an ownership gate covering all 724 selected method/function
  declarations (54 explicit runtime services, the remainder generated). Record
  Object.GetType as a later preview API candidate and the development policy allowing
  deliberate compatibility breaks while retaining useful .NET ergonomics.

- Port managed Array members, callbacks and private iteration to Raven (73 slices).
  Preserve intrinsic storage and adapt the source iterator factory to the existing
  dispatch ABI. Retain static property metadata and reject fabricated array storage.
  Eight admission checks, 25 array/collection tests and four saved array programs
  pass. The historical Neo array profile remains unchanged.

- Port empty Object and UnionAttribute declarations to Raven (72 source slices).
  Check exact empty constructor and declaration shape while preserving the existing
  runtime root/marker ABI and compiler-facing recognition metadata. Nine admission
  checks, 22 attribute/union/reflection tests and the saved generic-union program pass.

- Port BindingFlags as a normal Raven Flags enum (70 source slices), preserving
  its six Int32 values, unknown bits and reflection filtering. Checked intrinsic
  enum lowering supplies the existing runtime ABI. Seven admission checks, 31
  enum/reflection tests and the saved flags program pass. Integrate Raven's general
  CLI enum backing-field metadata correction independently on main (266b457f5;
  13 focused checks), keeping neoCLR-specific lowering separate.

- Port the five Func declaration arities to Raven (69 source slices). Check exact
  CLI runtime delegate metadata, generic positions and the complete family;
  invocation and closure lifetime remain runtime-owned. Six admission checks,
  28 delegate tests and the saved Raven delegate program pass.

- Port System.Fault to Raven (68 source slices), preserving dynamic diagnostic text
  and terminal guest failure without aborting the host. Three admission cases and
  seven fault/query tests pass; a compiled Raven program verifies and raises the
  expected Unicode diagnostic. Refresh source ownership audits for Fault,
  descriptors and native allocation.

- Port NativeMemory overload composition to Raven (67 source slices), with checked
  bootstrap-only allocation/release/native multiplication operations. Preserve the
  consumer API, overflow, bounds and lifetime behavior. Five admission cases, 28
  native/pointer tests and the saved Raven allocation program pass.

- Port MemberInfo, FieldInfo, MethodInfo and PropertyInfo to a checked Raven source
  slice (66 total). Preserve inherited snapshot layout and runtime field names;
  Raven now owns parameter-array copies and accessor visibility filtering. The
  bootstrap-only vector read view is absent from consumer metadata. Nine admission
  checks, 23 reflection tests and the freshly compiled Raven reflection sample pass.
  Legacy Neo descriptors retain their existing representation.

- Integrate the independently validated Raven namespace/generic-constructor lookup
  correction on main (`d7292b935`) and the neoCLR feature branch (`d833ef2f3`). The
  ordinary .NET regression now returns 42 instead of silently returning 0; 78 focused
  main checks and 18 feature checks pass. No neoCLR policy was integrated into main.

- Extend the API-preserving Raven library port from 28 to 65 source slices: generic
  fundamental/collection contracts, SystemClock/LocalDateTime, native-sized integer
  comparisons, complete Int32 methods, Console, Environment, File.ReadAllText,
  String, opaque Error, five empty error types, Void, seven typed error carriers, Propagatable, Option/Result and their cases. Check intrinsic String storage and mixed receivers,
  preserve Error message delegation and runtime-owned payloads, and reject opaque
  allocation/defaults and String storage writes (11 admission cases pass). Preserve
  empty error defaults/constructors and nominal Void with nine further admission
  cases. Extract a general Raven unit-assembly lookup fix onto main (`c17cb8397`):
  same-named source types no longer shadow the configured metadata contract
  (19 .NET checks and 24 feature-branch checks pass). The empty-value slice passes
  14 focused Rust checks, four saved-program executions, and wrong-case/default
  error rejection checks.
  Check generic interface inheritance and nested calendar layout against reference
  metadata, and preserve library method parameter names for introspection. Keep native
  payload decoding bootstrap-only and preserve legacy Neo receiver/process conventions.
  Regenerate implementation snapshots and API audit.
  Typed carrier checks preserve case storage, value constructors, wrong-case faults
  and invalid-default rejection: 14 admission checks, 25 focused Rust tests and all
  64 saved-program cases pass with a fresh consumer core.
  Propagatable now has a checked Raven declaration preserving readonly receivers
  and true-only output initialization; five declaration admission checks and 20
  focused runtime propagation/union-output/generic-bound tests pass.
  Extract the ordinary CLI out-forwarding assignment fix onto Raven main
  (`5f6e17347`; 41 focused parameter checks pass); keep the target on its feature
  branch with cherry-pick `2d2a1d586`. Port Option/Result storage, factories and
  conditional extraction with checked generic constructors, readonly copy adapters
  and literal Boolean return proofs. Twelve admission cases, 41 focused Rust tests
  and all 64 fresh saved-project cases pass. A bootstrap-only miss intrinsic leaves
  output untouched; invalid defaults remain rejected.
  Source migration is still incomplete for remaining
  descriptors and runtime adapters; proposal API alignment remains a subsequent step.
  Pass 64 saved-project compile/import/verify/execute cases and retain native
  failure-status tests. Extract general Raven interface-binding and no-result delegate
  bridge fixes onto Raven main while retaining the neoCLR target on its feature branch.
  Update the inline managed-array fixture to embed generated collection declarations;
  the text assembler does not expand the new wrapper includes. All eight fixture
  checks pass after this correction; all 1241 Rust tests pass against the final
  foundation library, with all 178 integration-test binaries covered. The following
  String/Error slice adds native UTF-8 status injection and preserves existing
  introspection parameter names and String method order; all 21 focused
  string/error/default/interface Rust checks pass.

- Organize documentation with a top-level index and categorized guides, reference,
  design, Raven integration, history, contribution and experiment indexes, retaining
  existing document paths and published notes. Include the author-supplied original
  proposals with links to maintained design notes and explicit proposed status.
  Record source porting before the subsequent API-alignment step.

### 2026-09-17

- Plan System.Runtime as the minimal Raven-authored managed foundation, derived
  from the current System project. Document candidate scope, .NET comparison,
  namespace/assembly separation and coordinated identity migration. Link the plan
  from library docs and website; no assembly rename or new project is implemented.

- Add an executable RuntimeContext POC: opt-in Raven typeof returns the shared
  TypeInfo interface through Current.GetTypeInfoFromHandle, with internal runtime
  adapters. Run the source entry unchanged and separately check repeated/distinct
  scalar and array handles. Update integration docs, proposal and website sample.
  The installed core and production descriptor identities are not migrated yet;
  the bridge requires the updated Raven neoCLR feature branch.
  Validate 23 focused compiler tests, the context and existing introspection
  consumers, bootstrap snapshot consistency and the website build/highlighter.

- Extend the isolated Introspection prototype with TypeInfo.GetFields(flags) and
  FieldInfo interfaces. Field Type and DeclaringType use the shared TypeInfo model;
  preserve runtime filtering, ordering and unsupported-flag faults. Verify interface
  inheritance, structural signatures and rejection of dynamic field access. Update
  samples, docs and website. The eager Iterable adapter is experimental; production
  descriptor identities and RuntimeContext remain unmigrated.
  Record the required hidden Runtime*Info implementations and shared runtime-owned
  type-of/RuntimeContext acquisition yielding identical or value-equivalent descriptors.

- Fix neoCLR importer conversion of mixed constructor arguments: convert each
  argument in an adapter rather than repeatedly converting the top stack value.
  Cover class and value constructors, multiple Boolean arguments and single
  left-to-right source evaluation. Update integration docs; no Raven compiler or
  public API change.

- Admit checked nongeneric interface declarations in the Raven runtime-library
  importer and port the existing Clock contract to Raven. Preserve its Now property,
  System namespace and runtime-backed implementation. Reject mismatched interface
  shapes, properties and referenced type identities; validate FixedClock/SystemClock
  consumers and update samples, docs and website. This enables interface authoring
  but does not yet migrate production Info identities or introduce RuntimeContext.

- Add an isolated executable Raven probe for TypeInfo/MemberInfo interfaces with
  internal runtime-backed adapters and DeclaringType returning TypeInfo. Check
  emitted interfaces, execution and rejection of invocation/legacy Info access.
  Preserve BindingFlags in acquisition and expand existing member-consumer samples.
  Update docs and website; production descriptor identity and RuntimeContext are
  not migrated by this experiment.

- Refine the Introspection proposal to one `*Info` interface model without public
  Type/TypeInfo pairs, runtime-backed v1 through RuntimeContext, separate Reflection
  binding and Emit generation, and collection-oriented queries. Defer offline
  MetadataContext and typed introspection. Update migration docs and website;
  preserve superseded designs as history. These are target contracts, not changes
  to the existing executable API.
  Follow-up clarification retains BindingFlags during the current migration;
  collection-oriented querying remains a future option.

### 2026-09-16

- Port existing TypeInfo queries and its internal factory to Raven, retaining
  runtime-backed snapshots and filtering. Validate the opaque-handle layout and
  internal factory contract, and preserve internal visibility in generated code.
  Flag overloads now use the parameter name `flags`; regenerate metadata and
  update named calls using `arg0`. Clarify that Introspection need not be runtime
  independent: Reflection and Emit compose capabilities within one descriptive
  model. Update docs and website; member hierarchy bodies remain neoIL.
  Record future mixed-origin descriptor binding requirements for Emit, including
  context/provenance and identity validation, without adding new public APIs.
  Update Raven introspection samples with named `flags` arguments, an explicit
  TypeInfo binding and expanded readable blocks.

- Port System.Introspection.ParameterInfo's six read-only properties to Raven in
  the Raven runtime profile. Check snapshot field order/types and private construction;
  reject added/reordered fields, missing exports and public constructors. Regenerate
  bootstrap artifacts and update the documentation and website. Public APIs are
  unchanged; TypeInfo and member hierarchy bodies remain neoIL.

- Move existing TypeInfo, member/parameter descriptors and BindingFlags from
  System.Reflection to System.Introspection across runtime factories, reference
  metadata, Raven/neoIL consumers and bootstrap artifacts. Rebuild System and
  applications together and update imports; no old-name aliases are provided.
  Query behavior is preserved. Type remains Raven-authored; descriptor bodies
  remain neoIL and Info remains an instance property for this first slice.
  Update website status and migration documentation.

- Clarify the next release as an API-shape demo/POC across the text, time,
  globalization, async, introspection, filesystem and stream proposals. Plan Raven
  declarations and matching metadata alongside selected executable paths, with
  incomplete capabilities explicitly identified; full implementations are not a
  release prerequisite.
  Reflect this objective on the website with all seven proposal families, their
  benefits, design links and current implementation status.

- Record the proposal-driven Raven library migration priority: recommend porting
  existing introspection descriptors into System.Introspection next, followed by
  the existing time family under System.Time. Namespace moves and remaining Raven
  ports are planned; reference metadata and consumers must migrate together.

- Record a provisional file-system capability proposal: keep Path as a pure value,
  resolve it through an injectable FileSystem with a Default implementation, and
  separate File/Directory handles from byte-oriented streams. Compare the current
  static File/Directory APIs and System.IO.Abstractions, and leave authority,
  lifetime, enumeration, error, sandbox and async contracts open. No existing I/O
  API or runtime behavior changes.

- Record the requested Globalization, Introspection/Reflection and Stream proposals.
  Globalization separates immutable Culture/CultureId, resolution, contextual providers,
  formatting and collation; Introspection separates descriptive metadata from optional
  Reflection/Emit capabilities; Stream uses composable synchronous/asynchronous byte
  capabilities above FileSystem. Existing APIs and runtime behavior remain unchanged;
  ownership, data/versioning, capability availability, cancellation and exact members
  remain provisional.

### 2026-09-15

- Add the initial Clock.Now/Instant API with SystemClock, Raven-authored Instant
  and Duration values, and system-zone Instant.ToLocalDateTime conversion. Values
  use signed 100 ns ticks; Instant uses the Unix epoch. LocalDateTime now contains
  only Date and Time. Remove Clock.GetLocalNow and UtcOffsetSeconds; examples use
  Raven and clock injection. Configurable zones/calendars and duration arithmetic
  remain future work. Runtime tests replace the archived Neo clock smoke example.

- Replace placeholder parameter names in Raven-authored Date, Time and Type APIs
  and their reference metadata with descriptive names. Named calls must use the
  new labels (for example `year`, `ticks`, `other` and `index`); positional calls
  are unchanged. Exercise named arguments in calendar and reflection samples.

- Port System.Type to Raven for the Raven runtime profile. Type keeps identity and
  basic shape; metadata enumeration, base/interface lookup and enum metadata are
  exposed through System.Reflection.TypeInfo using the same opaque handle. Migrate
  Raven consumers to `.Info`; reflection descriptors remain in NeoIL. Validate Type
  layout and private construction, support matched class factories and self-array
  signatures, and require handle fields to be assigned before constructor return.
  The archived Neo profile retains forwarding APIs during migration. Descriptive
  parameter names are planned as the next slice.

- Clarify the closed reflection model: TypeInfo is a separate type-metadata view;
  MemberInfo remains the base for FieldInfo, MethodInfo and PropertyInfo.

- Introduce `System.TypeInfo` as the explicit member-lookup view returned by
  `Type.Info`. Type remains the identity and shape descriptor; field, method and
  property enumeration moves behind TypeInfo while the descriptor contracts remain
  unchanged.

- Add `System.Reflection.TypeInfo` and `Type.Info` as the first Type/Reflection
  separation slice. TypeInfo owns explicit member lookup while existing Type query
  methods remain forwarding compatibility shims during the transition.

- Move the memberless Value and RuntimeTypeHandle declarations to Raven. Check
  declaration-only imports against reference shape and reject added members,
  storage or constructor behavior. Preserve runtime erasure and opaque handle
  representation. Void remains declared in IL because the separate implementation
  assembly cannot define Raven's configured target-core unit type.

- Complete the Char struct port in System/Char.rvn, including CompareTo and all
  sixteen predicates. Move the previously standalone functions onto the type;
  retain existing UTF-16 code-unit and Unicode-category behavior through the native
  category service. The proposed text redesign is not part of this migration.

- Port Boolean, SByte, Byte, Int16, UInt16, UInt32, Int64, UInt64, Single and Double
  structs to Raven. Preserve comparison/NaN ordering and intrinsic storage; checked
  primitive backing fields lower to value loads and verified empty constructors are
  omitted. Reject extra storage, writes and effectful constructors. IntPtr/UIntPtr
  remain IL pending native-integer operator/conversion reference contracts.

- Port File.WriteAllText outcome handling to Raven, preserving all seven typed errors,
  Void success and the terminal fault for unknown native status. Keep native writes
  and ReadAllText in their existing implementation layers. Match the two target Void
  metadata encodings only in generic arguments, with core identity and empty-value
  checks; no-result return matching stays distinct.

- Organize Raven library sources into namespace folders under src/System, including
  Collections, Linq and function namespaces. Update project inputs, bootstrap source
  maps and documentation; public contracts and generated instructions are unchanged.

- Port Path.Combine and Path.GetFileName to Raven using the checked bootstrap host
  service catalog. Preserve lexical/native platform behavior, public parameter names
  and direct IL callers; no new path API or compiler special case.

- Complete the Math port with all 15 Double operations authored in Raven and the
  same native numeric services underneath. Add a bootstrap-only, signature-checked
  service catalog; guest imports cannot use it. Preserve numeric behavior and the
  public namespace contract without Raven compiler changes.

- Port Date to Raven, preserving Gregorian validation, day-number limits, component
  properties, comparison and Result errors. Reuse the checked Time value-import
  support without compiler/runtime changes. Keep API redesign separate from the port.

- Port Time to Raven with its existing Result factories, tick boundaries, properties
  and comparison contracts preserved. Extend matched value-library imports with
  interfaces, owned static factories, private constructors and verified readonly
  receivers; project matching calendar layout metadata. Share calendar regression
  cases across bundled and Raven profiles and test readonly/private access rejection.
  Date migration and API redesign remain separate work; no Raven compiler change.

- Record a new provisional Unicode-centred text-model proposal: canonical UTF-8 String
  storage, explicit UTF-8/UTF-16 representation views, constrained ASCII subset types
  and codec-based handling of other encodings. Mark the Raven-shaped examples as design
  notation only. Preserve the current UTF-16 code-unit Char contract until migration,
  metadata, interop and validation questions are resolved; no implementation changes.

- Refine the String proposal into a consolidated Unicode/text model: immutable String
  as scalar Char values, canonical UTF-8 storage, explicit encoded-string views for
  UTF-8/UTF-16/ASCII, separate Encoding transformations, and Option/Result/Fault
  handling. Keep the current UTF-16 Char and String-slicing implementations unchanged;
  the proposal's Raven-shaped examples, view lifetime, metadata and migration rules
  remain provisional.

- Update the date/time design proposal as NeoCLR Time API v1: layer Instant, Duration,
  civil values, offsets, timezones, calendars and Period; retain a narrow injectable
  Clock; and model DST gaps/overlaps and parsing/lookup failures explicitly with
  unions/Result. Keep the current Date/Time and local-clock implementation unchanged;
  the Raven-shaped examples, exact members, defaults, timezone data and arithmetic
  policies remain provisional.

- Add the first matched Raven value-library gate: sequential nongeneric scalar records
  must match reference field layout, representation and public instance contracts.
  Preserve instance ownership for value constructors and constructor debug identities;
  admit checked managed-reference `ldobj` copies without widening guest imports.
  Independent Int32/Int64 probes cover construction, copying, receiver mutation and
  private-field/layout/category rejection. Existing class/generic probes and bootstrap
  regeneration pass. Date/Time migration, value interfaces and static factories remain
  subsequent work; no Raven compiler or runtime instruction-set change is included.

- Port HashMap algorithms to Raven while retaining Map/MutableMap contracts in IL.
  Preserve explicit callbacks, chained hashing, growth, Option lookup, duplicate
  handling, shallow key snapshots and reentrancy faults. Store callbacks directly;
  diagnose capacity overflow before multiplication. Admit private nonvirtual instance
  helpers in matched library classes, preserving private visibility and validating
  every body without relaxing the public reference contract or guest imports.
  No Raven compiler or Runtime Contract change is required. All 18 collection/query
  runtime tests and 63 saved-project checks pass, together with private-method,
  private-storage and instance-contract probes and clean bootstrap regeneration.

- Port the complete ArrayList class and its private iterator to Raven, including
  constructors, indexing, growth, shallow copies and all seven predicate searches.
  Replace the legacy shared-state wrapper with direct checked array storage while
  preserving aliasing, captured-buffer iteration and Option outcomes. Remove the
  handwritten search fragment and constructor-rewriting adapter. Build the class as
  a checked slice of the shared project; preserve the reference/implementation gate.
  Capacity overflow now has an explicit list diagnostic. Update the reflection sample's
  build-local definition index and document the temporary bootstrap split. Recognize
  compiler array-invariance diagnostics when test projects inherit their configuration
  through MSBuild imports, while still requiring rejection without stale execution.
  Validation: 18 runtime tests, 30 query cases, eight mutation cases and 63 saved-project
  checks pass; bootstrap regeneration and API inventory/coverage checks pass.

- Port Where/Select sequences and iterators to Raven, completing all nine current
  query overloads in the shared runtime project. Remove handwritten deferred query
  bodies; preserve callback timing, cached Current, disposal and terminal faults.
  Retain generated internal classes in reproducible bootstrap fragments. Thirty Raven
  query cases, 33 delegate/query/reservation runtime tests, 203 scalar outcomes and
  13 cross-library cases pass, alongside authoring probes and clean regeneration.

- Add bootstrap-only checked generic array storage and scoped internal helper classes
  to matched Raven library imports. Keep consumer metadata and ordinary application
  admission unchanged. Validate Int32/String/Void storage and private helper dispatch,
  reject unwritten reads, invalid capacities and leaked helper contracts. Independently
  fix imported generic calls and generic array operations in Raven main with .NET
  execution tests and integration gates; retain neoCLR policy on its experimental track.

- Permit class constructors to initialize delegate fields. Reserve those fields as
  typed uninitialized storage instead of trying to manufacture a default delegate;
  reject early reads and constructor return with an uninitialized field. Other
  defaults are unchanged. All 25 delegate tests pass, including three new construction
  cases. This does not introduce null/default delegates.

- Add System.Fault(message) as a namespace function exposed to Raven through its
  existing CLI container contract. Preserve computed UTF-8 diagnostics and terminate
  guest execution through the existing Fault outcome; embedding hosts are not aborted.
  No cleanup guarantee or compiler non-returning-call analysis is introduced. Add
  runtime and Raven consumer checks and document the FailFast comparison. Deferred
  query migration uses the checked-storage and private-helper slices recorded above.

- Extend matched Raven library class imports to unconstrained generic parameters,
  constructed fields/methods, generic locals and supported interface declarations.
  An independent Cell<T> probe validates Int32/String/Void payloads, construction,
  mutation, copies and Iterable dispatch; arity/interface mismatches are rejected.
  Fix explicit generic self-construction independently in Raven and cherry-pick it
  into the experiment, with four new .NET execution cases and 17 focused tests.
  Raven main integration passes 311 compiler, 73 core and 249 language-server checks
  (three existing skips); existing neoCLR authoring/cross-library regressions pass.
  Subsequent same-day slices add checked storage, fault operations and private
  implementation dependencies, then port the deferred query bodies.

- Port Int32.Divide and seven Char predicates to the shared Raven runtime project,
  preserving Result errors, UTF-16 code-unit/Unicode behavior and existing public
  static APIs. Native parsing/category services remain in the runtime. Admit checked
  static implementation fragments on nominal core owners; refresh bootstrap snapshots
  and API coverage. Correct CLI Boolean joins for argument/local/field loads and
  ordinary call results while preserving conditional-out assignment proof. A separate
  Raven consumer passes 203 scalar and short-circuit outcomes; 30 query cases and
  19 focused runtime tests also pass. Raven compiler and
  Runtime Contract settings are unchanged; generic collection instance ports remain.

- Record Clock/SystemClock and List<T>.Create proposals, their .NET/Noda Time
  comparisons and open design choices. Confirm the current mutable List contract;
  no new clock abstraction, default accessor or collection factory is implemented.

- Add a bounded Raven instance-library import gate: validate one nongeneric class
  against its separate reference contract, retain its public runtime identity, and
  import constructors, private fields, methods and properties. Preserve ordinary
  guest assembly identities. An independent Raven execution probe and five invalid
  contracts pass, alongside 13 cross-library checks, generic-library checks and
  clean Math/query snapshot regeneration. Generic instance classes, interfaces and
  further System API ports remain subsequent work; Raven itself is unchanged.

### 2026-09-14

- Port seven query terminal overloads to the shared Raven System project: ToList,
  First, Last and Single, including predicate overloads. Import constructed generic
  collection/delegate/Option/Result signatures with scoped parameters and generic
  adapters. Preserve outcomes, cleanup and fault boundaries; five runtime terminal
  tests and 30 query integration cases pass, alongside generic/Math/Void checks.
  Refresh the signature probe for Math namespace metadata; all 121 contract checks pass.
  Build and flatten both Math and query snapshots for standalone use. Rename the
  extension-method container to System.Linq.Operators; rebuild consumers and core
  metadata together. Extension syntax is unchanged. Instance types/deferred iterator
  implementations remain neoIL pending shared implementation/reference identity.
  Extract general imported-signature, member-proxy, sibling-type lookup and empty
  union-case construction fixes to Raven main; retain target policy in its experimental
  branch and document both sides. Raven main CI passes 311 compiler, 73 core and
  249 language-server checks (three existing skips).

- Resolve generic unit-return invocation behavior through an independent Raven main
  fix (327335699; experimental cherry-pick ef352917e). Preserve the actual generic
  return value when consumed and pop it when discarded; ordinary no-result calls
  retain their existing behavior. All 22 focused .NET checks pass. Extend the generic
  library probe to consumed/discarded Void results; existing importer/runtime handling
  passes without changes. Update compiler/integration documentation and close the
  previously recorded unit-result candidate. Existing Void propagation and all
  69 Math results/six rejected contracts continue to pass.

- Admit bounded unconstrained generic namespace implementation bodies against an
  existing reference contract. Check generic arity/parameter positions, preserve open
  bodies and validate same-fragment calls using existing runtime generic functions.
  Add a test-only Int32/String probe and reject mismatched export contracts. Generic
  classes, constructed signatures and constraints remain later migration gates.
  Fix a general Raven generic-method projection crash independently on main
  (5f93eef6a; experimental cherry-pick 64a5497ec), with 12 focused .NET checks passing.
  Document the separate unresolved generic unit-return invocation issue in both repos.
  The generic probe, all 69 Math results/six rejected contracts, Void propagation
  and clean Math regeneration pass. No public System API or SDK package changes.

- Select System.Void through Raven's unit Runtime Contract in the neoCLR project
  properties. Document one platform unit type, no-result calls versus explicit value
  contexts, and the compiler/target integration boundary. Add metadata and runtime
  checks for Void arguments and Result<Void, E> propagation. Track the shared project
  properties in Math bootstrap snapshot inputs and regenerate with the new contract.
  General function return
  diagnostics were fixed on Raven main (0c66fbaa7), separately copied to the experiment
  (e20534894); invalid unqualified annotations no longer silently emit Object returns.
  Reusable Runtime Contracts are independently integrated into Raven main through
  2d17199a1; experimental unit projection is fc4592663. Require documentation of
  compiler-affecting changes in both repositories. The combined Raven contract suite
  passes 87 checks and its bounded CI passes; 13 experimental unit checks, the Void
  propagation probe and existing Math validation pass. No SDK refresh is included.

- Selected namespace functions for the Math migration and documented a general
  preference for namespaces over utility classes used only to group operations.
  Retained Raven's existing CLI container/TopLevelAttribute contract for now;
  recorded cross-language and reflection costs and the required migration checks.
  Added the shared Raven System.rvnproj with its first migrated Int32 Math bodies
  (Abs, Min, Max, Sign and Clamp), a reference-checked importer, generated bootstrap
  snapshots and source-release freshness validation. Typed Result outcomes and the
  existing internal runtime method owner are preserved; Raven consumers see the
  namespace API. Documented builds, bootstrap identities and the remaining generic
  implementation gate. Qualified lookup with local namespace declarations exposed
  a general Raven bug, fixed independently on Raven main (3ec32c96e) and copied to
  the experimental branch (008cb3245). The installed .14 SDK predates that fix.
  Math uses imported Ok/Error cases; recorded the remaining unqualified carrier
  return-annotation issue for independent Raven investigation.

- Reconciled the published Preview 7 documentation with the subsequent async/API
  direction notes, preserving both histories before resuming library migration.

- Recorded the selected task-based async direction with intended runtime-owned
  suspension, Task<Void> for no-payload completion, and optional transitional Raven
  state-machine lowering. Clarified that System APIs target modern language consumers
  with tasks as the normal async pattern; public runtime APIs may use callbacks where
  they better express the contract, and application code remains free to use them.
  Added a source-backed assessment of transitional compiler-generated async and its
  remaining library, lowering and execution-ownership gaps. Selected Result-bearing
  task completion for recoverable async errors. Selected ordinary async API names
  identified by Task return types, without mandatory Async suffixes; explicitly named
  blocking/sync alternatives remain exceptional. Documented .NET naming migration
  costs and provisional comparisons for markers, cancellation and operation ownership.
  Updated planning and conversation records; concrete task contracts and async
  implementation remain future work, without a requirement to preserve historical
  .NET compatibility.

- Published Preview 7 from 5da27a7 after all six source CI jobs and all 18 packaged
  validation suites passed. Updated the website and current walkthrough to point
  to the release. Recorded the validation outcome and installed a fresh local runtime
  bundle with the existing .14 SDK/VSIX. Candidate-preparation notes below describe their pre-publication
  state; published release content remains unchanged.

- Prepared Preview 7 candidate with the combined MSBuild/library workflow and
  Type.IsValueType. Refreshed the API declaration and coverage inventories to
  include the property and getter (616 declaration candidates). Publication remains
  subject to packaged validation and CI.

## 0.1.0-preview.7 — 2026-09-14

Release candidate; publication awaits packaged validation and all required CI checks.

### 2026-09-14

- Added read-only Type.IsValueType classification from runtime type categories,
  including the Raven reference/import surface and reflection sample. Managed arrays,
  classes, strings, interfaces, pointers and byrefs are distinguished from values;
  allocation location does not determine the result.

- Planned a needs-driven introspection/reflection review: compare the .NET
  Type/TypeInfo model, metadata inspection, execution capabilities and AOT retention
  before expanding descriptor APIs. No replacement hierarchy is selected. Extended
  API planning to distinguish a proposed common platform core, optional capabilities
  and host-specific services, with availability and conformance questions left open.

- Added a minimal standalone MSBuild build path for Raven `.rvnproj` applications
  targeting neoCLR. Props/targets select the supplied reference contracts and run
  the installed compiler, importer and verifier, without Microsoft.NET.Sdk or Raven
  changes. Build does not execute the program and invalidates prior runnable output
  on failure. Added a packaged demo, VS Code build-task configuration and regression
  checks. Installed a fresh local MSBuild demo with the existing .14 tools: 15 build
  scenarios, 68 language-server checks and the propagation demo pass; all 758 manifest
  files verified. Expanded the candidate to one application-to-library ProjectReference,
  with core-pack compatibility checks, changed-library rebuilds, failure invalidation
  and pre-build library completion. Added the two-project demo and made MSBuild the
  primary documented Build/Run workflow. Incremental builds, restore and deeper
  dependency graphs remain out of scope.

- Synchronized reviewed Raven main fixes into the experimental branch (`ee7b2e5af`)
  and installed SDK/VSIX `0.1.12-neoclr.14` with a fresh matching runtime bundle.
  Recorded installation instructions, revisions, hashes and packaged validation:
  63 saved-project, 15 application, 30 query, 5 compiler, 121 signature checks;
  68 editor checks each against bundled and installed servers; four array and four
  neoIL programs. All 751 manifest files verified. This is a local candidate only.

- Completed the independent Raven empty-array factory review (`f70ba5026` on main):
  use target metadata capabilities instead of assuming the host factory exists.
  All 93 focused checks pass, including .NET 10/.NET 11 reference-pack cases.
  Include the generic-array API execution verifier in future runtime bundles, and
  extend packaged editor checks for Empty, instance ForEach and indexed access.

- Recorded the proposed separation of Raven target profiles, symbol projections and
  emission backends, plus a staged evaluation plan. Preserved possible Raven compiler
  bootstrapping and neoCLR architecture/AOT/microcontroller work as long-term questions,
  not implemented features or preview commitments. Linked the roadmap and conversation
  record to the architectural proposal. Clarified the overarching theme as targetability
  and portability, with microcontroller architectures as potential targets. Added
  a future minimal MSBuild build path for Raven projects targeting neoCLR, independent
  of .NET SDK integration. Broader build features are optional later slices; this
  records a plan, not implemented build support.

- Implemented `Array<T>.Empty` and instance `ForEach(Func<T, Void>)` in the Raven
  runtime profile, reference surface and importer. Empty currently allocates a
  zero-length array without a shared-identity guarantee. Removed the profile's static
  ForEach helper; use `values.ForEach(action)`. Updated samples, reflection coverage
  and source validation instructions. Four Raven samples, 121 signature checks and
  32 runtime/collection regressions pass; installed tools still require refresh. Kept configured array-member projection
  and nominal generic Void emission on Raven's experimental branch.
- Independently fixed void-call stack-result tracking on Raven main (`8dbd96fb6`),
  then cherry-picked it into the experiment (`5c32d1d06`). Both ordinary and target-
  metadata .NET reproductions now run; 39 focused runtime and 21 initial checks pass.

- Recorded Raven main's qualified union type-pattern fix (`b0681f32b`), including
  closed variant member types and target-core locals. All 272 focused checks and the
  .NET 10/.NET 11 build/run matrix passed. Kept array-factory review, experimental
  synchronization and installed-tool refresh pending; library migration stays paused.

- Recorded the author's indexer-completion report and Raven main fix `ac4901f6b`:
  indexers require `[index]` access; their metadata names no longer behave as ordinary
  properties in lookup, hover or completion. All 440 focused checks passed. Kept
  installed-tool refresh explicitly pending and documented the general CLI/C# basis.

- Recorded Raven main's imported-union emission fix (`43f288b05`): closed pattern
  locals, valid primitive method signatures, retained assembly scopes and correct
  metadata containers for Raven union-case accessors. All 291 focused checks and the
  .NET 10/.NET 11 matrix passed. Kept bare type-pattern failures, boxing optimization
  and array-factory review explicit; experimental tools remain unsynchronized and
  runtime-library migration remains paused.

- Recorded Raven main's metadata core-library identity fix (`11e5c57ec`): imported
  structs retain their category using the supplied core assembly's full identity.
  All 33 focused checks, the full baseline (5,515 reported passes), and the .NET
  10/.NET 11 build/run matrix passed. Marked the general core-library prerequisite
  resolved while retaining the separate union-emission/array-factory reviews and
  the paused runtime-library migration. Experimental tools remain unsynchronized.

- Independently integrated imported member-union shorthand binding into Raven main
  as `b60e3635f`; 219 focused checks and the .NET 10/.NET 11 build/run matrix passed.
  Recorded the remaining target-core struct/signature failures exposed by execution
  and assembly inspection, and prioritized general metadata core-library identity
  alignment before further emission changes. Kept the boxing optimization and array-
  factory review separate; library migration and experimental synchronization remain
  pending.

- Expanded the website with project influences, six implemented feature summaries
  and nine excerpts sourced from executable Raven/neoIL samples. Added comparisons
  and tradeoffs, corrected stale importer limitations, and separated open research
  from preview capabilities. Led with Raven code, retained neoIL as a supporting
  example, and invited general feedback and discussion. The Pages workflow now
  rebuilds for Raven sample changes; desktop/mobile layout and link checks passed.
  Refined the narrative around familiar semantics and metadata, generic Void and
  Func callbacks, evolving class-library APIs including date/time, and Raven
  migration/tooling. Added planned library migration from neoIL to Raven and
  missing API work, without claiming those plans are implemented. Highlighted the
  current interpreter, tracing GC and terminal debugger; listed a possible JIT and
  collector improvements as future investigations, separate from Raven VS Code
  source-debugging support. Positioned neoCLR as an independent managed software
  platform inspired by .NET, evolving through community feedback, with Raven
  demonstrating the platform rather than defining its scope. Reorganized the page
  around .NET-familiar platform layers and colocated library examples, retaining
  prominent UTF-8 coverage. Used qualified Result factories and imported Some
  construction, with additional union forms behind an optional disclosure. Added
  build-time Raven TextMate highlighting using pinned tokenizer dependencies;
  syntax highlighting needs no browser JavaScript. Token escaping/state tests and
  focused shorthand execution checks passed. Added the requested Google Analytics
  tag (`G-SVXYRRCEEK`) to the page head and documented the analytics script.

- Completed the independent interface implementation metadata review: integrated
  Raven `f8f7568a1` into main after 54 focused checks and the .NET 10/.NET 11
  build/run matrix passed. Recorded the primitive-signature failure caught by .NET
  execution, independently extracted dependencies from the mixed candidate, and
  remaining union-pattern/array-factory reviews. The experimental branch remains
  separate and unsynchronized; runtime-library migration stays paused.

- Completed the independent application-generic metadata review: integrated Raven
  `b5ce4023b` into main after 42 focused checks and the .NET 10/.NET 11 build/run
  matrix passed. The review also fixed ordinary .NET generic field resolution.
  Recorded validation scope, the unchanged experimental branch and remaining
  interface/union-pattern/array-factory reviews. Marked the pre-Preview-6 release
  work order historical; library migration remains paused.

- Updated the website download link after publishing Preview 6 at `5c52b4b`, with
  experimental Raven `0.1.12-neoclr.13` from `246d697bf`. All six source CI jobs,
  15 extracted-package suites, both language-server checks and isolated VSIX
  installation passed. Verified all eight uploaded asset digests. Recorded the
  release conversation and completed outcome; published release sections stay frozen.

## 0.1.0-preview.6 — 2026-09-14

Stabilization and collection APIs, with a freshly synchronized experimental Raven
SDK/VSIX. See [release notes](docs/preview-6-release-notes.md). Source and package
validation are required before publication; release assets carry the evidence.

### 2026-09-14

- Refreshed the experimental Raven .13 extension notice inventory from its actual
  production inputs, adding preserved minimatch, brace-expansion and semver licenses.

- Fixed array-interface dispatch compilation on the declared Rust 1.85 minimum.
  Moved managed-array and native-memory IL samples into the current preview sample
  set so they use its System library, separate from historical Neo library tests.
  Both are now verified and executed by the packaged direct-IL smoke check.
  The packaged match-matrix verifier also accepts the matching System library
  explicitly, avoiding accidental fallback to the historical bootstrap library.

- Recorded the clarified CLI-compatible target-model direction and release-first
  stabilization plan: no neoCLR-specific code, mappings or tests enter Raven main
  yet; semantic-model nullability and alternative backends remain future questions.
  A broader Raven audit found an attribute-emission regression missed by focused
  checks. Corrected it on Raven main at 5a67d5d4c; 39 focused checks, four NanoFramework
  builds and the modern .NET matrix passed. The audit also passed 5,493 baseline
  tests, 173/172 standalone builds/runs and 38 eligible project runs. MacCatalyst
  remains blocked by its Xcode prerequisite; no full green release gate is claimed.
  Integrated the general delegate metadata fix at Raven 35a9df494 after 55 combined
  checks passed, and removed temporary branches. Added the current release work order.

- Removed completed Raven integration branches and superseded experiment branches
  locally/remotely after checking their history is retained in main or the active
  experiment. Removed the merged local neoCLR experiment branch as well. Active
  worktrees and unrelated branches were preserved. Integrated the independently
  reviewed reference-only constructor metadata fix into Raven main at f6b4748e6;
  26 focused checks and the repository .NET 10/.NET 11 build/run matrix passed.
  Removed its temporary branch after integration as well. Subsequently integrated
  closed generic method metadata into Raven main at e14d23d32: 27 focused checks and
  the same target matrix passed. Its completed branch was removed too. Integrated
  closed generic field metadata at ea6f3383b with 22 focused checks and the matrix
  passing. Recorded the author's clarification in both repositories: general Raven
  fixes, including those benefiting .NET Framework and NanoFramework, belong on main;
  neoCLR-specific integration stays experimental. These tests do not claim execution
  on .NET Framework or NanoFramework.

- Recorded the directive to integrate general Raven fixes into Raven main while
  keeping neoCLR experiments on separate feature branches. Updated repository
  instructions and the development conversation record; retained the explicit review
  queue for remaining general metadata-emission candidates. Raven main now contains
  the reviewed numeric, binding/dispatch and namespace fixes at 8fa59a967; 47 focused
  tests and the 5,490-test broader baseline passed. Synced main into the experiment
  at 5d1022ced with 50 focused checks passing. Subsequently integrated the general
  pointer metadata-emission fix into Raven main at 521711bec: retargeted void-pointer
  signatures now emit successfully, with 53 focused checks passing. The experimental
  branch already has the implementation; migration remains paused. No neoCLR runtime
  behavior change. Subsequently integrated closed-generic reference metadata emission
  into Raven main at 031b9aaaa; 18 focused checks and the repository .NET 10/.NET 11
  build/run matrix passed. Recorded the author's release ordering: complete the
  stabilization fixes before the next release, then revisit neoIL-to-Raven migration.

- Extended explicit Raven library imports to nongeneric class/value types,
  constructors, fields/accessors, inheritance, interface/virtual calls and static/class
  method-group delegates. Added cross-assembly type visibility checks and qualified
  delegate helper identities. Regressions cover GC, value copies, duplicate type names
  across assemblies and rejected hidden types/private constructors/generic bodies.
  Source maps now qualify type records by assembly. Rebuild generated IL/maps; no Raven
  or runtime opcode change, installed-tool refresh or System-library migration.

- Fixed direct wildcard imports of namespace functions from Raven libraries by
  adding the target TopLevelAttribute metadata declaration and correcting Raven's
  marker lookup and imported-member completion on an isolated feature branch.
  Added a target namespace probe and switched the library regression to direct calls.
  Regenerate core metadata and rebuild libraries/consumers with matching Raven tools;
  installed SDK/extension artifacts remain unchanged. Library migration stays paused.

- Added bounded separate Raven library importing for nongeneric static methods,
  including intra-library namespace functions through existing CLI metadata. Dependencies
  are explicit and closure-audited; cross-assembly private calls and unsupported
  library instance bodies are rejected. Method queues now distinguish assemblies
  with reused tokens; source maps carry assembly identities and explicit output labels.
  Breaking generated-map schema: rebuild IL/maps and use the updated tools. Library
  migration stays paused; generic bodies and library-owned instance types remain open.
  Seven library, 15 application and five compiler/import checks pass. Recorded a Raven
  gap in direct wildcard discovery of namespace functions from separate DLL metadata.

- Replaced application type/field/function row-token names in the Raven importer
  with assembly/signature-based identities and collision-safe member escaping.
  Added readable metadata-to-runtime mappings to source-map sidecars and regression
  checks for declaration insertion, assembly scope, overloads and closed signatures.
  Rebuild generated application IL/maps together. This does not enable separate
  library loading; runtime-library migration and installed tools remain unchanged.

- Added ordinary Raven compiler emission followed by independent artifact import.
  Projects now select matching metadata/emission cores; regenerate editor projects
  and use the matching Raven target-contract branch and bridge. Added five direct
  compiler/import checks; all 63 saved-project checks pass. Documented host-reference
  isolation fixes, the passing .NET 10/11 matrix and remaining importer boundaries.
  Extracted general Raven fixes onto a separate review branch (5,489 baseline tests
  passed); Raven main and installed SDK/extension artifacts remain unchanged.

- Paused the Raven runtime-library migration at the author's direction, preserving
  the scalar pilot locally. Recorded the Raven branch assessment, compiler-fix review
  batches and a plan for coherent target configuration and metadata-driven importing.
  Updated the roadmap and conversation record; no Raven branch was merged and broad
  library-authoring support remains unimplemented.

- Fixed Raven string equality/inequality by declaring the missing target String
  operators and binding them to existing value equality. Interface query predicates
  now compile without changing Raven or CLI reference comparison semantics. Added
  constructed-string, captured-predicate and metadata regression coverage; importer
  stack errors now identify the method, IL offset and expected/actual types.
  Rebuild the bridge and reference metadata together; installed .12 tools are unchanged.

### 2026-09-13

- Added Raven-profile First/Last/Single predicate overloads with existing Option/
  Result outcomes, forward callback order and normal-outcome iterator disposal.
  Direct iteration avoids Where wrapper allocations; no runtime/compiler change.
  Added callback/cardinality/fault/allocation tests, metadata rejection and editor
  signature-help checks, updated samples and .NET comparisons. Regenerate core
  metadata and System together; existing source-only overloads remain supported.
  Installed .12 tools are unchanged. Recorded the decision to migrate ordinary
  library authoring to Raven after a source checkpoint, retaining neoIL for low-level
  code/tests and documenting bootstrap and generic-import groundwork.

- Built and locally installed the side-by-side Raven SDK/VSIX .12 and matching
  neoCLR collection bundle from neoCLR 6fd3729 and Raven 854cd4d3d. The isolated
  VS Code profile opens the combined order-collection demo. Eight extracted-bundle
  suites, 61 installed-editor checks and the configured Run task passed. Recorded
  payload/artifact hashes, dependency-notice checks, build evidence and instructions;
  existing demos and normal Raven launchers were preserved. No public release.

- Added a combined Raven order-collection scenario using application-defined class
  payloads, HashMap duplicate checks, ArrayList filtering, array/interface LINQ and
  Option/Result propagation. Added application checks under GC pressure and hash
  collisions, plus completion checks for inferred Order payloads. Documented shallow
  filtered membership versus shared object state and how to run with fresh matching
  core metadata/System. No runtime/compiler change or installed-tool refresh.

- Added direct-storage Raven-profile ArrayList filtering: Find/FindLast return
  Option<T>, FindIndex/FindLastIndex return Option<Int32>, Exists/TrueForAll answer
  Boolean questions, and FindAll returns an independent shallow list. Breaking:
  FindIndex no longer returns Int32/-1; regenerate core metadata and System together
  and recompile callers. Scalar scans avoid managed iterator/query allocations and
  retain the initial buffer/extent across callbacks and GC. Added IL, Raven,
  metadata/editor checks, .NET comparison and API/migration documentation. Historical
  Neo and the installed .11 SDK/extension remain unchanged.

- Added Raven-profile LINQ terminals: First and Last return Option<T>; Single
  returns Result<T,SingleError> with Empty and Multiple cases. Existing IL and
  metadata support the APIs without compiler changes. Normal outcomes dispose
  iterators, including early exits; iterator/callback/disposal faults stay terminal.
  Added propagation/pattern samples, metadata/editor checks, direct IL tests and
  .NET comparisons. Core metadata and System must be regenerated together; installed
  .11 tools are unchanged. Recorded the preference for concrete collection helpers
  to avoid query allocations while retaining LINQ composition and specialization.
  Planned ArrayList filtering and outcome review as a separate upcoming slice.

- Added an experimental Raven-profile Map/MutableMap/HashMap slice with Option
  lookup, duplicate-preserving TryAdd, Set, count and independent key snapshots.
  Managed storage, collision chains and growth use existing IL; equality/hash
  callbacks are explicit constructor arguments, with no default comparer or uniform
  null-key policy yet. Added Raven samples, signature/editor/capability checks and
  direct IL tests for GC retention, collisions, snapshots and reentrancy faults.
  Corrected an older collection test to use the inherited indexer owner and the
  target library when assembling. Regenerate core metadata and System together;
  the installed .11 tools are unchanged. Documented .NET/Rust/LanguageExt comparisons
  and remaining Map work. LINQ terminal changes remain separate.

- Implemented a bounded Raven-profile collection capability prototype: Collection
  for count/iteration, Sequence for indexed reads, MutableSequence for replacement,
  and List adding growth through Add. Arrays and ArrayList share those contracts
  through ordinary inherited-interface dispatch without wrapper allocations. Added
  array Count, samples, negative/metadata tests and editor checks. Callers must
  regenerate declarations and recompile; installed .11 tools are unchanged. Recorded
  .NET comparisons and provisional names; immutable/frozen families and variance
  remain separate. Map/dictionary work and LINQ terminal Option/Result contracts
  follow as separate slices.

- Built and installed experimental Raven SDK/VSIX 0.1.12-neoclr.11 with the generic
  array integration in an isolated local demo/profile. All seven packaged suites
  passed, including 59 saved-project cases, 28 query checks and 52 editor checks;
  the installed VSIX and configured Run task passed. Verified archive/payload hashes,
  preserved existing source files and recorded provenance and launch instructions.
  This is a local build, not a published release.

- Added the generic managed System.Array<T> shape to the Raven runtime profile,
  sharing identity/storage with ordinary array signatures and explicitly declaring
  Iterable<T>. Added Length/Item members and closed generic/member reflection.
  Raven now reads vector interfaces from the generic array reference declaration,
  including interface inheritance and element substitution, through an optional
  target mapping. Raven now unifies Array<T> and T[] in source/imported signatures,
  including indexing, nested arrays, typeof and interface member calls. Added a
  readable end-to-end sample and editor completion/hover checks.
  Indexed loops and the default .NET target are unchanged.
  Mutable arrays remain invariant; no wrapper allocation or growable List contract
  is added. Removed the profile's native array descriptor without a compatibility
  alias; added bounded NativeMemory.Alloc/Free APIs and direct IL examples. Updated
  samples and documented remaining Raven native-cast gaps and historical Neo scope.

- Added a bounded read-only view experiment in Raven and direct IL, with .NET/API
  comparisons and checks for live aliases, shared element identity, GC retention,
  access restrictions, bounds and null behavior. Recorded generic-variance prerequisites
  and reviewed the supplied collection hierarchies, including Set/Map variance and
  fixed-size array mutation, without adopting a taxonomy. No library interface or
  runtime variance support is added by that adapter experiment; mutable arrays remain
  invariant. Its generic System.Array<T> direction led to the implementation listed above.
  Prioritized that mapping, minimal collection contracts and basic implementation
  prototypes in the API plan, ahead of broader text/clock expansion.
  Audited the existing native System.Array<T> naming conflict and recorded .NET-aligned
  separation of managed arrays, native allocation and borrowed views. The subsequent
  implementation removes the native descriptor from the Raven profile; borrowed
  views remain future work.

- Clarified platform direction: retain CLR-like type categories and language ergonomics,
  develop library APIs by concrete need, and evaluate text/encoding, memory views,
  nullability metadata, callable types, runtime async and injectable clocks. Updated
  stale default-semantics policy summaries and recorded the development conversation.
  Broadened substantive API reviews to other platforms, .NET feedback and independent
  .NET libraries, with source status, counterevidence and transfer costs recorded.
  Distinguished immediate reversible prototypes from evidence-backed adoption of
  major platform contracts; retaining existing behavior remains an explicit option.
  These are design reviews and plans, not newly implemented runtime capabilities.

- Built and installed experimental Raven SDK/VSIX 0.1.12-neoclr.9 with the recent
  array, numeric and invariance fixes, then refreshed to .10 with target-aware
  array conversion diagnostics. Both isolated builds passed all six packaged suites
  and the installed order-workflow task; 701 payload hashes were verified for .9
  and 702 for .10. Verified .10 editor diagnostic recovery, recorded provenance and
  updated the local walkthrough. Existing demos, the default SDK and published
  releases are unchanged. The .9 installation retains its earlier editor behavior.

- Made mutable-array invariance an explicit runtime contract: verifier, interpreter
  and Raven importer reject differing array element-type casts, including typed-null
  casts; exact interface-to-array casts retain allocation checks. Added inheritance,
  value-element, jagged-array and interface-view regressions. Documented the deliberate
  CLR covariance difference and a future read-only projection; read-only variance is
  not implemented. New Raven target projects now disable covariance through the companion
  compiler's target-neutral option. Added editor diagnostic/recovery checks and saved-project
  coverage so invalid array conversions fail during binding. Default .NET behavior is
  unchanged; the refreshed local .10 tools include this policy, while .9 remains unchanged.

- Admitted existing numeric division, remainder and shift instructions in the Raven
  importer with operand validation. The companion Raven fix preserves unsigned
  division/remainder/right-shift semantics instead of treating high-bit values as
  negative. Added signed/unsigned/floating execution and division-fault regression
  checks. Requires refreshed experimental tools; runtime semantics and published
  artifacts are unchanged.

- Extended companion Raven fixed-width implicit numeric conversions and corrected
  unsigned-to-floating emission using existing CLI instructions. Added a saved-project
  sample checking signed, unsigned, floating and Char widening at boundaries.
  Requires the updated experimental compiler; the runtime and published tools remain
  unchanged. Native-sized conversion rules and Decimal target APIs are not added.

- Stabilized mixed numeric operator binding in the companion Raven experiment:
  binary operator candidates now require implicit conversions, fixing the previously
  recorded `ulong`/signed lookup failure. Added target regression checks for valid
  mixed comparison promotion and compile-time rejection. No runtime instruction or
  metadata changes are needed; rebuilding the experimental compiler picks up the fix.

- Added runtime vector views as Iterable<T>, preserving array identity with separate
  iterators and no sequence wrapper. Raven opts into the target interface through
  project configuration; ordinary query extensions now accept arrays, including
  reflection results. Added runtime, GC, conversion, query and completion checks.
  This fixes Preview 5's array-query limitation for the next release. Projection is
  invariant and vector-only; existing tools/artifacts remain unchanged. The Rust
  ObjectReference::concrete_type accessor now returns an owned Type so array concrete
  identity can be recovered independently of its interface view.

- Fixed Raven vector `for` loops and ordinary numeric comparisons/query predicates
  by admitting the existing CLI numeric comparison and conditional branch families
  in the importer, including short and unsigned/unordered forms. Operand and
  control-flow validation remain enforced; Boolean results use the existing CLI
  adapter. Added array-loop and signed/unsigned/floating boundary regressions. This
  fixes Preview 5 importer gaps for the next release. A companion fix on Raven's
  experiment branch corrects unsigned and NaN comparison emission for both targets;
  refreshed tools are required. Published artifacts remain unchanged.

- Published Preview 5 at c76ee57 with eight verified assets after all six exact-source
  CI jobs and thirteen package suites passed. Recorded publication evidence, match
  checks and artifact provenance; published release notes remain unchanged.

## 0.1.0-preview.5 — 2026-09-13

Application and query preview: see [release notes](docs/preview-5-release-notes.md).
Includes the experimental Raven 0.1.12-neoclr.8 SDK and VSIX.

### 2026-09-13

- Extended the order workflow with a deferred pending-order query and materialized
  name summary, exercising queries, interfaces, Result/Option patterns and file I/O
  together on the installed .8 tools. Added failed-write and all-saved summary checks,
  prepared a separate local demo, and corrected stale preview-readiness descriptions.
  This sample update does not change the archived .8 build or its recorded results.

- Built and installed local Raven SDK/VS Code extension 0.1.12-neoclr.8 with the
  pattern, query and custom-iterator fixes. Prepared an isolated query demo and
  recorded six passing packaged suites and artifact hashes. Preserved existing
  demos and the default SDK; this build is not published.

- Enabled custom Raven Iterable/Iterator implementations in the query pipeline:
  close generic MethodImpl signatures before bridge validation and promote the
  earlier compiler repro to an executable sample. The companion Raven fix normalizes
  target interface references; ordinary CLR behavior is covered separately. Updated
  query documentation and regression coverage; cleanup limitations still apply.

- Added prototype Raven-target `System.Linq.Enumerable` extensions: deferred generic
  Where/Select and eager ToList returning ArrayList. Ordinary NeoIL classes retain
  source/callback state, cache current values and dispose upstream on exhaustion or
  explicit disposal. Added execution, GC, lifetime, negative and completion checks;
  documented cleanup, mutation and compiler boundaries. Corrected Void projection
  for generic method arguments. No query opcode, Raven compiler change, legacy Neo
  update or archived toolchain replacement.

- Enabled nongeneric Raven application extension methods by completing their
  generated marker metadata dependencies. Added readable value/reference/interface
  receiver and callback examples, execution and completion checks, and explicit
  rejection coverage for generic application extensions and executable marker
  construction. No new runtime opcode or Raven compiler change; archived toolchains
  remain unchanged. Documented the generic bridge boundary before the LINQ slice.

- Added Raven union payload destructuring through the existing extraction and
  payload APIs: imported `Ok(let text)` / `Error(let error)`, target-typed cases,
  and explicit generic case patterns. Updated the order workflow and added a
  readable pattern sample and executable compatibility checks. The bridge tracks
  unconditional deconstruction outputs and nested pattern assignment; the source
  experiment requires Raven 04c953d67 and refreshed declaration metadata, not
  archived .7 tools. Prepared a separate source-backed local demo and verified
  payload completion/hover with its updated language server.

- Added target completion checks inside constructors and installed a patched Raven
  language server after reproducing missing Int32 static members in constructor
  bodies. The general compiler fix is isolated on Raven at 55c0f7ef5; archived tools
  remain unchanged.

- Recorded the future modern-library API direction and mockable-clock investigation,
  comparing .NET TimeProvider with narrower library contracts. No clock behavior changed.

- Extended experimental bundle packaging with the application and order-workflow
  validation scripts and documented how to try the new samples. Updated bundle
  capability boundaries; package validation now includes dispatch, captures and GC.
  Built and installed local Raven SDK/extension 0.1.12-neoclr.7 with a separate demo
  and VS Code environment. All twelve package suites and installed-server checks pass;
  recorded artifact hashes and usage instructions. No new release was published.

- Extended runtime delegate binding to nominal heap receivers and the Raven source
  bridge to instance/virtual/interface method groups and shared or escaping lambda
  captures through Func, including Func<Void>. Added binding-time null checks,
  closure GC tests and ordinary CLI add/sub/mul admission; documented adapter costs
  and remaining boundaries. Raven's experimental method-group emission now preserves
  virtual/interface dispatch and explicit base selection. Expanded application sample
  blocks for readability and recorded that preference in the repository workflow.

- Added a Raven order-workflow application with domain classes, an application
  interface, ArrayList storage, Option lookup and Result<Void, FileWriteError>
  propagation. Isolated checks verify persistence and unchanged order/report state
  after rejected writes; documented source-toolchain setup and current limitations.

- The Raven source bridge now imports ordinary application classes/value types,
  fields, constructors and instance accessors/methods, including ArrayList storage.
  Extended this to application class interfaces, abstract/virtual inheritance and
  nominal constructor chaining; runtime checks preserve concrete dispatch and reject
  invalid chaining. Added runtime and Raven examples/checks for the contracts.
  Added class-alias/value-copy, saved-source, mapping and rejection checks. Recorded
  Raven targeting findings and the future generic-Void async requirement; the source
  bridge requires the updated experimental Raven branch, not the published Preview 4 SDK.

- Recorded the implementation-language discussion, distinguishing Rust build
  dependencies and implementation memory safety from neoCLR's guest GC; language
  migration remains an open question, with no runtime changes.

- Published Preview 4 at c135659 with eight runtime/Raven/source/validation assets
  after all six exact-source CI jobs and ten package suites passed. Updated the
  website download link, current installation guide and published validation record;
  published release notes and the Preview 4 changelog section remain frozen.

## 0.1.0-preview.4 — 2026-09-13

Runtime/Raven preview: see [release notes](docs/preview-4-release-notes.md) for the
prebuilt macOS arm64 distribution, matching Raven tools, migration and limitations.

### 2026-09-13

- Updated release smoke validation for the guest-only stdout contract: request
  --show-result explicitly and verify successful completion on stderr. Retained
  environment, clock and file content assertions without the retired output suffix.

- Added a responsive project website explaining neoCLR, Raven integration, runtime
  APIs and preview limits, with excerpts built from executable samples. Added a
  separate GitHub Pages workflow: pull requests build/check; main and manual runs
  publish through a restricted deployment job. Documented local preview and hosting.
  Enabled Pages and verified the successful deployment and live asset contents at
  https://marinasundstrom.github.io/neoCLR/.

- Built and locally validated the runtime/Raven candidate with direct neoIL samples,
  source archive, toolchain notices and matching .6 Raven SDK/VSIX. Ten extracted
  package suites pass; all runtime test executables pass after the recorded stale-test
  corrections, with strict Clippy/format checks. Recorded revisions, asset hashes and
  remaining final-version/CI/publication gates; refreshed the prepared local demo.

- Simplified the existing virtual-call admission condition and empty verifier match
  arms for strict Clippy validation while preserving the supported receiver modes.
  Updated stale initialization tests and String API prose to the implemented typed-null
  String default; non-defaultable Error/Value rejection remains covered. Updated
  legacy Neo test expectations for that shared runtime behavior without changing the
  Neo frontend.

- Preserved version-specific notices for 26 NuGet and five bundled JavaScript
  dependencies in the experimental Raven tools, with source/hash provenance and
  a dependency-manifest coverage checker. Future runtime bundles carry these texts;
  separate SDK/VSIX assets require the companion attribution archive.

- Accepted ordinary lowercase void returns in neoIL using the existing empty-stack
  calling convention, and updated preview samples. System.Void remains a unit type
  in generic/value contexts; legacy uppercase Void and noresult behavior is retained.
  Added return-stack and generic-unit regressions; binary CLI encoding is unchanged.

- Added a runtime/Raven release walkthrough and direct neoIL demonstrations for
  class aliasing/value copies and Result<Void, Error>, verified against the adapted
  library. Bundles now include these samples and their repeatable output check.
  Marked old Neo language guides as historical and recorded the two-part release
  direction; existing local archives remain unchanged pending a fresh candidate.

- Updated the repository introduction and roadmap to present the current Raven/CLR
  type-category direction and completed runtime API milestone, separating the legacy
  Neo experiment. Future portable bundles now include both repositories' license and
  notice files; existing .6 assets are unchanged. Binary dependency attribution review
  remains a publication check.

- Added published-bridge execution and a checkout-independent experimental bundle
  builder with pinned metadata/library/server, samples, setup and file-hash provenance.
  Shared validation scripts accept either source or published toolchains. All nine
  suites passed outside both checkouts. Installed experimental SDK/VSIX .6 and
  verified its server and saved demo; recorded artifact hashes and setup instructions.
  This is a local macOS arm64 build, not a published release.

- Completed the source-by-source existing API audit, including runtime-service
  callers, signature markers and explicit importer limits. Preserved writable union
  case payloads through Value setters and verified case-copy independence; requires
  the Raven value-property receiver fix, included in the validated .6 toolchain.

- Generalized Raven collection/union payload mapping to existing references, nested
  collections, interfaces, unions and delegates, with a nesting bound. Expanded
  managed-array access and ForEach to admitted defaultable elements, checking opcode
  widths/signedness and preserving canonical Boolean joins. Added executable shapes
  and exposed the opaque System.Value metadata identity.

- Added internal-library array.reserve allocation with ordinary managed-array
  identity and checked unreadable slots. Adapted ArrayList capacity now supports
  elements without defaults without inventing union cases; ordinary newarr retains
  default initialization. Added publication and direct/indirect-read regressions.

- Projected Equatable, Comparable, Clonable and Closable contracts into Raven metadata
  and calls. Restored Type equality conformance and value/interface implementations;
  added boxed primitive/calendar, String and class-reference samples and checks.
  Clonable/Closable have no existing concrete library implementations to demonstrate.

- Added bounded CLR-style box allocation and nominal interface dispatch into value
  payloads, preserving value-copy independence, alias identity and managed lifetime.
  Intrinsic String interface views use managed handles. Added GC, escape and invalid
  operation tests; generic reference/nullable boxing and unboxing remain separate.

- Exposed unsafe native Array<T> buffers through Raven for primitive elements,
  including allocation/views, fields, indexers, addresses and explicit release.
  Added saved-source success, bounds/lifetime faults, metadata checks and completion.
  Int32 pointer reads/writes are admitted; general native layouts remain bounded.

- Replaced provisional BindingFlags wrapper metadata with a standard Int32-backed
  CLI enum, literals and FlagsAttribute. Raven uses casts and bitwise operators
  instead of the wrapper factories/combinators; typed adapters retain runtime
  nominal storage. Added flags samples, metadata rejection and completion checks.

- Exposed Raven type tokens, reflection queries, descriptor properties and managed
  snapshot arrays, including base-class views and Option<Type/MethodInfo> results.
  Added full public-getter/query samples, signature checks and editor completion.
  BindingFlags currently projects its factory/combinator API as a value wrapper;
  true enum metadata/operator syntax remains a following slice.

- Migrated reflection snapshot storage in the Raven runtime profile to ordinary
  Type/descriptor classes and the MemberInfo hierarchy. Trusted query factories
  allocate nested snapshots under heap limits and retain them through GC. Source
  metadata/import projection remains separate; added runtime regression coverage.

- Added ordinary class ancestry, implicit base-reference conversions and checked
  downcasts, preserving allocation identity and inherited fields. Allow abstract
  class declarations without abstract members; reject mixed storage categories.
  Class constructor chaining and virtual dispatch remain separate groundwork.

- Projected Array.ForEach for managed Int32/String arrays and static Func<T,Void>
  callbacks. Resolve closed method-generic signatures against target metadata,
  retaining the existing callback algorithm and ordinary no-result API boundary.

- Admit no-result static generic IL methods while retaining the distinction between
  generic Void payloads and absent call results. Added execution and invalid-return
  regressions; generic instance-class methods remain outside this subset.

- Projected the five Func arities for static Raven callbacks, including named
  Func<Void> completion, and exposed ArrayList.FindIndex/Exists/Find. Preserve
  generic Void return context and the adapted no-result Dispose calls. Added
  executable callbacks and predicates; capturing/instance callbacks remain outside
  this importer slice. Requires the corresponding Raven experiment compiler fixes.

- Projected Boolean.CompareTo and Raven canonical Boolean literals/equality across
  locals, parameters, returns and generic API calls. Typed adapters preserve argument
  order and output contracts while retaining the runtime Boolean representation.
  Added saved-source coverage; arbitrary CLI Boolean bit patterns remain unsupported.

- Extended Raven collection bindings beyond Int32 to admitted primitive, String,
  calendar and empty-error elements, and exposed ArrayList.Copy. Added growth,
  aliasing, independent-copy, iteration, completion and invariant-signature checks.
  Elements without runtime defaults and predicate/delegate APIs remain separate.

- Projected Console.ReadByte and all three existing Environment APIs through Raven,
  including nested Result/Option propagation and managed string argument arrays.
  Added controlled environment/argument/byte-input tests, signature rejection and
  completion checks. Argument snapshots currently copy through an adapter; installed
  SDK/extension packages have not been refreshed.

- Added typed-null String defaults for managed locals, constructed fields and array
  elements, aligning string-array initialization with CLR expectations. This
  replaces the previous default-initialization fault; intrinsic string receiver
  representation and native pointer rules are unchanged. Updated generic constructor
  coverage to verify String initialization after the newly supported typed-null default.

- Generalized Raven Option/Result bindings for admitted primitive, string, calendar,
  error and nested union payloads. Exposed factories, predicates, checked case
  access and extraction/propagation members, retaining guarded-output validation.
  Handle both Raven pattern branch forms; added nested execution, rejection and
  completion checks. Arbitrary application/reference payloads remain unsupported.

- Projected existing error-value constructors, predicates, checked accessors and
  ToString APIs, including all 23 nested error cases and System.Error message APIs.
  Added complete case round-trips, wrong-case/default rejection checks and completion.
  Empty values have valid defaults; carriers/messages require initialization. Existing
  description differences are preserved; no Exception hierarchy or runtime change.

- Projected all 24 current public Date/Time/LocalDateTime/Clock methods/accessors
  through Raven, including validated Result factories and value-payload propagation.
  Added fixed calendar/tick cases, live host-clock validation, completion/signature
  checks and usage docs. Defaults use initobj with private storage preserved; no
  new globalization, formatting, runtime or installed-tool behavior is implied.

- Added Raven primitive storage/stack normalization, ordinary numeric conversions,
  concrete comparisons and all 16 existing Char classifiers. Added boundary/Unicode
  samples and completion/signature coverage. Requires the isolated Raven integral-cast
  fix `26907410f` (also tested on CLR output). Boolean normalization, general interface
  dispatch and arbitrary generic payloads remain pending; runtime and installed tools
  are unchanged.

- Added bounded Raven Double literal/slot/receiver import and projected all 15
  existing Double Math methods plus Double.CompareTo. Added all-method, rounding and
  NaN examples, signature checks and Math completion coverage. All 20 current Math
  methods are projected; conversions, floating operator lowering and formatting are
  not implied. No runtime, Raven compiler or installed SDK/VSIX changes.

- Exposed the existing Int32 Math.Clamp API through Raven with Result matching and
  propagation for InvalidRangeError. Added inclusive/equal/reversed-bound and Int32
  extreme-value examples plus completion coverage. The five current Int32 Math
  methods are projected; floating-point Math and the error's constructor/ToString
  remain pending. Runtime semantics and installed tools are unchanged.

- Exposed Int32.Equals/CompareTo/ToString through Raven, adapting ordinary managed
  receivers to the existing readonly/snapshot runtime contracts. Added bounded
  Int32 parameter-address import for these receivers, local/parameter examples,
  signature checks and completion coverage. No boxing, runtime or Raven compiler
  changes; general byref/constrained calls and interface projection remain separate.

- Projected the existing Int32.Divide Result API into Raven, including division-by-zero
  and overflow predicates, matching and propagation. Added signed/boundary examples,
  checked-signature coverage and completion. The helper remains an experimental
  API extension rather than a matching .NET member; arithmetic/runtime semantics and
  installed tools are unchanged.

- Exposed Int32.Parse through Raven with its existing Result<int,Int32ParseError>
  contract, typed matches, propagation and error predicates. Extended the shared
  carrier catalog without changing existing Ok<int> handling. Added boundary and
  unsafe-extraction checks, a saved-project sample and completion coverage. Documented
  disabling Raven's .NET parsing projections for this target. No parser semantics,
  Raven compiler or installed SDK/VSIX changes; other numeric/error APIs remain pending.

- Projected existing Path.Combine/GetFileName through the Raven target with a shared
  declaration/binding catalog, lexical-path sample, signature rejection checks and
  editor completion coverage. Host path semantics remain unchanged; Windows execution
  remains unverified. No Raven compiler or installed SDK/VSIX changes.

- Recorded the author's POC framing: visible rough edges, .NET familiarity,
  deliberate differences and unfinished API choices are useful material for evaluating
  the experiment. Updated preview/API design guidance while keeping support and
  test evidence explicit. Clarified the invitation for API feedback and the expected
  compatibility cost of Result-based error contracts.

- Exposed nine existing String methods through Raven: concatenation, ordinal
  comparison/search, equality, empty checks, UTF-8 byte counts and slicing. A shared member
  catalog generates declarations and checked bindings; guest adapters supply readonly
  receivers. Added readable and Unicode-boundary samples, invalid-call checks and
  editor completion coverage. Slicing shares a Result/error catalog with file APIs;
  range/boundary errors propagate and unsafe extraction is rejected. Error-carrier
  case constructors/accessors and ToString remain pending. No runtime semantics or
  Raven compiler changes, and no SDK/VSIX rebuild in these slices.

- Shared Raven bridge signature substitution and checking across file and collection
  calls, including constructors. Nested type arguments now use one recursive path;
  catalogs retain explicit API admission and Void stays distinct from no-result
  returns. Added 18 focused signature checks and verified existing execution paths.
  This is groundwork for broader runtime API projection, not additional API coverage;
  no Raven compiler or installed package changes.

### 2026-09-12

- Expanded the Raven POC plan to cover all existing public runtime APIs while
  retaining explicit compiler/CLR feature limits. Added a regenerable source
  declaration inventory and a coverage work plan; the inventory does not claim
  complete executable support. Documented a repeatable experimental distribution
  procedure with separately packaged Raven SDK/VSIX builds, focused integration
  checks and provenance instead of Raven's full release cycle. Recorded the author's
  directions and the remaining standalone bridge distribution gap; no release made.

- Projected bounded UTF-8 File.ReadAllText/WriteAllText into Raven with Result error
  predicates, typed matches and success/error propagation. The importer preserves
  conditional output initialization without inventing default error cases. Added
  temporary-file round-trip and rejection checks, saved-project execution and editor
  completion coverage. Extended named Void handling to generic return metadata;
  Raven's corresponding loader fix is isolated on its experimental branch. These
  calls require regenerated declarations and the updated compiler; installed .4
  packages are not claimed to contain this slice.

- Added an executable Raven match support matrix and readable Result/Option/Void
  samples. Six admitted fixtures verify and run; seven record compiler rejections,
  including exhaustiveness, deconstruction and arm-return limits. The bridge now
  admits definitely assigned string locals for match results and rejects uninitialized
  reads. Generalized the existing terminal-failure message beyond propagation.
  Documented two observations that need reconciliation with Raven's language docs;
  no Raven compiler or default .NET behavior changed.

- Extended Raven propagation to Option<Int32> and Result<Void,OverflowError>, including
  early absence/error returns and completion without a payload. Recorded Void's unit
  type semantics while retaining CLR no-result calls. Validate encoded Void storage
  separately from Cecil member resolution; reject CLI VOID markers in value slots.
  Added propagation workflow and editor checks. Raven's discarded-Void fix remains on
  its isolated target branch; ordinary .NET behavior is unchanged. Built and installed
  local Raven SDK/VS Code extension 0.1.12-neoclr.4; the prepared propagation demo runs,
  and installed-server checks verify target completion, constructors and inferred output
  types. Updated local setup instructions; these are not published release artifacts.

- Replaced ArrayList.Allocate with parameterless and initial-capacity constructors,
  including the Raven class projection. Both start empty; default capacity is zero,
  storage grows on demand and negative capacity faults. Updated callers and samples;
  no compatibility alias is retained. Raven also exposes Capacity for inspection.

- Added the runtime-library Propagatable<TSelf,TOutput,TResidual> extraction interface,
  implemented by Result and Option with readonly receivers and conditional output
  initialization. Added carrier FromResidual factories, including Option's Void residual.
  The static factory is provisionally outside the interface because static abstract
  interface members are not implemented. Documented that enforcement boundary and the
  bounded Raven target work. The bridge now runs `Result<int, OverflowError>` `?`
  with success extraction and early error return using target-selected Propagatable
  metadata, checked adapters and terminal faults for invalid carriers. Added repeatable
  execution and malformed-contract rejection checks. Option/Void propagation and an
  updated installed SDK remain pending. Four new runtime
  tests cover channels, Void, reconstruction and invalid output reads.

- Fixed pre-merge generic Clonable and Neo calculator regressions. Assembly field fixups
  now resolve layout with available constraints before complete linked validation;
  generic contract walks recognize repeated constructed types instead of recursively
  revisiting self-referential contracts. Constraint failures remain enforced. Primitive
  conversion syntax no longer gets intercepted by library constructor lookup. These
  restore existing behavior; no Raven target or library contract changes. Updated stale
  native-helper inventory assertions to include the existing enum helpers. Recorded a
  bounded preview acceptance matrix, including error flow, text files, date/time and
  target-aware VS Code completion. All 162 integration suites pass after the regression
  fixes and targeted reruns; unit/doc checks and the prepared Raven demo also pass.

- Normal CLI runs now emit only guest output; removed the automatic return-value suffix
  such as `=> Void`. Use `run --show-result` after the input path for opt-in return-value
  diagnostics on stderr. Scripts expecting the former stdout suffix must be updated.
  Updated CLI and Raven demo checks, and recorded propagation/text-file APIs as release
  requirements with reflection coverage desirable.

- Prepared local Raven SDK/VS Code extension 0.1.12-neoclr.3 from the isolated Raven
  feature branch, with a fresh combined demo workspace and updated try-it instructions.
  The SDK is installed alongside the default tools; the workspace selects the new server.
  Recorded plans for a NeoCLR preview bundle including matching Raven tools, runtime-owned
  propagation support and verified match syntax. Pre-merge testing found a Clonable generic
  assembly-resolution regression; main merge and public release remain pending.

- Fixed the Raven saved-project importer rejecting the advertised Int32 Math.Min,
  Math.Max and Math.Sign APIs. It now reuses the target declaration catalog with
  resolved-signature checks and executes the existing runtime methods. Added a sample
  covering integer limits, equal operands and all Sign outcomes to both project profiles.
  No runtime semantics, opcodes or Raven source changes.

- Combined the Raven collection declaration profile with Result/Option/Void and added
  a product-workflow demo using the existing adapted runtime library. Project checks
  cover the combined workflow, standalone unions, value-carrier copying, class/array
  aliasing and iteration. Editor checks now require both collection and union APIs.
  Regenerate older collection demo folders for the added declarations. No runtime
  opcode or Raven source changes; the compiler stays on its separate experiment branch.
  Documented the author's fundamental-demo scope and separate future Raven evaluation.

- Added a Raven for-loop demo using the feature-branch RuntimeIterationContract option
  to bind Iterable/Iterator/GetIterator rather than .NET enumerable names. The existing
  collection import profile executes normal completion, break and return; output is
  41, 42, 41, 41. Extended target selection to evaluated Raven project properties and the saved-project
  runner, which generates and verifies against the required collection library profile.
  Added a collection editor setup and actual LSP completion/loop-type checks; collection
  and existing Result/Option/Void project tests pass, including stale-output protection.
  Recorded missing automatic iterator Dispose and the proposed finally/defer mechanism
  as future work, per author direction. No cleanup behavior or Neo source changes are
  included. Future NeoCLR cleanup must leave default CLR support unchanged. Raven's
  four project configuration tests and nine related tests pass in this follow-up.

- Added CLR-compatible callvirt admission for nonvirtual nominal class instance methods,
  including closed generic classes and no-result calls. Null receivers fault before
  method entry; wrong types and byrefs to reference slots are rejected. This supports
  Raven's existing emitted calls without rewriting the instruction. Four new regressions
  and 49 related tests pass; nominal inheritance/virtual slots remain separate work.

- Added an isolated Raven collection library profile generated from existing System
  algorithms: nominal ArrayList/state/iterator classes, ordinary interface receivers,
  managed array buffers and no-result mutation/disposal. Runtime tests cover aliasing,
  growth, Copy, iterator GC retention, disposal and invalid accesses (seven new tests
  and 35 related tests pass). Neo and the default
  library remain unchanged. The profile requires defaultable element types and excludes
  predicate/delegate helpers. Documented how to generate and run it. Recorded proposed target-specific Raven language contracts
  for renamed iteration/disposal APIs while preserving the default .NET target; that
  project configuration is now implemented as described above.
  Follow-up admits two Raven Int32
  collection programs through an explicit import profile: ordinary class/interface
  calls and upcasts execute the adapted library, including iteration, growth, returned
  aliases and indexers. Six malformed/unsupported inputs reject; a null-default fixture
  verifies and faults at invocation. Added execution evidence and reproduction commands.
  Existing array and eight-program Result/Option/Void regression probes still pass.
  Raven and default VS Code project-profile configuration remain unchanged.

- Ran Raven-emitted static programs on neoCLR against the real System.Console through
  a bounded neoCLR-owned Cecil import bridge, with no Raven changes. Added verified
  empty/nested-call and Int32/local cases, invalid-input rejection checks, provenance
  maps and a reproducible runtime verification script. Direct PE loading and broader
  library/class-program support remain unfinished. Recorded staged VS Code development
  support as planned work, with target-aware Raven code completion as the editor MVP.
  Expanded the shared declaration/binding catalog to Math.Min/Max/Sign and integer
  Console.WriteLine, with an editable Raven sample and five verified runtime programs.
  Raven's completion API now passes checks against those target declarations without
  host API leakage; VS Code integration remains planned. Added a real Result/Option
  demonstration to MVP acceptance criteria; generic union import is not yet supported.
  Added an isolated Result metadata/type-pattern probe and incorrect-argument rejection.
  Recorded Raven's host-type-resolution emission blocker; the union sample binds but
  did not emit or execute in that slice. Kept probe-only declarations outside admitted
  runtime APIs. Follow-up now emits the sample after a feature-branch Raven fix for
  target metadata types and generic/byref member signatures. Corrected the probe's
  missing union-recognition Value property; verifies actual TryGetValue lowering instead
  of ordinary type tests. Added a bounded Result importer with control-flow stack and
  definite-assignment checks, case defaults, value-receiver/out-case adapters, and
  four malformed-input rejection probes. Raven's Math.Abs sample now executes against
  the real neoCLR System library and prints success (42) and overflow (Overflow).
  Added it as the sixth runtime regression program, with provenance and run instructions.
  Observable default Result carriers remain unsupported and are rejected. Recorded the
  POC priorities: own library, Result/Option, generic Void, and Raven VS Code completion.
  Extended the shared union import profile to Option<Int32>, case/carrier construction,
  supported case initialization and integer equality. A Raven price lookup now uses
  the real Option library and prints 42 and Product not found; seven runtime programs
  and three additional Option rejection probes cover the slice. Raven constructor
  emission support remains isolated on its feature branch. Added the eighth runtime
  demo: Raven Option<System.Void> constructs and matches Some(()) and None through
  a neoCLR-owned metadata projection, without further Raven changes. Ordinary no-result
  returns are unchanged; generic Void uses a named target type and the VM's existing
  inhabited Void marker. Added binary signature checks, three rejection probes, saved
  imports and run instructions. This is a bounded target adapter, not general CLI
  generic compatibility or a zero-stack optimization. Added project-backed editing
  with an explicit metadata core, reproducible LSP completion checks, a local VSIX build
  and installation walkthrough, plus a locally installed SDK alongside existing builds.
  Verified target Math suggestions in VS Code itself;
  normal Raven Build/Run buttons do not yet invoke the neoCLR import pipeline. Added
  dedicated VS Code build/run tasks using saved project sources, target emission and
  verifier-gated execution. Each build has fresh outputs; failed compilation/import
  cannot run stale IL. Verified Result/Option/Void, changed output and rejection cases.
  Initially recorded propagation as next; subsequent author direction puts the
  interface contract first and defers propagation. Closed the bounded Raven POC as
  an experiment checkpoint, with acceptance evidence, limitations and pinned Raven
  dependency for local tag milestone/raven-poc-2026-09-12; no preview release published.
  Clarified the broader desired demo: Raven should consume an adapted version of the
  existing library used by Neo, with shared implementations and matching declarations.
  Proposed existing collection/iteration contracts as the interface milestone scenario.
  Added the candidate interface contract and isolated Raven collection metadata probe:
  inherited callvirt emission and wrong-type diagnostics pass, while runtime admission
  remains deliberately rejected. Documented nominal interface dispatch and generic-class
  support as prerequisites to adapting the actual ArrayList/iterator library. Added
  non-generic nominal-class interface conformance, checked castclass views, inherited
  callvirt dispatch and identity-preserving GC-rooted interface fields/returns. Supports
  no-result interface contracts with exact return-mode matching and typed null faults.
  Ten new regressions and 108 related tests pass. Implicit storage
  conversions and collection import/adaptation remain unfinished; legacy borrowed
  interface behavior is retained. Added closed generic nominal classes
  using existing owner substitution for constructors, fields and interface dispatch.
  Nine additional regressions cover distinct instantiations, constraints, Void payloads,
  typed defaults and nested references surviving GC/artifact round trips; 96 focused
  tests pass for this generic-class slice. Constructor field-default
  limits (including String/arrays) remain; collection import/adaptation is still pending.
  Added ordinary interface-reference array elements and indirect interface-slot loads/
  rebinding, with typed null defaults, identity/GC retention and invariant element checks.
  Ten new regressions and 90 related tests pass.
  Added ordinary `arrayref<T>` storage for CLI-style arrays, including typed null field
  defaults, generic constructor allocation, jagged arrays and GC retention. Breaking
  neoIL change: `newarr` now returns an ordinary array reference; migrate old `T[]&`
  allocations to `array.new` or recompile Neo sources. Neo source behavior is unchanged.
  Runtime metadata identities distinguish both array forms; 13 new regressions and
  127 related tests pass. Added bounded Raven Int32 vector import and an executable
  alias/mutation/length sample, reusing Raven's standard IL and adding Array.Length to
  target declarations. Five malformed-IL imports reject without output; a typed-null
  local fixture verifies and faults at execution. Result/Option/Void compatibility checks
  still pass. Documented reproduction and target-core refresh instructions. Broader array
  import, String defaults and collection adaptation remain pending. Added implicit
  nominal-class/interface upcasts at runtime assignments, calls, returns, fields and
  array/indirect stores, preserving identity, typed nulls and GC roots. Byrefs and whole
  array types remain invariant; downcasts still require castclass. Eight new regressions
  and 104 related tests pass. Raven class/interface
  import and adapted collection execution are still pending. Clarified the target
  architecture: retire value-by-default/explicit-reference ordinary semantics in favor
  of .NET type categories, including reference-type arrays. Documented the existing
  Neo array split as transitional and planned its migration; no runtime behavior changes
  are made by this clarification. Recorded .NET semantics as the baseline unless a
  concrete improvement justifies divergence; Result handles recoverable failures and
  terminal host faults do not form a guest Exception class hierarchy. Corrected stale
  API/migration and CLI Void policy text; current Fault representation already follows
  the host-diagnostic model. Documentation only; cleanup/containment contracts remain open.
  Corrected experiment scope after author clarification: adapt neoCLR and its library
  for Raven; leave Neo outside this exercise. Removed the uncommitted Neo migration
  attempt and its helper instructions, and revised migration plans accordingly. Existing
  committed runtime and Raven behavior is unchanged.

- Began nominal class reference semantics: `.type class` uses managed heap handles in
  ordinary class-typed storage, while managed byrefs still address slots. Assignment,
  static calls, fields, return values and GC preserve object identity; existing value
  records still copy contents. Added alias/value-copy and .NET comparison tests.
  Added class instance constructors/direct calls with no-result bodies and constructor
  results supplied by newobj; class field stores now consume operands without a result
  (remove the previous trailing pop). GC retains objects throughout construction.
  Decodes standard constructor/field tokens and adds a runnable Console example.
  Added typed null defaults for nominal class fields/slots, alias-safe initobj reset,
  null comparison, null field-access faults and GC handling. Uninitialized slots remain
  distinct. Inheritance and Raven binary execution remain unfinished;
  legacy value/byref field-store behavior and System API returns still need migration.

- Added a bounded standard CIL code-stream decoder for the static-call experiment,
  preserving byte offsets and tokens and rejecting unsupported/truncated instructions.
  Added bounded PE32 CLI container and tiny/fat method-body inspection with .NET fixture
  comparisons; rejects truncated/overlapping ranges and unsupported header modes.
  Added a reproducible .NET-emitted byte fixture with matching neoCLR execution through
  test-only binding; PE loading and production token resolution remain planned.
  Clarified that real Void type participation need not occupy an evaluation-stack slot;
  the existing generic representation has not yet been migrated. Updated the experiment
  order to prioritize .NET value/reference type semantics and a useful Raven scenario,
  while retaining standard external metadata and instructions wherever possible.

- Added explicit no-result metadata and assembly syntax for static, non-generic IL
  methods. The existing call/ret instructions now preserve CLI-style empty results;
  verifier/runtime reject invalid stacks and unsupported return-mode combinations.
  Existing inhabited Void behavior remains the default. Hosting and reachability
  expose the return mode; generic Void storage/returns remain intact. Documented the
  separate declared-type/return-convention layers and runtime-async design comparison;
  binary loading and broader dispatch support remain planned. Recorded intended
  later alignment with .NET value/reference type semantics; no type-model migration
  is included in this slice.

- Added the first Raven integration map, pinned to the inspected compiler revision.
  Identified existing core-library retargeting, reflection/PE emission dependencies,
  framework discovery and semantic gaps in Void/Unit and error projections. Documented
  the next minimal-target probes. Added a reproducible emission probe and minimal
  contract map: target Console binding succeeds, while omitted-library resolution
  exposes host fallback and core declarations remain incomplete. Captured metadata,
  diagnostics and call-boundary alternatives; binary execution is not yet claimed.
  Added explicit-only metadata dependency auditing with positive and
  negative fixtures, and selected CLI PE reuse for the bounded binary experiment.
  Added probe coverage for Raven's opt-in explicit-only metadata import API: Console
  binds when supplied and is unavailable when omitted, even after host-mode binding.
  Added a minimal core reference assembly and core-only Console, empty/nested-call
  and Int32-return emission probes, with dependency and negative binding checks.
  Corrected application AssemblyRef inventories to exclude references synthesized
  by Cecil during inspection. Runtime loading/translation and executable System
  binding remain planned. Recorded the workflow requirement to keep Raven work
  isolated on a feature branch.

- Documented the Raven target/binary-artifact experiment on its own branch: inspect
  the existing compiler, choose an evidence-backed format/library contract, and
  compile/run HelloWorld against neoCLR's own System library. Recorded deliberate
  Result-based migration differences and acceptance criteria; no backend or binary
  loader implementation is included yet.

- Added a bounded reference-defaults architecture evaluation comparing the order
  workflow under current, shared type-default and language-only models. Includes
  illustrative source/lowering, .NET comparison, migration costs and a proposed
  compiler/import experiment. Validated current behavior with 22 focused tests,
  emitted IL and the pinned .NET baseline; no runtime or compiler semantics changed.
  Clarified after review that runtime improvement and useful CLR compatibility lead
  the evaluation; Neo demonstrates/tests the contracts. A runtime-contract assessment
  precedes the optional compiler experiment, with compatibility scope still open.

- Recorded evaluation criteria centered on familiar developer experience, possible
  type-level reference/value defaults, Result/Option APIs, Faults, nullable support
  and language/runtime boundaries. Identified migration questions without selecting
  or implementing an architecture change.

### 2026-09-09

- Added a development conversation record of author directions/questions, assistant
  proposals, subsequent decisions and actions/outcomes, in one chronological document covering
  the original shared chat from the founding brief through Preview 1. Earlier
  continuation notes explicitly identify unavailable assistant replies; recent exchanges retain attribution and open issues.
  Added workflow instructions to maintain it as a minutes-like record, separately from
  the implementation changelog, without inventing dates or missing conversation. Expanded the
  timeline with available founding exchanges on memory policy, signature/name mapping,
  ordinary union cases, output-reference corrections, equality, release scope, Windows
  CI diagnosis and verified Preview 1 publication. Preserved attribution, the distinction
  between unavailable replies and implementation evidence, and later changes of direction.
  Recorded the subsequent discussion of reference defaults and performance-oriented
  complexity, with .NET/Valhalla sources and an explicit distinction between runtime
  uniformity and demonstrated usability. Recorded the author's exploration
  of ordinary class references and opt-in value behavior, separating it from the
  assistant's metadata proposal, and linked existing evaluation documents. The author
  clarified that this is not a selected direction; the current model remains in place. No runtime
  or language contract migration is implemented by this documentation change. Extended
  the exploration with transparent class usage, value-design intent and Java wrapper
  evolution, distinguishing lifetime, identity and preview status. Added the clarification
  that lightweight values need not be short-lived and the preference for potentially
  inline value classes over a separate struct concept, without selecting a migration.
  Recorded the follow-up separation of storage, mutation, identity and equality using
  C# records as a comparison. Recorded the exploration of runtime contracts versus
  conventions, tentative inline/value metadata, and the clarified priority of
  object-like primitive and user-defined types with familiar .NET usage. Proposed
  syntax and evaluation work remain explicitly unselected and unimplemented.

- Added emit-il for inspecting Neo compiler output on stdout or in a new neoIL file,
  preserving original source sequence points. It validates without executing and
  refuses file overwrites. Added CLI round-trip/error regressions and debugging
  instructions. JSON artifact disassembly and binary encoding remain future work.

- Added initial Int32-backed enum/flags metadata and Neo declarations, named constants,
  typed bitwise/equality operations, explicit integer conversion and zero/unnamed values.
  Runtime validation enforces enum layout and nominal calls; enum construction accepts
  the underlying integer without granting private payload access. Added IsEnum,
  GetEnumNames and GetEnumUnderlyingType reflection APIs. Migrated BindingFlags to enum
  metadata while retaining its existing factories, bit values and filtering policy.
  Added artifact/source regressions, an example and a pinned .NET comparison probe.
  Older tools reject the new format-5 metadata and Neo reserves enum; other integer
  widths, formatting and general const/literal-field support remain future work.

- Added generic Neo union carriers composed from independent source case types, with
  substituted constructors, exact case conversions, exhaustive matching and conditional
  bindings. Cases retain their own generic parameters and existing reference/value
  contracts. Explicit constructor type arguments are required; inline generic cases
  remain deferred. Added an example, grammar/API notes and artifact regressions.
  Planned enums/flags with BindingFlags as the first API migration, plus CLI inspection
  of emitted Neo IL; binary encoding remains future work.

- Added initial runtime-enforced notvoid/notreference generic constraints on types and
  methods, neoIL .constraint directives and Neo where clauses. Restrictions apply to
  the outermost argument; symbolic forwarding is rechecked at concrete resolution.
  Extended this with nominal base/interface bounds, substituted/scoped bound metadata,
  and Neo constrained method lookup through explicit T& receivers. Calls reuse checked
  reference views and ordinary dispatch, preserving readonly access and concrete identity.
  Added metadata/host/source/IL regressions, an executable example and a reproducible
  .NET comparison probe.
  Added constrained T& conversions for API arguments, assignments and returned
  base/interface views, with inherited conformance and readonly/lifetime enforcement.
  Included a view/identity example and .NET reference-versus-boxing comparison,
  distinguishing platform capabilities from high-level language usability.
  Added constrained calls on addressable T values with notreference, reusing slot
  addresses and views with existing binding mutability, shallow copy and escape rules.
  Included value-copy and receiver-evaluation regressions and an executable example.
  Added preview ldreceiver adaptation for open generic parameter/local receivers,
  preserving stored reference identity or borrowing value storage with a chosen readonly
  capability. No boxing or nested managed references are introduced. Fixed inferred
  readonly-reference call signatures; added raw IL/artifact/Neo validation and CLR comparison.
  Broader receiver operands and member lookup remain later work; notnull awaits
  nullable metadata. Older tools reject constrained artifacts; Neo reserves the new keywords.

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
