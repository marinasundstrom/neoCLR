# Collections and query APIs

Sequence provides count and indexed read access. MutableSequence adds replacement; List adds growth. Arrays and lists can be consumed through these capabilities.

**Preview 9 implementation.** Collection capabilities are included in Preview 8. Preview 9 adds a basic set of iterable operators and uses Filter and Map; Preview 8 packages still use Where and Select.

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

When moving from Preview 8, replace `Where` with `Filter` and `Select` with `Map`, then rebuild against the matching Preview 9 references and runtime library. The old names are not aliases. `First`, `Last`, `Single` and `ToList` retain their names.

<a id="basic-operators"></a>

## A basic operator set
**Preview 9 API.** Any and All test elements; Count counts them. Take and Skip select a page. Concat joins two sequences in order, and FlatMap turns each element into a sequence and flattens the results. Fold accumulates from an explicit seed, returning that seed for empty input.

```raven
{{QUERY_BASICS_SAMPLE}}
```

The page contains 2 and 3. Both tests are true, the matching count is 2, and the total is 10. The final pipeline prints 2, 20, 3, 30, 5 and 50.

[Complete executable sample →](../../samples/library-query-basics.rvn) · [Expected output →](../../samples/library-query-basics.expected.txt)

Any and All stop as soon as the answer is known. Empty Any is false and empty All is true. Take, Skip, Concat and FlatMap are lazy. Non-positive Take yields nothing; non-positive Skip skips nothing. Fold consumes its input from left to right. These methods require the matching Preview 9 library.

<a id="dotnet-mapping"></a>

## From .NET LINQ to neoCLR
This maps the Preview 9 API. It is not a promise of full LINQ compatibility; Preview 8 retains Where/Select and has fewer operators.

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

Development generic API signatures now admit Object map keys and values. `HashMap<Object, Object>` uses explicit equality and hash callbacks: Path and type descriptors use their own contracts, supported boxed integers and Booleans compare by value, and ordinary classes retain allocation identity. A tested sample covers mixed keys, collisions, replacement, table growth and reference-preserving values through GC. This does not add a default comparer or string-to-Object conversion.

## API reference

Browse [ArrayList](xref:System.Collections.ArrayList`1),
[HashMap](xref:System.Collections.HashMap`2),
[collection interfaces](xref:System.Collections) and
[query operators](xref:System.Linq.Operators) for signatures,
member descriptions and the current development contract.
