# Reference values and assignment in Neo

```swift
let foo = new Foo()
let foo2 = foo // Copies the reference; both address the same object.
foo2.Bar()     // Automatically follows the reference for member access.
```

A mutable reference binding follows the same rule on reassignment: `selected = other`
copies an existing reference. `selected = &value` explicitly creates a reference to
a value. `selected = new Foo()` stores the newly allocated reference. Member access
and invocation need no dereference operator. No automatic borrowing is introduced.

## Stored reference versus referenced target

Reference RHS assignment stores the reference in mutable locals, captured mutable
bindings, writable fields and array elements. Collection indexer setters already
use their declared element type, including T&. No T&& is formed for these stores;
ordinary slot stores enforce runtime type, readonly and lifetime restrictions.

`let` bindings and source parameters remain immutable bindings and reject retargeting.
They still permit mutation of a writable target's members. An immutable directly held
record cannot have its reference field replaced, but the stored reference may still
permit mutation of its target. Readonly reference compatibility remains enforced.

A T value assigned through T& still replaces the referenced T. To request a value
copy from a reference, write `let copy: Foo = source; destination = copy`. This also
works when destination is a let reference. This explicit intermediate distinguishes
copying a referent from copying a reference without introducing a dereference operator.
An out Foo& parameter always addresses Foo output storage; `output = sourceReference`
reads and writes the Foo and fulfills the output obligation. It does not initialize
or replace a caller's Foo& slot. A reference-returning call similarly supplies target
access, not a caller-visible reference binding to retarget.

## Comparison and decision

The [C# assignment reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/assignment-operator)
(consulted 2026-09-08) distinguishes copying class-reference values from writing
through ref locals; C# uses ref assignment to retarget the latter. Neo previously
applied the ref-local target-write rule even between two reference-valued bindings.
The chosen rule aligns reference-valued assignment with initialization and ordinary
C# class-reference assignment, while retaining target writes for value RHS and outputs.

This is a Neo compiler projection over existing runtime operations, not a change to
CLI-style slot instructions. It removes a surprising difference between initialization
and reassignment, at the cost of RHS-type-dependent behavior and a breaking change
for existing reference-to-reference target copies. No performance benefit is claimed.
A separate universal reference-slot output contract remains future runtime/language
work; C# out-class-reference behavior is not yet available through Neo out Foo&.

## Migration and validation

Old `left = right` that copied a Foo from Foo& must use an explicitly value-typed
intermediate. If assignment was intended to retarget, make the binding var; let now
rejects this operation. Existing `left = &right` retargeting remains valid for mutable
locals. Recompile source; already compiled artifacts retain their emitted operations.

Run the checked C# comparison with SDK 10.0.100:

```sh
cd docs/experiments/reference-assignment/dotnet
dotnet run --project Comparison.csproj
```

It prints 1, 11, 40, 11, 10 and checks class-reference assignment, parameter-local
replacement, output replacement and ref-local writes/reassignment. The output-slot
case intentionally shows a facility beyond current Neo output projection.

```sh
cargo test --locked --test neo_reference_assignment --test neo_managed_access --test neo_outputs --test neo_closures --test neo_arrays --test neo_collections --test readonly_storage --test reference_experience
```

Neo tests use source/artifact loading and cover identity-preserving stores, closure
cells, exactly-once factory evaluation, immutable binding failures, output target
writes, readonly conversion rejection and runtime rejection of heap-stored frame references.

## Base, interface and readonly views

Run `cargo run --locked -- run examples/source/reference-views.neo`. It prints 42
three times and returns 42. An inferred generic result retains its concrete reference
before assignment converts it to a base view. Assigning that view to a readonly
interface reference retains the same concrete owner and virtual dispatch. The example
checks identity with ReferenceEquals across aliases, a field and an ArrayList element;
the former owner remains unchanged. This reuses the reference-copy decision above and
Neo's existing view conversions; it adds no allocation or conversion mechanism.

There are two separate readonly boundaries. A writable holder can replace a field
whose stored reference is readonly; that qualifier limits access through the stored
reference. A readonly view of the holder prevents replacing its field. Neither choice
makes other writable aliases to the target readonly. Regression tests cover the latter
write rejection and a reassigned interface reference surviving its factory frame and
collection pressure with a two-object heap budget.
