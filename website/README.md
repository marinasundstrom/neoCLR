# Project website

A small static site presenting the current runtime/Raven experiment. Plain HTML/CSS,
no JavaScript runtime, remote fonts, analytics or package installation. The Python
build inserts escaped excerpts from the executable Raven and neoIL examples, checks
local assets and anchors, and writes only the public site to `target/website`.

```sh
python3 scripts/build-website.py
python3 -m http.server 8765 --directory target/website
```

Open http://localhost:8765. Check desktop and narrow mobile layouts when editing CSS.
Update the release-status paragraph when the runtime/Raven assets are actually
published; do not imply that the local candidate is already downloadable.

`.github/workflows/pages.yml` is separate from runtime CI. Relevant pushes to main
and manual dispatch build and deploy; pull requests build/check without deployment.
Only the deployment job receives Pages/OIDC permissions. Publication uses the
`github-pages` environment and GitHub Actions as the repository's Pages source.
The expected project URL is https://marinasundstrom.github.io/neoCLR/.

The workflow follows GitHub's [custom Pages workflow documentation](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).
To enable once, use Settings → Pages → Build and deployment → Source → GitHub Actions,
or `gh api --method POST repos/marinasundstrom/neoCLR/pages -f build_type=workflow`.
The site is independent of publishing a runtime release or a Raven Marketplace extension.
