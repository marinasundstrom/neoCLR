# Bounded UTF-8 file output

Implemented 2026-09-08:
`System.IO.File.WriteAllText(String path, String text, Int32 maxBytes) -> Result<Void,FileWriteError>`.

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
