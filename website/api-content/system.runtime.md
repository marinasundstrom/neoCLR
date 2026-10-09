---
uid: N:System.Runtime
---
## Runtime context and managed lifetime

RuntimeContext connects application code to the executing assembly and resolves
runtime type handles into TypeInfo descriptors. GC exposes the implemented
managed-lifetime controls and counters.

See [introspection](/docs/introspection.html) and
[garbage collection](/docs/gc.html). These APIs describe the current runtime
contract; reflection availability and collection behavior can differ by execution mode.
