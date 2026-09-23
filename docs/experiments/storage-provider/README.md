# Provider-bound File and Directory exploration

**Development experiment, 2026-09-23. Application-owned Storage/capability types using development System.Streams APIs.**
The same Raven `RoundTrip(Directory)` workflow writes and reads a real UTF-8 file
and then runs against a one-slot memory provider. The application sees File and
Directory objects; each retains the provider that interprets its address.

Run against matching development artifacts:

```sh
python3 docs/experiments/storage-provider/verify.py --toolchain-root /path/to/bundle
```

The verifier builds the saved MSBuild project with all four Raven files, runs in a
disposable working directory, checks
exact output and actual disk bytes, and removes its fixture. No sample file is
written into the checkout. It also checks that rejected oversized writes preserve
contents, UTF-8 byte limits count bytes, invalid direct-child names fail, and separate
provider instances do not share state.

## Evidence and architectural choice

An initial draft let Directory combine paths with the host Path helper. Reviewing
that draft showed a provider leak: generic storage objects would impose filesystem
syntax on memory, archive or remote providers. The tested revision delegates child
address resolution to the provider. HostStorage combines native paths; MemorySlotStorage
uses `directory::name` logical keys. File retains its name separately instead of
recovering it with host path parsing. This separation works without runtime or
compiler changes and uses ordinary Raven interface dispatch.

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
  current requirement to dispatch every operation through a provider and string key.
- **Path identity:** strings are temporary provider-owned addresses, not a selected
  public Path contract. The direct-child helper rejects empty, dot, dot-dot, slash,
  backslash and NUL names. This is name validation, not a sandbox: native symlinks and
  direct provider calls still use host permissions.
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

The author now directs Storage alignment, including exploring a Path value object,
after this working read/write slice. Keep addresses and provider identity separate;
do not promote the temporary text-helper provider contract unchanged.

The verifier also compiles `Contracts.rvn` separately through the normal SDK. It
checks missing/empty/directory paths, rejected ranges before input consumption or
output mutation, managed-buffer aliases and untouched elements, zero-count calls,
Closed errors, and 70 open/close cycles against the 64-live-file budget.

Validation on 2026-09-23: the product sample and separate wrapper contracts passed
through normal MSBuild compilation. Five native file-resource unit cases, eleven
VM integration cases and eight existing Thread/worker cases passed. All Raven
library slices regenerated, snapshot checks matched, and the website/API-reference
build and four website tooling tests passed. No cross-platform matrix was rerun.
