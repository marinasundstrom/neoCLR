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
