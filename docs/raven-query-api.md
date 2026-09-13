# Prototype query API for Raven

The Raven-target runtime library now exposes `System.Linq.Enumerable` extension
methods over `System.Collections.Iterable<T>`:

| Method | Result | Evaluation |
| --- | --- | --- |
| `Where<T>(Iterable<T>, Func<T, bool>)` | `Iterable<T>` | Predicate runs while advancing an iterator |
| `Select<T, U>(Iterable<T>, Func<T, U>)` | `Iterable<U>` | Selector runs once per produced element |
| `ToList<T>(Iterable<T>)` | `ArrayList<T>` | Consumes the sequence immediately into a new list |

Import `System.Linq.*` to use member syntax. See the readable
[query sample](experiments/raven-target/samples/library-queries.rvn). Raven infers
type arguments from the receiver and callback; no explicit reference operators
are needed. These are generic library methods with validated bridge bindings,
not support for arbitrary generic application method bodies.

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
expression trees, async queries or implicit array-to-Iterable projection. Source
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
