# Minimal numerics contracts and Complex demo — 2026-10-16

This began as a source-local experiment; the public library adoption and its
qualification are recorded below. It responds
to the author's request for minimal System.Numerics interfaces and another useful
number type to test the model. See the [design comparison](../../design/numeric-contracts.md#minimal-systemnumerics-capabilities-and-complex-experiment--2026-10-16).

`Complex.rvn` defines four explicit nominal static operator interfaces in
System.Numerics, using native Self operands/results, and a double-component Complex
struct implementing them. No ordering is invented: Complex does not implement the
current System.Number contract. Candidate interface names and their same-type-only
scope are provisional. Number remains unchanged in the installed library.

## Initial experiment (historical)

The direct consumer returns 42 on interpreter, managed JIT and AOT on macOS ARM64.
It checks addition, subtraction, multiplication, finite nonzero division, i squared,
conjugate multiplication and a polynomial. The AOT host uses undefined-behavior and
bounds sanitizers. `direct-results.json` retains commands and source/tool/bundle hashes.

`check-conformance.py` verifies two compile-time failures: an addition operator
without explicit AdditionOperators conformance does not satisfy the generic bound;
and implementing the arithmetic capabilities does not satisfy Number.
`conformance-results.json` retains diagnostics and generated source hashes.

`ComplexGeneric.rvn` differs by routing addition through a top-level constrained
function. Compilation and interpreter execution succeed, but AOT and JIT reject
`unsupported generic method constraint`. `generic-results.json` intentionally has
passed=false: it records the blocker, not successful full qualification. The shared
specializer currently admits only function-kind method bounds in
runtime/codegen/src/specialization.rs::check_method_constraints. Admission must
validate nominal conformance and then resolve constrained Self calls; removing the
gate without those checks is not a fix.

A preliminary generic method inside a class also failed the Raven native emitter's
ConstraintClauses gate in Int32Emitter. Top-level syntax isolates the runtime gap;
it does not close class-contained generic method support. A local full library build
with candidate interfaces and Number inheritance succeeded, but those library edits
were withdrawn pending useful cross-backend constrained calls and API documentation.
The unqualified local bundle target/numerics-libraries-20261016 is not a release artifact.

## Reproduce

```sh
python3 docs/validation/compound-operators/run.py \
  --compiler ../Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/function-reflection-library-validated-20261016 \
  --source docs/validation/numerics-interfaces/Complex.rvn \
  --output target/numerics-complex-check
python3 docs/validation/numerics-interfaces/check-conformance.py \
  --compiler ../Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/function-reflection-library-validated-20261016 \
  --output target/numerics-conformance-check
```

Use ComplexGeneric.rvn with a different output directory to reproduce the AOT
failure after interpreter success. The recorded JIT rejection uses the direct
consumer's host with the generic image. Matching compiler: Raven codex/neoclr
371363a67; runtime/SDK baseline 1f1e99af3, neoCLR source 76e67df6c plus these samples.
No compiler/runtime changes, Runtime Contract switch, CLI encoding or native opcode
is introduced. Ordinary Raven/.NET and broader structural work remain unchanged.

## Initial limits and next work (historical)

The textbook complex division formula is for finite, moderate magnitudes; production
scaled division, zero divisors, NaN/infinities, extreme magnitudes, equality/hash,
formatting and performance are not qualified. Do not treat this as System.Numerics.Complex
parity or an allocation/performance claim. Exact component comparisons in the tests
use deliberately exactly representable results.

First close nominal interface bounds and constrained Self dispatch in JIT/AOT,
including invalid implementation rejection and imported contracts. Then add the
small public interfaces, compose Number with them, and refresh the complete API
reference. Independent operand/result types, identity capabilities and a possible
unordered NumberBase-like aggregate require separate decisions. Recoverable
Result-returning methods remain distinct from operator support.


## Nominal-bound specialization follow-up

The native specializer now admits closed nongeneric nominal interface method bounds
and resolves static Self calls through the runtime's exact nominal conformance
checks before lowering to ordinary concrete calls. It preserves original metadata
and reuses the interpreter's implementation resolution. Instance/borrowed Self,
constructed interface bounds and other method constraint categories remain gated.
The old generic-results.json and jit-rejection.json remain historical failure evidence;
follow-up results are stored separately. No new serialized opcode or metadata encoding.

StaticSelf.neoil exercises the same contract at the metadata layer. The specialization
regression checks executable selected code, unchanged original metadata, and rejection
of undeclared membership, private or wrong-return implementations, and missing bounds.
The existing function-kind and type-constraint regressions run alongside it.

Follow-up qualification: 18 focused specialization tests pass, including the linked
value-conformance projection regression. Both ComplexGeneric and
ComplexGenericMethod return 42 in interpreter/JIT/AOT (generic-fixed-results.json
and method-results.json). Raven `38fb410df` removes the class-method syntax gate;
its callable capability checks remain active and the .NET generic math test passes.
A separate-provider compile now identifies the next gap: imported constrained
methods require a complete symbol-authored SDK contract. This remains unqualified.


## Separate-provider follow-up

Raven `2e4776f7d` and the SDK's symbol-authored interfaceBounds contract close the
imported static class-method gap for nongeneric nominal interfaces. Provider.rvn
contains the public Complex/capabilities/Arithmetic declarations; Consumer.rvn
uses them through a separately built NumericProvider.dll. import-results.json
records interpreter/JIT/AOT exit 42 and fingerprints the provider assembly.
import-negative-results.json records RAV0320 when the imported Add is instantiated
with int, which does not implement the prototype AdditionOperators.

To reproduce, build Provider.rvn as a Library project importing the same bundle's
NeoCLR.ClassLibrary.props, with AssemblyName NumericProvider. Run the shared
operator runner with --source Consumer.rvn and --reference <NumericProvider.dll>.
Run check-conformance.py with --provider <NumericProvider.dll> for the invalid call.
The provider was compiled with Raven 38fb410df; the consumer uses 2e4776f7d.
The compiler/library bundle otherwise remains the same as the preceding checks.
The new runner option adds an exact project reference and runtime module; it does
not copy provider source into the consumer.

Focused SDK checks (--authored-method-bounds) cover reference interning, defensive
copying, wrong type arguments, missing/different bounds on the same callable,
invalid ordinals, duplicate bounds, noninterfaces and foreign output ownership.
No dependency reader fallback or structural interface conformance is introduced.


## Public library adoption

The development System.Runtime library now provides AdditionOperators,
SubtractionOperators, MultiplyOperators and DivisionOperators in System.Numerics.
Number inherits all four while retaining its declared members and identities.
The final multiplication capability follows the .NET MultiplyOperators spelling;
older source-local fixtures above preserve their experimental name.

LibraryConsumer.rvn imports the actual library interfaces. It checks all four
operations across ten primitive number types, forwarding a Number bound to an
AdditionOperators helper, plus the custom Complex implementation without ordering.
`library-results.json` records successful interpreter, managed JIT and sanitized
AOT execution (42) against the freshly built five-library bundle
`target/numerics-public-libraries-20261016`. Raven revision: `2e4776f7d`; runtime/SDK
revision: `57c31e9bc` plus this slice's library declarations. The manifest fingerprints
the compiler, runtime, source and bundle. Production Complex edge behavior remains
outside this demo's qualification.

`number-signatures.json` records 218 passing reference signature checks, including
the four inherited capabilities and their exact static Self signatures. The
reference assembly inventory and native RavenDoc snapshot include all four public
types and operators. The native rendering audit and four snapshot tests pass.
No new Runtime Contract switch or serialized opcode is required. The native
signature uses Self; the documentation CLI projection uses the existing Self
marker. Ordinary Raven/.NET behavior remains unchanged.


## Imported top-level function follow-up

FunctionProvider.rvn exports Add, Subtract, Multiply, Divide and a Number-constrained
Forward function. FunctionConsumer.rvn exercises the imported operations with int
and double and calls Forward<int>. The initial compiler rejected this path with
NEOMETA001 (unsupported symbol-only contract). Preserving the bounds exposed another
SDK gap: scalar signatures were not resolved to their selected nominal owner during
imported-bound validation. Both paths now use explicit nominal conformance; no
operator-based inference or dependency-reader fallback is added.

`function-results.json` records the separate-provider interpreter/JIT/AOT run against
`target/numerics-public-libraries-20261016`; `function-negative-results.json` records
rejection of Add<string>. The provider is compiled separately as a library with
AssemblyName NumericFunctions, using the existing bundle props. Reproduce with the
shared operator runner's --reference option and check-conformance.py's
--function-provider option. Compiler and SDK hashes are retained in the results.

The SDK --authored-method-bounds check now covers both callable forms, defensive
copying, conflicting interning, malformed bounds and scalar arguments before/after
explicit nominal conformance. Existing authored-reference and method-constraint
regressions also pass. Constructed interface bounds remain outside this slice.
