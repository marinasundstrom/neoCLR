# System.NamespaceMembers

**Development compiler-reference container.** Raven emits namespace-level functions
inside this public metadata type. Call these functions through `System`; applications
do not construct a NamespaceMembers instance.

[Fault](xref:System.NamespaceMembers.Fault) terminates the current invocation with
the supplied message. It represents a runtime failure, not an expected error value.
See [fault behavior](/docs/faults.html).

RavenDoc presents the function on the [System namespace page](xref:System).
This page preserves the public container's reference entry.
