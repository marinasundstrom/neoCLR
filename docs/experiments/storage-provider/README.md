# Provider-bound File and Directory exploration

**Development experiment, 2026-09-23. Application-owned providers using integrated System.Storage descriptors, Path and System.Streams APIs.**
The same Raven `RoundTrip(Directory)` workflow writes and reads a real UTF-8 file
and then runs against a bounded memory provider. The application sees File and
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
native root; MemoryStorage uses rooted logical keys. File retains its display
name derived from Path. The original experiments used ordinary Raven interface
dispatch; the subsequent platform integrations are recorded below.

The fixture providers use the **StorageExperiment** namespace; File/Directory and
the byte/lookup interfaces now use **System.Storage**. Their documented surface is in
[the on-site API guide source](../../../api-docs/storage-experiment.md); it is linked
from `/docs/` and the Files feature page, with the exact tested source downloadable.
The guide distinguishes fixture types from integrated platform types, which also
have generated DocFX member coverage.

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
- **Memory provider:** MemoryStorage replaces MemorySlotStorage. Text and stream
  operations share one byte payload per address. There are at most eight addresses,
  each with at most 64 KiB of current contents; nested paths are flat keys, not a
  directory tree. Separate input cursors and two-byte transfers exercise loops.
  Text replacement publishes a new payload; already-open streams retain the old
  one. This is a provisional policy, not a common host/memory identity guarantee.
  Retained old payloads are outside the current-content limits.


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
makes no durability guarantee. InputStream and OutputStream are now platform
interfaces implemented directly by the file classes and the sample memory classes.
Memory views and asynchronous ownership remain open.

Storage alignment began with an application-owned Path experiment; Path is now integrated. Keep
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

Provisional contract recorded before the lookup implementation below (historical design input):

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

The subsequent coherent-memory slice resolves the separate text/byte slot obstacle.
The typed metadata adapter and matched disk/memory lookup checks are now implemented
as described below, including a native failed open after successful lookup. Independent .NET filesystem abstractions and
permission-error portability remain research gaps before promoting this API family.

## Coherent text and byte contents (2026-09-23)

The same disk/memory product now reads a text-created file through an input stream,
then reads a stream-created file through the text helper. The verifier checks
`MemoryContracts.rvn` separately: invalid UTF-8 and byte-limit ordering, empty files,
exclusive creation after text writes, relative/rooted aliases, root kind, replacement
with retained input/output streams, capacity and preservation after rejection.
The eight-address limit returns StreamError.LimitExceeded for creation; the temporary
text error family can only return FileWriteError.WriteFailed for address exhaustion.
This mismatch is evidence for the next Storage error design, not an ideal final API.

The implementation uses ordinary Raven lists, UTF-8 conversion and interface dispatch.
No runtime hooks or compiler changes are needed. A small linear address search keeps
this bounded prototype readable; there is no performance claim. Text conversion
copies bytes and publishes a replacement list only after validation. Existing streams
retain their original list, including readers observing writes to that original list.
An alternative shared resizable file object could preserve item identity through
truncation, but adds a lifetime/mutation contract to settle with disk providers.

