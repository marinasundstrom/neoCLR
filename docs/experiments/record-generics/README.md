# Generic native record equality and hashing — 2026-10-20

This extends [automatic nongeneric interfaces](../record-auto-interfaces/README.md)
to generic positional record structs and classes. Generic records remain ordinary
classes/structs with generated methods and self-equality interfaces. No record
representation, instruction or ABI is introduced.

The compiler emits typed/Object Equals, GetHashCode, == and != together with
EquatableTo<Self>. Constructed source and imported component contracts use substituted
methods. KeyValuePair<K,V> consequently exposes these generated members. Index/Range
now override Object.Equals and GetHashCode, so their typed value contracts also work
through Object and generic parameters. Ordinary .NET compiler behavior is unchanged.
The previous generic-interface capability exclusion has been removed.

## Equality policy and cost

The [C# record struct specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/structs)
uses default comparers selected from the declared field types for memberwise equality
and hashing. The native implementation preserves that selection principle using its
existing EquatableTo contract and native typeof/AssignableTo APIs. A type parameter
selects its own equality interface when assignable to that contract; otherwise the
body calls virtual Object.Equals. Nulls compare equal and contribute zero hashes.
Text keeps canonical equality and its original UTF-8 storage. An Object-typed field
does not adopt an unrelated typed interface just because its runtime instance has one.

This implementation uses ordinary type relationships and boxes, not optimized .NET
comparer specializations. Each generic equality component boxes its two operands once;
each generic hash component boxes its operand once. Type descriptor resolution adds
allocations too. Nongeneric direct component operations are unchanged. This is a
bounded correctness foundation, and replacing this overhead with specialized default
comparer dispatch is the next performance task.

[Allocation results](allocation-results.json) compare 128 iterations of one typed
integer equality and two hashes. Baseline and nongeneric records each allocate 259
managed objects; generic records allocate 1,155: **896 extra, or 7 per iteration**.
All tracked objects were reclaimed. Collection counts are diagnostic eager-collector
activity, not a throughput measure. This does not measure host allocator traffic or
claim parity with .NET/Go. The compiled host command is retained in the result.

## Validation and remaining limits

[Program.rvn](Program.rvn) tests source generic structs/classes, interface conversion,
boxed records, nested/constructed source records, imported KeyValuePair both directly
and as a wrapper component, canonical/nullable text, Int32/Int64/Boolean, Index/Range,
custom typed equality and declared Object policy. [validation.json](validation.json)
records passing interpreter/JIT/AOT executions and exact artifact hashes.

The native Object regression separately compares primitive equality/hash results
against the interpreter, covering unequal values, mismatched kinds, null and full
Int64 payloads. Native dispatch never treats these boxes as reference identities.
The focused compiler probe checks generic owner metadata/self-interface synthesis
and targeted rejection when generic component operations lack the native typeof
policy. Ordinary .NET generic class/struct interface identity remains tested.

Unsupported ordinary value structs still lack default structural Object methods;
the [negative consumer](Unsupported.rvn) faults rather than silently comparing fresh boxes by
identity, with [three-engine results](unsupported-results.json). Generic enum/floating-point Object adapters, broader component support,
generic formatting, record inheritance and authored record members remain separate
gaps. These are incomplete capabilities, not new language rules. Public API details
are in [the pair reference](../../../api-docs/key-value-pair.md). The native snapshot
and legacy reference are refreshed; publication and replacement-release qualification
remain separate.

Compiler integration is on Raven codex/neoclr. The tested precommit compiler label is
`6f7f32901-dirty-generic-records-final`; retained hashes identify the actual binaries.
The matching five-library rebuild is `target/generic-record-libraries-final`, bundle
`target/generic-record-bundle/bundle`, application `target/generic-record-app` and
allocation application `target/generic-record-allocation-app`. Historical baseline
captures the original generic-value-method rejection. Failed intermediate attempts
remain under ignored target paths, not qualified product evidence.

```sh
python3 docs/experiments/record-generics/check.py target/generic-record-bundle/bundle target/generic-record-app --output docs/experiments/record-generics/validation.json
python3 docs/experiments/record-generics/check.py target/generic-record-bundle/bundle target/generic-record-unsupported-app --expect-fault --output docs/experiments/record-generics/unsupported-results.json
python3 docs/experiments/record-generics/measure.py target/generic-record-allocation-app --output docs/experiments/record-generics/allocation-results.json
```

Compiler source is committed as Raven `4c051e332`. Tested artifacts retain their
precommit labels and exact hashes. Both .NET generic identity tests, the source-core
probe and native Object regression pass. Four API snapshot checks pass; the website
build verifies 4,905 pages. Neither website publication nor release creation was performed.
