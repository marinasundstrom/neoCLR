# CI efficiency for the next release

Author direction recorded 2026-09-23, after Preview 9 publication: improve CI for the
**next release, not this release**. Avoid running the entire sample and validation
suite on every platform. Isolate platform-specific behavior so validation remains
useful without multiplying identical work. The broader matrix redesign remains planned; the immediate trigger correction
below was separately directed after publication.

## Evidence and problem

Preview 9's [release matrix](https://github.com/marinasundstrom/neoCLR/actions/runs/35868351666)
ran full source/archive validation in six OS/toolchain jobs. Linux, macOS and Windows
each repeated stable and Rust 1.85 checks, sample execution, archive membership and
notice validation. The Windows minimum-Rust job was the last release gate.
The [release evidence](preview-9-validation.json) preserves actual timings and results.

The current workflow also repeats direct sample/native checks after the release
validator. Audit overlap before removing jobs; a different host boundary or artifact
may justify a check even when its command looks similar.

## Proposed split — to validate before the next release

| Responsibility | Proposed placement |
| --- | --- |
| Formatting, ordinary static analysis, archive membership, notices and metadata snapshots | One canonical job; do not repeat OS-independent inventory checks per platform |
| Portable VM/library contracts and complete sample outcomes | One comprehensive canonical run; identify any host dependencies before moving tests here |
| Platform boundaries | Focused Linux/macOS/Windows jobs covering native FFI/ABI, filesystem paths/errors, environment/time, worker lifecycle/cancellation and other demonstrated host-dependent behavior |
| Minimum Rust support | A focused compatibility job plus necessary target-specific compilation; avoid a second complete sample suite per OS |
| Cross-platform execution | A small representative smoke set for packaging/loading/execution, with explicit reasons for each case |
| Distributed binary bundles | Build and exercise each binary target actually shipped; do not confuse source compilation with package execution evidence |
| Website/API documentation | Relevant-change validation and the existing separate manual deployment |

These are assistant-proposed assignments, not yet an implemented matrix. Inventory
tests and measure time first. Portable behavior can still expose OS-specific bugs;
retain targeted checks or a small cross-platform regression when evidence warrants
it. Avoid merely deleting coverage or labelling mixed tests platform-independent.

## Completion criteria

- Classify existing checks by contract and host dependency, with owners and explicit
  evidence for the reduced matrix. Split mixed tests where necessary.
- Remove duplicated setup, sample execution and inventory work; reuse artifacts and
  caches only with keys that preserve compiler, target and source provenance.
- Keep exact-candidate evidence, published hashes, supported binary-target checks and
  clear failure reports. A faster pipeline must still identify what actually passed.
- Compare critical-path duration and runner minutes with Preview 9, and document any
  remaining coverage tradeoffs. Select a time budget from measurements, not a guessed
  promise.

Implement this before the next release's validation cycle. It does not change the
published Preview 9 gates or displace the foundational Streams/Storage/Encoding work.


## Immediate trigger correction — 2026-09-23

After observing new full runs from documentation changes, the author directed that
those runs be avoided now. The runtime workflow now ignores Markdown, top-level
JSON evidence records in docs/, website/API-reference files and their build tooling.
It still runs for runtime/source/tests, Cargo inputs, release validation tooling and
executable code or fixtures under docs/experiments/. Mixed code/documentation changes
still run the matrix. Website validation retains its own relevant-path workflow.

Automatic push validation is branch-only: creating a release tag no longer repeats
an already validated candidate. `workflow_dispatch` permits an explicit full run on
a selected branch or tag. The six jobs themselves are unchanged; their future split
and duplicate-work reduction remain the next-release task.

The trigger-only change was checked with actionlint. A subsequent ordinary docs-only
push is used to verify filtering without a skip directive. No full matrix is needed
to validate these trigger rules. Redundant runs 35876872188 (docs/site push) and
35876772578 (release tag) were cancelled; the successful release matrix 35868351666
and successful website deployment were retained.

These filters use [GitHub's documented path and branch rules](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax).
If runtime jobs later become required PR checks, account for path-skipped workflows
remaining pending; do not enable that branch policy without an appropriate lightweight
required check. Main had no branch protection when this correction was made.

## Pre-release Raven example portability pass — author direction, 2026-09-23

Near the next neoCLR release, select representative existing Raven examples that run
on .NET, port a bounded set to neoCLR and investigate crashes, missing APIs and
semantic differences. Prefer real small programs over isolated syntax checks. This
is a pre-release task, not a reprioritization of the current Object slices.

Establish compiler provenance before interpreting differences:

1. Extract shared compiler fixes from the neoCLR integration branch, validate them
   independently on Raven main (including relevant .NET Framework/NanoFramework
   targets), and make a Raven release containing those fixes. Do not merge the
   neoCLR branch wholesale or release target-specific policy as a shared fix.
2. Integrate the same shared fixes into the neoCLR branch. Record Raven release/tag,
   main commit, integration commit, compiler hashes, target configuration and the
   patch difference between the two builds. Branch ancestry alone is insufficient
   evidence that the binaries contain the same fixes.
3. Run the original examples on the released Raven/.NET baseline, then their minimal
   ports on the matching neoCLR bundle. Record exact source revisions, edits,
   expected behavior, compiler diagnostics and runtime outcomes. Examples relying
   on intentionally unsupported platform capabilities need an explicit scope note.
4. Classify failures as shared compiler, target metadata/importer, missing library
   contract, runtime crash/semantic mismatch or expected platform limitation. Reduce
   unexpected failures, fix them in the appropriate repository/branch, and rerun the
   affected originals and ports. Shared defects discovered here require another
   validated main/release integration before claiming a clean compiler baseline.
5. Keep passing ports as reproducible compatibility cases. Give remaining failures
   an explicit release disposition; do not silently skip crashes or label every
   unsupported API a required release feature. The pass aims to exclude known shared
   compiler causes, not to prove that no shared compiler bug remains.

Choose cases by coverage: record equality/hash once available, ordinary class/value
behavior, Result/Option, collection operations, and a small Console or file program.
The selected list and expected differences should be recorded with release evidence.
Run platform-neutral cases once on the primary host; repeat only platform-specific
boundaries on other hosts, in line with this plan's efficient CI split.
