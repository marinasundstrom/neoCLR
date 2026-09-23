# UTF-8 chunks: a text consumer for Byte Copy

**Development experiment, 2026-09-23 · Partial M1/S2 evidence.** This saved Raven
project reuses the [Byte Copy](../byte-copy/README.md) array-range validation and
short-read memory input. It runs on neoCLR without changing its runtime, reference
library, Char semantics or public text API.

## The useful case

[Main.rvn](Main.rvn) reconstructs `Aé€😀é` from two-byte reads. Its 13 UTF-8 bytes
encode six scalars and five graphemes. Some reads split multi-byte scalars; another
boundary separates the final e from its combining accent. Decoding a read as if it
were a complete byte string would fail on otherwise valid input.

[Utf8Chunks.rvn](Utf8Chunks.rvn) carries incomplete scalar bytes between calls.
It uses the existing strict `Utf8.Decode` to validate each completed scalar and
returns a text fragment. `ReadText` combines fragments and finalizes at EOF. Only
the combined String is counted in graphemes: neither byte counts nor fragment
character counts can be added to obtain the final grapheme count.

[Expected output](expected.txt):

```text
Text reconstructed from short byte reads:
Aé€😀é
UTF-8 bytes:
13
Graphemes:
5
```

## What the primitive contract needs

- **State across calls:** a successful non-final call retains zero to three bytes.
  They are copied into decoder-owned managed storage; reusing or mutating the input
  array afterward cannot change the pending scalar. No input-array reference or
  native pointer is retained by the decoder.
- **Explicit finalization:** an empty non-final chunk means “nothing available in
  this call,” not EOF. Finalization rejects an incomplete scalar. The MemoryInput
  adapter calls finalization only after a nonempty read request returns zero.
- **Strict validation:** reject invalid leading/continuation bytes, overlong forms,
  surrogates and scalars above U+10FFFF. There is no replacement-character fallback.
  Invalid prefixes may be detected when the scalar completes or when finalization
  reveals truncation; exact error timing is not a selected contract.
- **State after failure:** invalid range or oversized chunk is rejected before
  changing state. Invalid encoding, truncated final input and successful finalization
  stop the decoder; later calls return an error. Create a new decoder for a new input.
- **Output ownership:** returned String fragments are ordinary immutable values.
  Previously returned fragments are not revoked if a later call fails. A failing
  call returns no fragment, even if a valid prefix was processed during that call;
  this is not a transactional whole-stream decoder or a consumed-byte reporting API.
- **Resource scope:** Decode accepts at most 256 new bytes per call. The illustrative
  ReadText adapter caps total input at 4,096 bytes; runtime instruction/allocation
  budgets still apply and can terminate execution earlier. These are experiment
  limits, not proposed platform defaults or performance guarantees.

String-valued errors, method names, finalization flags and the concrete memory input
are provisional. This is not the final Encoding hierarchy or a standard TextReader.
There is no async work, disposal, reset operation, normalization or BOM stripping.
A UTF-8 BOM is retained as U+FEFF, matching the explicit decoder comparison.

## .NET comparison and alternatives

Sources checked 2026-09-23:

- [.NET 10 Decoder](https://learn.microsoft.com/en-us/dotnet/api/system.text.decoder?view=net-10.0)
  preserves state across byte blocks and has explicit final flushing. We reuse that
  functional distinction, but return immutable text/Result rather than filling a
  UTF-16 Char buffer with exception-based fallback behavior. The executable baseline
  uses `new UTF8Encoding(false, true).GetDecoder()` for strict error handling.
- [RFC 3629](https://www.rfc-editor.org/rfc/rfc3629.html), §§3–4, provides the UTF-8
  scalar encoding and valid ranges. The experiment keeps strict scalar validation
  in the existing codec rather than adding another native decoder.
- Reuse [the current text design](../../design/text-abstraction.md) and its Swift,
  Rust and Unicode comparisons for the distinction between bytes, scalars and graphemes.

| Alternative | Benefit | Cost / reason for this experiment |
| --- | --- | --- |
| Decode each read independently | No persistent state | Wrong when a valid scalar crosses a read boundary |
| Buffer the complete input then decode once | Reuses the codec with less per-scalar overhead | Delays all output and requires input-sized storage; still a useful choice for small JSON bodies |
| Retain only an incomplete scalar (this fixture) | Bounded carry; demonstrates progressive decoding and reusable input buffers | Allocates a small list per scalar and repeatedly concatenates Strings; not an efficient production implementation |
| General incremental native codec / Encoding API | Can report consumed/written ranges and reduce copies | Larger state, output-capacity and error contracts; not required to learn from this small consumer |

The evidence supports a library-level stateful adapter as a prototype. It does not
justify a new runtime instruction or freeze its public API. A production design must
address output buffering, error offsets, limit behavior and efficient allocation.
No throughput or allocation-speed improvement is claimed.

## Reproduce and validation

```sh
python3 docs/experiments/utf8-chunks/verify.py --toolchain-root /absolute/path/to/development-bundle
```

The verifier needs the matching Raven/neoCLR bundle and .NET SDK **10.0.100** for the
reference program. It pins that SDK in a temporary `global.json` and builds the
reference for `net10.0`; no package dependencies are added. Raven still uses the
matching bundle's own compiler. Temporary projects leave generated files outside
source control, and cases are batched within the normal runtime instruction budget.

The validation corpus contains 41 cases: every split of the teaching text, one-byte
chunks, empty input, NUL, valid scalar boundaries, BOM, malformed and truncated
encodings. [Reference.cs](Reference.cs) checks the same corpus with strict .NET
stateful decoding. The verifier also checks Python's strict UTF-8 result and then
asserts the Raven result matches. It compares final acceptance and decoded text,
not error classification/timing, .NET UTF-16 lengths or grapheme fragment boundaries.

[Checks.rvn](Checks.rvn) separately exercises empty intermediate chunks, bad ranges
that preserve state, caller mutation after a partial scalar, finalization/reuse,
terminal decode failure and pending managed carry across allocation pressure. The
runner requires a nonzero guest collection count for that case. This is synchronous
GC evidence, not proof of suspended native I/O rooting. The sample itself exercises
repeated short reads into the same scratch array and EOF finalization.

For a standalone build/run:

```sh
export NeoCLRRoot=/absolute/path/to/development-bundle
export RavenSdkRoot="$NeoCLRRoot/raven-sdk"
dotnet msbuild docs/experiments/utf8-chunks/Utf8Chunks.rvnproj -nologo
"$NeoCLRRoot/bin/neoclr" run docs/experiments/utf8-chunks/bin/neoclr/Debug/App.neoil --system "$NeoCLRRoot/lib/System.neoil"
```

## Next questions

Keep the whole-buffer and incremental alternatives available for the small JSON
consumer. Determine whether its first useful case needs progressive text at all.
If it does, output buffering and error location are more urgent than new character
types. The byte-copy source contracts need no change for this consumer; their
managed arrays and checked ranges were adequate. Public streams and encoding APIs,
nonblocking I/O and delayed-operation GC checks remain separate work.
