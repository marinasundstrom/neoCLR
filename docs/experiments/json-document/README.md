# JSON document: a sensor acknowledgement

**Development experiment, 2026-09-23 · Partial M1/S3 evidence.**
A Raven consumer reads a small sensor report, accesses fields explicitly and writes
an acknowledgement. It extends [JSON messages](../json-message/README.md) using
existing managed objects, collections and text. It adds no runtime instructions,
public library API, reflection serializer or streaming parser.

## Run the product

[Main.rvn](Main.rvn) receives:

```json
{"station":"Café","readings":[21,22.5]}
```

It reads `station`, checks that `readings` is an array, converts the first reading to
Int32 and constructs a fresh reply:

```json
{"station":"Café","accepted":true,"count":2,"note":null}
```

The number `22.5` is accepted without needing a floating-point conversion. The
[expected output](expected.txt) also displays the station and first reading. This
is the original in-memory checkpoint. The later [HTTP report](../http-json/README.md)
reuses these sources between separate client/server applications without promoting
a public JSON API.

```sh
python3 docs/experiments/json-document/verify.py --toolchain-root /absolute/path/to/development-bundle
```

Use the matching Raven/neoCLR bundle and .NET SDK **10.0.100**. The verifier pins
net10.0 in a temporary project; no extra NuGet dependencies are added. Standalone:

```sh
export NeoCLRRoot=/absolute/path/to/development-bundle
export RavenSdkRoot="$NeoCLRRoot/raven-sdk"
dotnet msbuild docs/experiments/json-document/JsonDocument.rvnproj -nologo
"$NeoCLRRoot/bin/neoclr" run docs/experiments/json-document/bin/neoclr/Debug/App.neoil --system "$NeoCLRRoot/lib/System.neoil"
```

## Explicit provisional contracts

[JsonValue.rvn](JsonValue.rvn) uses a sealed class family: object, array, string,
number, boolean and null. Fields and indexed children return Result; missing fields,
wrong conversions and bad indices are errors. A present JSON null is a JsonNull
value, so it is distinct from a missing field. Count is the child count (zero for
scalars); Item enumerates children of either container, and Name names object entries.

Object names compare ordinally, case-sensitively **after JSON unescaping**, without
Unicode normalization. Duplicate names are rejected, including `"a"` and `"\u0061"`.
Put adds a field; it does not replace one. Objects retain insertion order. Add appends
to an array. Failed duplicate, wrong-container and capacity operations do not mutate
the container. Factories and checked conversions are convenience APIs for this
sample, not a selected public DOM contract.

Numbers retain their validated JSON spelling, including negative zero, fractions,
exponents and values outside Int32/Double range or precision. Writing preserves that
spelling. AsInt32 performs checked conversion of an integer token only: `1.0` and
`1e0` are not converted to 1, and overflow returns an error. No decimal/floating-point
conversion, coercion from strings, or arithmetic is implied by accepting a number.

The reader accepts exactly one JSON value, with JSON whitespace around it. Strings
reuse the strict message codec, including surrogate-pair validation. Comments,
trailing commas, BOMs, unknown literals and trailing values are rejected. Input is
already a valid neoCLR String; invalid wire UTF-8 belongs at the preceding codec
boundary. Parsing and writing return one complete result or an error.

Limits apply to both reading and writing:

- **128 UTF-8 bytes** for the complete input or escaped output, including punctuation.
- **Four nested containers**; a primitive inside the fourth container is permitted.
- **32 value occurrences**, including the root and containers, excluding field names.
  Shared children count on every visit, as they occupy output more than once.
- Each constructed container admits at most **31 children**. A constructed graph
  can still exceed overall limits; the writer checks them before returning output.

The tree is mutable and may share children. A cycle fails the writer's depth limit;
there is no dedicated identity-based cycle diagnostic or concurrency guarantee.
Per-call reader/writer helpers own their cursor and output; use ReadDocument and
WriteDocument to start a fresh operation. Runtime instruction/allocation budgets
are additional constraints, not throughput promises. This deliberately small limit
keeps the sample comprehensible; it is not a proposed HTTP body limit.

## Comparison and choice

Primary sources checked 2026-09-23; executable baseline is .NET 10:

