# ArrayList filtering in the Raven profile

The 2026-09-13 prototype provides eager searches and filtering directly over
ArrayList<T> storage. It complements LINQ composition when the concrete collection
is available. This is library code using existing IL, delegates and union metadata;
no Raven compiler or runtime instruction change is required.

| Method (all take Func<T, Boolean>) | Result | Search and empty behavior |
| --- | --- | --- |
| Find | Option<T> | First match, or None |
| FindLast | Option<T> | Last match, or None |
| FindIndex | Option<Int32> | First matching zero-based index, or None |
| FindLastIndex | Option<Int32> | Last matching zero-based index, or None |
| Exists | Boolean | Stops at first match; false for empty input |
| TrueForAll | Boolean | Stops at first failure; true for empty input |
| FindAll | ArrayList<T> | All matches in source order; independent empty list if none |

FindLast and FindLastIndex invoke predicates in reverse order. Other methods scan
forward. Scalar searches short-circuit; FindAll invokes the predicate for every
visited element. Absence is not a default element or a negative index. Boolean
questions and empty filtered lists already have complete outcomes, so these methods
do not add Result wrappers. Predicate/runtime faults remain terminal faults.
Range overloads, RemoveAll and fallible-predicate overloads are outside this slice.

## Storage, callbacks and allocation

Each invocation retains the initial backing array and count, following the existing
predicate-search extent policy. Appends are not included. Writes to that captured
buffer can affect elements not yet visited; growth can replace the source's buffer,
while the active search retains its original buffer. This is not a snapshot of all
element values and not a concurrent collection guarantee. A returned matching value
is the value passed to the predicate, even if the predicate replaces that slot.

FindAll creates separate list storage and shallowly copies matches: reference
values still refer to the same objects. Changing a result slot does not replace a
source slot; mutating an object reached through either list remains shared.

Scans take O(n) predicate invocations in the worst case. Scalar operations allocate
no managed iterator or query objects. FindAll allocates its result and backing
storage, growing as needed. Predicates, captures and copied values can have their
own costs. Null callbacks fault when invoked; empty scans do not invoke them.
There is no new eager argument-validation contract or fault cleanup mechanism.

The equivalent lookup in `tests/query_terminals.rs` now measures four managed
allocations for Find versus nine for Where(...).First(), including common setup.
Before this slice Find measured five because it used an iterator. These are
interpreter managed-object counts, not host allocations, bytes or elapsed speed.
Prefer suitable concrete operations to avoid unnecessary query objects; retain
LINQ for composition and interface access. Future LINQ specialization is permitted.

## .NET comparison and provisional choices

Primary sources consulted 2026-09-13: shipped .NET 10
[List<T>.FindAll](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1.findall?view=net-10.0)
returns a list, including an empty list for no matches;
[FindIndex](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1.findindex?view=net-10.0)
uses -1 for absence. The method family and delegate-based predicates are familiar,
while neoCLR uses Option for both element and index absence. The local
[.NET comparison](experiments/list-filters/dotnet/Program.cs) exercises matching,
missing, empty, reverse-search and independent-result cases on net10.0.

Keeping -1 would reduce migration and case-handling overhead, but would preserve a
second absence convention alongside Find's Option. Choosing Option<Int32> makes
absence explicit at the cost of a changed binary signature and more explicit use.
Using LINQ internally would reuse iteration machinery but add objects in this
prototype; direct scans require additional library loops and regression coverage.
A new intrinsic is unnecessary. This choice remains an experimental API policy,
not a claim that the CLI needs new semantics.

The broader comparison in the [query API review](raven-query-api.md#concrete-collection-operations-versus-linq)
includes .NET's applicable direct-access guidance and LINQ specialization;
its terminal review also considers Rust Option and alternative .NET Maybe APIs.
That evidence supports explicit absence as a library design rather than a new
runtime mechanism. It does not establish universal speed gains. Mutation during
callbacks follows the bounded neoCLR policy above, not a promise to reproduce
.NET List's behavior under mutation. Usability, range overloads and specialization
remain subjects for later measurements and API feedback.

## Migration and validation

FindIndex previously returned Int32 with -1. Recompile callers against matching
regenerated core metadata and System library; existing binaries with the old
signature are not compatible. For example:

```raven
import System.Option.*

match values.FindIndex((value: int) -> bool => value == 42) {
    Some(let index) => WriteLine(index)
    None => WriteLine("No index")
}
```

The [Raven sample](experiments/raven-target/samples/library-list-filters.rvn) covers
all operations, destructuring, independent results, reference sharing and callback
growth. Direct IL tests in `tests/list_filters.rs` cover order, short-circuiting,
empty/missing results, allocation counts, predicate faults and retained buffers
across collection growth and GC. Metadata probes verify closed result signatures
and reject forged scalar returns. Editor checks exercise method completion.
The historical Neo profile retains its old API; this experiment targets Raven.
The installed .11 SDK/extension has not been refreshed by this source slice.

Source validation passed 63 saved-project cases, 12 application cases, 104 signature
checks, 59 editor checks and four direct filtering tests, plus Clippy, formatting
and the API audit. The .NET comparison targets net10.0. These checks validate the
bounded prototype, not a complete .NET List API or a new distributable release.
