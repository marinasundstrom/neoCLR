# Shared native scheduler step

2026-10-11, development implementation. JIT, HTTP AOT and retained HTTP hosts use
one private completion-state boundary in `socket-listener.c`. They no longer scan
operation slots to decide whether to keep pumping. This separates host lifetime and
waiting from completion-source storage. The interpreter retains its existing Rust
scheduler; both now distinguish ready, pending and idle at their adapter boundary.

This is not public Scheduler API, parallel dispatch or runtime-managed async suspension.
It preserves source-owned callback roots and owner-thread scope lifetime. A host must
consume each returned callback before stepping again. Socket outcomes awaiting result
consumption keep the source pending. A final timer delivery is released at the next
step, including an idle step. Fault teardown still releases abandoned operations.

## Evidence

- [Six native C contracts](contracts.json): listener, accept, transfer, client, timer,
  and scheduler. The new case covers pending/ready/idle, GC retention/release,
  delivered-but-unconsumed socket results, and rejection of missing/wrong scopes and
  invalid active-frame boundaries without changing the output handle.
- [Timer and fault cases](timers.json): six interpreter/AOT/JIT executions; successful
  workers each check ten timer scenarios. A callback fault leaves another operation
  pending, exercising teardown before code release.
- [HTTP AOT host timers](http-host-timers.json): ten callback scenarios using the
  separate project host, with command/input provenance and compact build evidence.
- [Retained HTTP](retained-http.json): three requests, fragmented requests, and a
  terminal callback fault after success, including listener shutdown.

All executions were local macOS ARM64. The shared C test joins the existing Windows
native-network workflow, but no Windows execution is claimed for this revision.
No performance comparison or public API/metadata change is involved.

## Reproduce

Set the supported Xcode `DEVELOPER_DIR` and `SDKROOT` on macOS, and use matching
compiler/runtime libraries and tools. Each output path must be fresh.

```sh
python3 scripts/validate-native-network.py --output target/native-scheduler-contracts
python3 docs/experiments/jit-async/verify.py \
  --bundle target/data-cases-bundle \
  --console-qualification target/jit-startup-validation/validation.json \
  --timers --output target/native-scheduler-timers
python3 scripts/build-native-project.py --profile http \
  --bootstrap-root Bootstrap \
  --project docs/experiments/retained-http/Native.rvnproj \
  --bundle target/data-cases-bundle \
  --aot tools/aot-poc/target/release/neoclr-aot-poc \
  --output target/native-scheduler-retained
python3 docs/experiments/native-scheduler/verify-retained.py \
  --build target/native-scheduler-retained \
  --output target/native-scheduler-retained/validation.json
```

The focused retained verifier reuses the established HTTP protocol checker without
rebuilding unrelated runtime/compiler binaries. It validates input hashes before
and after execution. The full retained-HTTP verifier remains available for a broader
interpreter comparison.

## Owned dispatch slice — 2026-10-11

The outer HTTP and retained-session hosts now opt into `neoclr_scheduler_acquire_v1`
and `neoclr_scheduler_complete_v1`. Acquisition retains an independent dispatch root;
all polls reject advancement until the host completes that ticket. Completion requires
the original boundary after callback return, including guest fault. Duplicate, stale,
wrong-context and wrong-boundary tickets are rejected. Scope teardown releases an
abandoned ticket. Operation-result consumption remains independent: either lifetime
may finish first without invalidating the other.

The additional root uses one slot in the bounded host-root registry, with no new
managed allocation. Acquisition failure is terminal for the host and requires scope
teardown; it is not retryable delivery. There is no parallel dispatch or throughput
claim. The generated entry pump (including the JIT adapter) still uses the legacy
step protocol; migrating it requires a separate entry-lifecycle adapter. This slice
does not establish full cross-mode acknowledgment parity or a stable interop ABI.

[Owned-delivery contracts](owned-contracts.json) pass all six native contract
executables with bounds/undefined-behavior instrumentation on macOS ARM64.
[Retained HTTP](owned-retained-http.json) passes normal, fragmented and terminal-fault
cases. The HTTP-profile timer sample also passes its ten scenarios from
`target/scheduler-owned-http/app`. Windows runs remain pending for this revision.

Next boundaries: generated-entry delivery acknowledgment, awaiting execution's
continuation destination, and wait policy alignment. See the
[scheduler design](../../runtime-scheduling-design.md#native-completion-state-boundary--2026-10-11).


## Owned entry delivery — 2026-10-11

JIT and HTTP AOT entry adapters now opt into owned scheduler delivery. The private
TaskQueue host scope accepts a completion hook alongside its poll hook. After a
callback returns to the exact published entry boundary, the next poll acknowledges
its ticket before acquiring more work. Entry end acknowledges the final ticket on
normal or guest-fault exit. Reading a ticket twice, reading a different ticket, and
advancing without first reading a delivery are rejected. Completion-hook failure is
sticky, prevents a new entry, and is reported by scope leave; the guest fault remains
unchanged and source teardown releases the abandoned root.

This reuses the generated wrapper's existing poll/read/end calls; no compiler change
or new generated-code import is required. Poll-only hosts remain supported. The
completion adapter knows about entry lifecycle, while the TaskQueue host scope has
no dependency on socket storage. A ticket may remain rooted until next poll/end after
callback return; no earlier release or parallel dispatch guarantee is implied.

[Six native contracts](entry-contracts.json) include entry lifecycle, nested-boundary
rejection, single consumption, GC retention, fault cleanup and completion-hook failure.
[Timer consumers](entry-timers.json) pass success and fault cases in interpreter,
AOT and JIT (six executions). The existing task-queue C contract passes with bounds
and undefined-behavior instrumentation. The HTTP-profile timer app in
`target/entry-owned-http/app` passes ten scenarios. All evidence is macOS ARM64;
Windows qualification remains pending. No performance claim is made.

Next: continuation destination and cross-mode wait policy, followed by the retained
activation boundary for runtime suspension. Owned callback dispatch does not itself
implement runtime async suspension or select a public Scheduler API.
