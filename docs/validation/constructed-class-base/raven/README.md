# Raven constructed class bases

Development native consumer on Raven `codex/neoclr`: a leaf declared before its
bases, `Leaf : Derived<bool, int> : Base<int>`, inherited `View<int>`, integer and
reference payloads, exact reference identity and inherited interface dispatch.
The expected result is 42.

```sh
python3 docs/validation/constructed-class-base/raven/run.py \
  --compiler /path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/raven-constructed-base-20261016
```

The runner requires three invalid conversion errors (reverse/invariant), and the
explicit deferred-binding diagnostic for `Recursive : RecursiveRoot<Recursive>`.
It checks interpreter, AOT and managed JIT execution, native cleanup and AOT
sanitizers; sources, compiler/adapter/SDK/runtime artifacts and bundle are hashed.
The focused Raven inheritance suite also checks capability gating and CLR behavior.

`--imported` builds the hierarchy as a separate native library before compiling
the consumer. Imported constructed hierarchies are still a separate validation
step; the initial attempt rejects Leaf's symbol-only emission contract.
External constructed base authoring, recursive base arguments and generic class
virtual slots remain unsupported. These are implementation gaps relative to .NET.
Structural types and Sliceable/Result variance remain deferred.

The local source run returns 42 in interpreter, AOT and managed JIT; cleanup and
sanitizer checks pass. All three focused Raven inheritance tests pass with CLR
execution. `results.json` retains the native validation evidence.
