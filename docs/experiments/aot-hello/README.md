# ARM64 AOT Hello World

**First executable milestone, 2026-10-07 — development experiment on main.**
Compile [Raven Hello World](hello.rvn) to neoCLR metadata containing its IL, then
compile that artifact into ARM64 machine code. Link the native startup
adapter and UTF-8 console implementation into the executable, and run that executable
by itself. It prints exactly `Hello, world!` followed by LF and exits with code zero.
No shared neoCLR/.NET framework, managed runtime, compiler, interpreter, source IL or
separate service library is needed beside the executable. The macOS system library
`libSystem` remains an OS dependency; this is not a freestanding binary.

The author's target is **neoCLR CIL to a self-contained native executable**, with
required library code and runtime support baked in. The input pipeline now exercises
Raven's actual PE/#Neo output, using neoCLR's native decoder and verifier. It never
imports or executes the PE container's CLI projection. The original [neoIL sample](hello.neoil)
remains a small backend conformance input.

## Raven to metadata/IL to native code

The tested source is deliberately small:

```raven
func Main() -> int {
    System.Console.WriteLine("Hello, world!")
    return 0
}
```

Use a native-enabled Raven compiler. The recorded build uses revision
`70aea9a7e9e424159a48b0227869245a95bf2ec2` from the
[qualified compiler bundle](../extended-cli-metadata/clean-bootstrap-reproduction-2026-10-07.md),
with compiler/metadata assembly hashes in [Raven pipeline evidence](raven-validation.json).
This does not claim that an arbitrary Raven main checkout contains the same native adapter.
No Raven compiler or metadata encoding changes are needed for this slice.

```sh
cargo build --locked --manifest-path tools/aot-poc/Cargo.toml
python3 docs/experiments/aot-hello/verify_raven.py \
  --compiler /absolute/path/to/native-enabled/rvnc.dll \
  --compiler-revision 70aea9a7e9e424159a48b0227869245a95bf2ec2 \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/aot-raven-hello
```

The output directory must be new. Select a compatible Apple SDK as described below.
The script performs these separate compiler/linker steps and records their output:

```sh
dotnet /absolute/path/to/native-enabled/rvnc.dll neoclr \
  -o target/aot-raven-hello/RavenHello.dll docs/experiments/aot-hello/hello.rvn
tools/aot-poc/target/debug/neoclr-aot-poc \
  target/aot-raven-hello/RavenHello.dll @entry target/aot-raven-hello/hello.o --console
clang -arch arm64 -Wall -Wextra -Werror \
  docs/experiments/aot-hello/main.c docs/experiments/aot-scalar/console.c \
  target/aot-raven-hello/hello.o -o target/aot-raven-hello/hello
```

Raven's existing primitive/console bootstrap uses host reference metadata at compile
time; it emits native neoCLR metadata and instructions. Neither that compiler bootstrap
nor the .NET-hosted compiler becomes an executable dependency. Importing a complete
source-built System.Runtime dependency graph and compiling its general library bodies
remain later work. This profile recognizes only the bundled console wrapper described
below; no arbitrary .NET compatibility or complete class-library AOT is claimed.

`@entry` selects the exact entry name from decoded metadata. Raven's parameterless
`Main() -> Int32` is called by a generated adapter preserving `neoclr_entry_v2`;
the host ABI's old Int32 input is ignored, and status/result-pointer behavior is unchanged.
The tool also accepts standalone NEOX encoding of the same module, and retains explicit
named `(Int32) -> Int32` roots. Both binary transports go directly to the module model,
without JSON/disassembly round trips. Inputs are limited to 16 MiB; malformed binding,
envelope or metadata, absent roots and unsupported IL fail before object creation.
Ordinary CLR PE files without neoCLR's recognition marker are not accepted.

The checked-in [Raven-produced fixture](RavenHello.pe) retains native body/origin metadata
for tool tests without requiring Raven on every Rust test run. It was produced from
`hello.rvn` with the same pinned compiler. Its hash and the fresh end-to-end run's
artifact hashes are recorded separately; byte-identical Raven rebuilds are not asserted.
Tests execute both PE/#Neo and NEOX forms, compare with the interpreter, and check
corruption, stack/type errors, missing console capability and output preservation.
A parameterless entry Fault test checks status propagation and an untouched result slot.

## Original neoIL build and run

On macOS ARM64, from the repository root:

```sh
mkdir -p target/aot-hello
cargo run --locked --manifest-path tools/aot-poc/Cargo.toml -- \
  docs/experiments/aot-hello/hello.neoil Main target/aot-hello/hello.o --console
clang -arch arm64 -Wall -Wextra -Werror \
  docs/experiments/aot-hello/main.c \
  docs/experiments/aot-scalar/console.c \
  target/aot-hello/hello.o -o target/aot-hello/hello
target/aot-hello/hello
otool -L target/aot-hello/hello
```