- [RFC 8259](https://www.rfc-editor.org/rfc/rfc8259.html), §§4–9: JSON objects,
  arrays, number grammar, strings and parser limits. Reuse the earlier message
  experiment's distinction between grammar-valid escapes and interoperable Unicode.
- [.NET JsonElement.GetProperty](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonelement.getproperty?view=net-10.0)
  uses ordinal matching and selects the last duplicate name. The reference probe
  verifies that baseline, then separately applies the fixture's rejection policy.
- [.NET JsonElement.TryGetInt32](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.jsonelement.trygetint32?view=net-10.0)
  provides explicit checked conversion; the probe compares boundary, fractional and
  exponent cases. The fixture preserves raw number text until conversion is requested.
- [Serde JSON 1.0.151 Value](https://docs.rs/serde_json/1.0.151/serde_json/enum.Value.html)
  provides an explicit JSON value family in Rust. It supports the case-oriented
  model; its ownership and numeric representation are not neoCLR contracts.
- [Json.NET duplicate-name handling](https://www.newtonsoft.com/json/help/html/T_Newtonsoft_Json_Linq_DuplicatePropertyNameHandling.htm)
  exposes replacement, ignoring and rejection as policy choices. Rejection is not
  a new runtime mechanism or a general improvement over .NET's default behavior.

The selected layer is an application library. Neither CLI metadata nor the runtime
needs a JSON type. Compared with .NET's document-backed elements, these managed
objects retain their own text and children without a disposable document lifetime.
The cost is many allocations and linear field lookup. Duplicate rejection makes the
report unambiguous but rejects some inputs .NET accepts. Exact number retention
avoids silent rounding at the cost of deferred conversion and no numeric operations.
There is no measured performance advantage.

| Alternative | Benefit | Cost / provisional decision |
| --- | --- | --- |
| Whole-buffer mutable value tree | Simple field access and explicit construction | Allocations, linear lookup, aliases and repeated String concatenation; selected for this tiny consumer |
| Document-backed slices | Less per-value copying may be possible | Source ownership, lifetime and parser indexing contracts; not selected here |
| Incremental tokens | Progressive processing of larger input | Suspended token state and partial failure contracts; this bounded consumer does not require it |
| Reflective typed serializer | Convenient model binding | Missing/type/default/number policies plus reflection coupling; premature for this sample |

## Target constraints discovered

The current Raven bridge rejects **application-defined enums** (its existing library
enum bindings remain available). An enum-tagged draft could not be imported; the
sealed class family uses already supported nominal types instead. This is an
importer admission limitation, not evidence that the runtime cannot represent enums.
A general application-enum probe remains deferred foundational work.

The bridge also rejects protected cross-type constructor calls. The sealed base's
constructor is public for derived calls; Raven's sealed-family base is abstract, so
it cannot be instantiated directly. Propagated Result<unit, string> statements use
an explicit discard (`_ = operation()?`) to avoid a residual Void on the current
bridge's stack. Number factory results need a JsonValue-typed binding before Ok to
supply the intended carrier type. No compiler or importer changes are made here;
these constraints should be revisited before choosing the public API.

## Validation and remaining work

The verifier compares **54 valid/invalid documents** with System.Text.Json plus the
explicit fixture policies. It parses every accepted target output independently
with Python, retaining number lexemes for comparison. Coverage includes all six
value kinds, escaped names/strings, duplicate names after unescaping, malformed
containers/literals/numbers, depth/node/byte boundaries, and large or tiny exponents.
Twelve numeric cases compare checked Int32 outcomes with .NET. Additional target
checks cover missing versus null, wrong-type access, invalid indices, rejected
construction, duplicate mutation, cyclic graphs and writer output/node limits.
This is targeted evidence, not an exhaustive JSON conformance suite or proof of
pending native-I/O GC rooting. The earlier string codec remains unchanged.

The product makes the small in-memory JSON case concrete. Public error types and
locations, efficient buffering, API ergonomics, broader conformance and production
limits remain open. At this earlier checkpoint, the next step was controlled delayed
completion with actual guest GC retention, then reuse the pipeline for files before
sockets. The application-enum and protected-constructor findings are bounded future
integration probes, not automatic reasons to redesign metadata or the runtime.
