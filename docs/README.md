# Documentation

The [platform roadmap](platform-roadmap.md) is authoritative for work priorities
unless the author directs otherwise. It organizes themed milestones and concrete
sample products. The [HTTP POC plan](http-poc-roadmap.md)
details the first major milestone.

Start with the [runtime and Raven walkthrough](runtime-raven-preview.md).
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

## Current work sequence — 2026-09-19

Organize and commit the documentation first. Then port the remaining managed System
library implementation from neoIL to Raven with the existing API preserved. Align
the API with the supplied proposals in a subsequent step. The
[System.Runtime project](system-runtime-assembly.md) is now established as
`System.Runtime.rvnproj`. The next API slice implements RuntimeContext and shared
Info interfaces, with Object.GetTypeInfo as canonical instance acquisition.
