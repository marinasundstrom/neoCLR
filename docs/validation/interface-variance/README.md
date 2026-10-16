# Runtime interface variance — 2026-10-15

Development runtime metadata and interpreter qualification. The original interpreter evidence is retained below; the 2026-10-16 section adds Raven and native qualification. Structural work stays parked.

`Main.neoil` declares a producer of Derived and a consumer of Base, then uses them
through Producer<Base> and Consumer<Derived>. It checks receiver/payload identity;
the consumer replaces its local parameter with a new Base to exercise argument-slot
adaptation. There are four explicit object constructions and no variant wrapper.
The retained NEOX round-trip returns 42 and reports four allocations, zero live
objects and four reclaimed objects after execution.

Reproduce from the repository root after building the runtime:

```sh
mkdir -p target/interface-variance
cargo build --bin neoclr
target/debug/neoclr assemble docs/validation/interface-variance/Main.neoil \
  target/interface-variance/Variance.neox --format neox
target/debug/neoclr run target/interface-variance/Variance.neox \
  --execution interpreter --gc-stats
```

Exit status 42 is the consumer's success result. Remove an earlier generated
Variance.neox before reassembling if needed; the CLI refuses to overwrite artifacts.

Focused automated evidence:

```sh
cargo test --test interface_variance
cargo test --lib variance::tests
cargo test --test nominal_interfaces --test boxed_interfaces \
  --test explicit_interfaces --test default_interfaces --test interface_inheritance
cargo test -p neoclr-codegen --lib \
  variant_interface
```

The checks cover metadata defaults/round-trip, malformed annotations, nested
polarity, invariant Result-like wrappers, Self rejection, allowed reference
relationships, forbidden directions/value arguments/array covariance, exact
matching before variant matching, ambiguity, explicit nominal conformance,
producer/consumer calls, already-boxed receiver identity and no additional box,
and conservative generic dispatch-graph rejection. Standard interface regression
checks retain inherited/default/explicit and boxed behavior.

The runtime keeps distinct constructed type identities and declared interface
closures. A widened reference view does not change the type's declared interfaces.
Reference-only here excludes implicit boxing; an explicitly boxed receiver may
reuse its existing box. The original qualification excluded intrinsic String;
[String qualification](strings/README.md) now covers identity-preserving
materialization and its interpreter allocation cost. Borrowed slot/pointer view conversion remains exact.

## Native qualification — 2026-10-16

Native specialization now preserves the original interface definition, lowered
arguments, annotations and contract slot as private compiler facts. Selection
retains compatible contract peers and uses the same exact-first, unambiguous
nominal resolution as the interpreter. These facts cannot be supplied by serialized
metadata and do not add declared inheritance edges.

`VarianceLibrary.rvn` and `VarianceConsumer.rvn` are separately compiled by Raven
revision `44bbd2934`. The consumer exercises both directions of declared variance.
It returns 42 in interpreter, macOS arm64 AOT and managed JIT using the bundle
`target/sliceable-libraries-20261015-final`. `native-results.json` retains artifact
hashes, host commands and results. `native-host.c` checks no fault, an empty root
chain and complete collection; AOT runs with undefined/bounds sanitizers.

The compiler probe reproduces these images with `NeoClrMetadataProbe
--interface-variance <NeoCLR.CoreProbe.dll> <output>`. Supply both library images,
the bundle's `System.Runtime.dll` as module and object root, and
`System.runtime.neox` as system when running. Native compilation uses
`--closed-world`, `--compile-system`, `--reference-arena`, `--bind-user-fault`,
`--native-gc` and `--native-stack-budget`; managed JIT adds `--jit-managed`.

Codegen library regression tests pass (62 passed, one ignored at qualification),
plus the subsequent focused generic-implementation test. Existing runtime
interface suites pass. This does not widen Result/Sliceable or resume structural
work. Standard-library annotations and guest generic-parameter introspection are
subsequent slices.

## Bounded scope checkpoint — 2026-10-16

The author postponed Sliceable and Result variance. Both remain invariant; their
adapter/value-conversion proposals are not pending acceptance gates for the
reference-only interface feature. The runtime regression
`reference_variance_does_not_cross_invariant_boundaries` verifies that reference
widening does not cross unannotated interfaces or nominal value wrappers, even
inside variant contracts, and accepts each exact-signature control. Existing
String, array and imported-base executable evidence remains applicable; this
checkpoint changes no runtime ABI or compiler contract.
