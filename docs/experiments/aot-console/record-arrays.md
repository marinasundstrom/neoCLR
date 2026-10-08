# Reserved native value arrays — work in progress

The HTTP task path stores Result values in checked ArrayList capacity. Native AOT now
supports `array.reserve` of closed admitted value records, with indexed snapshot loads,
stores, length and null tests. Closed nominal generic method arguments allow the real
`CheckedStorage.Reserve<T>` wrapper to specialize. The original load set is verified
before lowering; array element identities and generic contracts remain checked.
Ordinary default-initialized nominal arrays, interface-element arrays, element borrows,
array equality and nominal array/interface views remain outside this bounded profile.
This adds no Raven syntax, public library API or temporary CLI bridge encoding.

## Representation and collection

Private storage consists of a 24-byte header (kind, length and padded lane count),
then inline snapshots and one initialization byte per element. The existing value
layout has at most 64 eight-byte lanes; array length remains bounded by 65,536 and the
host's actual heap byte budget. The allocator validates ranges before allocating,
clears all storage, never collects internally and publishes output only on success.
Native stores write the complete typed snapshot before marking its slot initialized.
Loads check null, bounds and initialization before exposing a copied value.

The matching collector gains private allocation kind 5 for these arrays. It validates
the header and payload extent, skips uninitialized elements and conservatively traces
lanes of initialized snapshots. Nested references, including callbacks and their
receivers, therefore stay alive; replacement removes the old element's edges. Numeric
words can conservatively retain unrelated allocations if they resemble heap addresses,
as with existing object scanning. This is not a precise field map or moving collector.
A malformed descriptor returns RuntimeError without sweeping and clears temporary
worklist links so a repaired descriptor can be collected safely. Image and native
adapters must be built together; this private layout is not a stable external ABI.

## .NET baseline and tradeoff

This applies the existing [managed array](../../managed-arrays.md) and
[checked reservation](../../reserved-array-capacity.md) research. .NET value-array reads
copy values while references inside a copied value still share their targets. That is
also the intended neoCLR behavior. The library's checked capacity additionally avoids
fabricating a default Result case: unwritten elements cannot be read, including when
their zeroed bytes happen to resemble a valid discriminator. See the linked design for
the distinction from .NET GC.AllocateUninitializedArray and its reference initialization.

Flattened inline snapshots reuse the backend's existing record ABI and avoid per-element
boxing. Costs are padded storage, one marker/check per element, conservative scanning
and a new private GC allocation kind. Boxing every element would reuse reference-array
storage but add allocations and identity/copy complications. Precise layouts and tighter
packing remain future alternatives. No memory or speed advantage over .NET is claimed;
the interpreter already implements snapshot/reservation semantics and needs no change.

## Validation and HTTP boundary

`record-arrays.neoil` covers independent copied values, a callback receiver reachable
only through an array element, 1,000 replacements in a 2 KiB heap, unwritten reads and
negative indexes. Native and interpreted results/fault traces match. Sanitized C kernel
tests run with and without GC, verify no publication on invalid/exhausted allocation,
trace only initialized elements, release replaced references, preserve canaries and
reject/repair malformed headers, lengths and lane counts. Native admission rejects
record newarr and element borrows explicitly. Relevant codegen, linking, value, root
and collector checks pass (143 tests across six focused integration suites).

[ResultList.rvn](../../../benchmarks/native-web/ResultList.rvn) uses the actual Raven
ArrayList<Result<int,string>> library path: growth, Copy, Error/Ok replacement, let-pattern
and if-let matching, shared string payloads and 1,000 discarded allocations. The same
metadata runs interpreted and natively with a 64 KiB heap, sanitized adapters and an
executable linked only to libSystem. The interpreter correctness run allows 100 million
instructions; this is a ceiling, not an instruction count or equal native execution
budget. [Evidence](../../../benchmarks/native-web/record-array-validation.json) records
commands/hashes and server admission. Run `verify_callbacks.py --case ResultList` with
the same required paths as the other native-web consumers. No server timings are claimed.

