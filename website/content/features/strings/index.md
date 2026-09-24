# Strings, characters and UTF-8

String represents Unicode text. Char represents one grapheme cluster: a practical approximation of a character as a reader perceives it. UTF-8 is the canonical internal encoding; scalar and byte access are explicit.

**Preview 9 implementation · September 19, 2026.** This page records the current implementation, not a settled design. Names, signatures and behavior may change as we learn from use and feedback. Use the matching Preview 9 toolchain.

[See a working example ↓](#characters) · [Download the complete sample](../../samples/library-utf8.rvn)

<a id="characters"></a>

## Character counting and iteration

`Length` counts grapheme clusters. Iterating a string yields `Char` values, including combining sequences and emoji sequences. There is no integer string indexer in this small API.

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

## Use the matching development toolchain
Open the prepared Raven project in VS Code, replace Main.rvn with the complete sample, save it, then run the neoCLR build/run task. Refresh both the reference core and System library when moving from an older workspace. Existing `IsEmpty()` calls must become `IsEmpty`.

[Download Raven source](../../samples/library-utf8.rvn) · [Download expected output](../../samples/library-utf8.expected.txt) · [Toolchain setup →](../../try/#development)

The downloadable source and expected output are the same files used by the saved-project integration check.

<a id="direction"></a>

## Planned work and open questions

This is an evolving preview API. We are considering more efficient iteration, text positions for slicing, normalization and language-sensitive comparison. A dedicated scalar value type may replace uint in the explicit scalar view. Other encodings would be conversions at the boundary, leaving the meaning of String and Char unchanged. Possible future Utf8String and AsciiString types could offer encoding-specific operations and guarantees alongside the default text container. Those types and an Encoding hierarchy remain deferred.

Unlike .NET’s UTF-16 Char, neoCLR’s Char can contain multiple Unicode scalars. Numeric character casts and fixed-size character memory assumptions must change. Compiler literal diagnostics currently follow the host Unicode rules; the runtime validates the pinned target rules. CLI constant fields are outside the tested character surface.

.NET offers configurable UTF-8 decoding, including a strict mode. This implementation starts with a strict Result and Sequence-based byte access. Whether those are the right long-term contracts is open for feedback.

A separate [UTF-8 chunk experiment](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/utf8-chunks/README.md) explores carrying incomplete scalar bytes between short reads, strict finalization and buffer reuse. It is a runnable application experiment, not a public incremental Encoding API. Graphemes can still span decoded fragments. A [JSON message experiment](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/json-message/README.md) tests quoted-string replies and Unicode escapes over bounded text; its [document follow-up](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/json-document/README.md) reads a sensor report and builds an acknowledgement with explicit fields and number conversions. Both are development experiments, not a published JSON API.

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


## Char through Object (development)

Boxing a Char preserves a copy of its complete grapheme text. Object equality and
hashing use that exact text; Object ToString returns it unchanged. Separate boxes
retain separate identities. This includes combining sequences and joined emoji.
Composed `é` and decomposed `é` remain different keys because comparison does not
normalize text. .NET Char instead represents one UTF-16 code unit; neoCLR's hash
therefore follows its text representation, not .NET's code-unit formula.

Use the [Char API reference](xref:System.Char) for construction, typed equality and
comparison, and the [Object guide](/docs/objects.html) for boxing contracts.


## String through Object (development)

Text now retains exact content equality and matching hashes through Object, so it
can be used as a HashMap key with explicit Object callbacks. ToString returns the
unchanged text. Comparison does not normalize Unicode or apply culture rules.

The current runtime wraps shared immutable UTF-8 text when converting it to Object.
String is a reference type: ReferenceEquals compares the shared text owner, even
through separate Object/interface wrappers. An alias is identical; a separately
constructed equal string has equal contents but a different identity. Interning is
not provided. Virtual GetHashCode remains content-based; explicit Object base hashing
uses identity.
The current UTF-8 content hash also differs from .NET's randomized hash and is not
a persisted identifier or a defense against deliberate collisions.

See the [String API reference](xref:System.String) and
[Object contract](../../docs/objects.html#string-through-object-development).


Development String copies now share immutable text internally. Owned text producers
transfer their buffer; Object conversions still allocate wrappers. Internal checks
cover owner retention across conversions, storage and GC, including host results
and fault teardown. Reference comparison and identity hashing now follow that retained text owner.

Future work includes a coherent System.Text API and comparer infrastructure, especially
string comparers. Equality, hashing and ordering should use compatible, explicit
policies. Ordinal, case-insensitive and culture-aware behavior will need their own
design and validation; these comparers are not available yet.


Planned text work also includes ToUpper/ToLower-style casing and comparison methods.
.NET is a reference, not an API-copy requirement: adapt names and contracts where a
concrete benefit justifies the compatibility cost. Culture selection, Unicode casing
(including length changes), ordering and equality/hash consistency need explicit
choices and tests. Additional casing/comparison APIs remain planned; the existing
bounded ordinal helpers retain their current contract.

## Constructing and indexing text (development)

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
