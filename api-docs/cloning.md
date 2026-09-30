---
title: Cloning with Self
---
# Cloning with Self (development)

[Clonable](xref:System.Clonable) is nongeneric. Its Clone method returns native
`Self`, so a generic caller retains its implementing type:

```raven
func Copy<T>(value: T) -> T where T: Clonable => value.Clone()
```

The reference assembly displays the [Self transport marker](xref:System.Runtime.CompilerServices.Self).
It represents the implementing type, not a value to construct. The native receiver
contract borrows a struct slot or loads a class reference from its slot without
boxing. Null class receivers fault; erased Clonable values cannot invoke Clone.
Current application import supports direct class/struct implementations through
bounded closed generic helpers. An inherited conformance keeps the base class as Self. A derived type must
redeclare Clonable and supply its own result contract before satisfying T: Clonable.
Native explicit interface mappings preserve the inherited public clone method;
virtual overrides retain the base result signature. The experimental Raven
compiler/importer supports this bounded inheritance contract. Broader target
integration awaits Raven’s multi-target refactor.

Clonable guarantees the result type, not deep copying. Implementations must describe
which state is copied or shared and how resource ownership is handled. The tested
Cell/Box consumer copies its integer field and verifies that mutating the original
Box leaves its clone unchanged; this is not proof of arbitrary object-graph cloning.

This development change replaces Clonable<T>. Remove its type argument, return
Self (or the concrete implementing type), and rebuild reference, library and
application artifacts together. A former Clonable<OtherType> relationship cannot
be expressed by this contract; model that operation as a conversion or factory.
Published releases and the archived Neo bootstrap profile are unchanged.
