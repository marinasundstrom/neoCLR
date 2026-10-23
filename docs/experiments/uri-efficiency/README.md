# URI allocation review — 2026-10-23

URI parsing borrows UTF-8 bytes instead of allocating an encoded array. URI path
scanning reads bytes instead of materializing single-character strings. Empty and
whole-input component slices reuse existing strings. Resolution constructs its
result from validated components instead of parsing the assembled text again.
The ambiguous newly exposed double-slash authority case still reparses; a chained
resolution test protects component interpretation. The 4096-byte quota is retained.

The existing .NET/RFC comparison in [URI design](../../uri-design.md) still applies:
no grammar, normalization, equality or public signature changes. Component strings
and intermediate concatenations still allocate. String resolution still creates a
temporary parsed Uri; this is an optimization opportunity, not a contract defect.
Path already retains its input string and requires no encoded array, so its
implementation is unchanged.

For 32 repeated operations, interpreter tracked managed object counts are:

| Operation | Before | After |
| --- | ---: | ---: |
| Path.Parse | 32 | 32 |
| Uri.Parse | 64 | 32 |
| Resolve(string) | 128 | 64 |
| Resolve(Uri) | 64 | 32 |

The baseline log records expected failures of the proposed regression budgets
against the old implementation. These budgets describe this fixture, not a rule
limiting value objects to one allocation. Useful component storage may require
multiple allocations. GC.AllocatedObjectCount excludes host/native allocation and
some string storage; the removed character slices are verified by source inspection,
not counted by this metric. No HTTP throughput gain is inferred.

All 12 URI and five Path contract tests pass in interpreter, JIT and native (51
executions), plus four interpreter allocation checks. The matched bundle is
target/uri-efficiency-bundle/bundle with Raven 1128b98cf; test reports retain commands
and identities. API snapshots are refreshed; no website feature/API behavior changed.
