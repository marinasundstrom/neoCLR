# Imported class inheritance in interface variance

The separately compiled library declares `Base : Readable`, `Middle : Base` and
`Leaf : Middle`. The consumer uses `Producer<Leaf>` as `Producer<Readable>` and
`Consumer<Readable>` as `Consumer<Leaf>`. It checks receiver and payload identity,
calls the inherited interface member, and observes mutation of the inherited
field through the widened view. Reverse producer/consumer conversions and an
unrelated class reject with three RAV1503 diagnostics.

```sh
python3 docs/validation/interface-variance/inherited-interface/run.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bundle /absolute/path/to/native-bundle \
  --output target/inherited-interface-variance
```

The SDK walks explicitly registered dependency class bases when finding nominal
interface views. Importing names alone does not assert ancestry: focused SDK tests
reject missing and incomplete base chains, admit the complete chain in both
variance directions, and reject reverse widening. CLI emission is checked by
loading and invoking the generated conversion methods on the CLR.

This fixes metadata validation, not dispatch representation. Runtime Contracts,
CLI/native encodings and the native call ABI are unchanged. Generic class bases,
Sliceable/Result variance and structural work remain outside this slice.

Validated on 2026-10-16 with Raven `codex/neoclr` revision `4a08a7848`, rebuilt
against this SDK fix, and bundle `variance-argument-sequence-20261016`.
Interpreter, AOT and managed JIT return 42. Three source rejections, native
fault/root/collection checks and AOT undefined-behavior/bounds sanitizers pass.
`results.json` records commands and hashes for the compiler, SDK, runtime,
backend, bundle and emitted artifacts.