Use a fresh object path when rebuilding; the tool refuses to overwrite an existing
artifact. The [scalar probe instructions](../aot-scalar/README.md#reproduce-on-macos-arm64)
record the matching SDK selection used on this machine. The linker consumes the
console implementation as source here; it becomes executable code in the final
image, not a shared runtime dependency. The compile tool and Apple toolchain are
build-time requirements only. Artifact creation still uses two explicit commands;
a supported single publish command remains future work.

## Contracts and evidence

`--console` enables the existing `neoCLR.Runtime.WriteLine(String)` service contract
and the exact static `System.Console.WriteLine(String)` wrapper in bundled System.
The wrapper must still be `ldarg 0; call neoCLR.Runtime.WriteLine(String); ret`, return
inhabited Void, and have no locals, generics or native import. The backend checks this
body before treating it as a console intrinsic; ordinary metadata resolution and typed
verification also run. Application replacements, different owners/overloads and arbitrary
external calls are rejected. This avoids silently skipping future wrapper behavior.
The native service implementation is linked into the image. Its native import is versioned as
`neoclr_console_write_line_utf8_v1(bytes, length)`, defined in the
[service header](../aot-scalar/console.h). Linking without that service fails.

String literals live in immutable object data as a private byte-count/UTF-8 payload.
Typed stack joins preserve literal references and inhabited Void separately from
Int32 values. This does not implement a managed String layout, allocation, generic
native imports or the full public System.Console library lowering. The host receives
a borrowed valid UTF-8 byte span, appends LF and flushes synchronously. It must not
retain or modify the bytes. Embedded NUL is an ordinary byte, not a terminator.
Any nonzero service result becomes experiment status 3 (`RuntimeError`), matching
the interpreter's console failure category. Partial output may remain after failure;
there is no output rollback. The caller's scalar result stays untouched on Fault.

Native code is position independent. The first data/service linking attempt exposed
illegal Mach-O text relocations under the previous default; enabling Cranelift PIC
allows the normal macOS executable link without weakening linker protection.

[Original neoIL validation](validation.json) records the exact compiler/runtime base and source and
artifact hashes. Twelve focused tests pass, retaining the existing scalar/control-
flow/Fault cases and adding:

- Hello World's exact stdout, empty stderr and zero exit from a directory containing
  only the executable, with an empty environment. Dependency inspection finds only
  `/usr/lib/libSystem.B.dylib`.
- Three interpreter/native byte comparisons with multilingual text, emoji, combining
  text, an empty line, embedded NUL and different literal control-flow paths.
- Missing capability, incompatible operand/stack types and absent service rejection.
- A failing service stub proving first-fault propagation and preserved output, plus
  the actual console implementation failing to flush a closed stdout descriptor.

These establish a small deployable native program, not throughput or release
qualification. GC, native execution budgets/cancellation, richer diagnostics, general
String/library lowering and HTTP Server remain future work.

## Progression toward HTTP Server

The author selects Hello World first, then increasingly complex samples until HTTP
Server. Use the existing scalar/Fault cases as supporting conformance tests rather
than separate product milestones. Raven-to-native Hello World now passes;
the author selects types and members next, especially value types enabling small
Result/Some union samples with payload access and branching. Inspect actual Raven
metadata/IL and add the smallest required representation/member contracts before
claiming general unions, managed memory or library support. A one-endpoint HTTP Server comes after its actual dependencies
compile and execute. Do not build a large benchmark suite or a new web framework as
prerequisites. Keep interpreter parity and explicit unsupported-feature diagnostics
as the platform grows; later benchmark .NET and other platforms with matched behavior.

Trimming is an author-identified future step. The current tool emits every declared
function and literal; this sample's small dependency set is not evidence of a trimmer.
Future removal must respect code roots, metadata retention, initialization, runtime
helpers and reflection/dynamic use. It is not a prerequisite for Hello World.

## Design comparison and remaining boundaries

[.NET Native AOT](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/)
compiles IL before execution and includes required runtime support. That deployment
property is the comparison target (source checked 2026-10-07), not an instruction to
consume ordinary .NET assemblies. Here the author explicitly selects Raven-produced
**neoCLR** metadata/IL. Reusing the native reader/verifier preserves platform semantics
and identity checks without another importer; the cost is the intentionally small
supported backend and explicit console lowering. A separate .NET CLI importer would
not validate this requested frontend-to-backend pipeline and is outside this slice.
The existing [native execution research](../../native-execution-investigation.md)
records the broader alternatives. No performance advantage is claimed.
