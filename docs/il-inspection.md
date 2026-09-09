# Inspecting generated Neo IL

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

A future disassemble command needs a metadata-to-text writer, including instruction
operands, signatures and source maps, with round-trip validation. Binary encoding is
an independent future format choice; readable IL inspection does not depend on it.
The command name deliberately reflects the implemented source-emission behavior.

## Validation

```sh
cargo test --locked --test emit_il --test cli --test cli_modules
```

Tests check clean stdout, exact file output, enum metadata and receiver instructions,
source paths, reassembly/execution equivalence, no-overwrite behavior, invalid-input
handling and successful emission of a program that would fault if executed.
