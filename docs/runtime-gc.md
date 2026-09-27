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
