# Socket persistent-memory contracts

Run `python3 runtime/System.Networking/test/SocketMemoryTests/verify.py --bundle target/socket-memory-final-bundle/bundle --output target/socket-memory-consumer`.

One named async `NeoClr.Testing` case opens a bounded loopback pair and qualifies the production
Memory/ReadOnlyMemory API in interpreter/JIT/AOT: nonzero windows and sentinels, send
snapshot stability after alias mutation, GC during pending receives, sole-owner
receive buffers, default/end-position empty windows, deadlines, cancellation
acknowledgement, pre-cancellation, EOF and closed operations.

The explicit async entry point awaits the full lifecycle and registers the completed
result with TestSuite through a bound method on a reference owner. This is an adapter
for the synchronous runner, not automatic async discovery. Keeping one combined case
preserves pending operations, collections and alias/owner lifetimes across phases.
Assertions now return TestFailure messages instead of numeric failure exits. The
cancellation observer intentionally uses OnCompleted so an acknowledged cancelled
receive does not cancel the test entry point.

The verifier retains build logs, exact process output and artifact hashes. It checks
the standard one-test summary and empty guest stderr; only the exact documented
DAM2004 JIT compiler warning is admitted and recorded separately. It uses
the HTTP native profile for socket/timer/task ownership support. This is local
loopback correctness evidence, not throughput or Windows qualification. The
interpreter socket tests additionally verify sole-owner rooting/release and that
uninitialized bytes outside the send window are never read.

The public API has no array/range compatibility overload. Select Memory windows
before calling; constructing an invalid window faults before invocation.

[Migration evidence](../../../../docs/validation/testing-migration-20261101/README.md#socket-memory-ownership)
records validation and the adapter's compiler limitation. Production socket APIs,
provider semantics and native/interpreter kernel checks remain unchanged.
