# HTTP foundation qualification

Development sources on `codex/http-foundation`, 2026-10-11. See the
[contract, .NET comparison and remaining hosting work](../../http-foundation.md).

Build a private library bundle using a verified native compiler bundle. An optional
`--libraries` directory selects separately hash-verified library inputs; this allows
preserving a separately rebuilt Runtime/Data/Networking baseline while changing
only Web. `--compiler-directory` plus `--compiler-revision` pins a local native
compiler copy when updating to a newer library baseline. No compiler checkout or existing bundle is mutated.

```sh
python3 scripts/prepare-http-foundation-bundle.py \
  --bundle /path/to/verified-native-bundle \
  --libraries /path/to/verified-library-directory \
  --output target/http-foundation-bundle
python3 scripts/validate-http-foundation.py \
  --bundle target/http-foundation-bundle/bundle \
  --runtime /path/to/neoclr --aot /path/to/neoclr-aot-poc \
  --output target/http-foundation-validation
```

The harness builds actual Raven consumers, then runs interpreter and native AOT
against independent Python TCP peers. It checks binary fixed/chunked transfers,
fragmented requests, close-delimited responses, informational responses, framing
ambiguity, truncated input, overflow and message/header bounds. Native runs include
32,769-byte bodies. `--large-bodies` additionally attempts the 1 MiB admission
ceiling; this currently fails the fixed network deadline under the native host's
collect-at-every-safepoint policy. A sampled attempt spent most CPU in collection.
The ceiling is not a throughput guarantee. Interpreter body cases use 2,049 bytes
because slower chunked parsing can also exhaust the fixed 15-second server budget. JSON consumers check awaited model reads, DOM/runtime-type reads,
repeatability, typed invalid-UTF-8 errors, pre-cancellation, unconsumed stream rejection,
atomic response configuration and JSON media type/status on the wire.

Compilation uses ordinary existing Raven async lowering. Small synchronous helpers
keep the fixture within the qualified compiler's definite-assignment and bounded
native state-machine carrier support; no compiler fixes are assumed. Immediate-task
observations in Json.rvn deliberately test completion/cancellation contracts.

The report records source/tool/bundle hashes and each result. Published POC evidence
and archived verifiers retain their original bounds; use this verifier for the new
HTTP contract. Published downloads are not rebuilt by this development change.

Native JSON fixture retention is explicit: `Json.reflection-roots.json` retains
Message construction and properties; `JsonServer.reflection-roots.json` admits the
fixture module catalog without retaining model construction. The AOT descriptor
binder supports closed function arguments and compact shape dispatch for the
combined HTTP/JSON type inventory. Its existing selector-parity test covers valid
shape flags and invalid-selector faults against the interpreter.

After integrating JSON main commit `2c8204d5`, rebuild all libraries first using
`scripts/build-native-class-library.py`, then pass that verified library directory
and the same pinned compiler to the HTTP preparation command. The integrated
consumer also checks JsonArray indexing and iteration after reading HTTP content.
The interpreter skips only the two 32 KiB header-byte-limit socket cases, which
can expire the fixed transport deadline; native mode covers those cases. Size
admission and deadlines remain independent constraints.
