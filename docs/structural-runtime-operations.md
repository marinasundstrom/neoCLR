# Structural runtime operations

Development contract on `codex/structural-union-intersection`, recorded 2026-10-15.
This guide separates type identity, runtime operations and execution-backend
qualification. A valid native signature is not proof that every JIT/AOT backend
can execute every operation on it. See the [family reference](../api-docs/structural-types.md)
and [native qualification record](experiments/native-structural-types/README.md).

## Signatures and values

| Family | Native signature notation | Value and operations |
| --- | --- | --- |
| Union | `union<A,B>` | Structural value holding one active payload; membership tests and checked extraction |
| Intersection | `intersection<A,B>` | View of a value satisfying every nominal bound; checked admission and interface dispatch |
| Tuple | `tuple<A,B>` | Ordered inline value; construction, positional reads/writes, copies and addresses |
| Function | `fn<P0,P1,R>` | Callable shape; bind a compatible method, then invoke with its parameters |

These are signature types, not synthesized nominal declarations. Component types
retain their resolved identities. Union/intersection normalization removes order
and duplicate distinctions; tuple order and function parameter order matter.
Tuple element names are preserved metadata but are not part of positional identity.
Function signatures also preserve parameter modes and no-result conventions;
`fn<...>` examples here show the simple by-value/result form.

A union remains a value when its payload is an object reference: copying that union
copies the reference, preserving the object's identity. It does not give the union
Object inheritance. Tuples likewise have value semantics without nominal ancestry.
Function objects have their own Object-compatible callable contract. Arrays are
structural at runtime and also representable through nominal `Array<T>`; that
spelling does not require an additional wrapper object.

## Testing and extracting a union alternative

There is no universal structural-union `GetCaseN` operation. Test the active
payload against a type contract. `isinst T` produces a compatible view on success
or null on failure; it does **not** directly produce a Boolean. Convert its result
with `ref.isnull` when branching. For a reference alternative:

```text
ldloc value
isinst Response
ref.isnull
brtrue not_response
ldloc value
castclass Response
stloc response
```

This is a schematic instruction fragment; Response, the union local and labels
must be declared by the containing program. Membership uses nominal assignability,
including explicitly implemented interfaces. Several interface alternatives may
match the same payload; the runtime does not assign semantic priority by union
constituent order. Raven's pattern-arm order supplies that priority.

`castclass T` performs checked extraction/conversion; an invalid extraction fails.
A pattern first tests and branches, so a nonmatching arm is not an extraction
fault. Null passes through reference casts, but fails non-null type patterns.
An intersection target requires every bound; a union target requires at least one.
Fault categories for legacy nominal casts and structural casts are not yet fully
uniform; focused native tests compare the relevant interpreter behavior.

For value alternatives, the current instruction-level conversion is:

```text
ldc.i4 42
box Int32
castclass union<Int32,Int64>
stloc value
ldloc value
isinst Int32
ref.isnull
brtrue not_int32
ldloc value
unbox.any Int32
```

The interpreter stores the value payload in a box for this conversion. Unboxing
requires the exact type: Int64 is not Int32 even if both contain 42. Wrong-type
unboxing raises InvalidCast; null unboxing raises NullReference. These are payload
operations, not permission to box an entire union as its active payload and lose
the union's type identity.

