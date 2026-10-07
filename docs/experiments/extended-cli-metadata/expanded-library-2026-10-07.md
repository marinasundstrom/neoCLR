# Bounded larger libraries and full-source emission — 2026-10-07

The 197-input diagnostic System build contains 430 types and 2,643 functions. Its
10,080,354-byte host JSON requires 8,770,829 CBOR bytes, exceeding schema 3's
8,388,576-byte payload allowance by 382,253 bytes. Function metadata/bodies account
for 8,139,504 bytes. This is a transport budget failure, not a binding failure.

## Decision and compatibility

Required execution schema **4** uses the same CBOR grammar and native semantic
format 5 as schema 3, with a **16 MiB envelope**. Schema 3 remains capped at 8 MiB;
schemas 1/2 retain their original limits. Library writers choose schema 3 when the
encoded payload fits and schema 4 otherwise. Both the .NET metadata library and
Rust standalone writer follow this rule. Old readers reject required schema 4.

All other library limits remain: 32 MiB host/reconstructed JSON, 2,097,152 CBOR
items including keys, depth 64, and **16 MiB total PE image**. The PE cap includes
CLI projection and container overhead, so not every maximum standalone envelope
fits a PE. Invalid encodings, oversized data and failed semantic encoding reject
before compiler output publication. This does not introduce new guest APIs.

Reuse the [metadata-root/CLI design comparison](README.md): CLI tables, signatures
and instruction semantics are unchanged. The custom #Neo envelope is a temporary
native execution representation; this budget is our implementation policy, not a
.NET/CLI limit. Alternatives were splitting the diagnostic build, eliminating
redundant payload data, or increasing schema-3 bounds silently. A distinct required
schema preserves prior readers' limits and supplies headroom without making a new
indexed/compressed encoding a prerequisite. More bytes can mean more allocation and
validation work; no speed improvement is claimed. Existing node/depth and PE bounds
remain, and payload compaction remains future work.

## Evidence and remaining work

- [Recorded artifacts, hashes, commands and outcomes](expanded-library-2026-10-07.json).
- 164 C# metadata groups pass: schema selection, round trips, exact byte boundary,
  one-byte overflow, unchanged JSON/node/depth limits, old-reader required-schema
  rejection, and falsely downgraded schema-3 rejection. Stale unsupported-version
  fixtures now use schema 5; malformed framing/digest coverage is unchanged.
- Four Rust binary-codec and two container tests pass, including direct writer
  schema selection and downgrade rejection. Formatting and API snapshot checks pass. Website feature review found no
  guest API/example changes; full-bootstrap claims remain explicitly pending.
- A 9.2 MiB API-authored PE library plus separate digest-bound consumer verify and
  execute with exit 42 and empty output. This is transport/link/execution evidence,
  not a performance measurement or whole-System runtime qualification.
- [All 197 System inputs emit](native-bootstrap-emitted-2026-10-07.json) an 8.6 MiB
  native PE with no binding/encoding errors. Compiler is ca4aeccfb-equivalent with
  this metadata DLL; source/ownership/dependency hashes are recorded. The fixture
  still names its diagnostic owner Numbers; this is not proposed production layout.
- Loading that whole artifact as an explicit dependency with the selected Object
  root and retained seed rejects `invalid nominal array backing storage contract`.
  Next inspect the source Array shape against runtime admission. No full-System
  execution or seed-free bootstrap is claimed.

## Packaging direction

The author suggested separating System.Runtime/core from System.Data,
System.Networking and System.Web. This is a packaging candidate, not an implemented
split or approval of particular ownership boundaries. Keep the aggregate build as
coverage evidence. Before splitting, inventory actual dependencies and native service
ownership, avoid core-to-optional cycles, then compile and execute each consumer with
library sources absent. Assembly identities and manifests must be updated together;
merely moving source files does not prove separately compiled import/emission.
