# Dynamic-member retained-body closure — 2026-10-28

This follow-up analyzes annotated calls inside constructors and property accessors
that earlier annotation inference retained. It repeatedly adds those public bodies
to the analysis roots until the finite root set stops growing. This is a backend
extension of the existing TypeInfo parameter contract, with no new public API,
compiler encoding, Runtime Contract setting or runtime library build required.

## Focused validation

- `cargo test -q --lib dynamic_access::tests`: 11 tests pass. New checks cover
  reflection-only constructor dependencies and cycles, private-constructor exclusion,
  property-category selection, accessor dependencies and unknown input diagnostics.
  The malformed parameter-token check also rejects an out-of-range origin index
  instead of indexing an absent parameter.
- `python3 scripts/build-api-docs.py --check`: existing API snapshot fingerprints
  remain valid. No new API snapshot or website build is required.
- [Native consumer](../../experiments/dynamic-access/closure/Main.rvn): Message's
  reflection-retained constructor calls the annotated JSON helper with `typeof(Other)`.
  There is no retention file or per-type retention argument. Both builds use the
  same Raven `ed1d08def` compiler and annotated libraries from the initial validation.
  The existing HTTP service profile supplies the selected JSON graph's socket bindings;
  the consumer does no networking.
- Before: saved pre-follow-up backend builds successfully, but execution exits 1
  with missing property metadata inside Message's constructor. Only Message is inferred.
- After: both Message and Other are inferred with flags 513. Execution prints
  `retained` twice and exits 0. [Recorded results](results.json) include diagnostics,
  artifact hashes and execution output for both builds.

Reproduce by using the [sample instructions](../../experiments/dynamic-access/README.md)
with `--project docs/experiments/dynamic-access/closure/App.rvnproj`. Debug backends
were copied to stable task-local paths before the builds to avoid a concurrent
Cargo rebuild changing a hashed input. No JIT, Windows or performance claim is made.

## Scope and .NET comparison

Following dependencies inside retained bodies is consistent with .NET annotation
propagation; it changes which calls the existing analysis visits, not the flags'
meaning. Existing primary-source guidance and tradeoffs are linked in
[the contract](../../dynamically-accessed-members.md). This implementation repeats a
bounded flow pass and can cost additional compilation time. It conservatively
includes public constructors admitted by the existing constructor policy. The
backend continues to validate whether a requested reflection shape is supported.

Dynamic-dispatch implementation bodies, generic/field/return/receiver annotations,
types recovered through storage or reflection descriptors, and roots supplied only
by explicit policies remain gaps. The native sample still reports the existing
nested-JSON DAM2001 and dynamic-boundary DAM2004 warnings; the known type flows
required by this consumer are inferred. These warnings are not globally suppressed.
