# Console memory integration

Run the discovered behavior test and host byte checks with a matching bundle:

```sh
python3 runtime/System.Runtime/test/ConsoleMemoryTests/verify.py \
  --bundle /absolute/matching-bundle --output /fresh/output
```

The public standard-console streams are synchronous `SyncOutputStream` values.
One `NeoClr.Testing` case checks selected `ReadOnlySlice<byte>` windows, NUL/high
bytes, end-position empty writes, flush, and closed nonempty/empty writes and flush.
The normal discovered registry feeds TestSuite. Its quiet success path preserves
binary stdout `2800ff` and stderr `00ff`; ConsoleReporter prints named diagnostics
only on failure. Python verifies the exact bytes in interpreter, JIT and AOT.
JIT may prefix the exact documented DAM2004 compiler warning; the report preserves
raw stderr and separates only that exact warning from the guest payload.

Three subprocess guards reject negative, oversized and beyond-end empty slice
windows with IndexOutOfRange before Write is invoked. The original async
array/range consumer was stale: invalid ranges are now rejected while constructing
the slice, and synchronous console streams have no task/pre-cancellation contract.
Those removed assertions are not claimed as passing current console behavior.
See the [provider capability design](../../../../docs/stream-design.md#text-capabilities-and-provider-dependent-modes--2026-10-20-proposal).

[Migration evidence](../../../../docs/validation/testing-migration-20261101/README.md#console-memory-windows)
retains successful mode checks, bounds guards and an open read-only-stderr diagnostic.
Existing Rust host-call snapshot/rooting and native memory-binding tests remain
separate and unchanged. This test migration does not implement asynchronous console
I/O, change the provider or alter public APIs.
