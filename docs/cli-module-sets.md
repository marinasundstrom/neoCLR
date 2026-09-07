# Module sets from the command line

All four commands accept repeatable `--module <input>` dependency arguments and one
optional `--system <input>` runtime library. Inputs ending in `.neoil` are sources;
other module inputs are JSON artifacts. The root of `assemble` is always treated as
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
