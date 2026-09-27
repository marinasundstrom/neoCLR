# RavenDoc semantic reference integration

RavenDoc owns reference rendering and navigation. neoCLR supplies its documented
assembly/XML snapshot, content, styling and site configuration. The portable
.NET 10 build is pinned in `tools/ravendoc/version.json`; website CI needs no
compiler checkout or full runtime release build. See the
[build procedure](../api-docs/README.md#build-and-refresh).

## Shared behavior

Type pages show inherited instance members and implemented interface contracts
with links to their origin. Interface pages state whether implementation is
required or a default exists. The consuming page distinguishes declarations,
inherited implementations, overrides and interface defaults. Static members stay
on their declaring type. Member-kind and declaring-type grouping share the same
member set, with a separate reader toggle for inherited members.

Applicable extensions from the namespaces selected in `website/site.json` join
the corresponding member lists with a purple E marker; static members use red S.
Extensions group under their declaring type and are independent of the inherited
member toggle. Lookup uses Raven semantics and only the documented assembly,
avoiding host framework APIs leaking into the neoCLR reference.

Union cases have individual reference pages. Generic companion members merge
into the logical union. Union/enum navigation does not expand to cases/constants,
and namespaces without direct members are hidden by default. Declared generic
and delegate names remain nominal in navigation, links and compact lists. Closed
hierarchies expose permitted direct subtypes and links to closed ancestors.

## Additional authored content

Upstream RavenDoc supports an `apiContent` directory containing Markdown files
with a front-matter `uid` matching an exact type/member documentation ID. The
content supplements generated contracts and existing documentation. Missing,
duplicate and unmatched IDs fail the build. The upstream markdown-docs sample
demonstrates this; neoCLR has not enabled a separate authored API tree yet.

## Compiler interpretation

Two independently tested general compiler fixes support these pages: projected
extension properties preserve their containing assembly/module, and imported
method override flags use CLI virtual slot reuse even in metadata-only reflection.
Constructors and new-slot interface implementations are not overrides. These
changes are on Raven main and neoclr; no Runtime Contract settings, target
policies or emitted IL conventions change.

## Validation

The upstream change passed 36 focused RavenDoc/site-builder tests, source and
metadata extension-lookup coverage, and a targeted metadata override regression.
The sample site and neoCLR preview were generated. Browser checks cover grouping,
inherited visibility, linked generic interface origins and removal of inherited
`Object.ReferenceEquals` from both ArrayList and the InvalidFormat union case.
The final pinned publisher built and checked 1,753 website pages, and all 18
Python website tests passed. The sidebar was verified at the top of the article
with the notice visible, and the mobile drawer was checked. Long declaring-type
headings wrap without widening JsonNumber at a 390px viewport. The existing
reference assembly was unchanged; only the exclusion-policy fingerprint changed.
Local generation is not publication; deployment remains the manual website workflow.
