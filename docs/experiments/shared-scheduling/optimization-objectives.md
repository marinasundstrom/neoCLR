# Shared runtime optimization objectives — 28 October 2026

Author direction: make these three points the objectives and continue work.
The preceding scheduling decision remains intact; shared queues stay opt-in.

1. **Root publication and runtime boundary bookkeeping.** Start from profiles of
   enter_owned, stack_roots and TLS access. Inspect generated calls and native entry
   work. Preserve dynamic address/width/flag validation, current owner, fault roots,
   mutable root values, park/resume and frame lifetime. Compiler-known immutable
   facts are not proof of dynamic addresses. Existing owned-chain/state handles
   already remove work; do not repeat the rejected retained-TLS grouping or assume
   the split small-frame experiment is a universal win.
2. **Concurrent store checks.** Attribute allocation lookup, ownership/extent
   validation and marker-gate work. Any cached proof needs invalidation on reclaim,
   reset, heap release/adoption and epoch transitions. Preserve shared references,
   scalar/reference classification and fail-closed unsupported-provider behavior.
3. **Shared I/O completion notification.** Measure DNS polling and completion-to-
   runnable latency, idle CPU and interference. Reuse worker/owner completion
   services where beneficial; avoid an HTTP-specific scheduling path. Keep I/O
   services owner-affine unless a separately qualified ownership contract allows
   migration. Cancellation, late completion and shutdown must retain bounded roots.

For each candidate: qualify correctness, compare identical workload/resources under
both collectors and the affected JIT/AOT paths, preserve regressions, then enable a
default only where evidence justifies it. Include ordinary compute/low-load controls
and the mixed HTTP workload; distinguish startup, first-use and warm performance.
Do not promise that every objective will produce a winning optimization.

## Current work

Root-publication source review finds owned ancestor proofs and retained state handles
already implemented. Prior split-validation results are mixed and remain opt-in.
The existing owned-chain/checked-root validation was rerun successfully on current
source before proposing a change, including paced/worker configurations and sanitizer
checks. No validation has been elided and no performance improvement claimed yet.

The author additionally requests a standardized way to serve diagnostic data for a
future GUI. The [versioned session contract](../../diagnostic-session-format.md)
extends the existing read-only local snapshot service with lossless exported sessions.
GC metrics are implemented; timelines and heap/root-path payloads are explicitly
unsupported in this first adapter. This supports measurement, not a requirement to
build a GUI before optimizing. Screenshot-inspired region/type/root-path views must
represent neoCLR's real memory model and validated snapshots.

The app's older HTTP goal is still recorded as paused and prevents creation of a
replacement goal through the available API. These repository objectives record the
author's new direction; implementation can continue under that explicit authorization.
Do not mark the older unfinished goal complete merely to replace its tracking entry.
