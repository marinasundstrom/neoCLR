# Native runtime declarations

These Raven sources are inputs to the native class-library build, alongside their
source-owned callers. They are not CLI bridge implementations and are not included
in the legacy .NET bridge source glob.

`RuntimeHandleServices.rvn` declares two internal runtime services using the selected
primitive core's `MethodImpl(InternalCall)` marker. The native emitter consumes that
marker into the existing implementation flag; no attribute object or placeholder
method executes. Object and RuntimeTypeHandle retain their existing bootstrap/runtime
identities. The service implementations already exist in the runtime.

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
