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

Compact extension labels omit receiver-bound type parameters on receiver pages
(for example, `String.Any()`), preserving independent caller-selected parameters.
Receiver substitutions also apply inside parameter and return types, such as
`Map<U>(selector: Func<Char, U>)` on String.
The extension container and declaration signature keep the full generic contract.

## Site search, copying and namespace documentation

The site enables RavenDoc's `search` and `copyCode` configuration options. The
navbar search icon opens a field covering guides and API articles, including
manual API pages. A nonempty query keeps the panel open; Escape closes it without
clearing the query. Results are served locally from a generated index. Copy
buttons preserve displayed code and report clipboard failures accessibly. They
float over samples and signatures without adding top padding or pushing text down.

Public nested types appear under Nested types; union cases remain under Cases.
Namespace `N:` comments are supported in source and Markdown/XML sidecars.
`website/api-content` supplies additional namespace guidance without changing the
reference snapshot. The build finalizes the index after assembling manual routes.
These are shared RavenDoc/compiler documentation changes: no Runtime Contract,
native metadata, CLI bridge encoding or runtime execution behavior changes.

Validation for this update: pinned Raven revision
`816b5d8a63e71891e364d98a470aa506debc1609`, 1,803 generated pages with local
links/anchors/HTML checked, and all 18 Python website tests passing. Browser
checks passed at 390px and 1280px in light/dark themes on the homepage, API
landing, System namespace and ArrayList pages, including search retention and
API results, copy controls and namespace guidance. No native runtime execution
is claimed for this documentation update. Local preview is not publication.

The reviewed publisher also indexes explicitly documented compile-time macros,
uses a separate collapsible section sidebar for ordinary articles, supports API source
`title` display names (`apiTitle` for a single input), and keeps mobile theme/search
controls alongside the brand. neoCLR retains its `apiNavigationRoot` browser for
authored reference pages. Assembly identity and native runtime behavior are unchanged.

A compatibility check preserves automatic library links on sites with no authored
menu. Article sections open for the current page and support native keyboard
toggling; generated API pages retain their symbol browser.

`navigationScope` chooses `site` (one root TOC across article folders) or `section`
(the nearest section TOC). neoCLR explicitly keeps `section` alongside its
`apiNavigationRoot`, while Raven uses `site` and a composed root hierarchy.
Both modes are covered by upstream site-builder tests; the website regression
suite cross-checks the unchanged neoCLR API and article behavior.

Sites can further define explicit `navigationSections` by directory, each using
its conventional `toc.yml` and optional sidebar title. The most specific boundary
keeps one hierarchy throughout its descendants. Raven uses this for its separate
Getting started and Language reference navbar areas; neoCLR needs no new boundary
and retains its existing configuration.

Wide tables scroll inside articles on mobile screens, including when platform
font metrics make a comparison table wider than the available content column.

The mobile main navbar uses a three-dot disclosure alongside color theme and
site search. Copy controls stay fixed at the visible edge while code scrolls.
The 390px, 768px and 1280px previews passed menu/Escape, theme/search visibility,
code-scroll alignment and overflow checks in both themes; all 18 website tests
and the complete site build passed. These changes are staged locally for neoCLR;
publication remains a separate action.
