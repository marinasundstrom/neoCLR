# Explicit retention as dynamic-member analysis roots — 2026-10-28

Executable bodies admitted by an explicit policy now seed DynamicallyAccessedMembers
analysis. The existing construction, constructor and property binders validate the
policy on a disposable module copy and return admitted executable rows. The analysis
visits their annotated calls, then the final explicit/inferred union binds the real
module. This preserves source access, policy validation and the distinction between
metadata inspection and invocation.

This follows the existing .NET-style annotation propagation design; it adds no new
attribute targets, flag values, compiler transport or Runtime Contract configuration.
The cost is additional build-time binding and a temporary module copy; no performance
improvement is claimed. [Contract and source research](../../dynamically-accessed-members.md).

## Validation

- 12 focused analysis tests pass, including explicit body seeds, dependencies without
  implicit member grants on the seed's owner, and out-of-bounds seed rejection.
- The new backend policy test checks executable constructor admission, metadata-only
  exclusion, method-inspection exclusion, private constructor exclusion and malformed
  policy rejection. Six existing construction-policy tests also pass.
- The API fingerprint check passes; no public runtime API changes or snapshot refresh
  are needed. No website build/publication was performed.
- The [native project](../../experiments/dynamic-access/explicit/Main.rvn) retains only
  Message in its [project policy](../../experiments/dynamic-access/explicit/App.retention.json).
  Main forwards its TypeInfo through an unannotated function. Message's constructor
  calls an annotated helper for Other. Only the new policy seed path can infer Other.
- Saved pre-follow-up backend: build passes, execution exits 1 with missing property
  metadata while mapping Other from Message's constructor.
- Updated backend: Other alone is inferred with flags 513, and Message remains an
  explicit policy root. The final policy contains both types. Execution prints
  `retained` twice and exits 0. [Recorded results](results.json) include diagnostics,
  policy rows, artifact hashes and execution output from both builds.

Both builds use compiler bundle `ed1d08def` and the same annotated libraries built
for the initial implementation. Stable copies of each backend avoid concurrent
Cargo rebuilds changing build inputs. The HTTP service profile supplies socket
bindings selected by the JSON library graph; the app itself does no networking.
No JIT, Windows or performance qualification is claimed.

Reproduce with the [consumer instructions](../../experiments/dynamic-access/README.md),
using `--project docs/experiments/dynamic-access/explicit/App.rvnproj`. The adjacent
App.retention.json is discovered automatically. Focused checks:

```sh
cargo test -q --lib dynamic_access::tests
cargo test -q -p neoclr-codegen explicit_retention_analysis_tests
cargo test -q -p neoclr-codegen reflection_construction::tests
python3 scripts/build-api-docs.py --check
```

Explicit retention still does not prove unknown flows or suppress warnings.
Generic-owner body specialization, dynamic implementation dispatch and types
obtained through arbitrary storage/reflection descriptors remain outside this slice.
Metadata-only policies do not become executable analysis roots.
