# Explicit interface implementations

Implemented 2026-09-08. A record can give different bodies to same-signature interface
members, independently of its public methods. For example:

```swift
interface Readable { readonly func Read() -> int }
interface Identified { readonly func Read() -> int }
record Counter(Value: int): Readable, Identified {
    readonly func Readable.Read() -> int { return this.Value }
    readonly func Identified.Read() -> int { return 1 }
}
```

Borrow Counter as Readable& or Identified& to select the contract. Counter.Read() is
not introduced. The existing automatic managed-reference access rules apply. The
qualifier names the interface declaring the member, including for inherited contracts.
Neo currently accepts local nongeneric interface names here, consistent with its small
declaration subset. Generic mapping is supported in IL/artifacts.

## Runtime contract and IL

Function.interface_implementations stores explicit FunctionRef declarations for that
body. Identity includes the declaring interface, member identity and substituted type
arguments; qualified body names alone have no dispatch meaning. IL can use any valid
body name. Multiple declarations can map to one body when their contracts match:

```text
.method private instance readonly byref Readable.Read() -> Int32
.override instance Readable::Read()
ldarg this
ldfld 0
ret
.end
```

The directive precedes instructions. This is distinct from `.methodimpl InternalCall`,
which describes implementation flags, and from the class `override` modifier.

Each body must be private, concrete, instance, managed-receiver IL on a record. It
cannot be a constructor, native method, abstract method or class virtual/override.
The owner must declare the interface, directly or through an interface base; an
inherited conformance alone does not authorize adding a new mapping. Parameter and
return types, readonly receiver/parameters and output guarantees must match exactly.
Duplicate, unresolved, noninterface and incompatible mappings fault during loading.
Generic mappings that become ambiguous after substitution fault on use/verification.

The nearest class declaring conformance anchors mapping. At each class, explicit
mapping takes precedence over a matching public method; otherwise continue to the
next ancestor. An inherited mapping keeps its body until interface redeclaration
restarts that search. Explicit bodies do not occupy class virtual slots, so a public
virtual method with the same signature remains independent. An explicit body may
call a class virtual method when that is the desired extension point.

Interface callvirt checks access to the contract, then enters the selected body with
the original managed owner projected to its declaring class. Ordinary calls retain
private access checks. This introduces no boxing or additional managed allocation.
Heap views retain the complete owner; frame escapes fault, and readonly/output
contracts remain enforced even without running the verifier first.

Closed interface analysis includes selected private bodies. Stack traces identify the
actual body. MethodInfo queries with NonPublic/Instance flags expose private bodies;
Neo-generated names retain the interface qualifier, while public queries exclude them.
There is no GetInterfaceMap API yet. Property/indexer accessors can use the same IL
method mapping mechanism; new Neo property/indexer declaration syntax is deferred.
[Default interface bodies and reabstraction](default-interface-implementations.md)
are now supported on interface owners. Existing
rejection of incompatible contracts combined into one interface is unchanged.

## .NET comparison and tradeoffs

Primary sources consulted 2026-09-08:

- [C# interface specification §§19.6.2, 19.6.5–19.6.7](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/interfaces):
  explicit implementations provide a separate interface surface and participate in
  inheritance/reimplementation.
- [TypeBuilder.DefineMethodOverride (.NET 10)](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.typebuilder.definemethodoverride?view=net-10.0):
  associates a body with a declaration independently of body naming. Its MethodImpl
  description identifies the two metadata tokens; the example uses private, virtual,
  final body flags.

The SDK 10.0.100/net10.0 probe records those flags and tests distinct interfaces,
private visibility and reimplementation. neoCLR uses explicit mapping metadata
without marking the body as class-virtual: class overridability and interface dispatch
are separate characteristics. This avoids conflating two dispatch mechanisms, but
MethodInfo.IsVirtual differs from .NET for these bodies and an IL importer would need
to translate flags. This is an intentional preview adaptation, not binary compatibility.

One observed edge differs: Base explicitly implements Readable.Read and also declares
public virtual Read; Middle overrides Read; Leaf inherits Middle and redeclares
Readable. The pinned .NET probe returns Base's explicit result (21), while neoCLR's
per-class search selects Middle's public override (42). Redeclaring and overriding in
the same class returns 42 on both. We keep neoCLR's uniform per-class search as the
provisional rule: it avoids a special distinction between inherited and newly declared
overrides during reimplementation. The cost is this observable compatibility gap.
The probe establishes behavior, not its Roslyn/CoreCLR cause; deeper import compatibility
research remains open before promising C#-identical mapping.

Name-based conventions alone would be easier for one compiler but ambiguous across
languages and overloads. Explicit runtime metadata costs validation and mapping
traversal, and needs preservation in linking, generic substitution and artifacts.
It supplies a stable contract for future default-body precedence. No performance
improvement is claimed.

## Run, validation and migration

```sh
cargo run --locked -- run examples/source/explicit-interfaces.neo --gc-stats
cargo test --locked --test explicit_interfaces
(cd docs/experiments/explicit-interfaces-dotnet && dotnet run)
```

The Neo sample returns 42. Tests cover source/artifact roundtrips, distinct mappings,
class virtual independence, inheritance/reimplementation, private access, malformed
metadata, generic substitution/collisions, external module identities, accessor mappings,
readonly/output enforcement, GC, frame
escape and private reflection names.

Old artifacts omit the new field and retain implicit mapping. New artifacts with
explicit mappings require this runtime; Rust Function literals need the new vector.
There is no new opcode or System descriptor layout. Existing public APIs are unchanged.
[Default interface bodies](default-interface-implementations.md) now build on these
mappings; record explicit implementations still cannot be abstract or class-virtual.
