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
