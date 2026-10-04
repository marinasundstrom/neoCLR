# Native encoding layer after source-owned text (2026-10-04)

The seven unchanged encoding sources now compile into Encoding.dll against the emitted
75-source Numbers.dll text/numeric library. A third assembly imports both libraries with
no library sources and executes incremental UTF-8 and ASCII operations. This is a native
metadata gate, not a claim of full System or dual-target class-library completion.

## Fresh frontier and priority

[Post-text audit](system-post-text-audit-2026-10-04.json) records compiler 1bf03e88c and
runtime 9114fe4d, source/input hashes, commands and binding/emission diagnostics. The
75-source baseline passes; memory streams are already included (zero added sources),
and the 81-source Tasks/Concurrency combination passes compilation. The 166-source
attempt reports 379 errors, many cascading from absent bootstrap services. Counts are
not a count of missing VM implementations and do not establish execution.

The cumulative 82-source text attempt fails on String.SliceUtf8 because source-owned
String implementation signatures still bind through the smaller primitive bootstrap.
Building the seven additional encoding sources separately against native Numbers.dll
resolves that member without a projection or source changes. Keep the cumulative
source/bootstrap binding limitation explicit; separate native library composition is
also the intended acceptance boundary.

This reveals a portable emitter bug: TextEncoder.Drain assigns a match result to an
instance field; the failure arm reaches the terminal guard with the field receiver
still on the stack. Raven now spills reference-field receivers before RHS evaluation,
then reloads receiver and result for the store. Receiver-first evaluation happens once.
This adds temporary locals; no performance improvement is claimed. Managed value-type
receiver handling is unchanged. There is no metadata format, instruction, runtime or
Runtime Contract change and no new guest public API.

## Executable evidence

[Encoding evidence](source-encoding-native-2026-10-04.json) records the exact tested
compiler component, runtime, core, seed, library, source and consumer hashes, commands,
stdout and exit statuses. Its compiler revision names the pre-commit working-tree base;
Raven commit `7516a4902` contains the implementation.

- UTF-8 encoder drains into two-byte buffers; a separate decoder reconstructs Aé😀 across
  scalar boundaries. Seven encoded bytes and exact reconstructed text are checked.
- ASCII rejects unrepresentable text; an incomplete four-byte UTF-8 prefix is rejected.
- A separate field assignment consumer checks receiver/value evaluation order and
  once-only receiver evaluation (exit 42).
- A terminal failure in a field RHS verifies and executes with the expected message and
  exit 1; it does not publish an unverifiable assembly or continue to return 99.
- Twenty focused ordinary .NET emission tests pass, including Debug/Release field-order
  execution. Rebuilt Char/String, numeric and unchanged text sample acceptance passes.

Reproduce with a fresh output directory and the completed text bundle:

```sh
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_source_encoding.py \
  --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --core /path/to/TextCore.dll --seed /path/to/text/System.neox \
  --base-library /path/to/text/Numbers.dll --ownership /path/to/text/ownership.json \
  --output /tmp/native-encoding-fresh
```

The tool declares Encoding ownership explicitly, including its internal implementation
classes. The retained seed supplies runtime services, not duplicate text/encoding owners.
The .NET comparison is the same evaluation-order contract; encoding sources use neoCLR's
existing Unicode/UTF-8 contracts rather than System.Char's UTF-16 code-unit model.

## Next bounded work

Adding unchanged StreamReader, StreamWriter, TextReader, TextWriter and TextReadError
now reaches a portable BoundRequiredResultExpression statement rejection. Fix that
lowered-node handling and execute text streams over source-built MemoryStream before
expanding JSON document/value support. Ordinary cross-assembly class inheritance and
metadata-handle services remain independent blockers; the audit's inheritance probe
still rejects emission. No storage/network service expansion is justified by this gate.
The previously recorded stale full API snapshot remains open; no public signatures
changed here and no website build was needed.
