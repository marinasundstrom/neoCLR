# Pre-release development log through 2026-10-26

Preserved verbatim before Preview 18 release consolidation, from `dce8207ca`.

## Unreleased

Preview 18 candidate preparation is underway for macOS ARM64 and Windows x64;
publication date and final assets remain pending.
The theme is clearer platform contracts, more efficient HTTP/value handling, and
stronger runtime execution. Synchronous GC remains the default; concurrent GC
continues as an experimental native development path.

### 2026-10-26

#### Release qualification

- Repair task-test references to the renamed generated `Optional` fixtures and
  use a bounded, NUL-inclusive copy in the root-validation test so Windows can
  compile it with warnings-as-errors. Record macOS extracted JIT/interpreter
  acceptance and the remaining source/Windows qualification failures separately;
  all five affected task tests and the 188-command macOS native GC gate pass;
  hosted requalification remains required. Migrate the header and plain/JSON
  release benchmark fixtures to `Failure`; all 132 macOS header cases pass and
  both benchmark inputs compile. Packaged async (24 executions) and JSON (three
  modes) also pass. Correct the editor hover harness to expect `Optional`; all
  26 macOS editor checks pass on a fresh workspace. Record source-CI, HTTPS,
  native-kit and notice-audit acceptance, while keeping Windows editor rerun and
  final measurements pending. Retain the release compiler pin; backticked test
  names are deferred. Add retained macOS candidate benchmark tables on the
  website (five stages, 55 CPU/JSON processes and 1,152 HTTP responses), plus
  editor recovery, standalone SDK attribution and Windows HTTPS evidence.
  Windows package/editor qualification and all five final benchmark stages pass.
  Retain the slower hosted measurements and changed CPU-model conditions. Verify the qualified source archive checksum and record lazy JIT with
  trampolines as a post-release investigation. Preserve the macOS PostgreSQL
  provider by rebuilding its candidate; update the SQL qualification harness to
  import native SDK runtime contracts. Live queries pass interpreter/JIT/AOT and
  all ten protocol-peer cases pass. Final macOS package checks and measurements
  are being repeated for the provider-inclusive archive; extracted JIT/interpreter,
  24 async executions, 132 headers and native-kit HTTP/C/C++ checks pass. The
  rebuilt package also passes all 26 editor checks, inlays, twenty recovery cycles,
  15 HTTPS cases and the 2,418-file archive audit. Its final measurements remain
  pending. Preview 18 is not yet published.

### 2026-10-25

#### Release preparation

- Select runtime `0.1.0-preview.18` and SDK/editor candidate
  `0.1.18-neoclr.20261025.1`, reconciling the stale Cargo version. Begin fresh macOS
  ARM64 and Windows x64 qualification with Raven neoCLR `ed1d08defe2778f965951747d5f48311ce3daf83`.
  Freeze optimization scope, retain sync GC/default JIT placement, update candidate
  docs and website preparation status, and keep Preview 17 download links until
  new package acceptance. Refresh the matching API-reference fingerprints and apply
  required Rust formatting before source qualification. Repair legacy source tests
  and imported IL fixtures left behind by the Optional/Failure migration, including
  a nonexistent fixture filename. This does not publish a release.

#### Platform APIs and migration

- Rename `System.Option<T>` to **`Optional<T>`**, retaining `Some`/`None`, and rename
  `Result.Error` to **`Result.Failure`**, retaining `Ok`. Update construction,
  patterns and affected imports, and rebuild matching consumers. Payload-oriented
  error methods and Optional JSON/propagation behavior retain their semantics.
- Add `System.Error`, typed context decorators and failure-context helpers with
  optional diagnostic capture and bounded formatting. Capture defaults off;
  source file/line resolution remains deferred. Add ConditionalAttribute support.
  Result and ad-hoc unions remain independent of these conventions.
- Expand `System.Environment` with fallible OS discovery, process/OS architecture,
  `NewLine` and structured version information. Environment is now a static class;
  qualified calls remain supported, while former module-function imports must
  migrate. Linux distribution discovery and additional-host qualification remain open.

#### HTTP and efficient value contracts

- Add structured **MediaType**, cached common values and typed HTTP content
  overloads. HTTP metadata retains validated primitive storage and projects
  structured values lazily. Distinguish absent, malformed and duplicate
  `Content-Type`; legacy outbound strings now receive full syntax validation
  before I/O. Accept ranges and weights are not media-type values.
- Reduce temporary URI parsing/resolution work by reusing validated components
  and UTF-8 storage, preserving validation semantics. Value objects remain the
  consumer-facing contract; internal code avoids unnecessary materialization.

#### Runtime execution, GC and concurrency

