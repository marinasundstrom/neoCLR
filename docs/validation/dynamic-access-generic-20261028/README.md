# Generic dynamic member contracts — 2026-10-28

The author requested all remaining annotation targets/member categories and then
explicitly directed fixing Raven gaps. This layer adds generic parameter contracts;
it does not complete the broader member-category request.

Raven main `80550fa23` adds generic parameter syntax, binding, diagnostics,
GetDeclaredSymbol, CLR custom-attribute emission and PE import. The focused carry
is `a9aeb8aa4`; native transport is `3a79bee37`. Native metadata SDK changes preserve
type/method ordinal attributes in CLI/native output, loaded views and CLI projection.
New optional GenericParam origin tokens reject malformed identities. Components
must be rebuilt together; the old receiver backend rejects this metadata field.

The native [consumer](../../experiments/dynamic-access/generic/Main.rvn) exercises
annotated generic type and method parameters and the newly annotated generic
JsonSerializer.Deserialize API. With no retention file it prints:

```text
retained
1
1
```

Exit status is zero. Dynamic analysis retains Message with flags 515. The four
existing ObjectMapper nested descriptor-flow DAM2001 warnings and the dynamic
implementation-body DAM2004 warning remain; this result proves the simple model,
not arbitrary recursive JSON mapping. The initial native run caught generic origin
tokens surviving executable specialization; closed shapes now discard those slots
and their annotations after source analysis, covered by a focused regression.

Checks:

- Raven attribute suites: 58-test baseline, 61 tests after the change; required
  syntax-generation builds pass on main and native branches.
- Metadata `--generic-attributes`, `--return-attributes`, `--storage-attributes`:
  pass, covering native/CLI projection, introspection, immutable loads and malformed
  identities. Native compiler source/import/usage probe passes.
- Rust dynamic-access suite: 37 pass, including known arguments, generic forwarding,
  typeof(T), absent guarantees and constructed type arguments. Specialization
  regression: 1 pass.
- Rebuilt System.Runtime, System.Data, System.Networking, System.Web and PostgreSql
  libraries; matching native API render, CLI reference check and four native
  snapshot tests pass. No website build was required.

[Results](results.json) contain the native command, hashes, diagnostic report and
old-backend control. Libraries were built with the final compiler changes before
committing; the build provenance labels that worktree. Runtime Contract settings
and .NET default compilation are unchanged.

Remaining gaps include additional member flags, arbitrary constructed reflection
targets, generic interface/virtual comparison, full generic field/ancestry graph
propagation, descriptor-discovered types and string type-name flows. None is
silently treated as supported or as a permanent divergence from .NET.
