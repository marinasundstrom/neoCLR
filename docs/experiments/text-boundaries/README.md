# Text boundary experiment — 2026-09-27

Application-only evidence for the [text foundation review](../../design/text-abstraction.md#foundation-scope-and-swift-direction--author-clarification).
These names/signatures are not proposed as a package to ship. The immediate scope
is basic encoding/decoding and possibly text construction, with Swift as the closer
text API model. No System API, compiler, metadata format or runtime was changed.

## Findings

- A validated scalar can be an ordinary Raven value. NUL is a valid default;
  surrogates and values above U+10FFFF are rejected. This does not replace grapheme
  Char, and neither an encoding API nor a builder requires publishing this type.
- A source-bound range can select the combining mark inside `é`. Exact byte search
  over valid UTF-8 supplies scalar boundaries, not necessarily grapheme boundaries.
  The wrapper owns an immutable String; ranges retain that wrapper. A different
  wrapper, even for equal text, is rejected. This explicit UTF-8 projection is not
  a candidate spelling for an opaque, character-facing position API.
- Public range construction deliberately permits hostile fixtures; Slice rechecks
  source, bounds and scalar boundaries. Null/default ranges are rejected. Production
  factories and views need their own usability/retention decision. Small ranges can
  retain large sources. Search currently copies encoded bytes and scans naively.
- Bounded decoding returns text, input bytes consumed, scalar count and status.
  Output limits count UTF-8 bytes, never grapheme count. Returned String chunks can
  end inside a future grapheme: concatenation of `e` and a combining mark produces
  one Char. There is no grapheme-stream or partial-character guarantee.
- Carry owns up to three incomplete bytes after successful non-final input. Caller
  buffer mutation cannot alter it. Output exhaustion preserves carry and leaves
  input for retry. A complete scalar requires enough output budget; callers can
  increase it to four bytes to guarantee capacity for any valid UTF-8 scalar.
- Final input is honored when all supplied input has been consumed. After OutputFull,
  resubmit the unconsumed suffix with final=true. Empty non-final input is not EOF.
  Successful finalization and malformed/truncated data stop the decoder.
- Failures preserve the current call's valid prefix and consumed count. Error offsets
  are absolute stream byte offsets at the offending scalar's start. Invalid leads
  and bad continuations are unconsumed; completed invalid scalar sequences have
  already been consumed. A malformed lead can be diagnosed even with zero capacity;
  otherwise output exhaustion may defer validation until a retry with more room.
- Invalid ranges and experiment limits reject before mutating state. Limits are
  256 offered input bytes/call, 256 output bytes/call and 4096 accepted stream bytes.
  Aggregate preflight checks the offered count, even if output exhaustion would
  consume less. These are fixture limits, not recommended platform defaults.

Compared with .NET [Decoder.Convert](https://learn.microsoft.com/en-us/dotnet/api/system.text.decoder.convert?view=net-10.0),
the prototype preserves explicit progress/finalization roles but produces immutable
text rather than filling UTF-16 char buffers. Unlike the earlier
[chunk experiment](../utf8-chunks/README.md), it preserves failure progress and bounds
output. Repeated scalar decoding/concatenation is deliberately simple; this is no
performance claim or production codec implementation. A later builder/codec consumer
should select allocation and error timing policies before exposing public APIs.

## Focused validation

`results.json` records three passing executions on macOS arm64 with Raven
`a108df82a` and the existing development bridge/reference/library. One checks scalar
bounds/defaults, combining-mark and delimiter ranges, source identity, forged/null
ranges, carry ownership, retryable range/limit errors, output exhaustion, final retry,
valid failure prefixes, absolute errors and strict overlong/surrogate/out-of-range
UTF-8 rejection. Two cover all 14 split points of `Aé€😀é` with four-byte output
budgets: 13 bytes, six scalars, five graphemes. Separate runs respect the default
runtime instruction budget. No full suite, website build or performance test ran.

The probe uses standard Raven unions. Integer-only payload unions were rejected by
the existing application bridge's explicit-layout guard. `StreamBytePosition` is a
reference payload workaround for this experiment, not a reason to prescribe heap
positions in the public API. Status assertions use case matches. A compound negated
`is` assignment did not terminate the first fixture loop; the final fixture uses
explicit match control flow. This observation is not an isolated compiler diagnosis.
A nested match-expression statement also left a Void stack value during import;
using an explicit statement block resolved it. Neither limitation was fixed here.

Run with mutually matching development artifacts:

```sh
python3 docs/experiments/text-boundaries/verify.py \
  --runtime /path/to/neoclr \
  --bridge /path/to/Probe.dll \
  --system /path/to/System.neoil \
  --reference /path/to/NeoCLR.CoreProbe.dll
```

The bridge's adjacent dependencies must be present. `--evidence /path/to/results.json`
optionally records passing output and artifact hashes. Compilation/verification runs
once; execution covers only this fixture. Existing Sequence, equality/hash and Char
contracts need no migration. Scalar traversal, opaque positions, builder mutation,
foreign UTF-16 policy and Unicode-version alignment remain separate work; this probe
does not claim to settle those interfaces.

## API sketch consumer — 2026-09-27

`Consumer.rvn` adds an application-only accumulator and exact delimiter extraction.
It checks immutable Build snapshots, Clear/reuse, grapheme resegmentation on append,
missing versus empty suffixes, exact matching inside a grapheme, and incremental
UTF-8 output composed into the same text. The paired String/Text vocabulary is in
[the design sketch](../../design/text-abstraction.md#consumer-api-sketch-identical-behavior-two-vocabularies).
Text is not introduced as an alias or wrapper. The accumulator uses concatenation
and makes no efficiency claim.

Use the command above with `--consumer-only` to compile/verify the combined fixture
and execute only the new consumer. `consumer-results.json` records this focused run;
the original `results.json` remains evidence for unchanged boundary checks. Without
that option, the verifier also executes the three original runs. No website build,
full suite, new public API snapshot or performance benchmark is needed for this
application-only semantic sketch.

## Internal StreamReader integration — development

`Reader.rvn` exercises the actual library StreamReader.ReadToEnd, not the sample's
BoundedUtf8Decoder. `--reader-only --runner /path/to/measure_async` runs reader
contracts plus the 256-byte carry/output boundary and 65536-byte maximum. Build the
matching host with `cargo build --release --example measure_async`; its larger
instruction budget accommodates these managed fixtures. The contract run still
uses the normal CLI limits. `--baseline-system /path/to/prior/System.neoil` optionally
runs the identical boundary and maximum fixtures against the prior implementation.
Use matching core/bridge artifacts and the regenerated current System library.

The library decoder is private, with one synchronous consumer and caller-validated
ranges. It carries incomplete bytes, reports current-call consumption and output
exhaustion, and validates complete groups through existing Utf8.Decode. Unlike the
rich experimental decoder, it does not expose public statuses, prefix recovery or
error offsets. These are not silently claimed as production capabilities. Returning
an existing Result and private progress properties avoids widening bridge support
or publishing provisional types merely to satisfy private dependency restrictions.

Contract coverage includes one-to-four-byte partial reads, supplementary and combining
text, BOM preservation, strict malformed/truncated data, early rejection without a
subsequent read, exact/invalid/overflow limits, repeated EOF, stream error propagation,
close ownership and ReadLine followed by ReadToEnd. A 514-byte fixture puts a four-byte
scalar across a 256-byte read boundary and forces an output-budget retry. The maximum
fixture checks the existing 65536-byte accepted limit with 256-byte input chunks.

`reader-results.json` records artifacts and focused outcomes. Host timing and object
counts are a bounded regression diagnostic, not a throughput benchmark or a claim
of lower total memory. The reader still concatenates accumulated output; short reads
can cause repeated copying, and a future construction primitive should address that
when justified. UTF-8 carry is bounded, but output memory is proportional to returned
text. No malformed-prefix recovery or async/Encoder abstraction is added.

Recorded reader results (macOS arm64, single diagnostic run): the 514-byte case took
494 ms in the current host versus 286 ms with the prior library, with 40 versus 32
managed objects allocated. This is an observed cost of the new path, not evidence
of a speedup or a stable benchmark ratio. The 65536-byte synthetic stream completed
in 20322 ms; the prior implementation faulted while growing ArrayList under the same
array budget and therefore has no comparable completion time. The current run
reclaimed all 778 allocated objects (peak live 64). Host instruction allowance was
500000000 for these two larger fixtures; heap allowance was 100000, and other limits
were unchanged. Guest fixture work is included in timings. Faster construction and
codec traversal remain concrete follow-up questions, not reasons to claim that this
slice reduces total allocations. The old-library maximum fault is explicitly allowed
and recorded by the diagnostic runner; it is not reported as a successful old read.

## Shared encoding selection — application contract probe

`Selection.rvn` and `SelectionMain.rvn` test a codec interface, separate decoder
instances and reader/writer selection. UTF-8 is the default; strict ASCII is the
second implementation selected by the author. This is not an enum switch, a second
text representation or a public System API. Two runs pass in `selection-results.json`.
Use `verify_selection.py` with the same runtime/bridge/system/reference arguments;
it compiles only these two sources to stay within the application's method budget.

The [design discussion](../../design/text-abstraction.md#shared-encoding-selection-probe-utf-8-and-strict-ascii)
records all-input acceptance on successful decoding, owned carry, strict errors,
byte counts, fixture bounds, comparison with .NET, integration gaps and future HTTP
reuse. The writer prevalidates ASCII and snapshots bytes before output; no partial
write occurs for unrepresentable text. The prototype owns its stream and has no
ReadLine or leaveOpen overload yet. A stateful Encoder, public reference projection
and actual production stream constructor integration remain outstanding. Tests do
not claim to establish performance or arbitrary expanding-codec quota semantics.

## Public encoding integration

`EncodingMain.rvn` exercises the actual System.Text encoding interfaces and selected
stream constructors. Run `verify_encoding.py` with matching `--runtime`, `--bridge`,
`--system` and `--reference` paths. It covers independent carry ownership, strict
UTF-8/ASCII, lifecycle, line reading, partial writes, byte counts and leaveOpen.
`verify.py --reader-only` retains the existing chunk/boundary/maximum reader checks.
The Selection prototype remains application-only; the public test does not compile it.

## Bounded report construction evaluation

`Builder.rvn` and `BuilderMain.rvn` evaluate an application-only append accumulator
against ordinary concatenation. They do not add a public StringBuilder/TextBuilder.
The fixture exercises a small report, a hard UTF-8 byte quota, atomic expected
limit failures, immutable snapshots, clear/reuse, combining boundaries and output
from the public Decoder. An empty append consumes neither bytes nor a fragment.
Build combines adjacent fragments in balanced rounds and compacts retained state.

Run `verify_builder.py` with the same four artifact arguments as `verify_encoding.py`,
plus `--runner target/release/examples/measure_async` and `--evidence OUTPUT.json`.
It runs the focused contract and invalid-limit checks, then three fresh invocations
of each construction strategy at 8 and 1024 pieces of 16 bytes. Order alternates;
every final byte is checked. Timings include guest construction, checks and captured
output, but exclude assembly/loading/verification. GC counters omit native String
payload allocations; zero surviving managed objects is not a native-byte measurement.

[Recorded evidence](builder-validation.json): at 1024 pieces (16 KiB), concatenation
runs in 51–58 ms and the balanced managed accumulator in 561–574 ms. Eight-piece
runs are 0–2 ms versus 5–6 ms; the timer resolution limits small-case interpretation.
The explicit quota/snapshot contract works, but these measurements do not support
promoting this implementation as the public builder. Do not turn this result into
an optimization project without a concrete consumer requiring bulk construction.
See the [design decision](../../design/text-abstraction.md#bounded-report-construction-evaluation--2026-09-27).

`BuilderBaseline.cs` is a semantic .NET comparison, run in a temporary net11.0 console
project. It confirms immutable snapshots and reuse while distinguishing 21 UTF-16
units, 22 UTF-8 bytes and 20 graphemes for the same report. It is not a cross-runtime
performance comparison; its exact tested version is in the evidence.

## Encoder acceptance/drain evaluation

`Encoder.rvn` and `EncoderMain.rvn` test whole-text acceptance followed by bounded
byte draining. UTF-8/strict ASCII keep at most 32 pending encoded bytes while retaining the
accepted String; the experimental source/request ceiling is 4096 bytes. This is not
a public Encoder API and does not make arbitrary Encoding.Encode implementations
incrementally composable. A synthetic trailer provider tests final output only.

Run `verify_encoder.py` with `--runtime`, `--bridge`, `--system`, `--reference` and
optional `--evidence`. Three separate focused invocations cover acceptance/progress,
scalar boundaries/limits/final output, and a short-write OutputStream consumer.
[Evidence](encoder-validation.json) records matching artifact and source hashes.

The writer distinguishes Finish (conversion completion), Flush (stream flush) and
Close (ownership). Failed output is not automatically retried by a later Write;
caller input may already be accepted and output partial. Finish and Flush errors
must be observed before Close. One-byte drain fragments may split a UTF-8 scalar.
See the [contract and integration recommendation](../../design/text-abstraction.md#encoder-progress-and-writer-evaluation--2026-09-27).

`EncoderBaseline.cs`, run as a temporary net11.0 console project, compares .NET
Convert's UTF-16 input counts and scalar-sized output capacity with the candidate's
whole-text acceptance and byte drains. Its tested runtime is recorded in the evidence.
No performance comparison or public String/Char contract change is implied.

## Public Encoder integration

`PublicEncoderMain.rvn` uses the production Encoding.CreateEncoder, Encoder,
EncoderProgress/EncoderState and StreamWriter.Finish APIs. Its custom trailer codec
is a synthetic contract test, not a supported encoding. `verify_public_encoder.py`
uses the four artifact paths plus `--runner target/release/examples/measure_async`.
It runs progress, boundary, writer and maximum-bound checks; the boundary/65536-byte
fixtures use the established host limits of 100000 heap objects and 500000000
instructions. Runtime defaults are unchanged. `verify_encoding.py` covers existing
selection/line behavior and a migrated custom Encoding implementation.

[Public integration evidence](public-encoder-validation.json) is separate from the
earlier application-only Encoder probe. No full suite or website build is needed.
