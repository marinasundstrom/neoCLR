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
headings and page-outline links wrap without widening JsonNumber at 390px or
1280px viewports. The existing
reference assembly was unchanged; only the exclusion-policy fingerprint changed.
Local generation is not publication; deployment remains the manual website workflow.

## Typography defaults

The shared publisher uses `--doc-text-size: 0.9375rem` for prose/member labels
and `--doc-code-size: 0.8125rem` for code blocks and signatures, with a 1.6 line
height. The site adopts these defaults instead of enlarging article titles.
Sites can override the variables in their configured stylesheet; relative units
retain browser font preferences and zoom. Long inline code wraps in prose;
code blocks retain internal horizontal scrolling and syntax highlighting.

The typography review covered the landing page, guides, a code-heavy HTTP guide
and API reference at desktop and narrow widths in light and dark themes.

Type pages now link documented derived types; interface pages separate derived
interfaces from implementing types. Indirect relationships are marked, and generic
contracts match their original definitions. Selection/exclusions still apply.
The Show extension members toggle defaults on and persists independently of the
inherited-member toggle, updating group counts and the outline in either mode.
Validation covers source and imported metadata, selection filtering, generic
contracts and browser interaction with the ArrayList page.

Validation for this publisher update: 36 focused RavenDoc/site-builder tests,
a targeted two-case source/imported XML-text regression rerun, 18 website tests,
and a clean website build validating 1,755 generated pages and local links.

Member toggles now share a row when space permits and wrap on narrow screens.
Changing the controls updates `groupBy`, `inherited` and `extensions` in the URL,
preserving other query parameters and anchors. Valid shared settings override
saved preferences; opening a shared URL does not change those preferences.
Desktop/mobile browser checks verify layout, URL precedence, invalid-value fallback
and preservation of existing query parameters and fragments.

The website enables extensions from `System`, `System.Tasks`, `System.Linq` and
`System.Runtime.Reflection`. This includes outcome and task composition methods
on their applicable receiver pages; extension namespaces are an explicit site
selection rather than a RavenDoc hard-coded list.

RavenDoc resolves open generic receivers with their type parameters and keeps
extension containers’ own declarations visible even with `extensions=false`.
Static classes show no inherited instance members: `Task` is the static submission
API, while `Task<T>` is the awaitable operation type.
