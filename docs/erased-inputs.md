# Owned erased values at the host boundary

Loaded functions now accept explicit System.Value parameters and ordinary records
containing them. This permits an ordinary System.Option/Result created in guest IL
to leave the runtime and be supplied to a later invocation on the same loaded program.
There is no special host dispatch for those carrier names or their union marker.

```sh
cargo run --locked --example ordinary_union_inputs
```

The [Rust example](../examples/ordinary_union_inputs.rs) invokes the ordinary IL
Create(Int32) factory, receives System.Result<Int32,String>, and passes it to Read.
Expected output is `Int32(42)`. The [guest source](../examples/ordinary_union_inputs.neoil)
contains all carrier construction/query operations. No representation-specific host
factory or conversion intrinsic is required.

## Validation contract

System.Value input requires the explicit Rust `Value::Erased` wrapper. A plain Int32
does not implicitly become erased. Validation derives the enclosed payload's type,
checks it against the loaded metadata, and recursively imports it using the ordinary
primitive, record and existing bootstrap-union rules. Nested erasure stays nested.

- Scalars retain their exact storage types; a Byte input is not normalized to Int32.
- Record fields must have the right count and exact declared types, including generic
  substitution. A forged Object tagged as a primitive cannot satisfy that primitive.
- Scoped record tags normalize only when the supplied module is the definition's owner.
  Unknown types, unbound type parameters and invalid generic signatures are rejected.
- Pointer and Ref payloads remain unsupported, including inside records or nested erasure.
  Guest code may erase such values internally; that does not grant host re-entry rights.
- Each argument/receiver import limits value depth to 64 and total visited values to
  16,384, and shares a 16,384-node dynamic schema budget across its erased payloads.
  Nested erasure does not restart these budgets. Initial signature schema construction
  retains its existing shared limit. These are complexity guards, not total memory quotas.

Import errors precede execution and have no guest stack trace. Successfully imported
data follows ordinary value-copy semantics; native layout and guest allocation rules
are unchanged. Re-import validates against the destination loaded program; values are
not portable schema-independent identities or a serialized exchange format.

## Shape is not constructor provenance

The host input API remains a trusted data-construction boundary. As with ordinary
record inputs, it can populate private representation fields after shape checks.
It does not execute constructors, recognize union conventions, or prove that a marked
carrier contains an allowed wrapper. A host-built Result whose private System.Value
contains Void is structurally valid but violates the library's behavioral contract.
Its predicates/accessors behave according to their ordinary IL, potentially Faulting.

Prefer guest factory/constructor paths when creating invariant-bearing data for an
application. This slice makes guest-produced carriers reusable without teaching the VM
special union rules. Existing native helpers, public APIs, samples and bootstrap host
union paths still need migration before the old union representation can be removed.
