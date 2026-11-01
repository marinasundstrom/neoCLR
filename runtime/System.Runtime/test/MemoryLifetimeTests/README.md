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

An explicit async Main runs these tests sequentially because the discovery runner
is synchronous. Each completed Result is retained by a reference adapter and
registered with TestSuite; ConsoleReporter prints the actual outcomes and Main
returns the report's ExitCode. Recoverable assertion failures are reported by name
and do not prevent the remaining independent cases from running. Terminal runtime
faults still terminate execution. This replaces the handwritten Panic reporter and
hard-coded success summary; it does not add automatic async discovery.
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

[Migration evidence](../../../../docs/validation/testing-migration-20261101/README.md#persistent-memory-lifetimes)
records four passing tests in each of interpreter, JIT and AOT, plus an isolated
wrong-count negative control reporting three passes, one named failure and exit one.
