# Production BindingFlags metadata — 2026-10-05

The unchanged System.Introspection.BindingFlags source now compiles as a native library.
An artifact-only consumer combines NonPublic, Static and DeclaredOnly and returns 42.
This also closes the portable emission gap for Int32 enum bitwise and/or/xor: operations
use the existing enum-to-storage conversion, integer opcode and enum reconstruction.

## Metadata and target boundaries

The writer uses standard core FlagsAttribute in CLI metadata and the pre-existing
native enum-info flags classification used by runtime enums. No new category or format
version is introduced. Definition setters and manually authored attributes share
validation; duplicate or malformed core markers reject. Readers/introspection expose
IsFlagsEnum. Native raw attributes do not invent an executable marker constructor.
CLI reference projection recreates the marker; Raven projects the facade fact into a
semantic AttributeData referring to the configured core's actual FlagsAttribute.
There is no loader/emitter object reuse or new default .NET behavior.

This reuses the established CLR attribute contract and existing native enum design,
with no claim of a new semantic improvement. Generic enums/wider enum storage remain
unsupported by this writer profile. No guest API changed; the host API reference is
updated. The guest documentation snapshot check retains its pre-existing stale status.

## Reproduction and evidence

```sh
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_flags.py \
 --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
 --core /path/to/Core.dll --seed /path/to/System.neox \
 --ownership /path/to/ownership.json --base-library /path/to/Numbers.dll \
 --output /tmp/flags-gate
```

150 C# metadata groups pass, including definition/builder/manual-attribute equivalence,
ordinary CLI execution, native round trips, facade facts and invalid marker rejection.
Raven's C# NeoClrMetadataProbe --flags-symbols Core.dll verifies that native import
exposes the configured semantic attribute. [Execution commands/hashes](flags-enums-evidence-2026-10-05.json).

The full descriptor/JSON build now reaches the params-array declarations in the source
reflection extensions. Parameter-array metadata/import remains the next blocker; JSON
object mapping is not complete.

Twelve focused ordinary .NET EnumCodeGenTests pass.
