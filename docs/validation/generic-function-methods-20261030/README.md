# Generic function methods and library signals — 2026-10-30

Author-directed development on neoCLR main and Raven codex/neoclr. The original
2026-10-29 receipt records unsupported syntax before implementation; it is historical,
not the current status. Use matching compiler, runtime and library artifacts.

## Scope

Native format 6 adds a whole-function method signature and a FunctionReturn type
expression. Generic substitution materializes the public signature. A boxed-body
transport marker lets a general adapter pack invocation arguments into object?[]
and adapt the result. Raven supports `func name: T with args`, with T a declaring-type
func-constrained parameter; the body returns ReturnType<T>. No event metadata,
nominal function requirement or Task unwrapping is introduced.

Neo.Signals is a separate extension assembly with Signal<T>/SignalSource<T>, stable
subscription-only properties and disposable registrations. Unit results are checked
at source construction. Reentrant changes affect subsequent snapshots, faults stop
dispatch, and concurrent access requires external synchronization.

## Focused checks

- SDK `--dependent-function-signatures`: native binary roundtrip, open constrained
  MethodInfo.FunctionType, closed zero/one/two-parameter shapes, result projection,
  and separately authored imported call validation.
- Rust `dependent_function_methods`: argument packing/result adaptation and malformed
  constraint/body-contract rejection.
- Rust `dynamic_function_invocation`: checked invocation, receiver retention,
  argument rejection, fault propagation and inhabited unit result support.
- Raven `NativeStructuralFunctionTests`: structural identity, projection substitution,
  parser roundtrip, nullable Object argument binding, materialized parameters and
  target/invalid-body diagnostics.
- `samples/signals/SignalsDemo`: separate assembly import; empty and zero-argument
  dispatch, nullable reference arguments, bound receiver, duplicate registrations,
  idempotent disposal, order and reentrant snapshots.
- `samples/signals/SignalReflectionDemo`: closed MethodInfo.FunctionType and
  ordinary parameter/result reflection, kept separate from the usage example.
- `samples/signals/AsyncSignalsDemo`: timer-delivered callback observed through
  Promise<int>/await, with an independent five-second timeout. No reset event or
  asynchronous signal policy is added.

The initial typed helper consumer returned Int32(42). Final run outcomes and exact
artifact hashes are recorded in results.json. Logs retain actual pass/failure output;
there is no throughput or allocation claim. Website publication is separate.

## Reproduction

Build the matching Raven native driver with `NeoClrMetadataProject` pointing at
this checkout. Build System.Runtime, then the three sample projects through
`rvnc neoclr --project` (the project graph includes Neo.Signals). Use
`--no-build-references` only after building the matching dependencies.

Run each sample with the native System seed plus explicit System.Runtime and
Neo.Signals modules, selecting System.Runtime as Object root. Interpreter uses
`--execution interpreter`; native execution requires the matching compiler and host.
The sample projects and extension README contain the source-level usage.

The initial profile excludes ref/out packs, arbitrary value aggregate reflection
marshalling, projected virtual/async methods and method-generic signature operands
in Raven syntax. Task results remain ordinary results. CLI/neoIL export of the new
signature forms is unsupported. Rx completion/error/scheduling operators remain
proposed library work.

Execution qualification: the signal consumers pass in the interpreter. The native
compiler builds and shares signature specialization, but the signal JIT probe stops
at unsupported boxing of inhabited unit (`Void`). JIT/AOT execution of this library
is not qualified; unit boxing/unboxing support remains follow-up work.
