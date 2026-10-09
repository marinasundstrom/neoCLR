# Documentation

The [platform roadmap](platform-roadmap.md) is authoritative for work priorities
unless the author directs otherwise. It organizes themed milestones and concrete
sample products. The [HTTP tracker](http-capabilities.md) owns the first milestone finish line.
The [theme trackers](platform-roadmap.md#theme-trackers) group current work across
HTTP/networking, runtime/language, library/data and tooling/release.

For the native macOS arm64 bundle, start with [Preview 12](preview-12-release-notes.md)
and its [bootstrap/package procedure](native-poc-bundle.md).

For source and older workflows, see the [build instructions](../README.md#build-and-run-a-sample),
[website build](../README.md#build-the-website), or
[runtime and Raven walkthrough](runtime-raven-preview.md).
The [changelog](../CHANGELOG.md) records implemented changes; proposals and plans
are not evidence that an API is available.

| Section | Contents |
| --- | --- |
| [Guides](guides/README.md) | Running the library, authoring workflow, debugging and the System.Runtime assembly plan |
| [Original proposals](proposals/README.md) | Author-supplied API and architecture proposals, with links to maintained design notes |
| [Design and direction](design/README.md) | API policy, comparisons, tradeoffs, roadmap and unresolved decisions |
| [Runtime and language reference](reference/README.md) | Runtime contracts, neoIL and the earlier Neo language |
| [Raven integration](integration/README.md) | Compiler contracts, importer boundaries, library ports and validation |
| [History](history/README.md) | Published releases, validation records and development conversations |
| [Contributing](contributing/README.md) | Research, changelog and release workflow |
| [Experiments](experiments/README.md) | Executable probes and their supporting notes |

## Platform foundations

- [Architecture](architecture.md): source, runtime layers, services and execution modes.
- [Metadata format](metadata-format.md): implemented PE/NEOX transport, payload and ownership.

## Organization and status

The section indexes categorize the existing documents while preserving their paths.
Those paths are used by published release notes, executable checks and the website.
Keep new original proposals in `proposals/`; preserve their supplied wording and
record subsequent implementation decisions in the maintained design/integration notes.
Do not silently rewrite a proposal to describe today's implementation.

For new documentation, add a link to the relevant section index. Each substantive
design should distinguish proposed behavior, implemented behavior, experiments and
open validation. Follow [design research](design-research.md) and the
[changelog workflow](changelog.md). Published release documents remain frozen.
