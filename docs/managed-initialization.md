# Managed value initialization

Status: initobj T accepts a managed T& destination in addition to its existing
native-pointer path. It default-initializes existing typed storage without allocating
a new guest object, invoking a constructor or pushing a result onto the stack.

```text
.type Counter
    .field Age Int32
.end

.function Main() -> Int32
    .local Counter counter
    ldloca counter
    initobj Counter
    ldloc counter
    ldfld Counter::Age
    ret
.end
```

The destination may be an uninitialized whole slot, an initialized slot being reset,
or an initialized record's addressed field. Exact target types must match. Managed
initialization constructs a complete typed default before replacing the destination;
a failed default construction does not write or fulfill an output promise. It uses
the normal slot write path, preserving root identity and existing field aliases.

## Supported defaults

| Type | Default |
| --- | --- |
| Integer and native-integer primitives, Char | Zero in the exact declared storage type |
| Single and Double | Positive floating-point zero |
| Boolean | false |
| Void | The ordinary inhabited Void value |
| Ptr<T> | Null raw pointer with no allocation identity; dereference remains invalid |
| Ordinary records, including closed generic records | Recursively default every field under these rules |

Zero-field records are supported. Pointer fields can refer to recursive record
types without recursively constructing pointed-to values. Recursive by-value shapes
are rejected; construction is bounded to 64 levels and 16,384 value nodes.

String, Error, System.Value, RuntimeTypeHandle, legacy Ref, managed references and
interface views have no default in this subset. A record containing such a field
also has no default. No null managed reference, empty erased payload or default
String is silently invented. Broader defaults need their own semantic decision.

Default initialization bypasses constructors, including for records with private
fields. Field visibility is not a defaultability marker. This means constructor-only
invariants are not a guarantee for a type that supports initobj; use the explicit
initialization contract rather than infer constructor provenance from its type.
Type accessibility still applies. Native-pointer initialization continues to require
its supported layout, valid allocation, bounds and alignment.

## Outputs, constructors and verification

initobj is a write, so it can fulfill out/out(true) obligations without first reading
an uninitialized whole destination. Resetting a field fulfills an output promise
for that field; resetting its ancestor also fulfills it. Sibling writes do not.
Forming a field address still requires an initialized containing record; initobj
does not introduce general partial record construction.

An existing value-construction .ctor body can use ldarga this and initobj T to
initialize its own receiver before reading or addressing fields. Verification tracks
that whole-receiver initialization on every control-flow path. The
[value initialization example](../examples/value_initialization.neoil) demonstrates
this followed by an ordinary field assignment. Existing calls to .ctor on a copied
receiver are unchanged; construction into an arbitrary caller-supplied destination
remains a separate gate.

Closed unsupported defaults are rejected during validation. Open generic bodies are
checked when specialized types are available, with execution enforcing the actual
type even if verification is skipped. The verifier recognizes initobj through a
direct local address as definite initialization. Reference-local aliases still use
runtime output and initialization checks where static provenance is unavailable.

The native and managed operand paths share the existing opcode and stack effect.
Runtime-service analysis conservatively reports PointerMemory and SlotReferences.
There is no format change, newval instruction or new managed heap allocation here.
See the [heap strategy](managed-heap-strategy.md) for the next construction and
allocation decisions.
