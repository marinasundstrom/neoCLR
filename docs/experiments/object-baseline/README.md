# .NET Object/value baseline

Run `dotnet run` from this directory with .NET SDK 10.0.100 (pinned in global.json).
The program fails on a mismatched assertion and prints each checked scenario plus
its runtime version. It verifies class aliasing, shallow value copies, boxed-value
copying and type identity, equality/hash agreement, null identity and type-name
formatting. Hash values themselves are deliberately not golden outputs.

This is comparison evidence, not a neoCLR feature sample. The corresponding current
neoCLR evidence is `tests/class_semantics.rs`, `tests/boxed_interfaces.rs`,
`tests/value_storage.rs` and the `object_get_type` cases in `tests/raven_reflection.rs`.
Object equality/hash remain missing executable APIs despite reference stubs; bounded
class formatting is now implemented.
See [the review](../../object-model-review.md) for the implementation order and gaps.

Initial observation 2026-09-23: SDK 10.0.100, runtime .NET 10.0.0, all eight assertions passed.
The current neoCLR class semantics (10), boxed interface (9), erased storage (10)
and Object.GetType (3) regression cases also passed. These checks establish only
the listed behavior; they do not test yet-unimplemented Object default methods.

The identity follow-up adds six assertions (14 total), covering mutation/GC hash
stability, array identity, string identity through Object conversions, independently
allocated equal strings, custom equality/hash overrides and static null handling.
The GC request does not prove that a particular object moved. No unequal-hash or
cross-process-hash guarantee is asserted. SDK/runtime pins are unchanged.
`tests/object_identity_contract.rs` characterizes neoCLR's corresponding raw
identity prerequisites and its current string-wrapper gap; it does not expose a
new Object API or bless that gap as the eventual contract.

Follow-up observation 2026-09-23: all 14 .NET assertions and the six new neoCLR
identity cases passed, together with four existing reference-identity tests.

The record follow-up adds a fifteenth assertion using `record KeyRecord(int Number)`:
class identity remains distinct while generated typed/Object equality, operators
and hashes agree. The corresponding Raven acceptance source is
[RecordProbe.rvn](../object-equality/RecordProbe.rvn), currently blocked at emission
by a missing comparer contract; it is not a passing neoCLR sample.
