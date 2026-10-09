# Queue and Stack review — 2026-10-10

Status: proposed library contracts and a passing .NET comparison, not implemented
neoCLR APIs. The author requests useful collection additions after basic JSON and
explicitly clarifies that TaskQueue is not necessarily connected to this work.
Queue and Stack are general-purpose collections; neither scheduler migration nor
TaskQueue replacement is a requirement or justification for this slice.

## Scope and current library

FIFO worklists support breadth-first traversal; LIFO worklists support iterative
depth-first traversal and explicit pending-work stacks. These are suitable small
consumers without introducing runtime scheduling. Current ArrayList has Add,
indexed access, searches and Copy, but no removal or Clear. Existing Collection
promises Count and iteration; Sequence additionally promises indexed access.
Queue/Stack need not implement Sequence or change List's existing contracts.

The recommendation is to prototype Queue first, then Stack using the same storage
findings. Other candidates (sets, deques, priority queues, persistent collections)
remain outside this bounded review until a concrete consumer selects them. Existing
list/map removal gaps deserve separate review; this is not a complete collections
redesign. See the [implemented hierarchy](../../collection-contracts.md).

## Evidence and alternatives

Sources inspected 2026-10-10:

- .NET 10 [Queue source](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Collections/Generic/Queue.cs)
  uses a circular array; [Stack source](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Collections/src/System/Collections/Generic/Stack.cs)
  uses a growing array. Both offer throwing and Try removal/peek APIs, clear removed
  reference-containing slots, and detect mutation during enumeration. These are
  library policies, not new CLR instructions.
- In [API discussion 80327](https://github.com/dotnet/runtime/issues/80327#issuecomment-1377454266),
  maintainer eiriktsarpalis cautions against abstractions without multiple BCL
  implementations and asks for evidence before bulk-operation optimization.
  [Abrynos's experience](https://github.com/dotnet/runtime/issues/80327#issuecomment-1431563774)
  describes needing a custom priority-queue interface to substitute implementations.
  That is evidence of a substitution scenario, not proof every queue needs an
  interface. The issue closed for inactivity, not an accepted interface contract.
- Rust's [VecDeque](https://doc.rust-lang.org/std/collections/struct.VecDeque.html)
  provides ring-buffer FIFO operations with Option removal results. Its ownership
  and borrowing rules do not transfer to managed aliases or GC retention.
- C5 at `224386389d83bce81a55f9da304a95d18be15aac` supplies
  [IQueue](https://github.com/sestoft/C5/blob/224386389d83bce81a55f9da304a95d18be15aac/C5/Interfaces/IQueue.cs),
  [IStack](https://github.com/sestoft/C5/blob/224386389d83bce81a55f9da304a95d18be15aac/C5/Interfaces/IStack.cs)
  and [CircularQueue](https://github.com/sestoft/C5/blob/224386389d83bce81a55f9da304a95d18be15aac/C5/Arrays/CircularQueue.cs).
  This demonstrates library-only capability interfaces, but its indexed contracts
  are broader than needed here. The inspected Dequeue leaves its old slot occupied;
  that retention policy should not be copied into neoCLR.

Keeping concrete-only .NET-style types would be simplest and avoids interface
versioning costs. Small separate contracts fit neoCLR's existing capability model
and the author's interface candidate, at the cost of extra public types and dispatch
coverage. Linked storage avoids wraparound but allocates nodes; a full deque exposes
more operations than these scenarios need. Neither alternative is excluded forever.
No speed or allocation improvement is claimed without measurements.

## Provisional contract for the prototype

| Contract | Proposed members and behavior |
|---|---|
| Queue<T> : Collection<T> | Enqueue(T), Dequeue() -> Option<T>, Peek() -> Option<T>, Clear(); FIFO iteration |
| Stack<T> : Collection<T> | Push(T), Pop() -> Option<T>, Peek() -> Option<T>, Clear(); top-first iteration |
| ArrayQueue<T> | Growable circular buffer; empty/capacity constructors; Capacity |
| ArrayStack<T> | Growable contiguous buffer; empty/capacity constructors; Capacity |

Keep type parameters invariant. Duplicates are accepted. Empty reads/removals return
None without mutation; a stored nullable element remains Some(null). Peek preserves
Count. Clear releases held element references. Capacity is not a maximum backlog:
growth must check overflow and must never silently overwrite elements. These are
ordinary unsynchronized collections, with no waiting, cancellation or backpressure.

Option results deliberately differ from .NET's Try/out and throwing pair: they fit
existing Map.Find and ArrayList.Find, but add union representation/dispatch costs.
The prototype should start with snapshot iterators to make alias behavior explicit:
iteration captures values in removal order, shallowly, without consuming the source.
This costs allocation/copying and retains captured references until iterator release;
it avoids version-counter semantics and must not be described as .NET parity.
Compare this with fail-fast iteration before production adoption. Existing ArrayList
captures its buffer/extent with visible edits; its behavior is not changed here.

## Storage question and executable next slice

CheckedStorage currently exposes Reserve only; reads of unwritten elements remain
checked. Clearing a generic non-nullable T slot by manufacturing null would violate
that contract. First prototype occupied slots as Option<T>, explicitly initialize
unused slots to None and replace removed slots with None. This keeps occupancy
in ordinary library values and avoids a new runtime service. Its costs are tag/
payload storage and initialization work; reference release through union storage
must be tested in both backends rather than assumed. If this fails admission or
retains references, investigate a narrow checked-storage operation separately,
including forged-IL safety, instead of weakening array reads.

Acceptance: empty and nullable elements; FIFO wraparound and growth; LIFO order;
peek/count; duplicate and reference identity; Clear/reuse; snapshot alias behavior;
invalid capacity and overflow checks; reference release under GC. Exercise ordinary
interface calls with Int32, reference and admitted value elements, then a small
breadth/depth-first traversal consumer. Require matching interpreter/native results,
public API docs and Windows/macOS qualification before calling the additions ready.
Do not add automatic JSON collection admission: Stack round-trip ordering and any
new construction policy require their own serializer slice.

## Comparison execution

`dotnet run --project docs/experiments/queue-stack/dotnet/Comparison.csproj -c Release`
passed on macOS arm64, SDK `11.0.100-rc.1.26425.128`, runtime `.NET 10.0.0`.
The committed probe checks FIFO wrap/growth, LIFO iteration/removal, non-removing
peek, mutation rejection, Clear, empty Try results and present-null distinction.
This is baseline evidence, not a neoCLR implementation or performance benchmark.
