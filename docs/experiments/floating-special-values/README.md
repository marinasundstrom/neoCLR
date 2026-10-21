# Floating special values — development 2026-10-21

Single and Double expose NaN, PositiveInfinity and NegativeInfinity as actual
constant fields, plus IsNaN, IsInfinity, IsPositiveInfinity, IsNegativeInfinity and
IsFinite. This follows [.NET Double.NaN](https://learn.microsoft.com/dotnet/api/system.double.nan)
and its classification API. Classification preserves the operand, accepts all NaN
sign/payload representations and regards either signed zero as finite. Primitive
operators retain IEEE comparisons; Equals retains value equality. No static storage,
initialization, runtime division or managed allocation is needed to load a constant.

The shared metadata SDK now carries typed object? constants, with ordinary CLI
Constant rows and native exact-bit string payloads. Old Int32 literals retain their
representation. Floating payloads require matching readers. The metadata tests cover
exact NaN payload and signed-zero bits across CLI/native/CLI-reference snapshots and
reject malformed bits and mismatched types. Runtime reflection/dynamic field reads
preserve primitive type and literal immutability. See the bridge docs and experimental
metadata API reference for host migration from int? Constant.

The Raven source uses constant expressions; the compiler folds them into literal
metadata and inline values. Native type-constant support and Single constant loading
were gaps. Portable lowering exists only on codex/neoclr, not Raven main. A separate shared
constant-evaluator correction is developed on Raven main and carried deliberately:
qualified imported literal fields now initialize dependent constants on both targets.
Runtime Contract configuration is unchanged. Constant instance calls use the
ordinary temporary-value receiver path instead of attempting to address nonexistent
field storage. Main 13f34e357 passes all 46 focused constant tests.

Built-in tests cover public constants, source/imported constant use, classification
of both widths/signs, finite values and signed zero, IEEE versus value equality and
format/parse integration. The record suite now uses named constants. Results and
artifact hashes are retained in validation.json. This is development qualification,
not a matched release bundle, Windows acceptance or performance evidence.

## Generic math scope

.NET [generic math](https://learn.microsoft.com/en-us/dotnet/standard/generics/math)
exposes classification through INumberBase and special floating values through
IFloatingPointIeee754. neoCLR Number remains the smaller existing arithmetic/ordering
contract. A future Self-based floating capability is only a proposal; this change
does not introduce it or claim generic numeric parity. The author subsequently
proposed IntegerNumber and FloatingPointNumber derived capabilities for future
work; an IEEE-754 sub-contract is an assistant proposal to keep decimal formats
possible without NaN/infinity requirements. No hierarchy expansion is implemented.
Resume release stabilization
after this bounded concrete API increment.

## Native scalar-array admission

The classification table tests exposed a separate admission omission: Single/Double
arrays were rejected during native specialization, although the existing scalar-array
layout and typed load/store path already represent both widths. They now use that
shared pointer-free one-lane path, retaining element identity, bounds checks and
initialization markers. This is ordinary .NET-like array behavior, not a new
representation or a claim of packed storage; the private native layout uses eight-byte
slots for all scalar elements. No byte-buffer or Slice ABI is changed.

The same tests exposed Single unary negation missing from native admission/emission.
It now follows the existing floating negation path while preserving Single width.
The interpreter already normalizes Single storage onto its Double evaluation stack;
that existing policy is unchanged and tested through the normalization boundary. A focused Rust regression verifies the sign bit for zero,
negative zero, a finite value, infinity and a quiet NaN payload, and retains rejection
of floating bitwise complement. Built-in classification includes negated NaNs.

## Qualification result

All 33 built-in executions pass (three floating tests plus eight record tests on
interpreter, JIT and native) with Raven 58980b280. The metadata roundtrip/malformed
checks, two Rust metadata tests, floating-stack negation regression, strict root/backend
Clippy checks, API snapshots and 5,070-page website build pass. See validation.json
for exact executable/compiler/library artifacts and the reproduced failure baselines.
