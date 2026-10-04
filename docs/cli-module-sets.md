# Module sets from the command line

All four commands accept repeatable `--module <input>` dependency arguments and one
optional `--system <input>` runtime library. Inputs ending in `.neoil` are sources;
other module inputs are JSON artifacts or recognized PE/#Neo and standalone NEOX
containers (feature-branch metadata support). The root of `assemble` is always treated as
source, preserving the existing command's behavior.

```sh
cargo run -- run examples/modules/app.neoil \
  --module examples/modules/operations.neoil \
  --module examples/modules/models.neoil

cargo run -- verify examples/revisions/app.neoil \
  --module examples/revisions/answers.neoil
```

The first command prints 42 and returns Void. `check` validates the complete supplied
metadata set. `verify` additionally runs the existing typed-stack analysis over all
IL functions. Neither activates native imports or executes guest code. `run` retains
the CLI's existing trusted native-execution contract and default resource limits.

## Explicit run instruction budget (development)

`run --instructions <positive-count>` overrides the existing per-execution
`Limits.instructions` value. The default remains 100,000; frame, stack, heap and
other limits are unchanged. The budget covers execution across the supplied module
set, not each assembly separately. It counts interpreter instructions, not elapsed
time, and does not interrupt native code. This exposes the existing bounded execution
control; it does not change CLI metadata or guest semantics.

    neoclr run App.dll --module NeoCLR.Collections.dll --system System.neox --instructions 1000000

The option is accepted once, before `--`, by `run` only. A positive decimal count
must fit the host's unsigned pointer-sized integer. Missing, zero, negative,
non-numeric, overflowing and repeated values reject before loading any input.
Arguments after `--` remain guest arguments. Unlike ordinary .NET application
execution, this interpreter already imposes an instruction quota; the flag makes
that existing host control explicit for larger samples. It is not a timeout or a
performance claim.

## Compile separate artifacts

Use a fresh output directory; assembly continues to refuse overwriting existing files.

```sh
mkdir out
cargo run -- assemble examples/modules/models.neoil out/models.neo.json
cargo run -- assemble examples/modules/operations.neoil out/operations.neo.json \
  --module out/models.neo.json
cargo run -- assemble examples/modules/app.neoil out/app.neo.json \
  --module out/operations.neo.json --module out/models.neo.json
cargo run -- run out/app.neo.json \
  --module out/operations.neo.json --module examples/modules/models.neoil
```

Source and artifact inputs can be mixed in either order. Cross-module field aliases
resolve against imported metadata, including legacy artifacts whose definition rows
are absent. `assemble` validates the whole set before creating its output, but writes
only the root artifact; it does not rewrite dependencies or serialize the linked image.
Scopes, references, and revision pins stay in the source artifact.

## Selecting System

`--system runtime/System.neoil` or `--system System.neo.json` supplies a self-contained
System module. It is used during initial application resolution, including compilation;
the application does not first need to resolve against bundled System. Missing or
mismatching revision pins therefore fail against the selected runtime library.

The previous `run <input> System.neo.json` positional form remains supported. It cannot
be combined with another System argument. Repeated System selections, missing option values,
unknown options, missing modules, and invalid load sets return a failure exit code.
Single-input System assembly/checking/verification remains supported without options.

## Library API and limits

The CLI uses `assembler::read_modules(&[ModuleInput::Source(...), ModuleInput::Json(...)],
system)` to validate mixed inputs while retaining separate Module artifacts, then
prepares a LoadedProgram. Existing `assemble_modules` and `load_modules` helpers use
the same reader with bundled System.

The caller must supply the entire dependency set. There is no search path, manifest
discovery, automatic rebuild, or file downloading. References, exact revision pins,
scope checks, unique-name restrictions, and entry-point rules follow the existing
[module-set contract](module-sets.md). CLI flags do not introduce a new module format
or alter execution and memory-management semantics.


## Direct native binary output (feature-branch development)

```sh
cargo run -- assemble examples/modules/models.neoil out/models.neox --format neox
cargo run -- assemble examples/modules/operations.neoil out/operations.neox \
  --format neox --module out/models.neox
cargo run -- assemble examples/modules/app.neoil out/app.neox --format neox \
  --module out/operations.neox --module out/models.neox
cargo run -- run out/app.neox --module out/operations.neox --module out/models.neox
```

`--format neox` serializes the root native Module directly to required execution schema 3.
There is no serialized JSON module intermediate, PE projection or dependency flattening. References,
definition identities and bodies retain the assembled module's values. A supplied
`--system` can also be binary; use the matching target profile for Raven-generated IL.

Before creating output, binary assembly requires typed verification of the complete
load set and checks schema-3 encoding limits. Invalid bodies/options or exceeded limits
leave no output file. The write still uses create-new semantics. This adds verification
work during assembly; it neither executes guest entry points nor claims faster assembly.
A later I/O failure can leave a partial file, as with the existing JSON writer.

The default and explicit `--format json` retain the existing JSON output/validation
behavior. Output extension does not select the format. Only one format option is
accepted, and it is available only for `assemble`. Standalone native assemblies require
a schema-3-capable runtime, with 8 MiB envelope and 2,097,152-item limits. Debug/source
text and a .NET compiler reference projection are not embedded.
