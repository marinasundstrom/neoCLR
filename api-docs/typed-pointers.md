# Typed native pointers

Development API in `System.Runtime.InteropServices`, declared by `System.Runtime.dll`.

```raven
public struct Ptr<T> : EquatableTo<Ptr<T>> where T: struct {
    init(pointer: *T)
    val Pointer: *T
    func Equals(other: Ptr<T>) -> bool
    override func Equals(other: Object?) -> bool
    static func ==(left: Ptr<T>, right: Ptr<T>) -> bool
    static func !=(left: Ptr<T>, right: Ptr<T>) -> bool
}
```

The constructor wraps an address without allocating or taking ownership. `Pointer`
returns that address. The default value is null. Wrapping, copying and comparing
pointers do not validate that their storage is still alive. Free storage through
its original allocator; copying this value does not retain storage or duplicate
ownership. Using the raw pointer requires an unsafe context.

## Equality operators

`==` returns true when both operands contain the same address; `!=` returns its
negation. Neither reads the pointed-to value. Comparisons require the same closed
`Ptr<T>` type. Two default values compare equal. These operators do not allocate
managed objects or release storage.

Typed `Equals` has the same semantics. Boxed `Equals` returns false for null or a
different closed type. Converting a wrapper to Object boxes it normally. Inherited
value-type `GetHashCode` hashes its pointer field without boxing that field or
reading memory. Equal pointers have equal hashes; hashes are not persistent IDs.
The inherited Object documentation describes the class default; this value type
uses the runtime's fieldwise value contract.

A struct constraint is necessary but not sufficient for an unmanaged layout.
Existing native layout and pointer-target checks still apply; not every struct is
currently admitted as a pointer target. No implicit conversion to Slice/Memory,
implicit ownership, pointer arithmetic or ordering is provided by this wrapper.

The built-in `native-pointers` suite covers allocated aliases, null, generic
forwarding, boxed equality and hashes across collection. Foreign native imports
remain interpreter-only; this type does not add JIT/AOT import support.

RavenDoc currently maps the `==` and `!=` detail pages to the same sanitized file
name. This page is the complete reference for both operators until that renderer
gap is fixed; the generated type page lists both signatures.
