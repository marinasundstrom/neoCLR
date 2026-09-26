# Client request association and default headers

This focused consumer uses immediate and delayed custom handlers. It verifies:

- standalone responses start with Request=None; successful Send retains the effective request;
- exact identity without defaults, pending completion under allocation pressure and Respond preserving association;
- failure/cancellation isolation and pre-cancellation before invoking a handler;
- copied client defaults, explicit case-insensitive overrides and ordered repeated defaults;
- unchanged caller requests and snapshots retained after reconfiguration;
- invalid framing/content headers and injected values failing before the handler.

```sh
python3 docs/experiments/http-request-association/verify.py \
  --toolchain-root /absolute/path/to/matching-development-bundle \
  --runner target/release/examples/measure_async
```

The focused run passes with 637 allocations, 14 collections and zero final live
objects on a 256-object heap. The existing [wire-header fixture](../http-request-headers/README.md)
passes GET/POST defaults against an independent peer (2,157 allocations, zero final
live objects). All 569 bridge signature checks and API/library snapshot checks pass. No full suite or website build
is required for this bounded validation; API/library snapshots are refreshed.
