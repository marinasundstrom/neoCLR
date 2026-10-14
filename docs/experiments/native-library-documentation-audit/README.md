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
pipeline is unchanged; unmatched old pages remain with a migration-gap notice. The selection is saved locally in ignored `target/native-api-preview.json`.
Ordinary `scripts/build-website.py` rebuilds use the committed native API snapshot;
the historical local selection file no longer overrides it. To preview this audit
through the complete builder, pass `--native-audit /absolute/path/to/audit` explicitly.
The selected reference is integrated and validated before replacing the visible site.
A missing explicit audit fails the build and preserves the visible site. Selection
does not regenerate native metadata or change the committed publication snapshot.

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
passed. Subsequent investigation resolved the failure in Raven `92a593ff7`:
inhabited Void inside generic constructor signatures selected inconsistent library
owners. A reduced regression reproduces the old failure; both unit spellings and
the complete native bundle pass with the explicit unit-owner correction.

Raven implementation: `502d1f19d`. Browser review before commit verified Math XML,
Pi/E/Tau navigation, both Abs overloads and the CancellationToken → GetHashCode →
Object link chain in the neoCLR theme at canonical API routes.

## Native function documentation fallback (2026-10-12)

With Raven `5614397c5`, native namespace functions render correctly but
`DocumentationCommentIdBuilder` does not provide their XML IDs, preventing the
recovery above. The themed audit merges shared API content with native-only
`api-content/` entries into its fresh output directory. Math.Exp uses a reviewed
manual summary and parameter entry until canonical IDs and comment lookup work.
RavenDoc validates each content UID against an actual rendered symbol; metadata
continues to supply signatures. See the tracked gap in
[API maintenance](../../../api-docs/README.md#native-namespace-function-xml-documentation-gap).

The historical sidecar normalizer also accepts a plain-text summary followed by
XML parameter/return/remark tags (Encoding.Decode). It converts those contracts
to Markdown directives before rendering; malformed or nested XML still fails.

### Declaration source links

The themed audit forwards `sourceRepository` from `website/site.json`, resolving
its root against the website directory exactly as the normal site builder does.
Keep the declaration paths inclusive of provider libraries. Snapshot packaging
checks type/member/generic/provider links before replacing the publication archive;
extraction repeats those checks when provenance settings are present in the manifest.
These are configured declaration-file links, not exact member-span source maps.
