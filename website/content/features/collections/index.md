# Collections and query APIs

Sequence provides count and indexed read access. MutableSequence adds replacement; List adds growth. Arrays and lists can be consumed through these capabilities.

Struct elements are copied into collections; class elements retain shared object identity.

<a id="example"></a>

[Arrays: shared storage, generic shape and the .NET comparison →](../arrays/)

## Collections and queries in Raven
```raven
{{COLLECTION_SAMPLE}}
```

The full sample reads both arrays and lists, replaces elements through MutableSequence, and grows a List. A Sequence view sees changes made through another alias: read-only access does not mean immutable storage.

[Complete executable sample →](../../samples/library-collection-capabilities.rvn) · [VS Code setup →](../../try/#development)

<a id="query-names"></a>

## Filtering and mapping

Import `System.Linq.*` for the query operators. Filter keeps matching elements; Map transforms them. This sample prints `10`, then `30`. Neither operator invokes its callback until iteration begins.

```raven
{{QUERY_NAMES_SAMPLE}}
```

[Complete executable sample →](../../samples/library-query-names.rvn) · [Expected output →](../../samples/library-query-names.expected.txt)

<a id="basic-operators"></a>

## A basic operator set
Any and All test elements; Count counts them. Take and Skip select a page. Concat joins two sequences in order, and FlatMap turns each element into a sequence and flattens the results. Fold accumulates from an explicit seed, returning that seed for empty input.

```raven
{{QUERY_BASICS_SAMPLE}}
```

The page contains 2 and 3. Both tests are true, the matching count is 2, and the total is 10. The final pipeline prints 2, 20, 3, 30, 5 and 50.

[Complete executable sample →](../../samples/library-query-basics.rvn) · [Expected output →](../../samples/library-query-basics.expected.txt)

Any and All stop as soon as the answer is known. Empty Any is false and empty All is true. Take, Skip, Concat and FlatMap are lazy. Non-positive Take yields nothing; non-positive Skip skips nothing. Fold consumes its input from left to right.

<a id="dotnet-mapping"></a>

## From .NET LINQ to neoCLR
The supported operators follow these contracts; the library does not provide full LINQ compatibility.

| .NET Enumerable | neoCLR | Behavior |
| --- | --- | --- |
| `Where(predicate)` | `Filter(predicate)` | Lazy filtering. |
| `Select(selector)` | `Map(selector)` | Lazy projection. |
| `SelectMany(selector)` | `FlatMap(selector)` | Lazy flattening; no indexed or result-selector overloads. |
| `Any(), Any(predicate)` | `Any(), Any(predicate)` | False for empty input; short-circuits. |
| `All(predicate)` | `All(predicate)` | True for empty input; short-circuits. |
| `Count(), Count(predicate)` | `Count(), Count(predicate)` | Int32 result; overflow is a runtime fault, not OverflowException. |
| `Aggregate(seed, accumulator)` | `Fold(seed, accumulator)` | Left-to-right, seeded accumulation; empty input returns the seed. |
| `Take(count), Skip(count)` | `Take(count), Skip(count)` | Lazy prefix/suffix; non-positive bounds follow .NET behavior. |
| `Concat(second)` | `Concat(second)` | Lazy concatenation in source order. |
| `ToList()` | `ToList()` | Materializes an ArrayList rather than a .NET List. |
| `First(), First(predicate)` | `First(), First(predicate)` | Returns Option; empty/no match is None rather than an exception. |
| `Last(), Last(predicate)` | `Last(), Last(predicate)` | Returns Option; empty/no match is None rather than an exception. |
| `Single(), Single(predicate)` | `Single(), Single(predicate)` | Returns Result with distinct Empty/Multiple errors. |
| `FirstOrDefault / LastOrDefault / SingleOrDefault` | `No direct equivalent` | Use the Option/Result outcome and an explicit fallback. |
| `Aggregate without a seed, Sum, Average, Min, Max` | `Not yet implemented` | Seeded Fold can express basic accumulation. |
| `OrderBy, ThenBy, GroupBy, Join, Distinct, Union, Intersect, Except, Zip` | `Not yet implemented` | Ordering, equality and pairing contracts remain future work. |

<a id="limits"></a>

## Behavior and limits
The roles are comparable to .NET collection interfaces, with replacement separated from growth. More precise contracts help describe what an algorithm requires, at the cost of more interface distinctions. Filter and Map are lazy; ToList materializes, First/Last return Option and Single returns Result.

[Detailed contract and comparisons →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/collection-contracts.md)

<a id="direction"></a>

## Planned work and open questions

Variance, immutable or frozen providers, and broader query coverage need separate decisions. The aim is useful collection contracts, not reproducing every .NET collection type. Iterator and mutation behavior should stay explicit.

[Related proposals and open questions →](../../proposals/#collections)

<a id="feedback"></a>

## Questions and contributions

Questions, examples and documentation corrections are welcome. See [how to contribute](../../#feedback).

Report issues with a small program, the toolchain version, expected behavior and observed output. API proposals should identify the missing operation or contract.

[Discuss on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)


## Mixed Object keys

Generic API signatures admit Object map keys and values. `HashMap<Object, Object>` uses explicit equality and hash callbacks: Path and type descriptors use their own contracts, supported boxed integers and Booleans compare by value, and ordinary classes retain allocation identity. A tested sample covers mixed keys, collisions, replacement, table growth and reference-preserving values through GC. A default comparer is not supplied. Strings use content equality and hashes through Object; callers still select the callbacks.

<a id="comparer-policies-development"></a>

## Comparer policies

`EqualityComparer<T>` pairs equality with hashing and `Comparer<T>`
supplies ordering. `StringComparer.Ordinal` implements both for exact string content.
HashMap accepts a reusable policy; the callback constructor remains supported.

```raven
{{COMPARER_MAP_SAMPLE}}
```

This excerpt comes from the [executable comparer checks](../../samples/library-comparers.rvn).
The `Check` helper faults if a condition fails. Equal text has equal hashes, while
case and normalization differences remain distinct. Ordinal ordering follows UTF-8
bytes/Unicode scalar values, which differs from .NET UTF-16 ordering for some characters.

Use `DelegateEqualityComparer<T>` or `DelegateComparer<T>` to adapt callbacks, or
implement the interfaces for a named policy. Policy behavior and keys must remain
stable while stored; callbacks must not reenter the same map. No universal default or
culture policy is supplied. `StringComparer.OrdinalIgnoreCase` additionally uses
Unicode simple folding consistently for equality/hash and ordering; see the
[string comparison contract](../strings/#explicit-comparison-policies-development). Use matching compiler and runtime library artifacts.

## API reference

Browse [ArrayList](xref:System.Collections.ArrayList`1),
[HashMap](xref:System.Collections.HashMap`2),
[EqualityComparer](xref:System.Collections.EqualityComparer`1),
[Comparer](xref:System.Collections.Comparer`1),
[StringComparer](xref:System.StringComparer),
[collection interfaces](xref:System.Collections) and
[query operators](xref:System.Linq.Operators) for signatures,
member descriptions and the current development contract.

<a id="value-tuples-development"></a>

## Value tuples

`System.Tuple<T1,...,TN>` provides heterogeneous value tuples with one
through seven components. Raven tuple syntax uses this family; copying a tuple
copies its fields while retaining the identity of referenced objects. See the
[value-tuple API guide](/docs/tuples.html) for construction, fields and limitations.


### Development: type filtering

Import `System.Linq.*` and use `source.OfType<ResultType>()` to lazily retain
non-null compatible values in source order. The source type is inferred; matching
values are narrowed to the requested type without numeric conversions. Each
iteration is independent, and completing or disposing the iterator releases its
source. See the [OfType member reference](xref:System.Linq.Operators.OfType).

## Queues, stacks and sets (development)

The development library adds `Queue<T>`/`ArrayQueue<T>`, `Stack<T>`/`ArrayStack<T>`
and `Set<T>`/`MutableSet<T>`/`HashSet<T>`. Queue and stack removal and peek return
`Option<T>`; set insertion and removal report whether membership changed. HashSet
uses an explicit equality comparer. These concrete types are unsynchronized and
iterate shallow snapshots; queue/stack snapshots follow removal order and set order
is unspecified. Count followed by another operation is not an atomic transaction.

Queue, Stack and Set pass native macOS, Windows x64 and interpreted checks.
Development `Map<K,V>` also implements `Iterable<KeyValuePair<K,V>>`. Each pair has
`Key` and `Value` properties with initialization-only setters and supports `let (key, value) = pair`.
HashMap iteration captures a shallow snapshot in unspecified order. Pair iteration
passes native macOS, Windows x64 and interpreted checks.
The native pair supports construction, copying and deconstruction; full record
equality, hashing and formatting remain unsupported. Development object initializers
can set pair components; ordinary later assignments are rejected. This is a compiler
restriction, not runtime object freezing. Init checks pass on macOS and Windows x64.
Future concurrent implementations remain separate work.
See the type and member contracts in the [API reference](/docs/).

## Development: map initialization

`HashMap<K,V>(items, comparer)` accepts an iterable of `KeyValuePair<K,V>`,
including sequences and existing maps. `pairs.ToMap(comparer)` provides the query
form; `items.ToMap(keySelector, valueSelector, comparer)` selects both components.
Import `System.Linq.*` for these operators. The callback constructor also accepts
`items, equal, hash`.

Each eagerly builds independent storage in one pass. Referenced objects stay shared.
The explicit comparer defines key uniqueness; duplicate keys cause a terminal fault.
Use an explicit `TryAdd` loop for recoverable duplicates. Current collections are
unsynchronized. These additions require rebuilt development libraries.

[Executable collection checks](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/native-collections/Main.rvn)

The map materialization checks pass on macOS ARM64, Windows x64 and their matching
interpreters.

## Development: collection initialization

`ArrayList<T>(items)`, `ArrayQueue<T>(items)`, `ArrayStack<T>(items)` and
`HashSet<T>(items, comparer)` accept iterables, including sequences and lazy queries.
Each consumes the input once and creates independent storage; referenced objects
remain shared. Lists and queues preserve input order. Stacks push in input order,
so the last input is popped first; constructing from another stack reverses its
pop order. Sets discard duplicates under the explicit comparer.

These overloads require rebuilt development libraries and retain the existing
unsynchronized collection contracts.

The development compiler also disposes reference iterators on structured exits
from `for`, including early return and break, in lifetime order with `use`. Terminal
Faults do not unwind these scopes. Prefer for where consuming elements expresses
the operation; use retains explicit control for advance-only or stateful iteration.

Development runtime-library tests now use ordinary Raven module functions and a
Raven runner, with native/interpreter checks for collection ordering, copying,
comparers and iterator ownership. The
[test contract and executable suites](https://github.com/marinasundstrom/neoCLR/blob/main/runtime/raven/tests/README.md)
separate discovery from execution. Host TestAttribute discovery generates typed
registration adapters; interpreted module metadata can also inspect descriptions.
Development runners accept `--filter <text>` for literal case-sensitive ID/name
substring selection and `--id <id>` for exact IDs, including manual registrations.
The framework now includes migrated map snapshot, copy and materialization checks,
plus queue/stack growth, set collisions, iterable construction, reference identity,
snapshots and query/loop iterator cleanup;
`--filter Map` runs that group of cases without requiring a grouping attribute.
No matches or invalid arguments are configuration errors. Grouping attributes and
.NET-style filter expressions remain future work.
