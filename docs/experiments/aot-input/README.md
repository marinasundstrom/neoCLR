# Native console-input prerequisites (2026-10-07)

The [outcomes sample](outcomes.rvn) compiles Raven to native neoCLR metadata/IL and
then a standalone macOS ARM64 executable. It uses the actual runtime library's
`Result<Option<byte>, ConsoleReadError>`: bytes 0, 128 and 255, EOF, Unavailable and
ReadFailed. Nested `if let` and `let ... else` distinguish the cases. This sample
constructs fixed outcomes; it does **not** read stdin.

[Validation evidence](validation.json) records the pinned compiler, runtime library,
source/artifact hashes, interpreter execution, matching AOT inspection/emission reports,
no object imports and execution with only the executable present and an empty environment.
The executable links only macOS libSystem. `Outcomes.pe` and `ReadByte.pe` are the
producer artifacts, not native executables.

## Metadata-only generic relationships

Option and Result implement different closed forms of Propagatable. Option also uses
Void as its error argument. The previous AOT load-set path tried to specialize these
metadata-only interface shapes as executable values and rejected the nested result.
Now the fully verified original relationships are removed from the private projection
**before** executable specialization. The report substitutes the owner's actual generic
arguments into each original relationship, preserving its constructed types, source
owner identity and metadata-only Void. Interface rows do not consume native shape limits.

Original runtime conformance verification still precedes this step. Executable generic
values still have the one-shape-per-definition limit; executable Void payloads, interface
storage/dispatch, explicit implementation mappings and selected generic methods remain
unsupported. This is bounded code selection, not general trimming or a metadata sidecar ABI.
Twelve focused load-set tests pass, including the new distinct-interface-shapes/Void
regression and existing invalid-conformance, generic-value and access-control checks.

## The actual read boundary

[read-byte.rvn](read-byte.rvn) calls the existing `Console.ReadByte()`. The pinned
interpreter returns exit 0 for EOF or byte 42 and exit 2 for another byte (tested with
`x`). AOT inspection/emission currently reject selection of the reference-type static
Console owner. The script asserts this exact boundary and absence of an object.

Removing that first restriction alone will not supply native input. The existing
[console contract](../../console-io.md) implements the public method in Raven; its
runtime service transports Byte, Void or Int32 status through erased `System.Value`.
The wrapper calls generic value-test/unpack services. These are further native backend
requirements, not permission to replace the public method by name with a host intrinsic.

The next bounded task is metadata-only static member owners, followed by inventory and
an explicit native input capability/service contract. Service tests must distinguish EOF,
zero/high bytes, unavailable capability and I/O failure, including interrupted reads.
An input-driven ASCII integer parser can consume bytes without allocating strings;
UTF-8 line decoding and text ownership remain subsequent work.

## Comparison and provisional direction

This reuses the platform's existing input contract rather than adding a new public API.
.NET's [Stream.ReadByte](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream.readbyte?view=netframework-4.8.1)
returns an Int32 with -1 for EOF. Rust's [Read](https://doc.rust-lang.org/std/io/trait.Read.html)
separates I/O errors from counts, with zero indicating EOF for a nonempty buffer
(primary API documentation reviewed 2026-10-07). neoCLR's existing nested union makes
byte/absence/failure explicit in the type; its cost here is nested layout and generic
specialization. No speed or allocation advantage over those platforms is claimed.

A byte service needs no retained guest buffer: it returns a copied scalar. Starting
there postpones the choice among bounded owned text buffers, tracing and reference
counting without selecting any of them. Compiling the ordinary wrapper preserves its
status-to-union semantics, but requires more backend work than a hard-coded Console
intrinsic. A versioned, explicitly enabled host service linked into the executable is
preferred provisionally; its ABI, binding validation and failure behavior are still to
be implemented and tested. General .NET-style stream objects and native managed text
would address broader scenarios but are unnecessary for the first byte-input consumer.

## Reproduce

Set SDKROOT to the matching Xcode SDK on macOS ARM64, then run:

```sh
python3 docs/experiments/aot-input/verify.py \
  --compiler /absolute/path/to/rvnc.dll \
  --compiler-revision 70aea9a7e9e424159a48b0227869245a95bf2ec2 \
  --bundle /absolute/path/to/neoclr-native-poc \
  --runtime /absolute/path/to/neoclr \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/aot-input-outcomes
```

The output directory must not exist. Producer fixtures use the same pinned bundle as
the [library Result experiment](../aot-library/README.md); no shared framework/runtime
is needed to execute the resulting native outcomes app.
