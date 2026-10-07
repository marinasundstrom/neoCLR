# Raven union app compiled to native ARM64

**2026-10-07 — completed bounded development milestone.** An ordinary Raven `union`
with Some(Int32) and None now compiles through neoCLR metadata/IL to a standalone
macOS ARM64 executable. It needs no shared managed framework, interpreter or runtime
installation. macOS `libSystem` remains its sole dynamic dependency.

The [source](../aot-values/union-app.rvn) constructs both cases, returns unions through
functions, extracts Some payloads with patterns, checks None, checks non-matching
patterns and verifies independent value copies. It exits zero only if all checks pass.
This is a correctness sample; it does not yet read console input or print union values.
The [native fixture](../aot-values/UnionApp.pe) uses ordinary compiler-emitted union
storage and members. No manual carrier or union-name-specific lowering is used.

## Reproduce

Use the native-enabled pinned Raven bundle described by the earlier
[producer inventory](../aot-values/inventory.json), compiler revision
`70aea9a7e9e424159a48b0227869245a95bf2ec2`. The bundle's Core, System runtime seed,
System.Runtime reference, ownership configuration and explicit Object library are
build/interpreter-validation inputs, not deployed companions.

```sh
cargo build --locked --manifest-path tools/aot-poc/Cargo.toml
python3 docs/experiments/aot-values/verify_union.py \
  --compiler /absolute/path/to/bundle/sdk/tools/rvnc/rvnc.dll \
  --compiler-revision 70aea9a7e9e424159a48b0227869245a95bf2ec2 \
  --bundle /absolute/path/to/bundle \
  --runtime /absolute/path/to/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/aot-union-run
```

The output directory must be new. Set a compatible `SDKROOT` as in the
[Hello World build instructions](../aot-hello/README.md). The script compiles fresh
Raven source, runs the complete artifact in the interpreter using explicit native
System/Object-root dependencies, compiles native code, links the startup adapter,
checks undefined symbols/dynamic dependencies, and runs the copied executable alone
with an empty environment. [Recorded evidence](../aot-values/union-validation.json)
contains exact inputs, hashes, commands, selections and results. Fixture rebuilds
are not assumed byte-identical; fixture and fresh producer hashes are recorded separately.
All 44 focused AOT tests pass: 26 value/union, 15 scalar/console and three inspection
checks, including private/readonly rejection and unsupported selected-code rejection.

## Explicit selection and limits

The native command is:

```sh
neoclr-aot-poc --closed-world UnionApp.pe @entry union-app.o > selection.json
```

The current app has 22 functions and six types. Selection compiles ten functions and
three value types, reporting twelve excluded generated methods and three attribute
classes. The complete original artifact is preserved. Normal whole-module compilation
still rejects it. Selected instruction bodies are retained, including direct calls in
dead code; normal AOT verification, constructor/output checks and access/readonly rules
still apply. Source origins retain semantic access facts in the internal verification
projection. Invalid foreign assembly origins are rejected.

This is an explicit, limited code-selection step brought forward from later trimming,
not general reflection-aware trimming. Unused generated formatting, boxing and virtual
members are excluded; executing them is unsupported. There is no dynamic loading,
reflection, native managed heap or general library linking. A subsequent
[bounded generic Result slice](../aot-values/README.md#generic-result-and-pattern-bindings-2026-10-07)
now supports one closed shape per local value definition.
Reference-free union layout is the existing private bounded value ABI, not a stable
foreign ABI. General union/library support and metadata retention remain unfinished.
See the [selection contract](../aot-values/README.md#explicit-closed-world-selection).

## Reassessment: next sample

Use an **interactive integer reader** as the next driving sample: read a line, match
end-of-input, parse into `Result<int, ParseError>`, report success or invalid/overflow
input, then continue until EOF. This follows the author's console-input direction and
provides concrete tests for Option/Result, UTF-8 strings, input services and ownership
before a network server adds sockets and concurrency.

Implement it in bounded stages:

1. Compile a value-only Result success/error consumer with fixed inputs, and establish
   generic value specialization/dependency contracts needed by the actual library Result.
   **Partial completion:** local ParseResult<int, byte> now runs with bounded specialization;
   actual library dependencies and multiple instantiations are still open. The
   [real library probe](../aot-values/README.md#real-library-result-dependency-boundary-2026-10-07)
   now passes the interpreter and records the runtime-library AOT boundary. Separate
   [value libraries](../aot-library/README.md), including bounded generic values, now
   compile. Explicit runtime-owned validation contexts now pass too; the Result
   interface conformance is now verified before direct-call compilation; the actual library
   Result success/error consumer runs standalone. Multiple instantiations remain open.
2. The [nested input-outcome sample](../aot-input/README.md) now validates the actual
   Result<Option<byte>, ConsoleReadError> shape. Metadata-only static member owners now
   compile. Bound erased-value transport and explicit native byte-input services remain
   before extending to text.
3. Define the native UTF-8 line-input and lifetime contract, including empty line versus
   EOF and I/O failure. Link the service into the executable and retain interpreter parity.
4. Compile parsing and output with valid, invalid, overflow and EOF tests. Use the ordinary
   library APIs once supported; do not replace them with silent backend intrinsics.

The input/lifetime step should compare bounded owned buffers, native tracing and reference
counting before selecting a mechanism; reference counting has not been chosen. An HTTP
server remains the later motivating sample. Stable metadata/native ABI exploration and
benchmarks remain separate work, driven by concrete consumers rather than prerequisites
for this next console milestone. This is the assistant's recommendation after the union
milestone, not a claim that input or string lifetimes already work in native code.
