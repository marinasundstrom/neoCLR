---
uid: N:System.Storage.Metadata
---
## Host-path kind lookup

GetKind checks whether a host path denotes a supported file or directory and
returns EntryKind or StorageLookupError. Lookup is synchronous and does not retain
file contents or an open resource.

See [storage lookup](/docs/storage-lookup.html). The result describes the lookup
at that moment; a later open can still fail if the filesystem changes.
