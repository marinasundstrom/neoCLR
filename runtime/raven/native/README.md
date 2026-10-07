# Native runtime declarations

These Raven sources are inputs to the native class-library build, alongside their
source-owned callers. They are not CLI bridge implementations and are not included
in the legacy .NET bridge source glob.

`RuntimeHandleServices.rvn` declares two internal runtime services using the selected
primitive core's `MethodImpl(InternalCall)` marker. The native emitter consumes that
marker into the existing implementation flag; no attribute object or placeholder
method executes. Object and RuntimeTypeHandle use the explicitly selected owners,
including source owners during full bootstrap. The service implementations already
exist in the runtime. `NativeObject.GetTypeHandle` is a distinct internal facade for
source Object.GetType: an extension cannot override the existing bootstrap static
RuntimeServices member. The legacy ObjectIntrospection extension retains its prior
bootstrap path. See the [source-owner gate](../../../docs/experiments/extended-cli-metadata/object-handles-2026-10-07.md).

Production introspection/JSON sources now compile and execute with the associated
`RuntimeIntrospection*` adapters; see the recorded native JSON integration gates.

The storage adapters are also explicit native inputs:
`RuntimeStorageCalls.rvn` declares the internal ABI; `RuntimeStorageServices.rvn`
and `RuntimeFileTextServices.rvn` expose it in the namespaces used by the unchanged
production sources. They perform real runtime calls, not CLI projections or stubs.
`StorageNames(Value)` materializes a native string snapshot as a managed `Array<String>`;
its array storage is independently owned and subject to array/heap limits. Runtime
service analysis marks this conversion as ManagedArrays, while StorageList owns the
filesystem access. The older ordinary neoil StorageNames helper remains compatible.

The reproducible [storage gate](../../../docs/experiments/extended-cli-metadata/source-storage-2026-10-05.md)
builds the production library and executes consumers with only native artifact references.
Seeds and ownership manifests remain explicit; source declarations must not compete
with retained runtime definitions.

`RuntimeNetworkCalls.rvn` and `RuntimeNetworkServices.rvn` expose the existing native
DNS/socket services, including callback functions, managed buffers, cancellation and
deadlines. DnsAddresses uses the same managed string-snapshot materialization as
StorageNames. Callbacks retain existing native function signatures, with no nominal
delegate ABI or structural Function experiment dependency. The unchanged network
cancellation fixture runs over localhost and an ephemeral loopback listener through
[the source network gate](../../../docs/experiments/extended-cli-metadata/source-network-2026-10-05.md).

`NativeReflection.rvn` and `RuntimeConstructionCalls.rvn` provide parameterless
construction and its preflight check for source-owned Object/RuntimeTypeHandle.
The runtime executes the actual constructor and retains access/missing-constructor
errors. Full-source builds omit the seed-only ObjectIntrospection extension because
source Object owns GetType. The older POC manifest remains unchanged.
See the [source construction gate](../../../docs/experiments/extended-cli-metadata/reflection-construction-2026-10-07.md).