Compared with [.NET File.WriteAllText](https://learn.microsoft.com/en-us/dotnet/api/system.io.file.writealltext?view=net-10.0)
(reviewed 2026-09-23), the UTF-8-without-BOM representation is familiar. .NET truncates
and overwrites an existing file; this memory prototype publishes a different payload.
Do not infer matching already-open-handle behavior. The benefit is that rejected text
writes cannot alter visible contents; costs include copying and retained old payloads.
The earlier [lookup probes](#lookup-and-identity-evidence-2026-09-23) demonstrate why
address identity and retained stream identity must be described separately. The
on-site reference records all new behavior and the experimental type rename.

Validation on 2026-09-23: normal SDK compilation and execution passed for the product
sample, stream contracts, Path contracts and coherent-memory contracts; private Path
construction/mutation rejection checks also passed. The API snapshot, combined site
and four website tooling tests passed. An initial 64-entry capacity test exceeded the
VM's 100,000-instruction default with linear lookup; the fixture provider now uses
eight entries so this bounded case fits the unchanged runtime budget. This does not
establish scalable storage performance. No cross-platform matrix was rerun.


## Typed metadata and provider lookup (2026-09-23)

System.Storage.Metadata.GetKind(string) wraps the existing StorageKind internal
service as Result<EntryKind, StorageLookupError>. It accepts native strings and
observes regular files/directories; it does not open contents or retain resources.
EntryKind and the five lookup error cases have generated on-site reference coverage.
StorageProvider.GetFile(Path) and Directory.GetFile(name) are still application-owned
experiments. Both providers distinguish missing entries and wrong kind; Directory
also rejects invalid direct-child names. FileAt remains pure descriptor construction.
The author clarifies that Path belongs to Storage, not every API in the platform.
Strings remain valid API parameters elsewhere; callers may opt into parsing and
pass Text. No implicit conversion or new string-overload family was added.

The product sample now looks up the written notes on both providers and reads the
same content. LookupContracts checks missing/root/invalid-child outcomes, repeated
lookup, native invalid paths and directory kind. Native service tests establish
that lookup still succeeds at the 64-live-handle limit, consumes no IDs and leaves
an open reader's position unchanged. A separate test removes a file after successful
lookup and confirms a later open fails. The Unix VM case covers a followed link and
its dangling target. Actual permission-denial portability remains untested; existing
host error mapping is retained. No cross-platform matrix was rerun.

GetFile derives the display name using the existing filename helper on the current
shared slash-only logical grammar. This remains a limitation for broader provider
naming. Common directory structure, replacement identity and async lookup are next
design questions; do not infer a final capability or security contract from this
blocking sample. The .NET/Rust comparison above still applies.

Validation on 2026-09-23: all Raven library slices regenerated; bootstrap and API
snapshot checks passed. The normal SDK product sample, stream/Path/memory/lookup
contracts and negative Path construction/mutation checks passed. Seven native
file-resource unit cases and twelve VM file-resource cases passed (eleven existing
cases plus the targeted Unix link case). DocFX checked summaries for 130 generated
items; the combined site and four website tooling tests passed. No Raven compiler
change or cross-platform run was required for this bounded library adapter.


## Relative directory lookup (2026-09-23)

The application-owned Directory.GetFile(Path) overload resolves relative, potentially
nested paths while retaining the directory's provider. It accepts `nested/note.txt`
under `/context`, `context`, `/` or `.` as appropriate. Absolute paths are rejected
with StorageLookupError.InvalidPath before provider lookup, although they remain
valid Path values accepted by provider GetFile. The existing string overload still
means one direct child name. This follows one exploratory proposal distinction,
not an author decision to forbid path strings throughout Storage or other APIs.

DirectoryContracts uses different disk and memory contents at the same logical
address to check provider retention. It checks returned spelling/name, relative and
rooted bases, nested lookup, absolute rejection, the direct-child string restriction,
missing items and `.` at the root. The verifier creates only the disk fixture's
parent directories; the memory provider's flat-key limitation remains explicit.
No new runtime or core reference API is added by this slice.

The [.NET Path.Combine contract](https://learn.microsoft.com/en-us/dotnet/api/system.io.path.combine?view=net-10.0)
(reviewed 2026-09-23) permits a rooted later argument to replace the earlier prefix.
The .NET lookup probe now checks that behavior on its current host. Here the
operation-specific restriction instead preserves directory context, at the cost of
an extra validation failure and overload asymmetry. Neither design alone establishes
filesystem confinement. The existing host metadata/error and lexical Path research
still apply; general provider hierarchy and capability authority remain open.

Validation on 2026-09-23: the SDK product sample and stream, Path, coherent-memory,
lookup and relative-directory contracts passed on macOS arm64. The independent
.NET/Rust comparison, API snapshot, combined website and four website tooling tests
also passed. Core API metadata and runtime implementations did not change.


## Integration begins with System.Storage.Path (2026-09-23)

Following the author's request to integrate as soon as possible, Path and
InvalidPathError now live in the platform library. The sample imports
System.Storage.Path; its Path.rvn is only a fixture helper. The previously tested
parser, immutable text and explicit lexical comparison are retained. Existing native
Combine/GetFileName helpers keep their string signatures on the same type. This
backfills generated reference coverage for those helpers as well as the new members.
The ordinary SDK sample, negative constructor/mutation checks and archived lexical
path artifact test exercise the integration, not a copied application-local parser.

The next priority is integrating the working provider/File/Directory and stream
capability contracts. An expanded memory hierarchy was proposed but not implemented
in this turn; it is not a prerequisite for these initial integration slices. Existing
experimental limitations remain documented. No core/static File API is removed.

Integration validation on 2026-09-23: library regeneration and snapshot checks,
normal SDK product/stream/Path/memory/lookup/relative-directory cases, and the
existing lexical path artifact/service test passed. Imported private construction
is rejected with RAV1501 (no accessible matching constructor), while Text assignment
is rejected as read-only; targeted negative builds passed after updating the prior
application-local diagnostic expectation. API summaries cover 142 generated items;
the combined website and four tooling tests passed. Provider integration remains
next; Unix/Windows normalization and a future Uri value are recorded directions.

## Minimal Storage design: WinRT lessons (2026-09-23)

The author asks us to learn from Windows Runtime while implementing only immediate
needs. File/directory properties must earn their place in the sample; a broad
metadata model is not a prerequisite for the POC.

The next integrated descriptors should retain their provider and logical Path.
File.Name is useful for display; it should be derived from the path, not supplied
as a second potentially inconsistent constructor argument. Directory.Path supplies
the current context; a Directory.Name property can wait until a consumer needs it.
GetFile is an explicit typed lookup, while an address for exclusive creation need
not exist. Neither property access nor descriptor construction should perform I/O.
Opened streams own resources; descriptors do not retain an open handle.

The reviewed provider FileAt(path, name) factory was sample scaffolding. The first
simplification below removes it: a descriptor can retain a provider directly,
without requiring every provider to implement an identical factory. Keep existing
whole-text static helpers compatible, but do not require all future providers to
implement text encoding helpers merely to offer byte access. The next bounded
integration should settle this small byte-oriented contract using the working
read/write sample. These are preferred next-slice choices, not APIs already shipped.

Primary sources reviewed 2026-09-23:

- [WinRT StorageFile](https://learn.microsoft.com/en-us/uwp/api/windows.storage.storagefile?view=winrt-26100)
  separates properties such as Name/Path from opening streams; it also offers many
  content, display and mutation operations. Borrow the descriptor/open distinction,
  not the whole surface or its asynchronous signature model.
- [WinRT StorageFolder](https://learn.microsoft.com/en-us/uwp/api/windows.storage.storagefolder?view=winrt-26100)
  provides contextual lookup and creation. Keep the useful directory context;
  defer query objects, bulk enumeration and collision-option families.
- [.NET FileInfo](https://learn.microsoft.com/en-us/dotnet/api/system.io.fileinfo?view=net-10.0)
  caches retrieved metadata; [Refresh](https://learn.microsoft.com/en-us/dotnet/api/system.io.filesysteminfo.refresh?view=net-10.0)
  obtains another snapshot. Our initial descriptors expose address information
  only; defer size/timestamps/attributes and an explicit snapshot API. This avoids
  an immediate cache/freshness policy but offers less information than FileInfo.
- [.NET Stream](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream?view=net-10.0)
  combines directions and seeking with capability checks. The integrated directional
  interfaces let consumers require only input or output; the cost is more types
  and no current seek/duplex abstraction. Ordinary interface dispatch suffices.

These are library choices. They add no VM identity, security or scheduling mechanism.
Root mapping remains a convenience rather than a sandbox; successful lookup remains
an observation. Future Windows/Unix Path formats, richer metadata and Uri remain
outside this POC. Existing disk/memory tests are the integration acceptance cases;
new descriptor invariants need targeted checks when implemented.

Stream integration validation: the normal SDK disk/memory sample and all six
consumer/contract programs pass, including close through an interface and subsequent
use of the concrete alias. Opposite-direction source calls are rejected. The core
interface import probe, bootstrap snapshot, API reference (148 documented generated
items) and combined website also pass. Two ordinary Raven compiler corrections were
integrated from main; see [the integration evidence](../raven-target/README.md#directional-stream-interface-integration-2026-09-23).

### Descriptor simplification after stream integration

The sample now constructs File(provider, path), deriving Name from the validated
path instead of accepting a separate name. StorageProvider.FileAt and its duplicate
provider implementations are removed; Directory.FileAt still constructs a child
address, while GetFile performs lookup. No new metadata properties or filesystem
queries were added. The current GetFileName helper is sufficient for the validated
slash grammar; future Path formats will need their own component rules.

This implements the first simplification from the WinRT-informed review, without
promoting the whole provisional provider contract. Whole-text methods and provider
lookup remain the next integration choices. The verifier covers a nested Unicode
name without an existing provider root and rejects the old independent name argument.


### Platform byte-provider integration — 2026-09-23

System.Storage.StorageProvider now requires only OpenRead(Path) and CreateNew(Path),
returning the integrated directional streams. The sample interface inherits this
contract and adds its provisional lookup/text methods. File byte operations dispatch
through the core interface, exercising inherited interface conversion with both
providers. Existing static string-based File text helpers are unchanged.

This follows the .NET/WinRT comparison above: keep opening separate from descriptors
and text conversion. It avoids forcing every byte provider to implement text policy;
the cost is an additional interface layer while sample conveniences remain. Lookup
capabilities and File/Directory integration are next; no parent hierarchy or metadata
snapshot is introduced. Public signatures and limitations are covered by the
[provider API guide](../../../api-docs/storage-provider.md) and generated reference.

Validation: the normal SDK disk/memory product, stream/Path/memory/lookup/directory
checks, a provider implementing only the two core methods, and negative callers
pass. The interface import probe and regenerated bootstrap snapshot pass. DocFX
covers 151 items with summaries; four website tooling tests and the combined
13-page website build pass without documentation warnings. No native runtime or
Raven compiler change was needed for this slice.


### Platform File/Directory integration — 2026-09-23

File now retains a StorageProvider and Path, derives Name and delegates byte opening.
Its static native string-based ReadAllText/WriteAllText methods are preserved in the
same Raven-authored class. StorageLookup extends StorageProvider with GetFile(Path);
Directory requires this capability and provides FileAt plus both GetFile overloads.
Byte-only providers do not need lookup or text methods. Descriptor construction
performs no lookup and retains no stream. Directory.FileAt uses StorageLookupError
instead of the old sample FileReadError. All public members have on-site reference
coverage; WriteAllText has a manual entry for DocFX's unit-result limitation.

The sample imports these platform types. Its text workflow explicitly receives the
fixture text provider; File has no ReadText/WriteText instance members. Descriptor
lookup assertions read through byte streams, checking actual retained provider
context. Arrays of descriptors and inherited lookup-to-byte capability conversion
are admitted by the strict bridge. Existing .NET/WinRT research above supports
separate addresses, observations and handles; optional lookup adds an interface
but avoids imposing metadata operations on every byte backend. Rich metadata,
parent hierarchy and stable replacement identity remain provisional.

Next is a minimal integrated host provider; the memory implementation remains a
bounded comparison fixture, not a general filesystem.


Descriptor integration validation: normal SDK product and all contract/negative
checks pass, including a byte-only provider counter confirming File construction
and property access make no provider calls. Native static file input/output and
Path tests pass (10 cases, including artifacts/CLI). Strict interface/foundation
checks and bootstrap/API snapshots pass. All 245 generated API items have summaries;
four website tooling tests and the combined site pass with zero DocFX warnings.


### Provider-implemented Storage interfaces — 2026-09-23

The author's selected hierarchy is integrated: sealed StorageItem has two open
interface branches, File and Directory. Name/Path are common represented state.
The sample now owns ProviderFile/ProviderDirectory, using the previous descriptor
logic with its chosen backend. Both provider branches can be converted to
StorageItem and placed in one typed array. They are no longer public core classes.

The Raven metadata closes the root, not the implementation set of File/Directory.
The strict importer validates that metadata and rejects a raw CIL third branch;
it accepts transitive InterfaceImpl rows on real branch implementations. The
compiler rejects unrelated direct branches. Raw neoIL does not enforce closure.
No compiler changes were needed. The native runtime does not support interface
static methods, so development text calls migrate to FileText.ReadAllText/WriteAllText.
The legacy baseline keeps File aliases, while the Raven collection profile uses
FileText and the File interface. This is a development API migration, not a change
to published Preview 9 artifacts.

This follows the selected proposal model without claiming it is finished: provider
GetItem/GetFile/GetDirectory alignment, GetItems enumeration and creation semantics
remain next. Current StorageLookup and address-level FileAt/CreateNew are temporary.
Metadata queries remain an extension direction, not a new implemented API.


Hierarchy validation: the SDK product and all contract/negative cases pass, including
mixed StorageItem arrays and Raven RAV0306 for a third branch. The dedicated importer
probe rejects a C#-emitted third branch and missing closure metadata. Interface,
foundation and static file import probes pass. All 10 native file/Path regression
cases pass, retaining legacy call-site identity and native status behavior. Runtime
and API snapshots pass; 244 generated items have summaries. Four website tooling
tests and the combined 13-page site build pass without DocFX warnings.


### Provider directory lookup — 2026-09-23

StorageLookup.GetDirectory(Path) returns Result<Directory, StorageLookupError>.
The product now obtains its root through the provider, leaving ProviderDirectory
construction inside provider code. Existing address-only contract fixtures still
construct it intentionally. Disk lookup checks Metadata.GetKind and preserves errors;
the flat memory fixture exposes only its root (`/` or `.`). Slash-containing keys
do not imply directories. No creation, stream ownership or metadata cache is added.

This extends the .NET/WinRT comparison above (primary references checked 2026-09-23):
unlike constructing a [.NET 10 DirectoryInfo](https://learn.microsoft.com/en-us/dotnet/api/system.io.directoryinfo.-ctor?view=net-10.0)
address without checking existence, this operation queries current kind before
returning an interface, similar to [WinRT folder retrieval](https://learn.microsoft.com/en-us/uwp/api/windows.storage.storagefolder.getfolderasync?view=winrt-26100). It is synchronous and
returns typed errors; it does not promise WinRT async behavior. Keeping construction
as the public entry point was simpler but did not meet the selected provider
boundary. The cost of lookup is a provider call and a stale observation can still
precede failure on later access. Native implementation and GC ownership are unchanged.

This required member is a development compatibility change for StorageLookup
implementers. It is an incremental step toward consolidating resolution on
StorageProvider, not a commitment to the temporary byte/lookup split. Generic item
lookup, directory-to-directory traversal and enumeration remain next. Tests cover
root aliases, disk nested lookup, missing/wrong-kind errors, the memory policy and
reading through the returned Directory without a concrete cast.


Directory-lookup validation: the normal SDK product and complete contract/negative
suite pass, including missing/wrong-kind directories, root aliases, nested disk
lookup and subsequent file reads through interfaces. Directory lookup creates no
entry. The expanded contract fixture was reduced to one run of each existing case
after duplicated runs hit the VM instruction budget; the runtime limit is unchanged.
Strict interface import, bootstrap snapshot hashes, 245 documented API items and
the combined website pass. No native runtime or Raven compiler changes were required.


### Consolidated provider resolution — 2026-09-23

System.Storage.StorageProvider now requires GetItem, GetFile and GetDirectory.
All return core interfaces in Result with StorageLookupError. StorageLookup is
removed. The former provider OpenRead/CreateNew methods are now a sample-owned
ByteStorage detail used by ProviderFile; applications can implement a platform
provider without any byte or text helper methods. This follows the author's selected
resolution-centered proposal and the .NET/WinRT comparison above. Keeping the
previous split would preserve development source compatibility but make the public
provider contract describe backend plumbing rather than item lookup.

GetItem queries current kind once on disk, returning File or Directory as
StorageItem; unsupported native kinds preserve WrongKind. Memory resolves its root
or one of its stored file keys. Missing entries remain NotFound; no entry is created
and no content stream is held by lookup. Generic item tests inspect both interface
branches and a resolution-only provider. No compiler or native runtime change is
needed. Regenerate core/runtime artifacts together; Preview 9 is unchanged.

The host/memory implementations are still sample-owned at this checkpoint.
Platform host integration and enumeration follow; metadata breadth is not required.


### Integrated FileSystem and bounded enumeration — 2026-09-23

FileSystem(root) now implements the platform StorageProvider. LocalFile and
LocalDirectory remain internal; provider results expose only public interfaces.
The sample HostStorage delegates lookup to FileSystem, retaining text helpers only
as fixture conveniences. Directory supports GetItem(relativePath), both directory
lookup overloads and GetItems(maxItems). Enumeration returns a complete bounded
Sequence<StorageItem>, with no retained native cursor or partial-success result.

Primary comparison checked 2026-09-23:
[.NET 10 DirectoryInfo.GetFileSystemInfos](https://learn.microsoft.com/en-us/dotnet/api/system.io.directoryinfo.getfilesysteminfos?view=net-10.0)
returns an array of file/directory objects, while
[WinRT GetItemsAsync](https://learn.microsoft.com/en-us/uwp/api/windows.storage.storagefolder.getitemsasync?view=winrt-26100)
returns storage items asynchronously. The POC chooses a synchronous bounded
snapshot rather than a resource-owning iterator or async stream. This reduces
lifetime/scheduling machinery but allocates all names and descriptors up front;
large directories fail instead of paging. FileSystem sorts names, caps count at
1024 and native UTF-8 names at 64 KiB, and observes runtime array limits. Invalid
bounds use InvalidRange; overflow uses LimitExceeded. Enumeration and child lookup
are not atomic. Unsupported names/kinds, deletion or permission failures fail the
operation. No metadata cache, immutable identity or symlink-safe sandbox is claimed.
Memory lists only direct root children, preserving its explicitly bounded model.

StorageList is a FileInput runtime service. Its tagged transport is bootstrap-only;
StorageNames unpacks a checked names array for copying into managed storage. The
strict bridge admits FileSystem's constructor and three lookup methods and the
new Directory signatures. No Raven compiler configuration change is required.


The System.IO reader contracts also run against disk and deliberately short-read
memory input: strict UTF-8, BOM preservation, bounds, EOF, underlying errors, close
ownership and byte seeking. The [standalone Storage POC](../storage-poc/README.md)
combines those capabilities using the platform FileSystem provider alone.
