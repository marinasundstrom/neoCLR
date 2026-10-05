# Inspecting Neo IL and compiled metadata

Use emit-il to inspect the compiler's textual lowering without executing a program:

```sh
cargo run --locked -- emit-il examples/source/enums.neo
cargo run --locked -- emit-il examples/source/generic-receivers.neo /tmp/receivers.neoil
```

Without an output path, stdout contains only reassemblable neoIL. Errors go to stderr
and return a nonzero exit code. With a path, the command writes the IL to a new file
and prints a confirmation. Existing files are never overwritten, including the input.
Compilation and verification complete before the output file is created.

The command accepts one .neo input using bundled System, just like current Neo
compilation. It does not accept module/system overrides, guest arguments, stdin or
JSON/neoIL inputs. It preserves the exact lowering, including generated types and
methods, local declarations, labels and source sequence points. Bundled System methods
are referenced rather than copied into the emitted text.

The source document recorded in sequence points is the original input path. Keep that
source available at the recorded path when debugging; a relative path is interpreted
from the debugger's working directory. Emitting IL does not embed the source file.

## Reassemble, verify and debug

```sh
cargo run --locked -- emit-il examples/source/generic-receivers.neo /tmp/receivers.neoil
cargo run --locked -- assemble /tmp/receivers.neoil /tmp/receivers.neo.json
cargo run --locked -- verify /tmp/receivers.neo.json
cargo run --locked -- debug /tmp/receivers.neo.json
```

Choose fresh output paths when repeating the commands. You can also run or debug the
emitted .neoil directly. See [the debugger](debugger.md) for stepping and memory views,
and [neoIL](neoil.md) for instruction semantics.

Emission performs the same assembly/linking and typed verification as ordinary Neo
compilation. It does not execute Main, evaluate runtime calls or invoke native code.
A program that would fault at runtime can still emit successfully. Conversely, this
command does not dump partially generated IL when compilation or verification fails.
Host tooling can use frontend::lower_to_il_named directly when investigating a verifier
failure in compiler development.

## Scope and CLR comparison

This is source-to-IL emission, not artifact disassembly. The existing
[Ildasm comparison](neo-roadmap.md#immediate-union-and-inspection-work) describes .NET's
compiled-artifact-to-text workflow. This first Neo command exposes compiler decisions
using the already available textual lowering; it adds no metadata, opcodes or binary
format. The benefit is a small debugging tool that produces reassemblable input. The
limitation is that it cannot inspect an arbitrary existing artifact or reproduce the
original lowering from JSON alone.

## Disassemble compiled native metadata (development)

```sh
cargo run --locked --release -- disassemble tests/fixtures/metadata-container/raven-source-object-root.pe
cargo run --locked --release -- disassemble tests/fixtures/metadata-container/models.neox /tmp/models.disassembly.txt
```

`neoclr disassemble <metadata-input> [output]` reads PE files containing `#Neo`,
standalone NEOX envelopes, or format-5 native JSON. Detection uses the file contents.
It uses the runtime's existing container decoders without constructing a load context,
resolving dependencies, selecting an Object root, verifying bodies or executing code.
An unresolved reference or invalid branch target can therefore still be inspected.
Malformed encodings and unsupported formats fail before creating output. An ordinary
.NET PE without native metadata is not supported.

The listing contains the decoded module metadata, including assembly/dependency
identities, types, fields, relationships and origins. Each `.method #N` contains the
function's metadata (signatures, owners, declaration flags, generic scopes, locals and
source maps where present), followed by indexed instructions and their serialized
operands. Metadata and operands use the canonical native model's JSON spelling.
`I_0000` labels are **decimal instruction indices**, matching native branch operands;
they are not CLI byte offsets. Function indices are decoded module positions, not CLI
MethodDef tokens. All serialized declaration facts are retained; absent optional facts
retain the model's defaults. This does not dump PE sections or the CLI projection.

The output is a deterministic **diagnostic listing, not reassemblable neoIL** or a
source decompiler. Unlike the reassembly workflow in the Ildasm comparison above, this
first implementation prioritizes visibility into native metadata and semantic operands.
It adds no encoding or opcode schema; richer text formatting and reassembly remain
future work. Reusing decoders keeps inspection aligned with runtime decoding, at the
cost of being unable to inspect a container rejected by those decoders.

Without an output path, stdout contains only the listing. With a path, it creates a new
file and prints confirmation. Existing files (including the input) are never overwritten.
Errors use stderr and a nonzero exit status. No `--system`, dependency overrides, source
inputs or guest arguments are accepted. This development CLI command adds no guest API
or metadata-library public API, and does not change compiler or runtime configuration.

## Validation

```sh
cargo test --locked --release --test disassemble --test emit_il --test cli
```

Tests check clean stdout, exact file output, enum metadata and receiver instructions,
source paths, reassembly/execution equivalence, no-overwrite behavior, invalid-input
handling and successful emission of a program that would fault if executed.

Disassembly tests cover the executed Raven source-root PE, generic library facts,
equivalent PE/NEOX/JSON listings, unresolved exact dependencies and invalid bodies,
malformed/unsupported inputs, stdout/file equivalence and no-overwrite behavior.

For bootstrap execution after inspection, see [explicit Object-root loading](raven-cli-bridge.md#explicit-object-root-cli-loading-development-2026-10-05).
Disassembly itself still needs no root selection or dependencies.
