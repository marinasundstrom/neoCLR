# Closed generic base annotation contracts — 2026-10-28

DAM2006 now checks nongeneric reference-class overrides whose ancestry contains
closed generic types. The prior check skipped every constructed ancestor. The shared
verifier slot lookup already substitutes ancestor type arguments before matching
signatures; this follow-up admits that path for annotation comparison.

This preserves the exact-match rule described in [the contract](../../dynamically-accessed-members.md)
and .NET IL2092 comparison. It does not implement annotations on generic parameters,
generic derived classes/methods, generic interface mappings or value-type overrides.
A verified nongeneric derived class supplies a closed ancestry context. Existing
metadata and Runtime Contract settings suffice; no compiler emission change is needed.

## Validation

- 19 focused `dynamic_access::tests` pass. New cases exercise substituted parameter
  signatures, distinct same-named overloads and multi-level ancestry with nested
  array type arguments. Fixtures use native open generic owner representations.
- [Native consumer](../../experiments/dynamic-access/generic-base/Main.rvn):
  DefaultReader derives from Reader<int>, overriding Read(TypeInfo, T) as
  Read(TypeInfo, int). Matching TypeInfo annotations produce no DAM2006; execution
  through Reader<int> prints `7`, exit 0.
- Mismatch control changes only the override's annotation to PublicProperties (512)
  while its base declares PublicConstructors (3). The saved previous backend builds
  and executes without DAM2006, demonstrating the skipped check. The updated backend
  reports exactly one DAM2006 naming DefaultReader.Read and Reader`1.Read. Execution
  still prints `7`, exit 0; the method performs no reflection.
- All three consumers infer only Model's constructor requirement (3). No properties
  or implementation bodies are silently retained. DAM2004 remains because dynamic
  implementation-body traversal is still a separate gap.
- API reference fingerprint check passes. No public API signature changes or snapshot
  rebuild; no website build/publication. The previous inheritance regression evidence
  remains applicable because the shared slot lookup is unchanged in this follow-up.

[Results](results.json) include diagnostics, policy rows, backend/executable hashes
and outputs. Inputs: compiler bundle ed1d08def, the previously qualified annotated
libraries and stable debug-backend copies on macOS ARM64. The HTTP native service
profile is used; the app performs no networking. No Windows/JIT execution or
performance qualification is claimed.

Reproduce using the [consumer instructions](../../experiments/dynamic-access/README.md)
with `--project docs/experiments/dynamic-access/generic-base/App.rvnproj`. Change only
the override's requirement to PublicProperties for the mismatch control. Run
`cargo test -q --lib dynamic_access::tests` and
`python3 scripts/build-api-docs.py --check` for focused checks.

Removing the conservative ancestry exclusion adds metadata traversal for these
previously skipped overrides. No speedup is claimed; diagnostics do not change
runtime dispatch, invocation rights or retention policy.
