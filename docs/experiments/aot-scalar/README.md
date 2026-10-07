# First ARM64 scalar AOT probe

**Development experiment, 2026-10-07.** A separate Rust tool lowers a deliberately
small neoIL subset, now including branches and locals, through **Cranelift 0.121.2**, emits an ARM64 Mach-O object, and
links it to a C executable. The generated functions execute native code; there is
no interpreter, JIT or neoCLR runtime dependency in the scalar executable. The
compiler itself uses neoCLR for assembly and validation. This is the first bounded
implementation slice from the [native execution investigation](../../native-execution-investigation.md),
not a general AOT mode, stable ABI or production backend selection.

Original source/evidence branch: `codex/structural-types`, base `a081c6e3`, plus this local
experiment. This probe uses no structural Function feature, changes no Raven bridge
or compiler, and originally did not establish backend availability on main. At the author's
correction, commit `3571ab79` is cherry-picked onto main with fresh main validation;
the old validation.json remains historical evidence, not a claim about the new base.
[Main revalidation](main-validation.json) passes the same three tests, including
native C/interpreter parity and eleven rejection cases, against Preview 12 main. The source input
is hand-authored neoIL; Raven-produced native execution remains to validate.

## Supported contract

- One source module, no declared types; at most 128 uniquely named functions.
- Nongeneric free functions with Int32 parameters/results and up to 1,024 Int32
  locals; no native imports, special parameter contracts or managed runtime services.
- `ldc.i4`, `ldarg`, `ldloc`, `stloc`, `dup`, `pop`, wrapping `add`/`sub`/`mul`,
  direct unqualified local `call`, and `ret` with exactly one result. Early returns
  are supported; reachable paths must return or remain within the function's CFG.
- `br`, Int32 `brtrue`/`brfalse`, and `beq`, `bne.un`, `bgt`, `blt`, `bge`, `ble`
  with signed/unsigned forms. Loops and nonempty operand-stack joins are supported.
  Boolean-producing comparison instructions, switch and local reset remain outside
  this profile. The ordinary verifier enforces definite assignment of locals.
- An explicitly named `(Int32) -> Int32` root exports the C symbol `neoclr_entry`.
  Other functions are local object symbols. The module's ordinary entry-point
  spelling does not select the export; this is a C-hosted experiment.
- All declared functions are checked and emitted, including unused functions; there
  is no function trimming. Unsupported opcodes are rejected even in unreachable
  instructions; supported unreachable instructions are omitted during lowering.
  Calls must match their local signature. Call-graph cycles are rejected
  because native recursion budgets and Fault propagation are not implemented.
- Per-function body limit: 8,192 instructions. These are compiler acceptance bounds,
  not a native execution sandbox or the interpreter's instruction/stack quotas.
- Unsupported instructions, malformed stacks, inconsistent stack joins,
  uninitialized-local reads and unsupported metadata fail before an output file is
  created. A pre-existing output is never
  overwritten. The ordinary loader/verifier also runs before native code generation.

The target is explicitly `aarch64-apple-darwin`, with baseline ARM64 code generation
and a Mach-O macOS 11.0 minimum-version marker. The scalar compiler does not use a
platform SDK and records SDK version zero; the C host link supplies its own SDK and
deployment target. The marker is not a claim of macOS 11 qualification. Only the
recorded local host has been tested. Other architectures and OS object formats are
not yet exposed. Cranelift default optimization settings are used; these are
correctness results, not performance measurements.

The existing interpreter defines unchecked Int32 arithmetic by wrapping operations
([implementation](../../../src/numeric.rs)); Cranelift I32 add/sub/mul preserves that
behavior without C signed-overflow assumptions. The fixture computes `2*x + 2` via
separate multiply and two-argument subtraction helpers. Values come from a C host
at execution time, so native output is not just a precomputed constant. Checked
arithmetic, division, recursion, pointers, allocation, GC, Strings, HTTP,
JIT and hot reload remain unsupported. AOT does not yet run the web-app POC.

## Reproduce on macOS ARM64

From the repository root, with Rust and a matching Apple C toolchain/SDK:

