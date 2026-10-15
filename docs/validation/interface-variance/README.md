# Runtime interface variance — 2026-10-15

Development runtime metadata and interpreter qualification. This is not Raven
compiler/SDK support or native AOT/JIT qualification. Structural work stays parked.

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
  variant_interfaces_reject_before_erasing_generic_identity
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
reuse its existing box. Intrinsic String widening needs separate materialization
support and remains excluded. Borrowed slot/pointer view conversion remains exact.

Native specialization rejects used variant interfaces before dropping their
annotations. The test verifies the explicit diagnostic; it does not claim native
execution. The SDK now inspects annotations and preserves native snapshots; its
authoring/emission import, AOT/JIT dispatch adaptation and Raven emission remain
next stages. Sliceable and Result remain invariant. Structural compositions
containing variant interfaces have not been qualified by this slice.
