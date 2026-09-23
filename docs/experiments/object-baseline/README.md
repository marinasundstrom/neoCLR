# .NET Object/value baseline

Run `dotnet run` from this directory with .NET SDK 10.0.100 (pinned in global.json).
The program fails on a mismatched assertion and prints each checked scenario plus
its runtime version. It verifies class aliasing, shallow value copies, boxed-value
copying and type identity, equality/hash agreement, null identity and type-name
formatting. Hash values themselves are deliberately not golden outputs.

This is comparison evidence, not a neoCLR feature sample. The corresponding current
neoCLR evidence is `tests/class_semantics.rs`, `tests/boxed_interfaces.rs`,
`tests/value_storage.rs` and the `object_get_type` cases in `tests/raven_reflection.rs`.
Object equality/hash/formatting are missing executable APIs despite reference stubs.
See [the review](../../object-model-review.md) for the implementation order and gaps.

Observed 2026-09-23: SDK 10.0.100, runtime .NET 10.0.0, all eight assertions passed.
The current neoCLR class semantics (10), boxed interface (9), erased storage (10)
and Object.GetType (3) regression cases also passed. These checks establish only
the listed behavior; they do not test yet-unimplemented Object default methods.
