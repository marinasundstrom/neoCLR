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

## Local neoCLR website integration

The updated website is intended for the upcoming release (author clarification,
2026-10-09). To view development work in the real neoCLR shell, build the complete
website first, then pass the repository root as the audit's third argument:

```sh
dotnet run --project docs/experiments/native-library-documentation-audit/Audit.csproj \
  -p:RavenRoot=/absolute/path/to/Raven \
  -p:NeoClrMetadataProject=/absolute/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj \
  -- /absolute/path/to/bundle /tmp/fresh-themed-audit /absolute/path/to/neoclr
python3 docs/experiments/native-library-documentation-audit/preview.py /tmp/fresh-themed-audit
```

This reuses `website/site.json` branding and `website/api-content`, integrates the
reference at the normal `/docs/api/` URLs, redirects the earlier `/docs/native-api/` URLs and
refreshes site-wide search. It validates HTML and all local links on a staged copy
before updating `target/website`. This is a local preview operation; the publishing
pipeline is unchanged; unmatched old pages remain with a migration-gap notice. Rebuilding the normal site
removes this optional preview.

The historical bundle contains XML documentation blocks mislabeled as Markdown.
For this audit only, the themed mode copies the bundle and converts simple complete
summary/parameter/returns/remarks XML blocks into Markdown documentation tags,
recording each affected sidecar in `normalized-sidecars.json`. It rejects unsupported
nested content instead of silently losing it. Original artifacts remain untouched.
Fixing the producer and rebuilding matching production sidecars remains a release
qualification task; this compatibility step does not establish full comment parity.

The themed audit also recovers reviewed XML from `api-docs/NeoCLR.CoreProbe.xml`
only for IDs matched to actual native declarations. The explicit temporary
`NamespaceMembers` carrier is removed from those IDs; the mapping and owning library
are recorded in `restored-xml-comments.json`. This preserves authored help in the
historical bundle preview. Raven's producer must emit canonical native IDs directly
for fresh target builds; the checked-in snapshot is not a permanent runtime dependency.


The final 2026-10-09 preview uses a freshly rebuilt four-library bundle and renders
1,642 native pages, including Pi/E/Tau, with 1,482 reviewed XML entries recovered.
The integrated site checks 3,483 pages (including compatibility redirects), retaining
136 explicitly marked legacy pages. [Website checks and remaining routes](website-validation.json)
and [actual constant declarations](constants-validation.json) record the result.
The first bundle build failed in HttpContext's constructor with an intermittent
NEOMETA003 stack-type comparison; a direct retry and complete second bundle build
passed. That failure remains a separate investigation, not a resolved compiler bug.

Raven implementation: `502d1f19d`. Browser review before commit verified Math XML,
Pi/E/Tau navigation, both Abs overloads and the CancellationToken → GetHashCode →
Object link chain in the neoCLR theme at canonical API routes.
