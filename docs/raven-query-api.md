# Prototype query API for Raven

The Raven-target runtime library now exposes `System.Linq.Enumerable` extension
methods over `System.Collections.Iterable<T>` and, after Preview 5, vector arrays:

| Method | Result | Evaluation |
| --- | --- | --- |
| `Where<T>(Iterable<T>, Func<T, bool>)` | `Iterable<T>` | Predicate runs while advancing an iterator |
| `Select<T, U>(Iterable<T>, Func<T, U>)` | `Iterable<U>` | Selector runs once per produced element |
| `ToList<T>(Iterable<T>)` | `ArrayList<T>` | Consumes the sequence immediately into a new list |
| `First<T>(Iterable<T>)` | `Option<T>` | Reads at most the first element |
| `Last<T>(Iterable<T>)` | `Option<T>` | Consumes the sequence, retaining the last element |
| `Single<T>(Iterable<T>)` | `Result<T, SingleError>` | Requires exactly one element; stops on a second element |

Import `System.Linq.*` to use member syntax. See the readable
[query sample](experiments/raven-target/samples/library-queries.rvn). Raven infers
type arguments from the receiver and callback; no explicit reference operators
are needed. These are generic library methods with validated bridge bindings,
not support for arbitrary generic application method bodies.

## Array receivers after Preview 5

Managed vectors implement `System.Collections.Iterable<T>` in the runtime. An array
can be assigned to that interface, passed to an Iterable parameter or returned as an
Iterable value. The conversion retains the original array reference; it does not box,
copy elements or allocate a sequence wrapper. Casting the view back to its exact
vector type preserves identity. Each `GetIterator()` returns independent position
state retaining the original buffer. Changes to unvisited elements remain visible.
`ToList()` copies elements into independent storage while preserving object references.

Consequently, `typeof(int).GetMethods().ToList()`, `Where` and `Select` use the same
Iterable extension methods as lists. No array-specific query overloads are needed.
See the [array query sample](experiments/raven-target/samples/library-array-queries.rvn).

This follows [.NET's vector enumeration pattern](https://learn.microsoft.com/en-us/dotnet/api/system.array?view=net-10.0)
(consulted 2026-09-13): runtime interface support and a separate library iterator.
The interpreter validates the exact Iterable/Iterator contract and dispatches array
acquisition through the internal `ArrayEnumerable.GetIterator<T>` library factory.
Existing array instructions and ArrayIterator implement traversal; no new opcode is
needed. The call graph includes the factory as a possible interface target. This is
more general than the initially considered query overloads and avoids their sequence
wrapper allocation, at the cost of runtime dispatch and compiler-symbol support.
No overall performance advantage over .NET is claimed.

Raven enables this target capability with
`<RavenIterationArraysImplementIterable>true</RavenIterationArraysImplementIterable>`.
The compiler then exposes the configured Iterable interface on vector symbols for
conversion, generic inference and completion. Ordinary .NET targeting stays unchanged.
Updated bundle/source project templates set this property. Both the compiler and
runtime/library must be refreshed; published Preview 5 and installed `.8` tools are
unchanged. This is a next-release capability requiring Raven `ec88c4474` on
`codex/neoclr-target-resolution` and the corresponding neoCLR development build.

The implemented contract is invariant and limited to vectors. Mutable-array covariance
is intentionally excluded by the [array variance contract](array-variance.md); no
rectangular-array interface projection or general IList/ICollection contract is added.
The sequence is not itself an Iterator; independent iterator objects own position.
Binding the runtime-provided acquisition member directly as a delegate is not part
of this slice; call it normally or through a lambda. Existing terminal-fault and
early-exit cleanup limitations remain.

## Execution and ownership

Constructing a query retains its source and delegate without enumerating or invoking
callbacks. Each `GetIterator()` creates fresh state and acquires an upstream iterator.
`MoveNext()` processes elements in source order. `Current` reads a cached result;
reading it twice does not repeat a selector or predicate. Reading before a successful
advance, after exhaustion, or after disposal faults. Exhaustion is sticky.

Re-enumerating a query repeats its callbacks and sees the source through a fresh
iterator, including updated captured state. It does not memoize results. `ToList()`
copies the sequence into independent list storage: values are copied and object
references retain their identity. It does not deep-clone objects.

Queries, source collections and captured callback state are GC-managed and can outlive
the method that builds the pipeline. Each iterator has a one-element checked array
for its current value, avoiding a fabricated default for arbitrary T. This adds an
allocation per iterator stage. Disposal drops the cached element, but references to
the source and callback remain until the iterator is collected. No performance or
allocation advantage over .NET is claimed; fusion and storage optimization can follow
behavioral validation.

Normal exhaustion disposes upstream. Explicit disposal is idempotent and propagates
through the pipeline once. `ToList()` also explicitly disposes its iterator after
normal completion. This does not add fault unwinding or fix Raven's existing
[automatic foreach cleanup gap](raven-target-contracts.md). Consumers that stop early
should explicitly dispose their iterator. A callback fault terminates execution;
it does not become a Result, resume iteration, or guarantee disposal.

## .NET comparison and deliberate boundaries

Primary sources consulted 2026-09-13:
[Where](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.where?view=net-10.0),
[Select](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.select?view=net-10.0),
and [ToList](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.tolist?view=net-10.0).
The familiar deferred filter/projection and eager list materialization contracts are
retained. neoCLR substitutes Iterable/Iterator and ArrayList for the corresponding
.NET names. Compiler extension lookup and ordinary generic calls are sufficient;
there is no query opcode or new runtime primitive. An eager Where/Select alternative
would simplify iterator state but change callback timing and require intermediate
storage, so it was not selected.

This is a bounded API, not complete LINQ compatibility. There are no indexed overloads,
query-expression syntax guarantees, ordering/grouping, SelectMany, providers,
expression trees or async queries. Source
mutation during an active enumeration follows the existing source iterator contract;
ArrayList currently retains a buffer and extent, without .NET List's mutation-version
exception. Underlying iterator acquisition happens in `GetIterator()`. Null arguments
do not have .NET's eager ArgumentNullException contract: invalid receivers/callbacks
are subject to runtime validation when used. No exception classes or nullability
model were added by this slice.

`Select` can produce `System.Void` when its callback has an explicit
`Func<T, System.Void>` target. This exercises Void as a generic argument and one unit
result per element; it is not a recommended replacement for ordinary iteration.
An unannotated no-result lambda may infer Raven's `System.Unit`, which is not admitted
as the target Void type. The adapter now projects Void in generic *method* arguments
as well as generic type arguments, while preserving no-result CLI returns.

Custom Raven [Iterable/Iterator implementations](experiments/raven-target/samples/application-iterable.rvn)
now pass end to end. The emission failure found during the initial slice was fixed
in Raven's target-metadata MethodImpl normalization (`000ed511e` on the experiment
branch). The bridge closes generic
interface declaration signatures before comparing them with application methods.
The sample counts acquisition and disposal through an implicitly converted receiver;
the low-level lifetime fixture independently verifies the runtime contract.

