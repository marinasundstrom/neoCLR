# Execution-local garbage collection API

Author-directed development slice, 2026-09-27. `System.Runtime.GC` is a static
class with basic information and control over the current execution's managed heap.
This is a provisional API, not part of Preview 10.

## Contract

All six read-only properties return `long` and query without collecting or allocating
tracked heap objects:

| Property | Meaning |
| --- | --- |
| CollectionCount | Completed collections in this execution |
| AllocatedObjectCount | Total tracked allocations since execution started |
| HeapObjectCount | Objects retained in the heap, including uncollected garbage |
| PeakHeapObjectCount | Highest retained object count observed |
| ReclaimedObjectCount | Total objects removed by completed collections |
| HeapObjectLimit | Host-configured object limit, unchanged by collection |

Counts concern tracked objects, including reference arrays and boxes, not bytes,
inline values, native allocations, all string backing buffers or process memory.
Counters saturate at `long.MaxValue` if their host representation exceeds that range.
Automatic, explicit and final execution collections contribute to the host totals;
guest code cannot observe its own execution-completed collection after it returns.
Each invocation and isolated worker has its own heap and counters.

`Collect() -> ()` synchronously performs one full mark/sweep collection before
returning. It preserves frame arguments, locals, evaluation stacks, interior
references, objects under construction, scheduler roots, the registered default
queue and any pending invocation result. It uses the same root walker as ordinary
allocation-pressure collection and updates the next automatic collection threshold.
It does not run queued guest callbacks, finalizers or isolated workers, alter limits,
compact objects, promise an OS memory reduction, or suppress later automatic GC.
Repeated calls consume the ordinary instruction budget.

`KeepAlive(value: object?) -> ()` expresses that the reference must remain reachable
through the call. Null is a no-op. It neither pins objects nor owns native resources
and gives no post-call lifetime guarantee. Current interpreter locals are already
conservative roots; future optimizing backends must preserve this call's lifetime
semantics. Explicit collection does not shorten the lifetime of still-rooted locals.
No API here returns `Result`: these operations have no expected input-validation
failure. Runtime faults and resource/cancellation limits retain normal behavior.

## Comparison and provisional design decision

Primary sources reviewed 2026-09-27:

