# String from characters (development)

The Raven sample constructs `String(['F', 'o', 'o'])`, copies a mutable char array,
uses String through `Sequence<char>`, and indexes combining text and emoji.
`Length` is public; `Count` is explicitly implemented for Collection and is available
through Sequence. String's indexer is read-only. These are development APIs requiring
matching target metadata, importer and runtime, not the published Preview 9 bundle.

Construction reads Count once, consumes the iterator once and snapshots the characters
into a bounded array before joining their UTF-8 text. The source must remain stable
and report its actual count. A mismatch faults; null input faults. Subsequent source
mutation cannot change the resulting immutable String. No normalization occurs.
Adjacent characters can form a single grapheme, so output Length need not equal input
Count. Length/Count and indexed access scan graphemes; they are not constant-time.
String iteration currently materializes characters, adding another allocation.

Unlike [.NET String(Char[])](https://learn.microsoft.com/en-us/dotnet/api/system.string.-ctor?view=net-10.0),
which copies UTF-16 code units, this API accepts the platform's Sequence abstraction
and grapheme-based chars. The broader input contract costs traversal and temporary
storage. An Iterable-only overload remains undecided: a growable buffer could consume
it without Count, but would need allocation limits and a policy for unbounded inputs.
An explicit count parameter is not selected; its meaning would need to distinguish
an exact count, prefix length and allocation hint.

Empty input currently uses `let empty: char[] = []` followed by `String(empty)`.
The installed Raven compiler crashes emitting `String([])` directly; this existing
empty collection-expression emission limitation is not hidden by the sample.

```sh
python3 docs/experiments/string-sequence/verify.py \
  --toolchain-root /path/to/matching-bundle \
  --runner target/debug/examples/measure_async
cargo test --test string_construction
```

The verifier checks exact output, complete managed-object reclamation and compiler
rejection of direct String.Count, index assignment and non-character construction.
Native tests cover empty/text/NUL copying, grapheme boundaries, index Fault codes,
null construction and invalid explicit interface metadata.

Validation on 2026-09-24: the Raven sample and all three negative compilation cases
pass, with 21 managed allocations and zero retained objects after collection. Four
construction/metadata tests and twelve explicit-interface regressions pass. The
bridge signature probe, generated runtime snapshot check, .NET comparison and
522-page combined website build pass. These are local development checks, not a
published SDK or website deployment.
