# Optional object hierarchy and common value methods

The [library-focused next preview](library-preview.md) prioritizes shared type
relationships, interface inheritance and class inheritance to support useful library
contracts and their Neo projection. [Interface inheritance](interface-inheritance.md) is now implemented; class inheritance
remains planned.

Status: [inherited value layout and BaseType](inherited-layout.md) are implemented
as preliminary groundwork. [Base references](base-views.md) are now implemented;
virtual dispatch, constructor chaining
and common Object methods remain planned. The existing interface system and Equatable<T> remain
available. Heap allocation and managed references are independent of this proposal.

The runtime should support types with or without a base class. There is no mandatory
System.Object root. Languages may insert that base implicitly as a language policy;
the standard library can derive from Object where shared behavior is useful.
Static-only types do not need a base solely for uniform metadata.

Inheritance describes relationships and shared behavior. It does not assign value
or reference semantics to a type or force heap allocation. A type derived from
Object can still be used as an ordinary value or explicitly accessed through T&.
Heap allocation places a value into managed storage and returns T& directly; it
must not require a conventional boxing/unboxing representation or conversion step.

## Equality and hashing describe the value

Equals and GetHashCode belong to the value's contract. Their meaning is the same
when a value is local, heap allocated, or accessed by reference. There is no inherent
value-type/reference-type distinction that switches these methods to reference
identity behavior. Equal values must yield equal hashes under the same equality
policy. This is value equality and value hashing, independent of addressing mode. The
default record policy and the relationship to Equatable<T> need to be
specified before adding universal method implementations.

Reference identity is separate: whether two references designate the same stored
instance or location. The preliminary [ref.eq / ReferenceEquals facility](reference-identity.md)
now compares managed locations independently of value equality. Any reference-identity hashing intrinsic must preserve that
identity across moving GC and account for interior locations consistently. This
intrinsic must not become Object.GetHashCode's implicit meaning. Raw native pointer
addresses are a separate capability and cannot define managed value identity.

## Library methods with runtime support

System.Object can supply familiar Equals, GetHashCode and ToString methods with
ordinary override rules once inheritance exists. Types without Object may implement
those methods or suitable interfaces directly. ToString needs no heap allocation or
Object base just to format a primitive or retrieve type information.

The runtime should supply the low-level mechanisms the library needs: type metadata,
primitive formatting/hashing and a separate managed-reference identity operation.
Their library bindings should use the existing validated runtime-helper registry.
Do not infer special runtime behavior solely from a user method's name.

Before implementation, define base metadata, inherited field layout, constructor
chaining, dispatch slots and override validation. Base views of ordinary values need
an explicit copying/slicing rule; managed base references need a view preserving the
complete root's lifetime and identity. Neither should silently box a value. Decide
the default value equality/hash policy and generic equality dispatch alongside these
rules, rather than baking class-versus-struct behavior into the collector.

The next object-model slices follow the direct managed-reference/GC milestone.
Memory management will be adapted as inheritance introduces base views and derived
layouts. [Pinning](pinning.md) must likewise cover the complete derived allocation.
