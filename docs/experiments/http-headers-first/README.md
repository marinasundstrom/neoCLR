# Headers-first server dispatch

Development after Preview 16. All HttpServer entry points deliver headers before
requiring the body: context Serve, response Serve, ServeOne, Accept and AcceptHeaders.
Nonempty content retains a bounded read-ahead buffer and reads further only on
consumer demand. Empty requests reuse the receive buffer without a body stream.

The independent peer deliberately withholds the body until the handler announces
HEADERS. It tests response without consuming the body, delayed UTF-8 text, delayed
JSON and server closure during a pending body read. The matrix covers all five
entry points in interpreter, JIT and AOT. It is a correctness probe, not a
throughput benchmark. Raw result files record artifact hashes and commands.

Build Probe.rvnproj with `scripts/build-native-project.py --profile http` and a
matching native bundle, then run:

```sh
python3 docs/experiments/http-headers-first/validate.py \
  --bundle target/matching-bundle/bundle --build target/headers-first-probe \
  --output target/headers-first-validation.json
```

Unread framing is not validated after an early response. The connection closes;
there is no keep-alive drain/reuse policy. Body framing and size failures happen
at read time, and the original request deadline still applies. JSON stream reads
currently materialize text within the existing 1024-byte quota. Measurements are
still needed to quantify allocations, CPU and throughput with both GC modes.

## Qualification notes

Raven `daa9d9070` fixes structural Function overload identity, native async base
receivers, imported CancellationToken capture and runtime Int32 range-array
construction. The library cohort was built immediately before that commit with
the same compiler bytes, labelled `34f1720f1-dirty-http-gaps-v4` in its manifest.
The website's simple context callback compiles with inferred locals and `unit`.
The installed JSON server sample compiles with asynchronous content consumption.

`evidence/aot-interpreter.json` records 40 passing withheld-body cases. Every run
also retains a CancellationToken across an async entry and explicit collection.
The initial context lifecycle report retains six passing AOT/JIT cases and an
interpreter peer-reset failure. Connection reset is now accepted as an aborted
exchange, while successful-response assertions still require exact bytes. The
separate interpreter rerun passes all three shutdown/admission cases.

The initial lifetime reports preserve ten passing native and ten passing
interpreter cases. The native harness initially expected an obsolete eleventh
case; that async capture check moved to the real async socket probe. A synchronous
unit-test adapter now uses Task.Map deliberately to observe TaskQueue boundaries.
Debug JIT compilation exceeded startup guards during concurrent builds; those
failures are not serving-throughput measurements. The initial typed-JSON native
probe also lacked reflection roots; the socket probe now uses DOM parsing and
typed mapping remains in the reflection-configured serializer suite.

The wider fixture compile audit is not a release gate pass: shared-TLS projects
need their generated certificate fixtures, and the large mapped-JSON `Public.rvn`
probe still encounters native async definite-assignment validation (`local loaded
before store on some path`). Its API spellings are migrated, but it is not claimed
as executable evidence. Routing projects compile after explicit application error
adaptation and equivalent `if` pattern dispatch. Remaining compiler gaps are
recorded in [the bridge notes](../../raven-cli-bridge.md).

For a throughput follow-up, compare headers-only early responses, delayed uploads
and fully consumed bodies separately, using identical callback, payload and GC
settings. Record completions, latency, allocation bytes/counts and CPU. Awaiting
I/O releases execution time but retains an exchange slot in the current 16-lane
Serve implementation; additional continuations do not themselves increase capacity.

Final qualification: all 60 withheld-body cases pass (40 AOT/interpreter and 20
JIT), and all 117 header/response, client-lifetime and JSON-stream contract
executions pass across the three engines. `evidence/jit.json` and
`evidence/contracts.json` retain the successful optimized-codegen runs. The
website checks 4512 pages and all 23 website tests pass; four native snapshot
tests pass. This evidence does not establish a throughput gain.
