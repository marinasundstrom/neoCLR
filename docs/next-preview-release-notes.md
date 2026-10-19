# Next preview — Preview 17 candidate

Selected for qualification on 2026-10-19; **not published**. The candidate SDK/editor
version is `0.1.17-neoclr.20261019.1`, using Raven native compiler
`2d22055dca58dcf991c6001dfe5d93657577d9d6`. Publication date remains unset.
[Preview 16](preview-16-release-notes.md) and its assets remain unchanged.

## Selected scope

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

String and StringBuilder have separate source reviews. Swift is the text-semantic
model; canonical equality, typed text positions, missing search/formatting APIs and
builder storage improvements remain proposed work, not features in this candidate.

## Qualification remaining

Require canonical committed-source tests/format/Clippy and host checks, freshly
packaged macOS ARM64 and Windows x64 acceptance, matching editor tests, API/site
validation, and current performance evidence. Record failures and fix them before
publishing. Development bundles are not release assets. Website publication remains
a separate manual operation after validation.
