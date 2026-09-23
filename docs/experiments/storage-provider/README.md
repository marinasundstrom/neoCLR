# Provider-bound File and Directory exploration

**Development experiment, 2026-09-23. Application-owned Storage/capability types using development System.Streams APIs.**
The same Raven `RoundTrip(Directory)` workflow writes and reads a real UTF-8 file
and then runs against a one-slot memory provider. The application sees File and
Directory objects; each retains the provider that interprets its address.

Run against matching development artifacts:

```sh
python3 docs/experiments/storage-provider/verify.py --toolchain-root /path/to/bundle
```

The verifier builds the saved MSBuild project with all five Raven files, runs in a
disposable working directory, checks
exact output and actual disk bytes, and removes its fixture. No sample file is
written into the checkout. It also checks that rejected oversized writes preserve
contents, UTF-8 byte limits count bytes, invalid direct-child names fail, and separate
provider instances do not share state.

## Evidence and architectural choice

An initial draft let Directory combine paths with the host Path helper. Reviewing
that draft showed a provider leak: generic storage objects would impose filesystem
syntax on memory, archive or remote providers. The earlier tested revision delegated child
address resolution to the provider and used `directory::name` memory keys. The next
exploration now introduces shared validated logical Path syntax: Directory parses
child paths, while providers resolve them. HostStorage maps them under a configured
native root; MemorySlotStorage uses rooted logical keys. File retains its display
name separately. Both experiments use ordinary Raven interface dispatch without
runtime or compiler changes.

The types use the **StorageExperiment** namespace. Their documented surface is in
[the on-site API guide source](../../../api-docs/storage-experiment.md); it is linked
from `/docs/` and the Files feature page, with the exact tested source downloadable.
It is a manual reference to application-owned experiment types, not a DocFX metadata
claim that these types are part of the platform's core reference assembly.

## Provisional choices, costs and alternatives

- **Addresses versus lookup:** FileAt constructs a descriptor without touching
  storage. It is deliberately not named GetFile: the storage proposal's GetFile
  performs a lookup returning an error when the file does not exist. Lazy descriptors
  are useful for destinations that do not exist yet, but defer missing/permission
  errors and cannot promise an item still exists when used. We still need to compare
  both operations, not replace all proposal lookups with lazy addressing.
- **Provider ownership:** a Directory passes its provider to a File, and the File
  retains it. No global provider replacement or native handles enter the application.
  A future provider could instead return its own File implementation, avoiding the
  current requirement to dispatch every operation through a provider and Path value.
- **Path identity:** `Path.Parse(text)` returns `Result<Path, InvalidPathError>`.
  Private construction and read-only text keep accepted syntax valid. The immutable
  reference class provides explicit lexical Equals, not filesystem identity or value
  operators. Logical syntax uses slash-separated names, rejecting parent/dot segments,
  empty names, NUL, colon and backslash; standalone `/` and `.` are accepted. This
  provisional grammar excludes native Windows drive/UNC spellings. HostStorage takes
  a separate native root string; root mapping is not a symlink-safe sandbox.
- **Call-site ergonomics:** provider and descriptor APIs currently require Path.
  String overloads where a path is expected remain open; the assistant recommends
  parsing and delegating to the typed implementation. More Path operations are deferred.
- **Text:** ReadText/WriteText temporarily reuse the existing bounded whole-file
  helpers and their error families. A final provider should expose byte streams;
  text/encoding convenience belongs above that boundary. The byte workflow now
  uses native-resource-backed System.Streams wrappers independently of those helpers.
- **Lifetime:** these descriptors hold no open file; the existing helper opens and
  closes per text operation. Byte streams require explicit Close and otherwise live
  until invocation teardown. Disposal across suspension is still unproven.
- **Memory provider:** retains one text slot and one separate byte slot during this
  migration experiment. They are not coherent views of the same file. Byte streams
  share a list, have separate input cursors and transfer at most two bytes per call.
  This tests application progress loops, not filesystem conformance or concurrency.

The existing compiler reference's error cases require explicit carrier construction
inside inferred Result.Error values; direct nested case construction did not compile.
The experiment keeps this localized in the provider implementation rather than
claiming unsupported source syntax. No Raven compiler code or target policy changed.

## .NET comparison

