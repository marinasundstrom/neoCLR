# Independent HTTP message surfaces — 2026-10-31

This slice separates HttpClientRequest/HttpClientResponse from
HttpServerRequest/HttpServerResponse. Incoming message headers expose the read-only
HttpHeaderCollection interface; outgoing headers expose MutableHttpHeaderCollection.
Empty, Snapshot and Create are static interface helpers. No common message base or
matching client/server body contract is imposed. HttpContent remains in this slice.
See the [message reference and migration](../../../api-docs/http-messages.md).

## Inputs and scope

- neoCLR base: `9e3358295`, plus this change's library, metadata producer and codegen
  changes. The interpreter engine is unchanged.
- Raven unified main: `a1f2c8588c0db37ee1dba141b18d7e00f2756c16` (native compiler and
  native documentation renderer). The compiler uses the matching neoCLR metadata
  producer from this change. No Runtime Contract override is added.
- Qualified composed bundle: `target/http-message-surfaces/inputs9/bundle`, rebuilt
  from `target/http-header-collection/inputs7/bundle`. The composition script records
  library source/artifact/compiler hashes; committed reports identify actual inputs.
- macOS ARM64 execution. No Windows or legacy .NET runtime execution claim.

## Results

- Header contracts: **16/16 in interpreter, JIT and AOT**. Covers named lookup,
  repeated fields, client framing restrictions, atomic mutation, attached outgoing
  views, immutable Empty, read-only incoming views, snapshots and content disposal.
- Allocation checks: **2/2 interpreter**. Warm named single-value lookup and cached
  views allocate no managed objects; parsed fields materialize HttpHeader entries
  only when accessed positionally. Native allocation counters are not qualified.
- Neo.Web framework: **6/6 per mode**; WebDemo logic: **4/4 per mode** on inputs7.
  This broad application matrix is retained as unaffected evidence after the final
  private header-replacement delegation. Inputs9 reruns the expanded header suite,
  allocation checks and primitive HTTP in their stated modes, plus the WebDemo AOT
  HTTP consumer on the final library.
- WebDemo external consumer: **30 HTTP responses plus a disconnect/cancellation
  check per mode**, with joined shutdown. Covers binding, DI, middleware and cache
  behavior using server primitives and client consumers with their distinct types.
- Primitive HttpServer: **15 HTTP responses across the three modes**. This is a
  correctness smoke, not a sustained capacity or before/after throughput comparison.
- Raven: **13 interface declaration tests**, plus same-assembly and imported native
  interface-factory probes. Pre-change extension/indexer baseline: 12 tests.
  Minimal interpreter factory execution returns 42.
- Metadata producer: static helper roundtrip/implementation-obligation checks plus
  existing static-interface, constrained-value and imported-interface regressions.
- Codegen: concrete helper selection and existing interface dispatch/fault execution
  checks pass. Helpers are direct-call bodies, excluded from dispatch contracts.
- API snapshot: **4 checks pass**. Website/link validation: **7,043 pages**, with
  133 legacy pages retained. Publication remains a separate manual workflow.

The native API reference includes all new message/header members. The legacy CLI
facade lacks MediaType, so five GetContentType members are explicitly excluded from
that facade only and linked to the manual reference. Its bootstrap C# declarations
are projected to interface metadata because that bootstrap core lacks default-interface
feature markers. No executable class carrier is introduced.

## Investigation notes

Compiler admission initially rejected static interface bodies. The bounded fix also
admits owned static property calls. Metadata authoring, import and readback previously
classified every interface member as abstract; concrete helper identity is now explicit.
An existing imported-interface regression caught and locked preservation of abstract
contracts. The code generator likewise needed to exclude helpers from dispatch-table
discovery. The runtime already supported concrete static interface helper execution.

Preliminary validation caught four client request factories incorrectly migrated to
server types in the framework's HTTP consumer; those now use HttpClientRequest.
A codegen execution test encountered a local linker/SDK mismatch and passed with the
explicit Xcode 26.2 SDK used for HTTP builds. Parallel/default multi-target compiler
builds hit host SDK/output conflicts; the successful compiler build targets net10.0,
and the successful focused test command reuses built project references. Website
validation caught legacy-facade coverage gaps and unescaped generic notation in the
new guide; final checks supersede those attempts.

Final author review moved content-type replacement from HttpServerResponse into
its header implementation without changing the public API or validation timing.
Extended assertions protect the different constructor/Respond clearing rules.
The composed bundle retains historical top-level provenance from its base; the
componentBuilds records identify the new libraries by exact source/compiler hashes.

No throughput improvement or release-default change is claimed. Independent message
storage enables later specialization; request/response body boundaries and middleware
rewriting remain open design work.

## Reproduce

Build Raven main's native compiler with the matching metadata producer:

```sh
dotnet build /path/to/Raven/src/Raven.Compiler/Raven.Compiler.csproj -f net10.0 \
  -p:UseRavenCoreReference=false -p:WarningLevel=0 \
  -p:NeoClrMetadataProject=/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj
cargo build --release -p neoclr-codegen
python3 docs/validation/http-message-surfaces-20261031/compose-input-bundle.py \
  --base-bundle /path/to/qualified-header-bundle --output target/message-inputs \
  --compiler-directory /path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0
python3 scripts/test-runtime-library.py --bundle target/message-inputs/bundle \
  --output target/message-contracts --aot target/release/neoclr-codegen \
  --runtime target/release/neoclr --suite http-header-values \
  --mode interpreter --mode native --mode jit \
  --jit-host target/message-inputs/bundle/bin/neoclr-jit-host.dylib
python3 scripts/test-runtime-library.py --bundle target/message-inputs/bundle \
  --output target/message-allocations --runtime target/release/neoclr \
  --suite http-header-allocations --mode interpreter
python3 scripts/validate-neo-web.py --bundle target/message-inputs/bundle \
  --output target/message-web --codegen target/release/neoclr-codegen \
  --mode interpreter --mode native --mode jit
```

The primitive smoke uses the existing
[HTTP consumer script](../http-header-collection-20261031/smoke-primitive.py); its
committed report records the exact commands. The other JSON reports retain input
fingerprints and results; focused compiler/metadata/codegen logs accompany them.
