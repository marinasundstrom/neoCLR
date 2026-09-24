# neoCLR website

One RavenDoc site contains the product overview, feature guides, proposals, setup
instructions and the API reference at `/docs/`. Author pages in Markdown under
`content/`; API guides remain in `../api-docs/*.md`. RavenDoc supplies the page
layout, light/dark theme, outline, tables and Raven code highlighting. Customize
branding, navigation and the release notice in `site.json`, and presentation in
`custom.css`. Do not reintroduce standalone HTML page shells; HTML body content is supported.

## Build and preview

Requires Python 3.9+ and the .NET 10 SDK:

```sh
python3 scripts/build-website.py
python3 -m unittest discover -s scripts -p 'test_build_website.py'
python3 -m http.server 8765 --directory target/website
```

Run from the repository root. No Raven checkout, Node install, NuGet restore or
DocFX is required. The repository carries a checksum-verified portable
[RavenDoc build](../tools/ravendoc/README.md). The build checks the reference snapshot,
expands tested sample excerpts, generates the complete site, and validates local
assets and anchors. Output is only `target/website`; tool binaries and reference
assemblies never enter the deployed artifact.

Use ordinary Markdown headings, lists, tables and fenced `raven`, `neoil`, `xml`,
`shell` or `text` code. Existing explicit `<a id="…"></a>` anchors retain public
links; do not duplicate a generated heading ID. Quote type spellings such as
`Equatable<T>` in backticks. Prefer `[Type](xref:System.Storage.Path)` for API links.
Unresolved xrefs fail the build. Conceptual `.md` links become `.html` links.
Page paths under `content/` retain their public directory routes. Sample placeholders
such as `{{ARRAY_TOUR}}` inside fences expand from executable source files via
`scripts/build-website.py`; do not duplicate those samples in documentation.

Selected complete source/expected-output files and project ZIPs are still downloadable
from `/samples/`. Preserve release-specific examples in `samples/preview9/` until a
new release changes that baseline. See [API maintenance](../api-docs/README.md) for
snapshot refresh and coverage.

## Documentation can precede a release

The site tracks development and can be published before the corresponding runtime
release. Every content/reference page carries a visible notice from `site.json`
stating that documented features may not yet be released, plus a link to the latest
published release. Keep released examples, development changes and proposals labeled
locally too. Never infer release availability from a docs publication or CI success.
The compact notice is mandatory and validated by the website checks.

The Pages workflow validates relevant pushes and pull requests. It publishes only
on manual dispatch on `main`, with Pages/OIDC permissions limited to the deploy job.
Review the revision, release notice, downloads and page-specific availability before
**Project website → Run workflow → main**. The expected public URL remains
https://marinasundstrom.github.io/neoCLR/; relative links also work at a domain root.
A successful local build is not a deployment. The manual website workflow does not
build the runtime or create a release. If the author explicitly requests a site-only
publication while a push includes code changes, a `[skip ci]` commit message skips
push-triggered validation; then dispatch `pages.yml` manually on `main`. This is a
per-publication choice, not a permanent bypass of runtime validation. See
[GitHub skip instructions](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/skip-workflow-runs).


Follow [website maintenance](../docs/design/feature-pages.md) for feature/release
reviews. Inspect desktop and narrow layouts when changing CSS or upgrading RavenDoc.

## Page metadata and a custom landing page

Pages may be Markdown or HTML body fragments. Both use RavenDoc's shared shell.
The homepage is `content/index.html`, with a hero, feature cards and links to the
same guides/reference. An HTML page must not contain doctype/html/head/body wrappers.
Use ordinary links in HTML; Markdown also supports xrefs.

Optional front matter supports these scalar fields:

```yaml
---
title: A page title
layout: landing
toc: false
---
```

`layout` is `docs` (default) or `landing`; `toc` explicitly shows/hides the page
outline, independently of layout. `title` overrides the Markdown heading as browser
title and is required for HTML pages. Values may be plain, JSON-style double-quoted
or YAML-style single-quoted scalars. Duplicate/unknown fields, invalid booleans and
unclosed front matter fail the build. This is a small strict scalar subset, not
arbitrary YAML or a dynamic CMS. Site-wide `showToc` defaults to true. The homepage
sets it to false; long guides and generated API pages retain the useful outline.

API navigation lists use `"memberListStyle": "compact"` by default: names first,
properties and fields as `Name: Type`, and methods as `Name(parameters) -> Type`.
Generic arguments, parameter types and return types distinguish overloads, which
keep separate entries. Unit returns display as `()`. Optional parameters use `?`
and variadic parameters use `...`; reference-passing annotations remain visible.
Static members have an `S` badge on their icon, a tooltip and an accessible label. Icons indicate symbol kinds;
declaration keywords, accessibility and other declaration modifiers are omitted.
Set `"memberListStyle": "signatures"` in the site configuration (or pass
`--list-signatures` to the assembly CLI) to show full Raven declarations in lists.
Type and member detail pages always retain full signatures.

RavenDoc's shared renderer supplies an API Browser sidebar on generated reference
pages, listing selected namespaces and types. Namespace groups expand and collapse;
the current type is highlighted and its namespace opens automatically. Set
`"apiNavigationRoot": "docs"` to include the browser on authored pages under that
site-relative directory too. On screens up to 760px, Browse API opens a modal
off-canvas drawer with a close button, Escape support and focus containment.
The API Browser remains independent of the page's `toc` control.

The three navigation levels are independent: `site.json` `links` defines the main
menu (with optional nested `children`), `toc.yml` defines authored side-menu items
and where the generated API tree appears, and page headings define the outline.
RavenDoc loads the authored YAML into its own navigation model; it does not run
DocFX or require authors to repeat the generated namespace/type list.

## Themes, favicon and syntax highlighting

The header theme icon offers Light, Dark and Auto. RavenDoc persists the choice
and follows system changes in Auto; `custom.css` overrides both palettes for
neoCLR. `favicon` in `site.json` selects the N-logo SVG copied by the build.
RavenDoc embeds the Raven website's shared Highlight.js lexer and color rules,
so fenced Raven examples and generated declarations use the same highlighting
without a CDN. The generic implementation and demonstration site live in Raven.
