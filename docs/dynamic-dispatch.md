# Dynamic dispatch service

neoCLR should support languages whose dispatch model is more dynamic than the
ordinary statically bound call instruction. Dynamic behavior is an optional VM
service; it is not imposed on typed code and does not require a separate DLR.

A dynamic call carries an explicit call-site descriptor: the operation name or
token, argument values, optional receiver, and the expected result contract. The
host or language runtime can register a dispatch handler for that descriptor. The
handler may resolve a target, invoke a language object protocol, or return a typed
failure. The VM records the selected target and preserves the invocation's resource,
interrupt, and stack-trace rules.

Typed calls remain directly verifiable and directly lowerable to native code.
Dynamic calls are marked in metadata, require the dynamic-dispatch service, and may
be rejected by a restricted host or AOT profile. A language can therefore opt into
dynamic dispatch without weakening the type safety of the rest of its program.

Handlers must declare their result type or an explicit erased result contract. They
cannot silently manufacture null values, bypass accessibility, or reinterpret raw
memory. Unsafe pointer operations and native calls remain separate explicit services.
Caching, inline dispatch, language-level method lookup, and reflective invocation are
implementation choices for a language handler, not hidden VM semantics.

The initial prototype does not execute dynamic calls. This document establishes the
service boundary so a future instruction can be added without making ordinary calls
depend on a DLR-shaped object model.
