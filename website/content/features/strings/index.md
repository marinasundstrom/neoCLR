# Strings, characters and UTF-8

String represents Unicode text. Char represents one grapheme cluster: a practical approximation of a character as a reader perceives it. UTF-8 is the canonical internal encoding; scalar and byte access are explicit.

**Preview 10 API.** Use matching runtime, SDK and library artifacts. See [setup](../../try/#development) for package availability.

[See a working example ↓](#characters) · [Download the complete sample](../../samples/library-utf8.rvn)

<a id="characters"></a>

Development builds also support `String.Concat(Object?, Object?)`, using virtual
`ToString` in left-to-right order and empty text for null. This supplies the runtime
contract for Raven interpolation of numbers and other objects. Formatting follows
the value's `ToString` implementation; format-provider overloads are not included.

## Character counting and iteration

`Length` counts grapheme clusters. Iterating a string yields `Char` values, including combining sequences and emoji sequences. The read-only integer indexer addresses grapheme clusters.

```raven
{{GRAPHEME_COUNTS}}
```

This prints 4 characters, 12 Unicode scalars and 37 UTF-8 bytes.

```raven
{{GRAPHEME_ITERATION}}
```

`Char.FromString(text)` accepts exactly one cluster and faults otherwise. Character literals such as `'é'` and `'👨‍👩‍👧‍👦'` also work. Use `GetScalars()` for a `Sequence<uint>` of scalar values and `UnicodeScalar` for classification.

Segmentation is pinned to Unicode 16. Length scans the text; iteration currently copies a snapshot. Graphemes are not always single displayed glyphs, and concatenation can change boundaries. Equality remains ordinal: precomposed é and e plus a combining accent are distinct even though both count as one character.

[Download the complete character sample](../../samples/library-grapheme-strings.rvn) · [Expected output](../../samples/library-grapheme-strings.expected.txt)

<a id="conversion"></a>

## UTF-8 encoding and decoding

`Utf8.Encode(text)` returns `Sequence<byte>`, with Count, indexed access and iteration. `Utf8.Decode(bytes)` accepts that same collection interface and returns `Result<string, InvalidUtf8Error>`.

```raven
{{UTF8_ROUNDTRIP}}
```

Use patterns to extract a successful value or its error. For example, `RoundTrip("café 🌍")` prints `Round trip`. Malformed UTF-8 produces an error instead of replacement text. Conversion currently copies bytes; it is not a zero-copy API.

<a id="try"></a>

## Use the matching Preview 10 toolchain
Open the prepared Raven project in VS Code, replace Main.rvn with the complete sample, save it, then run the neoCLR build/run task. Use matching reference and runtime libraries.

[Download Raven source](../../samples/library-utf8.rvn) · [Download expected output](../../samples/library-utf8.expected.txt) · [Toolchain setup →](../../try/#development)

The downloadable source and expected output are the same files used by the saved-project integration check.

<a id="direction"></a>

## Planned work and open questions

This is an evolving preview API. We are considering more efficient iteration, text positions for slicing, normalization and language-sensitive comparison. A dedicated scalar value type may replace uint in the explicit scalar view. Encoding conversion happens at boundaries, leaving the meaning of String and Char unchanged. Possible future Utf8String and AsciiString types could offer encoding-specific operations and guarantees alongside the default text container. Those additional string types remain deferred; the development conversion interfaces are described below.

Unlike .NET’s UTF-16 Char, neoCLR’s Char can contain multiple Unicode scalars. Numeric character casts and fixed-size character memory assumptions must change. Compiler literal diagnostics currently follow the host Unicode rules; the runtime validates the pinned target rules. CLI constant fields are outside the tested character surface.

.NET offers configurable UTF-8 decoding, including a strict mode. This implementation starts with a strict Result and Sequence-based byte access. Whether those are the right long-term contracts is open for feedback.

Text readers default to strict UTF-8 and support selected encodings in development;
the [HTTP and JSON guide](../web/) describes the existing bounded document conversion.
Future HTTP charset handling can reuse the encoding foundation.

[Implementation details, tests and design comparisons →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/raven-string-api.md)

[See the broader proposals and open questions →](../../proposals/#text)

<a id="feedback"></a>

## Questions and contributions

- Is strict Result-based decoding useful for your input format?
- Do you need an invalid-byte offset or streaming before you can use it?
- Does Sequence provide enough access for your byte-processing code?
- Does grapheme iteration match the text your application handles?
- Which operations need explicit scalars, normalization or text positions?

Share a concrete input, the code you tried and the result you expected.

[Discuss on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)

Questions, sample programs and documentation corrections are welcome. See [how to contribute](../../#feedback) for ways to participate.


<a id="char-through-object-development"></a>

## Char through Object

Boxing a Char preserves a copy of its complete grapheme text. Object equality and
hashing use that exact text; Object ToString returns it unchanged. Separate boxes
retain separate identities. This includes combining sequences and joined emoji.
Composed `é` and decomposed `é` remain different keys because comparison does not
normalize text. .NET Char instead represents one UTF-16 code unit; neoCLR's hash
therefore follows its text representation, not .NET's code-unit formula.

Use the [Char API reference](xref:System.Char) for construction, typed equality and
comparison, and the [Object guide](/docs/objects.html) for boxing contracts.


<a id="string-through-object-development"></a>

## String through Object

Text retains exact content equality and matching hashes through Object, so it
can be used as a HashMap key with explicit Object callbacks. ToString returns the
unchanged text. Comparison does not normalize Unicode or apply culture rules.

The current runtime wraps shared immutable UTF-8 text when converting it to Object.
String is a reference type: ReferenceEquals compares the shared text owner, even
through separate Object/interface wrappers. An alias is identical; a separately
constructed equal string has equal contents but a different identity. Explicit String.Intern shares text within one execution; literals are not
automatically interned. Virtual GetHashCode remains content-based; explicit Object base hashing
uses identity.
The current UTF-8 content hash also differs from .NET's randomized hash and is not
a persisted identifier or a defense against deliberate collisions.

See the [String API reference](xref:System.String) and
[Object contract](../../docs/objects.html#string-through-object-development).


Development String copies share immutable text internally. Owned text producers
transfer their buffer; Object conversions still allocate wrappers. Internal checks
cover owner retention across conversions, storage and GC, including host results
and fault teardown. Reference comparison and identity hashing follow that retained text owner.

Development APIs include explicit ordinal comparers and full Unicode casing, described
below. Culture-sensitive comparison and language-specific casing remain possible
future directions; equality, hashing and display transformations have separate roles.

<a id="constructing-and-indexing-text-development"></a>

## Constructing and indexing text

```raven
let text = String(['F', 'o', 'o'])
let characters: Sequence<char> = text
Console.WriteLine(text)
```

With `System.*` and `System.Collections.*` imported, arrays and String can supply
`Sequence<char>`. Construction copies the characters into immutable text. String has
public Length and a read-only indexer (`text[1]` is `'o'`); Count is available only
through Collection/Sequence. Indexes refer to graphemes, unlike .NET's UTF-16 code
units. Length and indexing currently scan the text.

The input must stay stable during construction. No normalization is performed, but
adjacent characters can merge into a grapheme: output Length can differ from input
Count. Empty input currently requires a typed empty char array. See the
[String API](xref:System.String) and [Sequence contract](xref:System.Collections.Sequence`1).


```raven
let text = String(['F', 'o', 'o'])
let alias = text
let separate = String(['F', 'o', 'o'])
// Object.ReferenceEquals(text, alias) is true.
// Object.ReferenceEquals(text, separate) is false; text == separate is true.
```

These development semantics are checked across Object/Sequence conversions, array
storage and garbage collection. Identity hashes may collide and are not persistent IDs.


<a id="explicit-interning-development"></a>

## Explicit interning

```raven
let first = String(['F', 'o', 'o'])
let second = String(['F', 'o', 'o'])
let canonical = String.Intern(first)
let repeated = String.Intern(second)
// Object.ReferenceEquals(canonical, repeated) is true.
// first and second still have their original, distinct references.
```

String.Intern returns a canonical reference for exact text within the current execution.
Use its return value: existing references are not rewritten. Matching does not normalize
Unicode or fold case, and literals are not automatically interned.

The pool retains text until execution completes, faults or is cancelled. Independent
host invocations and isolated workers have separate pools; host-held results remain
valid after execution ends. This differs from .NET's longer-lived interning pool.

Hosts can limit the number of distinct entries and unique UTF-8 payload bytes. Defaults
are 4096 entries and 1 MiB; exceeding either budget for a new entry raises
InternPoolLimitExceeded. Existing entries remain available when full. Table/owner
overhead and temporary inputs are not included in those payload counts. See the
[String API reference](xref:System.String) and [Fault/limit reference](../../docs/faults.html).


## Explicit comparison policies (development)

`StringComparer.Ordinal` combines exact equality, String's Object content hash and
native UTF-8 ordering in one reusable policy. It accepts non-null strings and does
not normalize text or ignore case. Use it with the development HashMap constructor
or through `EqualityComparer<string>` and `Comparer<string>`.
See [collections and comparer policies](../collections/#comparer-policies-development)
for a tested example and [StringComparer](xref:System.StringComparer) for signatures.
This addition is not included in Preview 10.


`String.Compare(left, right, StringComparison.Ordinal)` selects exact UTF-8 ordering.
Use `StringComparison.OrdinalIgnoreCase` or `String.CompareOrdinalIgnoreCase(left,
right)` for Unicode 17 default simple folding. `StringComparer.OrdinalIgnoreCase`
uses the same rules for comparison, equality and hashing, including HashMap keys.

The tested comparison sample compares `"ReadMe"` with `"README"` and uses the policy
for duplicate detection and lookup. Simple folding equates `ẞ/ß` and `K/k`, but not
`ß/ss`; it does not normalize text or apply language-specific rules. These results
and folded ordering can differ from .NET's OrdinalIgnoreCase. An unknown mode faults.
See [StringComparison](xref:System.StringComparison) and [StringComparer](xref:System.StringComparer).
Use matching development runtime and SDK artifacts; Preview 10 lacks these additions.

The [text foundation review](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/text-abstraction.md#systemtext-foundation-review--2026-09-27)
uses Swift as the closer model for character-facing text APIs, with explicit scalar
and encoding views. The immediate direction is encoding/decoding foundations and
possibly a small builder for later APIs, not System.Text parity. Boundary types
remain experimental; these plans do not change current equality or indexing.

## Encoding foundations (development)

[Encoding](xref:System.Text.Encoding) converts valid text into bytes and creates
independent [Decoder](xref:System.Text.Decoder) instances. [Encodings](xref:System.Text.Encodings)
provides UTF-8 and strict ASCII. ASCII rejects unrepresentable text and bytes above
127; it never silently substitutes characters. StreamReader/StreamWriter accept
these policies while keeping UTF-8 defaults. See [encoding contracts](../../docs/streams.html#selected-encodings-development)
for ownership, bounds and errors. [Encoder](xref:System.Text.Encoder) now accepts valid text and drains bounded byte
output, with explicit progress and finalization. A public builder and broader codecs
remain possible next steps; these additions are not part of Preview 10.

## Unicode casing and decimal reports (development)

`String.ToUpperInvariant()` and `ToLowerInvariant()` use Unicode 17 full default
casing. `"Straße ﬃ".ToUpperInvariant()` produces `"STRASSE FFI"`, and
`"ΟΣ".ToLowerInvariant()` produces `"ος"`. They return new text values, preserve
existing references and can change byte, scalar and grapheme counts. No culture
selection or normalization is applied. These rules differ from .NET invariant
casing; comparison and hashing still use their separately selected policies.

`Int64.Parse(text)` returns a typed Result for signed ASCII decimal input.
`Int64.ToString()` produces round-trippable decimal text; static read-only
`MinValue` and `MaxValue` properties expose the bounds. Parsing rejects whitespace
and distinguishes malformed input from overflow. These operations support reports
and counters beyond Int32 without adding culture or format-provider APIs.
See the [API contract](/docs/text-numbers.html) and the tested report consumer in
`docs/experiments/casing-integer/Main.rvn`. Matching development runtime and library
artifacts are required.

Development primitive `Parse` methods use strict whole-text grammars: decimal
integers, decimal/exponent Single and Double, and case-insensitive Boolean words.
They return typed errors, reject whitespace and grouping, and have no ambient
culture dependency. Number provides a separate arithmetic contract; parsing is not
part of it. See [numeric API contracts](/docs/text-numbers.html) for exact rules and the
current generic-import limits. A shared numeric parsing interface remains a possible
future direction.
