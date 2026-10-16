# Raven generic classes over local ordinary bases

This development consumer qualifies Raven `codex/neoclr` native emission of
`Box<T> : Base`. It uses `Box<int>` and `Box<Base>`, a computed direct-base
constructor argument, private generic payload storage, a nominal upcast,
inherited-field reads/writes and reference identity/aliasing. Successful execution
returns 42. The library base remains nongeneric; this does not qualify
constructed base authoring or generic derivation from external bases.

Run on macOS ARM64 from the neoCLR checkout:

```sh
python3 docs/validation/generic-class-base/raven/run.py \
  --compiler /path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/raven-generic-base-payloads-20261016
```

Build the compiler against the matching local metadata SDK. The runner records
compiler/runtime/bundle hashes and validates interpreter, AOT and managed JIT,
with native cleanup and AOT undefined-behavior/bounds sanitizers. The focused
Raven `PortableExternalInheritanceTests` also checks capability admission and
emitted CLR behavior. The pre-change source consumer was rejected with
NEOMETA001 on `Box<T>`; no metadata format or Runtime Contract setting changed.

The native emitter still rejects reference `!=` here; the sample deliberately
uses `Object.ReferenceEquals` to express the identity check. Reference comparison
coverage is separate from this generic-inheritance qualification. Constructed
generic bases and helper/control flow in base initializer expressions remain open. Structural work and
Sliceable/Result variance remain deferred.

## Separately compiled library

Pass `--imported` to compile `Library.rvn` to a native library first, then compile
`Main.rvn` against that exact dependency. Without the flag, the runner combines
both sources in one compilation. Both modes also require three source conversion
errors: Base to Box<int>, Box<Base> to Box<Object>, and Box<int> to Box<bool>.
Generic classes remain invariant even when their arguments are reference types.

Native import now admits generic class definitions with a nongeneric class base;
it preserves exact dependency identity and registers the declared base edge.
This is importing a completed generic hierarchy, not authoring a new generic
class over an external base. The latter and constructed generic bases remain gaps.

Both modes return 42 in interpreter, AOT and managed JIT. Native cleanup and
AOT sanitizer checks pass. `results.json` records the local run;
`imported-results.json` records the separate-library run with dependency hashes.
Before the import fix, library compilation passed but the consumer rejected
the generic Box definition with NEOMETA001 (unsupported symbol-only emission contract).
