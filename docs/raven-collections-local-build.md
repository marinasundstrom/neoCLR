# Try the collection library with Raven (.12)

This local experimental build includes the newer collection capabilities, Map and
HashMap, Option/Result-based LINQ terminals and direct ArrayList filters. It is not
a newly published neoCLR release. It uses ordinary Raven classes and array syntax;
no explicit managed-reference syntax is required for this workflow.

## Open the demo

```sh
code --new-window \
  --user-data-dir /Users/robert/.neoclr/vscode/collections-20260913 \
  --extensions-dir /Users/robert/.neoclr/vscode/collections-20260913/extensions \
  /Users/robert/.neoclr/experiments/collections-20260913/demo
```

Open Main.rvn, save edits, then select **Terminal → Run Task → neoCLR: Run saved
project**. The normal Raven Run/Debug toolbar is not the neoCLR execution pipeline.
The task compiles the saved source, imports and verifies its IL, then executes the
bundled runtime library. Completion uses the installed experimental extension's
language server and matching neoCLR declaration metadata.

The initial [order collection workflow](raven-order-workflow.md#collection-integration-scenario-2026-09-13-source-slice)
shows duplicate rejection, Option lookup/propagation, shallow FindAll results that
retain class identity, array/interface queries and Result-based cardinality errors.
Expected output:

```text
Duplicate order rejected
3
3
Unknown order
Order already shipped
Multiple
2
Filtered orders share identity
2
303
303
0
Empty
```

Other samples are in `../tools/samples`, including `library-list-filters.rvn`,
`library-maps.rvn`, `library-query-terminals.rvn` and the file/report workflow
`application-orders.rvn`. Preserve your edits before replacing Main.rvn.

## Installation and boundaries

- SDK: `/Users/robert/.raven/sdk/0.1.12-neoclr.12`
- Demo: `/Users/robert/.neoclr/experiments/collections-20260913/demo`
- Isolated VS Code profile: `/Users/robert/.neoclr/vscode/collections-20260913`
- Artifacts: `/Users/robert/.neoclr/builds/collections-toolchain-20260913`

The workspace selects this SDK explicitly. Existing SDK versions, normal selection
and earlier demos remain separate. Do not mix its regenerated core metadata with
an older System library. Breaking change: FindIndex now returns Option<int>, not
Int32/-1. Match Some/None as shown in the sample.

[Collection capabilities](collection-contracts.md) remain a bounded prototype:
arrays allow element replacement but do not promise growth. Map requires explicit
hash/equality callbacks and lacks a universal default comparer, removal and pair
iteration. [Query terminals](raven-query-api.md) and [ArrayList filters](arraylist-filtering.md)
use explicit absence/cardinality outcomes; callback faults remain terminal, without
fault-unwind cleanup. Read-only interfaces do not imply deep immutability.

## Validation and provenance

All eight suites passed against an extracted bundle: 63 saved-project cases,
29 query checks, 15 application checks, six file/order-workflow checks, ten collection
capability rejection cases, native buffers, direct neoIL samples and 61 editor checks.
All 733 pristine runtime payload hashes matched the bundle and archive; 439 SDK
archive payload files matched the SDK staging directory (macOS resource-fork entries
are not SDK payload). The installed VSIX server also passed 61 editor checks, and
the exact configured Run task produced the output above.

The SDK and extension report .12. Hashes confirm that 1,255 existing Raven source
files and the normal Raven launcher scripts were preserved. Configuration adds local
tasks/settings and selects the order sample after verifying pristine installed files.
No Raven source change or public release was made during this refresh.

[Build evidence](experiments/raven-target/collections-toolchain.json) records exact
revisions, artifact hashes, validation commands and installation paths. This build
uses neoCLR `6fd3729` and Raven `854cd4d3d` on its experimental branch, with .NET SDK
11.0.100-rc.1.26425.128 on macOS arm64. Artifacts, SHA256SUMS, logs and companion
Raven dependency notices are retained in the build directory above. The evidence
and installation documentation were committed after building these source revisions.
