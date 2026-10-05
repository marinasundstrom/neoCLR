# Post-HTTP compilation frontier — 2026-10-05

The combined source library compiles: 139 production System sources plus nine native
adapters, including the previously separate HTTP/network and storage groups. The
[compile evidence](post-http-compilation-2026-10-05.json) records exact commands,
ownership manifests, source hashes, diagnostics and the emitted artifact hash.
Two JSON consumers and the Tasks consumer also compile separately and execute against
that single artifact with their existing output/value assertions. Their
[execution evidence](combined-library-consumers-2026-10-05.json) records commands and
hashes. This is an expanded native compilation/execution checkpoint, not full-System
completion; HTTP loopback execution evidence still belongs to the separate HTTP gate.

## Array dispatch repair

Inheritance substitution previously extracted arguments only from nominal constructed
types. CLI array storage uses ArrayRef(element), backed by System.Array<T>; scanning its
members for Object virtual dispatch substituted an empty type argument list and faulted.
Base traversal, field layout, abstract contract validation and declared method dispatch
now use the existing shared generic-argument accessor. No array representation, metadata
encoding, public API or compiler contract changes. Compared with .NET, arrays retain
reference identity and stable default hashing; neoCLR's existing generic Array<T> API
shape remains unchanged. Existing website array documentation still describes this
contract correctly; no website build or API snapshot refresh is needed for this fix.

## Next bounded work

The full inventory attempt (166 production sources plus nine adapters) fails binding
and publishes no assembly. The manifest retains the previous bootstrap ownership;
therefore this is a diagnostic probe, not a claim that full source ownership is configured.
Prioritize these shared dependencies before fixing individual downstream API diagnostics:

1. Reconcile source Object and RuntimeTypeHandle with primitive/retained ownership.
   The typeof contract and Object override failures indicate mismatched core identities;
   establish canonical source/primitive contracts before interpreting cascaded errors.
2. Complete native core service declarations for console/environment, Object/GC,
   pointer conversion and math, following the tested explicit adapter approach.
   RuntimeFailure and NativeAllocation source/import ownership also need resolution.
3. Add the time-zone service family and its source consumers after those prerequisites.
   Re-run the inventory to distinguish real generic/union errors from current cascades.

Native async emission remains separate. The successful callback HTTP gate need not be
reopened for these source ownership changes, but affected consumers must be rerun when
core identities change. Full-library .NET parity and post-bootstrap benchmarking remain
future gates. No performance improvement is claimed for this dispatch correction.

## Isolated core probes

Adding only source RuntimeTypeHandle to the successful combined inputs reproduces
RAVT003 typeof-contract rejection and generic mapping conversion diagnostics. Adding
only source Object reproduces override/conversion errors and the three missing Object
service names. Both commands publish no output; the compile evidence includes their
exact inputs and diagnostics. These isolate the high-fan-out core definitions, without
asserting that every downstream diagnostic has the same cause.

The runtime revision in consumer evidence is the working-tree base fc66fb63; the runtime
binary hash identifies the build containing this inheritance fix. No Raven compiler
change was required in this slice (48ff053196). No Runtime Contract or bridge encoding
changed. Previously recorded HTTP and focused .NET evidence remains unaffected.

Raven's current MetadataImportOptions.SupportsPrimitive admits numeric, Boolean,
String and Char source providers, not RuntimeTypeHandle or Object. Its typeof resolver
requires SpecialType.System_RuntimeTypeHandle. The next compiler investigation should
start at that explicit contract boundary; weakening the typeof signature check or
silently substituting a reference would obscure the ownership issue.

## Focused validation

The unchanged regression suites pass: object_display 9/9, object_equality 36/36,
generic_array_shape 8/8, class_inheritance 8/8 and inherited_layout 9/9 (70 total).
Both formerly failing array cases now pass. Commands:

```sh
cargo test --test object_equality --test object_display
cargo test --test generic_array_shape --test class_inheritance --test inherited_layout
```

This machine used the explicit MacOSX26.2 SDKROOT documented in the HTTP gate to avoid
its installed linker/default-SDK mismatch. Consumer execution used the rebuilt debug
runtime; the long debug suite duration is not a runtime benchmark or release claim.
