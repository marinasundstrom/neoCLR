# Imported value components in native records

Development qualification, 2026-10-20. The native positional-record generator now reuses
explicit imported value contracts, including the existing Index and Range APIs.
A record such as `record struct Position(Value: Index)` can compare and hash its
component directly, without boxing it or inspecting its private storage.

## Contract and comparison

[.NET EqualityComparer<T>.Default](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.equalitycomparer-1?view=net-6.0)
selects IEquatable<T> when available. The native compiler follows that explicit
contract approach through the selected core's EquatableTo<T> identity; it does not
infer an imported record marker from a collection of method names.

Admission is deliberately bounded to nongeneric imported native value types that:

- implement the exact selected `EquatableTo<Self>` interface;
- expose one public, concrete, nongeneric `Equals(Self) -> bool` by-value method;
- expose one declared public, concrete, nongeneric `GetHashCode() -> int` method;
- have no explicit interface mappings requiring separate implementation resolution;
- are not borrowed/ref-like or intrinsic primitive types with separate policies.

Generated methods call the selected concrete methods using borrowed receivers.
Hash selection intentionally honors the declared typed hash API even when it is
not an Object override, as in the existing Index/Range contracts. This is a bounded
native policy, not a claim that arbitrary non-overriding CLR methods participate in
EqualityComparer<T>.Default. The benefit is no comparer object or component box;
the cost is narrower admission than a complete runtime comparer and a requirement
that the type author keep equality and hashing consistent.

Types that lack these contracts remain rejected with NEOMETA001 for record value
operations. Storage/accessors/deconstruction remain available. In particular,
ConsoleKeyInfo can compare/hash itself but does not yet declare EquatableTo<Self>,
so using it as an imported component remains a gap. Duration lacks a declared hash
method. Neither is silently accepted using Object identity or a guessed contract.
Generic record owners, constructed generic components, imported record discovery,
explicit value-interface implementation resolution and generated formatting remain
separate work. Source-defined ordinary structs are not newly admitted by this slice.

## Reproduction and evidence

[Baseline](baseline.txt) records the earlier rejection for Position(Index).
The compiler source-core metadata probe now covers an imported positive contract,
missing hashing, a method-only lookalike and an interface with the wrong self type.
The executable consumer exercises Index direction/offset, both Range boundaries,
Option<Index>, nested source records, default storage and boxed wrapper operations.
The two negative consumers retain targeted diagnostics for missing hash and missing
interface contracts. Boxing a wrapper explicitly still allocates.

Build with `scripts/build-native-project.py --profile console` using:

- `RecordComponents.rvnproj` for the functional consumer;
- `Allocation.rvnproj` for 128 repeated typed Index/Range wrapper comparisons/hashes.

```sh
python3 docs/experiments/record-components/check.py \
  target/record-component-bundle-v1/bundle target/record-component-app-v1 \
  --output target/record-components-validation.json
python3 docs/experiments/record-components/measure.py \
  target/record-component-allocation-v1 --output target/record-components-allocations.json
```

The allocation probe reuses the existing native counter host. It measures managed
allocations only; collection counts, host allocation and throughput are distinct.
Native linking on this machine uses Xcode clang and the MacOSX26.2 SDK.

## Integration scope

This is a native Raven emission change. Ordinary .NET synthesis, Runtime Contract
configuration, native metadata layout and runtime instructions are unchanged. The
CLI reference bridge does not implement these bodies. Raven owns contract admission
and method selection; neoCLR owns runtime/backend qualification. Wider native
comparer and interface-resolution support should replace the conservative policy.

No public class-library signatures change. Existing API snapshots remain applicable;
the website's Raven integration page describes this development support. Publication
is a separate manual action, and this development bundle is not a release package.


## Validated results

- [Functional results](validation.json): interpreter, JIT and AOT pass; missing-hash
  and missing-interface consumers receive the expected rejection.
- [Union-record regression](union-regression.json): all three engines and the earlier
  unsupported-payload check pass against this bundle.
- [Allocation results](allocation-results.json): 128 iterations allocate two managed
  objects, identical to baseline, and reclaim both. Collection activity rises from
  8 to 8,200; no throughput or CPU improvement is claimed.
- [Compiler probe](compiler-probe.txt): supported imported contract and three negative
  contract shapes pass. Touched C# files were whitespace-formatted with dotnet format.

Artifacts are target/record-component-compiler-v1, target/record-component-libraries-v1,
target/record-component-bundle-v1/bundle, target/record-component-app-v1,
target/record-component-allocation-v1 and target/record-component-union-regression-v1.
Compiler provenance remains `7a411fbf6-dirty-record-components`: it was built before
commit and final whitespace formatting. The change is now committed as Raven
`cf66a235c` on codex/neoclr. The validation JSON records actual compiler,
runtime, backend, library and application hashes. These identities are not relabeled
as release assets. The backend includes the preceding String call-projection fix;
no runtime/backend source change was needed in this slice.
