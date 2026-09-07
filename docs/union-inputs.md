# Bootstrap Option and Result inputs

Resolved IL functions accept owned bootstrap Option<T> and Result<T,TError> values,
including inside records, generic record receivers, and other Option/Result payloads.
This extends the experimental Rust invocation boundary. It does not add IL operations,
metadata categories, or a permanent union ABI. The [ordinary-type union convention](unions-and-enums.md)
remains the planned replacement for these bootstrap representations.

## Validation contract

Resolution builds schemas for every alternative, using the same depth-64 and
16,384-node limits shared across parameters and the receiver. Recursive by-value
schemas are rejected, including cycles through Option or Result. A finite input value
such as None does not make a recursive declaration importable in this subset.

Import requires Value::Union with the exact declared type after scoped type
normalization. An Object carrying a union type tag does not satisfy this contract.
Only these cases are accepted:

| Declared type | Case | Required payload |
| --- | --- | --- |
| Option<T> | Some | Exact owned T |
| Option<T> | None | Value::Void |
| Result<T,TError> | Ok | Exact owned T |
| Result<T,TError> | Err | Exact owned TError |

Payload validation is recursive. Byte requires Value::Byte, Single requires
Value::Single, and records require exact field counts and types. None's placeholder
is validated even though it carries no user data. Some<Void> remains distinct from
None; Ok and Err remain distinct in Result<T,T>. TError need not be the built-in Error.
Malformed inputs produce a Fault identifying the argument or receiver, case payload,
and nested field indices before any guest instruction executes.

Every declared alternative must be importable. Option<Int32*> and
Result<Void,Ref<Int32>> fail resolution even when a caller intends to pass None or Ok.
Pointers and prototype Ref values still need a separate lifetime/transfer contract.
Ordinary records marked with UnionAttribute continue to use ordinary record validation;
the attribute does not grant these bootstrap case semantics.

## Owned transfer and execution

Values enter a fresh execution as owned data. Clone a value first to retain a host
copy. Supported returned values can be re-imported, with validation repeated against
the destination definitions. This does not retain guest allocation identities or
prove provenance, construction invariants, or pointer validity.

The existing IL rules still apply: ldarg loads the union value unchanged and ldcase
extracts its stored payload. This slice introduces no new stack normalization, native
layout, implicit allocation, exception mechanism, or memory management.

`cargo run --example union_inputs` supplies successful and failed Result<Void,Error>
values to an IL function and passes an Option<String> result into another invocation.
