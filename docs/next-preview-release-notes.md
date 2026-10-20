# Next preview — Preview 17 candidate

Initial candidate selected on 2026-10-19; **superseded before publication**.
The author now requires String and StringBuilder fixes before release. The initial
Windows package run was canceled; existing source checks remain baseline evidence.
A new package identity and qualification run are required after text changes. The candidate SDK/editor
version is `0.1.17-neoclr.20261019.1`, using Raven native compiler
`2d22055dca58dcf991c6001dfe5d93657577d9d6`. Publication date remains unset.
[Preview 16](preview-16-release-notes.md) and its assets remain unchanged.

## Replacement readiness (2026-10-20)

The identity above belongs to the superseded candidate and must not be reused as
qualification for current source. [Current triage](validation/release-readiness-20261020/README.md)
tracks bounded native record equality/hash fixes and remaining component/formatting gaps, raw
null String direct-call parity, and orders fresh source/package/editor/performance
gates after those issues are resolved. No replacement package identity is selected.

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
model. Canonical String/Char equality and matching hashes, Unicode 17 segmentation,
search predicates, trimming/line-ending utilities, Char helpers, builder reuse and
String.Format/Formattable are implemented with focused development qualification.
Original UTF-8 bytes are preserved. Shared-storage String views remain deferred;
position-based search/splitting and provider/primitive-specifier formatting remain
coverage gaps. Native generated record equality/hash is qualified for bounded
nongeneric positional records, including supported closed tagged-union components.
Generic record owners, unsupported components, imported value-record discovery and
generated formatting remain gaps; see [record union evidence](experiments/record-unions/README.md).

Console now exposes synchronous byte/text capabilities with configurable line endings,
and scoped ReadKey with logical keys, modifiers and optional Unicode scalar text.
POSIX entry-thread keys, any-key exit and Ctrl+C-only exit pass interpreter/JIT/AOT.
Windows key input, text-to-key switching and native worker attachment remain unsupported.
These additions require a new matched package run; development evidence is not
extracted release-package acceptance.

## Qualification remaining

Require canonical committed-source tests/format/Clippy and host checks, freshly
packaged macOS ARM64 and Windows x64 acceptance, matching editor tests, API/site
validation, and current performance evidence. Record failures and fix them before
publishing. Development bundles are not release assets. Website publication remains
a separate manual operation after validation.