## Run and verify from source

A validated [local .8 installation](raven-query-local-build.md) is also available on
the author’s machine, including the custom iterator compiler fix.

Use a disposable project and fresh declaration metadata; archived .7 tools do not
include these APIs. Follow the [source setup](experiments/raven-target/README.md),
build the bridge and generate metadata with `--interfaces OUTPUT`, then use
`samples/library-queries.rvn` as the project's `Main.rvn`.

```sh
python3 docs/experiments/raven-target/run_project.py /absolute/path/to/Demo.rvnproj \
  --raven /absolute/path/to/Raven --runtime /absolute/path/to/neoclr
python3 docs/experiments/raven-target/verify_queries.py /absolute/path/to/Demo.rvnproj \
  --raven /absolute/path/to/Raven --runtime /absolute/path/to/neoclr
```

The runner includes `runtime/raven/Linq.neoil` in its generated collection profile.
This modern class-based library file is separate from the legacy Neo library profile.
For a published bridge, supply its matching `--system` library explicitly.
`verify_editor.py --queries` adds completion checks; also supply the other flags
matching the project's declaration profile.

Validation covers deferred/cached callbacks, empty and fully filtered input, repeated
enumeration, mutable captures, independent materialization, reference/Date/Boolean/Void
payloads, a returned query across GC, invalid Current access, terminal callback faults,
and rejection of unsupported overloads. The
[NeoIL fixture](experiments/raven-target/samples/query-lifetime.neoil) counts underlying
acquisition/disposal, including repeated Dispose. A corresponding C# pipeline on
.NET 11 preview (SDK 11.0.100-rc.1.26425.128) produced the same output as the main Raven
sample for timing, caching and repeated enumeration; this does not establish full
framework equivalence.


The array contract slice passed the full runtime test suite, Clippy, 35 focused Raven
regressions, 51 saved-project checks, 17 query checks and target editor completion.
A small .NET 11 SDK comparison of array-to-IEnumerable conversion, independent
iterators and mutation visibility produced the same `7, 7, 42, 7, 99, 99` sequence
as the neoCLR interface test. These are development results, not updates to the
published Preview 5 validation record.


## Terminal outcomes (2026-09-13)

The Raven profile adds source-only overloads of First, Last and Single. First/Last
return None for no element and Some for a present value, including a present zero.
Single returns Ok for exactly one element, Error(SingleError.Empty) for none, and
Error(SingleError.Multiple) for more than one. SingleError is a value union with
IsEmpty/IsMultiple, checked GetEmpty/GetMultiple and ToString; it is not an exception
class. There is no requirement to default-initialize T to express absence.

Use `values.Where(predicate).First()` (or Last/Single) for filtered selection in
this slice. Predicate overloads, OrDefault aliases, count/aggregation operators,
ordering and specialized collection paths remain outside this implementation.
An empty filtered sequence is handled identically to an empty source. The
[terminal sample](experiments/raven-target/samples/library-query-terminals.rvn)
shows arrays, lists, reference and Result payloads, case destructuring, Option
propagation and Single's Result propagation. It uses the currently supported
explicit Option carrier constructor when returning a Some value.

