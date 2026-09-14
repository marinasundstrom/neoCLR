# Runtime and Raven preview

Preview 6 is [available on GitHub](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.6). This
release has two entry points: neoCLR directly through neoIL, and the experimental
Raven toolchain compiling programs against neoCLR's own runtime library.

The purpose is to demonstrate that the platform's Result/Option error flow, generic
Void and runtime APIs can be used from an existing CLR language. Ordinary classes,
arrays and values follow the familiar reference/value categories. Raven remains on
its experimental branch; its normal .NET target is a separate product path.

## What the distribution contains

| Asset | Purpose |
| --- | --- |
| neoCLR runtime bundle | Interpreter, adapted System library, neoIL samples, Raven examples, declaration metadata, compiler bridge, language server, documentation and notices |
| Experimental Raven SDK archive | Matching compiler/editor toolchain, installed alongside an ordinary Raven SDK |
| Experimental Raven VSIX | Matching VS Code extension for target completion and hover |
| Source archive and provenance | Exact neoCLR/Raven revisions, prerequisite versions, asset checksums, validation results and documented limitations |

The current tested binary host is macOS arm64. The Raven tooling requires the .NET
11 preview recorded in the manifest; direct execution by the Rust runtime does not.
The saved Raven project tasks additionally require Python 3.9 or later. Do not infer
other binary platforms from neoCLR's source-build portability.

## Start with the runtime

From the extracted runtime bundle:

```sh
./bin/neoclr verify samples/neoil/type-categories.neoil --system lib/System.neoil
./bin/neoclr run samples/neoil/type-categories.neoil --system lib/System.neoil
./bin/neoclr run samples/neoil/result-void.neoil --system lib/System.neoil
```

`type-categories.neoil` prints `42`, `7`, `9`: two class variables share one object,
while copying a value leaves the original unchanged. `result-void.neoil` prints
`Completed`, `Not saved`: completion is a Result containing Void, and a recoverable
failure is an error payload. The sample uses branches and carrier APIs; Raven's
propagation syntax performs the corresponding control-flow lowering.

neoIL is the prototype's textual instruction format. It is not a claim that every
spelling is standard MSIL: `.type class`, plain value declarations and `ldvoid` expose prototype conventions. Lowercase
`void` denotes ordinary no-result returns; `System.Void` names the unit type. Ordinary void-returning CLI signatures still
have no stack result. The importer translates the admitted CLI subset into this
runtime representation; the interpreter does not directly execute arbitrary .NET PE
files. See [format direction](format-direction.md) and [Void mapping](void-semantics.md).

## Continue with Raven

Follow the bundle README to install the matching VSIX, run `configure.py`, and open
**demo** in VS Code. The initial `Main.rvn` demonstrates collection iteration and
Result/Option/Void propagation. Select **neoCLR: Run saved project** from Tasks.
The ordinary Raven toolbar build/run/debug buttons do not implement this pipeline.

Replace `demo/Main.rvn` with a sample from `tools/samples`, save, and rerun:

| Sample | What it demonstrates |
| --- | --- |
| `library-propagation-workflow.rvn` | Error/absence propagation and successful Void completion |
| `library-collection-aliases.rvn` | Collection reference identity and shared mutation |
| `library-files.rvn` | Bounded text-file APIs with typed failures; creates a demo file |
| `library-calendar.rvn` | Separate Date and Time values and their validation |
| `library-clock.rvn` | Host local date/time |
| `library-reflection.rvn` | Type/member introspection |
| `library-value-interfaces.rvn` | Interface dispatch, including boxed value copies |
| `library-reference-payloads.rvn` | Nested and reference-bearing collection/union payloads |

Completion resolves the supplied neoCLR declaration metadata. Execution compiles the
saved project through the matching Raven compiler, imports its admitted IL, verifies
it and runs neoCLR's library. It does not fall back to executing the program on .NET.
Both the declaration catalog and bounded importer remain experimental infrastructure.

## What this preview establishes

The [runtime API coverage](raven-runtime-api-coverage.md) records the existing library
surface and adaptations, including enum BindingFlags and reference-based collection
storage. The [match matrix](raven-match-matrix.md) distinguishes tested expression
and statement forms from unsupported patterns. API contracts are provisional and
feedback is welcome; familiar names do not promise complete .NET compatibility.

General application class hierarchies and interface implementations, capturing
closures, unrestricted reflection, nullable metadata and fault-unwind cleanup are
not part of this importer. Terminal faults are not catchable guest exception objects.
Native buffers retain explicit allocation/free and bounded layout support.

## Earlier Neo experiment

The Neo concept language remains available in the source repository under
`examples/source` and its [historical language guide](neo.md). It tested the earlier
value-by-default and explicit-reference model. It is outside this release's primary
walkthrough and is not the language used to demonstrate the current Raven integration.
Its existing sources and published release notes remain preserved as development
history; new Raven work does not require migrating that frontend.

## Validation and release status

The Preview 6 release carries [exact-source CI evidence and package validation logs](preview-6-validation.md),
including application classes, order persistence and deferred queries. Use its
manifest/checksums for exact revisions and artifacts. Source CI does not imply
prebuilt Raven toolchain support on other hosts. The
[release procedure](experiments/raven-target/RELEASING.md) describes repeat builds.
