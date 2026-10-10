# Public JSON DOM and stream consumer

Development after Preview 9, 2026-09-25. This fixture consumes the provisional
System.Data.Json library APIs. It does not compile private copies of the parser or DOM.
The author selected a closed JsonValue hierarchy with kind-specific APIs; object
serialization through reflection remains a later milestone.

[Sample.rvn](Sample.rvn) parses a borrowed input stream, requires an object, adds an
acknowledgement and writes it to a borrowed output. It uses Result propagation for
expected failures and a type pattern to obtain JsonObject's APIs. The fixture executes
that sample. No network, routing, reflection, asynchronous I/O or HttpClient JSON
extensions are introduced.

[Design and limitations](../../json-dom-design.md) describes types, mutation, errors,
stream ownership, identity and resource limits. The earlier application-local
[document experiment](../json-document/README.md) remains historical evidence; it is
not the new public contract. In particular Add/Field/Item/Count now belong only to
appropriate containers, scalar values are typed properties, and errors use a union.

## Focused validation

```sh
python3 docs/experiments/json-dom/verify.py \
  --toolchain-root /absolute/path/to/development-bundle \
  --runner target/release/examples/measure_async
python3 docs/experiments/json-dom/verify-corpus.py \
  --toolchain-root /absolute/path/to/development-bundle \
  --runner target/release/examples/measure_async
```

Use the matching reference/bridge/System.neoil and the frozen compiler SHA-256
`57b6c6e33727de470fe529ee4b5c2b8fb338aeb60d1d6405d9e515276dccd96a`.
The independent reference uses .NET SDK 10.0.100.

The main fixture covers the actual acknowledgement sample, memory and one-byte
stream round trips, borrowed ownership, UTF-8, exact/oversized input, failed and
zero-progress writes, cyclic output validation, kind-specific APIs, missing vs null,
duplicate names including escaped aliases, invalid indices and nested stream causes.
Validation on 2026-09-25 passed with 665 allocated objects, peak 108, 16 collections
and zero final live objects under a 512-object heap. This is synchronous GC evidence,
not pending native-I/O retention evidence.

Validation on 2026-09-25 also passed all 54 valid/invalid documents and 12 checked integer
conversions against System.Text.Json and the explicit duplicate/limit policies. Python
compares parsed values while retaining number lexemes. Inputs are batched to avoid
loading the whole runtime for every individual JSON value. The measured runner
accepts guest arguments after `--` and uses a 100-million-instruction fixture budget
for each batch; the ordinary CLI default budget is too small for several documents.
Every batch must finish with zero live managed objects. Signature checks separately
cover all public members, internal helper rejection, upcast direction, closed-family
metadata and external-subclass rejection. No broad platform or website build is run.


The DOM entry points are `JsonSerializer.DeserializeNode` and `SerializeNode`,
with string and borrowed-stream overloads. Object mapping retains `Deserialize<T>`
and `Serialize`; rebuild older consumers after updating their DOM call sites.
The experiment-local serializer under json-streams is an earlier checkpoint and
retains its historical names; it is not the public System.Data.Json API.


The Node-name migration passes the focused public DOM/stream consumer and typed
object mapper, both with zero final live objects, plus 551 bridge signature checks.
Matching API and library snapshots validate. The website build is skipped by author
direction; its example is sourced from the updated compiled Sample.rvn.
The default managed HTTP pair also passes using Node calls, with 310 client and
309 server allocations and zero final live objects on both sides.

## Development byte-budget follow-up — 2026-09-27

The integrated consumer now checks the 1,024-byte document/number cap, multibyte
UTF-8, escaped output, one-byte overflow, borrowed input ownership and output
remaining untouched on validation failure. Shape limits remain unchanged. See
[payload validation](../json-object-mapping/payload-validation.json) for this run;
earlier results above describe their original, smaller fixtures.

## Raven framework migration — 2026-10-10

The in-memory DOM assertions in `DomContracts` now also run as focused attributed
module functions in [the JSON DOM suite](../../../runtime/raven/tests/json-dom/JsonDom.rvn),
alongside document round trips from the native JSON consumer. The framework reports
individual assertion failures and runs the same source in native and interpreted
modes. Stream failures, ownership, memory stream behavior and quota boundaries remain
in this consumer; this migration does not replace its broader integration coverage.

The subsequent [JSON stream batch](../../../runtime/raven/tests/json-streams/JsonStreams.rvn)
ports partial UTF-8 I/O, memory round trips, borrowed ownership, nested error causes,
zero-progress output, preflight cycles and quota checks. The original consumer still
covers the acknowledgement sample and independent MemoryStream contracts; corpus
validation also remains separate. Native/interpreted evidence is linked from the
framework README.

Development test migration (2026-10-10): MemoryStream seek, range, quota, overwrite and closed-state behavior now also run as 5
attributed framework tests in `runtime/raven/tests/memory-stream`. The framework README
links native/interpreted evidence. This original consumer retains its integration purpose.
