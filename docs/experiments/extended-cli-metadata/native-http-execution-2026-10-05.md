# Native async propagation and HTTP execution — 2026-10-05

The unchanged HTTP JSON application sources now compile through ordinary `rvnc neoclr`
with native Numbers/Http references and execute on neoCLR. No application sources or
references are translated to a CLI projection. The explicit primitive core, retained
runtime seed and ownership manifest remain bootstrap inputs; full System is not claimed.

## What changed

Raven's portable emitter preserves empty-stack context for a reference field RHS and
permits lowered method exits only there. It keeps distinct generated local symbols in
distinct storage slots. Generated async self receivers are reloaded after the RHS rather
than saved in a temporary that resumption can skip. Ordinary receivers still evaluate
before the value. These are compiler emission fixes; no metadata schema, runtime scheduler,
public library API or JSON/HTTP sample rewrites are required.

Compared with Raven's .NET target, this consumes the same heap async lowering and semantic
Result propagation while authoring native metadata through the existing builders. The
Reflection.Emit implementation is retained. A separate ordinary .NET field-return bug
was discovered and recorded in Raven's `docs/compiler/neoclr-fix-integration.md` for main
reproduction and isolated repair; it is not claimed fixed by this native emitter change.

## Execution gates

`bootstrap/native-async-propagation.rvn` awaits a pending Result, proves success returns
42, and proves error returns its original message without executing subsequent mutation.
The earlier async-state consumer still checks completion, cancellation, capture identity
and private receiver mutation.

`scripts/verify-native-http-json.py` runs the emitted unchanged server against four Python
requests: GET report (200, Unicode Café), POST report (201), malformed JSON (400), missing
route (404). Then a fresh server runs against the emitted native Raven client for its
GET/POST mapping flow. The client yields `{"accepted":true}` (compared as JSON); each server
prints exactly `Reports served\n` after its port line. All processes exit zero with empty
stderr. The runner records artifact hashes, commands, response bodies and exit/output
assertions and cleans up its server on failure.

Reproduce after compiling the selected cases using the ordinary inventory command:

```sh
python3 scripts/check-native-poc-samples.py \
  --compiler /path/to/rvnc.dll --core /path/to/IntrospectionCoreParams.dll \
  --seed /path/to/System.neox --ownership /path/to/ownership.json \
  --reference /path/to/Numbers.dll --reference /path/to/Http.dll \
  --async-library Numbers --case native-async-propagation --case native-async-state \
  --case http-json-server --case http-json-client \
  --runtime target/release/neoclr --output /tmp/fresh-native-http
python3 scripts/verify-native-http-json.py \
  --runtime target/release/neoclr \
  --server /tmp/fresh-native-http/http-json-server.dll \
  --client /tmp/fresh-native-http/http-json-client.dll \
  --seed /path/to/System.neox --module /path/to/Numbers.dll --module /path/to/Http.dll \
  --output /tmp/fresh-native-http/localhost-evidence.json
```

[Compilation/runtime controls](native-http-compilation-2026-10-05.json) and
[localhost execution](native-http-execution-2026-10-05.json) preserve exact input hashes,
commands and outcomes. The compiler revision in the initial evidence is the parent of
this slice; binary hashes identify the tested implementation. No runtime rebuild was
needed. These are development-branch results, not a published release.

## Remaining bounded work

Async Main needs an explicit completion contract that preserves result and failure
handling. Both HTTP samples use synchronous entry points and existing queue dispatch;
this gate does not implement async entry completion, generic async methods, runtime
suspension or green threads. Keep full class-library expansion and unrelated APIs outside
this sample-driven POC slice. No new guest public APIs were added; API snapshot gaps
remain tracked separately and no website build was required.

Implemented compiler revision: Raven `8abaab09b`. All 36 focused async/portable-body
tests pass. The new field-return plan test does not claim the separately recorded
Reflection.Emit field-return case executes; that candidate still needs main validation.
