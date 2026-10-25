# Concurrent GC across isolated HTTP owners

2026-10-25. The existing HTTP application-worker host now has a qualified
experimental live-GC combination with one and four isolated owners. This closes
an actual provider-admission gap exposed by the existing `Task.Run` HTTP fixture.
It is separate from shared-heap compiled read-phase experiments. Production GC
and HTTP defaults remain unchanged.

## Boundary and compiler change

The first live compilation rejected function 39, `neoCLR.Runtime.ScheduleTask`.
The retained linked report identifies `neoclr_schedule_task_managed_v1`; the
initial rejection is archived. The native adapter delegates to scheduler submission:
it creates a temporary owner host root, reads the callback under the published
boundary, creates the runnable's independent host root, queues native storage and
releases the temporary root. It does not execute managed code inline, mutate
managed payloads, or write to unit result storage. Failed queue allocation/root
creation unwinds before publication. Scheduler consumption/cancellation retains
its existing owner-local lifetime rules. The live collector captures initial roots
and remarks current host roots; this is not concurrent cross-owner queue access.

neoCLR codegen now admits **only** this additional body and its exact three-pointer,
Int32-status native signature. Existing generated wrapper publication and guarded
result handling remain in place. The first focused test also caught the missing
native-signature admission, which was added before successful qualification.
Other unreviewed scheduler/provider calls remain rejected. Compiler tests cover
sync/live wrapper emission, existing queue/timer wrappers and unknown-provider
rejection. No Raven syntax, semantic model, CLI metadata encoding or Runtime
Contract configuration changes. The existing internal-call bridge still maps the
managed scheduling operation to the private native provider; eventual native
metadata must express that provider capability explicitly.

The [isolated worker ownership contract](../../../docs/experiments/http-worker-integration/README.md)
and [.NET comparison](../../../docs/design/native-http-worker-threads.md) remain:
each owner runs the whole application with its own heap/statics/tasks/socket scope
and pinned async stacks. Only the kernel listener is shared through descriptor
duplication. `Task.Run` still queues work on that owner; it is not .NET shared-heap
thread-pool dispatch. Replicated startup/state and heap capacity remain costs.

## Qualification

`build.json.gz` retains sync/live object selection and one/four-owner host commands.
The synchronous object from the initial attempt is reused explicitly (same sample
and selection); the admission-only change does not alter its code path. Live uses
the rebuilt release test runner and current native sources with existing scalar
validation, allocation index and inline lookup options. The original failed build,
compiler tests and source digests are retained. The driver can recompile both
objects instead of using `--sync-object`.

`validation.json.gz` passes all four configurations. Each performs three runs of
256 checked responses, eight pending incomplete reads during useful requests,
shutdown with those reads pending, and an injected guest fault that terminates the
whole group. Each normal response awaits `Task.Run` and checks its result. Every
owner serves responses and closes its private control listener. Normal group exit
is zero; injected-fault exit is two. This is **3,072 checked normal responses** plus
four failure runs. Live one/four-owner cases complete 457/1,137 epochs respectively
across normal and fault runs. Each normal live owner completes positive epochs;
all report balanced started/completed/cancelled accounting, no active candidate,
and zero retained index metadata after cleanup. Diagnostics are emitted on their
owning thread, under a stderr lock to avoid interleaved multi-line reports.

`sanitized.json.gz` adds a four-owner live build with undefined/bounds sanitizers:
768 more checked responses, pending-read shutdown and group failure propagation,
806 completed epochs and the same per-owner cleanup checks. In total these runs
validate 3,840 responses and 2,400 live epochs. This is not TSan qualification;
previous race-sanitizer startup problems remain open. No ASan or arbitrary native
provider/interleaving guarantee is claimed.

These are lifecycle/safety runs, not throughput benchmarks. The fixture includes
an awaited task and dynamic owner identity in every response; it is not the minimal
plaintext baseline. Process CPU snapshots printed by each owner are process-wide
cumulative counters and must not be summed as per-owner CPU. No allocation, CPU,
pause or throughput improvement is inferred from passing the requests.

## Reproduce

Build the current release `neoclr-codegen` test runner using the toolchain recorded
in `compiler-tests.json.gz`, then supply its path:

```sh
python3 benchmarks/execution/concurrent-http-owners/build.py \
  --runner target/release/deps/neoclr_codegen-12f33f26d2ff4df0 \
  --output target/concurrent-http-owners-new
python3 benchmarks/execution/concurrent-http-owners/validate.py \
  --build target/concurrent-http-owners-new --output target/concurrent-http-check-new
```

Use fresh directories. The builder consumes retained source-selection/link recipes
and the existing worker fixture assembly; input hashes identify them. The sanitizer
archive records its complete relink and validation commands. No compiler binary is
published or production default changed by this experiment.

Next establish matched minimal plaintext and JSON owner workloads, then measure
one/four-owner sync/guarded-sync/live at equivalent pacing with aggregate process
CPU, allocations, memory and pause distributions. Retain standalone tasks,
collections and arrays to separate HTTP scheduling cost from common GC services.
JIT multi-owner hosting and shared-heap HTTP admission require independent evidence;
this AOT checkpoint does not imply either. Default promotion remains open.

## Performance follow-up

[Plaintext measurements and the connection-close stall](measurement-results.md)
retain synchronous/live CPU, allocations, RSS, client phase timing and both passed
and incomplete cohorts. The live CPU penalty remains; EOF failures prevent a
clean four-owner capacity claim or default promotion.

See the [native close investigation](close-results.md) for corrected NoDelay evidence,
EOF-phase timing, bounded traces and a timeout correlated with prompt native close.

The [guarded CPU comparison](cpu-results.md) separates automatic marking from the
remaining checked-store/generated-code cost and retains separate loaded stack profiles.

[Diagnostic counter removal](counter-results.md) did not demonstrate a performance
gain across plaintext, tasks, collections and arrays; counter-enabled behavior stays.