The initial full HTTP probe reached the former 128-specialized-type limit. A narrower
[TaskResultList.rvn](../../../benchmarks/native-web/TaskResultList.rvn) admission probe
uses the exact Result<Void,HttpError> shape; an initial direct probe exposes unsupported
HttpStatusCode enum metadata, not a Raven emitter failure. That initial probe did not execute natively; the later result below supersedes it. Direct Int32.ToString receiver projection and Object-display candidate selection
also surfaced while developing the storage sample; the passing consumer uses the existing
explicit String.Concat path to keep this slice about arrays. Those backend gaps and
bounded full-server selection remain follow-up work, alongside task/socket completion.

### HTTP nested results (2026-10-08)

TaskResultList now passes with the real ArrayList<Result<Void,HttpError>> path,
including an unnamed HttpStatusCode payload, let/if-let patterns, copies and 1,000
string replacements. HttpError needs 40 flattened lanes. The bounded profile and
array allocator/collector now agree on a 64-lane maximum; layouts beyond it remain
rejected. Call scratch is sized for selected layouts, retaining a 256-byte floor,
and contributes to the existing 64 KiB frame bound. This can increase stack use in
programs selecting wide types. No public type, metadata format or stable ABI changes.

This continues the value-copy/.NET comparison above. Overlaying inactive union
payloads could reduce these widths, but requires a separate layout and precise GC
contract. Increasing the checked bound lets the current source representation run
without pretending it is an efficient final union layout. Existing smaller array
storage is unchanged. No speed or memory advantage is claimed.

Validation covers a 64-lane returned record, oversized-layout rejection, sanitized
allocation and GC of a last-lane reference with uninitialized elements ignored, and
buffer canaries. [HTTP result evidence](../../../benchmarks/native-web/http-result-validation.json)
records interpreter/native parity in a 64 KiB heap and libSystem-only standalone
linkage. The next selection slice now admits the full server within coordinated 256-type/1,024-function/256-clone bounds; reference arrays are the next rejection.

### Reserved reference arrays (2026-10-08)

The same checked storage now admits nominal reference-class elements. Each element
is one strong pointer slot, with its existing initialization marker. The original
runtime verifier checks the declared element identity; no array covariance or
interface-element admission is added. Unwritten reads fault, whereas an explicitly
stored null is initialized and reads successfully (dereferencing it then faults).
Default newarr, element borrows, array identity and nominal array/interface views
remain unsupported in this bounded native slice.

As with .NET List<T> for reference types, copying the container copies references:
mutating an object through either copy is visible through both, and replacing one
slot does not replace the other's. This reuses the checked-capacity comparison above;
neoCLR's reservation markers distinguish unwritten storage from initialized nulls.
Reusing the record-array allocator/scanner avoids another C ABI or allocation kind,
but retains its 24-byte header and conservative scanning. No performance improvement
is claimed; the interpreter already has these semantics.

`reference-arrays.neoil` verifies array-only owner retention, shared mutations,
1,000 replacements in a 2 KiB heap, unwritten/bounds/null fault parity, and canaries.
Negative tests reject default creation, element borrows and wrong nominal elements.
[ReferenceList.rvn](../../../benchmarks/native-web/ReferenceList.rvn) exercises the real
Raven ArrayList<Counter> growth/copy/replacement path in both modes with sanitized
adapters, 64 KiB GC and libSystem-only standalone linkage. [Evidence](../../../benchmarks/native-web/reference-array-validation.json)
records the full-server's next missing binding: neoCLR.Runtime.ObjectReferenceEquals.

### Object identity service (2026-10-08)

Reference-arena compilation now binds the exact reserved
neoCLR.Runtime.ObjectReferenceEquals(Object,Object) -> Boolean InternalCall to the
existing native reference-equality instruction. The public Raven wrapper remains
ordinary CIL. Managed bodies, altered signatures and unrelated declarations are not
substituted. This is reported as object-reference-equals-intrinsic-v1 in the binding
inventory; no external symbol, allocation or hash algorithm is introduced.

The [object-model review](../../object-model-review.md) records the .NET baseline:
reference identity is separate from value/content equality, including null/null.
The native adapter preserves the existing interpreter contract rather than changing
it. ReferenceList now checks aliases, replaced objects, null in either position,
and separately allocated equal-content strings. [Evidence](../../../benchmarks/native-web/object-identity-validation.json)
records interpreter/native parity and standalone linkage. Exact-contract rejection
has focused coverage. General Equals/GetHashCode dispatch is not implied. The HTTP
server now reaches the value-type StreamError.ToString member contract boundary.
