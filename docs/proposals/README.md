# Original proposals

[Documentation index](../README.md) · [Maintained design notes](../design/README.md)

These are the original proposal texts supplied by the author and added to the
repository in the documentation slice of 2026-09-19. Their original composition
dates are not established here. They describe proposed contracts, not a statement
of the currently executable API. The texts are preserved as supplied.

The [unified platform roadmap](../platform-roadmap.md) organizes these inputs into
themed milestones and concrete sample products. The [2026-09-23 HTTP POC roadmap](../http-poc-roadmap.md) provides the complete
current proposal triage, including newer collection, storage, environment, type-system,
metadata and dynamic-dispatch proposals not listed in the original import table
below. It separates near-term application needs from exploratory designs. The author
clarifies that proposals may conflict and do not prescribe the eventual API: use
them as inputs to scenario-driven experiments, not specifications to reconcile into
a final design.

| Proposal | Maintained design or implementation context |
| --- | --- |
| [Runtime architecture and language interoperability](runtime-architecture.md) | [Platform direction](../platform-direction.md), [execution architecture](../execution-architecture.md), [System.Runtime assembly](../system-runtime-assembly.md) |
| [Strings and encoding](string-api.md) | [Text model](../text-model.md), [Raven string API](../raven-string-api.md) |
| [Date and time](datetime-api.md) | [Date/time design](../date-time-design.md), [current calendar API](../raven-calendar-api.md) |
| [Globalization](globalization-api.md) | [Globalization design](../globalization-design.md) |
| [Introspection, Reflection, RuntimeContext and Emit](introspection-and-reflection-api.md) | [Introspection design](../introspection-design.md), [current descriptor port](../raven-reflection-api.md) |
| [Storage/filesystem](storage-api.md) | [Filesystem design](../filesystem-design.md), [current file API](../raven-file-api.md) |
| [Streams](streams-api.md) | [Stream design](../stream-design.md) |
| [Task and async](task-model.md) | [Async API design](../async-api-design.md), [implementation assessment](../async-state-machine-assessment.md) |
| [Networking and HTTP](network-api.md) | [Platform backlog](../platform-backlog.md); the proposal does not establish an implemented networking API |

The author requested source porting before API alignment on 2026-09-19. Differences
between these proposals and the current library remain explicit follow-up work.
Consult the [conversation record](../development-timeline.md) for directions and
corrections, and the [changelog](../../CHANGELOG.md) for implemented slices.
