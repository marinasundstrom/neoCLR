# Explicit ordinal text operations

Implemented 2026-09-08. This bounded text slice supports identifier, prefix and
suffix checks in ordinary Neo programs. Culture, search indexes and general encoding
abstractions remain unsettled. The development Raven comparison modes below add
explicit simple folding without changing these existing operations. The later
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

The original ordinal matching methods are case-sensitive, with no normalization or culture processing. Empty
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
The original methods expose the ordinal choice in their names; the development
comparison overload below now supplies an explicit enum for ordering. Reusing the unqualified names with only ordinal
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
Char now represents a grapheme cluster; explicit scalar access uses uint. See the
current [text abstraction](design/text-abstraction.md).

## Explicit comparison modes (development)

Implemented after Preview 10, 2026-09-27, for Raven consumers:

```raven
String.Compare("ReadMe", "README", StringComparison.OrdinalIgnoreCase)
String.CompareOrdinalIgnoreCase("ReadMe", "README")
HashMap<string, int>(StringComparer.OrdinalIgnoreCase)
```

`StringComparison` has `Ordinal = 0` and `OrdinalIgnoreCase = 1`. The numeric values
are neoCLR's own contract, not .NET enum values. Compare requires the mode; there is
no culture-dependent or two-argument default overload. An unnamed mode faults with
`Unsupported StringComparison`. All inputs are non-null valid Unicode strings.
Use the sign of comparison results. `CompareOrdinal` and the existing exact equality,
matching, Object hashing and interning contracts remain unchanged.

Ignore-case applies Unicode 17 default **simple** case folding (CaseFolding.txt C/S
mappings), then compares scalar sequences. Hashing encodes those same folded scalars
as UTF-8 and uses the existing content-hash algorithm. Equality means comparison
returns zero. Folded strings need not have equal UTF-8 byte lengths. The reusable
policy implements both comparer interfaces, so HashMap lookup, duplicate detection
and replacement use consistent equality and hashing while retaining the original
key spelling. Instance identity is not promised; hash codes are not persistent IDs
or a defense against deliberate collisions.

There is no normalization, culture tailoring, Turkic mapping or multi-scalar
expansion: `ß` differs from `ss`, and `é` differs from `e` plus combining acute.
`ẞ/ß`, `K/k`, `ſ/S`, sigma variants and supplementary Deseret case pairs compare equal.
Dotted/dotless I remain distinct from ordinary i/I. Empty strings, prefixes and NUL
follow ordinary lexicographic rules. Ordering is by the folded scalar, not by
uppercase text; for example `[` sorts before folded `A`.

### Comparison and alternatives

Primary sources reviewed 2026-09-27:

- [.NET String.Compare](https://learn.microsoft.com/en-us/dotnet/api/system.string.compare?view=net-10.0)
  provides explicit comparison modes; this is the ergonomic reference for callers.
- [.NET OrdinalIgnoreCase](https://learn.microsoft.com/en-us/dotnet/api/system.stringcomparer.ordinalignorecase?view=net-10.0)
  and the [pinned .NET 10 implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Globalization/OrdinalCasing.Icu.cs)
  use invariant uppercase-oriented comparison. neoCLR's simple folding deliberately
  differs in equivalence classes and ordering. The [local .NET 10 baseline](experiments/ordinal-text-dotnet/ignore-case-results.json) distinguishes
  `ẞ/ß`, `K/k` and `ſ/S`; neoCLR equates those pairs. Exact .NET compatibility would
  require a different policy and is not claimed. UTF-8 storage alone does not require
  this difference; simple folding is a separate provisional library choice.
- [Unicode 17 CaseFolding.txt](https://www.unicode.org/Public/17.0.0/ucd/CaseFolding.txt)
  defines C/S simple folding separately from F expanding and T tailored mappings.
  [Go strings.EqualFold](https://pkg.go.dev/strings#EqualFold) also uses simple Unicode
  folding for equality, demonstrating a useful UTF-8-oriented alternative; it does
  not establish neoCLR's ordering, hash algorithm or Unicode-version policy.

Simple folding offers one deterministic relation shared by comparison and hashing
without allocating transformed text. Its costs include differences from familiar
.NET results, the absence of linguistic matching, a checked-in Unicode mapping table
and version maintenance. ASCII-only folding would be smaller but surprising for
ordinary Unicode text; full folding would introduce expansions and different key
equivalence. Neither is silently substituted. No performance improvement over .NET
is claimed. Broader framework/alternative-.NET research belongs to the author-directed
[string review](tracking/library-data.md#string-design-review-before-further-expansion).

### Implementation and focused evidence

Raven owns the enum, mode dispatch and comparer policy. Two signature-checked native
helpers provide folding comparison and hashing through the existing StringOperations
service. No opcode, compiler configuration or Raven compiler change is needed.
The archived Neo String profile retains its existing methods; new modes belong to
the Raven profile. Rebuild reference, bridge, managed library and native runtime
together. Runtime quotas continue to apply; individual folding steps are not guest
instruction-metered, consistent with existing native text helpers.

`scripts/generate-string-case-folding.py` validates the pinned Unicode data SHA-256
before generating `src/string_case_folding.rs`; ordinary builds need no download.
The existing Unicode license covers the derived table. Folding uses Unicode 17;
segmentation and scalar classification still use 16, a review item rather than an
implicit platform-wide Unicode upgrade.

Focused checks: `cargo test --locked --lib string_comparison`, the executable
[sample](experiments/raven-target/samples/library-string-comparison.rvn) through
`verify_string_comparison.py`, and `--comparer-signatures`. They cover every mapped
scalar's idempotency/hash consistency, edge cases, interface dispatch and map usage,
invalid modes, explicit-mode enforcement and metadata admission. The API snapshot
is refreshed; no full test suite, performance claim or website build is required
for this bounded contract. See the [recorded outcomes](experiments/raven-target/string-comparison-validation.json).
