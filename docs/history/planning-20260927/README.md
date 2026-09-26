# Planning archive — 2026-09-27

These plans preserve the pre-consolidation record. Current priorities belong to the
[platform roadmap](../../platform-roadmap.md); current status belongs to its theme trackers.
Original wording is retained, with relative links relocated and an archival banner added.

- [http-poc-roadmap](http-poc-roadmap.md)
- [issue-fix-roadmap](issue-fix-roadmap.md)
- [platform-backlog](platform-backlog.md)
- [platform-roadmap](platform-roadmap.md)
- [roadmap](roadmap.md)
- [runtime-api-plan](runtime-api-plan.md)

## Earlier documentation-index sequence

The following is retained from docs/README.md, not an active instruction.

## Current work sequence — 2026-09-19

Organize and commit the documentation first. Then port the remaining managed System
library implementation from neoIL to Raven with the existing API preserved. Align
the API with the supplied proposals in a subsequent step. The
[System.Runtime project](../../system-runtime-assembly.md) is now established as
`System.Runtime.rvnproj`. The next API slice implements RuntimeContext and shared
Info interfaces, with Object.GetTypeInfo as canonical instance acquisition.
