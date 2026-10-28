# Gzip stream foundations (development, 2026-10-12)

The author requested foundational `System.IO.Compression` APIs and identified HTTP
as a likely consumer. `GZipStream` composes the current async InputStream and
OutputStream contracts. It lives in System.Runtime and requires no native compression
service, compiler option, external library or new metadata encoding. This is an
author-directed library slice, not a permanent roadmap reprioritization.

## Contract and implementation

`CreateCompressor(output[, leaveOpen])` creates an output wrapper.
`CreateDecompressor(input[, leaveOpen])` creates an input wrapper. Both default to
owning the underlying stream. This capability-based factory shape avoids ambiguous
constructor overloads for streams implementing both interfaces. It adapts .NET's
GZipStream to neoCLR's existing split stream contracts rather than introducing a
second stream hierarchy. CompressionMode and CompressionLevel are not implemented.

Read, Write and Flush use the existing Task/Result contracts, with cancellation-token
and default-token overloads. Wrong-direction operations return WrongAccess. Invalid
ranges do not mutate state. Positive reads return zero only after validated EOF;
zero reads consume nothing. Sequential operations are required. An overlapping call
is rejected with IoFailure; this is not thread synchronization.

Compression emits the standard gzip header (MTIME zero, no optional fields, OS 255),
fixed-Huffman DEFLATE with a single-candidate hash-based LZ77 matcher, and a CRC32/ISIZE
trailer. Each Write is divided into at most 16384-byte input blocks; the compression
history resets per block. Empty stored blocks align completed writes. There is no
whole-stream input bound on compression; ISIZE wraps modulo 2^32 as required by gzip.
Caller input remains borrowed until the Task settles. The simple encoder can produce
larger output than zlib and can expand incompressible input; no speed or compression
ratio advantage is claimed. Dynamic-Huffman encoding and compression-level tuning
remain gaps.

Await `Finish()` to emit a final block and trailer, even for an empty stream.
Successful repeated Finish does nothing. Flush forwards to the destination and does
not finalize. Close is idempotent disposal, matching the existing StreamWriter
convention: it never silently starts asynchronous finalization. Await Finish and,
when needed, Flush before Close. `leaveOpen` preserves the underlying stream.
An output error or cancellation after work begins poisons the wrapper to prevent
replaying partially delivered bytes; subsequent operations return IoFailure.
Pre-cancellation cancels the Task without mutation, including before state/range
validation. CPU compression/decompression is inline; there is no worker scheduling
or cancellation polling inside the codec.

The decoder accepts stored, fixed and dynamic blocks, optional extra/name/comment
fields, header CRC16, and concatenated members. It checks reserved flags, Huffman
shape, distances, stored lengths, truncation, CRC32 and member size. Trailing garbage
is rejected. It buffers to underlying EOF, with limits of 65536 compressed bytes and
65536 decoded bytes in total across members. Overflow returns LimitExceeded before
caller output is touched. Malformed data returns the new StreamError.InvalidData;
exhaustive StreamError matches must account for this additive case. Underlying stream
errors are preserved on the first failure. Input already consumed is not rewound.
No decoded bytes are exposed until the complete input has passed validation.

The bounded decoder is an initial support gap relative to .NET streaming decompression,
not a permanent platform rule. It cannot be used directly on an unbounded HTTP socket;
a future HTTP adapter must supply the framed body as an EOF-delimited InputStream.

## .NET baseline, alternatives and tradeoffs

Primary sources retrieved 2026-10-12:

