# Persistent memory across managed suspension

This test-only transfer interface exercises Memory<byte> destinations and
ReadOnlyMemory<byte> sources before changing production stream contracts.
It follows .NET's distinction between synchronous borrowed spans and persistent
memory retained by asynchronous operations, using implicit typed conversions.

Run against a matching native compiler/library bundle:

```sh
python3 scripts/test-runtime-library.py --bundle <bundle> \
  --output target/memory-lifetimes-tests --suite memory-lifetimes \
  --mode native --mode jit --mode interpreter
```

An explicit async Main runs these tests because the discovery runner is synchronous.
Progress promises identify the intermediate suspension; draining TaskQueue alone
is not a completion barrier for native suspended frames.

The four tests cover two pending awaits, a creator frame that has returned,
explicit collections, read/write subwindows, surrounding sentinels, cancellation,
recoverable rejection, and default/end-position empty windows. Temporary borrowed
views are confined to synchronous helpers. Results are integers, not memory views.

Negative fixtures must fail with RAV2205 (ref-like capture). Compile each of
`reject-slice-capture.rvn` and `reject-readonly-capture.rvn` using:

```sh
NeoClrBundleRoot=<absolute-bundle> BorrowedCaptureSource=reject-slice-capture.rvn \
  dotnet <bundle>/sdk/tools/rvnc/rvnc.dll neoclr \
  --project runtime/System.Runtime/test/RejectBorrowedCapture/RejectBorrowedCapture.rvnproj
```

This does not qualify native I/O completion ownership, pointer retention/pinning,
root release after settlement, terminal runtime faults, Task<Memory<byte>> results,
non-byte native views, or Windows execution. Production stream/socket migration
must validate those relevant boundaries separately.
