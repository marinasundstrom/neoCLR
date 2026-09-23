# Async and Tasks preview checkpoint

Author-selected immediate priority, recorded 2026-09-23. After asking when to release
the async/Task work, the author accepted this scope and directed that it take priority
before further platform expansion. This is a release scope and readiness plan,
not a release announcement or publication approval.
The prepared candidate version is **0.1.0-preview.9**. Publication date and final
artifact hashes remain unset; record the exact versioned commit in the manifest.
The earlier e9bb28a checks do not certify the subsequent versioned candidate.

## When to release

Start stabilization now, with the existing supported Task surface as the feature
boundary. Release once a fresh, extracted evaluator bundle reproduces the async
samples and the exact candidate passes the source/platform and package gates below.
Do not wait for HTTP, all of M1, a new suspension mechanism or a complete Task library.
If a gate fails, fix that failure or narrow the claimed scope; do not keep adding
unrelated features while the candidate is being stabilized.

The product demonstration is **Async Workbench**: small runnable Raven programs that
start an isolated worker, await its result, compose Promise completions, propagate
Result errors and observe producer cancellation. Keep the programs separate enough
that each explains one contract. Use the existing tested samples, not a new framework.
The host cancellation example adds the embedding perspective on stopping an invocation.

## Scope to stabilize

| Included development surface | Required explanation |
| --- | --- |
| System.Tasks.Task<T>, Promise<T>, State/Outcome and producer Complete/Cancel | First terminal transition wins; completion authority is separate from consumption; no exception-based Task fault state |
| Named async functions, await and Task<unit> | Compiler-generated state machines; supported suspension/GC behavior and diagnosed unsupported constructs |
| Map, Then, MapResult and `(await input)?` | Expected errors remain Result values; cancellation bypasses continuations and propagates through await |
| Default TaskQueue dispatch and explicit queues | Normal samples require no manual pumping; execution is serialized and unresolved Promises alone do not keep an invocation alive |
| Thread.Start and ThreadPool.Queue | Isolated String-to-String work; ordinary implementations still queue blocking joins; no shared guest objects or general nonblocking scheduler |
| Worker quotas and host invocation cancellation | Default payload limit, terminal Fault behavior, cooperative teardown and already-visible output |

Do not advertise the separate Delayed Copy notification adapter as the ordinary
Thread implementation. Retain it as a clearly labelled experiment. It is not a gate
to replace queued joins with that adapter before this preview. Guest cancellation
tokens, native I/O completion/acknowledgement, explicit-queue notification affinity,
streams, sockets/HTTP, preemption, runtime suspension and async disposal remain outside
the release scope. Known unsupported async constructs must be documented and rejected
clearly; their full implementation is not a gate.

This is a focus, not a selective history rewrite: review all changes since Preview 8
that are present in the selected candidate, including non-async migrations. Do not
omit compatibility changes merely because the release theme is Tasks.

## Release gates and evidence

All gates below are **open for the final candidate**. Focused local tests, samples and
editor probes exist, but they do not certify a future archive or another platform.

- [ ] Pin matching neoCLR and Raven revisions, SDK/reference/runtime-library inputs
  and VSIX build. Record the candidate manifest; preserve Raven branch isolation.
  Regenerate library artifacts and verify their snapshot and runtime-service contracts.
- [ ] Run the Task, composition, async, default-queue, worker, MapResult and editor
  probes from [the Task experiment](experiments/task-contract/README.md), plus runtime
  GC, cancellation, quota and invalid-IL regressions. Check both immediate and resumed
  paths, success/error/cancellation, one-shot completion and unsupported diagnostics.
- [ ] Build fresh packages with [the bundle tooling](experiments/raven-target/package_bundle.py).
  Extract them into new directories and build/run the Async Workbench samples via
  saved `.rvnproj` files. Verify expected output, editor completion/hover and normal
  build/run tasks using only those packages. Temporary adapter replacements, local
  development symlinks and stale compiler/reference fallbacks cannot satisfy this gate.
- [ ] Pass the [source/archive validation matrix](next-preview-validation.md) on the
  exact selected commit, including minimum/stable Rust, formatting and strict Clippy
  on the configured Linux/macOS/Windows jobs. Record separate package execution
  evidence for each claimed binary target; source CI does not establish binary support.
- [ ] Review the complete candidate diff, migration notes, notices, checksums and
  release notes. Align README, Tasks/overview/setup pages and downloadable samples
  with the actual package contents. Include the DocFX `/docs/` overview and useful
  main async API descriptions; exhaustive documentation is not required for this release.
  Verify supported APIs against .NET comparisons
  already recorded in [Task contracts](task-contracts.md) and [workers](isolated-workers.md).

Minimum migration review: System.Tasks namespace and producer API changes; terminal
Outcome and cancelled-await behavior; MapResult versus ordinary Map; unit/Void and
Result propagation lowering; matching compiler/reference/library requirements;
worker limits and Rust embedding changes. Audit the rest of Unreleased against
Preview 8 instead of assuming this list is exhaustive.

## Next action and exit

Local progress is recorded in [the readiness inventory](async-preview-readiness.md).
The corrected package, full library regeneration and protocol/MSBuild checks now
pass locally. Interactive build/run and Task type hover now pass in a separate VS Code
profile against the extracted `e9bb28a` bundle. Exact-candidate local source validation
and the six-job CI matrix are running. All six deeper packaged Task probes pass. Complete the source checks
and the final migration/distribution review; see the [release-note draft](async-preview-release-notes.md).
The [migration draft](async-preview-migration.md) inventories known breaks; the
minimum-Rust archive check found a dispatch syntax issue, now corrected locally.
The rebuilt runtime and corrected sample archive now have
[separate package evidence](async-preview-package-followup.json), including eight
Workbench samples and 84 saved-project outcomes. Full local source suites exposed a standalone worker sample with missing service
imports. The sample now supplies them directly; preserve failed archive reports
and record correction checks separately. Final-candidate full-source/platform gates
remain open.
Record blockers with evidence and only claim platforms actually tested. Remote candidate CI and publication remain distinct
steps; this plan does not tag, push, upload artifacts or publish the website.

When the gates pass, select the actual version/date and publication scope through
the [release process](source-release.md). After this checkpoint, resume M1 with Streams, Storage and Encoding before networking,
as directed by the author. Use bounded file-transformer cases to expose the needed
operation cancellation and queue-affinity contracts; the [authoritative roadmap](platform-roadmap.md)
keeps the sequence and ownership prerequisites explicit. The first major application destination
remains the HTTP client/server pair.
