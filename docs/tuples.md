# Value tuples (development)

Author direction, 2026-09-28: neoCLR calls its value-tuple family `System.Tuple`,
corresponding to .NET's `System.ValueTuple`, rather than .NET's reference `System.Tuple`.
This is an additive development API after Preview 11. Matching Raven, compiler
reference, importer and System library artifacts are required.

## Contract and implementation boundary

The first slice provides ordinary sequential generic value types with one through
seven components. Each exposes mutable public `Item1` through `ItemN` fields and a
constructor accepting the components in order. Copying the tuple copies its fields;
reference components still refer to the same objects. Element labels are compiler
metadata, not additional runtime fields or separate nominal identities.

Raven tuple expressions, type annotations and deconstruction use this family on
the neoCLR target. Empty `()` remains `System.Void`, following the existing unit
contract. The one-component type is available through explicit construction; Raven
does not have single-element tuple type syntax. Larger flat tuples and the `Rest`
family, tuple-specific equality/ordering/formatting interfaces and an `ITuple`
reflection facade are outside this initial slice. Nesting ordinary tuples is the
bounded composition mechanism. A probe of an eight-element expression is currently
rejected during emission with a generic failure; a dedicated source arity diagnostic
is still pending. No tuple-specific native allocation or opcode is
introduced. These limits are not a claim of full ValueTuple API compatibility.

## Comparison and tradeoffs

Primary sources checked 2026-09-28:

- [.NET 10 ValueTuple pair API](https://learn.microsoft.com/en-us/dotnet/api/system.valuetuple-2?view=net-10.0)
  documents mutable fields, construction, equality, hashing and comparison. The
  [v10.0.0 implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/ValueTuple.cs)
  is the pinned library baseline; these are ordinary generic structs, not special
  CLI tuple storage. neoCLR adopts that basic representation under the author's name.
- [C# tuple design](https://github.com/dotnet/csharplang/blob/main/proposals/csharp-7.0/tuples.md)
  is the historical language design, distinct from current runtime implementation.
  It separates tuple syntax/names from underlying types. neoCLR keeps that separation;
  Raven, rather than C#, owns the actual language contract here.
- [Rust tuples](https://doc.rust-lang.org/std/primitive.tuple.html) are heterogeneous
  positional values with element-dependent traits. Their language-native representation
  and ownership rules do not transfer directly to neoCLR's managed metadata model.

Keeping .NET's ValueTuple name would reduce compiler and interoperability work, but
would contradict the chosen public name. Reusing the reference Tuple model would
change copy/allocation semantics. A new native tuple mechanism would duplicate
existing value/generic storage without a demonstrated benefit. Ordinary generic
values keep layout, lifetime and field checking in existing runtime mechanisms.
The cost is compiler target mapping, importer admission and larger copies as the
component list grows. No performance improvement is claimed. Runtime names differ
from .NET, so cross-runtime binaries require explicit translation, not simple aliasing.

The bounded API does not need an independent .NET tuple library; no such alternative
was evaluated as a replacement. The upstream language design supplies rationale,
not evidence that every historical discussion describes a current defect.

## Validation

The [consumer](experiments/tuples/Main.rvn) and the experiment's focused verifier
own executable evidence. Compiler metadata tests must check value-type identity,
closed nested arguments and target assembly scopes; importer negatives must reject
malformed tuple layouts/signatures. Native behavior checks cover field access,
copying, nested values, labels, deconstruction and explicit construction. Wider
arities and additional APIs need their own contracts before being advertised.
