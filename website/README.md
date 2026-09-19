# Project website

The What's next section presents the next release's API-shape demo/POC objective
and links to the separate proposal overview. Each proposal states its benefit and current
implementation status and links to the design record. Keep these descriptions
aligned with the library-preview plan as APIs become executable.

A small static site presenting the current runtime/Raven experiment, its influences, implemented features and open research questions. Plain HTML/CSS,
no browser JavaScript required for content or highlighting, and no remote fonts.
Google Analytics loads the Google tag for measurement ID `G-SVXYRRCEEK`. Node.js 22 and two pinned
build-time tokenizer dependencies provide Raven syntax highlighting. The Python
build inserts escaped, highlighted excerpts from the executable Raven and neoIL examples, checks
local assets and anchors, and writes only the public site to `target/website`.

```sh
npm ci --prefix website --ignore-scripts
npm test --prefix website
python3 scripts/build-website.py
python3 -m http.server 8765 --directory target/website
```

Open http://localhost:8765. Check desktop and narrow mobile layouts when editing CSS.
Update the release-status paragraph when the runtime/Raven assets are actually
published; do not imply that the local candidate is already downloadable.

`.github/workflows/pages.yml` is separate from runtime CI. Relevant pushes to main
and pull requests build/check without deployment. Manual dispatch on main builds
and deploys; pushes never publish. In GitHub Actions, choose Project website → Run
workflow → main only after reviewing the revision and release status.
Only the deployment job receives Pages/OIDC permissions. Publication uses the
`github-pages` environment and GitHub Actions as the repository's Pages source.
The expected project URL is https://marinasundstrom.github.io/neoCLR/.

The workflow follows GitHub's [custom Pages workflow documentation](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).
To enable once, use Settings → Pages → Build and deployment → Source → GitHub Actions,
or `gh api --method POST repos/marinasundstrom/neoCLR/pages -f build_type=workflow`.
The site is independent of publishing a runtime release or a Raven Marketplace extension.

The homepage embeds eleven excerpts: Result propagation, generic Void in neoIL,
optional query results, collection capabilities, UTF-8 slicing, query cardinality,
Func callbacks with a Void result, calendar validation, imported union patterns, the initial Clock API and RuntimeContext assembly discovery. The latter two are explicitly marked as development APIs, separate from the published download. The narrative covers
familiar semantics/metadata, the type system, the class library, Raven migration
and tooling, and the planned Raven-authored library and missing API work.
Edit the executable samples rather than duplicating code in HTML. The Pages workflow
also rebuilds when Raven sample files change. Feature captions link to their complete
samples and relevant contract documents; keep preview capabilities separate from
open research and avoid unsupported performance or compatibility claims.

The invitation welcomes discussion in the general sense, including questions,
criticism and use cases. GitHub Issues is an available contact route; the page does
not require the GitHub Discussions feature.

Highlighting follows MyServiceBus's TextMate/Oniguruma integration, adapted to static
HTML generation in `highlight.mjs`. The vendored Raven grammar comes from
`src/Raven.VSCode/syntaxes/raven.tmLanguage.json` at Raven revision
`246d697bf6c69ff8cc56ca4859879edd7c081e7d`; its MIT license is preserved in
`syntaxes/Raven-LICENSE`. Update the grammar deliberately and run the tokenizer test.
HTML/CSS, the logo and explicitly selected sample downloads are published; packages and WebAssembly stay build-time.
NeoIL remains readable plain code; Raven tokens receive syntax colors.

The primary order is Familiar, Runtime, Type system, Runtime class library, Migration,
Tooling, and What's next. Library examples carry their own topic labels; extra union
forms and deeper research are optional disclosures to keep the overview concise.

## Feature guides

`features/introspection/index.html` is the first in-depth development guide. It covers
TypeInfo acquisition, RuntimeContext discovery, tokens, Sequence results and sealed
member matching. Its code excerpts and downloadable program come from
`library-introspection-tour.rvn`; expected output is shared with the saved-project
check in `library-introspection-tour.expected.txt`. Run that check with a matching
built toolchain via `verify_project.py --collections --only IntrospectionTour` and
the usual project/runtime/bridge/system arguments.

The builder emits HTML throughout `website`, including the homepage, feature pages
and proposals overview, preserving paths
and copying only the selected public samples/assets. It validates relative links,
directory index links and cross-page fragments after every page is rendered. Raven
blocks are highlighted; blocks marked `data-language="text"` preserve plain output.
Run `python3 -m unittest discover -s scripts -p test_build_website.py` for the nested
link/download checks. The Pages workflow watches expected-output files as well as
Raven sources. Build locally and inspect desktop/mobile layouts before publishing.

This guide describes development APIs, not an update to a published runtime release.
TypeInfo is part of the sealed MemberInfo hierarchy. The String page shows the
minimal strict UTF-8 conversion slice; dynamic loading and broader text design
remain open.

## Keep the site aligned with the product

Follow [website structure and feature maintenance](../docs/design/feature-pages.md)
when changing features or preparing a release. Review site content with each feature:
update current behavior and limits, move implemented proposals into the appropriate
status, and keep development examples separate from published downloads.

The homepage is the overview. Feature pages explain the current implementation
(with examples where useful), plus a short “Where we’re heading” section.
`proposals/index.html` gathers open ideas and links their design records. It is not
a release checklist. Short pages are welcome: do not expand the API or documentation
just to appear complete. Keep the design open for user feedback.

Publication is deliberately manual and independent of code pushes. Until this
workflow change is pushed to GitHub, the previously configured remote workflow
remains in effect. This local change does not publish the site.

Additional current-state notes cover outcomes, collections/queries, dates/clocks
and bounded UTF-8 files. Their snippets reuse the same executable samples as the
homepage and target checks. The feature-page index is `/#feature-pages`; keep it
current when adding or removing pages.
