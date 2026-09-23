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

## String and nested-record components — 2026-09-24

The next checked case is Person(Name: string, Age: int), wrapped by an Entry record.
The interface remains Equatable<Record>; a separate concern is comparing each
component. The [.NET record specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/proposals/csharp-9.0/records)
uses EqualityComparer<T>.Default for fields. [String equality](https://learn.microsoft.com/en-us/dotnet/api/system.string.equals?view=net-10.0)
is ordinal; these sources were reviewed on 24 September 2026. A general target
comparer would cover more types but would require settling boxed values, null
representations and comparer selection first.

This bounded adaptation uses the target String equality operator and HashCode.Add(string),
and invokes typed Equals/GetHashCode on same-compilation record-class components.
It retains the target's UTF-8 string content semantics with no normalization; it does
not promise .NET hash values or support invalid UTF-16 data. Nested hash values enter
Add(int). Formatting passes strings directly and invokes nested ToString, avoiding
host reflection formatting. Deconstruction copies string contents/reference values
through declared outputs; nested class references preserve identity.

The benefit is a useful domain sample without boxing or a new public comparer API.
Costs: generated policy is specialized, external record metadata is not yet recognized,
string hashing allocates, and recursive graphs have no cycle detection. Nullable
components, arbitrary objects, inheritance and record structs remain diagnosed.
Defensive null handling for nested class values is tested through metadata invocation;
this does not add nullable Raven component support or null-string runtime support.
The normal Raven/.NET synthesis path is unchanged. The experimental hash contract now
requires Add(string) as well as Add(int) and ToHashCode; incomplete providers receive RAVT003.
