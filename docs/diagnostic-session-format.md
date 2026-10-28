# Diagnostic session contract, version 1

Development contract, 28 October 2026. Separate runtime data collection, transport,
archive representation and presentation. A future terminal or GUI client consumes
this contract without parsing a benchmark's stderr or knowing collector internals.
The first implemented adapter exports the existing native owner GC snapshot service;
it does not implement a heap inspector or a new runtime event stream.

## Serving and capture

The existing opt-in local Unix socket serves `NEOCLR/1 SNAPSHOT` responses.
[Attachment configuration and trust/lifetime rules](experiments/runtime-diagnostics/README.md)
remain unchanged: private directory/socket, bounded messages, owner-published scalar
copies, nonblocking owner publication, stale-cache reporting and no debugger commands.
No application HTTP server, remote TCP endpoint or automatic runtime listener is added.
The runtime never waits for a GUI or grants it direct guest heap access.

```sh
python3 tools/profiler/neoclr-profiler.py --socket /private/path/neoclr-PID.sock \
  --count 100 --interval 0.1 --session capture.json
```

The client retains existing NDJSON stdout and optionally writes a new session file
after successful capture. It refuses to overwrite an existing file. Sessions are
bounded to 10,000 observations. Export does not request collection or reset counters.
The socket's request version and archive version are independent contracts. A GUI
can read saved sessions now; a future live adapter must produce the same meanings.
The tool is developmental and not bundled as a supported installed product.

## Schema and compatibility

Machine-readable schema: [session-v1.schema.json](../tools/profiler/session-v1.schema.json).
Format discriminator: `neoclr.diagnostics.session`; major `version: 1`.
Reject unsupported major versions and required capabilities. Additional optional
fields may be added compatibly; existing meanings, units and scopes cannot change
under the same major version. Consumers ignore unknown optional fields. The first
schema describes only the GC adapter's supported payload. Future timeline/heap
payloads need their own specified schemas and validation before being advertised.

- `sessionId` identifies a recording, not an OS process.
- `sources` identifies PID plus runtime-instance nonce, owner-thread scope and clock.
  PID reuse must never join unrelated processes. This adapter exports one owner.
- `capabilities` distinguishes supported GC metrics from unsupported execution
  timelines and heap snapshots. Unsupported is not an empty/zero measurement.
- `metricDefinitions` gives units (`count`, `bytes`, `ns`) and cumulative counter,
  gauge or maximum semantics. Names match the existing served snapshot fields.
- `observations` preserves source sequence, timestamp, snapshot age and freshness.
  Repeated cached observations are allowed; their data cannot silently change.
- `limitations` travels with a recording so a GUI can display capture scope.

Every unsigned 64-bit integer is a decimal **string**, including PID, sequence,
timestamps, metrics and histogram counts. This preserves integers above JavaScript's
2^53 exact-number limit. Consumers range-check against 2^64−1 and use integer/BigInt
arithmetic before converting to chart coordinates. Version numbers remain ordinary
small JSON integers. Source timestamps are monotonic nanoseconds, not UTC; clocks
from separate processes or recordings cannot be overlaid without explicit alignment.
Observation array order is capture order. Source sequence/time must not regress.
Counters/maxima cannot reset within a session; a new session is required after reset.

Unavailable observations have null timestamp/age/metrics/histogram. Fresh/cached
observations contain all supported metric values. A cached observation does not
prove the application was idle or its allocation rate was zero. Do not derive rates
from repeated source timestamps. Gauges may fall; cumulative counters may not.

`collectionDurationBuckets` has 64 cumulative counts: bucket i represents
[2^i, 2^(i+1)−1] ns; bucket zero also includes zero. Quantiles from these buckets
are bounds, not exact p99 measurements. Collection duration is not universally
stop-the-world pause duration. No generation, region, fragmentation or object graph
is inferred from these scalar counters. Post-collection live payload is neither
instantaneous retained memory nor RSS; peak gauges are not sums of all owners.

## Future visualization boundaries

The author supplied screenshots showing timelines, heap regions, type totals,
objects, root paths and capture comparison. They are useful presentation references,
not evidence that the pictured tool can attach to neoCLR. neoCLR should expose its
actual collector/arena model rather than invent .NET generation labels.

Future payloads should include:

- Bounded timeline events with clock domain, source/owner/worker identity, event
  definitions and attempted/dropped counts; correlate JIT, GC, queue and I/O events.
- Explicit heap snapshots with capture time/safepoint semantics, object/type identity,
  shallow size, references and root categories. Root paths and retained sizes need
  validated graph analysis; address reuse must not merge object lifetimes.
- Allocation and root-publication profiles with method identities, symbol coverage
  and sampling/instrumentation costs. Unsupported/unattributed data stays visible.

Heap capture requires its own owner/marker synchronization and lifetime validation;
never read a changing heap from the diagnostics thread. Reuse existing trace and
snapshot producers rather than introducing GUI-owned runtime hooks. Measure capture
overhead separately from benchmark timings. A full GUI is not an optimization gate.

## Validation

Five focused exporter tests cover lossless large integers, unavailable/fresh/cached
observations, schema-version/source rejection, counter resets, gauge decreases and
capture bounds. The native attachment lifecycle test now exports a real session
through the standalone CLI and compares its values/identity with served snapshots.
Existing sanitizer-backed socket lifetime, slow-reader, malformed-request and
configuration tests still pass. This validates the GC adapter, not future payloads.

.NET's separation of runtime diagnostics transport from clients remains the baseline;
see [existing design research](runtime-diagnostics.md#baseline-and-alternatives).
This archive is a neoCLR-specific contract, not EventPipe or .NET dump compatibility.
