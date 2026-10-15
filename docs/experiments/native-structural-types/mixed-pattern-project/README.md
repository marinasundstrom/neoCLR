# Raven mixed scalar/reference patterns

Development qualification, 2026-10-15, on macOS ARM64.

`Main.rvn` checks `int | long | bool | Response` through a union parameter and
typed match arms. Six scalar inputs cover guarded/fallback arms, negative Int32,
wide Int64 and both Boolean values. A seventh input creates a nominal Response
and reads its Code property through the matched reference binding. The match
result joins as an integer; interpreter and native AOT both return 42.

Run from the neoCLR checkout using the experimental Raven compiler:

```sh
python3 docs/experiments/native-structural-types/verify-raven-scalar-patterns.py \
  --mixed --compiler /absolute/path/to/rvnc.dll \
  --output target/mixed-pattern-new-run
```

The runner requires the retained `target/structural-integrated-libraries-v3`
bundle, native debug tools and a new output directory. Set `SDKROOT` to a working
macOS SDK if necessary. It checks a native four-alternative union signature,
exact scalar tests/extractions, interpreter result, native imports and execution.
The sole native import is `neoclr_allocate_object_v1`; the linked context-v4 host
reports exactly 16 arena bytes, all for the explicitly constructed Response.
Scalar alternatives and the union itself add no allocation. This is allocation
qualification, not a throughput benchmark or stable external layout contract.

[Saved evidence](../raven-mixed-pattern-results.json) records artifact hashes,
checkout revisions, the retained bundle labels, commands and outcome. The bundle
libraries were not rebuilt. No Raven compiler or Runtime Contract change was
needed. Named guards retain the source-local scalar example's documented parser
workaround. Imported mixed calls, other payload shapes, full Task applications,
managed JIT/collector stress and throughput qualification remain open.
