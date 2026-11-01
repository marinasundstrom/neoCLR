# HTTP content-copy behavior tests

Eight named async NeoClr.Testing cases cover copy backpressure and buffer ownership,
memory windows/aliasing/replay, read/write failures and invalid counts, declared
lengths and empty I/O, cancellation, disposal, string decoding/quotas and stream
creation validation. Assertions and lifecycle sequencing live in Tests.rvn.

Main awaits each case in its existing order, stores its completed Result in a
reference adapter and registers the adapter with TestSuite. ConsoleReporter prints
actual outcomes and Main returns report.ExitCode. Recoverable assertion failures
allow the remaining independent cases to run; terminal runtime faults still end
execution. This replaces the handwritten Panic reporter and literal success summary.
Automatic async discovery remains unsupported.

```sh
python3 scripts/test-runtime-library.py --bundle <matching-bundle> \
  --output target/http-content-copy-tests --suite http-content-copy \
  --mode interpreter --mode jit --mode native
```

The Python runner retains compilation, exact transcript and bundle integrity checks.
Its existing interpreter timeout accommodates the 65,537-byte text-quota case;
this migration does not change budgets or production APIs. Independent network-peer
checks remain separate host orchestration.

[Migration evidence](../../../../docs/validation/testing-migration-20261101/README.md#http-content-copy)
records eight passing cases in each mode and a wrong-count JIT control reporting
seven passes, one named failure and exit one. Existing
[content-copy design and .NET comparison](../../../../docs/design/http-content-streaming.md)
remain applicable.
