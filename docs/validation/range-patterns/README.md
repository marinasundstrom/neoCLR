# Native signed integer range patterns — 2026-10-15

Raven compiler commit: `e1ec333df35a6571f1f53f4192420d354395c5a0` on codex/neoclr.
Runtime and libraries remain at the prior range qualification.

This follow-up closes the signed Int32/Int64 range-predicate gap from the
[endpoint carry](../range-endpoints/README.md). Structural work remains parked.

## Contract

Raven's native backend consumes the same range-pattern nodes as ordinary .NET:
`..` excludes the upper endpoint; `..=` includes it. The lower bound is inclusive.
Open bounds need one comparison. A bounded pattern saves the subject once and
short-circuits the upper comparison when the lower test fails. Inclusion compares
directly, without adding one to the endpoint. Reversed bounds match no value.

No new runtime service, native opcode, library API, metadata format or Runtime
Contract option is required. Raven owns the portable lowering and neoCLR uses its
existing signed comparisons. This restores the ordinary Raven/.NET behavior in
native conditions, Boolean values and match arms; it is not a new numeric or
ordering contract.
The portable backend lives on codex/neoclr, not on Raven main.

## Qualification

[Main.rvn](Main.rvn) checks exclusive and inclusive boundary cases, open bounds,
match arms, Boolean return values, reversed bounds, subject side effects,
values above Int32.MaxValue,
and an inclusive Int64.MaxValue endpoint. [host.c](host.c) checks native faults
and zero arena allocation. The existing five-library range bundle is reused;
no library or runtime sources changed in this slice.

[results.json](results.json) retains executable commands and binary/source hashes.
The consumer returns 42 in interpreter, macOS ARM64 AOT and managed JIT.
AOT uses undefined/bounds sanitizers; JIT is not sanitizer-instrumented.
This is correctness evidence, not a throughput claim or cross-platform matrix.

## Limits

- This slice supports signed int/long patterns without bindings in conditions,
  Boolean values and match arms. Other numeric types and user-defined ordering
  remain unqualified.
- The shared binder requires matching endpoint types: use `1L..=3L` for a long
  subject. This is an existing binder rule, not a native-specific restriction.
- Inclusive stored/sliced System.Range construction still requires checked Index
  conversion and native Fault transport. Integer loop advancement still needs
  overflow qualification. Neither gap is closed by pattern comparisons.

The initial regression fixture omitted the existing pattern capability, so it
rejected pattern expressions. Enabling the actual target capability qualifies both
conditions and direct Boolean returns. Unsuffixed int bounds for long subjects
were rejected by the existing shared binding rule. An initial ref-parameter helper
passed the interpreter but required an AOT borrow output contract; the final
single-evaluation check uses local block side effects, independent of that ABI.

Focused compiler validation: 32 range/index/portable-pattern tests pass on .NET 11.
The native-enabled compiler build passes; no library rebuild or website build is
needed for this backend-only change and accompanying text documentation.
