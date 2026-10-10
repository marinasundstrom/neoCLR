# Encoding API review — 2026-10-10

Status: review of the development API, with provisional recommendations. The author
asked for this review after the time parsing slice. This document does not add APIs
or approve additional codecs. It revisits the
[2026-09-27 text/encoder decisions](text-abstraction.md#public-encoder-integration-development--2026-09-27)
under the author's current .NET-first design rule.

## Assessment

Keep the three roles: encoding provider, stateful encoder and stateful decoder.
The most useful next work is conformance coverage, paired whole-value conversion,
and a completion contract usable through TextWriter. Adding UTF-16 or code-page
providers first would multiply unresolved lifecycle and input-shape questions.

The existing strict UTF-8/ASCII baseline is an explicit author selection. Preserve
it. Interface versus abstract class and Encode versus GetBytes are separate
ergonomic choices; neither follows necessarily from UTF-8 storage.

## Implemented behavior

Source reviewed: [Encoding](../../runtime/raven/src/System/Text/Encoding.rvn),
[Encoder](../../runtime/raven/src/System/Text/Encoder.rvn),
[Decoder](../../runtime/raven/src/System/Text/Decoder.rvn),
[providers](../../runtime/raven/src/System/Text/Encodings.rvn), and
[StreamWriter](../../runtime/raven/src/System/IO/StreamWriter.rvn).

| Area | Current contract | Consequence |
| --- | --- | --- |
| Provider | Encoding interface; Encode returns Result of Sequence<byte>; CreateEncoder/CreateDecoder create independent sessions | No matching one-shot Decode. Encode's output cannot be handed directly to Decoder's byte[] input. |
| Encoder | Accept(string, final), then Drain(byte[], offset, count) with written count and state | Pending input prevents another Accept. Positive output capacity can split a scalar's encoded bytes. |
| Decoder | Decode(byte[], offset, count, final) returns a string | Retains a copied incomplete UTF-8 suffix. No caller-supplied output buffer or consumed count. |
| Errors | Strict validation; range/quota errors retryable; malformed input or completed final input ends the decoder | A failing decode call returns no valid prefix. Earlier successful output remains valid. |
| Limits | Built-ins limit each accepted text chunk to 65536 UTF-8 bytes; decoder input and drain requests to 65536 bytes | These are different units. They are bounded development policies, not Unicode requirements. |
| Writer | Finish emits final bytes; Flush flushes the stream; Close manages ownership | Finish is absent from TextWriter. Generic callers cannot express successful conversion completion. |
| BOM | UTF-8 data is preserved, without implicit stripping or a provider preamble API | Protocol/file BOM policy remains explicit work. |

Utf8.Encode/Decode also provide direct UTF-8 conversion, with a different input/error
surface and no equivalent Encoding quota policy. That duplication needs a clear
convenience-versus-policy explanation.

TextEncoder retains the immutable accepted string and at most 256 pending encoded
bytes. StreamWriter drains through a 256-byte buffer and handles short writes.
Custom encoders may exceed the writer's 65536 encoded-byte ceiling after partial
output. Failed output/draining poisons subsequent writer conversion; ASCII/source
preflight failures are retryable. These details are observable, not implementation
freedom to change silently.

## Comparison and tradeoffs

Primary sources checked 2026-10-10; .NET links select the .NET 10 contract.

- [.NET Encoding](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoding?view=net-10.0)
  is an abstract class with paired byte/string conversions and encoder/decoder
  factories. Default ASCII replacement differs from neoCLR's author-selected strict
  rejection. Keep strictness for loss detection, documenting that ports cannot
  assume replacement. The interface currently permits custom codecs without a base
  class, but makes new required members a migration for every implementation.
  UTF-8 storage does not itself justify different provider organization.
- [.NET Decoder.Convert](https://learn.microsoft.com/en-us/dotnet/api/system.text.decoder.convert?view=net-10.0)
  exposes bounded input/output progress and completion. neoCLR's whole-chunk return
  is simpler for StreamReader but allocates output and cannot report a usable prefix
  on failure. Adopt progress only for a consumer that needs bounded destination
  buffers; do not turn Char/grapheme counts into UTF-16 code-unit counts.
- [.NET StreamWriter.Flush](https://learn.microsoft.com/en-us/dotnet/api/system.io.streamwriter.flush?view=net-10.0)
  flushes encoder state; Close also flushes, while AutoFlush preserves encoder state.
  neoCLR's Flush/Close behavior is therefore a real divergence. Explicit Finish can
  expose Result failures before ownership cleanup, but callers familiar with .NET
  can otherwise omit final bytes. A concrete-only Finish leaves generic writers
  incomplete. Matching .NET requires more than renaming: terminal neoCLR sessions
  cannot simply receive final=true on every reusable Flush.
- [WHATWG Encoding](https://encoding.spec.whatwg.org/#interface-textdecoder)
  separates streaming decode, fatal-error selection and BOM handling. Its browser
  compatibility defaults are not a reason to replace strict ASCII or adopt browser
  labels. The transferable lesson is explicit stream and BOM policy.
- [encoding_rs](https://docs.rs/encoding_rs/0.8.35/encoding_rs/)
  separates stateless and incremental APIs, including buffer-oriented conversion.
  This is evidence that codec algorithms and progress can live in a library.
  Rust ownership/lifetime contracts do not directly supply managed-runtime safety.

The existing [Accept/Drain comparison](text-abstraction.md) and
[public encoder probe](../experiments/text-boundaries/PublicEncoderMain.rvn)
provide concrete experience with tiny output buffers, custom final output and
partial writes. No new independent .NET codec project or API-review issue was
qualified for this bounded review. Broader ecosystem experience remains a research
gap before adopting a new conversion hierarchy; no popularity or performance claims
are made here.

## Recommended sequence

1. **Move existing encoding probes into attributed framework tests.** Cover strict
   UTF-8/ASCII, all split positions of multibyte scalars, malformed/overlong/surrogate
   bytes, final truncation, embedded NUL, BOM preservation, independent sessions,
   zero/one-byte buffers, retryable range/quota errors, terminal states, short writes,
   final trailers and failure after partial output. Run interpreter and native;
   include the Windows gate. Existing 2026-09-27 evidence is not current native
   qualification.
2. **Close the whole-value conversion asymmetry.** Prefer a Decode operation paired
   with Encode, accepting the read-only sequence produced by Encode. Compare a
   byte-array/range overload for stream consumers. Keep naming coherent with Utf8
   unless a tested .NET-style GetBytes/GetString migration earns its cost. Adding
   Decode to an interface breaks custom implementations; update a custom-codec
   fixture, references and docs together. Do not implement it by pretending an
   arbitrary codec is stateless across independently decoded fragments.
3. **Resolve general writer completion before adding stateful codecs.** Prototype
   Result-returning completion on TextWriter against a custom final-trailer codec.
   Compare it with .NET-compatible flushing and encoder reset/reuse. Prefer .NET
   behavior where feasible; if explicit terminal completion remains necessary,
   document why, how generic callers reach it, and how Close handles failure.
   Do not silently discard final bytes or convert a recoverable Result into a fault.
4. **Make limits and error transitions discoverable.** Document input UTF-8 bytes,
   encoded output bytes and per-call versus total limits separately. Evaluate a
   shared options/limit contract only when consumers need configuration. Review
   InvalidUtf8 versus InvalidData and useful failure offsets alongside tests.
5. **Defer optional expansion.** Reset, destination-buffer conversion, BOM/preamble
   policy, UTF-16LE/BE and replacement fallback need separate consumer-driven slices.
   Code-page registration, HTTP charset negotiation and broad fallback hierarchies
   are not implied by this review.

The next bounded implementation is step 1, not a wholesale rewrite. Steps 2–4 are
proposals to validate with both built-in and custom providers.

## Placement and validation

These changes should primarily affect Raven library contracts and implementations.
The bridge projects declarations; native metadata must preserve interfaces, unions
and custom provider dispatch. Neither new compiler syntax nor new native opcodes
are established requirements. Any actual backend gap found during migration should
be fixed and documented in both repositories.

Retain the tested valid-UTF-8 String contract. Treat byte-array input as borrowed
only during a call; any retained decoder suffix must be copied. Encoder sessions
may retain an immutable String reference. No thread-safety or concurrent session
sharing is promised; future concurrency should use independent sessions.

Use the existing [validation report](../experiments/text-boundaries/public-encoder-validation.json)
as historical evidence, then record new attributed-suite hashes and platform scope.
Add small .NET comparison cases when changing completion, replacement or progress
semantics. Update public API references with implementation changes. Benchmark
allocation/copying only after a concrete performance question emerges; this review
makes no throughput claim.

## First migration slice

The new [encoding framework project](../../runtime/raven/tests/encoding/Tests.rvn)
translates the provider/decoder and encoder-progress probes into eight discoverable
tests. It covers all byte split positions of Aé€😀, strict malformed input,
copied pending bytes, independent sessions, preserved BOM/NUL data, retryable
range/preflight errors, terminal states and bounded drains at the internal chunk
boundary. It uses recoverable assertions rather than terminating the whole probe.

This is a partial migration. Custom writer trailers, partial-output failures,
reader integration and 65536-byte quota fixtures remain in the historical probes
until their own attributed batch is qualified. Conversion symmetry and completion
API proposals are unchanged.

All eight cases pass native macOS ARM64 and interpreter execution
([qualification](../../runtime/raven/tests/encoding-validation.json)); no library
implementation changes were needed. Windows qualification remains pending.
