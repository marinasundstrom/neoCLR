# JIT fallback cost and whole-process memory — 2026-10-25

Follow the [fallback ownership qualification](../jit-arena-fallback-20261025/README.md)
with separate timing and memory cohorts. Production placement and GC defaults are
unchanged. The same pinned runner/native hosts and qualified compiler policy are
used; only the test-only system/fallback placement choice differs within each pair.

## Unprofiled concurrent HTTP

Three interleaved rounds pass 49,344 plain/JSON responses, four clients and pipeline
depth 16. System versus fallback medians:

| Handler | System requests/s | Fallback requests/s | Invocation CPU change |
| --- | ---: | ---: | ---: |
| Plain | 3,764 | 5,585 | −32.85% |
| JSON | 2,223 | 3,201 | −30.32% |

This reproduces the large placement gain with the fallback implementation; it does
not establish a concurrent-over-sync advantage, saturated capacity, confidence
intervals or pause improvement. Compile time is excluded from invocation CPU.
Normal workloads do not deliberately force reservation exhaustion; forced-spill
correctness remains covered by the preceding qualification.

## Separate process snapshots

Eight diagnostic runs pass another 8,320 responses across plain/JSON and both
collectors/providers. New `http.py --process-memory` invokes macOS `ps` against its
owned server PID at readiness, after warmup and after traffic. Reports retain raw
commands and fields, converting RSS/virtual-size Kbytes to bytes. Identity, row
shape and nonnegative counters are checked. Three tests cover conversion, malformed
rows/wrong PID/negative counters, and a live owned-process read.

[RSS table](memory-table.md) retains after-traffic snapshots. System/fallback RSS
is 963.9/970.8 MiB for sync plaintext, 1083.8/1095.6 for concurrent plaintext,
960.5/969.3 for sync JSON, and 1067.0/1098.4 for concurrent JSON. Fallback is about
6.9–31.4 MiB higher in these single diagnostic cases. RSS is nearly flat across the
three snapshots within each process. Repeated cohorts would be needed to characterize
variation or establish peak behavior.

These figures include the compiler/test runner, loaded metadata, native libraries,
JIT mappings and managed heap. They are not native-allocator attribution, physical
footprint, peak memory or proof of a leak. Virtual sizes around 466 GiB include the
process's broad mappings/reservations and must not be interpreted as resident RAM.
The 64 MiB arena is an address-space reservation, not an equal RSS increase. `ps`
child CPU contaminates whole-child accounting, so the harness marks these runs as
profiles and excludes their timing from performance conclusions.

## Finding and next action

The approximately 1 GiB whole-process residency is much larger than the managed
arena counters. Source inspection shows `run_internal` retains its compilation
scope while `jit::execute_managed` both compiles and invokes the guest. That is a
concrete lifetime to investigate, not attribution of the entire RSS to one object.
Audit which selection/verification/metadata structures can be released after code
publication, preserving code, static data, native host and argument lifetimes.
Then repeat snapshots and timing before choosing a production placement policy.
The GC comparison/default and broader memory-pressure/platform gates remain open.

## Reproduction and provenance

Use the preceding fallback build with fresh output directories. Run `http.py`
with `--modes live-jit fallback-live-jit --protocols pipeline --requests 4096
--rounds 3` for timing. Separately run `--modes sync-jit fallback-sync-jit live-jit
fallback-live-jit --protocols pipeline --requests 1024 --rounds 1 --process-memory`.

The timing cohort completed before adding the diagnostic harness extension; its
original script hash is preserved. The memory cohort records the new harness and
helper hashes. Raw reports preserve commands, runner/host identities and individual
trials. No current source hash is substituted into an older report.
