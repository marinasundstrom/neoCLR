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
namespace expands to its overview, including namespaces without direct types.
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
