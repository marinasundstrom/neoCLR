# Loaded program boundary

`LoadedProgram` owns an immutable snapshot of validated, linked metadata. Preparation
normalizes source definition identities and binds calls before generic specialization.
Execution, typed verification, and closed type identity resolution consume that same
snapshot without relinking. Original Module values may be changed or dropped afterward.

```rust
let module = neoclr::assemble(source)?;
let program = neoclr::LoadedProgram::new(&module)?;
program.verify()?;
let first = program.run(neoclr::Limits::default())?;
let second = program.run(neoclr::Limits::default())?;
```

`new` prepares an application with bundled System. `with_library` accepts an explicit
System artifact. Additional dependencies can be supplied through [with_modules](module-sets.md).
Preparing System alone through `new` supports analysis. Modules
without an entry point can be prepared, verified, and queried, but cannot be executed.
All existing free run/verify/type-identity helpers remain available and prepare a fresh
LoadedProgram per call; hosts that want reuse retain the object themselves.

## Preparation and execution are separate

Preparation checks metadata and resolves references; it does not execute IL, load
foreign libraries, allocate guest objects, or perform typed verification automatically.
`verify()` remains an explicit analysis. A structurally valid program can still fail
typed verification or terminate with a runtime Fault.

Every `run` starts fresh frames, output, bootstrap heap, and native allocation tracking
with the supplied resource limits. A prior result or Fault does not change the loaded
snapshot. Returned execution values and allocations belong to their own Execution;
pointer/Ref values from one execution are not transferable handles into another.
There is no persistent guest state, argument passing, cancellation, or arbitrary
function invocation API in this slice.

Native imports remain disabled in safe `run`. The separate unsafe `run_with_native`
method has the same C ABI and native-code trust requirements as the existing helper.
Each such run gets its own native-library tracking, retained by the returned Execution
when successful. Foreign libraries may themselves maintain process-global state;
fresh guest execution state does not isolate or reset native code.

## Architectural scope

This is a shared metadata boundary for the current interpreter and analysis, and a
place from which future backends can consume the same resolved definitions. It does
not define a JIT/AOT backend interface, stable native hosting ABI, executable image
format, or general module loader. The linked Module remains private so consumers
cannot mutate bound identities or accidentally treat the combined table as a source
artifact whose rows should be renumbered.

Type lookup still uses globally unique names within the explicitly supplied module set.
Optional [direct reference lists](module-references.md) now constrain metadata uses
and root type queries. [Scoped type operands](scoped-types.md) check origin during
preparation; duplicate type names and module revisions remain pending.
Definition identities retain their existing build-local limits. No compiled-code
cache, generic specialization cache, automatic memory-management policy, or implicit
execution-mode fallback is introduced.

Run `cargo run --example loaded_program` for an embedding example that prepares and
verifies HelloWorld once, drops the source Module, and executes it twice.
