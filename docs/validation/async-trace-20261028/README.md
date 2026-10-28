# Owner-thread async tracing qualification — 2026-10-28

Implemented the second opt-in tracing slice: owner-thread context entry/return, root publication before parking, operation completion, finish/abandonment and scheduler wait. [Contract and reproduction](../../experiments/execution-trace/async-README.md). No scheduler policy, public managed API or production default changes.

## Correctness and boundaries

Optimized and fail-fast UBSan fixtures pass with synchronous GC and the configured owned concurrent collector. Each checks parked live locals across collection, independent nested work while another context waits, completion-before-wait, repeated physical carrier reuse, callback failure, forced shutdown of a parked body, complete/one-record-overflow captures, trace API lifetime and a fresh second recording session. Export checks reject malformed sequence numbers and mark truncated captures incomplete; nested execution spans do not overlap on the single owner.

The concurrent fixture confirms 40 started and finished marking epochs in each eight-round correctness invocation and 5,120 in each 1,024-round timing invocation. These are real collector epochs, not a sync build relabeled concurrent. A separate existing owned-body test with synchronous GC and UBSan verifies 321 entered bodies, 160 resumed, 161 expired/abandoned, zero retained stack bytes and balanced mappings. This does not claim concurrent qualification of every owned-body expiry case. UBSan is not a race detector.

The scheduler adapter deterministically completes an operation during its wait call. It is not a socket/network benchmark. The events identify operation completion and wait boundaries, not backend I/O submission/readiness or GC/JIT phases. Logical context tracks share one OS owner thread; their wall spans do not measure CPU occupancy or parallel managed execution.

Normal builds compile out hooks, recorder storage and the extra fiber field. Default `-O2` ARM64 carrier assembly is byte-identical to `5e5f124c7` with the same compiler; both assemblies and the comparison command are retained. Tracing is macOS ARM64 only; Windows instrumentation remains explicitly unsupported.

## Paired overhead screen

Three alternating recording-off/on pairs per collector; 1,024 rounds per invocation, each with a real park/resume, independent nested work and a completed wait. Setup, fault/shutdown checks and JSON export are outside timing. Recording on emits 33,802 events including post-timing checks; no losses. The configured 65,536-record buffer reserves 2.5 MiB (not a resident-memory measurement). The off control is an instrumented build without an active recording session, not the normal compiled-out build.

| Collector | Off/on elapsed median (ms) | Elapsed change | Off/on process CPU median (ms) |
| --- | ---: | ---: | ---: |
| sync | 12.495 / 14.329 | +14.7% | 12.199 / 13.699 |
| concurrent | 43.710 / 46.710 | +6.9% | 44.447 / 47.297 |

These short desktop measurements establish material, variable diagnostic overhead, not a collector recommendation or HTTP capacity gain. The earlier screen, before adding the forced-shutdown assertion, showed about +8%/+1% elapsed overhead; it is retained separately. Use untraced runs for performance decisions. Broader CPU/I/O/HTTP loads remain necessary to compare collector advantages.

## Evidence and next work

The host is Apple M1/macOS with eight logical CPUs and 16 GiB. Desktop activity is uncontrolled. `qualification.json.gz` retains final source hashes, exact commands, every timing sample, sanitizer output and epoch counts. Generated collector C and complete/overflow raw captures are retained alongside Chrome JSON exports. Interactive Perfetto import was not exercised.

The initial link attempt lacked an unused string-provider symbol from the generated collector. The focused executable now uses ordinary linker dead stripping to remove unused provider wrappers; it does not stub or replace a called runtime service. The failed report is retained. The current schema/fixtures supersede that initial diagnostic run.

Next introduce shared trace session identity across the worker/async/compiler/collector producers, then JIT wait/publication and GC phase events. The two current exports use a compatible monotonic clock but are separate sessions and cannot simply be concatenated. Real I/O correlation and cold/warm HTTP measurements under both collectors remain open.

All 26 website contract tests pass; local documentation links resolve. Website content is prepared for the next publication; no deployment was performed.
