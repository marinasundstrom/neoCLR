---
uid: System.Runtime
---
# System.Runtime

Runtime integration services for the executing application. RuntimeContext exposes
assembly and type discovery; its member reference coverage is still pending. See
[the runtime discovery guide](/features/introspection/index.html) for current behavior.

The nested [System.Runtime.CompilerServices](xref:System.Runtime.CompilerServices)
namespace contains compiler-facing metadata. Its IsExternalInit marker identifies
init-only property setters; it is not an executable service.
