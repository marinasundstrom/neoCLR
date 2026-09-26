# CI efficiency for the next release

Author direction recorded 2026-09-23, after Preview 9 publication: improve CI for the
**next release, not this release**. Avoid running the entire sample and validation
suite on every platform. Isolate platform-specific behavior so validation remains
useful without multiplying identical work. The matrix redesign below implements that direction; the earlier trigger correction
is retained as history.

## Evidence and problem

Preview 9's [release matrix](https://github.com/marinasundstrom/neoCLR/actions/runs/35868351666)
ran full source/archive validation in six OS/toolchain jobs. Linux, macOS and Windows
each repeated stable and Rust 1.85 checks, sample execution, archive membership and
notice validation. The Windows minimum-Rust job was the last release gate.
The [release evidence](preview-9-validation.json) preserves actual timings and results.

The current workflow also repeats direct sample/native checks after the release
validator. Audit overlap before removing jobs; a different host boundary or artifact
may justify a check even when its command looks similar.

## Implemented next-release split — 2026-09-27

The author clarifies that the HTTP POC is intended for release and reiterates that
full suites on every target are too costly. The default workflow now separates:

| Responsibility | Placement |
| --- | --- |
| Format, Clippy, archive membership, notices, library snapshot | Canonical Linux stable job |
| Complete runtime tests and source/artifact sample outcomes | Once, canonical Linux stable, optimized release profile |
| OS/ABI boundaries | Focused macOS/Windows stable jobs; Linux is covered by the canonical full suite |
| Minimum Rust 1.85 | `cargo check --locked --all-targets` on Linux/macOS/Windows; compile tests and target cfg branches without repeating execution |
| Full stable OS regression | Opt-in `full_matrix` manual dispatch; replaces focused host jobs with full source/archive validation |
| Distributed SDK/runtime packages | Exact-artifact checks for each binary target selected for release; source CI does not certify SDK packages |
| Website/API reference | Separate relevant-change validation and manual publication |

`validate-release.py --release` uses optimized binaries/tests and records that profile.
The existing default remains debug for explicit diagnostic runs. CI's release profile
matches the distributed runtime; debug-only assertions are not exercised by that run.
The host validator reports each command, duration and outcome and fails if a private
unit filter silently selects zero tests. Superseded automatic runs are cancelled;
manual candidate runs are not automatically cancelled by later pushes.

### Test classification and coverage boundary

`scripts/validate-host.py` is the executable host inventory. Integration targets cover
worker/cancellation behavior; environment, clock and dates; files, paths and console;
CLI/process/module/source-file loading; native calls, memory and target layout.
Private unit groups cover DNS, sockets, completion-to-VM roots, scheduler progress,
file services and workers. All remaining integration targets and unit groups execute
in the canonical full suite. This retains every existing test in default CI without
claiming that a portable test can never expose an OS-specific bug. New host-sensitive
tests must be added to this inventory; mixed targets remain whole rather than
excluding individual inconvenient cases. The manual full matrix is available for
cross-platform regressions or substantial runtime changes.

The old post-validator native build/PInvoke and feature-tour runs duplicated existing
validator/tests. Native execution stays in the validator and host checks; embedding
`invoke` is checked explicitly. Minimum Rust remains checked against each platform's
conditional code, rather than assuming Linux compilation proves Windows compatibility.
No target support is removed by the split. Actual shipped binary targets still need
extracted-package execution, including the separate Raven toolchain where applicable.

### Baseline and acceptance

The successful Preview 9 run recorded 15.9/29.7/51.1 minutes for stable Linux/macOS/
Windows and 21.5/39.1/68.0 for minimum Rust: approximately 225 runner-minutes, with a
68-minute critical job. These are historical measurements, not a forecast for the
new workflow. The September 26 run at `73e27938` failed format checks before reaching
validation and cannot serve as a performance baseline.

Record local host/profile validation and the first hosted run before claiming a speedup
or choosing a duration budget. A successful local macOS run does not certify Linux,
Windows or the minimum-Rust matrix. Keep exact revisions, artifact hashes and known
failures with release evidence; do not shorten gates by hiding failures.

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
