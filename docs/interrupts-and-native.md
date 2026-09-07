# Interrupts and native development

neoCLR remains usable for systems and native-oriented languages even when a
managed language is hosted on top of it. Interrupts and native calls are therefore
VM capabilities, not privileges reserved for one language profile.

## Interrupt delivery

The execution engine must expose an explicit interrupt state for each invocation.
Hosts can request cancellation, termination, or a debugger break; the interpreter,
JIT, and AOT backends observe that state at instruction boundaries and other defined
safepoints. A pending interrupt produces a classified terminal result or transfers
to a host-defined handler according to the invocation policy. It must not be silently
converted into a guest exception.

The current prototype provides cooperative cancellation through `ExecutionOptions`.
Asynchronous signal delivery, resumable handlers, thread interruption, and debugger
breakpoints remain backend work. Their contracts should be shared by all execution
modes so a program does not acquire different semantics merely by switching from
interpretation to JIT or NativeAOT.

## Native boundaries

Native development uses explicit declarations describing the calling convention,
argument and return layouts, pointer safety, and ownership transfer. A native call
may be unavailable in a safe host or sandbox, but the metadata remains inspectable
and the failure is reported before execution when possible. Native code can access
raw `Ptr<T>` storage and allocation services through the declared ABI; managed
wrappers may provide safer APIs above it.

Native calls do not imply garbage collection, reference counting, or object identity.
Those policies must be declared by the wrapper or allocator. The same boundary must
be representable for an interpreter, JIT, and NativeAOT backend, with backend-specific
lowering kept behind the stable metadata contract.

This keeps a managed runtime from becoming a closed security box while retaining
explicit checks at boundaries chosen by the host and language.