- [.NET 10 GC.Collect](https://learn.microsoft.com/en-us/dotnet/api/system.gc.collect?view=net-10.0)
  supplies a blocking full-collection overload alongside generation, mode and
  compaction options. neoCLR retains the familiar Collect action, without settings
  its non-generational, non-moving collector cannot implement.
- [.NET GC APIs](https://learn.microsoft.com/en-us/dotnet/fundamentals/runtime-libraries/system-gc)
  include memory information, generation queries and KeepAlive. neoCLR's requested
  Runtime namespace and object-count properties are deliberate differences. Byte
  reporting would currently hide substantial untracked/inline allocation details;
  generation numbers would be invented. The cost is less .NET source compatibility
  and less detailed memory diagnostics. This is narrower capability, not a claim
  that neoCLR collection performs better.
- [Go runtime](https://pkg.go.dev/runtime#GC) also offers a blocking explicit GC;
  [MemStats.HeapObjects](https://pkg.go.dev/runtime#MemStats) distinguishes object
  counts from byte counters. Its [KeepAlive](https://pkg.go.dev/runtime#KeepAlive)
  provides an explicit reachability point. These are useful comparisons rather than
  promises of Go's runtime, cleanup or concurrency behavior.

Alternatives were a .NET-shaped byte/generation facade, a new statistics snapshot
type, or continuing with host-only statistics. Scalar properties reuse existing
counters without a new allocation or snapshot carrier; separate reads are not an
atomic snapshot across intervening guest allocations or collections. A richer
snapshot can be considered when byte accounting or concurrent heap operation exists.
No independent library can safely request a VM collection or enumerate all roots
without runtime support; no alternative .NET library implementation was evaluated
for this minimal boundary. The API shape remains provisional.

## Implementation and validation

Raven implements the public facade. Exact private InternalCall signatures expose
scalar counters, collection and lifetime use; the VM owns heap traversal and limits.
No new opcode or Raven Runtime Contract option is introduced. Private calls use the
existing inhabited Void service ABI; the public wrapper has no CLI return value.
ManagedHeap is the reported runtime service requirement. Host code exhaustively
matching CollectionReason must handle the new ExplicitRequest case.

Focused tests cover counter accuracy and execution isolation, explicit reclamation,
root survival, inaccessible malformed signatures and heap-limit enforcement. The
[Raven consumer](experiments/runtime-gc/Main.rvn) uses the public class. Existing
GC tests exercise transitive graphs and automatic collection. Future byte accounting,
generations, finalizers, tuning, asynchronous collection and no-GC regions remain
outside this slice; they require separate collector work and measurement.

### Recorded evidence — 2026-09-27

Eight GC API tests, eight existing collection tests and three diagnostic tests pass.
The combined focused run also passes nineteen Reflection checks and fifteen
interface checks, for 53 native tests. Sixteen exact GC public signature checks and
26 Reflection signature checks pass. The GC consumer compiles against the public
class and verifies reclamation, counter deltas and retained values. The website
build checked 1,254 pages, including both feature pages and API reference links.
Raven integration documentation is commit `d81235208` on its neoclr branch.
Publication is a separate operation and was not performed by this work.

After integrating the concurrent numeric and interface work, both Reflection
consumers, the GC consumer, Number/concrete parsing, interface helpers and explicit
interface consumers pass against the same regenerated library/reference boundary.
Library and API snapshot checks pass. The integration preserves source-qualified
interface method names and reflected static application method ownership.

The final Result/Task entry integration passes five entry-dispatch native checks
and repeats all eight GC API checks. A second GC consumer collects before awaiting,
inside a queued callback and after async Main resumes: captured objects survive,
and Collect itself does not pump callbacks. Pending integer and Task<Result> error
entries also pass. The public reference assembly is byte-identical across this last
integration; library hashes and updated website sample templates pass checks.

## Generational collection evaluation — 2026-10-13

**Exploratory next architecture candidate, not implemented behavior.** The author
suggested that a generational collector could relieve collection pressure and
latency. This complements the allocation/root-admission work; it does not establish
that the current workloads have the lifetime distribution needed for a net gain.

The [.NET generation model](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/fundamentals#generations)
collects younger portions more often and promotes survivors. The
[CLR collector design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md)
explains the allocation/collection split and generational machinery. These describe
shipped CLR behavior, reviewed 2026-10-13, rather than a neoCLR implementation plan.
Existing [Go/V8 comparisons](../benchmarks/http-scheduling/README.md) remain relevant:
generations, concurrent collection and collection pacing are different choices.
A generational collector can still pause for a full collection.

The intended benefit is less tracing **and** reclamation work during young
collections, especially with a large stable live graph and many short-lived
allocations. Merely sweeping young blocks while retracing the full old graph would
leave much of that cost. Avoiding the old graph requires recording references from
old objects into young ones, normally through write barriers and a remembered set.
Costs include reference-store overhead, remembered-set memory, promotion work and
more collection paths to validate. Generations alone do not remove the current
native allocator's whole-heap integrity walk.

Compare the current nonmoving full collector with a provisional nonmoving young/old
prototype before considering a copying nursery. Nonmoving generations could preserve
current address stability but retain fragmentation; relocation would require precise,
updatable roots or explicit pinning/handles for interior and borrowed native pointers.
Neither alternative is selected yet. A public generation API is premature.

The next bounded evaluation should:

- Measure survival by allocation age, bytes retained, collection duration distributions
  and full-GC frequency on allocation churn, stable graphs, mixed lifetimes, suspended
  operations and HTTP. Measure promotion volume once a prototype actually promotes.
- Audit reference writes in interpreter, JIT, AOT and native helpers, including arrays,
  bulk copies and references inside value types. A missing old-to-young edge is a
  correctness failure; runtime roots and suspended frames must also remain visible.
- Test old-to-young updates, cycles, promotion, large allocations, interior references,
  ownership abandonment and exactly-once cleanup. Keep a full-collection oracle and
  stress mode; compare retained objects and complete final reclamation.
- Compare throughput, pause percentiles/maxima, peak memory and barrier cost against
  the full collector and safe pacing separately. Include high-survival and frequent
  mutation controls that might lose. Keep macOS/Windows and all execution modes explicit.

Finish helper-root coverage and scheduler idle cleanup qualification independently;
those correctness requirements remain relevant under any collection architecture.

### Review of the supplied collector overview — 2026-10-13

The author supplied a ChatGPT overview recommending precise tracing, fast local
allocation, generations, parallel collection, concurrent marking and eventually
relocation. Treat that ordering as an exploratory proposal, not an adopted sequence.
neoCLR already has tracing/root machinery: the interpreter uses Rust-managed state
and native JIT/AOT support includes a C collector. It is not a single all-Rust heap
waiting for its first collector.

Primary-source checks support separating the choices. The [Go runtime collector](https://go.dev/src/runtime/mgc.go)
describes concurrent non-generational, non-compacting mark/sweep and size-segregated
per-processor allocation areas. [.NET background GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/background-gc)
allows generation-2 work alongside application execution while ephemeral collections
still require suspension. These are shipped designs, reviewed on this date; neither
establishes the best neoCLR collector. Generations are not a prerequisite for concurrency,
and concurrent work consumes CPU/bandwidth and requires mutation/root coordination.

The provisional evaluation remains survival/pause/allocation measurement followed
by a nonmoving generation candidate against the current full collector and safe
pacing. Fast allocation can be evaluated independently. Worker-local allocation
should follow actual heap/worker ownership; a buffer per green thread is not implied.
A copying nursery would need updatable references or pinning/handles, including
suspended activations, interior references and borrowed native inputs. Merely adding
an abstraction around pointers does not establish relocation correctness.

For the server workload, compare throughput **subject to explicit tail-latency and
memory limits**. Report unfavorable high-survival/mutation cases too. This does not
select multiple shipping collectors, a public barrier ABI or a new default policy.

## Native exact-base admission — 2026-10-13

Host handles, resource owners, callback entry and owned suspended roots retain
neoCLR's full heap-integrity check before accepting an exact allocation base.
Validation and membership lookup now share one walk; after matching the requested
bases, the rest of the walk performs integrity checks only. Matching an early block
cannot hide trailing corruption, and failed calls do not publish outputs or mutate
registrations. Null values remain valid strong roots; owners must be live exact bases.

This is an internal optimization of the existing bounded host-handle contract,
comparable in purpose to CLR native handle rooting, but not an implementation of
CLR's handle tables or a new managed GCHandle API. No persistent lookup structure,
write barrier, GC policy or public ABI is introduced. Costs remain linear in heap
blocks; a trusted persistent index would need a separate integrity/lifetime design.
The [matched admission benchmark](../benchmarks/gc-handles/README.md) includes early
bases and small heaps so repeated-walk savings are not confused with universal gains.
The full collector, suspension ownership and non-GC arena behavior are unchanged.


## Concurrent implementation investigation — 2026-10-18

The author now explicitly directs work through concurrent collection and asks for
research beyond the CLR. See the [collector alternatives and validation gate](design/concurrent-gc-evaluation.md)
for Go/Green Tea, Shenandoah, LXR and recent scheduling research. This author-directed
work takes precedence over the earlier provisional generational evaluation order.
Concurrency, locality, generations and pacing remain distinct choices; no new collector
is shipped or selected by this research entry.
