# HashCode and record components

Development work, 2026-09-24. This is a bounded implementation, not a promise of
.NET API completeness or matching hash values.

## Baseline and choice

[.NET's HashCode](https://learn.microsoft.com/en-us/dotnet/api/system.hashcode?view=net-10.0)
is a mutable struct with generic component/comparer overloads and static Combine
helpers. Its [runtime implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/HashCode.cs)
uses an xxHash32-derived accumulator with a randomized seed. Records use component
equality and must give equal values equal hashes; reference storage remains distinct
from equality by represented value.

neoCLR starts with a mutable value accumulator, Add(int), Add(string), ToHashCode()
and Combine(int, int). This keeps hashing in the Raven library and avoids introducing
boxing or a generic comparer subsystem solely to enable the first record sample.
The cost is an incomplete overload set: callers cannot yet pass arbitrary values or
custom comparers. A copy preserves independent accumulator state. ToHashCode reads
the current state without consuming it.

The initial mixer combines ordered components with wrapping 32-bit arithmetic and
an avalanche finalization. String components use their UTF-8 contents without
normalization, through a temporary byte snapshot. Null strings are unsupported.
The implementation is deterministic today, has no randomized seed, and is not a
cryptographic or adversarial-input-resistant hash. Do not persist its output or
assume uniqueness, stable output between releases, .NET equivalence, or measured
performance/distribution quality. Revisit the algorithm and allocation cost before
using it as the basis of untrusted-key collections.

## Record integration

The author directs Raven to support neoCLR's record semantics. The implementation
must use the target's equality/hash contracts rather than silently importing host
.NET comparers, reflection formatting, or boxed-value behavior that neoCLR does not
provide. Default Raven/.NET record behavior must retain its existing path.

The first acceptance source is `docs/experiments/object-equality/RecordProbe.rvn`:
class assignment preserves reference identity, separately allocated equal records
retain distinct identities, typed and Object equality agree, operators agree, and
equal components produce equal hashes. Record structs, inheritance, generic records,
nullable components and arbitrary component types require explicit validation and
must not be inferred from a passing integer record-class example.
