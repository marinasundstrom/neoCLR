# Explicit ordinal text operations

Implemented 2026-09-08. This bounded text slice supports identifier, prefix and
suffix checks in ordinary Neo programs. It leaves culture, case folding, search
indexes and general encoding abstractions unsettled. The later
[grapheme contract](design/text-abstraction.md) defines String.Length and iteration.

## API contract

- `System.String.CompareOrdinal(String left, String right) -> Int32` compares
  lexicographically by UTF-8 bytes (equivalently Unicode scalar values for valid text). Use the sign of the result, not its
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
  compares numeric Char values and specifies the result's sign. The original implementation preserved that
  ordering, including supplementary characters sorting before U+E000. The author
  superseded this choice on 2026-09-19 with native UTF-8 semantics: supplementary
  characters now sort after U+E000. .NET also
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
String does not yet implement ComparableTo<String>: doing so would select a default
ordering, whereas this API keeps that choice explicit.

## Responsibility and cost

Public methods are ordinary System library IL. Four signature-validated InternalCall
helpers supply text access unavailable through current IL, all declaring the existing
StringOperations runtime service. No opcode, metadata format or Neo compiler change
is required. Readonly instance calls also require SlotReferences. There is no new
managed allocation or retained lifetime in the helpers. The interpreter's owned
String loads/calls can copy host buffers; readonly receivers do not promise zero-copy
execution. Ordering now compares the native UTF-8 bytes without UTF-16 transcoding.
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

The [Raven String projection](raven-string-api.md) exposes these operations through
ordinary member calls and documents the executable boundary samples.

## UTF-8 direction confirmed — 2026-09-19

The author selected native UTF-8 and the String proposal as the platform direction,
without a UTF-16 compatibility constraint. CompareOrdinal now follows unsigned
UTF-8 byte order, which agrees with Unicode scalar order for valid strings. This
is a deliberate ordering break from .NET and the earlier neoCLR implementation:
U+10000 now sorts after U+E000. Existing sorted data that relies on the old ordering
must be re-sorted. There is no culture collation or normalization change.

The benefit is a single native ordering with no UTF-16 projection; the cost is
compatibility with .NET ordinal sorting. Unicode recommends code-point order for
binary sorting ([Unicode 17, implementation guidelines](https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-5/));
[Rust str ordering](https://doc.rust-lang.org/core/primitive.str.html#impl-Ord-for-str)
compares byte values. Sources reviewed 2026-09-19. No speedup is claimed.
Char still has a legacy 16-bit representation pending the coordinated scalar-Char
migration; that is implementation debt, not the selected text model.
