# Native TaskQueue compatibility adapter

Development experiment, 2026-10-08. `--bind-task-queue` requires native GC and binds
verified closed instantiations of RegisterTaskQueue, GetDefaultTaskQueue and
GetCurrentTaskQueue. The source-owned nominal queue and its unique Drain contract
remain ordinary library metadata/CIL. A single queue type is admitted per image.
Managed impostors, mismatched signatures and unsupported owners are rejected.

An explicit host task scope retains the default queue through a strong GC handle.
Registration is allowed once, with a non-null live reference. Default lookup returns
null before registration. Current lookup walks the published frames for verified
TaskQueue.Run/Drain method indices and reads their receiver; otherwise it returns the
default. This preserves the interpreter's current explicit-queue behavior rather than
silently substituting the default. Scopes are thread-affine and nested LIFO across
separate contexts. Teardown rejects active guest frames, releases the queue root and
must occur on terminal success/fault paths before resetting the invocation.

This is deliberately a compatibility adapter, not the future scheduler. It neither
selects runnable work nor pumps the default queue after entry. Host-driven Drain
selection/dispatch remains the next integration step. The index/stack-slot convention
is private to the matched image and GC adapter; future runtime activations must replace
it when resumption no longer has live native frames. See the existing
[.NET affinity comparison and cross-runtime reassessment](../../runtime-scheduling-design.md).
No public Task, Function or queue semantics are changed, and there is no promise of
thread transfer, green threads, preemption or caller-affinity changes.

The native-body root inventory also now recognizes the recently added socket and
ordinal bindings, so their argument publication/reporting uses the native-wrapper path.
These services do not themselves collect, but their metadata must accurately describe
which CIL bodies are replaced. No guessed wrapper locals are used as roots.

## Validation and remaining gaps

- The sanitized task-scope kernel checks registration/null/duplicate failure, default
  retention through GC, active explicit Run/Drain receivers, context filtering, nested
  scopes, rejection during active frames, cleanup and canaries.
- Exact binding unit tests cover closed source ownership, missing Drain, mismatched
  types/body/flags and missing type arguments.
- The real Raven [TaskQueue consumer](../../../benchmarks/native-web/TaskQueue.rvn)
  passes interpreter, sanitized native and standalone native execution. An explicit
  queue runs a Promise continuation before the default queue drains; default identity
  and final counters agree. The native executable depends only on libSystem and releases
  all managed allocations after task-scope cleanup. [Recorded evidence](../../../benchmarks/native-web/task-queue-validation.json).
- The compiled deadline-aware echo regression passes after native-root classification
  changes. Full Server now gets past queue service binding and reaches the existing
  recursive-call rejection; it has not executed natively.

The probe uses a reference payload/capture to isolate queue ownership. An initial
mutable captured-local form was rejected by Raven native emission, and Promise<int>
reached unsupported ArrayRef<Int32> specialization. Those are deferred compiler/backend
candidates, not solved by this adapter. Do not infer support for them from this consumer.
No benchmark was added: the current claim is behavioral parity; scheduler/HTTP timing
still requires an executing server and representative completion workload.
