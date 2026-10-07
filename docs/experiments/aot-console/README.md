# Native Console progression (development, 2026-10-08)

The ordinary Raven `Console.ReadByte` wrapper now compiles to ARM64 together with
its Result/Option construction, generic seed helpers and failure path. Enable
`--compile-system --bind-user-fault --bind-console-read-byte` with an explicit
System seed and library load set. Inspection and emission use identical admission.
Only an exact, verified `neoCLR.Runtime.ConsoleReadByte() -> System.Value`
InternalCall is bound; managed functions bearing that name are rejected. Original
module/definition identities remain in the selection report. Both source-owned and
legacy seed services are supported; the public Console wrapper is never replaced.

`console.h` defines the experimental linked C input contract: 0–255 byte, -1 EOF,
-2 unavailable, -3 read failure. Codegen translates these outcomes to the existing
private erased transport; other results become an invalid Int32 status, reaching
Console's ordinary `System.Fail("invalid native I/O status")` path. The C adapter
uses stdin and distinguishes EOF from `ferror`. Embedding hosts may replace the
adapter, including returning unavailable. There is no global guest value buffer or
managed runtime dependency. Input is synchronous and can block; cancellation,
worker contexts and concurrent host service isolation are not provided by this POC.

The flag implies caller-owned fault ABI v3. Successful reads and expected errors
leave fault details clear; faults preserve managed wrapper/caller frames without a
synthetic input-service frame. The C startup/render helper comes from
[shared fault diagnostics](../aot-fault-details/README.md).

## Evidence

`verify.py` reuses the already compiled and provenance-recorded Raven
[read-byte.rvn](../aot-input/read-byte.rvn) / `ReadByte.pe` fixture. Its `let ... else`
and `if let` paths execute with byte 42, other bytes, NUL, 255 and EOF. Interpreter
and native exit results agree. Closed stdin exercises actual read failure;
replacement C adapters exercise unavailable, read failure and invalid statuses.
The executable runs alone with an empty environment and depends only on libSystem.
`validation.json` records inputs, hashes, commands and results; it is development
evidence against the pinned October 7 bundle, not release qualification.

```sh
SDKROOT="$(xcrun --sdk macosx --show-sdk-path)" \
python3 docs/experiments/aot-console/verify.py \
  --runtime target/debug/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --bundle /path/to/neoclr-native-poc --output /tmp/aot-console-input
```

Focused Rust tests cover all 256 bytes, distinct transport outcomes, invalid
statuses, opt-in requirements, duplicate options and attempts to bind managed bodies.

## Design and next slices

This implements the existing [Console contract and .NET comparison](../../console-io.md)
without changing its API. Like the established native Hello World experiment,
platform I/O is linked into the executable. Compared with .NET's runtime/library
implementation, this experiment retains neoCLR's explicit byte/EOF/error union
contract and compiles ordinary library code. The benefit is testing the real wrapper
without a managed runtime; the cost is an experimental platform adapter and limited
value/text support. No performance improvement is claimed.

Next: link WriteLine into the value backend so an interactive union consumer can
print prompts/outcomes. The broader Console class still needs dynamic text ownership,
numeric formatting, reference objects, arrays and stream dispatch for Write, ReadLine,
In/Out/Error and standard streams. The current literal-only String profile does not
provide these capabilities. These are implementation gaps, not permanent API limits.