The [.NET 10 FileInfo constructor](https://learn.microsoft.com/en-us/dotnet/api/system.io.fileinfo.-ctor?view=net-10.0)
creates a path wrapper, providing a familiar comparison for a descriptor distinct
from an open stream (reviewed 2026-09-23). neoCLR's experiment additionally retains
an explicit provider so the consumer can run against different storage backends.
The benefit is test substitution and provider-owned addressing; the cost is another
contract, retained references and unresolved provider identity/validation rules.
This is not a claim of performance or general superiority over .NET.

The [file-resource experiment](../file-streams/README.md) compares native file lifetime
and chunked I/O with FileStream and Rust's host APIs. The sample now connects File
objects to directional byte streams with explicit close. System.Streams wrappers
hide native handles, return StreamError cases and are documented through generated
DocFX metadata plus the Flush renderer exception in the on-site guide.

## Byte workflow evidence and next slice

`ByteRoundTrip(Directory)` writes `Hello, värld!` using three-byte buffers and loops
until every write completes. It flushes and closes, opens an input, reads until EOF,
closes, and decodes the accumulated bytes (limited to 64). It also checks exclusive
creation and repeated close/Closed failures. Memory transfers are capped at two
bytes to expose incorrect all-or-nothing assumptions; the disk fixture verifies
exact UTF-8 bytes. No Raven compiler changes were needed.

FileInputStream and FileOutputStream separate the .NET FileStream directions,
while retaining the familiar array/offset/count partial-transfer baseline. The
extra types and shared provisional error family are tradeoffs, not a superiority
claim. This first wrapper is synchronous, has no finalizer/automatic disposal and
makes no durability guarantee. Memory views, asynchronous ownership and general
System.Streams capability interfaces are still open.

Storage alignment has begun with the application-owned Path experiment. Keep
syntax validation separate from provider lookup and identity; do not promote the
temporary text-helper provider contract unchanged. The core System.Storage.Path
string helpers remain unchanged. See the [complete Path contract](../../../api-docs/storage-experiment.md#path-value-object)
for grammar, costs and comparison with .NET System.IO.Path.

The verifier also compiles `Contracts.rvn` separately through the normal SDK. It
checks missing/empty/directory paths, rejected ranges before input consumption or
output mutation, managed-buffer aliases and untouched elements, zero-count calls,
Closed errors, and 70 open/close cycles against the 64-live-file budget.

Validation on 2026-09-23: the product sample and separate wrapper contracts passed
through normal MSBuild compilation. Five native file-resource unit cases, eleven
VM integration cases and eight existing Thread/worker cases passed. All Raven
library slices regenerated, snapshot checks matched, and the website/API-reference
build and four website tooling tests passed. No cross-platform matrix was rerun.

Path validation on 2026-09-23: the normal SDK sample and stream contracts passed
again with the disk provider rooted in a scratch `sandbox` directory. PathContracts
checks valid and invalid spellings, lexical equality, provider resolution and valid
but missing files. Separate compiler checks reject direct construction and Text
assignment. The verifier confirms no sample files escape to the parent working
directory; this fixture check does not establish a filesystem sandbox.

## Lookup and identity evidence (2026-09-23)

The next question is what a lookup can guarantee before opening a stream. Run the
host-only comparison independently of the Raven sample:

```sh
python3 docs/experiments/storage-provider/verify_lookup.py
```

This compiles `LookupProbe.rs` and `LookupProbe.cs` in a disposable directory. It
prints installed toolchain versions and targets the installed .NET SDK's major
version. It does not implement or advertise a new neoCLR API. On macOS, select a
working SDKROOT if the default SDK cannot link Rust executables.

Observed on macOS arm64 with Rust 1.95.0 and .NET SDK
11.0.100-rc.1.26425.128: five Rust cases and the .NET comparison passed. The probes
show missing/file/directory distinctions, stale metadata after removal, independent
provider roots, and address replacement while an existing stream retains the old
file. The Unix-only case distinguishes following a symlink from inspecting it,
including a dangling link. No Windows/Linux runs or permission-denial probes were
performed; those remain targeted platform validation, not inferred coverage.

### Comparison and provisional direction

Primary contracts reviewed 2026-09-23:

- [.NET FileInfo.Exists](https://learn.microsoft.com/en-us/dotnet/api/system.io.fileinfo.exists?view=net-10.0)
  caches observations until Refresh and reports false for a directory or lookup
  error. The local .NET probe confirms caching across creation/removal and shows
  that FileInfo retains an address rather than an open item. The documented .NET 10
  contract is the comparison baseline; the executable used the installed .NET 11 RC.
- [Rust fs::metadata](https://doc.rust-lang.org/std/fs/fn.metadata.html)
  returns metadata or an I/O error and follows links. Our probe checks the host
  behavior with Rust 1.95.0; the online standard-library documentation can be newer.
  A metadata query is a possible host adapter, not a new runtime mechanism.

Alternatives are a boolean Exists, opening/closing a stream as a probe, or a typed
metadata lookup. A boolean collapses useful failures. Opening content introduces
read-access requirements and consumes the stream budget, so it is not an equivalent
lookup. Prefer investigating metadata lookup with typed errors while keeping the
actual open operation authoritative. This adds an I/O operation and does not remove
races; consumers only needing bytes should be able to open directly.

Proposed contract for the next implementation slice, **not shipped API**:

- Keep FileAt as descriptor construction without I/O. Add GetFile only when a
  provider can actually query kind/existence without opening content.
- A successful lookup returns a provider-bound address observed as a file. It does
  not retain an open handle or guarantee later availability, identity or permissions.
  Opens must perform their own checks and can return NotFound or other errors.
- Keep lexical Path equality distinct from provider address equality and stable item
  identity. Do not infer same-file identity from matching strings, or across roots.
- Consider NotFound, WrongKind, AccessDenied and IoFailure lookup errors separately
  from stream states such as Closed or invalid buffer ranges. Native mapping and
  provider-specific details still need implementation and tests.
- Select and test link-following behavior explicitly. Provider root mapping still
  does not confer sandbox authority. Async completion remains open with scheduling;
  do not advertise synchronous experiments as the final provider contract.

Before a shared lookup interface, make the memory provider's text and byte views
coherent: its current separate slots cannot honestly answer a single file lookup.
Then add a native metadata adapter and matched disk/memory checks, including a
failed open after successful lookup. Independent .NET filesystem abstractions and
permission-error portability remain research gaps before promoting this API family.
