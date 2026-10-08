# Production-library documentation migration audit

The author requires real declaring-library ownership instead of NeoCLR.CoreProbe.dll,
without losing API reference features or coverage. This audit renders the existing
four-library native metadata bundle using its explicit, temporary CLI primitive
bootstrap and ownership manifest. That staging input is deliberately different from
RavenDoc's no-fallback `--native-core-reference` loader. It is **not** no-bridge release
qualification and does not replace the website input.

```sh
dotnet run --project docs/experiments/native-library-documentation-audit/Audit.csproj \
  -p:RavenRoot=/absolute/path/to/Raven \
  -p:NeoClrMetadataProject=/absolute/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj \
  -p:WarningLevel=0 -- /absolute/path/to/bundle /tmp/fresh-doc-audit
```

The harness reuses the project metadata provider and its explicit primitive, Object,
async and service ownership contracts. Only Runtime/Data/Networking/Web are documented;
Core is a semantic dependency. The harness invokes the existing internal combined
renderer via host reflection; it does not load or execute inspected libraries. This
keeps a temporary audit mechanism out of RavenDoc's public configuration contract.

The first production render exposed a renderer crash on assembly-level functions:
its documentation ID builder assumed a declaring type. Correct namespace-qualified
IDs and constant documentation forwarding are now covered by reduced native fixtures
and ordinary .NET documentation controls. No runtime or IL behavior changes.

Initial output contains 1,632 API pages and no NeoCLR.CoreProbe labels. For example,
BooleanParseError and CancellationTokenSource identify System.Runtime.dll. Page counts
are not API-equivalence proof. The current reference has 384 public type names versus
346 in the native reference projections; many differences are bridge support or
assembly-level member carriers. `projection-preflight.json` records names only, not
semantic losses. The actual native symbol inventory and rendered member pages need
comparison with current public routes, authored XML/help and manual coverage before
switching the production website. Source-built native core ownership remains a
separate prerequisite for removing the target bridge entirely.

Validation: Raven `11825b08e`; 68 focused .NET documentation tests pass, together
with strict native fixture checks for global/namespaced functions, constant values,
comments and declaring libraries. See [audit results](validation.json) and
[native renderer checks](native-members-validation.json). The browser preview also
confirms the real owner; some production pages lack summaries, so documentation
coverage remains an explicit gate before replacing the full website.
