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

## Server cycle witness

The recursion diagnostic now preserves original member/owner names and compiled
indices. The full Server inspection reports:

```
Raven.Generated.UnitCallback::Invoke [#7]
 -> $closure$0::Invoke [#5]
 -> System.Web.Http.HttpServer::Close [#211]
 -> System.Concurrency.CancellationTokenSource::Cancel [#75]
 -> System.Concurrency.CancellationRegistration::Invoke [#83]
 -> Raven.Generated.UnitCallback::Invoke [#7]
```

These indices belong to this selection only. Interface/callable dispatch includes
all admitted targets, so the graph witness does not establish infinite recursion
in the running application. It does identify a path the backend cannot currently
bound. Admission remains unchanged: removing the guard requires either sounder
receiver/target analysis or a native stack-budget contract. The focused callback
rejection test checks the repeated member and cycle separator; the full Server
inspection supplies the integration witness. No native HTTP execution is claimed.

The guarded-stack and unit-entry follow-ups now allow full Server compiler admission.
The cycle above remains useful evidence about conservative dispatch. Default admission
still rejects recursion; `--native-stack-budget` provides the guarded path. Automatic
host queue draining and a real native HTTP request remain unvalidated.

## Private host Drain entry (2026-10-08)

With --bind-task-queue, a source-owned TaskQueue already selected by the application
retains its unique ordinary instance Drain() as an explicit host reachability root.
Specialization and closed-world selection include its body/dependencies; no fake call
is inserted into guest CIL. The inspection inventory records hostRoots. Generic,
ambiguous or nonordinary Drain contracts are rejected. Original scope verification
still precedes this private host projection.

The generated `neoclr_drain_default_queue_v1(context)` invokes that selected body
without resetting the heap. A missing default queue is a successful no-op. The C reader
requires a live task scope, no existing fault and no active guest frame for that
context. The registered queue's host handle roots it across collection. Stack-budget
images guard this entry too. Guest faults use ordinary capture; host misuse returns
3 and preserves any prior fault. Hosts drain after entry and after completion dispatch,
then release the task scope on terminal success/fault. This does not add a scheduler,
fairness, preemption or an instruction budget for an endlessly self-posting callback.

QueuePump posts work that posts more work without calling Drain in guest source;
QueuePumpFault faults after Main returns. The same Raven artifacts pass interpreter,
sanitized native and standalone native execution, including exact output/fault parity
and task-root/frame cleanup. The task-scope kernel checks quiescence, missing queues
and prior-fault rejection. [Evidence](../../../benchmarks/native-web/queue-pump-validation.json)
records a repaired duplicate inspection flag separately from passing consumers.
The full Server also passes compiler admission with the retained Drain body; an actual
native HTTP request is the next check.

This is explicitly a compatibility service adapter. The author reiterates a future
runtime Scheduler for green threads and [cross-cutting services](../../runtime-scheduling-design.md#cross-cutting-runtime-services--author-clarification-2026-10-08)
across interpreter/AOT/JIT. TaskQueue layout and this export do not define that future ABI.

## Queue-only async entry drain (2026-10-08)

`--bind-task-queue` also admits the exact `neoCLR.Runtime.DrainEntryTasks() -> Void`
service. The generated wrapper invokes the retained source `TaskQueue.Drain` while
startup remains on the published native frame stack. This is a distinct entry lifecycle
path; the quiescent host export retains its existing restrictions. Missing queues are
successful no-ops. The private task scope rejects nested entry draining, calls from
active Run/Drain callbacks, and calls outside the selected entry. Its guard is released
on both successful return and guest fault, preserving output atomicity and fault frames.
Startup locals and the default queue remain roots throughout callback collections.

This first path supports queued work only. Images selecting entry draining together
with socket completion services are rejected with an explicit host-I/O diagnostic.
They must not read an entry task's result before its host completion arrives. Native
host waiting/resumption, fairness, cancellation budgets and a future Scheduler remain
separate work; this change neither adds a blocking wait nor relaxes host callback entry.

Compared with .NET's async entry wrapper, the ergonomic goal is still an entry that
returns only after its task completes. neoCLR currently achieves queue-only progress
by running its explicit queue and then using its existing result/fault contract. An
unresolved promise faults; it is not an implicit indefinite wait. This reuses the
interpreter's entry-drain lifecycle described in the existing
[scheduling research](../../runtime-scheduling-design.md), with a narrower native
admission boundary. Native code keeps live C frames, so this is not the future green
thread or stackless suspension representation. No public API or compiler mapping changes.

Focused validation includes exact service admission/rejection, sanitized scope/root
and guard cleanup checks, and executable comparisons in the
[native-web workbench](../../../benchmarks/native-web/README.md).

All four async entry cases and the existing post-entry queue-fault regression pass
interpreter, sanitized native and standalone native execution, with exact fault/output
comparison and libSystem-only standalone dependencies.
[Commands, hashes and outcomes](../../../benchmarks/native-web/async-entry-validation.json).
Rebuild matched native images and C adapters together; the task scope and generated
entry helpers remain private experimental ABI.
