# neoCLR 0.1.0-preview.17 — 2026-10-23

Matching macOS ARM64 and Windows x64 packages with SDK/editor
`0.1.17-neoclr.20261022.1` and Raven `301e668a5`. JIT is the default; interpreter
and bounded native AOT workflows remain available. This is an experimental platform.

## Highlights

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

## Qualification

Canonical extracted-source tests, 25 smoke programs, both host suites and Rust 1.85
checks pass. Both versioned packages pass runtime/API/editor/HTTPS qualification and
fresh CPU/JSON/HTTP measurements. [Exact evidence and limitations](validation/release-20261023/README.md).

The matching native build kit is available for macOS ARM64, including console/HTTP
and scalar C/C++ interop consumers. Explicit HTTPS-server AOT remains source-built.
SDK/compiler tools require .NET 11 RC1 and .NET 10; the neoCLR runtime does not.
Standalone SDKs, editor VSIX packages, qualified source, native kit and dependency
notices are covered by the release SHA256SUMS file. Keep package versions together.
