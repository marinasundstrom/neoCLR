# Raven compound operator qualification (2026-10-16)

Development support, paired with Raven main `7819e2fea` and its deliberate carry to
`codex/neoclr` at `164ebf81f`; metadata SDK element-address support is neoCLR `1b4561f3b`.
Broader structural work remains parked. Nothing here declares a release.

Raven's initial compound overloads are `+=`, `-=`, `*=`, `/=`, `&=`, `|=` and `^=`.
A declaration such as `func +=(amount: int)` is a public instance method with one
by-value parameter and unit result. Applicable operators mutate writable variables;
ordinary properties/indexers keep binary-plus-setter fallback. Assignments retain
Raven's unit result. No Runtime Contract flag, carrier type or runtime opcode is
introduced. CLI metadata uses standard special-name methods; native metadata uses
ordinary instance method signatures and calls. This follows the
[C# 14 in-place operator model](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/proposals/csharp-14.0/user-defined-compound-assignment)
while preserving Raven assignment-result semantics.

## Evidence

- Raven main: 70 focused operator, conversion and reference-field tests pass on
  .NET 11, including an emitted-library consumer, ambiguity, class alias identity,
  struct locals/array elements, field/index side effects, and binary/property fallback.
- Native compiler line: 62 focused compound/operator/instance-plan/reference-field tests pass.
- Native `Main.rvn`: interpreter, macOS ARM64 AOT and managed JIT all return 42.
  AOT host uses undefined-behavior/bounds sanitizers. Covers class alias identity,
  explicit-over-binary precedence, struct-local mutation and a reference-array index
  evaluated once. See `results.json` for exact commands and artifact hashes.
- Native `ValueArrays.rvn`: interpreter returns 42; AOT explicitly rejects allocation
  of the nominal struct array. See `value-array-results.json`. This is an existing
  backend limitation, not a copy-and-writeback substitute or successful AOT claim.
- Metadata SDK `--array-address-integration`: CLI and native interpreter checked
  element-address/store/read return 42; malformed element types are rejected.

The matched bundle is `target/function-reflection-library-validated-20261016`.
The script records the compiler, metadata SDK, runtime, code generator, bundle and
source fingerprints for the supported full-backend case.

## Reproduce

Build Raven's native compiler against the matching SDK, then run from neoCLR:

```sh
python3 docs/validation/compound-operators/run.py \
  --compiler ../Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/function-reflection-library-validated-20261016 \
  --output target/compound-operators-check
```

Add `--value-arrays` with a separate output directory to exercise the interpreter
success and expected AOT rejection. That profile does not attempt JIT execution.
The scripts use the existing native host from `interface-variance` and do not
install or publish a bundle.

## Remaining boundaries

Checked forms, compound extension/interface/virtual declarations, instance
increment/decrement, and missing remainder/shift compound tokens are follow-up
work. Native imported-library compound calls and other architectures are not
claimed by this fixture. The SDK is a host API documented in the
[manual reference](../../../api-docs/experimental-metadata.md#managed-array-element-addresses-development-2026-10-16);
no Raven class-library API or reference snapshot changed.
