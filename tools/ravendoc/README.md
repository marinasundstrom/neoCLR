# Pinned RavenDoc build

`ravendoc-net10.0.zip` is a portable, framework-dependent Release build of the
upstream RavenDoc publisher. `version.json` records its immutable Raven commit
and archive SHA-256. It runs on .NET 10 in Linux CI and macOS development without
a Raven checkout, NuGet restore, Node.js or DocFX. It is build tooling, never
copied to the published site. The MIT license is retained in `LICENSE`.

General site publishing, page front matter, navigation and compact API lists
are implemented and tested in Raven. There is no neoCLR publisher overlay.
neoCLR owns its site configuration, content, styling, API selection and release
notice. The generator keeps main menus, section navigation (`toc.yml`) and
in-page outlines separate. Generated API navigation uses the same internal
section-navigation model as authored menus. On small screens the sidebar opens
as an off-canvas drawer. neoCLR selects `namespaceNavigation: flat`, listing full
namespace names as peers while each expands to its types. RavenDoc also supports
`hierarchical` (the default); this option does not change reference URLs. Every
populated namespace expands to its overview and direct members. Nested types
remain on their containing type pages rather than adding sidebar branches.
`apiInputs` combines assembly snapshots in that tree; API metadata retains the
real declaring assembly and configured Raven/C# declaration source links. Empty namespaces
are hidden by default; `showEmptyNamespaces: true` restores their grouping rows.
Type navigation and headings show declared names such as Object, String and Char;
code signatures keep Raven aliases.

## Updating

Select a reviewed immutable commit, then run:

```sh
python3 scripts/update-ravendoc.py --revision FULL_COMMIT_SHA
python3 scripts/build-website.py
python3 -m unittest discover -s scripts -p 'test_build_website.py'
```

`--source /path/to/Raven` reads that commit from a local Git repository. The updater
builds a disposable checkout under `target`, smoke-tests the publisher, and replaces
the archive and lock. It never changes a sibling checkout. Review source changes,
archive size, the lock, and desktop/mobile rendering before committing. Normal CI
uses the checksum-verified archive; upstream updates are deliberate.

## Content and rendering

Markdown pages and HTML body fragments accept scalar front matter: `title`,
`layout: docs|landing`, and `toc: true|false`. The shared shell supplies branding,
navigation and release notices. The landing layout permits a separate hero and
feature cards. `toc: false` removes the page outline and its grid column.

API lists default to compact name-first signatures, including parameter lists,
property/field types and method return types. Full Raven declarations remain on
individual pages; `memberListStyle: signatures` opts into them in lists. Icons
identify classes (C), interfaces (I), enums (E), unions (U), delegates (D) and structs (S), with
a separate static-member badge. `apiNavigationRoot: docs` enables the section
sidebar on authored pages under `docs` as well as generated reference pages.

The publisher reads CLI metadata and XML documentation, including attributed
custom union carriers. The metadata contract is described in the pinned Raven
source's `docs/lang/spec/dotnet-implementation.md`. Recognition does not add a new
runtime extraction ABI. A portable archive costs repository space, but avoids
compiling the compiler on every website build and makes the generator version
reviewable and reproducible. Publication remains a separate manual workflow.

Raven snippets use the Raven website's shared Highlight.js grammar and token
colors, including code fences and generated signatures. The publisher bundles
Highlight.js 11.11.1 core and its BSD license locally, so viewing snippets needs
no CDN access.

The shared header also supplies a keyboard-accessible Light/Dark/Auto icon menu
and a configurable favicon. neoCLR's CSS supplies matching light/dark project
colors; it does not implement a separate theme switcher or snippet lexer.

Member pages derive Parameters and Property value/Return value/Field value/Event
type sections from symbols, with navigable types even without authored comments.
XML and Markdown descriptions enhance that contract; remarks and examples can be
added over time. The owner is labeled Declaring type, distinct from the value type.

## Semantic reference browsing

The pinned publisher includes inherited instance members with linked origins and
interface implementation/default information. Static members appear only on the
type that declares them. Readers can group by member kind or declaring type and
toggle inherited members; these preferences persist in their browser.

neoCLR opts into applicable extensions from `System.Linq` and
`System.Runtime.Reflection`. Extensions join Methods or Properties when grouping
by kind, and their declaring container when grouping by type. A purple E badge
distinguishes extensions; ordinary static members have a red S badge.

Union and enum sidebar entries link directly to their overview. Union cases have
individual pages linked from the union, and generic companion members belong to
the logical union. The sidebar scrolls independently of the article.

See [RavenDoc integration](../../docs/ravendoc-integration.md) for authored API
content, upstream ownership, compiler interpretation changes and validation.

## Union documentation compatibility (2026-10-05)

The pinned publisher now includes the shared compiler fix from Raven main
`137431e44109c6050af23075401cc2090784729c`. A projected union case first looks up its
logical documentation ID, then falls back to its physical CLI carrier type ID.
This restores the already-authored Option/Result case summaries without changing
the documented union shape or adding duplicate public carrier pages.

## Callback contract presentation (2026-10-05)

Publisher `8ddd73c3187f1c31e60816f8598fd1da59c6d4fa` uses Raven function syntax in
member-list type uses and framework Func/Action contract tables. Named delegate
declarations keep their names; function return type links are retained recursively.
This corrects presentation of the CLI documentation reference without changing native
function semantics or introducing a Func identity into NeoCLR.

Validation: 20 source/metadata RavenDoc generation checks, all 18 neoCLR website
checks, a checked 1,803-page site build and browser inspection of Task.Run's parameter
table and String.FlatMap. The actual pages show `() -> void`, `() -> Task<T>` and
`char -> Iterable<U>`. Parameter names no longer wrap mid-identifier. The correction
was independently integrated/pushed to Raven main and cherry-picked into the native
integration branch (`2554322c4`).
