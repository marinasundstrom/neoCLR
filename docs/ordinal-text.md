# Explicit ordinal text operations

Implemented 2026-09-08. This bounded text slice supports identifier, prefix and
suffix checks in ordinary Neo programs. It leaves culture, case folding, search
indexes, Rune iteration and general String.Length unsettled.

## API contract

- `System.String.CompareOrdinal(String left, String right) -> Int32` compares
  lexicographically by UTF-16 code units. Use the sign of the result, not its
  magnitude. Equal text returns zero, consistent with String.Equals.
- `readonly String.ContainsOrdinal(String value) -> Boolean` finds exact text.
- `readonly String.StartsWithOrdinal(String value) -> Boolean` tests a prefix.
- `readonly String.EndsWithOrdinal(String value) -> Boolean` tests a suffix.

The instance methods use readonly managed receivers in metadata. Neo borrows
locals and materializes temporaries automatically; direct IL callers load an
address. Arguments and static comparison inputs are owned String values. No
reference is retained. Host invocation of instance methods needs a guest wrapper
that borrows its String parameter, as with Equals.

All matching is case-sensitive, with no normalization or culture processing. Empty
patterns match every string, including empty text. Embedded NUL is ordinary text.
Combining marks can match independently of a grapheme. Inputs are valid Unicode
text; null and unpaired UTF-16 surrogates are not supported String values. These
operations have no recoverable error result for valid inputs; existing runtime
faults and execution limits still apply.

## .NET baseline, alternatives and decision

Primary .NET 10 API sources consulted 2026-09-08:

- [String.CompareOrdinal](https://learn.microsoft.com/en-us/dotnet/api/system.string.compareordinal?view=net-10.0)
  compares numeric Char values and specifies the result's sign. Preserve that
  ordering over our valid-text domain, including supplementary characters sorting
  before U+E000. UTF-8 byte/scalar order would reverse that example. .NET also
  accepts null inputs; our current String contract does not.
- [String.Contains](https://learn.microsoft.com/en-us/dotnet/api/system.string.contains?view=net-10.0)
  provides ordinal matching and an explicit comparison overload.
- [String.StartsWith](https://learn.microsoft.com/en-us/dotnet/api/system.string.startswith?view=net-10.0)
  and [String.EndsWith](https://learn.microsoft.com/en-us/dotnet/api/system.string.endswith?view=net-10.0)
  provide explicit ordinal overloads, while their String-only defaults use culture.
  Empty patterns succeed in these APIs.

These are library behaviors, not new CLI instructions or C# syntax requirements.
We expose the ordinal choice in method names until comparison options have a
proper platform representation. Reusing the unqualified names with only ordinal
behavior would hide a difference from familiar prefix/suffix defaults. Introducing
an entire comparison enum and culture implementation now would enlarge the slice.
The suffix costs source compatibility and verbosity; future overloads can coexist.
It is a provisional API choice, not a claim that .NET lacks deterministic matching.
String does not yet implement Comparable<String>: doing so would select a default
ordering, whereas this API keeps that choice explicit.

## Responsibility and cost

Public methods are ordinary System library IL. Four signature-validated InternalCall
helpers supply text access unavailable through current IL, all declaring the existing
StringOperations runtime service. No opcode, metadata format or Neo compiler change
is required. Readonly instance calls also require SlotReferences. There is no new
managed allocation or retained lifetime in the helpers. The interpreter's owned
String loads/calls can copy host buffers; readonly receivers do not promise zero-copy
execution. UTF-16 ordering streams encoded units without constructing a UTF-16 buffer.
Exact UTF-8 matching has the same Boolean results as UTF-16 ordinal matching for
valid-text patterns; unlike ordering, it does not need transcoding. No performance
improvement is claimed. Host primitive work is not individually instruction-metered,
as with existing text helpers; bounded hostile-input CPU quotas remain future work.

## Validation and usage

`tests/ordinal_text.rs` covers Neo, direct IL and serialized artifacts, Unicode
ordering, empty patterns, NUL, normalization differences and service declarations.
The comparison probe uses the pinned .NET SDK 10.0.100 targeting net10.0:

```sh
(cd docs/experiments/ordinal-text-dotnet && dotnet run)
cargo test --locked --test ordinal_text --test strings --test library
cargo run --locked -- run examples/source/ordinal-text.neo
```

The Neo example searches a list of file names using an eager predicate and matches
its Option result. It prints `café.neo` and returns 42. LINQ remains future work.
