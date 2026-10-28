# Dynamic member parameter qualification — 2026-10-28

Development implementation on macOS ARM64. This is a bounded .NET-style annotation
slice, not full trim safety. [Contract and remaining gaps](../../dynamically-accessed-members.md).

## Inputs and checks

- Raven native compiler bundle revision `ed1d08def`; five freshly compiled libraries,
  including PostgreSQL, in `target/dynamic-access-20261028/libraries-final`.
- RavenDoc revision `256019f66`; native reference archive, CLI documentation reference,
  XML comments and type inventory refreshed. Four native snapshot tests and the
  reference fingerprint check pass. No website build/publication performed.
- Eight focused `dynamic_access::tests` pass: exact tokens, forwarding, locals,
  control-flow joins, unknown inputs, malformed/unsupported targets, unused callers,
  typed reference field-store effects and address escape conservatism.
- Six existing `reflection_construction::tests` pass. No broad suite or performance
  claim; no JIT/Windows execution qualification in this slice.

## Native consumer

The [sample](../../experiments/dynamic-access/sample/Main.rvn) passes `typeof(Message)`
through an annotated parameter to JSON deserialization. With no retention file or
per-type options, native compilation infers flags 513 (public parameterless
constructor plus public properties); execution prints `retained`, exit 0.

The source HTTP service profile supplies the socket bindings reached by the selected
JSON library graph. The console-profile attempt fails at `neoCLR.Runtime.SocketClose`;
this limitation is not represented as a successful console qualification. The app
itself performs no networking. Nested JSON mapping emits four DAM2001 warnings and
the untraversed dynamic-dispatch boundary emits DAM2004. These warnings remain visible
in build reports; the simple sample needs only the inferred Message root.

Removing the forwarding annotation produces DAM2001 and no inferred Message root.
Without any policy the native build then rejects an unsupported reflection service
contract; it does not silently retain the type. A separate method-only policy control
builds successfully and exits 1 with the expected missing-property-retention fault,
without granting construction/property retention.
Using unsupported `PublicMethods` on the parameter fails compilation with DAM1001.

A combined-policy control retains method inspection explicitly while inferring
constructors/properties from the annotation; it prints `retained` and exits 0.
Explicit policy validation runs on a
copy before binding the union once, because binding rewrites reserved InternalCalls.
See [compact recorded results](results.json) for final control outcomes, diagnostics,
artifact digests and the selected retention report fields.

Earlier attempts exposed a reference-field stack-effect mismatch (fixed by reusing
typed verification), a generic dispatch-resolution failure and overly broad dynamic
implementation traversal. The bounded pass now documents/diagnoses the dynamic
boundary rather than walking every same-signature method. An intermediate combined
build completed code generation/linking but publication correctly failed because a
concurrent Cargo test rebuilt its compiler input; the final control reruns with
stable inputs. These failed attempts are not positive evidence.

## Reproduction

Follow the [consumer instructions](../../experiments/dynamic-access/README.md) using
the freshly built annotated libraries and matching backend. Add
`--reflection-method-type Message` for the combined-policy control. For the negative
control remove the annotation from `Read`, leaving the runtime JSON annotation in
place; for the unsupported-contract control replace its flags with `PublicMethods`.

```sh
cargo test -q --lib dynamic_access::tests
cargo test -q -p neoclr-codegen reflection_construction::tests
python3 scripts/build-api-docs.py --check
python3 scripts/test_native_api_snapshot.py
```

Generic parameter/field/return/receiver contracts, dispatch override compatibility,
reflection-only body recursion, stored type values and nested object closure remain
open. Explicit project retention remains necessary where analysis cannot prove roots.
