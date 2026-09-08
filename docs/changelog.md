# Changelog workflow

Every commit must include an update to [CHANGELOG.md](../CHANGELOG.md). This applies
to implementation, fixes, tests, documentation, build tooling and maintenance work.
The changelog is a maintained release summary, not a verbatim copy of Git messages.

## During development

1. Update **Unreleased** using the current development date in `YYYY-MM-DD` form.
   Keep date headings in descending order.
2. If a related entry already exists for the same date, revise or extend it to
   describe the combined outcome. Otherwise add an entry under that date. Work on a
   later date gets an entry for the later date; do not silently rewrite earlier history.
3. Describe what changed for developers or users, including relevant API, artifact,
   behavior or migration changes. Related tests and documentation can extend the
   feature entry. Standalone maintenance gets a concise entry of its own.
4. Distinguish implemented capabilities from design decisions and future plans. Link
   to the relevant guide or example. Do not claim release validation from an earlier
   tag or infer unsupported platforms, performance, safety or compatibility guarantees.
5. Stage the changelog with the associated changes before committing. Review
   `git diff --cached -- CHANGELOG.md` and `git diff --cached --check` as part of the
   commit review. Splitting work into multiple commits means each commit includes
   its own changelog update; do not defer all updates to the final commit.

A documentation-only commit may update an existing same-date documentation entry;
it does not need a new feature announcement. Commit hashes are optional references,
not a requirement for new entries, since a commit cannot contain its own final hash.

## Published history

Once a version is published, its changelog section and published release notes stay
unchanged, including during later cleanup or backfills. Put a correction or erratum
under Unreleased with a reference to the affected release. Never move a later feature
into a published version or rewrite a published limitation to describe current code.

The initial historical reconstruction is based on the tagged source and retained
Preview 1 notes. The development README describes the current tree; it should link
readers to the published notes when they need the released feature set.

## Preparing a release

When preparing an explicitly requested release, choose the actual version/date and
move the selected Unreleased material into that release section. Preserve the dated
history, review migration guidance and limitations against the exact candidate, and
start a fresh Unreleased section. Freeze the release section once published.

Keep the release version and publication date unset while development is ongoing.
Updating the changelog does not itself authorize tagging, pushing or publication.
Follow the [source-release process](source-release.md) for candidate validation,
archive contents, licenses and release evidence.

Repository agents receive the same per-commit rule in [AGENTS.md](../AGENTS.md).
