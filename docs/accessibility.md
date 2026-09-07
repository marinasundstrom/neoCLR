# Initial method accessibility

Methods and free functions now carry explicit accessibility metadata. This first slice
restricts calls and host member invocation; it does not yet hide types or fields or
establish a complete encapsulation contract for ordinary union carriers.

The current direction is to retain familiar .NET-style access levels while building
the fundamentals. A possible improved access model is deferred; it is not required
for this slice. The prototype uses its module as the internal boundary, since a richer
assembly/package model has not been implemented.

## Syntax and scopes

```text
.method public static Create(Int32 value) -> Gauge
    ldarg value
    call Gauge::Clamp(Int32)
    newobj Gauge
    ret
.end

.method private static Clamp(Int32 value) -> Int32
    ; ordinary IL body
.end

.function internal Helper(Int32 value) -> Int32
    ; ordinary IL body
.end
```

The optional visibility keyword precedes static/instance on a method, or the signature
on a free function. Omitting it retains public visibility for existing sources and JSON.
The JSON `visibility` field accepts `public`, `internal`, or `private`; public is omitted
when serializing. Visibility is not part of overload identity or a call operand.

| Visibility | Allowed IL callers |
| --- | --- |
| public | Any otherwise valid caller |
| internal | Functions in the declaring module and revision |
| private | Methods of the same declaring type definition and module/revision |

Private requires a declaring type. Use internal for a module-level helper. Private
scope refers to the generic type definition, so closed instantiations of that definition
share its private member access; unrelated types do not. No protected, inheritance,
nested-type, friend-module, or package access rules are introduced.

## Enforcement

Loading checks every explicit IL call, including unreachable instructions, against the
resolved callee. The interpreter checks calls again using the active caller. Supplying
a method token, changing a symbolic spelling, or associating a property with an accessor
does not grant permission to invoke that method. Existing module reference constraints
remain independent requirements; declaring a reference does not grant internal access.

Resolved host invocation requires public methods, even for the root module. This applies
to static and instance calls and to native-enabled invocation; the native trust switch
does not grant private-member access. Once resolved, handles borrow immutable prepared
metadata with that access decision already checked.

The explicit entry point is a separate execution root. A module may designate its own
internal function or private static method as its entry point, and run executes it.
Choosing another module's non-public function as the entry point is rejected. A local
non-public entry does not become available through arbitrary host member resolution.

Read-only reachability analysis may use non-public roots: analysis does not execute or
return an invocation handle. Likewise property and marker-attribute metadata can refer
to methods without invoking them. These associations are still signature-validated, but
do not confer callable access. No guest reflection-based invocation is implemented.

## Property access and remaining boundaries

Properties still consist of metadata associations with ordinary methods. A public getter
and private setter are possible; direct calls to each accessor follow that accessor's
visibility. Properties have no separate visibility field in this slice.

Type and field accessibility remain unimplemented. Raw field loads/stores, field-based
newobj, public factories returning records, and validated host record inputs retain their
current rules. A private helper does not prevent code from constructing a record through
its public fields or bypassing a factory's policy. The Gauge example therefore demonstrates
method boundaries, not a protected representation invariant.

Construction/initialization, type and field access, and unsafe pointer boundaries must be
specified before claiming that a carrier's representation is protected. Accessibility
is not a security sandbox or an ownership policy, and it cannot contain arbitrary native
code. It does not add a value/reference distinction or any union-specific instruction.

The additive metadata field remains in prototype format 3. Old readers that reject unknown
fields cannot read new non-public declarations; this is not a CLI binary metadata writer.

## Demonstration

```sh
cargo run --locked -- verify examples/accessibility.neoil
cargo run --locked -- run examples/accessibility.neoil
```

The internal entry calls public Gauge.Create, which calls private Gauge.Clamp on its own
type. A public property accessor reads the resulting value, and a module-internal helper
doubles it. Output is `42`, followed by the CLI's `=> Void` line. Use the README's normal
assemble/run workflow for a serialized artifact.

Tests exercise permitted and rejected IL calls, inaccessible unreachable calls, module
revision boundaries, generic private members, local and foreign entry selection, host
invocation with symbolic and bound references, property associations, and legacy defaults.