The terminal acquires an iterator once and disposes it on each normal outcome,
including an early First result or Single cardinality error. First reads Current
once after one successful MoveNext; Single reads the first Current and calls
MoveNext again without reading a second Current. Last reads Current on each success
until exhaustion. It does not terminate on an infinite sequence. Null receivers,
invalid iterator state, callback faults and Dispose faults remain runtime faults.
There is still no promise to run cleanup during fault unwinding. A caller owns
normal iteration cleanup only when directly obtaining an iterator itself.

Implementation uses ordinary interface calls, generics and existing union
constructors in [Linq.neoil](../runtime/raven/Linq.neoil), plus
[SingleError.neoil](../runtime/raven/SingleError.neoil). The importer validates
closed generic outcome signatures. No Raven compiler, opcode or metadata-format
change is needed. Regenerate the metadata core and System profile together;
the installed .11 build is unchanged.

### Comparison and provisional choices

Sources reviewed 2026-09-13, with .NET 10 as the API baseline:

- [.NET First](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.first?view=net-10.0)
  and [Last](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.last?view=net-10.0)
  throw on empty input. Their OrDefault alternatives can conflate absence with a
  legitimate default value. neoCLR keeps familiar method names but returns Option.
  This changes source contracts and adds case handling; it does not imply binary
  compatibility or a performance improvement.
- [.NET Single](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.single?view=net-10.0)
  requires exactly one item. [SingleOrDefault](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable.singleordefault?view=net-10.0)
  still rejects multiple items. We retain the exactly-one requirement but represent
  the two cardinality failures explicitly. Result<Option<T>, MultipleError> would
  fit an at-most-one operation but weaken what the name Single asks for. Returning
  only Option would erase the difference between no match and ambiguous matches.
- [Rust Iterator](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.next)
  uses Option for end-of-iteration. The transferable lesson is explicit absence,
  not Rust's ownership model or its iterator API. neoCLR retains MoveNext/Current
  for familiar interface-based iteration and puts Option on these public terminals.
- [CSharpFunctionalExtensions](https://github.com/vkhorikov/CSharpFunctionalExtensions#tryfirst-and-trylast)
  describes the ambiguity of FirstOrDefault/LastOrDefault and supplies Maybe-based
  TryFirst/TryLast. This is .NET ecosystem experience supporting an absence carrier,
  not a claim that .NET's existing methods should have been changed incompatibly.

All outcomes remain ordinary library contracts. Future aggregation should be
considered individually: a well-defined identity, empty-input absence, arithmetic
failure and a runtime fault are different cases. This slice does not wrap every
terminal in Result or establish a general aggregation policy.

The [comparison program](experiments/query-terminals/dotnet/Program.cs) targets
net10.0 and illustrates both default-value ambiguity and .NET cardinality exceptions:

```sh
dotnet run --project docs/experiments/query-terminals/dotnet/Comparison.csproj
cargo test --test query_terminals -- --nocapture
```

`verify_project.py --collections` and `verify_queries.py` run the Raven sample;
`verify_editor.py --queries` checks discovery on concrete lists, filtered sequences,
arrays and reflection arrays. The signature probe rejects payload-only return
metadata and virtual invocation of these static extensions. Direct IL tests check
all cardinalities, short-circuiting, disposal, fault boundaries and allocations.

## Concrete collection operations versus LINQ

The author's guideline is to prefer an appropriate built-in operation when the
concrete collection is available. Use LINQ for interface-based querying and useful
pipeline composition. The reason is avoiding unnecessary query objects and
allocations by working directly with the collection's storage, not an assertion
that the concrete API is always faster. LINQ specialization remains a valid
optimization; this guideline does not forbid it.

For the current prototype, an equivalent one-element lookup in
`tests/query_terminals.rs` records five managed allocations for ArrayList.Find and
nine for Where(...).First(), including identical list setup. These are interpreter
heap-object counts, not host allocations, allocated bytes or elapsed-time results.
The existing Find still creates an ArrayIterator: it avoids query wrappers but
is not yet a direct-storage scan. The upcoming ArrayList filtering slice should
review that implementation and preserve these semantics while reducing overhead.
Re-measure when implementations change; these counts are not a permanent API promise.

[.NET CA1826](https://learn.microsoft.com/en-us/dotnet/fundamentals/code-analysis/quality-rules/ca1826)
prefers direct properties/indexers in applicable cases; it is not a rule covering
all filters. [.NET 10 First source](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Linq/src/System/Linq/First.cs)
already specializes for known iterator/list forms. Accordingly, avoid treating
interface syntax as proof of allocation or treating every LINQ chain as equally
expensive. Any neoCLR specialization must preserve the selected outcome, callback
and cleanup contracts, or explicitly document a contract change.

Terminal-slice source validation on 2026-09-13 passed 62 saved-project cases,
29 query checks, 90 signature checks, 59 editor checks and 18 focused runtime tests
(terminal, collection and reflection suites), plus Clippy, formatting and the API
audit. The .NET comparison ran with SDK 11.0.100-rc.1.26425.128 targeting net10.0.
These are source-build results; no new SDK/VSIX package was installed or released.
