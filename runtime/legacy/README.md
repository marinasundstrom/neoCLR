# Legacy runtime fixtures

All retained pre-native-source library and CLI bridge material lives here:
`bootstrap/` holds the old System manifest, mirrors and service declarations;
`raven/` holds bridge fragments, generated files and historical receipts.
Current library sources live in the sibling runtime component projects. Active
compiler-service templates live in `../compiler-services/`.

These files remain inputs to the embedded bootstrap and low-level tests. Update
consumers before removing them. Receipt paths and hashes retain their historical
meaning; do not rewrite receipts to imply fresh validation after relocation.

## Historical Neo bootstrap carriers

These snapshots preserve the historical Neo error-carrier and BindingFlags ABI from commit
`00c1828a`. The default `runtime/legacy/bootstrap/System.neoil` manifest still serves that bootstrap
profile, which has neither the Raven Object/interface profile nor its generated
union protocol. Importing the newly migrated union bodies there breaks loading.

The Raven profile builder replaces these includes with the current normal union
implementations in `runtime/legacy/raven`. Do not regenerate these snapshots from the new
union sources, add them to the current API reference, or use them as a convention
for new unions. Remove them when the historical bootstrap profile is retired or
explicitly migrated. No per-case Is*/Get* requirement is restored for Raven.

Optional and Result additionally preserve their manual snapshots from `1404454e`.
The Raven profile uses the standard source unions; the bootstrap profile retains
its existing predicates, checked accessors and storage contract.

The Propagatable snapshot preserves the Neo bootstrap profile's conditional-output
ABI. Raven's source-generated Propagatable and Optional/Result methods use ordinary
initialized outputs. TaskOutcome is Raven-profile-only and requires no legacy copy.

## Containment validation (2026-10-29)

- `cargo test --test source_files --test runtime_services --test raven_collections`:
  31 tests passed, including embedded manifest equivalence and CLI execution.
- Expanded bootstrap and Raven collection profiles compared byte-for-byte equal
  before and after relocation; 416 neoIL include targets resolve.
- 635 historical payloads and active templates compared byte-for-byte equal to
  their prior tracked versions. Modified Python/XML syntax and legacy project /
  PostgreSQL template paths were checked.
- `verify_source_ownership.py` still reports two pre-existing unowned declarations:
  `neoCLR.Runtime.StringFromSequence` and `get_Value() -> System.Object`. The same
  audit failures reproduce against the saved pre-move profile; this cleanup does
  not claim to fix that audit.

No website build or full native SDK rebuild was needed for this source relocation.
