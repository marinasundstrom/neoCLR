# Raven text files on neoCLR

The experimental target now projects the existing synchronous, bounded UTF-8 APIs:

```text
File.ReadAllText(string path, int maxBytes) -> Result<string, FileReadError>
File.WriteAllText(string path, string text, int maxBytes) -> Result<Void, FileWriteError>
```

Use `import System.IO.*`. The [sample](experiments/raven-target/samples/library-files.rvn)
propagates read/write errors with `?`, matches typed Result cases, and prints the
returned text. It creates or replaces `neoclr-file-demo.txt` in the process working
directory. Its second write exceeds the byte limit and leaves the first contents
intact. Empty text is a successful zero-byte write; success carries Void, not a
nullable payload.

The full contracts and .NET comparisons remain in [file input](file-input.md) and
[file output](file-output.md): familiar File names, explicit byte bounds and Result
errors instead of exceptions. This projection uses those runtime implementations;
metadata method bodies never execute and host .NET File is not substituted.
The existing caveats around OS error classification, synchronous access, non-atomic
writes and failure after opening still apply.

## Current projection boundary

Both error types expose their runtime `Is…` case predicates. Read errors include
InvalidLimit, InvalidPath, NotFound, AccessDenied, NotRegularFile, ReadFailed,
TooLarge and InvalidUtf8. Write errors replace ReadFailed with WriteFailed and have
no InvalidUtf8 case. `Result.Error<FileReadError>.Value` and the corresponding write
case expose the error for those predicates; `Result.Ok<string>.Value` exposes text.
Full error-union construction, case deconstruction and ToString are not yet admitted
by this slice. They remain part of the [existing-API release coverage gate](raven-preview-acceptance.md).

Result extraction and propagation call the runtime's conditional-output contracts:
an output is initialized only when extraction returns true. The importer requires an
immediate test (allowing Nop) and verifies assignment on the successful branch.
It also recognizes Raven's exact pattern-test Boolean diamond so typed Result matches
carry that proof into the arm. Storing the Boolean for a later test is not yet
supported. Ignoring the test or reading an uninitialized error is rejected; the bridge
does not invent a default file-error case to satisfy CLI `out` assumptions.

Named Void inside generic returns is preserved separately from the CLI void return
marker. Raven's metadata loader retains generic/storage type identity while ordinary
void returns remain Unit in its semantic model. This fix is on the isolated Raven
experiment branch at `0fad44881`. Existing installed tools must be refreshed before claiming the
new file sample works through those packages.

## Repeatable checks

Use a built Raven checkout with the metadata Void fix and a built neoCLR executable.
Set `RAVEN_ROOT` and `NEOCLR_RUNTIME` to their absolute paths. From the neoCLR root,
choose a fresh output directory (the probe refuses to overwrite an existing one):

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot="$RAVEN_ROOT" -p:BuildProjectReferences=false -p:WarningLevel=0 \
  -- --files /tmp/neoclr-files-check
python3 docs/experiments/raven-target/verify_files.py /tmp/neoclr-files-check \
  --runtime "$NEOCLR_RUNTIME" --system "$NEOCLR_SYSTEM_LIBRARY"
```

The checker runs the generated programs in a temporary directory, verifies UTF-8
round-trip bytes and preservation after preflight rejection, and tests missing input,
invalid UTF-8, invalid limits and empty text. The probe also rejects ignored/inverted
extraction tests and uninitialized error reads without producing executable output.

For saved projects, regenerate the target declarations using the current probe and
use the neoCLR build/run task with this sample as Main.rvn. Do not run the declarations
on .NET. [Editor setup](experiments/raven-target/VSCODE.md) describes the compiler/server setup and remaining distribution limits.

The direct file probe now uses the same target core, unit and managed-array profile
as saved projects. Set NEOCLR_SYSTEM_LIBRARY to the matching generated Raven System
library when running its fixtures.
