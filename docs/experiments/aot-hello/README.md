# ARM64 AOT Hello World

**First executable milestone, 2026-10-07 — development experiment on main.**
Compile [Hello World](hello.neoil) into ARM64 machine code, link the native startup
adapter and UTF-8 console implementation into the executable, and run that executable
by itself. It prints exactly `Hello, world!` followed by LF and exits with code zero.
No shared neoCLR/.NET framework, managed runtime, compiler, interpreter, source IL or
separate service library is needed beside the executable. The macOS system library
`libSystem` remains an OS dependency; this is not a freestanding binary.

The author's target is **CIL to a self-contained native executable**, with required
library code and runtime support baked in. This milestone proves native output and
linking with a **handwritten neoIL input**, not CIL/PE ingestion. The C startup adapter
and the unused Int32 Main parameter are temporary accommodations for the scalar
probe ABI. A supported CIL Hello World through the same verification/lowering/link
path is the next proposed input milestone, before broader sample expansion. No
arbitrary .NET assembly compatibility is claimed.

## Build and run

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

`--console` enables exactly the existing `neoCLR.Runtime.WriteLine(String)` service
contract in this bounded tool. Its native import is versioned as
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

[Validation](validation.json) records the exact compiler/runtime base and source and
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
String/library lowering, CIL AOT input and HTTP Server remain future work.

## Progression toward HTTP Server

The author selects Hello World first, then increasingly complex samples until HTTP
Server. Use the existing scalar/Fault cases as supporting conformance tests rather
than separate product milestones. Next validate Hello World from supported CIL;
then select the smallest consumers exercising values/managed memory and the required
library/services. A one-endpoint HTTP Server comes after its actual dependencies
compile and execute. Do not build a large benchmark suite or a new web framework as
prerequisites. Keep interpreter parity and explicit unsupported-feature diagnostics
as the platform grows; later benchmark .NET and other platforms with matched behavior.

Trimming is an author-identified future step. The current tool emits every declared
function and literal; this sample's small dependency set is not evidence of a trimmer.
Future removal must respect code roots, metadata retention, initialization, runtime
helpers and reflection/dynamic use. It is not a prerequisite for Hello World.
