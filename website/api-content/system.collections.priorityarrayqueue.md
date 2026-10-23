---
uid: T:System.Collections.PriorityArrayQueue`2
---
## Development priority queue

`PriorityArrayQueue<TElement, TPriority>` is an unsynchronized binary min-heap.
Pass a `Comparer<TPriority>` to the constructor; `FunctionComparer<TPriority>`
adapts a callback. Comparison uses the sign of its result. Lowest priorities leave
first; reversing the comparer produces a max-heap. Equal priorities have no FIFO
promise. The comparer and queued priority values must retain a consistent ordering.

`Peek` and `Dequeue` return `Optional<TElement>`; `PeekPriority` returns
`Optional<TPriority>`. Empty queues return `None`. `Count`, `Capacity`, and `Comparer`
expose the current count, storage extent, and supplied policy. `Clear` removes all
entries while retaining capacity. `EnsureCapacity` reserves slots without shrinking;
negative capacities and comparer reentry into queue operations cause terminal Faults.
Provider Faults are terminal, with no recovery/rollback guarantee. This is not a
concurrent collection; simultaneous access requires external coordination.

Insertion and removal take O(log n) comparisons, peeking O(1), and clearing O(n).
Storage uses parallel element and priority arrays, with geometric automatic growth.
There are no per-entry heap-node wrappers. Growth copies the live entries into two
new arrays, and removal clears vacated references. Returned Optional values and any
allocations in the supplied comparer follow their normal runtime representation;
this is not a blanket zero-allocation guarantee.

Compared with .NET's PriorityQueue, the min-priority and unspecified tie-order
semantics match. neoCLR follows its existing explicit comparer and Optional-returning
collection contracts, and names the array implementation. A binary heap keeps the
initial implementation small; .NET uses a quaternary heap. No performance advantage
over that implementation has been established. Sorted indexing, stable tie-breaking,
bulk heapification, iteration, and arbitrary removal are not provided in this increment.
