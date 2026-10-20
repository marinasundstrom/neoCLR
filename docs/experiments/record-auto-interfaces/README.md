# Automatic native record equality interfaces — 2026-10-20

The compiler now synthesizes the selected target's ordinary self-equality interface
for supported nongeneric positional records. Raven source does not need an explicit
base-interface clause. This fixes the missing semantic conversion and native metadata
contract, rather than adding a special record runtime representation or dispatch path.

## Implementation and .NET boundary

The existing target runtime contract supplies the record equality interface identity:
.NET retains System.IEquatable<T>, and neoCLR selects its existing EquatableTo<T>.
Binding constructs that interface with the record's own type; normal type emission
writes the interface relationship and existing generated Equals satisfies it.
Native component admission now resolves the same contract instead of independently
hard-coding its name. No new project setting, CLI marker, runtime instruction, layout,
calling convention, explicit interface workaround or hand-written key equality body
is needed. This is a neoCLR integration fix on Raven's native compiler branch;
ordinary .NET generic record synthesis is unchanged and separately tested.

ConsoleKeyInfo consequently implements EquatableTo<ConsoleKeyInfo> automatically.
It is usable through that interface and as an imported component of another supported
record, reusing its typed Equals and hash implementation. The application fixture
also checks source record structs and classes through their generated interfaces.
Records remain ordinary classes/structs with generated members/interfaces, matching
the author's [runtime and ABI boundary](../../design-research.md). The EquatableTo
name is the core's existing directional-interface naming choice, not a new execution
semantic. Future native metadata/codegen must preserve ordinary self-typed interface
identity and dispatch rather than introduce a record ABI.

## Compatibility and remaining gaps

Nongeneric records with unsupported component equality now fail at record admission,
even if the application only constructs/stores them. Previously the native backend
could silently emit storage without the expected record equality contract. The
component-contract diagnostic remains necessary; it now protects the automatically
synthesized interface as well as explicitly declared interfaces.

Native generic record equality/hash bodies are still unsupported. The target exposes
that capability limit explicitly and does not attach a generated interface without
its implementation. Existing generic storage-only records, including KeyValuePair,
retain their documented limited surface. This is an open backend gap, not the intended
record semantics. Complete generic record support must supply both the interface and
its methods; no new fallback based on Object identity has been added. Enum formatting
and other previously recorded component/display gaps remain separate work.

The explicit-interface experiment is retained as historical evidence and regression
coverage, not a recommended workaround. Its automatic-synthesis TODO is superseded
here. The old imported ConsoleKeyInfo rejection is replaced by positive coverage;
the missing-contract regression now uses the ordinary OverflowError struct. Old
baseline/result files retain their original cohort identity. Diagnostic scripts
accept both the historical and current targeted rejection messages.

## Validation

[baseline.txt](baseline.txt) retains RAV1504 for implicit source and imported record
interface conversions with the prior compiler. [validation.json](validation.json)
records interpreter/JIT/AOT results and exact artifact hashes. The fresh compiler
source-core probe checks automatic source interfaces, consumer interface calls,
imported value contracts, unsupported ordinary/union components and the retained
generic limitation. The focused .NET record suite passes 34 cases; two explicit
generic class/struct IEquatable identity cases also pass independently. Logs are
retained alongside this file.

The tested compiler is labeled `0c7ee885c-dirty-record-auto-interfaces`, built before
the commit. Five source libraries were rebuilt. The final execution bundle and API audit both use the v4 library rebuild.
An earlier documentation-only rebuild changed artifact hashes, so the final
consumer was rebuilt and revalidated against the exact documented library cohort. Public references
and the native API snapshot are refreshed from matching declarations. Explicit
interface conversion can box value records as usual; this change makes no new
allocation or throughput claim. Replacement release-package validation remains open.

```sh
python3 docs/experiments/record-auto-interfaces/check.py target/record-auto-interface-bundle-v2/bundle target/record-auto-interface-app-v2 --output docs/experiments/record-auto-interfaces/validation.json
```

Compiler source committed as Raven `6f7f32901`; tested precommit artifacts retain
their original dirty label and exact hashes. The final website build checked 4,898
pages successfully; publication was not performed.
