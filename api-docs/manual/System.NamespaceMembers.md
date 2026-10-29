# System.NamespaceMembers

**Development compiler-reference container.** Raven emits namespace-level functions
inside this public metadata type. Call these functions through `System`; applications
do not construct a NamespaceMembers instance.

[Panic](xref:System.NamespaceMembers.Panic) terminates the current invocation with
the supplied message. It represents a runtime failure, not an expected error value.
See [fault behavior](/docs/faults.html).

RavenDoc presents the function on the [System namespace page](xref:System).
This page preserves the public container's reference entry.

Prefer `import System.*` followed by `Panic("diagnostic")`. This development rename
removes `System.Fail`; existing terminal fault behavior is unchanged.