```sh
cargo test --locked --manifest-path tools/aot-poc/Cargo.toml
mkdir -p target/aot-scalar
cargo run --locked --manifest-path tools/aot-poc/Cargo.toml -- \
  docs/experiments/aot-scalar/scalar.neoil Calculate target/aot-scalar/scalar.o
clang -arch arm64 -Wall -Wextra -Werror \
  docs/experiments/aot-scalar/host.c target/aot-scalar/scalar.o \
  -o target/aot-scalar/scalar
target/aot-scalar/scalar 20
file target/aot-scalar/scalar.o target/aot-scalar/scalar
nm -u target/aot-scalar/scalar.o
otool -tv target/aot-scalar/scalar.o
otool -L target/aot-scalar/scalar
```

Use a fresh object output path when repeating the emission command. Expected result
for input 20 is `42`; `nm -u` on the object is empty. Disassembly contains ARM64
arithmetic and call instructions. The executable links Apple's `libSystem` for its
C input/output host. That is an ordinary system dependency, not a managed runtime.

Local validation used Rust 1.95.0 and Apple clang 17.0.0, with the Xcode 26.2 SDK.
The host's default SDK discovery selected a newer CommandLineTools SDK incompatible
with the selected linker (`tapi` unknown architecture). The successful commands
selected the matching SDK explicitly, without changing machine-wide settings:

```sh
export SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk
export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
```

These paths describe this machine, not portable prerequisites. Use matching installed
compiler/SDK paths on another host. Only the native-consumer test is gated to macOS
ARM64; do not mistake a skipped consumer test elsewhere for native qualification.

## Validation and next boundary

[Tests](../../../tools/aot-poc/tests/scalar.rs) cover:

- ARM64 Mach-O format, successful emission and refusal to overwrite existing output.
- Eleven rejected programs: division, checked arithmetic, stack underflow, recursion,
  unknown target, mismatched call signature, excess return values, invalid argument
  index, empty return, non-Int32 result, and an unsupported operation in an unused
  function. The original scalar-only evidence rejected early return instead; it is
  now a supported operation with explicit CFG validation.
- A C consumer matching interpreter results for ten runtime inputs, including
  negatives and Int32 boundary wrapping; three invalid C command-line inputs fail.
  The object has no undefined symbols. C compilation enables warnings as errors.

Original results and artifact/source hashes are in [validation.json](validation.json).
[Control-flow evidence](control-flow-validation.json) records seven passing tests on
main: 79 native/interpreter comparisons across 14 C executables, and 17 rejected
programs. Additional negative cases cover inconsistent joins/backedges, a partially
initialized local, unknown label, falling out of the function, and an unsupported
opcode after an early return.
The first linker attempt exposed a missing platform command in Cranelift's object;
the tool now writes an explicit Mach-O build-version command. Final validation uses
that corrected object. No website build or broad runtime suite is required for this
isolated tool; the runtime and public library API are unchanged.

The [control-flow consumer](control-flow.neoil) sums doubled indices using locals,
a loop and a helper call; negative and oversized inputs return early. Input 20
returns 380. Run it with the same tool/C host above, using a fresh object path.
The lowering uses one Cranelift block per reachable instruction and explicit stack
block parameters, with Cranelift SSA variables for locals. This simple representation
is a correctness baseline, not a compiler performance claim.

Native loops have no instruction budget, cancellation polls or safepoints yet;
nonterminating inputs can run indefinitely. This trusted-code probe is not suitable
for resource-limited hosting. The interpreter's quota behavior is not promised by
this native profile. Fixtures deliberately bound their loops.

Next proposed slice: establish an explicit Fault ABI before division or checked arithmetic, then
UTF-8 console/runtime service boundaries, managed allocation/root reporting, and the
minimal AOT HTTP consumer. Keep benchmarking as a later measured comparison.

The tool's [manifest](../../../tools/aot-poc/Cargo.toml) and separate lockfile keep
experimental compiler dependencies out of the runtime manifest/lockfile. Its
[dependency notices](../../../tools/aot-poc/THIRD_PARTY_NOTICES.md) record licenses.
The pinned Cranelift version is a feasibility baseline, not a current-version or
production-support recommendation.
