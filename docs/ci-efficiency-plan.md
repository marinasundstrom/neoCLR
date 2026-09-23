# CI efficiency for the next release

Author direction recorded 2026-09-23, after Preview 9 publication: improve CI for the
**next release, not this release**. Avoid running the entire sample and validation
suite on every platform. Isolate platform-specific behavior so validation remains
useful without multiplying identical work. No workflow change is made by this plan.

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