- [.NET 10 GZipStream](https://learn.microsoft.com/en-us/dotnet/api/system.io.compression.gzipstream?view=net-10.0)
  supplies stream compression/decompression, mode/level constructors and leaveOpen.
  Its library policy handles format and lifetime; no gzip-specific CLR instruction
  is required. neoCLR keeps the namespace, wrapper role, wire format and ownership
  concept. Its async naming, typed errors and explicit Finish follow existing
  platform stream contracts; factory construction and bounded input are deviations.
- [RFC 1951](https://www.rfc-editor.org/rfc/rfc1951.html) sections 3.2.3–3.2.7 and
  [RFC 1952](https://www.rfc-editor.org/rfc/rfc1952.html) sections 2.2–2.3 specify
  DEFLATE coding, member headers, CRC32 and modulo input size. These are wire-format
  requirements, independent of Raven syntax, CLI metadata or runtime implementation.
- [.NET's partial-read compatibility change](https://learn.microsoft.com/en-us/dotnet/core/compatibility/core-libraries/6.0/partial-byte-reads-in-streams)
  records the shipped change from filling a requested buffer to normal short-read
  behavior. The scenario matters for network latency; the initial neoCLR decoder's
  read-to-EOF buffering does not achieve that streaming behavior. Incremental decoding
  is required before broad HTTP response streaming support can be claimed.
- [Go compress/gzip](https://pkg.go.dev/compress/gzip) separates Reader and Writer and
  distinguishes Flush from finalizing Close; reader checksum validation requires
  consuming the payload. This supports explicit direction and completion contracts,
  but neoCLR's existing synchronous Close cannot report asynchronous trailer errors.
- [SharpZipLib GZipOutputStream](https://icsharpcode.github.io/SharpZipLib/api/ICSharpCode.SharpZipLib.GZip.GZipOutputStream.html)
  exposes Finish/FinishAsync alongside stream ownership. It demonstrates a library
  solution; it is not evidence that neoCLR needs runtime compression primitives.

Using a native zlib provider would offer a mature codec and tuning at the cost of
provider ABI, packaging and interpreter/JIT/AOT integration. The managed implementation
keeps this foundation portable through existing arrays, integers and stream dispatch,
but adds codec maintenance and allocation costs. A stored-block-only writer would be
smaller but would never reduce HTTP payload size. The selected fixed-Huffman matcher
provides actual compression without promising zlib parity. Buffering decoded content
permits checksum validation before exposing output and bounds expansion; its latency,
memory copies and 64 KiB ceiling are explicit costs. Incremental decoding with
configurable consumer quotas remains the next substantive codec improvement.

## HTTP follow-through

The author said, “We will probably use the Gzip in the Http API.” No automatic HTTP
behavior is enabled by this slice. Future work must define Content-Encoding versus
Transfer-Encoding, Accept-Encoding negotiation, Vary, compressed Content-Length or
chunking, Finish before response completion, decompression quotas, cancellation,
body ownership and policy for already-compressed content. Match .NET HTTP content
composition where applicable and qualify a real client/server consumer. Do not add
Content-Encoding until the body is actually gzip encoded.

## Validation

The focused `gzip` runtime suite uses independent Python/zlib fixtures for stored,
fixed and dynamic blocks, optional fields and concatenated members. It checks
checksums, truncation, expansion bounds, incremental writes, empty streams, ranges,
wrong direction, ownership, pending output and cancellation. The fixture generator
is checked in so the independent data and its provenance remain reproducible.
Execution results and external interoperability evidence are recorded alongside the
suite; other host architectures are not inferred from local validation.

StreamReader preserves binary format failures as `TextReadError.InvalidData`; this
additive text-error case also requires updating exhaustive matches.

Qualification on 2026-10-12: all six contract groups pass in local macOS ARM64 AOT
and JIT. The [source interoperability consumer](experiments/gzip/README.md) emits
79,986 source bytes as a 3,859-byte gzip stream, independently decoded by .NET 10.0.0
and Python/zlib. This is a correctness fixture, not a general compression-ratio or
throughput claim. The selected interpreter pending-input/cancellation/text-composition
contract passes. Full interpreter runs exceed both the initial 180-second and extended
600-second allowances; the complete interpreter suite is **not qualified**. Retain
this gap rather than inferring interpreter coverage from native results. See
[execution evidence](../runtime/System.Runtime/test/GZipTests-validation.json). Aggregate reference
fingerprints, native type/member rendering and both native snapshot unit checks pass.
No full website build or publication was performed.


## Memory-view integration (development, 2026-10-19)

The synchronous encoder and CRC accumulation now borrow ReadOnlySlice<byte>.
The write operation retains a bounded ReadOnlyMemory<byte> across output awaits,
then passes each block to those helpers. The decoder retains a readonly memory
descriptor and borrows slices only within synchronous reads. Produced bytes and
coding tables stay owned. This preserves .NET's distinction between borrowed
synchronous input and memory retained by async operations; no compatibility-only
overloads or conversion helpers are added.

The public GZipStream and InputStream/OutputStream signatures remain unchanged
until their shared service migration. Existing invalid-range precedence, quotas,
CRC/framing and partial-output semantics still apply. Decoder input buffering is
unchanged; this is not a transport zero-copy or performance claim.

[Current validation](validation/memory-views-20261019/gzip-integration-validation.json):
seven contract groups pass in each of AOT/JIT, including a nonzero-offset window
crossing the 16384-byte compression block boundary with source/destination
sentinels. Selected interpreter pending-input/cancellation and block-window tests
also pass. Independent stored/fixed/dynamic/header fixtures remain covered. The
full interpreter suite is not rerun; its earlier timeout remains open. The unchanged
external interoperability sample currently fails native emission with NEOMETA003
(return type mismatch in an async function); its earlier Python/.NET comparison
is historical evidence, not a new successful rerun. The reference assembly is
unchanged after regeneration; its source fingerprints are refreshed. Public API
rendering is unchanged. No website build or publication was performed.
