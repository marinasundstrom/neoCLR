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
existing EquatableTo contract and declared-type assignment rules. The latest compiler
uses the selected core's RuntimeHelpers.IsAssignable<TSource,TTarget> service;
older cores retain the native typeof/AssignableTo fallback. A type parameter
selects its own equality interface when assignable to that contract; otherwise the
body calls virtual Object.Equals. Nulls compare equal and contribute zero hashes.
Text keeps canonical equality and its original UTF-8 storage. An Object-typed field
does not adopt an unrelated typed interface just because its runtime instance has one.

This implementation uses ordinary type relationships and boxes, not optimized .NET
comparer specializations. Each generic equality component boxes its two operands once;
each generic hash component boxes its operand once. The latest policy selection
uses existing type tokens and ReflectionAssignable without descriptor/context
allocations. Nongeneric direct component operations are unchanged. Eliminating
the remaining boxes through specialized default-comparer dispatch is follow-up work.

The original [allocation results](allocation-results.json) compare 128 iterations of one typed
integer equality and two hashes. Baseline and nongeneric records each allocate 259
managed objects; generic records allocate 1,155: **896 extra, or 7 per iteration**.
All tracked objects were reclaimed. Collection counts are diagnostic eager-collector
activity, not a throughput measure. This does not measure host allocator traffic or
claim parity with .NET/Go. The compiled host command is retained in the result. This historical harness also
materialized argument strings in the loop; the corrected matched experiment below
supersedes its per-operation attribution.

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


## Descriptor-free policy and corrected allocation control

The source core now supplies an internal `TypeHandle<T>()` managed token helper and
public compiler service `RuntimeHelpers.IsAssignable<TSource,TTarget>()`. The native
compiler validates the service's core ownership and exact signature before selecting
it for generic records. It preserves the prior fallback for older cores. The internal
token declaration requires selected RuntimeTypeHandle ownership; malformed signatures
reject. This uses existing instructions and type relationship semantics, with no host
callback, special record ABI or friend grant. See [bridge design and tradeoffs](../../raven-cli-bridge.md#descriptor-free-generic-record-policy--2026-10-20).

The allocation harness now reads the argument and evaluates both mode strings once
before the loop. This removes repeated argument materialization and equalizes setup
across modes. The same source was rebuilt with both the prior compiler/library bundle
and the new bundle; the native backend and host remain the same.

| 128 iterations | Previous implementation | Handle-based policy |
| --- | ---: | ---: |
| Baseline | 5 | 5 |
| Nongeneric record | 5 | 5 |
| Generic record | 1,029 | 517 |
| Additional allocations per equality + two hashes | 8 | 4 |

[Before](policy-allocation-before.json) and [after](policy-allocation-after.json) retain
commands and compiler, core, application and harness hashes. All objects are reclaimed.
The reduction is **50% of incremental allocations**, removing four descriptor/context
objects per iteration. Four value boxes remain. The earlier reported seven included
unequal argument materialization between branches; it was not a precise count of record
allocations. No HTTP throughput or collector-policy improvement is established here.
The harness asserts the expected eight/four costs rather than merely checking a positive
allocation difference.

[Updated execution evidence](policy-validation.json) covers the full existing consumer
and direct positive/negative assignability checks for primitives, interfaces and reference
types. [Negative execution](policy-unsupported-results.json) still faults across all three
engines. [Compiler probe](policy-compiler-probe.txt) includes invalid token-helper signature
rejections. Ordinary .NET code is untouched; the prior focused .NET record evidence applies.

Tested compiler label: `4c051e332-dirty-record-policy`; source changes remain on Raven
codex/neoclr. Libraries: `target/record-policy-libraries`; bundle:
`target/record-policy-bundle/bundle`. Failed/intermediate experiments stay under ignored
target paths and are not qualified artifacts. API snapshots use the matching libraries
and pinned renderer `03bcc23d6`. Website publication and release qualification are separate.

```sh
python3 docs/experiments/record-generics/measure.py target/record-policy-allocation-before-final --expected-extra-per-iteration 8 --output docs/experiments/record-generics/policy-allocation-before.json
python3 docs/experiments/record-generics/measure.py target/record-policy-allocation-after-final --expected-extra-per-iteration 4 --output docs/experiments/record-generics/policy-allocation-after.json
```

Compiler source for this follow-up is committed as Raven `8da3a45f8`; artifact labels
remain the actual precommit build labels above. Four native snapshot checks pass,
and the site build checks 4,907 pages with 124 legacy pages retained. Existing unrelated
missing-description warnings remain. Neither website publication nor release creation
was performed.
