# Variance through an imported class base

`Library.rvn` declares an open Base class and variant Producer/Consumer interfaces.
`Main.rvn` is compiled separately and declares a local Derived/Leaf hierarchy.
The consumer widens Producer<Leaf> to Producer<Base>, narrows Consumer<Base> to
Consumer<Leaf>, and composes both directions in Producer<Consumer<T>>. Calls check
receiver and payload identity and mutate the inherited field through the widened
view. This exercises the SDK imported-base ancestry fix with actual objects.

Run on macOS ARM64 with a compiler rebuilt against the matching metadata SDK:

```sh
python3 docs/validation/interface-variance/imported-base/run.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bundle /absolute/path/to/native-bundle \
  --output target/imported-base-variance
```

`Rejected.rvn` verifies three source diagnostics: reverse covariance, reverse
contravariance and unrelated-class widening. The runner requires exactly three
RAV1503 diagnostics.

The runner emits the library and consumer independently, then uses the same images
in interpreter, AOT and managed JIT. Success returns 42. Native hosts check fault
state and root/collection cleanup; the AOT host also enables undefined-behavior and
bounds sanitizers. `results.json` retains commands and artifact hashes.

No new runtime conversion, adapter, box, generic class base, String materialization
or structural-type capability is introduced. Runtime Contract and metadata formats
are unchanged. The supplied native class-library bundle remains compatible; the
compiler must include the SDK ancestry fix in neoCLR commit `2c98cbd1b`.

Validation on 2026-10-16 uses Raven `9854031b8`, rebuilt against SDK fix `2c98cbd1b`,
and bundle `variance-argument-sequence-20261016`. Both assemblies compile, all three
execution modes return 42, and the three negative cases reject. Native and compiler
binaries, SDK DLL, sources and both emitted assemblies are fingerprinted in the
retained results. This qualifies the SDK fix through actual native Raven calls.

The consumer additionally checks interface-valued arguments widening to Object in
both producer and consumer variance positions. These checks exposed loss of the
native Object-root role during private renaming; the native fix retains that fact
without changing nominal identity or adding a conversion wrapper.

Array-valued arguments also pass in both directions: Producer<int[]> widens to
Producer<Object>, and Consumer<Object> narrows to Consumer<int[]>. Calls preserve
array identity, and a mutation is visible through the original producer. Array
element storage remains invariant. This extends the retained three-backend evidence
without a runtime or compiler code change.
