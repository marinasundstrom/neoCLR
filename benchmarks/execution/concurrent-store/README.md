# Generated scalar stores for live GC — 2026-10-24

The private `mutation_store` lowering pass emits the experimental noncollecting
`experiment_gc_store` hook from scalar Cranelift stores. It is exercised by focused
JIT/AOT consumers, **not wired into ordinary compilation or exposed by a CLI option**.
Synchronous GC remains the default. This is a correctness integration step, not a
performance result or admission of managed HTTP handlers.

## Contract and implementation

The pass requires a 64-bit entry context parameter and an I32 status return. It
validates all memory-writing instructions before modifying the function. Supported
scalar stores preserve widths and low bits; floats are bitcast, preserving signed
zero and NaN payloads. Offsets are folded into the destination. Each failed hook
call returns its status before subsequent stores execute. Proven `stack_store`
instructions stay direct. Generic stores through `stack_addr` and checked constant
`iadd_imm` chains also stay direct when their full extent is inside that slot.
Known out-of-slot stores are rejected before edits; loaded pointers, block
parameters and arbitrary arithmetic do not gain a stack exemption. Explicit-endian, vector, atomic and other unadmitted
writers are rejected; callees and native providers require separate admission.

This provenance handling matters before frame publication: the current value
compiler uses generic stores for argument copies before linking its GC frame.
Making those stores fallible would introduce an exit before frame initialization.
The pass must precede root-frame return cleanup instrumentation, so its added fault
return also unlinks frames. The isolated consumers have no compiler root frames;
real compiler pipeline ordering and cleanup still require integration tests.
The hook cannot collect, including when an overflowing barrier buffer cancels the
marker. Expression temporaries need not be newly published at each store.
See [the native boundary](../concurrent-live/store-boundary.md).

The current dynamic destination classification walks allocation descriptors. It is
a correctness baseline with a known scaling cost, not a production fast path.
Calls, checks and extra branches also cost CPU/code size. Preserve proven storage
provenance and measure indexed byref lookup before admitting this broadly.

## Reproduction and scope

```sh
cargo test -p neoclr-codegen --lib mutation_store --no-run
# Substitute the test executable printed by cargo:
python3 benchmarks/execution/concurrent-store/validate.py \
  --runner target/debug/deps/neoclr_codegen-<hash> \
  --output target/concurrent-store/qualification
```

The harness emits ARM64 Mach-O and x64 Windows COFF objects and verifies the hook
import. It links the ARM64 object to the generated live collector, runs optimized
and UBSan/bounds consumers, then links a native fixture dylib and invokes it from
JIT-generated code. Windows qualification is object emission only.

The scalar test covers six types and six patterns, including signed zero and NaN
payloads. Mock-hook JIT tests additionally reject the first or second store and
verify status propagation and untouched bytes. The native consumer checks 100
cycles of deleted-edge retention, inserted-edge transitive retention, unrelated
object reclamation, invalid destination rejection and final empty-heap collection.
It gates the marker before mutation to make barrier necessity deterministic; this
is not broad scheduling stress or proof over every interleaving.

Ordinary compiler output, native provider writes, arrays, suspended tasks and HTTP
handlers are not admitted by this pass alone. Existing native gate/scalar tests
cover overflow and narrow writes; this slice does not replace those tests or close
the documented host race-detector gap. Runtime Contract configuration, Raven source
semantics, temporary CLI metadata and public APIs are unchanged.

## Design comparison and next gate

Reuse the [CLR/Go mutation-barrier research and writer inventory](../concurrent-mutation-audit/README.md).
The shared purpose is compiler/runtime cooperation for concurrent tracing. neoCLR's
current conservative payload scanner requires scalar-word safety, unlike a barrier
limited to semantically typed references. The resulting broader hook coverage is a
provisional cost of that representation, not a new managed semantic contract or a
claim of improvement over CLR. Native metadata/codegen must eventually carry enough
layout/provenance information to narrow this work safely.

Next admit a closed set of generated functions and native providers with an explicit
matching host capability, including construction/publication and return cleanup.
Only then run managed standalone workloads and plain/JSON HTTP with throughput,
process CPU, pauses and memory measured separately. This experiment does not justify
a default switch or predict application throughput.

## Recorded outcome

`validation.json.gz` records successful optimized/UBSan AOT runs and the real-host
JIT test (100 liveness/reclamation cycles in each run). `compiler-tests.json.gz`
records three focused lowering tests and two existing execution-metadata controls;
the host-dependent JIT test is skipped by the ordinary filter and explicitly run
by the harness. `native-gc.c.gz` and `stores.o.gz` retain generated inputs; the
validation report includes input hashes. No performance measurements were taken.

The initial AOT fixture omitted PIC and failed linking imported-hook relocations
(`initial-link-failure.json.gz`). Enabling PIC, as production AOT does, fixes it.
The linker still notes the test object lacks a platform load command and assumes
macOS; this fixture is not a distributable native image.

An earlier newly built Rust test runner stalled before main at `_dyld_start`;
`loader-sample.txt.gz` retains the sample. A stable byte copy, ad-hoc signed, ran
successfully; the original waiting processes also later finished. This does not
establish a signing root cause or a collector failure. The recorded final runner
is the stable signed copy. No host security policy was disabled.

## Stack-provenance qualification

The follow-up `stack-validation.json.gz` and `stack-compiler-tests.json.gz` record
qualification of the stack exemption. A JIT consumer stores through a stack address
with constant displacement, reloads the value and copies it through a caller-provided
pointer. Exactly one hook invocation remains, including when that byref write is
rejected. Negative and over-end stack extents fail before any IR edits. This proves
call elimination for the tested compiler pattern, not application CPU savings.
Unknown provenance still pays dynamic classification. Full frame-entry dominance,
return cleanup and complete writer admission remain integration obligations.
