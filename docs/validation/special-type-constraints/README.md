# Special generic type constraints

```sh
python3 docs/validation/special-type-constraints/run.py \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/special-type-constraints-20261016
```

The runner assembles `Main.neoil`, executes it in the interpreter, compiles and
runs AOT with undefined/bounds sanitizers, and executes managed JIT. The expected
result is 42; the native host also checks cleanup. `results.json` records commands
and exact source/runtime/codegen/bundle fingerprints.

`ReferenceBox<T>` requires `reference new`; `Derived<Unused,T>` forwards those
requirements through its constructed base. `ValueBox<T>` requires `value new`
and is instantiated with both Int32 and ValueBox<Int32>. The constructor constraint
uses the ordinary public parameterless constructor contract, not a new allocation
operation. All conformance remains nominal. This reuses the existing [.NET-compatible special constraint contract](../../generic-constraints.md); the change is native specialization support, not a new constraint semantics.

The codegen regression checks the unchanged source metadata and rejects reference/
value mismatches, private constructors and abstract constructor arguments before
creating executable closed types. The runner asserts that all four selected constructions retain their original
constraints and exact arguments in the native metadata catalogue. Nominal bounds, constrained methods/interfaces and
Raven's constrained dependency import remain separate gaps. Structural work stays
parked. This is macOS ARM64 development qualification.
