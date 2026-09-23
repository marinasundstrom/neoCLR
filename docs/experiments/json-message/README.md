# JSON message: a string round trip

**Development experiment, 2026-09-23 · First, partial M1/S3 checkpoint.**
This small Raven application receives a JSON string message, reads its text and
writes a prefixed reply. It uses existing managed collections, strict UTF-8 and
Result; no runtime or public library APIs change.

## The product and the boundary

[Main.rvn](Main.rvn) receives `"Hello, caf\u00e9 \ud83d\ude00!"` and replies with
`"Received: Hello, café 😀!"`. It then reads the reply again. A top-level string is
a complete JSON value, but this fixture deliberately accepts **only strings**.
It does not represent completion of the roadmap's small JSON document API.

[JsonMessage.rvn](JsonMessage.rvn) exposes two provisional functions:

- `ReadMessage(payload)` returns decoded text or a string-valued error. It requires
  one quoted JSON string with optional JSON whitespace before/after it.
- `WriteMessage(text)` returns a quoted JSON string or a size error. Quotes and
  backslashes are escaped; control bytes use `\u00xx`. Other Unicode is emitted
  directly. This is JSON encoding, not an HTML encoder.

The reader accepts the standard short escapes, four-digit Unicode escapes and
paired UTF-16 surrogate escapes. It rejects raw control bytes, unknown/truncated
escapes, unpaired surrogates and trailing content. JSON whitespace is exactly space,
TAB, CR and LF. It neither strips a BOM nor normalizes text. Decoding whole strings
preserves combining sequences without mistaking grapheme indices for byte offsets.

The input and generated JSON payload are capped at **128 UTF-8 bytes**, including
quotes and surrounding input whitespace. Writer expansion counts toward that limit.
Limits are experiment policy, not platform defaults. Runtime instruction/allocation
budgets remain additional limits. The boundary takes a valid neoCLR String; malformed
wire UTF-8 must first be rejected by the existing codec. There is no partial output
on failure, numeric coercion, reflection, disposal or asynchronous work.

## .NET baseline and the experiment

Sources checked 2026-09-23:

- [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259.html), §§2, 7–9: JSON grammar,
  string escapes and Unicode interoperability. Its grammar can spell unpaired
  surrogate escapes; this experiment deliberately rejects them when materializing
  neoCLR text. This is a restricted interoperable string policy, not a claim that
  every grammar-valid JSON text must be accepted.
- [.NET System.Text.Json DOM](https://learn.microsoft.com/en-us/dotnet/standard/serialization/system-text-json/use-dom):
  explicit document/value access is an alternative to reflection-based serialization.
  [Reference.cs](Reference.cs) uses .NET 10 JsonDocument plus GetString and verifies
  a JsonSerializer string round trip. Reading the string matters: parsing alone
  does not establish successful Unicode materialization.

This is library policy over managed strings/collections, not a CLI metadata or
runtime feature. .NET offers a general DOM, token reader and serializer; neoCLR is
missing those facilities. The experiment does not identify a CLR defect. It returns
Result instead of exceptions and owns decoded text without a JsonDocument lifetime.
The costs are a narrow accepted value kind, allocations and provisional diagnostics.
No performance or allocation improvement is claimed. The writer's escaping need
not match .NET's spelling; comparisons are on the resulting text.

| Choice | Benefit | Cost / current decision |
| --- | --- | --- |
| Buffer then read one string | Small contract, atomic result, no parser suspension | Requires complete bounded payload; selected for this message |
| Incremental UTF-8 adapter before parsing | Handles split wire scalars | Still accumulates the JSON text; adds state without incremental JSON results |
| Incremental JSON tokens | Can process larger bodies progressively | Token carry, escaping, completion and error-location contracts remain open |
| General DOM or serializer first | Supports rich documents and typed objects | More contracts than needed to expose this string-boundary problem |

The [UTF-8 experiment](../utf8-chunks/README.md) remains useful for progressive text
consumers; it is not a prerequisite for parsing a small buffered JSON message.
A wire parser may instead scan bytes directly, but this fixture starts from an
already validated String and uses UTF-8 byte offsets internally. It copies encoded
input/output and has no zero-copy or borrowing contract. More extensive comparisons
with other JSON libraries and platforms belong to selection of a general JSON API.

## Validation and reproduction

```sh
python3 docs/experiments/json-message/verify.py --toolchain-root /absolute/path/to/development-bundle
```

Use a matching Raven/neoCLR development bundle and .NET SDK **10.0.100**. The verifier
pins that SDK in a temporary directory, uses net10.0 without package dependencies,
and builds target cases separately to stay within the ordinary execution budget.
It checks the teaching output and compares acceptance/materialized text for 69 cases
with .NET: literal/escaped Unicode, surrogate boundaries, all 32 raw controls,
whitespace, malformed escapes, wrong value kinds, trailing input and the byte limit.
The reference applies the fixture's size and string-only restrictions explicitly;
they are not .NET limitations. Every accepted target case is written and read back.
Additional writer checks inspect all 32 emitted control escapes and cover expansion to exactly 128 bytes, overflow by escaping
and excessive unescaped input. Error wording/timing and exact escape spelling are
not compared. This is targeted evidence, not exhaustive parser conformance.

For a standalone build/run:

```sh
export NeoCLRRoot=/absolute/path/to/development-bundle
export RavenSdkRoot="$NeoCLRRoot/raven-sdk"
dotnet msbuild docs/experiments/json-message/JsonMessage.rvnproj -nologo
"$NeoCLRRoot/bin/neoclr" run docs/experiments/json-message/bin/neoclr/Debug/App.neoil --system "$NeoCLRRoot/lib/System.neoil"
```

## Following checkpoint

The [JSON document experiment](../json-document/README.md) now extends this consumer
with a sensor report and acknowledgement, explicit fields, all JSON value kinds,
preserved number text and checked conversions. It records provisional duplicate-key,
depth and size policies. Neither fixture settles the public DOM shape, parser offsets,
error types or final API names.
