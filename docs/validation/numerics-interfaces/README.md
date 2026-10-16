# Numerics qualification

The development System.Runtime library provides AdditionOperators,
SubtractionOperators, MultiplyOperators and DivisionOperators in System.Numerics.
Number inherits them; the Complex demo implements arithmetic without ordering.
See the [design](../../design/numeric-contracts.md) for current contracts and limits.

## Current evidence

| Consumer/check | Coverage | Evidence |
| --- | --- | --- |
| LibraryConsumer.rvn | Four operations across ten primitive types, Number-bound forwarding, custom Complex | [library-results.json](library-results.json) |
| FunctionProvider.rvn + FunctionConsumer.rvn | Imported top-level operations on int/double and Number-bound forwarding | [function-results.json](function-results.json) |
| Invalid imported function argument | Add<string> rejects with RAV0320 | [function-negative-results.json](function-negative-results.json) |
| Provider.rvn + Consumer.rvn | Imported static class method with custom nominal interface bounds | [import-results.json](import-results.json) |
| Invalid imported class-method argument | Rejects int against the provider's source-local interface | [import-negative-results.json](import-negative-results.json) |
| ComplexGeneric.rvn / ComplexGenericMethod.rvn | Source-local top-level and class-contained generic calls | [generic-fixed-results.json](generic-fixed-results.json), [method-results.json](method-results.json) |
| Reference signatures | 218 checks, including exact static Self contracts | [number-signatures.json](number-signatures.json) |

Positive consumers return 42 on interpreter, managed JIT and AOT on macOS ARM64.
AOT hosts use undefined-behavior and bounds sanitizers. Manifests retain commands,
source/tool/bundle hashes and outcomes; they are not cross-platform qualification.

LibraryConsumer uses the five-library bundle
`target/numerics-public-libraries-20261016`, built with Raven `2e4776f7d` and neoCLR
`57c31e9bc` plus the library declarations committed in `c8bebcbfd`. Imported function
qualification uses Raven `ab010c687` and SDK `0cdfb228b` against that same bundle.
The complete public API inventory and native RavenDoc snapshot include all four
types and their operators; the rendering audit and four snapshot checks pass.

## Reproduce

```sh
python3 docs/validation/compound-operators/run.py \
  --compiler ../Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/numerics-public-libraries-20261016 \
  --source docs/validation/numerics-interfaces/LibraryConsumer.rvn \
  --output target/numerics-library-check
```

For imported functions, compile FunctionProvider.rvn as a separate Library project
with AssemblyName NumericFunctions and the same bundle's NeoCLR.ClassLibrary.props.
Run the shared runner with FunctionConsumer.rvn and `--reference <NumericFunctions.dll>`.
Run check-conformance.py with `--function-provider <NumericFunctions.dll>` to check
invalid arguments. For the source-local interface/class-method fixture, use
Provider.rvn, AssemblyName NumericProvider, Consumer.rvn and `--provider` instead.
That fixture uses the bundle recorded in import-results.json; its source-local
interfaces must not be substituted for the installed library's interfaces.

Focused SDK checks:

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --authored-method-bounds
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --authored-references
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --structural-method-constraints
```

The bound checks cover both callable forms, copied input, interning conflicts,
invalid ordinals, duplicates, noninterfaces and foreign owners. Scalar arguments
are rejected before owner selection and before nominal membership is declared,
then accepted with explicit conformance. Existing method-constraint checks guard
the shared validation code; they do not extend the parked structural feature scope.

StaticSelf.neoil and 18 native specialization tests cover exact conformance,
selected implementations, unchanged original metadata, and rejection of undeclared
membership, private or wrong-return implementations and missing bounds.

## Limits

Complex arithmetic is qualified for finite, moderate magnitudes. Scaled division,
zero divisors, NaN/infinities, extreme magnitudes, equality/hash, formatting and
performance are unqualified. Exact comparisons use exactly representable results.
This demo is not a production System.Numerics.Complex implementation.

Constructed interface bounds, instance/borrowed Self specialization, mixed operand/
result contracts and separate identity capabilities remain outside this slice.
Recoverable Result-returning methods are distinct from faulting operators.

Raw failure manifests and source-local fixtures remain regression evidence only.
`generic-results.json` and `jit-rejection.json` record resolved failures, not current
limitations. The older MultiplicationOperators fixture name is not a public API.
