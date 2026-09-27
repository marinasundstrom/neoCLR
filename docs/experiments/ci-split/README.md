# Release CI split validation — 2026-09-27

Current CI ownership is in [toolchain/release](../../tracking/toolchain-release.md).
The [design and host inventory](../../ci-efficiency-plan.md) describe the split and
coverage tradeoffs. This directory records evidence, not another task list.

Initial macOS arm64 validation passed 202 selected tests and the embedding/native
examples in 124.2 command-seconds, including compilation. See the
[initial host report](macos-host-validation.json). This is a local observation with
existing build caches, not a hosted timing comparison. The report identifies the
working-tree validator by hash; its only subsequent cleanup removes an unused import.

Actionlint accepts the workflow. Two Python regressions check that command failures
and empty private-test filters preserve a failed report. Canonical source validation
still runs all tests; other stable OS jobs use the focused inventory. Minimum Rust
compiles all targets on each OS. Manual `full_matrix` restores full stable execution.

The baseline is [Preview 9's successful run](https://github.com/marinasundstrom/neoCLR/actions/runs/35868351666):
68 minutes for its slowest job and about 225 runner-minutes across six jobs.
The [first successful split run](https://github.com/marinasundstrom/neoCLR/actions/runs/36278756461)
at `1404454e` passed all six jobs: canonical 8.88 minutes, macOS host 2.55,
Windows host 3.77, and minimum-Rust compilation 0.52/1.02/0.88 minutes.
The slowest job was under nine minutes, with about 17.6 total runner-minutes.
This is an observed workflow comparison, not a controlled hardware/cache benchmark.
[Job metadata](hosted-acceptance.json) records the run. It precedes the subsequent
Option/Result migration and does not validate that later source change.
