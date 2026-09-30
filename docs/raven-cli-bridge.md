# Raven CLI bridge: temporary behavior and native replacement

Direction reaffirmed by the author, 2026-09-30: replace the neoCLR bridge with a
true neoCLR metadata/semantic-data layer supporting the runtime's features and
semantics. A compatible native code generator must also remove the output bridge.
The shared Raven semantic model remains authoritative; CLI shape limitations must
not become permanent neoCLR language restrictions.

The compiler-side inventory is `docs/compiler/neoclr-cli-bridge.md` in Raven.
Document every bridge behavior there with native intent, temporary CLI encoding,
loss/restriction, owner, validation and its replacement capability. Update the
corresponding runtime document here. Current examples include:

- [Void](void-semantics.md): inhabited unit in value/generic positions versus a
  no-result call convention. Named System.Void and CLI VOID encode distinct things.
- [Functions](function-types.md): structural function signatures transported through
  CLI delegate shapes. Native support is on `feature/function-types`, inherited by
  `codex/native-self`, and absent from neoCLR main at `e4f6fe41`.
- Tuples: value-type System.Tuple reference transport maps to Raven's historical
  ValueTuple special-type identifiers; do not infer .NET reference tuple semantics.
- [Signature projection](raven-signature-projection.md): generic substitution and
  API catalogs with path-specific admission restrictions. Bridge rejection does not
  by itself establish a runtime restriction.
- [Application identities](raven-import-identities.md): temporary encoded names,
  CLI tokens and sidecar maps should be replaced by structured native identities.
- [Self](self-types.md): native semantics and separate Raven feature work must be
  distinguished from the profile's baseline contracts. The main-line preset does
  not implement Self or configure record-equatability/hash mappings.

Iteration, propagation, typeof handles, characters, async state, terminal Fault and
nullability also need this classification. Separate platform semantics from
reference-exporter projection, compiler lowering/emission and application-importer
workarounds. Require explicit unsupported-use diagnostics until the necessary
native symbol/contract/backend support exists; never silently erase information.

## Feature scope and exploratory evidence

The author confirmed native Function types are deferred until neoCLR has its
metadata layer and complete compiler support. An exploratory run of Raven 9a58e1356
against the installed Function-types feature bundle compiled, imported, verified
and executed a small consumer with output 42, 7, True. It was not run on neoCLR main.
The bundle's recorded runtime source is 19c6725f, bundle revision 4d1e7506; its four
principal artifact hashes matched the manifest. No complete support claim follows.

The proposed Function-dependent smoke gate and props migration were withdrawn
before commit. Existing runtime props, installed bundles and applications remain
unchanged. The local exploratory record is /tmp/raven-neoclr-profile-slice39/evidence.json
on the validation host, not an acceptance gate for supported neoCLR-main behavior.

The recommended path is to retain compiler-side target plumbing and experimental
CLI handling on shared Raven main while preserving .NET defaults. Do not conflate
Raven function syntax/.NET delegate support with neoCLR native structural Function
semantics. The latter requires native metadata/symbol support, conversion and
assignability rules, capabilities and a validated codegen path before promotion.
There is no need for another permanent compiler branch split while those parts are
implemented. Classify and gate unsupported operations without a blanket syntax ban.

The next design slice should specify native structured identities, type forms and
constraints, semantic relationships/flags, symbol ownership/lazy loading, malformed
metadata diagnostics and matching output representation. An input loader alone
cannot remove the CLI output importer or establish full feature support.
