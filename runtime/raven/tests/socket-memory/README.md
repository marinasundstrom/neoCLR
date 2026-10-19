# Socket persistent-memory contracts

Run `python3 runtime/raven/tests/socket-memory/verify.py --bundle target/socket-memory-final-bundle/bundle --output target/socket-memory-consumer`.

The async Raven consumer opens a bounded loopback pair and qualifies the production
Memory/ReadOnlyMemory API in interpreter/JIT/AOT: nonzero windows and sentinels, send
snapshot stability after alias mutation, GC during pending receives, sole-owner
receive buffers, default/end-position empty windows, deadlines, cancellation
acknowledgement, pre-cancellation, EOF and closed operations.

The verifier retains build logs, exact process output and artifact hashes. It uses
the HTTP native profile for socket/timer/task ownership support. This is local
loopback correctness evidence, not throughput or Windows qualification. The
interpreter socket tests additionally verify sole-owner rooting/release and that
uninitialized bytes outside the send window are never read.

The public API has no array/range compatibility overload. Select Memory windows
before calling; constructing an invalid window faults before invocation.
