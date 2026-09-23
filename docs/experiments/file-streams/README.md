# File resource foundation

**Development experiment, 2026-09-23. Not a public Storage or Stream API.**
This is the first host-resource slice of the
[post-release checkpoint](../../platform-roadmap.md#post-release-concurrency-direction--2026-09-23).
The acceptance application will use Raven File/Directory and Stream abstractions;
those wrappers, their reference pages and that application remain to be implemented.

## Exploration, not an API commitment

The author clarified on 2026-09-23 that this work should investigate the best design
consistent with the platform's architectural direction. The disk application is an
acceptance experiment, not approval of every shape in the proposals or this backend.
Change or discard mechanisms as evidence warrants.

Evaluate the next slice against these questions:

- Can the application depend on a storage provider rather than host path operations?
- Do separate input/output capabilities keep unsupported operations out of contracts?
- Are file identity, open-stream ownership and explicit disposal clear to callers?
- Does typed failure stay separate from Task cancellation and invocation faults?
- Can byte-buffer ownership survive later suspension without tying Tasks to threads?
- Are limits and blocking behavior visible, without exposing raw transport handles?

The current table proves bounded ownership and incremental I/O. It does not select
integer handles as a public abstraction, whole-array copying as the final buffer
model, blocking dispatch as the scheduler, or an error taxonomy as permanent.
Compare the simpler alternatives and .NET baseline below when making those choices.

## Implemented boundary

The VM owns a file table for each invocation, alongside its worker resources.
Successful opens return an opaque positive integer, not an OS descriptor. Closing
an issued handle is idempotent. IDs never recycle during an invocation, so a stale
handle cannot accidentally address a later open. Another invocation has a separate
table; handles are not transferable capabilities. All remaining files are dropped
on success, fault, cancellation or instruction-budget exhaustion. GC does not own
the native table; the public wrapper will need explicit resource release.

The experimental services in `Services.neoil` support regular-file reads, opening
an existing file for writes without truncation, exclusive creation, chunk reads and
writes, flush, close, item-kind lookup and creating one directory. Creation does
not replace an existing entry or recursively create missing parent directories.
No implicit truncation, text decoding, seek, append or enumeration is implemented.

There are at most 64 simultaneously open files and 64 KiB per transfer. Closing
releases a table slot. Reads return up to the requested number of bytes, including
short reads and an empty array at EOF; a zero-size read does not establish EOF.
Writes report the actual byte count and may be short. A caller must loop until the
required bytes have been written. Invalid ranges and oversized requests fail before
I/O. Read buffers also respect the per-transfer guest array limits; subsequent
aggregate heap/array adoption can still fault after the host read advanced the file.

These are **blocking** Rust host calls. A future Task-shaped wrapper must not imply
nonblocking behavior, and this backend must not decide Task.Run scheduling. The
regular-file check happens both before opening an existing path and against the
opened handle, but a hostile path-replacement race may still cause a blocking open.
This is not a filesystem sandbox. Normal symlink resolution applies to existing
paths. File input/output retain the host process's permissions. A successful Flush
is not a durability guarantee; close/drop cannot report a delayed OS close error.

Bootstrap transport uses an erased Value: Byte means an error and other payloads
mean success (Int32 handle/count/status, or a Byte array for reads). Error codes are
internal: invalid path, missing entry, access denied, wrong item kind, existing
entry, closed/unknown handle, invalid range, resource limit, wrong access direction,
and other I/O failure. These must become typed public outcomes in the library, not
raw numeric errors in the application API. Invalid metadata or byte-array shape
remains a runtime fault. The declarations are deliberately excluded from the
application reference assembly and public DocFX API selection.

## Comparison and provisional decision

Primary sources reviewed 2026-09-23:

- [.NET 10 FileStream](https://learn.microsoft.com/en-us/dotnet/api/system.io.filestream?view=net-10.0)
  provides file byte access with synchronous/asynchronous operations and explicit
  disposal. That is the behavior baseline for the eventual file sample.
- [Rust File](https://doc.rust-lang.org/std/fs/struct.File.html) supplies the current
  host handle and blocking Read/Write implementation. Its drop closes the handle;
  closing errors are not reported by drop. Explicit flush is distinct from sync.
- [Rust OpenOptions](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html)
  supplies nontruncating opens and exclusive create_new behavior.

The proposals prefer separate input/output capabilities over .NET's common Stream
base with capability flags. This host slice keeps direction explicit without
settling the public interface or inheritance contract. Reading an entire file on
every operation would avoid retained handles, but would lose sequential positions
and bounded working buffers. Returning raw OS handles would expose platform details
and complicate ownership. An invocation table adds bookkeeping and requires explicit
close, but makes cleanup and stale-handle behavior testable before introducing
public wrappers. Chunk reads currently allocate and copy; no performance advantage
over .NET is claimed. Borrowed memory views and true asynchronous host completion
remain design/validation work.

## Validation and next slice

Run `cargo test --lib file_streams::` and `cargo test --test file_streams` with the
host SDK required by this checkout. The VM integration test writes and reads a real
file through the experimental service boundary. Unit cases cover short reads, EOF,
zero-size reads, direction errors, creation without overwrite, nontruncating writes,
invalid paths/ranges, handle limits and stale handles. This is lower-level evidence,
not the requested public Raven application.

Next: select and implement the minimal Storage provider/File/Directory wrappers,
directional byte streams with explicit disposal and typed errors, then document all
public members and compile the disk read/write application. Keep the existing
whole-text helpers usable during migration. Decide how the initial array-based
transfer shape relates to the proposals' Memory/ReadOnlyMemory before publishing it.

Validation on 2026-09-23: four host-resource unit cases and seven VM integration
cases passed, including a real disk round trip. Public Raven API validation remains
pending and must not be inferred from these lower-level checks.
