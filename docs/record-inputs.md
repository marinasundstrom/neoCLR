# Owned record inputs for host invocation

Resolved IL functions can accept ordinary record Values in addition to
primitive and bootstrap Option/Result inputs. Parameters may contain nested records,
closed generic records, and [validated union payloads](union-inputs.md).
Input fields are positional and must have their exact declared storage types:

```rust
let point = neoclr::Value::Object {
    ty: neoclr::assembler::parse_type("[Models]Point")?,
    fields: vec![neoclr::Value::Int32(20), neoclr::Value::Int32(22)],
};
let result = function.invoke(vec![point], neoclr::Limits::default())?;
```

## Schema and value validation

Function resolution builds a schema from the loaded definitions and substituted
generic fields. It is independent of native memory layout: records with String,
Error, or Void fields can be imported without having a native ABI or byte layout.
Finite generic nesting such as Box<Box<Byte>> retains every field type.

Before execution, import checks each record's nominal type, field count, and all nested
values. A forged Value::Object tagged as a primitive cannot satisfy a primitive field.
Byte fields require Value::Byte; Int32 values are not implicitly narrowed at the host
boundary. Scoped record tags must name the correct module and are normalized to the
loaded representation before reaching guest code. Failures identify the argument and
nested field indices without inventing a guest instruction location.

The schema is bounded to depth 64 (root depth zero) and 16,384 total nodes across input
parameters and the instance receiver. Recursive by-value definitions are rejected, and expanding generic schemas
also remain subject to existing signature nesting/substitution limits. These are
prototype import limits, not native layout constraints or a universal memory policy.

## Owned data and remaining limits

System.Value fields now use [bounded erased-payload import](erased-inputs.md). Their
concrete payload schemas are resolved at invocation, with shared per-value traversal
and dynamic-schema budgets. This supports ordinary Option/Result carrier inputs
without giving their marker special runtime meaning.

Arguments move into invoke as owned Rust Values. A caller retaining a copy can clone
the Value first; guest updates do not alter that retained copy. Valid record results
can be imported into later invocations, with the same validation performed again.
Import is typed data transfer against the destination definitions, not preservation
of an execution handle, pointer identity, or originating artifact provenance. It does
not run constructors or establish additional library invariants.

Pointer and Ref fields remain unsupported at this boundary, including when nested
in a record or a bootstrap Option/Result alternative, or ignored by the function body.
An unused generic argument is not a stored field: Empty<Ptr<Int32>> with no fields
can be imported because it carries no pointer value. Raw pointer/Ref transfer still
needs its own contract. Explicit instance receivers use the
same owned-data validation; see [invocation](invocation.md).

Invocation still starts fresh guest state, uses exact primitive storage values, and
returns the existing Execution. Native imports retain their separate unsafe contract.
No allocator, automatic destructor, reference counting, or garbage collector is added.

`cargo run --example record_inputs` keeps a copy of Point(20,22), updates another copy
to Point(40,22), and prints sums Int32(42) and Int32(62).
