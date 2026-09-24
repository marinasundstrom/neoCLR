# System.Runtime.CompilerServices.IsReadOnlyAttribute

**Development compiler-reference metadata.** This public marker identifies readonly
metadata emitted or consumed by the compiler. It is not an application resource or
an additional runtime service.

```raven
class IsReadOnlyAttribute : System.Attribute
```

## Constructor

```raven
init()
```

Creates the marker for metadata encoding. It has no parameters and carries no
payload. The reference constructor is compiler support; the reference assembly is
not an executable implementation. Applying a marker does not by itself create a
new runtime readonly capability.

The pinned Raven metadata model consumes this marker and does not expose its type
to RavenDoc. This manual page keeps its public declaration and constructor visible.