- Release managed JIT compilation/selection inputs before guest invocation while
  retaining executable code, static data and native-library ownership through host
  cleanup. Qualification covers 203 compiler tests, HTTP and six standalone
  patterns. Process-memory reductions vary by collector; invocation timings are
  mixed, so this is a lifetime improvement without a throughput claim. Add opt-in
  macOS test-runner malloc accounting around preparation, release and cleanup,
  separating live native allocations from allocator reservation and RSS. Memory
  profiles require complete snapshots; ordinary timing runs clear the switch.

- Extend the standard native HTTP matrix with optional macOS stack sampling,
  separating diagnostic timings from benchmarks and retaining capture hashes,
  TLS caller attribution, handler, protocol and execution-mode identities.

- Add an opt-in macOS ARM64 stack-bound layout that resolves lower/upper bounds
  through one TLS lookup while retaining current-owner checks, reserve limits,
  bound replacement and reset. Matched HTTP and standalone qualification reports
  modest, mixed changes; this is shared runtime work, not a collector-default switch.
  Evaluate composition with small-frame validation in longer matched HTTP and standalone
  cohorts, with separate checked owner-operation histograms. The combined AOT
  candidate reduces invocation CPU; matched concurrent JIT CPU also improves about
  4.8–6.6%. Integration on the qualified macOS ARM64 path is next. Owner-operation
  p99 buckets remain unchanged; synchronous GC remains the default.

- Reduce repeated native root publication/descriptor validation, metadata and
  managed-allocation lookup work, redundant initialization and allocation
  bookkeeping. Add an opt-in small/large storage-validation split that preserves
  activation checks while reducing small-frame register-save work; initial AOT
  HTTP results show modest CPU gains and four allocation-heavy standalone patterns
  improve about 4–6%. Matched JIT HTTP follow-up shows smaller gains (about
  1.2% concurrent plaintext CPU and 0.18% JSON), with mixed standalone JIT
  results. Keep the split opt-in; sync remains faster in HTTP.
  Preserve tracing of changing root values and explicit ownership
  across native boundaries and suspended work. Requalify test-only contiguous JIT
  code/static-data placement through the instruction-cache wrapper: matched HTTP
  shows about 30–33% lower concurrent invocation CPU and four allocation-heavy
  standalone patterns improve 32–35%, with sync also improving in HTTP.
  Add a test-only system fallback for unavailable/exhausted JIT reservations,
  retaining existing code/data through publication and teardown. Retain the
  system provider default pending memory/large-image/platform qualification.
  Repeated fallback HTTP preserves the CPU gain; separate whole-process RSS
  snapshots expose substantial JIT-host residency for compilation-lifetime
  investigation. Add checked, diagnostic-only process-memory profiling.
  Reuse completed macOS async
  contexts under their qualified lifetime rules. Explicitly publish macOS ARM64
  JIT instructions through the platform cache API, correcting intermittent
  execution failures when generated-code storage is reused.
- Continue the **concurrent native GC experiment**, including live marking,
  mutation/root safety and tunable collection policy. Integrate qualified
  complete-caller stack proofs and immutable byref handling into the private
  JIT/AOT backend, retaining roots and guards for unknown destinations. Avoid
  unused async queue-drain declarations and discarded compiler audit reports.
  Consolidate native candidate sources under the runtime, with a checked owned-heap
  policy and benchmark-independent source generation; preserve experiment forwards.
  Integrate qualified typed scalar stores within the private concurrent candidate,
  retaining a generic-store comparison. Earlier qualification halves CPU in the
  standalone buffer-reuse workload; HTTP effects are small and mixed. These
  results do not establish a universal throughput win or justify changing the
  synchronous default.
- Develop bounded worker coordination and internal persistent HTTP handling,
  with separate shared-heap and independent-owner experiments. These are not
  general virtual threads or new public persistent-server guarantees; public
  Accept/Serve connection behavior is unchanged by the private benchmark driver.

#### Diagnostics, documentation and release evidence

- Standardize checked plain/JSON HTTP and six standalone workload benchmarks,
  comparing execution modes and independently tuned collectors. Add owner-operation
  histograms and allocation, metadata, scheduling and context-transfer diagnostics;
  keep instrumented profiles separate from throughput measurements. Publish
  successes, regressions and remaining limits in the [benchmark evidence](benchmarks/gc-suite/README.md).
- Refresh API references, migration examples and error-handling guidance, and unify
  API navigation across native, authored and retained reference pages. Correct the
  Preview 17 website release banner without changing its published runtime assets.
- Prepare the next preview around these themes, keeping current improvements to
  ordinary execution distinct from experimental native paths. Fresh source,
  package, API/editor and target-host validation is still required before release.

The [detailed development history](docs/development-history/unreleased-through-2026-10-25.md)
preserves every earlier Unreleased entry, including experiments that were rejected
or superseded. The [runtime/GC development index](docs/development-history/runtime-gc.md)
links the research, implementation and measurement trail. See the
[draft release notes](docs/next-preview-release-notes.md) for the release narrative.

