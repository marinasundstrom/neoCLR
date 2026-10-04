# Native runtime declarations

These Raven sources are inputs to the native class-library build, alongside their
source-owned callers. They are not CLI bridge implementations and are not included
in the legacy .NET bridge source glob.

`RuntimeHandleServices.rvn` declares two internal runtime services using the selected
primitive core's `MethodImpl(InternalCall)` marker. The native emitter consumes that
marker into the existing implementation flag; no attribute object or placeholder
method executes. Object and RuntimeTypeHandle retain their existing bootstrap/runtime
identities. The service implementations already exist in the runtime.

The first executable gate compiles this file unchanged beside a test provider and
then builds a separate native consumer. See
[the integration record](../../../docs/experiments/extended-cli-metadata/source-internal-calls-2026-10-05.md).
This is not yet the production descriptor build. A seed must not retain competing
service declarations, and any seed callers must move with services that become
internal to the source-built library. Descriptor factories and snapshot/vector ABI
remain the next ownership boundary.
