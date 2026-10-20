# Grapheme text reads and peek — 2026-10-20

Status: implemented in the Raven class library and qualified in the interpreter,
JIT and AOT on macOS ARM64. This is text input, not terminal key-event integration.
The separate POSIX host transport is not yet connected to guest async scheduling.

TextReader and StreamReader now provide PeekChar, ReadChar and Read(count) with an
optional explicit result-byte bound. Console forwards the same operations through
an invocation-owned shared Console.In reader. Peek may read underlying bytes but
preserves the logical cursor. Read/ReadChar/ReadLine/ReadToEnd share lookahead.
Closing Console.In closes that text reader for the invocation, without closing stdin.
Do not mix buffered text reads with raw input handles.

A Char is a Unicode 17 grapheme. A combining sequence split across byte reads remains
one Char, with original UTF-8 bytes preserved. The final cluster waits for another
boundary, EOF or a definitive LF boundary. Counted reads fill the requested grapheme
count unless EOF intervenes; zero consumes nothing. Result byte limits are independent
of character count. Repeated peeks reuse the cached Char, and readers retain one byte
buffer. Task/Result/decoder allocations remain; no throughput or zero-allocation claim.

The consumer checks chunk sizes 1, 2, 7 and 256; combining accents, ZWJ emoji and flags;
repeated peek without another source read; CRLF graphemes and empty lines; mixed
character/count/line/whole-text reads; EOF; closed readers; zero/invalid counts; byte
limit retry; a genuinely pending input adapter; Busy on overlapping reads; and mixed
Console convenience calls supplied through stdin.

All three executions pass in [results.json](results.json). The pending test adapter
uses a Promise deliberately to control completion; it is not a proposed per-key
implementation. No compiler or runtime-service change was needed for these APIs.

## Reproduction

Compiler `2d22055dc` (`target/canonical-text-compiler-v1`), libraries
`target/console-text-libraries-v2`, matched bundle `target/console-text-bundle-v1/bundle`,
consumer `target/console-text-app-v3`. Build with the repository's native library,
matched-bundle and native-project scripts, then run:

```sh
python3 docs/experiments/console-text-reading/check.py \
  target/console-text-bundle-v1/bundle target/console-text-app-v3 \
  --output target/console-text-app-v3/results.json
```

Both API snapshots are refreshed with matching source/reference artifacts. Public
contracts, .NET comparison and limitations are in the [Console guide](../../../api-docs/console.md).

The API snapshot check, four native-snapshot tests and the 4596-page website build
pass (124 legacy pages retained). The subsequent author direction calls for optional
synchronous capabilities rather than forcing all providers into Task-returning APIs;
those capabilities are the next slice, not implemented by this text-read commit.

## Updated qualification — separate sync console and string delimiters

The fixture now uses Create factories and direct Console operations. It also
checks genuinely pending async reader/writer overlap, cancellation cleanup and
writer invalidation after an abandoned transfer. CRLF line consumption is configured
explicitly with NewLine. All three engines pass in the current results.json using
 target/line-text-bundle-v2/bundle and target/line-text-async-app-v3. Earlier
reproduction details above describe the historical first text-read slice. Exact
delimiter cases are separately qualified in ../text-line-endings.
