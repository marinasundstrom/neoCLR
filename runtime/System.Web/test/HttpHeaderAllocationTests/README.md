# HTTP header allocation tests

Three named NeoClr.Testing cases check warmed header lookup and cached views,
lazy entry materialization on real parsed request/response headers, and response
configuration that retains headers when content has no media type.

Main runs the cases in their existing order, explicitly awaits the HTTP lifecycle,
and registers completed-result reference adapters with TestSuite. ConsoleReporter
prints actual outcomes and Main returns report.ExitCode. Recoverable assertion
failures allow the other independent cases to run; terminal runtime faults still
end execution. This replaces a Panic reporter and hard-coded success summary.
Automatic async discovery is not added.

Framework registration, adapter construction and reporting happen outside the
allocation-counter regions in Tests.rvn. The fixture retains its original warmup,
loop counts, collection counters, zero-allocation expectations and one-allocation
positional-access check. It measures interpreter managed objects, not native
allocation counts or throughput.

```sh
python3 scripts/test-runtime-library.py --bundle <matching-bundle> \
  --output target/http-header-allocation-tests \
  --suite http-header-allocations --mode interpreter
```

Use a bundle containing the response-header retention fix qualified in
[the original evidence](../../../../docs/validation/http-header-noop-20261031/README.md).
The earlier HTTP-message bundle lacks that fix and intentionally fails the final
case with 512 allocations. Native/JIT GC counters are not bound; the runner keeps
this suite interpreter-only.

[Migration evidence](../../../../docs/validation/testing-migration-20261101/README.md#http-header-allocations)
records three passes with the matching bundle, the older-bundle failure and an
isolated wrong-expectation control producing one named failure and exit one.
