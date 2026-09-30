# Existing Raven sample translation experiment

Feature-branch development, 2026-09-30. This exercises the **existing CLI bridge**:
Raven source → projected CLI PE → imported neoil → assembled format-5 JSON → native
NEOX. It does not extend the new direct compiler emitter or provide a native symbol
loader. The independently built .NET translator supplies the same schema-2 binary
transport tested by the C# contracts.

The author requests a proper neoil assembler capable of producing native assemblies
as a future producer, and asks to explore translation of existing Raven test/sample
output now. Both producers should ultimately share metadata encoding rather than
maintaining unrelated assembly formats. A direct assembler output command is not
implemented by this experiment.

## Reproduce

From the neoCLR feature worktree, build the bridge against the selected Raven checkout:

```sh
dotnet build docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot=/absolute/path/to/Raven -p:WarningLevel=0 \
  -o target/extended-cli-metadata/sample-bridge
dotnet build tools/metadata/NeoCLR.Metadata.Translate
cargo build --release --bin neoclr
python3 docs/experiments/raven-target/verify_metadata_translation.py \
  --bridge target/extended-cli-metadata/sample-bridge/Probe.dll \
  --translator tools/metadata/NeoCLR.Metadata.Translate/bin/Debug/net10.0/NeoCLR.Metadata.Translate.dll \
  --runtime target/release/neoclr \
  --output target/extended-cli-metadata/raven-sample-translation
```

Use a fresh output directory. `--only Basics Strings FloatingMath ValueCopy` selects
a focused rerun while retaining all compile-rejection checks. `--system-json /path/to/System.json` can reuse an
already assembled matching **raven-collections** profile; it is not interchangeable
with bundled System. The default builds that profile from the current source manifest.
Reports include hashes of the runtime, compiler, bridge, translator and metadata DLL.
The recorded run uses Raven `codex/metadata-consumer` at `1ba9eaa8b9d7d9101ed5408bead1299c17043b94`
and neoCLR `codex/extended-cli-metadata` based on `6e1ad1df`, with this test-only slice.
A local macOS SDK override used SDK 26.2 to build the release runtime, as in the prior
binary-loading experiment. No global SDK configuration changed.

The harness verifies and executes each admitted application as neoil, JSON and binary,
using the same expected output from the existing match/project checks. It distinguishes
compiler/importer, source, JSON, translation and binary failures and saves the report
even when a case fails. Nonzero exit status also reports unsupported System translation;
a JSON-System fallback is explicit in the report and is not an all-binary success.

## Boundaries exposed

The Raven collection profile has 417 types and 4,090 functions, compared with 117 types
and 743 functions in the bundled System artifact from the previous slice. Its JSON is
15,017,185 bytes (7,104,628 compacted); schema 2 currently permits only 4 MiB input JSON
and 1 MiB envelopes. It also contains 846,454 encoding items including map keys,
exceeding the current 262,144-item guard. Removing whitespace alone cannot admit it.
The experiment retains those guards and runs application translations with the exact
same JSON System. This is a transport-bound failure before native admission, not a failing class-library
method. A larger supported profile or indexed representation needs an explicit
contract and cross-reader boundary tests before this library can become binary.

The existing OptionPositional fixture used obsolete explicit carrier construction and
failed in the compiler before transport. It now uses imported `Some(42)`/`None()` case
construction and retains the expected 42/-1 behavior. The wrong-arity expectation now
matches the compiler's RAV2106 diagnostic. Neither adjustment changes compiler semantics.

Project fixtures explicitly select the same iteration/collection Runtime Contract as
prepare_editor.py. The first project run omitted that selection and produced
bundled-system applications; pairing those with collection System failed before
translation. The corrected harness checks RequiredLibraryProfile and reruns the four
projects. Unchanged successful match evidence is reused.

Like .NET's separation of language compilation, IL assembly and runtime metadata loading,
these are distinct stages; compare the ECMA-335/reference-assembly baseline already
recorded in the [metadata design](../../design/extended-cli-metadata.md). Native value
preservation and equal outputs establish transport regressions, not independent proof
of the legacy compiler/importer or coverage of every Raven construct. No timing or
performance conclusion is drawn from this correctness run.


The floating-math sample passes source and JSON execution but schema-2 translation
rejects it with `schema 2 requires signed 64-bit integers`. This is not a floating
arithmetic failure: `ldc.r8` stores its exact IEEE-754 bits as a native UInt64 operand.
Negative double constants have bit patterns above Int64.MaxValue, outside schema 2's
numeric subset. For example, the sample's -1.0 operand is 13830554455654793216.
A future execution profile needs full UInt64 value preservation (including signed zero
and NaN bit patterns where supported); wrapping into a signed JSON integer would change
the native operand contract. Keep this rejection visible until both readers/writers and
runtime admission share an explicitly tested encoding.


## Recorded result

[Machine-readable evidence](raven-sample-translation.json) combines unchanged match
results with the focused four-project rerun; tool and System hashes match across both.
Fourteen of fifteen applications verify/run with their expected output after binary
translation: all eleven positive match cases, Basics, Strings and ValueCopy. Six
negative compiler cases retain their expected diagnostics. FloatingMath passes source
and JSON but fails translation as described above. Every binary application run uses
matching **JSON System**; this is not an all-binary Raven class-library claim. The
harness intentionally exits nonzero for the unsupported library and floating operand.

The next format work is explicit capacity and UInt64 coverage with cross-reader tests;
the future neoil producer and direct class-library compiler should reuse that contract.
The wider Raven sample suite, arrays/delegates/async workloads and native symbol import
remain untested by this bounded matrix.
