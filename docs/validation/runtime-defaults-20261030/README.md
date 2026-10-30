# Runtime optional arguments — 2026-10-30

Author-directed consolidation after the Configuration/default-argument and Result
entrypoint slices. This qualifies ordinary nominal struct zero defaults and removes
redundant forwarding overloads from streams, networking and HTTP. Scalar defaults
also cover text/gzip ownership, watcher filters and fractional time ticks.

## Compiler and metadata provenance

- Raven main `83e700ab6`: bind a null Constant on a value parameter as zero
  initialization. Baseline 11 and updated 13 parameter-default tests pass.
- Deliberate carry to `codex/neoclr`: `049c9fbd7`; no full branch merge.
- Native compiler `6ceeb1be8790e105a1394cb99fded41fafad5b4c`: admit ordinary
  non-union struct defaults and use the explicit default capability for interfaces.
  Five focused admission cases pass, including disabled-capability rejection.
- neoCLR source baseline `bfd312796`; final dirty source hashes and compiler inputs
  are retained in `build-evidence.json.gz`. This receipt belongs to the commit
  containing those changes, rather than claiming the baseline already included them.
- The metadata SDK round trip passes for native, CLI and projection paths; four Rust
  parameter-default tests pass, including nominal values and malformed rejection.

The existing typed Null default denotes reference null or nominal value zero;
there is no new custom attribute or runtime call-arity convention. Bare generic
defaults, union defaults and enum/char/decimal constants remain unqualified.

## Consumer verification

All 14 native libraries build. On macOS ARM64, the memory-stream suite (10 tests)
and HTTP lifetime suite (11 tests) pass in both native and interpreter execution:
42 passing test results. This includes newly added backtick-named behaviors for
omitted interface tokens, explicit/named cancelled tokens, text/gzip ownership,
encoding selection, watcher filters and fractional ticks. See `verification.log`
and the complete command/input/output report in `runtime-report.json.gz`.

```sh
python3 scripts/test-runtime-library.py \
  --bundle target/runtime-defaults-20261030/bundle \
  --runtime target/debug/neoclr --aot target/debug/neoclr-codegen \
  --suite memory-stream --suite http-lifetime \
  --mode native --mode interpreter \
  --output target/runtime-defaults-20261030/verification
```

The consumer run preceded restoration of documentation comments and removal of
the final context-callback `Serve(handler)` forwarding overload. System.Web was
rebuilt afterward, and the retained `Main.rvn`/`ServerCalls.rvnproj` compile-only
probe passes with omitted tokens for both request and context callback forms.
That final probe checks binding, not server execution. The previously passing
stream/client lifetime evidence is reused; no new platform matrix is claimed.

## Documentation verification and migration

RavenDoc rendered the final 14-library reference successfully. The authored Serve
pages were consolidated to the surviving signature. The native snapshot uses the
compiler revision above and exact source hashes; four snapshot tests pass. The
aggregate reference/source-fingerprint check also passes. The InputStream.Read
reference was reviewed in the local browser and displays `CancellationToken = default`.
No full website build was needed.

Rebuild binary consumers and interface implementations together. Default arguments
are inserted according to the static receiver at the call site; they do not add
delegate arities. Replace method-group bindings to removed overloads with lambdas
where needed. Encoding-selection and computed-default overloads remain distinct.

Retained text logs normalize trailing whitespace; compressed JSON retains raw evidence.
