# Experimental Neo.Web foundation — 2026-10-31

Neo.Web is an experimental web framework built on top of
`System.Web.Http.HttpServer`. This author-selected slice implements builders,
application lifetime, context handlers and request scopes. It does not implement
generic handler injection or typed HTTP result alternatives. Those are next-slice
work; sealed hierarchies and unions remain open result representations.

## Qualified behavior

On macOS ARM64, both native executables and eager JIT pass the three NeoClr.Testing
contracts and the independent Python HTTP consumer against the reference app:

- Builder consumption, owned configuration, DI availability and option validation.
- Route validation, ambiguity rejection, literal precedence and frozen registration.
- Two concurrent loopback requests with request-scoped dependencies alive across
  awaited work, followed by exactly-once scope disposal.
- Sanitized 400/500, malformed-target 400, 404, and 405 with Allow.
- Handler cancellation, pre-cancelled startup, caller cancellation, and Stop joining
  pending handlers before application disposal. Stop closes connections rather than
  draining responses; the pending handler retains its scope until it settles.
- The reference app returns greeting text and 201 visit responses, preserves
  singleton state between requests, logs visits, and prints STOPPED after joining.

[results.json](results.json) records final source/compiler/codegen hashes, commands,
native framework output and five independent HTTP exchanges. [probes.json](probes.json)
records final-source interpreter/JIT commands and JIT sample exchanges.
[native-builds.json](native-builds.json) records executable/object/library hashes,
build phases and toolchain. Raw focused logs accompany these receipts.

## Execution gap and prerequisites

The interpreter probe fails with RuntimeError at the call to
WebApplicationBuilder.Build, before listening. Typed-stack/control-flow verification
of the preceding same-contract consumer passed (5,146 functions, maximum stack 10).
Native and JIT final-source consumers pass; interpreter parity is an open runtime
investigation, not a new permanent Neo.Web restriction. The focused validator defaults
to native; explicit `--mode interpreter` continues to expose the failure.

The combined hosting graph exceeded the native TypeShape generated-body limit of
8,192 instructions. Native codegen now shares identical closed shape-result blocks
while retaining token tests and selector validation. Five focused reflection-query
tests pass, including 720 closed class tokens plus a distinct value shape. The real
HTTP graph supplies native/JIT reference-class coverage. The bound is unchanged;
this is a lowering change, not new reflection semantics or a performance claim.

The base is the verified Preview 19 development bundle, with Raven compiler revision
`6ceeb1be8790e105a1394cb99fded41fafad5b4c`, plus source-built Neo.Web and the source
code generator containing grouped TypeShape lowering. The receipt pins compiler,
original manifest and codegen hashes. Published Preview 19 packages do not contain
Neo.Web or this lowering change. No Windows qualification is claimed.

Existing conservative DAM2001/DAM2004 diagnostics appear during JIT compilation.
The sample uses explicit DI factories and does not claim arbitrary reflection-based
constructor activation. No new Raven Runtime Contract, metadata encoding, Delegate
base class or generic-handler compiler feature was added.

The explicit `import Neo.Web.HttpContext` is compiled in the HTTP tests alongside
`import System.Web.Http.*`. The library still qualifies its internal constructor
expression because this compiler selected the transport constructor for the simple
name; public handler annotations and normal app imports do not need that workaround.

## Documentation and reproduction

The native RavenDoc audit includes all five public types with type/member pages.
The unified native snapshot uses the source-matched library; its build evidence is
[build-evidence.json](build-evidence.json). Renderer revision:
`f4fd726ad47e76fbeef68650686818bc22f3ab65`. The aggregate CLI reference fingerprint
check passes; four native-snapshot checks and four library-layout checks pass.
The full local website build passes 6,841 page/link checks, including examples,
API navigation and sample download links; see [the receipt](website-validation.json). Existing unrelated missing-description notices remain. No website deployment
or release publication is part of this change.

```sh
cargo build --release -p neoclr-codegen --bin neoclr-codegen
python3 scripts/validate-neo-web.py --bundle /path/to/verified-bundle \
  --codegen target/release/neoclr-codegen --output target/neo-web-check \
  --mode native --mode jit
cargo test -p neoclr-codegen --lib open_query_tests
python3 scripts/build-api-docs.py --check
python3 scripts/build-website.py
```

See the [reference app](../../../samples/web/README.md),
[API contract](../../../api-docs/neo-web.md), and
[design proposal](../../design/neo-web.md). The app remains the evolving reference
for subsequent handler binding and typed-result work.
