# Scalar JIT feasibility probe

2026-10-11 development experiment, macOS ARM64 only. The private native compiler
now accepts `--jit-scalar`: it loads a module, verifies and lowers the admitted scalar
functions eagerly through the same lowering as AOT, finalizes machine code in memory and
invokes the selected entry. It creates no object file or executable and does not
launch a system linker. This is not yet the normal neoCLR runtime execution path.

```sh
cargo build --locked --release --manifest-path tools/aot-poc/Cargo.toml
tools/aot-poc/target/release/neoclr-aot-poc --jit-scalar \
  docs/experiments/aot-scalar/control-flow.neoil Calculate 5
```

The result has `status: 0` and `value: 20`. The command uses the existing input
reader for textual neoIL/native metadata containers; the qualified paths are neoIL
and standalone NEOX. A Raven project/test assembly with managed library dependencies
is outside this profile. There are no Raven compiler or CLI bridge changes.

## Contract and boundaries

The root has signature `(Int32) -> Int32` or `() -> Int32`; the command still requires
one Int32 argument, ignored by the parameterless adapter. Functions may have Int32
parameters/locals, direct acyclic local calls, wrapping/checked integer arithmetic,
comparisons, branches and loops admitted by the existing scalar verifier. Boolean
intermediate stack types retain the verifier's restrictions. Unsupported instructions,
types, native imports, generic/instance functions, recursive calls and invalid bodies
are rejected. Validation covers unused functions as well as the selected root.

String literals, console and other external services are rejected. No managed heap,
GC, callback registration, suspension or scheduling support is claimed. There is no
instruction budget or cancellation for a running native loop; the probe is not an
isolation boundary. Other hosts return an explicit unsupported-host error. Windows
object-emission checks are separate from Windows JIT execution, which is not supported.

The private scalar ABI returns status separately from an Int32 output pointer. The
probe reports JSON: `status` is 0 for success, 1 for divide-by-zero, 2 for overflow,
and 4 for UserFault; `value` is null on faults. Reserved console-failure status 3 is
not reachable in this profile. A guest fault completes the probe command with JSON
and exit 0; loading/validation/compilation or command errors return exit 1. This is
diagnostic host behavior, not a new guest recoverable-error contract.

`loadSeconds` measures module reading/decoding. `compileAndFinalizeSeconds` includes
ISA/module setup, scalar verification/lowering and JIT finalization. `invocationSeconds`
measures one call. Process launch, JSON output and code reclamation are outside those
fields. These tiny samples validate measurement boundaries, not throughput or an
edit–test improvement. Use the [iteration harness](../jit-iteration/README.md) for the
existing full test workflow.

## Implementation and comparison

`compiler::compile_scalar<M: cranelift_module::Module>` supplies one lowering to both
`ObjectModule` and `JITModule`. Managed/value lowering remains separately coupled to
object emission. AOT keeps position-independent object emission; Cranelift JIT 0.121.2
requires `is_pic=false`, so its in-memory backend has a distinct relocation policy.
Both use the existing scalar C ABI and explicit fault statuses. No tiering, profile
feedback, persistent code cache, hot reload or on-stack replacement is implemented.

`CodeOwner` retains the JIT module through the synchronous call and frees its code/data
on success and error exits. No code pointer escapes the owner. That ownership is only
sufficient for this scalar scope; suspended frames, callbacks and concurrent execution
will require longer-lived ownership and coordinated reclamation.

The .NET comparison remains [tiered JIT versus Native AOT](../../native-execution-investigation.md#jit-for-development-iteration-2026-10-11).
This probe establishes shared semantic lowering and direct native invocation, not
feature or performance equivalence with the CLR. Cranelift source was checked against
the exact `cranelift-jit` 0.121.2 crate pinned in Cargo.lock: `src/backend.rs` documents
`free_memory` and finalized pointer lifetime and asserts the non-PIC requirement.
[Versioned primary API source](https://docs.rs/cranelift-jit/0.121.2/cranelift_jit/struct.JITModule.html).

The next milestone is a real test-framework assembly: share the managed lowering,
bind a narrow set of runtime helpers, and qualify root ownership and fault behavior
before adding HTTP/JSON or concurrent callbacks. Keep AOT-only deployment and the
interpreter available; a scalar JIT proof is not evidence that all native runtime
services can already run in a JIT host.

## Qualification

[Retained evidence](validation.json) records the release tool and source hashes,
with successful value, divide-by-zero, overflow and UserFault invocations. The existing
21-test scalar suite passes with interpreter/AOT/JIT parity checks; a subsequent
three-test JIT run adds NEOX container/corruption coverage and repeats two checks
(22 distinct scalar tests covered). Three Windows object-emission tests pass on
macOS; four Windows execution tests are explicitly ignored here. No Windows JIT
claim follows from those object checks. The managed lowering path was not changed.
