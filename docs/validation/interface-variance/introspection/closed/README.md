# Closed generic introspection

This consumer checks that `Container<int>` and `Container<string>` remain distinct
runtime types, repeated tokens compare equal, both expose exactly one generic
argument, and those arguments retain their concrete identities rather than becoming
generic parameter descriptors. It compares indexed and sequence-returning queries for both constructions and
checks the empty sequence for a nongeneric type. It does not establish native
open-definition discovery.

Run on macOS ARM64:

```sh
python3 docs/validation/interface-variance/introspection/closed/run.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bundle /absolute/path/to/native-bundle \
  --output target/closed-generic-introspection
```

The runner compiles once and runs that image in the interpreter, AOT and managed
JIT. The native host checks guest faults and GC/root cleanup; AOT uses undefined
behavior and bounds sanitizers. `results.json` records artifact hashes and outcomes.

The [sibling open-definition consumer](../README.md) separately qualifies explicit
assembly/module catalog discovery. Neither consumer establishes unrestricted
member reflection or open generic invocation. Structural work remains parked.
