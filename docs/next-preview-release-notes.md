# Next preview — Preview 17 candidate

This is an unpublished draft. Preview 16 and its assets remain unchanged.
No replacement release version or publication date has been selected.

## Current qualification (2026-10-22)

The initial 2026-10-19 candidate `0.1.17-neoclr.20261019.1`, using Raven
`2d22055dca58dcf991c6001dfe5d93657577d9d6`, was superseded before publication.
Do not reuse that identity or its checks for current source.

The current development SDK/editor version is `0.1.0-dev.20261021`, using Raven
`301e668a51beb6ba307d63b6912b144d8ad6a6b0`. Extracted macOS ARM64 and Windows x64
packages, installed-editor checks, HTTPS checks and refreshed platform benchmarks
have passing evidence. Both archives pass manifest and SDK/editor identity audits.
These development packages are not published release assets. The
[current gate index](validation/release-stabilization-20261022/README.md) records
exact inputs, reports and limits. Canonical full-source qualification is running;
formatting, strict Clippy and Rust 1.85 checks now pass.

## Implemented scope for replacement review

- Slice/ReadOnlySlice and Memory/ReadOnlyMemory integration across supported byte
  buffers, codecs, streams, cryptography and networking; explicit ownership and
  borrowed-versus-persistent lifetimes. See [memory contracts](native-memory-views.md).
- Stream-capable HTTP content, canonical HttpContext callbacks and headers-first
  server dispatch with optional asynchronous body reads. HTTP message version,
  disposal and client response metadata, including lazy exact ReasonPhrase.
- JSON stream/content integration and compiler/runtime/native-backend fixes qualified
  by the [full public consumer](experiments/json-object-mapping/native-public-20261019/README.md).
- Website coverage of memory-view API philosophy and experimental concurrent GC.
  Collector measurements retain their original cohort and workload limitations.

## Migration and limits

Buffer API signatures now prefer views instead of compatibility-only array overloads.
Array slices alias existing storage; use an explicit copy when independent ownership
is needed. Borrowed slices cannot survive asynchronous suspension. Byte-buffer native
qualification does not imply all generic element types are supported.

HTTP handlers now receive requests after headers. Body errors can occur later during
explicit consumption; applications must dispose their context/messages and observe
body-read results. HttpContent is abstract with explicit memory, stream and string
producers. Transport remains HTTP/1.1; unknown-length outgoing framing and incremental
JSON token serialization are not claimed. Experimental concurrent GC is not promoted
to a universal throughput improvement or the production default.

String and StringBuilder have separate source reviews. Swift remains the text-semantic
model. Canonical String/Character equality and matching hashes, Unicode 17 segmentation,
search predicates, trimming/line-ending utilities, Character helpers, builder reuse and
String.Format/Formattable are implemented with focused development qualification.
Original UTF-8 bytes are preserved. Shared-storage String views remain deferred;
general custom formatting patterns and arbitrary providers remain deferred.
[Numeric formatting and culture parsing](experiments/numeric-formats/README.md)
are qualified across interpreter, JIT and native execution for the five bounded
culture profiles. [Decimal](experiments/decimal/README.md) and the
[Money value record](experiments/currency/README.md) add exact base-ten values and
explicit currency identity without a new numeric interface hierarchy.

[Record display and inheritance](experiments/record-formatting/README.md)
include generic component display, root record classes, source-record inheritance,
base copies and finite recursive display, with 45 built-in executions across the
three engines. Arbitrary imported record-base synthesis remains outside that
qualified scope. Generic boxing and display-string allocations remain costs;
these correctness tests do not establish a throughput improvement.

[Dependency injection](experiments/dependency-injection/open-generics.md) and
[localization](experiments/localization/README.md) cover constructor activation,
scopes, keyed/function services, open-generic typed localizers, per-key culture
fallback and lookup-first argument formatting. Their focused built-in evidence
remains distinct from package acceptance.

Console now exposes synchronous byte/text capabilities with configurable line endings,
and scoped ReadKey with logical keys, modifiers and optional Unicode scalar text.
POSIX entry-thread keys, any-key exit and Ctrl+C-only exit pass interpreter/JIT/AOT.
Windows key input, text-to-key switching and native worker attachment remain unsupported.
Focused key-input evidence does not imply that interactive Windows input is
qualified by the general extracted-package tests.

## Qualification remaining

Complete canonical committed-source qualification and resolve any failures. Select
final release identities and verify publication artifacts against the qualified
inputs; rebuilding or changing those inputs requires affected gates to be repeated.
Keep API/site documentation aligned with the selected scope. The current benchmark
reports preserve distributions and provenance, including the Windows dirty-checkout
limitation; they do not establish cross-platform rankings or controlled speedups.
Website publication remains a separate manual operation after validation.
