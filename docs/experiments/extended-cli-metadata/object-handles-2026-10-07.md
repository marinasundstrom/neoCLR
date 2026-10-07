# Source Object handle service — 2026-10-07

The source Object `GetType` implementation now calls the internal native facade
`NativeObject.GetTypeHandle`, which invokes the existing
`neoCLR.Runtime.ObjectTypeHandle` internal call. The declaration lives in
`RuntimeHandleServices.rvn`; no runtime or compiler change is required.

Previously the call selected the bootstrap `RuntimeServices.ObjectTypeHandle`
member, carrying the bootstrap Object signature. A same-named extension cannot
replace an existing static member. The distinct facade follows the NativeAllocation
pattern and makes the native boundary explicit. Its name is internal and not a
public platform API. Object's public API and runtime identity semantics are unchanged.
Compared with CLR Object.GetType, type identity still comes from the executing
runtime; this adapter separates that runtime service from the bootstrap facade.

## Validation

- The focused fixture failed before the adapter with `NEOMETA001`, an unavailable
  ObjectTypeHandle dependency signature; no library was published.
- `verify_generic_object_root.py --object-handles` now compiles a source-Object
  library with source-owned RuntimeTypeHandle and an explicit ownership manifest.
  An empty retained seed prevents a competing handle definition. The test obtains
  an object's handle, queries zero generic arguments, checks alias/distinct-object
  identity and hash stability, and returns 42. Verification succeeds; execution
  exits 42 with no stdout/stderr.
- The consumer is metadata-API authored and references the emitted library by
  artifact digest. This is not ordinary Raven imported-root acceptance and does
  not execute the complete production GetType/introspection path.
- [Execution evidence](object-handles-2026-10-07.json) records commands, source and
  dependency hashes, compiler `793220f33` and runtime binary hashes. NeoCLR base is
  `b881caa4` plus this source adapter slice. No .NET compiler behavior changes.
- [Full audit](native-bootstrap-object-handles-2026-10-07.json): **196 inputs**,
  zero binding errors, no output published. The ObjectTypeHandle failure is gone;
  the next failure is `ReflectionConstruct(RuntimeTypeHandle) -> object`, another
  bootstrap service signature. Investigate its source adapter at the same boundary;
  do not relax type identity checks or infer that full System now compiles.
- RavenDoc snapshot fingerprints refreshed and checked. Public signatures and
  reference assembly are unchanged; no website build was needed.

The bootstrap-only `ObjectIntrospection` extension retains its existing path for
previous POC bundles. Source Object owns GetType in the full-source build; migrating
all remaining bootstrap reflection services remains subsequent work.
