# Source-built native text streams (2026-10-04)

Five unchanged sources—StreamReader, StreamWriter, TextReader, TextWriter and
TextReadError—now compile to TextStreams.dll. An independently compiled consumer
imports that assembly, Encoding.dll and the source-owned text/collection Numbers.dll;
none of their sources are included in the consumer invocation. Native metadata import
and the explicit ownership catalog are used throughout. No seed service, runtime,
metadata format, public signature or instruction change was needed.

## Compiler fix

Match lowering wraps early returns and discarded blocks in required-result expressions.
The portable body adapter now exposes these at statement boundaries and normalizes
return expressions to return statements. For a local initializer whose lowered block
has no disposal/fixed storage, the prefix statements precede the final initialization.
This preserves evaluation order and allows a return to exit the method, rather than
continuing to evaluate the initializer or rejecting the entire method.

A value block inside a larger expression may have earlier operands on the evaluation
stack. That case remains rejected; the exit scanner now also recognizes wrapped
returns and initializer blocks. No blanket relaxation of verifier requirements or
rewriting of runtime-library sources is involved. The .NET comparison is ordinary
method-return control flow; the existing Reflection/Emit backend remains in place.

## Validation

[Recorded commands and hashes](source-text-streams-native-2026-10-04.json) include the
native runtime, core, seed, compiler components, source dependencies and emitted
artifacts. Revision fields identify working-tree bases; binary hashes identify the
actual tested compiler. Raven commit `354d0bf4d` implements the fix.

The text-stream consumer exits 42 with no stdout and verifies:

- Aé😀 plus newline writes exactly eight UTF-8 bytes and reads back through ReadLine.
- EOF yields None; seeking the same MemoryStream and ReadToEnd reproduces exact text.
- Closing leave-open readers/writers preserves the shared underlying stream.
- A two-byte read bound produces LimitExceeded through ReadPart's early return.
- An incomplete UTF-8 scalar produces InvalidUtf8; closing the owning reader closes
  its underlying MemoryStream.

The smaller match-initializer consumer checks both normal-result and early-return
paths, exiting 42. The nested-argument return regression rejects before any output is
published. The prior source encoding gate (UTF-8/ASCII, field order and failure) passes
again. All 22 focused ordinary .NET EmissionCapabilityTests pass and cover the corresponding observable
match-return paths alongside existing portable-emission controls.

Reproduce against the previous completed text and encoding bundles:

```sh
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_source_text_streams.py \
  --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --core /path/to/TextCore.dll --seed /path/to/text/System.neox \
  --base-library /path/to/text/Numbers.dll --encoding-library /path/to/encoding/Encoding.dll \
  --ownership /path/to/encoding/ownership.json --output /tmp/native-text-streams-fresh
```

The tool extends the explicit ownership manifest with all five stream declarations.
The test asserts existing APIs, not new behavior; public API snapshots require no
signature update. The previously recorded stale full snapshot remains open. Website
content is updated without an unrelated website build.

## Next high-leverage blocker

A focused unchanged JSON source set (JsonDocument, JsonSyntax, JsonValue, JsonError)
first needs the standalone Runtime.Reflection.ReflectionError union for JsonError's
payload signature. Including that source resolves binding without adding introspection
services. Emission then rejects DocumentReader.Value's Result<JsonValue, JsonError>
callable signature. JsonValue is the sealed reference hierarchy used by all document
nodes; the current portable nominal model does not admit it.

Next establish class hierarchy representation and admission with small base/derived
and separate-library consumers, then return to this focused document gate. JsonSerializer
also exposes object-mapping methods and has additional dependencies; this gate does not
claim those or full JSON execution. Full rebuilt-System dual-target parity remains open.
