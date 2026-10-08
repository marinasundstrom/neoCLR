# Bounded UTF-8 file output

Implemented 2026-09-08:
`System.Storage.File.WriteAllText(String path, String text, Int32 maxBytes) -> Result<Void,FileWriteError>`.

Writes valid UTF-8 without adding a BOM or newline, creating a regular file or
truncating an existing file. An empty string creates/truncates to an empty file.
Embedded NUL characters in contents are preserved. Parent directories must exist;
relative paths use the host current directory. Symlinks are followed by ordinary
host open semantics. This is synchronous host I/O.

The byte limit is explicit, matching ReadAllText's bounded contract. A negative
limit is InvalidLimit; an empty or NUL-containing path is InvalidPath; content whose
UTF-8 byte length exceeds the limit is TooLarge. These checks happen before opening
the destination, so they cannot truncate an existing file. Character counts are not
byte counts. The limit bounds requested output, not the already-created String's
memory usage.

Other FileWriteError cases are NotFound, AccessDenied, NotRegularFile and WriteFailed.
The error is an ordinary union with case predicates, extraction and ToString. OS
classification can differ: a directory open may report AccessDenied or NotRegularFile.
A handle's regular-file status is checked before truncation. Special-file opens can
still block under host semantics; this API is not for pipes/devices.

Successful return means the write and flush completed and the handle was released.
It does not guarantee durable storage. Failure after opening may leave a newly
created, truncated or partially written file; this is not atomic replacement. An
atomic-save API would require its own contract. Cancellation cannot interrupt an
in-progress blocking native call.

## .NET comparison and layers

The shipped [.NET File.WriteAllText](https://learn.microsoft.com/en-us/dotnet/api/system.io.file.writealltext?view=net-10.0)
creates or replaces a file and defaults to UTF-8 without a BOM. NeoCLR keeps that
familiar behavior, adding a caller-specified byte bound and a Result instead of
exception-based failure. This makes bounds and failure handling explicit but adds an
argument and matching at call sites. Void is a valid generic argument, so success
needs no extra Unit class. There is no encoding-overload or globalization framework.

One InternalCall returns a private integer status; library IL constructs public
Result and FileWriteError values. Reachability reports FileOutput for the helper.
No new opcode, metadata format, stream hierarchy or native handle is exposed.
Host access follows process permissions; service reporting is not an access policy.
The preview does not promise .NET file-sharing flag parity, atomicity or asynchronous
I/O. Those remain separate requirements for future stream/interop work.

## End-to-end report example

The Neo sample takes an input file and an existing output directory, reads at most
1 MiB, and writes at most 4096 bytes to `summary.txt` in that directory. It combines
Environment arguments, Path operations, String operations and Result matching.
It replaces an existing summary.txt; choose an output directory accordingly.

```sh
cargo run --locked -- run examples/source/file-report.neo -- README.md /tmp
cargo test --locked --test file_output
```

The report contains the input basename and its UTF-8 byte count. No date/time
formatting API is introduced. The CLI still prints the guest return value; nonzero
Main results do not yet become process exit codes.

Tests use temporary directories to check UTF-8/no-BOM output, embedded NULs, exact
byte limits, preflight preservation, truncation, empty files, missing parents,
directory/invalid-path failures, artifact execution and the CLI report. Actual
permission and disk-failure behavior remains dependent on OS/filesystem conditions.

The experimental [Raven projection](raven-file-api.md) documents admitted calls,
propagation, conditional-output checks and remaining API gaps.

## Native compilation (development, 2026-10-08)

The ARM64 POC now binds the exact `neoCLR.Runtime.WriteAllText(String, String, Int32)
-> Int32` InternalCall with `--bind-file-output`, requiring `--compile-system` and
`--reference-arena`. The explicit flag describes a statically linked POSIX adapter;
it is not a filesystem sandbox or permission grant. FileText's Result/error construction
remains ordinary compiled Raven CIL. The existing .NET comparison and synchronous
file contract above apply; no new guest API or compiler encoding is introduced.

`docs/experiments/aot-console/file-output.c` copies the bounded path into temporary
NUL-terminated host storage, opens without truncation, checks the descriptor is regular,
then truncates and writes all bytes with interrupted-write retries. It follows symlinks,
uses ordinary process permissions, closes the descriptor and releases temporary storage.
The adapter allocates no managed text and retains no input pointers. Its private ABI
separates terminal native failure from managed operation statuses; failure does not
publish an output slot. Opened files may still be created/truncated/partially written
on later failure. No fsync, atomic replacement, cancellation or file-sharing parity
with .NET is promised. Permission/disk-full and syscall-interruption injection remain
host-dependent validation gaps; ordinary error and preflight behavior is checked.

Eleven comparisons cover UTF-8, embedded NULs, truncation to empty, exact limits,
rejected oversized writes preserving previous bytes, invalid limits/paths, missing
parents, directories and symlinks. Tests also check descriptor counts, GC-root cleanup,
output/arena canaries, opt-in prerequisites and impostor contracts.
The [Raven consumer](experiments/raven-target/samples/native-file-output.rvn) exercises
`let` patterns with `else`, success and FileWriteError.TooLarge. The verifier runs each
mode in its own fresh directory and checks both output and preserved file contents.
[Execution evidence](../benchmarks/native-web/file-output-validation.json) records VM,
sanitized native, standalone/libSystem-only execution and retained HTTP admission.
Reproduce with `benchmarks/native-web/verify_callbacks.py --case FileOutput` and the
usual compiler/runtime/AOT/bundle/output arguments. This is not a performance benchmark.

ReadAllText is still unbound in this slice, so the original combined file sample is
not yet supported natively. The CLI Core.dll bootstrap remains a release gate.