Native reference-only unions have bounded membership/extraction qualification.
Scalar-only native union storage is being qualified with a private tag plus a
numeric payload for Int32, Int64 and Boolean. An adjacent exact scalar box/union
cast can omit the box allocation. Native `isinst Int32`, `Int64` and `Boolean` followed immediately by `ref.isnull`
now compare the inline discriminator, without boxing. Tests cover each alternative
and default null against the interpreter. The temporary test result cannot escape
or be duplicated. A [Raven-produced consumer](experiments/native-structural-types/scalar-pattern-project/README.md)
also validates typed bindings, guarded/fallback arms and a result join against the
interpreter and AOT. A separate [artifact-only library consumer](experiments/native-structural-types/scalar-import-project/README.md)
qualifies union returns and parameters with reordered alternatives and the same
zero-arena result. This does not qualify every match or arbitrary stack producer.
Mixed Int32/Int64/Boolean plus nominal reference alternatives now use a separate
reference slot alongside tag/numeric payload; see the [mixed storage qualification](experiments/native-structural-types/README.md#mixed-scalarreference-union-storage-2026-10-15).
A [fresh mixed Raven consumer](experiments/native-structural-types/mixed-pattern-project/README.md)
passes seven scalar/reference inputs in interpreter and AOT, allocating only its
explicit Response object. Closed nominal value-record alternatives now have bounded inline storage with
exact identity tests and copied field snapshots; see the [record-union qualification](experiments/native-structural-types/README.md#nominal-value-record-union-alternatives-2026-10-15).
A [fresh imported Raven struct consumer](experiments/native-structural-types/record-import-project/README.md)
also preserves Int32/Int64 fields through union calls and patterns with zero
arena use and no native imports. Combining records with direct reference alternatives and intrinsic String alternatives
remain native layout gaps.

An [artifact-only mixed library consumer](experiments/native-structural-types/mixed-import-project/README.md)
also passes imported union returns/parameters and Response property access with
only the explicit object allocation. The [closed-generic imported variant](experiments/native-structural-types/generic-mixed-import-project/README.md)
also preserves a stored Response<int> payload through mixed union calls.

The current interpreter admits a null/default union payload. Native bounded
storage preserves that behavior. It is not a separate named union case and is not
evidence that every alternative is nullable. See the [union contract](../api-docs/unions.md)
for the evolving default/boxing boundary.

## Intersections

`castclass intersection<Animal,Quacks>` checks both nominal contracts on the same
object. After a successful check, an interface view can be extracted and its
member invoked through ordinary interface dispatch. This does not inspect member
names or synthesize interface implementations. Bounds reject classes closed to
extension in the .NET sense; Raven sealed hierarchies are a different concept.
Intersections can appear in signatures and constraints, including results where
supported; their common use as inputs is guidance, not a return-type prohibition.

The bounded native representation is one traced reference slot. See the tested
[intersection instruction program](experiments/native-structural-types/intersection-views.neoil)
for casts, copies, interface calls, output parameters and null handling.

## Tuples

Construction consumes ordered element values. For an unnamed two-element tuple:

```text
ldc.i4 40
ldc.i4 2
newobj tuple<Int32,Int32>
stloc pair
ldloc pair
ldfld 0
```

The field operand is an element index. `ldloca`, `ldflda`, `stfld` and `stobj`
operate on mutable tuple storage or nested element addresses under normal borrow
and verification rules. Loading or passing a tuple copies its value; mutating a
copy does not mutate the original. Reference elements still share their referents.
Names select positional elements at source level and remain available in native
metadata/reflection without an attribute-based naming workaround.

Bounded native tuples use flattened eight-byte private slots, including nested
storage, and recursively trace only reference-bearing elements. This is not a
packed or public ABI. The [native values tests](../runtime/codegen/tests/values.rs)
cover copies, nested mutation, calls/results and layout limits. Tuple arrays,
boxing and complete managed JIT/collector qualification remain separate work.

## Function values

A function type describes a callable signature; a function value retains a checked
target and, where applicable, a bound receiver/environment. In the simple notation,
the last `fn` argument is the result, so `fn<Int32,Int32>` has one Int32 parameter
and an Int32 result; `fn<Int32>` has no parameters and an Int32 result.

```text
function.bind fn<Int32,Int32> = AddOne(Int32)
ldc.i4 41
call instance fn<Int32,Int32>::Invoke(Int32)
```

This fragment requires a compatible AddOne declaration. Binding validates its
signature. Invocation consumes the callable and its arguments, executes the bound
target and returns its result. Binding an instance target additionally consumes
its receiver; a retained callable must not let a frame-bound reference escape.
Closed generic targets and signatures containing structural arguments have their
own admission checks.

The tested composition callback form is:

```text
function.bind fn<Int32,union<Int32,Both>> = Wrap(Int32)
ldc.i4 42
call instance fn<Int32,union<Int32,Both>>::Invoke(Int32)
unbox.any Int32
```

The complete Wrap/Both declarations and incompatible-target checks live in
[structural composition tests](../tests/structural_compositions.rs). This is
interpreter composition evidence, not qualification of that mixed-union callback
in native AOT. Function shape/member reflection and Object behavior are documented
in the [Function reference](../api-docs/functions.md).

## Introspection and backend responsibilities

Common queries use TypeInfo; structural does not mean uninspectable.
UnionTypeInfo and IntersectionTypeInfo expose constituent Types. TupleTypeInfo
exposes ElementTypes and ElementNames. FunctionTypeInfo describes the callable
signature. Display text is diagnostic rather than an identity key. A structural
descriptor has no invented nominal declaration merely to support reflection.
Native descriptor reflection is not yet qualified for every backend/type family.

The compiler emits typed operations and signatures. The runtime verifier checks
them; the interpreter and native backends select representations, perform checks
and preserve copy/borrowing behavior. GC maps must distinguish references from
numeric payload bits. AOT must resolve and lay out the same admitted types without
requiring a JIT; implementation gaps are rejected rather than silently erased.
Whole-union boxing, mixed payloads, general type-test results, broader reflection,
arrays and full application/JIT/collector validation still block integration.

Raven's corresponding [lowering and pattern map](https://github.com/marinasundstrom/raven/blob/codex/neoclr-union-intersection/docs/compiler/neoclr-structural-lowering.md)
records the frontend path separately. Runtime instruction snippets here are
illustrations derived from the linked implementation/tests, not standalone sample
files or an assertion that every combination has been validated.
