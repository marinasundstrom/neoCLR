# Minimal read-only type inspection

Preview 1 exposes the loaded metadata's existing type identities through an ordinary
System.Type record and an opaque System.RuntimeTypeHandle. This is metadata inspection,
not payload erasure, reflection-based execution, or a common object base type.

| API / IL | Behavior |
| --- | --- |
| `ldtoken T` | Push an owned RuntimeTypeHandle for a closed type signature |
| `System.Type.GetTypeFromHandle(RuntimeTypeHandle)` | Wrap a handle as System.Type |
| `System.TypeOf<T>.Of(T value)` | Describe the declared T without examining or erasing the payload |
| `Type.Name` | Canonical qualified definition name; pointer signatures include their target spelling |
| `Type.GenericArgumentCount` | Number of generic arguments of a constructed type |
| `Type.GetGenericArgument(Int32 index)` | Return the selected closed argument descriptor; invalid indices Fault |
| `Type.Equals(Type other)` | Compare canonical type identities, not display names or object addresses |

The value helper is deliberately static: Of<Byte> describes Byte even though small
integers normalize to Int32 on the evaluation stack. Of<Void> works; pointer values
can be described without dereferencing them. Of<System.Value> describes System.Value,
not its erased contents. Inheritance/dynamic object dispatch is not implemented and
this helper does not predict its future semantics.

Name matches the host descriptor's existing naming contract. A Box<Int32> descriptor
has Name `Box` and a separate System.Int32 argument descriptor. This is not the CLR
Name/FullName/assembly-qualified-name formatting contract. A pointer to Box<Int32>
is named `Box<System.Int32>*`; pointer and managed-reference signatures expose no generic
argument list of their own. No element-type property is provided in this subset.

## Identity and lifetime

Handles contain owned read-only descriptor snapshots in the interpreter. A returned
handle remains readable by the embedding host after the execution and LoadedProgram
are dropped. Copying a handle or its System.Type wrapper copies metadata, not a guest
object or its payload. The implementation uses host-owned storage; this is not an
allocation-free guarantee. There is no guest pointer identity or cleanup obligation.

Identity uses the existing module, optional revision, definition row and recursively
closed signature arguments. Names alone do not identify a type. Identities describe a
resolved build, not an authenticated assembly identity or stable cross-build cache key.
A module name/row reused without revision changes is not a content hash. Duplicate
same-name definitions across one load set remain subject to existing loader limits.

RuntimeTypeHandle is a runtime-known type with no native layout or P/Invoke ABI.
It cannot be fabricated through record construction, pointer casts or native loads.
For this preview, host invocation rejects RuntimeTypeHandle inputs, including inside
System.Type, records or erased payloads. Hosts may inspect returned metadata and use
the existing host describe_type API; descriptor re-import needs a separate binding
contract. These restrictions avoid silently accepting handles from another load set.

## Validation and backend contract

ldtoken takes only a type signature. Method and field tokens are not supported. It
supports generic type parameters in generic bodies, substituted at closed invocation;
unknown, invalid-arity or unbound types are rejected. Normal module-reference and type
accessibility rules apply to the operand. A handle grants no ability to invoke private
methods, construct inaccessible types or mutate fields.

System.Type uses four narrow InternalCall helpers for name, identity comparison and
generic argument inspection. The existing binding registry validates their signatures.
Reachability reports the TypeInspection runtime service for ldtoken and those imports;
no System.Value or ValueStorage service is involved. A future JIT/AOT backend must
retain the required metadata and supply this service. The interpreter's owned snapshots
are not a prescribed native handle layout or an implemented AOT backend.

The type-only ldtoken spelling is CLI-aligned. The prototype serializes it as an
opcode in the current format 5 with a type operand and RuntimeTypeHandle as a canonical
signature. Older readers reject unsupported instructions/signatures. No CLI binary
token assignment or .NET artifact compatibility is claimed.

## Run the demonstration

```sh
cargo run --locked -- verify examples/type_inspection.neoil
cargo run --locked -- run examples/type_inspection.neoil
cargo test --locked --test type_inspection
```

Expected lines: `System.Int32`, `Box`, `1`, `System.Int32`, `Same type`, `=> Void`.
The sample creates a Box<Int32> value, gets its declared type and compares it with a
statically named Box<int> token. The walkthrough acceptance test also runs its source
and assembled artifact through the CLI.

Member enumeration, construction/invocation by descriptor, dynamic object GetType,
custom-attribute reflection, mutable metadata and a comprehensive Type API remain
outside this slice. The prototype names and API may evolve independently of .NET's
reflection hierarchy.

## Neo source operator

The companion compiler exposes `typeof(T)` as an ordinary System.Type-producing
expression, using the same ldtoken/GetTypeFromHandle path. It accepts type signatures,
not value expressions. `typeof(int).Name`, closed generic argument inspection and
`.Equals(...)` work through existing library getters/methods. Read-only property
syntax resolves declared public getters without granting access to backing storage.

```sh
cargo run -- run examples/source/typeof.neo
cargo test --test neo_typeof --test type_inspection
```

The [Neo guide](neo.md) and [grammar](neo-grammar.md) describe the source subset.
This addition changes neither the runtime opcode set nor the module format.

## Next implementation slice

[Reflection introspection](neo-roadmap.md#next-slice-reflection-introspection) is next:
field, method and property enumeration with familiar System.Reflection descriptor
names, parameter/type details, and explicit managed-reference metadata. These APIs
are planned, not implemented by the current minimal Type inspection surface.

The planned Type member queries use independent MethodInfo, FieldInfo and PropertyInfo
records; they do not depend on class hierarchies. FunctionInfo is reserved for possible
module-level free-function queries, not a category of Type member.
