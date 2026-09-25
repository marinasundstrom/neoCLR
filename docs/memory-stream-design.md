# Provisional MemoryStream — 2026-09-25

The author requested an in-memory stream while building the JSON DOM string/stream
POC. System.IO.MemoryStream implements InputStream, OutputStream and SeekableStream.
One cursor serves all capabilities. This is development work after Preview 9.

A parameterless constructor creates an empty managed buffer. Read copies available
bytes into a checked range and returns zero at/beyond EOF. Write copies a checked
range, overwrites existing bytes and extends when needed. Seeking uses an absolute
Int64 position; a gap is zero-filled only on a subsequent nonempty write. GetLength
and GetPosition return Result<Int64, StreamError>. Flush is a successful no-op while
open. Close is idempotent, releases backing storage, and makes every other operation
return Closed (including zero-length transfers). No array alias escapes.

The provisional maximum is 65536 bytes, matching the current text-reader/writer
operation bound. Negative positions and invalid array ranges return InvalidRange;
positions/end-of-write above the bound return LimitExceeded. Check all ranges before
mutation, using subtraction to avoid overflow. Seek alone and zero-byte writes do
not allocate padding or extend length. Closed takes precedence over other errors.
Allocation exhaustion remains a runtime Fault, not a recoverable stream error.

## Comparison and tradeoffs

Primary [System.IO.MemoryStream documentation](https://learn.microsoft.com/en-us/dotnet/api/system.io.memorystream?view=net-10.0)
checked 2026-09-25; executable baseline .NET SDK 10.0.100. Both designs provide managed
byte storage, an initially empty writable stream, a shared position and copy-based
I/O. The fixture compares overwrite, gap padding, EOF and JSON stream round trips.
neoCLR follows its existing capability interfaces and Result-returning position/seek
contract rather than .NET's Stream base, mutable Position property and SeekOrigin.

This POC deliberately lacks array-backed constructors, capacity/buffer exposure,
ToArray, SetLength, asynchronous operations and thread-safety. .NET offers a much
broader API. neoCLR's finite bound and zero-byte no-op policy are provisional choices,
not claimed .NET compatibility or improvements. Close releases the backing list;
there is no after-close buffer retrieval contract.

The managed ArrayList implementation reuses tested copying/growth/GC infrastructure.
A dedicated byte-array buffer could reduce indirection, but introduces another growth
implementation. Native allocation would add unnecessary ownership/rooting work.
There is no measured performance claim. Review the cap and backing representation
against larger concrete documents before promoting this as a general storage API.
No runtime instruction or compiler change is required; the bridge admits only the
explicit constructor/members and capability conversions.

## Evidence

The [JSON stream fixture](experiments/json-streams/README.md) exercises all public
members directly and through capabilities, text wrappers, overwrite/gap/EOF behavior,
invalid ranges, bounds, nonmutation on validation failure, close and JSON round trips.
The signature probe checks public members, conversion direction and Int64 seek
admission. API docs include every member. Website build remains skipped by author
direction; deployment is separate.
