# Bounded Unicode casing and Int64 reporting

**Development implementation, 2026-09-27.** The author accepted the assistant's
proposed Unicode casing and Int64 parsing/formatting slices, plus missing numeric
bounds. This extends the [text review](text-abstraction.md) after its encoding
foundation; it does not select a new text representation or a globalization stack.

## Scenario and selected contract

A report needs uppercase labels and decimal file sizes exceeding Int32. Keep these
ordinary library APIs on String and Int64, backed by narrowly typed native services.
No new language construct, CLI type category, encoding contract or compiler option.

- `String.ToUpperInvariant()` and `ToLowerInvariant()` return immutable String values
  using Unicode 17 **full default casing**, including expanding mappings and the
  default final-sigma context. No locale tailoring, normalization or titlecasing.
  `ß` uppercases to `SS`; `İ` lowercases to `i` plus combining dot; `ΟΣ` lowercases
  to `ος`. UTF-8 bytes, scalar count and grapheme count can all change. This is not
  the existing simple-fold comparison policy and must not replace its hash policy.
- `Int64.Parse(string)` returns `Result<long, Int64ParseError>` with standard Raven
  union cases `InvalidFormat` and `Overflow`. Grammar: optional ASCII `+`/`-` then
  one or more ASCII digits. Leading zeros and negative zero are accepted. Whitespace,
  NUL, separators, prefixes and non-ASCII digits are rejected. The whole grammar is
  checked before range conversion: malformed suffixes win over overflow.
- `Int64.ToString()` returns decimal ASCII, with `-` only for negative values; no
  grouping, leading zeros, format strings or culture provider. Every value round-trips.
  Static read-only `MinValue`/`MaxValue` properties expose -9223372036854775808 and
  9223372036854775807. These are bounds values, not compiler constant fields.

## Comparison and evidence

Primary sources retrieved 2026-09-27:

| Source / status | Relevant contract and decision |
| --- | --- |
| [.NET 10 ToUpperInvariant](https://learn.microsoft.com/en-us/dotnet/api/system.string.toupperinvariant?view=net-10.0), [runtime v10.0.0 String implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/String.Manipulation.cs), shipped | Familiar instance methods delegate casing to library globalization machinery. We retain the names and return shape, but explicitly choose full Unicode default casing rather than matching every .NET invariant result. The local .NET 10 probe leaves `ß` and the ffi ligature unchanged in uppercase and produces ordinary sigma at the end of `ΟΣ`; neoCLR expands/uses final sigma. |
| [.NET Int64.Parse](https://learn.microsoft.com/en-us/dotnet/api/system.int64.parse?view=net-10.0), shipped | Same integer range; overloads provide styles/culture and throw on failure. The local baseline accepts surrounding whitespace. neoCLR retains its existing Int32 strict ASCII grammar and typed Result errors; no overload family or exceptions are introduced. |
| [Unicode 17 §3.13](https://unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/), [SpecialCasing](https://www.unicode.org/Public/17.0.0/ucd/SpecialCasing.txt), normative | Full mappings combine UnicodeData and SpecialCasing; default lowercasing includes final sigma. Context examines original text, ignoring Case_Ignorable. Language-specific rules are excluded. No normalization is implicit. |
| [Rust str casing](https://doc.rust-lang.org/std/primitive.str.html#method.to_lowercase), [i64 parsing](https://doc.rust-lang.org/std/primitive.i64.html#method.from_str), shipped | Whole UTF-8 text can expand/contextually transform; signed integer parsing rejects surrounding whitespace. This is a closer behavioral fit than treating a grapheme as one case-mappable scalar. neoCLR still uses managed String ownership and Raven errors. |
| [.NET runtime #30960](https://github.com/dotnet/runtime/issues/30960), open design discussion when retrieved | The proposal discusses platform-dependent casing/comparison data and compatibility costs of pinning it. It is not evidence that a proposed change shipped. Our checked-in tables make the chosen Unicode version explicit, at the cost of maintaining/updating them. |

[Executed baseline and consumer](../experiments/casing-integer/README.md) record
observed results separately from platform documentation. The existing broader text
review supplies Swift/Go and ecosystem context. No additional independent .NET
library comparison was necessary to justify these small, ordinary library methods;
none is claimed as evidence of a CLR limitation.

## Alternatives, benefits and costs

Matching .NET simple invariant mappings would reduce porting surprises but omit
useful expansions/context for labels. Selecting culture implicitly would make output
depend on ambient state. Full default casing gives a documented, host-independent
transformation while leaving language-specific presentation for later selection.
The familiar `Invariant` suffix signals absence of culture selection; it does not
promise .NET-equivalent output. No parameterless culture-sensitive aliases are added.

Using Rust's host-version casing directly would be smaller to implement, but compiler
updates could change the text contract. Instead the generator checks SHA-256 for
UnicodeData, SpecialCasing and DerivedCoreProperties 17.0.0 and emits checked-in tables.
The only non-locale contextual rule is explicitly validated as Final_Sigma. Native
code traverses original UTF-8 scalars, emits mappings, and allocates an owned output
String; mappings can grow output. No borrowed view survives the call. No performance
claim or new benchmark requirement follows from this implementation. Existing runtime
resource/fault behavior applies; this API does not promise an output quota.

Int64 parsing reuses Int32's erased native status protocol; Raven constructs the
public Result/union, preserving ordinary union metadata, matching and GC behavior.
Formatting reuses the existing Int64 native formatting service. Public bounds are
computed static getters under the existing primitive bridge; constant-expression
use is not supported by these properties. A generic numeric parser/interface or
format-provider hierarchy would add scope without helping this consumer.

Casing/folding use Unicode 17, while current grapheme segmentation/classification
remain on Unicode 16. This existing version split stays explicit; new mapping data
does not silently change Char boundaries. Normalization, locale casing, Unicode-wide
version alignment and arbitrary numeric formatting remain separate work.

The archived Neo bootstrap does not adopt the new Int64 Parse/standard-union API.
Its generated primitive fragment omits Parse and unused adapters; the current Raven
library uses the complete source. This avoids inventing another manual carrier or
changing archived union contracts. An initial native test exposed this boundary;
focused default-library checks validate the repaired split.

## Validation boundary

Focused native mapping/context and parse/binding tests, exact bridge signature
checks and one executable Raven report cover the new paths. The report tests both
Int64 extrema, expansion, combining marks, supplementary text, NUL rejection and
format-before-overflow. API/library snapshots must match the tested bridge. Native
runtime services are additions: matching development runtime and System library are
required; Preview 10's binary cannot execute these new native calls. No full suite,
platform matrix, website build or new release is implied.
