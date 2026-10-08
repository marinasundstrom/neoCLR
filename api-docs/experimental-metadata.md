# Experimental .NET metadata library

**Feature-branch development only; not part of Preview 11 or the neoCLR guest API.**
Assembly and namespace: `NeoCLR.Metadata.Experimental`. Target: .NET 10.
Source project: `tools/metadata/NeoCLR.Metadata.Experimental`.
This is the first reusable reader/writer slice intended for Raven's future symbol
loader and code-generation adapters. It reads/writes **NEOX 0.1 framing, structural signatures, reference tables and synthesized-member tables**,
and derives structural identities/member contracts against an explicitly supplied host catalog.
Bounded PE32 recognition and a read-only manifest-module/TypeDef model are implemented;
explicit AssemblyRef, nominal TypeRef and bounded method MemberRef resolution are
implemented. A controlled primitive/root-object builder writes ordinary CLI PE and native
format-5 assemblies, including native top-level functions. Direct PE/#Neo runtime loading now uses a transitional native execution section
with a reference-only CLI projection. A bounded binary native payload now avoids JSON parsing at runtime. General rewriting
and guest Introspection assembly loading remain pending.

## Unmanaged pointer signatures (development, 2026-10-07)

`SignatureType.PointerTo(elementType)` creates an immutable unmanaged pointer signature.
`PointerElement` exposes its target. Supported targets are scalar primitives (including
Void), and nested pointers, up to sixteen pointer levels. String, RuntimeTypeHandle,
nominal types, generic parameters, managed references and structural targets reject with
`ArgumentException`; a null target throws `ArgumentNullException`. Pointer vectors,
pointer generic arguments and pointer-bearing Function shapes are currently rejected.
These bounds are library limitations, not changes to the CLI type system.

Use pointer signatures for callable parameters/results and local slots. Definition
attachment and builder convenience methods share validation. `GetILGenerator()` preserves
exact pointer target identity through arguments, locals, calls and returns; returning a
different pointer target rejects during writing. This does not provide allocation,
ownership, pointer arithmetic or dereference operations in the C# instruction API.

CLI output uses standard `ELEMENT_TYPE_PTR` (including `PTR VOID`), and native output
uses the existing `Ptr` category without a format-version change. Both readers preserve
callable signatures and import them into new builders. `MetadataLoadContext` projects a
canonical `PointerTypeInfo`, exposing `ElementType`, `DisplayName` and
`IsNominalType == false`. No dependency loading or runtime reflection is required.

```csharp
var pointer = SignatureType.PointerTo(PrimitiveType.Void);
var identity = owner.AddMethod("Identity", new MethodSignature(pointer, [pointer]));
var il = identity.GetILGenerator();
il.LoadArgument(0);
il.Return();
```

Validation: 162 C# metadata groups pass, including CLI/native round trips, manual
method definitions, import, canonical introspection, depth/shape rejection and pointer
target mismatch. An API-authored native assembly calls NativeMultiplyChecked,
NativeAllocate and NativeFree and exits 42. Follow-up native module-function references
also accept these pointer signatures via `CreateFunctionReference`; its explicit
identity/digest contract remains unchanged. Raven now maps and consumes these callable
signatures, and separately compiled source NativeMemory consumers execute. Pointer
arithmetic and nominal pointer targets are still outside this bounded compiler gate;
full System output remains pending. [Source acceptance](https://github.com/marinasundstrom/neoCLR/blob/d622395e765b20d2f257e8c8a415db7bdb6743cd/docs/experiments/extended-cli-metadata/source-native-memory-2026-10-07.md).

## Generic classes over the native Object root (development, 2026-10-07)

`AssemblyBuilder.AddGenericClass(namespace, name, genericParameterNames, baseType,
visibility)` accepts this output's explicitly designated native Object root. The matching
manual `TypeDefinition` constructor accepts the same local base reference. The base must
already be attached; foreign roots and ordinary class bases reject. This bounded overload
does not admit constructed or generic bases, nested derived types or general generic
class inheritance. Existing core-backed generic classes retain their behavior.

Constructors must explicitly call the root constructor exactly once before ordinary
receiver access. Signature validation retains the full open generic receiver identity.
Constructed instances may be passed to inherited root methods; type argument storage
is preserved. CLI projection keeps the ordinary TypeDef base token; native metadata keeps
the existing named base edge. No encoding/version change is required, and executable
CLI output of a native Object root remains rejected.

The native reader preserves the base reference, and `NominalTypeInfo.BaseType` resolves
the canonical root through the metadata context. The declaration fact applies to the
generic definition; this does not add reflection or inherited member enumeration.

Validation includes definition/builder parity, base round trips, canonical facade identity,
missing initialization rejection and executed generic storage plus inherited virtual dispatch.
[Source Raven regression and limits](https://github.com/marinasundstrom/neoCLR/blob/d622395e765b20d2f257e8c8a415db7bdb6743cd/docs/experiments/extended-cli-metadata/generic-object-root-2026-10-07.md).

## Namespace and types

- [Native-width integers](#native-width-integers-development-2026-10-06).

- [Explicit callable nullable annotations](#explicit-callable-nullable-annotations-development-2026-10-06).

- [Native Object root authoring](#native-object-root-authoring-development-2026-10-05).

- [Function signature views](#function-signature-views-development-2026-10-03): native callback signatures and substitution.

- [Parameter names and authored value references](#parameter-names-and-authored-value-references-development-2026-10-03).

- [Native nested case metadata](#native-nested-case-metadata-development-2026-10-03): scoped local/external native case identities.

- [Union payload foundation](#union-payload-foundation-development-2026-10-03): inline storage and direct native value declarations.

- [Parameter passing modes](#parameter-passing-modes-development-2026-10-03): native and supported CLI ref/out metadata.

- [Native Self signatures](#native-self-signatures-development-2026-10-03): implementing-type contracts and scoped views.

- [Declaration facts](#declaration-facts-development-2026-10-03): constructors, flags and accessibility.

- [Generic method construction](#generic-method-construction-development-2026-10-03): metadata signature inspection.

- [Inherited interface views](#inherited-interface-views-development-2026-10-03): bounded metadata closure.

- [Property and direct interface views](#property-and-direct-interface-views-development-2026-10-03): scoped property signatures, accessors and relationships.

- [Method and parameter views](#method-and-parameter-views-development-2026-10-02): callable signatures and separate generic scopes.

- [Constructed and field views](#constructed-and-field-views-development-2026-10-02): signature projection and declared field metadata.

- [Native class-base reader views](#native-class-base-reader-views-development-2026-10-04): bounded local bases and standalone snapshots.

- [Introspection facade and MetadataLoadContext](#metadata-only-introspection-facade-development-2026-10-02): context-owned assembly, module and nominal-type views.

- [Runtime internal calls](#runtime-internal-calls-development-2026-10-05): bodyless service declarations.
- [IILGenerator](#iilgenerator-development-2026-10-02): independent library body-authoring contract.

- [Authored interface contracts](#authored-interface-contracts-development-2026-10-02): interface identity, conversions and dispatch.
- [External interface declarations](#external-interface-declarations-development-2026-10-03): complete contracts, relationships and authored PE emission.

- [Authored field references](#authored-field-references-development-2026-10-02): explicit field contracts and native layout slots.

- [Authored method references](#authored-method-references-development-2026-10-02): member contracts without input definitions.

- [Authored type references](#authored-type-references-development-2026-10-02): nominal reference-class identities without input definitions.

- [Authored function references](#authored-function-references-development-2026-10-02): native call contracts without input definitions.

- [ReferencedGenericType](#referencedgenerictype-development-2026-10-02): immutable constructed signatures in loaded snapshots.

- [Imported type signatures](#imported-type-signatures-development-2026-10-01): external nominal and constructed reference types.
- [Model namespace](#model-namespace): Cecil-inspired assembly/module/type definitions and scoped references.
- [Integer shifts](#integer-shifts-development-2026-10-01): Shl/Shr with Int32 counts.
- [Integer bitwise operations](#integer-and-boolean-bitwise-operations-development-2026-10-01): And/Or/Xor and helpers.
- [Signed remainder](#signed-remainder-development-2026-10-01): dividend-signed Int32/Int64 remainder.
- [Signed division](#signed-division-development-2026-10-01): typed Int32/Int64 quotient and execution faults.
- [MethodVisibility](#methodvisibility-development-2026-10-01): public/internal/private static method declarations.
- [TypeVisibility](#typevisibility-development-2026-10-01): public/internal static types and projection.
- [Argument stores](#argument-stores-development-2026-10-01): typed by-value slot reassignment.
- [String values](#string-values-development-2026-10-01): literals, signatures, locals and computed console output.
- [Imported generic methods](#imported-generic-methods-development-2026-10-01): bounded static MethodSpec calls across assemblies.
- [Primitive signatures](#primitive-signatures-development-2026-10-01): Int32/Int64/Boolean/String parameters and results.
- [MethodDefinition](#methoddefinition): callable declarations and bounded signature recognition.
- [MemberReference](#memberreference): physical references and explicit method resolution.
- [Branch labels and control flow](#branch-labels-and-control-flow-development-2026-10-01): Boolean conditions, joins and loops.
- [Int32 local slots](#int32-local-slots-development-2026-10-01): method-owned locals, raw indices and initialization checks.
- [Nominal signatures](#nominal-signatures): MethodSignature, SignatureType and owned class parameters/results.
- [Root-class locals](#root-class-locals): owned nominal slots and aliasing.
- [Primitive property associations](#primitive-property-associations): static/instance getter and setter metadata.
- [Root construction and instance bodies](#root-construction-and-instance-bodies): constructors, receiver calls and field operations.
- [Root classes and primitive instance fields](#root-classes-and-primitive-instance-fields): mutable layouts and field snapshots.
- [OpCode and MethodBuilder.Emit](#opcode-and-methodbuilder.emit): bounded opcode/typed-operand construction.
- [NativeLibraryDefinition and NativeFunctionDefinition](#nativelibrarydefinition-and-nativefunctiondefinition): native inventory and explicit partial callable views.
- [NativeModuleContainer](#nativemodulecontainer): existing native JSON translation without a CLI projection.
- [RuntimeAssemblyContainer](#runtimeassemblycontainer): direct PE/#Neo native execution transport.
- [MetadataArtifactReader](#metadataartifactreader): bounded PE extraction and recognition.
- [MetadataArtifact](#metadataartifact): ordinary classification or owned extended profile.
- [MetadataProfile](#metadataprofile): reference-profile reader and writer entry points.
- [MetadataProfileDocument](#metadataprofiledocument): owned typed view and explicit catalog resolution.
- [MetadataSection](#metadatasection): immutable, owned opaque payload and section metadata.
- [MetadataEnvelope](#metadataenvelope): bounded envelope read/write operations.
- [TypeExpression](#typeexpression): immutable raw signature syntax tree.
- [SignatureContext](#signaturecontext): local generic arities and Self permission.
- [StructuralSignature](#structuralsignature): validated signature read/write operations.
- [MetadataReference and MetadataDefinition](#metadatareference-and-metadatadefinition): catalog keys and declarations.
- [ReferenceBindings](#referencebindings): owned local references and binder owners.
- [ReferenceTable](#referencetable): reference payload read/write operations.
- [StructuralIdentity](#structuralidentity): catalog validation and normalized equality.
- [ResolvedTypeIdentity](#resolvedtypeidentity): immutable resolved equality key.
- [StructuralMemberReference](#structuralmemberreference): operation and local owner/ordinal.
- [ResolvedMemberIdentity](#resolvedmemberidentity): resolved owner/operation identity.
- [StructuralMemberDescriptor](#structuralmemberdescriptor): immutable derived contract.
- [StructuralMembers](#structuralmembers): member table codec and contract resolver.

These .NET-host-only types have this complete manual reference because the site's
RavenDoc source is the neoCLR guest compiler-reference assembly. They are not omitted
guest types or claimed executable neoCLR APIs. The maintenance record names this
boundary explicitly; source XML documentation also ships beside the built .NET DLL.

## MetadataSection

```csharp
public sealed class MetadataSection
{
    public MetadataSection(ushort kind, ushort version, bool required,
                           ReadOnlySpan<byte> payload);
    public ushort Kind { get; }
    public ushort Version { get; }
    public bool Required { get; }
    public int PayloadLength { get; }
    public byte[] GetPayload();
}
```

The constructor requires nonzero `kind` and `version` and at most 1 MiB of payload.
It copies the span into private storage. Invalid arguments raise
`ArgumentOutOfRangeException`. The payload-size limit does not imply the section fits
inside a complete envelope; headers also consume its 1 MiB budget.

`Kind` and `Version` identify the section schema. `Required` determines whether an
exact schema match is required by the consuming layer. `PayloadLength` is the byte
count. `GetPayload()` returns a new caller-owned byte array on each call; changing it
or the constructor's original buffer cannot change this section. Payload semantics
are not interpreted. A supported schema dictionary alone does not establish that a
payload is valid or executable.

## MetadataEnvelope

```csharp
public static class MetadataEnvelope
{
    public const int MaxImageSize = 1048576;
    public const int MaxSections = 64;
    public static IReadOnlyList<MetadataSection> Read(
        ReadOnlySpan<byte> image,
        IReadOnlyDictionary<ushort, ushort> supportedSchemas);
    public static byte[] Write(
        IReadOnlyList<MetadataSection> sections,
        IReadOnlyDictionary<ushort, ushort> supportedSchemas);
}
```

`Read` takes one complete image. It validates magic/version, size/count limits,
nonzero unique section kinds/versions, known flags, canonical contiguous ranges and
required schemas. It returns a read-only section list with owned payload copies, in
input order; the input buffer is no longer needed after the call. Unknown optional
schemas are retained byte-for-byte. Unknown required schemas fail. An empty dictionary
admits only optional sections. Callers must not mutate inputs during a call.

`Write` takes sections in desired output order, validates duplicate kinds, total size
and required schemas, and returns a new caller-owned image. Empty envelopes and
zero-length payloads are supported. Both operations reject malformed/unsupported
framing with `InvalidDataException`; null list/dictionary arguments raise
`ArgumentNullException`. A null section entry raises `InvalidDataException`.
The helpers perform no file I/O or assembly loading.

The schema dictionary maps each kind to one admitted schema version. It is an explicit
capability declaration supplied by the higher-level consumer, which must still decode
and validate known payloads. These framing routines cannot be used by themselves as
semantic acceptance for Raven or neoCLR. Preserving opaque bytes does not validate a
rewrite of the declarations/tokens to which they might refer.

The current NEOX layout preserves section order and exact payload bytes and rejects
gaps/overlaps/trailing data. Reading then writing a valid image therefore reproduces
its bytes. This is deterministic framing, not structural type canonicalization.
Copies simplify ownership at a memory cost; no zero-copy/performance claim is made.

## Example and validation

The separate `tools/metadata/MetadataConformance` executable builds this example
against the project reference and tests it through Python's independent reader:

```csharp
var schemas = new Dictionary<ushort, ushort> { [7] = 2 };
var bytes = MetadataEnvelope.Write(
    [new MetadataSection(7, 2, true, "abc"u8),
     new MetadataSection(60000, 9, false, [0, 255, 0])], schemas);
var sections = MetadataEnvelope.Read(bytes, schemas);
```

Kinds 7 and 60000 here are opaque test payloads, not production declarations.

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet.py
```

Four shared fixtures round-trip identically through .NET. Independently emitted .NET
bytes match Python emission; both reject 49 malformed vectors. The consumer also
checks required-schema mismatch, writer limits, unknown optional preservation and
buffer ownership. The reference assembly/source snapshot check remains unchanged and
passes; there are no new guest reference types in this slice.

## TypeExpression

```csharp
public sealed class TypeExpression
{
    public TypeExpression(string kind,
        IReadOnlyList<TypeExpression>? children = null, int index = 0,
        IReadOnlyList<string>? modes = null, bool noResult = false);
    public string Kind { get; }
    public IReadOnlyList<TypeExpression> Children { get; }
    public int Index { get; }
    public IReadOnlyList<string> Modes { get; }
    public bool NoResult { get; }
}
```

The constructor copies both lists into read-only owned collections. Children themselves
are immutable nodes. Null lists mean empty. Null `kind` raises `ArgumentNullException`;
null list entries, more than 257 children, or more than 256 modes raise
`ArgumentException`. Do not mutate supplied lists while constructing the node.
Construction alone does not validate a tree's shape: unknown spellings, inappropriate
indices/modes and context-dependent errors are diagnosed by the codec.

`Kind` uses the exact lowercase spellings `int32`, `string`, `bool`, `unit`,
`type_parameter`, `method_parameter`, `self`, `array`, `array_ref`, `tuple`, `function`,
`union`, `intersection`, `nullable`, `nominal`. These are experimental wire syntax,
not resolved runtime type names. `Index` is a zero-based generic ordinal, or a
one-based nominal reference index, and must otherwise be zero. `Children` are ordered;
Function parameters precede its last child, the result. `Modes` correspond to the
parameters only: `value`, `ref`, `readonly_ref`, `out`, `out_when_true`.

`NoResult` applies only to Function and requires a plain unit result node. A unit
result without this flag is inhabited. Conditional output requires a bool result.
Other nodes cannot carry Function modes/flags. `array` and `array_ref` remain distinct.
Unions/intersections retain order and duplicate children. This class has ordinary
object identity; it does not implement structural equality, normalization or subtyping.

## SignatureContext

```csharp
public sealed class SignatureContext
{
    public SignatureContext(int typeParameters = 0, int methodParameters = 0,
                            bool selfAllowed = false);
    public int TypeParameters { get; }
    public int MethodParameters { get; }
    public bool SelfAllowed { get; }
}
```

Both counts must be 0–256 inclusive; other values raise `ArgumentOutOfRangeException`.
Properties retain these immutable counts and Self permission. Parameter indices must
be less than the corresponding count, and Self requires `SelfAllowed`. The context
does not contain declaring type/method/contract identity and cannot establish runtime
Self conformance or cross-module generic identity.

## StructuralSignature

```csharp
public static class StructuralSignature
{
    public static (TypeExpression Root, SignatureContext Context) Read(
        ReadOnlySpan<byte> payload, bool allowReferences = false);
    public static byte[] Write(TypeExpression root, SignatureContext context,
                               bool allowReferences = false);
}
```

`Read` consumes a complete signature payload, not an envelope or PE image. `Write`
produces a new caller-owned payload array. Decoded objects are independent of the
input bytes; callers must not mutate a span during reading. Both operations validate
context, kind-specific shape, flags/modes, lengths and resource limits. Invalid or
unsupported input raises `InvalidDataException`; null writer root/context raises
`ArgumentNullException`.

Limits are 1 MiB per payload, 4,096 nodes, depth 32 with root at zero, generic arity
256 and 256 Function parameters/compound operands. Tuple requires at least one child;
union/intersection at least two. Arrays and nullable require one; primitives, generic
parameters and Self have none. The only supported calling convention is managed.
Unknown nodes, flags, modes, trailing bytes and out-of-bounds counts fail.

With `allowReferences: false`, nominal nodes are rejected (section-1 profile).
With `true`, the section-3 grammar admits a nominal index in 1–256 and up to 256
argument children. This is grammar permission, **not** a check that the reference exists,
is a type or has matching generic arity; `StructuralIdentity.Resolve` performs those
checks against an explicit catalog. Standalone byref-result syntax, full primitive support, declaration resolution,
position legality, member synthesis and execution remain outside this codec.

The library preserves canonical payload framing, so valid read/write round-trips are
byte-identical. Payload encoding does not normalize structural semantic identity.
These type constructors and the following usage are exercised by the conformance
consumer's signature commands:

```csharp
var signature = new TypeExpression("function",
    [new("int32"), new("bool")], modes: ["out_when_true"]);
var bytes = StructuralSignature.Write(signature, new SignatureContext());
var decoded = StructuralSignature.Read(bytes);
```

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet_signatures.py
```

Fourteen cross-reader vectors pass, including existing reference-profile fixtures,
Function modes and nominal syntax. Independent .NET nested emission matches Python.
Both readers reject 103 malformed vectors, including every nested-fixture truncation,
invalid binders/Self contexts, unknown nodes/modes, bad conventions/flags, malformed
lengths and depth/node limits. Writer/ownership and exact depth/arity boundaries are
also exercised. None of these tests claims that the decoded metadata can execute.


## MetadataReference and MetadataDefinition

```csharp
public readonly record struct MetadataReference(Guid Assembly, Guid Module, uint Token);
public sealed record MetadataDefinition(string Kind, int Arity = 0,
                                        MetadataReference? Owner = null);
```

Positional constructors and matching public init properties store the given values.
Records provide value equality, hashes, deconstruction, printable representations and
`with` copies. They do not validate at construction; table/resolver operations validate
before use. Default MetadataReference is invalid. Assembly and Module must be nonempty
host-assigned scope UUIDs; Token must be a TypeDef (0x02) or MethodDef (0x06) token with
a nonzero row. These scopes are **not** CLI assembly names, MVID-derived assembly
identity or a promise that a token exists in a physical PE image.

Definition Kind is `type`, `interface` or `method`, with Arity 0–256. Only methods
require Owner, a declaring-type reference. All other declarations require null Owner.
The host supplies authoritative declarations, including referenced dependencies, and
must use compatible scope assignments across catalogs when comparing resolved keys.

## ReferenceBindings

```csharp
public sealed class ReferenceBindings
{
    public ReferenceBindings(IReadOnlyList<MetadataReference> references,
        int typeOwner = 0, int methodOwner = 0, int selfOwner = 0);
    public IReadOnlyList<MetadataReference> References { get; }
    public int TypeOwner { get; }
    public int MethodOwner { get; }
    public int SelfOwner { get; }
}
```

Copies the list into owned read-only storage. Null references raises
ArgumentNullException; more than 256 entries raises ArgumentException. Owners are
one-based local indices, with zero meaning absent. Further validation occurs in
ReferenceTable and StructuralIdentity. TypeOwner and SelfOwner must name TypeDefs;
MethodOwner must name a MethodDef. References must be unique.

## ReferenceTable

```csharp
public static class ReferenceTable
{
    public static ReferenceBindings Read(ReadOnlySpan<byte> payload);
    public static byte[] Write(ReferenceBindings bindings);
}
```

Reads/writes section 2, schema 1: four little-endian u16 values for count/type owner/
method owner/Self owner, followed by 36-byte reference rows. UUID bytes use network
order, tokens little-endian order. At most 256 rows are allowed; input must have the
exact expected length. Invalid GUIDs, token kinds/rows, duplicates, owner kinds/indices,
truncation or trailing bytes raise InvalidDataException. Null writer bindings raises
ArgumentNullException. Read owns its data; Write returns a fresh caller-owned array.
Neither resolves declarations or checks that nominal uses exist in the table.

## StructuralIdentity

```csharp
public static class StructuralIdentity
{
    public static ResolvedTypeIdentity Resolve(TypeExpression root,
        SignatureContext context, ReferenceBindings bindings,
        IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog);
}
```

Validates bounded signature syntax, reference-table rules, catalog kinds and arities,
nominal generic arguments, and declaring owners before returning an identity. Null
arguments raise ArgumentNullException; malformed or unresolved data raises
InvalidDataException. Do not mutate the catalog during resolution. Every reference
must resolve, including unused entries. A nominal use must point to a type/interface
with the exact number of arguments. Generic binders require matching declaring-owner
arities; a bound method's declaring type must match TypeOwner. Self permission must
match the presence of SelfOwner, which must identify a nongeneric interface contract.

Generic parameters are keyed by declaring reference and ordinal; Self by contract.
Local indices and table/directory order do not enter resolved identity. Unions and
intersections flatten the same operator, discard duplicate operands and ignore order.
A deduplicated singleton retains its operator wrapper. Tuples/arguments retain order;
Function modes and no-result, and Array versus ArrayRef, remain distinct. This does
not implement assignability, subtype reduction, distribution or null equivalences.

Callers select and validate the envelope profile: section 2/schema 1 plus section
3/schema 1 are required, and section 1 must not be mixed in. These payload-level APIs
do not implement a production profile loader or PE recognition. They cannot establish
that an assembly is safe to execute or populate Raven/neoCLR objects themselves.

## ResolvedTypeIdentity

```csharp
public sealed class ResolvedTypeIdentity : IEquatable<ResolvedTypeIdentity>
{
    public bool Equals(ResolvedTypeIdentity? other);
    public override bool Equals(object? obj);
    public override int GetHashCode();
}
```

Created only by Resolve; no public constructor or mutable state. Equals compares full
resolved keys, returns false for null/other object types, and is consistent with the
process-local hash. Use Equals or a normal equality-based collection; `==` is not
overloaded. Neither hash codes nor private canonical bytes are persistence formats.

The separate conformance consumer exercises these APIs with Python-generated tables,
signatures and explicit catalogs:

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet_references.py
```

All 95 shared vectors pass: 22 equality/distinction cases, four table round-trips and
69 rejections. Independent C# golden UUID emission/reading, writer validation and list
ownership checks also pass. The test-only JSON transport is not a library API.


## StructuralMemberReference

```csharp
public sealed record StructuralMemberReference(string Operation, int Owner = 1,
                                               int Element = 0);
```

The constructor and matching init properties retain values without validation.
The record supplies value equality, hashes, deconstruction, printable representation
and `with` copies. StructuralMembers validates before reading/writing/resolving:
Operation must be `array_length`, `tuple_element`, `tuple_deconstruct` or
`function_invoke`; Owner must be 1 (the section-3 root); Element is 0–255 and must be
zero except for a zero-based tuple element. A reference is not a MethodDef token.

## ResolvedMemberIdentity

```csharp
public sealed record ResolvedMemberIdentity(ResolvedTypeIdentity Owner,
                                            string Operation, int Element);
```

A schema-1 identity comprising a resolved owner, operation and ordinal. Positional
constructor/init properties and generated record equality/hash/deconstruction/`with`
behave as value operations. The resolver returns validated identities; constructing
one directly performs no validation and grants no capability. Equality compares full
resolved owner identity, so local reference numbering is irrelevant but declaration
scopes, array storage, Function modes/no-result and tuple ordinals remain significant.
Hashes are process-local and neither identity nor its string form is a persistence API.

## StructuralMemberDescriptor

```csharp
public sealed class StructuralMemberDescriptor
{
    public ResolvedMemberIdentity Identity { get; }
    public IReadOnlyList<ResolvedTypeIdentity> Parameters { get; }
    public IReadOnlyList<string> Modes { get; }
    public ResolvedTypeIdentity Result { get; }
    public bool NoResult { get; }
}
```

Created only by StructuralMembers.Resolve, with copied read-only parameter/mode lists.
Parameters exclude the receiver; modes correspond one-to-one in order. Result is unit
when NoResult is true; a unit result with NoResult false is distinct. Descriptor object
equality is ordinary reference equality; compare Identity for member identity and
properties for contract content. No method body, dispatch target or invocation API exists.

## StructuralMembers

```csharp
public static class StructuralMembers
{
    public static ResolvedTypeIdentity NativeUnsignedResult { get; }
    public static IReadOnlyList<StructuralMemberReference> Read(ReadOnlySpan<byte> payload);
    public static byte[] Write(IReadOnlyList<StructuralMemberReference> members);
    public static IReadOnlyList<StructuralMemberDescriptor> Resolve(
        IReadOnlyList<StructuralMemberReference> members, TypeExpression root,
        SignatureContext context, ReferenceBindings bindings,
        IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog);
}
```

Read/Write encode section 4, schema 1: little-endian u16 count followed by six-byte
rows (u16 owner, u8 operation, u8 zero flags, u16 operand). Operation codes 1–4 follow
the order listed above. At most 256 unique rows are permitted, including an empty table.
Read returns owned read-only rows; Write returns a fresh byte array. Table operations
validate counts, exact lengths, operations, flags, owners, operands and duplicates,
but do not validate owner shapes. Invalid data or null rows raises InvalidDataException;
a null writer list raises ArgumentNullException.

Resolve validates the entire owner using StructuralIdentity.Resolve, even for an empty
member list, then checks each operation's owner shape and derives these contracts:

| Operation | Owner | Parameters/modes | Result |
| --- | --- | --- | --- |
| array_length | Array or ArrayRef | None | NativeUnsignedResult; NoResult false |
| tuple_element | Tuple; ordinal within arity | None | Selected element; NoResult false |
| tuple_deconstruct | Tuple | All elements, each `out` | Unit; NoResult true |
| function_invoke | Function | Declared parameters and modes | Declared result and NoResult |

NativeUnsignedResult is an intrinsic descriptor identity for native unsigned/UIntPtr,
matching the experimental runtime ArrayLength contract. It adds no serializable
signature opcode and does not claim ordinary .NET Array.Length's Int32 contract.
Compare result identities with Equals. Resolve returns owned read-only descriptors in
input order. Null arguments raise ArgumentNullException; invalid tables, unresolved
catalog declarations, incompatible shapes or out-of-range tuple ordinals raise
InvalidDataException. Callers must not mutate input lists/catalogs during operations.

Envelope composition remains the caller's responsibility: this member profile needs
mandatory schema-1 sections 2, 3 and 4, with no local section-1 signature. These APIs do
not perform artifact recognition, PE loading, assembly emission or runtime invocation.

The compiled consumer validates all public operations with 49 Python/.NET shared cases:
14 contract vectors, five identity comparisons and 30 rejections. Coverage includes
256 tuple elements/deconstruction outputs, empty tables, all Function modes, reference
renumbering and malformed framing. Independent C# golden emission, writer rejection
and reader ownership checks also pass.

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet_members.py
```


## MetadataProfile

```csharp
public static class MetadataProfile
{
    public static MetadataProfileDocument Read(ReadOnlySpan<byte> image);
    public static MetadataProfileDocument Create(TypeExpression root,
        SignatureContext context, ReferenceBindings bindings,
        IReadOnlyList<StructuralMemberReference>? members = null,
        IReadOnlyList<MetadataSection>? optionalSections = null);
    public static byte[] Write(MetadataProfileDocument document);
}
```

These are the typed entry points for the **reference profile**, not a general CLI
assembly API. Read requires mandatory schema-1 sections 2 and 3. Section 1 is rejected
in any version, including when optional. A schema-1 section 4 must be mandatory when
present. Unknown required kinds/versions fail; unknown optional payloads are retained
opaquely, including future optional section-4 versions. Unsupported optional versions
of sections 2 or 3 cannot satisfy the mandatory reference-profile requirements.
A local-only, empty or incomplete envelope fails instead of producing a partial view.

Read performs envelope validation, signature/table decoding, local nominal index/kind
checks, generic/Self owner presence checks, and member shape/ordinal validation. It
owns the resulting data independently of the input buffer. It does not consult a
catalog: nominal arity, declaring method ownership and Self interface requirements
are checked later by the document's Resolve methods. Do not mutate input during a call.
Malformed or unsupported input raises InvalidDataException.

Create accepts typed syntax, explicit local bindings and optional members. It copies
through the codecs and performs the same local validation as Read. Null root/context/
bindings raises ArgumentNullException. Null members omits section 4; an empty list
emits a supported mandatory empty table. OptionalSections accepts only non-null,
optional sections with kinds greater than 4. Duplicate kinds, required extensions,
reserved kinds 1–4, invalid shapes/references or exceeded bounds raise
InvalidDataException. Output order is 2, 3, optional 4, then supplied extensions.
All existing envelope/signature/table resource bounds apply.

Write accepts only a document returned by Read/Create and returns a fresh array.
Null document raises ArgumentNullException. Read/Write preserves original order and
all bytes, including opaque optional sections. Semantic editing is explicit: construct
a new document and supply consistently remapped indices. Create does not copy unknown
extensions automatically or claim their meaning survives changes. In particular, an
unknown future optional section-4 payload can be round-tripped but cannot be attached
to a newly constructed schema-1 document through optionalSections.

## MetadataProfileDocument

```csharp
public sealed class MetadataProfileDocument
{
    public IReadOnlyList<MetadataSection> Sections { get; }
    public TypeExpression Root { get; }
    public SignatureContext Context { get; }
    public ReferenceBindings Bindings { get; }
    public IReadOnlyList<StructuralMemberReference> Members { get; }
    public bool HasMemberTable { get; }
    public IReadOnlyList<MetadataSection> UnknownOptionalSections { get; }
    public ResolvedTypeIdentity ResolveType(
        IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog);
    public IReadOnlyList<StructuralMemberDescriptor> ResolveMembers(
        IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog);
}
```

No public constructor or setters. Sections contains all owned immutable sections;
Root/Context/Bindings expose decoded syntax and owners. Members contains supported
schema-1 rows. HasMemberTable distinguishes an absent/unsupported table from a present
empty schema-1 table. UnknownOptionalSections exposes unrecognized sections without
claiming to understand them; callers must inspect it when their own workflow needs
additional semantics.

ResolveType returns full catalog-scoped identity. ResolveMembers validates the owner
even when Members is empty, then derives supported contracts in table order. Both
require a non-null authoritative catalog (ArgumentNullException otherwise); unresolved
or incompatible declarations raise InvalidDataException. These methods do not cache
or retain the catalog; do not mutate it during a call. No assembly loading or runtime
object construction occurs.

This independently emitted example is compiled by the conformance consumer and matches
the shared Python tuple-member fixture:

```csharp
var document = MetadataProfile.Create(
    new TypeExpression("tuple", [new("int32"), new("string")]),
    new SignatureContext(), new ReferenceBindings([]),
    [new("tuple_element", Element: 1), new("tuple_deconstruct")]);
var bytes = MetadataProfile.Write(document);
var imported = MetadataProfile.Read(bytes);
var contracts = imported.ResolveMembers(
    new Dictionary<MetadataReference, MetadataDefinition>());
```

The focused consumer covers 36 shared profile vectors (29 rejections), independent
creation/emission, owned data, optional preservation and explicit resolution failure.

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet_profiles.py
```

The future Raven adapter and potential Raven implementation for Metadata Introspection
are still planned. This one-root reference profile is not yet sufficient to represent
complete assemblies, their declarations, IL bodies or all compiler signatures.


## MetadataArtifactReader

```csharp
public static class MetadataArtifactReader
{
    public const int MaxImageSize = 4 * 1024 * 1024;
    public static MetadataArtifact Read(ReadOnlySpan<byte> image,
                                        bool expectedExtended = true);
}
```

Read consumes complete PE bytes without loading or executing the assembly. The default
requires a recognized extension: it rejects ordinary input and input with both markers
removed. With expectedExtended false, input having neither a `neoCLR.` version marker
nor a #Neo stream returns an ordinary classification. An unmarked #Neo stream, a marked
image missing #Neo, unknown marker versions or inconsistent bindings always fail,
regardless of that option. Automatic classification cannot recover erased provenance.

Supported input is deliberately restricted to the Python fixture transport contract:
at most 4 MiB, 1–16 nonoverlapping sections, PE32 with a 224-byte optional header and
16 directories, supported power-of-two file/section alignment (512–65536 for file
alignment), no overlay, no certificate directory or checksum, and a 72-byte CLI header
with flags exactly IL-only and no strong-name/native-header directory. PE32+, signed
and other unsupported layouts fail. This is not general-purpose PE validation.

The metadata root requires BSJB, an aligned version field of at most 256 bytes and at
most 16 streams. Stream names must be nonempty, unique, ASCII, null-terminated within
32 bytes and zero-padded to four bytes. Stream ranges must be aligned, file-backed,
outside the directory and nonoverlapping. All length/range arithmetic is checked
before accessing input; oversized and truncated values raise InvalidDataException.

The recognized marker is `neoCLR.NEOX.0.1;sha256=` plus 64 lowercase hexadecimal
characters and zero termination/padding. SHA-256 binds the domain separator and all
exact stream names/data in ordinal name order, including #Neo padding. #Neo's declared
envelope length must leave at most three zero padding bytes. The extracted envelope
must then pass MetadataProfile.Read, so a valid digest cannot bypass required schemas
or reference-profile validation. Local-only section-1 artifacts are not supported by
this higher-level reader.

All malformed, inconsistent or unsupported inputs raise InvalidDataException. Read
copies data needed by its result; callers must not mutate the span during the call.
The result does not depend on the input buffer or a retained stream lifetime. Only
span input is currently exposed; path/stream options and typed diagnostics remain
future API work.

## MetadataArtifact

```csharp
public sealed class MetadataArtifact
{
    public bool IsExtended { get; }
    public MetadataProfileDocument? Profile { get; }
}
```

No public constructor or mutable state. IsExtended is true exactly when Profile is
non-null: both recognition and local reference-profile validation passed. Ordinary
classification has IsExtended false and Profile null. It does not expose ordinary
CLI declarations or claim that conventional tables/signatures/bodies are valid.

The digest is a consistency check, **not authentication, IL verification or an
unaware-runtime execution guard**. A writer can recompute it around changed conventional
metadata; the reader deliberately does not claim to resolve or verify those tables.
Future compiler use still requires physical CLI binding and declaration validation.
The reader is a lower layer beneath the proposed Cecil-inspired assembly object model.

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet_artifacts.py
```

The compiled consumer passes 50 shared Python/.NET cases (44 rejections), including
Cecil's extension-stripping rewrite, marker erasure, rebinding, reordered streams,
malformed ranges/names, wrong profiles, image-size limits and result ownership.


## Model namespace

Assembly: `NeoCLR.Metadata.Experimental`; namespace:
`NeoCLR.Metadata.Experimental.Model`. This first Cecil-inspired object-model slice is
**read-only and experimental**. It uses the existing artifact recognition layer and
System.Reflection.Metadata internally to read actual declarations, with no Cecil
package dependency. It does not load runtime assemblies. These names are distinct
from System.Reflection.Metadata and Mono.Cecil types; qualify or alias when needed.

### AssemblyDefinition

```csharp
public sealed class AssemblyDefinition
{
    public string Name { get; }
    public Version Version { get; }
    public AssemblyIdentity Identity { get; }
    public ModuleDefinition MainModule { get; }
    public MetadataProfileDocument? Profile { get; }
    public uint EntryPointToken { get; }
    public MethodDefinition? EntryPoint { get; }
    public byte[] Write();
    public static AssemblyDefinition ReadAssembly(ReadOnlySpan<byte> image,
                                                  bool expectedExtended = true);
}
```

No public constructor or setters. Name is the simple assembly name and Version is the
manifest version. Identity adds culture, normalized public-key token and retained
flags for exact explicit dependency matching (see below). MVID is not an assembly
identity. CLR binding redirects, unification and trust policy are not implemented.
MainModule is the owned manifest module; Profile is attached structural metadata, or
null for explicitly admitted ordinary CLI input. EntryPointToken is a managed MethodDef
token or zero for a library; nonzero tokens are checked against the method table.
EntryPoint returns that same owned MethodDefinition instance, or null for a library.
Write returns a fresh byte-for-byte copy of the original immutable snapshot, including
opaque data. It does not rebuild declarations or apply edits. Use AssemblyBuilder for
new controlled output.

ReadAssembly first applies MetadataArtifactReader's existing 4 MiB, unsigned IL-only
PE32, stream and required-profile rules. The default requires extended metadata;
expectedExtended false permits ordinary CLI assemblies. The method then reads the
assembly and module rows, TypeDef names/namespaces, generic parameter counts and
NestedClass ownership, AssemblyRef identities, and MethodDef names, ownership, flags,
generic arities and owned signature blobs. It accepts at most 256 AssemblyRefs,
a cumulative 4 MiB of key/token blobs, and at most 4096 TypeDefs and a cumulative 4 Mi UTF-16
code units of decoded declaration names/namespaces (counting repeated uses). It rejects missing/invalid
or cyclic declaring-type relationships. MethodDefs are limited to 4096 and copied
signature bytes to a cumulative 4 MiB shared with MemberRef blobs (including repeated
references to one blob). MemberRefs are limited to 4096, require nonempty signature
blobs, and have their parent table kind and row bounds checked.
Missing/empty method signatures and instance global functions are rejected. Nonempty
signatures outside the supported decoder remain opaque and are not generally validated.
Missing assembly manifests (netmodules),
invalid/unsupported artifacts and malformed inspected metadata raise
InvalidDataException; underlying BadImageFormatException is retained as InnerException
when converted. Callers must not mutate the span during reading.

The returned graph owns its strings/values and profile. Readers/streams are disposed
before return; no assembly loading, dependency discovery or open resource remains.
Only the inspected declaration subset is validated: this is not full signature, IL,
attribute, constraint or execution verification. MetadataProfile's catalog scope UUIDs
are not automatically derived from assembly names or MVIDs.

### ModuleDefinition

```csharp
public sealed class ModuleDefinition
{
    public AssemblyDefinition Assembly { get; }
    public string Name { get; }
    public Guid Mvid { get; }
    public IReadOnlyList<TypeDefinition> Types { get; }
    public IReadOnlyList<MethodDefinition> Methods { get; }
    public IReadOnlyList<MethodDefinition> Functions { get; }
    public MethodDefinition? GetMethodDefinition(uint metadataToken);
    public IReadOnlyList<MemberReference> MemberReferences { get; }
    public MemberReference? GetMemberReference(uint metadataToken);
    public IReadOnlyList<AssemblyReference> AssemblyReferences { get; }
    public IReadOnlyList<TypeReference> TypeReferences { get; }
    public TypeDefinition? GetTypeDefinition(uint metadataToken);
}
```

No public constructor. Assembly points back to the owning snapshot. Name/Mvid are the
stored module values; Mvid is not an experimental catalog module scope. Types is an
owned read-only collection of every TypeDef in row order, **including nested types
and `<Module>`**. This intentionally exposes a flat table view in the initial slice;
it does not promise Cecil's exact collection organization. AssemblyReferences is an
owned read-only list of physical AssemblyRef rows in metadata order; reading it performs
no resolver calls.

Methods contains all physical MethodDef declarations in row order. Functions contains
those physically owned by the first, top-level, empty-namespace `<Module>` row. In the
model these have null DeclaringType and the pseudo-type's Methods collection is empty.
Other definitions retain their actual type owner. GetMethodDefinition returns the same
snapshot object as these collections, or null for absent and non-MethodDef tokens.
No overload selection or dependency lookup is implied. MemberReferences contains all
physical MemberRefs in row order, including field/general signatures that this model
cannot yet resolve. GetMemberReference returns the owned row or null for missing/wrong
kinds; neither operation resolves a dependency.

GetTypeDefinition performs local physical TypeDef-token lookup and returns the same
owned definition instance. Zero, other token kinds and absent rows return null. It
does not follow exports or dependencies; TypeReferences exposes physical TypeRef rows
for explicit resolution. At most 4096 TypeRefs and 32 enclosing TypeRef scopes are
admitted; missing/cyclic parent and out-of-range AssemblyRef scopes are rejected.

### TypeDefinition

```csharp
public sealed class TypeDefinition
{
    public ModuleDefinition Module { get; }
    public uint MetadataToken { get; }
    public string Namespace { get; }
    public string Name { get; }
    public int GenericArity { get; }
    public TypeDefinition? DeclaringType { get; }
    public IReadOnlyList<MethodDefinition> Methods { get; }
    public TypeReference ToReference();
}
```

No public constructor. Module is the owning snapshot and MetadataToken its physical
TypeDef address. Namespace/Name preserve metadata strings, including any backtick arity
suffix; display names are not identity. GenericArity counts owned GenericParam rows,
including captured outer parameters if encoded; it does not expose or validate the
full generic constraint contract. DeclaringType links to the same graph's enclosing
definition, or null for top-level types. ToReference creates a new definition-backed
reference that resolves to this exact object. Methods exposes directly declared
callables. Fields, custom attributes, base types and interfaces remain outside this slice.

### MethodDefinition

```csharp
public sealed class MethodDefinition
{
    public ModuleDefinition Module { get; }
    public TypeDefinition? DeclaringType { get; }
    public uint MetadataToken { get; }
    public string Name { get; }
    public ushort Attributes { get; }
    public ushort ImplementationAttributes { get; }
    public int GenericArity { get; }
    public bool IsStatic { get; }
    public byte[] GetSignature();
    public bool TryGetStaticInt32Signature(out int parameterCount, out bool returnsValue);
    public bool TryGetStaticPrimitiveSignature(out PrimitiveMethodSignature? decoded);
    public bool TryGetStaticValueSignature(out MethodSignature? decoded);
}
```

No public constructor or mutation. Module and DeclaringType refer to the same owned
snapshot; null DeclaringType means a global function. TypeDefinition.Methods contains
only its directly declared methods, not inherited or nested-type methods. MetadataToken
is a physical MethodDef token, not cross-module identity. Name is the stored name;
Attributes/ImplementationAttributes retain CLI flag bits and IsStatic tests bit 0x10.
GenericArity counts method GenericParam rows without interpreting their constraints.

GetSignature returns a fresh copy of the CLI signature blob. It preserves unsupported
encodings and never simplifies them into a supported signature. Returned bytes and
original input bytes can be modified without changing the snapshot. No body, parameter
names, custom attributes, constraints or general signature type resolution is provided
by MethodDefinition. Use MemberReference for the supported reference-resolution subset.

TryGetStaticInt32Signature recognizes the original Int32-only writer contract: static,
nongeneric, default calling convention, 0–256 Int32 parameters, and Int32 or absent
CLI void result. Success sets parameterCount and returnsValue; false resets them to
zero/false. Instance/generic/vararg headers, other types, noncanonical count encodings,
truncation and trailing data return false. False means the narrow decoder cannot accept
the signature; it does not distinguish a valid unsupported signature from malformed
opaque data. ReadAssembly is not a general signature or execution verifier.

The C# consumer reads an emitted entry point, globals and overloaded type-owned methods,
checks shared object identity and copy isolation, and covers counts 0/1/127/128/256,
no-result returns, unsupported signatures, 4096/4097 rows and repeated-blob amplification.

### MemberReference

```csharp
public sealed class MemberReference
{
    public ModuleDefinition Module { get; }
    public uint MetadataToken { get; }
    public uint ParentToken { get; }
    public string Name { get; }
    public byte[] GetSignature();
    public MethodDefinition ResolveMethod(IAssemblyResolver? resolver = null);
}
```

The snapshot owns these rows; there is no public constructor or mutation. Module is
the consuming module. MetadataToken is its physical MemberRef token; ParentToken is
the physical MemberRefParent token, and Name is the referenced name. GetSignature
returns a fresh copy, including opaque field or unsupported method signatures.

ResolveMethod supports the writer's static, nongeneric default-convention Int32/Int64/Boolean/String
scalar/vector parameters (0–256) and scalar/vector/no-result contract. It resolves a local TypeDef or nominal
TypeRef parent, requiring the explicit resolver for external scopes. It then selects
exactly one directly declared method by ordinal name and decoded parameter/result
contract, returning that target snapshot's owned MethodDefinition. There is no implicit
filesystem probing, binding cache or access-policy decision. Resolver errors propagate;
missing/wrong-identity dependencies, unsupported contracts/parents, and absent/ambiguous
matches raise InvalidDataException. A return-contract mismatch cannot select a method.

Field, instance/constrained-generic/vararg and nominal signature types remain opaque. ModuleRef,
TypeSpec and MethodDef parents are range-checked on read but unsupported for resolution.
Global MemberRefs and inherited method lookup are also unsupported. Local global calls
emitted by the writer use MethodDef tokens and remain available through method lookup.
Resolution never compares module-relative nominal signature tokens as cross-module
identity; general type-aware signature comparison remains future work.

C# tests resolve writer-emitted cross-assembly overloads and no-result methods, and
exercise local TypeDef/TypeRef parents, copy isolation, missing/mismatched dependencies,
return mismatches, ambiguity, host failures, unsupported signatures, invalid parents,
4096/4097 rows and the shared decoded-signature budget.

### TypeReference

```csharp
public sealed class TypeReference
{
    public ModuleDefinition Module { get; }
    public uint MetadataToken { get; }
    public string Namespace { get; }
    public string Name { get; }
    public uint ResolutionScopeToken { get; }
    public TypeDefinition Resolve(IAssemblyResolver? resolver = null);
}
```

Created from TypeDefinition.ToReference or a physical TypeRef row. Module is the target
module for a definition-backed reference and the consuming module for a physical row.
MetadataToken identifies that TypeDef/TypeRef in Module; Name/Namespace preserve stored
metadata strings. ResolutionScopeToken is the physical scope, or zero for a
definition-backed reference/nil scope.

Resolve returns the same owned TypeDefinition for a definition-backed reference. For
physical references it supports local Module scope, exact AssemblyRef dependencies
through the supplied resolver, and nested TypeRef scopes. Names/namespaces and nesting
must identify exactly one target definition; missing or ambiguous targets, missing
resolvers, nil/multi-module scopes or mismatched dependencies raise InvalidDataException.
Host resolver exceptions propagate. Exported-type forwarders and TypeSpec/generic
instantiation resolution are not supported. No implicit IO or assembly loading occurs.

All read-model classes use ordinary reference equality. Independent reads yield
distinct graphs even when tokens/MVIDs match. Structural identity remains separate.

### AssemblyIdentity

```csharp
public sealed class AssemblyIdentity : IEquatable<AssemblyIdentity>
{
    public AssemblyIdentity(string name, Version version, string culture = "",
                            string publicKeyToken = "", uint flags = 0);
    public string Name { get; }
    public Version Version { get; }
    public string Culture { get; }
    public string PublicKeyToken { get; }
    public uint Flags { get; }
    public bool Equals(AssemblyIdentity? other);
    public override bool Equals(object? obj);
    public override int GetHashCode();
}
```

Immutable exact metadata-matching identity. Name is nonempty; Version has four
components in 0–65535; Culture is the stored string (empty for neutral). PublicKeyToken
is empty or 16 hexadecimal characters, normalized to lowercase. Flags retains metadata
flags except PublicKey (bit 1), which expresses full-key versus token representation
rather than identity. Null constructor inputs raise ArgumentNullException; invalid
values or the PublicKey representation bit raise ArgumentException.

Equals uses ordinal name/culture comparison, exact version/flags and normalized token.
Null and other object types are unequal. Hashes are process-local and consistent with
Equals; `==` is not overloaded. This intentionally conservative policy does not implement
CLR case folding, redirects, version roll-forward or retargeting. Nonmatching flags,
including retargetable flags, fail exact resolution until a deliberate policy exists.

ReadAssembly converts a full public-key blob to the reversed final eight bytes of its
SHA-1 hash, yielding the conventional key token. Assembly definitions require a full
key with PublicKey set or an empty key with it clear; references permit a full nonempty
key with the flag, or an empty/eight-byte token without it. Invalid representations
raise InvalidDataException. Cryptographic key structure and signatures are not verified;
token matching is identity comparison, not authenticity or execution permission.
Original full keys, hash blobs and a complete writer representation are not exposed yet.

### AssemblyReference

```csharp
public sealed class AssemblyReference
{
    public ModuleDefinition Module { get; }
    public uint MetadataToken { get; }
    public AssemblyIdentity Identity { get; }
    public AssemblyDefinition Resolve(IAssemblyResolver resolver);
}
```

No public constructor. Module is the **consuming** module, MetadataToken its physical
AssemblyRef token (0x23), and Identity the requested dependency identity. Resolve calls
the supplied resolver, rejects null/missing or mismatched candidates, and returns the
matching snapshot unchanged. Null resolver raises ArgumentNullException; missing or
wrong identity raises InvalidDataException. Host resolver exceptions propagate without
being disguised as malformed metadata. Calls are not cached; dependency search, IO,
cache ownership and lifetime are explicit host policy. No runtime assembly is loaded by
the reference itself. Nominal TypeRef lookup uses this resolver; constructed TypeSpec/forwarder binding remains pending.

### IAssemblyResolver

```csharp
public interface IAssemblyResolver
{
    AssemblyDefinition? Resolve(AssemblyIdentity identity);
}
```

The host receives the requested identity and returns a candidate metadata snapshot or
null. It may choose how to obtain that candidate; AssemblyReference always rechecks
exact identity. Implementations own their concurrency, cache and resource policy.
Returning an arbitrary same-name assembly cannot bypass version/culture/key/flag checks.
The C# tests include both a candidate resolver and a throwing resolver.

Run the standalone C# contract suite directly (no Python or test packages required):

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests/NeoCLR.Metadata.Experimental.Tests.csproj --no-launch-profile
```

The original 11 identity/resolution groups (now part of a 14-group suite) pass: physical AssemblyRef ownership, explicit resolution and repeat calls,
missing/mismatched candidates, exact identity/hash rules, ECMA full-key/token golden
normalization, constructor and wire validation, 256/257-reference boundary, owned data
and exception propagation. Fixtures are real PE metadata images built in C# with
System.Reflection.Metadata. This executable returns nonzero on any failure; it is not
a `dotnet test` discovery project.


## Controlled PE and native assembly construction

Namespace: `NeoCLR.Metadata.Experimental.Model`. These builders are the first writer
part of the primary compiler abstraction. They construct new assemblies and allow body
editing before another write; they do **not** rewrite arbitrary read snapshots or claim
full Cecil compatibility. The current executable subset is top-level functions, public
static classes and methods with Int32/Int64/Boolean/String parameters and Int32/Int64/Boolean/String or CLI no-result return. This is a
compiler integration proof, not a complete language backend.

### AssemblyBuilder

```csharp
public sealed class AssemblyBuilder
{
    public AssemblyBuilder(AssemblyIdentity identity, AssemblyIdentity coreLibrary);
    public AssemblyIdentity Identity { get; }
    public AssemblyIdentity CoreLibrary { get; }
    public IReadOnlyList<TypeBuilder> Types { get; }
    public IReadOnlyList<MethodBuilder> Functions { get; }
    public MethodBuilder AddFunction(string name, int parameterCount = 0,
                                     bool returnsValue = true);
    public MethodBuilder? EntryPoint { get; set; }
    public TypeBuilder AddType(string @namespace, string name);
    public TypeBuilder AddType(string @namespace, string name, TypeVisibility visibility);
    public byte[] Write();
    public byte[] WriteNativeAssembly();
}
```

The constructor requires explicit output and core-library identities. Null raises
ArgumentNullException; output identities with a key token or flags raise
ArgumentException because signing/flagged output is unsupported. No host core library
is inferred. The core supplies the System.Object base reference. Types is a read-only
view of this mutable graph. Functions is its separate read-only view of top-level
functions. Do not mutate participating graphs during either write operation.

AddType adds a public abstract sealed class with a unique namespace/name pair. Namespace
may be empty; name must be nonempty and not `<Module>`. Combined length is at most 1024
characters and the assembly admits at most 4,095 declared types (plus the CLI module row). Invalid/duplicate inputs raise
ArgumentException. EntryPoint may be null for a library or a local static
Int32-returning or no-result method with no parameters or one String vector; it is checked at Write/WriteNativeAssembly.
A no-result entry uses CLI void for ordinary PE output and native `Void` with
`no_result: true` for format 5. Reference-only projections still have no CLI entry.
The native declaration reader accepts both supported results and the optional String
vector. Other parameter shapes, generic/foreign entries and ambiguous native entry names
remain invalid, and no-result bodies must return with an empty stack.
`EntryPointChecks.cs` verifies both global and type-owned entries, CLI invocation,
native container roundtrips and failures; Raven's consumer also verifies native zero exit.

AddFunction creates an assembly-owned function with no declaring type. Names must be
nonempty and at most 1024 characters, parameter counts 0–256, and an assembly admits
at most 256 top-level functions. Duplicate name/parameter-count pairs raise
ArgumentException. Function and type-method name scopes are independent.

Write validates every local body, assigns physical tokens/RVAs, imports foreign call
references, and emits an owned unsigned IL-only PE32 image. Bounds: 4096 total methods,
131072 total instructions, 256 imported assembly identities and 4 MiB output. Foreign
calls require the same core identity; an external dependency with the same identity as
the output is rejected. Invalid bodies, entry points, incompatible imports or exceeded
limits raise InvalidDataException. Call targets are typed builder methods; output
references include exact assembly identity, nominal type name and method signature.
Top-level functions are emitted as CLI global methods on the physical `<Module>` row;
that row is a transport detail, not a declaring TypeBuilder. Local calls and global
entry points are supported. Cross-assembly top-level calls currently raise
InvalidDataException in PE emission; use native emission for that case.

Unchanged repeated writes of the same graph are byte-identical. Each builder has a
fresh MVID, stable over its edits; equivalent independently constructed graphs need not
have identical bytes. PE content IDs/timestamps are derived deterministically.
No strong-name signing, resource/debug data, #Neo attachment or conventional-image
rewriting is implemented by Write. The writer cannot silently discard such data because
it accepts only its explicitly constructed subset, not an arbitrary loaded image.

WriteNativeAssembly validates the same local graph and emits owned UTF-8 bytes in
neoCLR's existing JSON assembly format 5. Save as `.neo.json` and pass dependencies
explicitly with `--module`; no PE importer, subprocess or runtime loading occurs inside
the writer. Native top-level functions have no type owner. Type-owned methods retain
an explicit owner. Both local and cross-assembly calls are supported; no-result returns
remain distinct from inhabited Void.

Native output contains assembly/module/type/method origin metadata, scoped tokens,
parameter-row absence (zero tokens), and exact dependency revisions. The transport
module name uses SHA-256 of a canonical JSON identity tuple; member names encode UTF-8
bytes with ownership separators. This is deterministic binding, not authentication.
The descriptive assembly identity retains name/version/culture/token/flags in that
canonical tuple. Different dependency builder objects sharing one identity are rejected.
Names used as native descriptions must be nonblank, free of control characters and
well-formed Unicode; failures raise InvalidDataException. Output is bounded to 4 MiB.
Native emission carries no MVID (format 5 has no such field), and does not attach NEOX
structural sections or promise arbitrary CLR/PE compatibility. The explicit core identity
is checked between builder call targets, but no core assembly is loaded for this
primitive-only subset. This format-specific backend is provisional and will evolve
with the native metadata format.

### TypeVisibility (development 2026-10-01)

`public enum TypeVisibility { Public, Internal }` describes top-level static type
visibility. `AssemblyBuilder.AddType(namespace, name)` remains public by default;
its three-argument overload accepts only these two values and throws
`ArgumentOutOfRangeException` for other values, before adding a type. Names,
uniqueness and the 4,095-type bound retain the existing contract. `TypeBuilder.Visibility`
is a read-only `TypeVisibility` property set at creation. Methods remain public static.

CLI output uses standard Public/NotPublic TypeDef flags. Native output retains the
existing runtime `visibility: "internal"` and `origin.publicly_visible: false` for
internal types; public output retains its omitted visibility default. The bounded
native reader accepts old public artifacts and preserves visibility in its reference
projection; unknown visibility or inconsistent origin flags raise `InvalidDataException`.
Internal helpers can be called within the assembly but are inaccessible to external
Raven consumers through the projection. No nested visibility, friend assemblies or
nonpublic methods are added. This host API is separate from the guest API reference.

### TypeBuilder

```csharp
public sealed class TypeBuilder
{
    public AssemblyBuilder Assembly { get; }
    public string Namespace { get; }
    public string Name { get; }
    public TypeVisibility Visibility { get; }
    public IReadOnlyList<MethodBuilder> Methods { get; }
    public MethodBuilder AddMethod(string name, int parameterCount = 0,
                                   bool returnsValue = true);
}
```

Created only by AddType. Methods is a read-only view of owned methods in declaration
order. AddMethod adds a public static hide-by-signature method. The legacy overload uses Int32 parameters; returnsValue selects Int32 or CLI void/no-result.
The signature overload preserves Int32/Int64/Boolean/String parameter and result types. Names must be nonempty and at
most 1024 characters; parameter counts are 0–256; a type admits at most 256 methods.
Duplicate name/parameter-count pairs and invalid inputs raise ArgumentException.
Generic methods, fields, instance receivers and signature variants are future work.

### MethodBuilder

```csharp
public sealed class MethodBuilder
{
    public AssemblyBuilder Assembly { get; }
    public TypeBuilder? DeclaringType { get; }
    public string Name { get; }
    public MethodVisibility Visibility { get; }
    public int ParameterCount { get; }
    public bool ReturnsValue { get; }
    public void LoadConstant(int value);
    public void WriteConsoleLine(string text);
    public void LoadArgument(int index);
    public void Add();
    public void Subtract();
    public void Multiply();
    public void Divide();
    public void Remainder();
    public void BitwiseAnd();
    public void BitwiseOr();
    public void BitwiseXor();
    public void ShiftLeft();
    public void ShiftRight();
    public void Call(MethodBuilder target);
    public void Return();
    public void ClearBody();
}
```

Created by AddMethod or AddFunction. Assembly always identifies its owning builder;
DeclaringType is null for a top-level function. Body operations append conventional IL: Int32 constants,
zero-based argument loads, arithmetic, static calls and return. Call accepts a method
from this graph or another builder; null raises ArgumentNullException. Each append
rejects more than 4096 instructions with InvalidDataException. ClearBody permits
replacement without changing signature/ownership.

Write checks argument indices, typed primitive stack effects, call parameter/result contracts,
a final return with the exact declared stack shape, and no earlier return. It derives
max stack and rejects underflow, extra results or missing return. Loops, branches,
general local types, exceptions and arbitrary raw IL are intentionally absent. Int32 locals are documented below. Runtime overflow
behavior remains that of the selected target/bridge; this slice does not reconcile
all CLR versus neoCLR arithmetic policies.

The C# writer consumer builds a dependency's `Twice(Int32) -> Int32` and an application
that calls it with 20, adds 2 and returns 42. It tests repeat writes, body edits, imported
TypeRef/MemberRef resolution, entry-point ownership and invalid-stack rejection. Twenty-one
standalone C# contract groups pass, including earlier identity/reference tests.

### neoCLR acceptance test

The C# integration runner creates both PEs through the public builder API, invokes the
existing Raven CLI bridge, assembles its output into a native `.neo.json` artifact,
then asks neoCLR to load/verify and run that artifact. Expected result and process exit
code are 42. No application IL or JSON is hand-authored for this test.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --runtime-integration /path/to/neoclr /path/to/Probe.dll \
  /path/to/NeoCLR.CoreProbe.dll /path/to/System.neoil /fresh/output/directory
```

Inputs must be a matching runtime, bridge, core reference and composed Raven System
library. The runner refuses an existing output directory and writes a report with
artifact/tool hashes. It is a test CLI, not a library API. The bridge converts ordinary
CLI signatures and bodies into the current native format; it does not enable native
#Neo semantics. Direct PE/#Neo runtime loading and general compiler coverage remain
separate work.

### Direct native acceptance test

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --native-integration /path/to/neoclr /fresh/output/directory
```

The C# test emits a top-level `Twice` function in one assembly and a top-level entry
function in another. It also exercises a no-result function, a type-owned method,
argument loads and all three arithmetic operations. neoCLR directly verifies/loads
these API-produced files and returns 42. Missing dependencies and incorrect revisions
are rejected with the corresponding diagnostics. The runner preserves the artifacts
and a hash report. This requires only the metadata test executable and native runtime,
with its bundled System library; no Raven bridge or hand-authored application JSON is
involved. See [recorded evidence](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/native-validation.json).

### Importing a read-only callable (development)

```csharp
ImportedMethodReference AssemblyBuilder.ImportReference(
    MethodDefinition definition, AssemblyIdentity dependencyCoreLibrary);
void MethodBuilder.Call(ImportedMethodReference target);

public sealed class ImportedMethodReference
{
    public AssemblyBuilder Owner { get; }
    public AssemblyIdentity AssemblyIdentity { get; }
    public string? Namespace { get; }
    public string? DeclaringTypeName { get; }
    public string Name { get; }
    public int ParameterCount { get; }
    public bool ReturnsValue { get; }
}
```

`ImportReference` copies a static signature containing primitives, vectors, dependency-local
reference types/constructions and optional unconstrained method parameters from an
external read-only definition. No producer builder, body, runtime load or resolver is needed.
`Owner` is the consuming builder; `AssemblyIdentity` is the exact dependency identity.
The namespace and type name are null for a global function. `ReturnsValue` is false
for no result. References expose no body editing or signature mutation.

The caller must supply the dependency's core-library contract explicitly; it must
equal the consuming builder's `CoreLibrary`. This is a host assertion, not a deduction
from CLI primitive bytes or a verification of the dependency's implementation. Native
dependencies must be separately supplied and use this writer's format-5 naming
contract. PE output uses ordinary AssemblyRef/TypeRef/MemberRef rows for type-owned
methods. Cross-assembly globals remain native-only. Access checks are not performed.

Nominal signature tokens must identify public top-level unconstrained invariant reference
types in that dependency snapshot (TypeDef). TypeRef signatures require a future explicit
resolver contract. Value types, nested/generic owners, instance/constrained-generic/other
signatures, signed or flagged dependency
identities and imports of the output identity throw `InvalidDataException`. A single
builder admits at most 256 imported assembly identities and 4096 imported methods.
Different module MVIDs under one identity, or differing callable contracts under one
MVID/token, are rejected. Repeated compatible imports return the same reference;
MVID consistency is not content authentication. Null arguments throw
`ArgumentNullException`. `Call` rejects references from another consumer with
`ArgumentException`; stack validation still occurs at emission. A null literal passed
to the overloaded `Call` now requires a cast to the intended target type.

```csharp
var dependency = AssemblyDefinition.ReadAssembly(dependencyPe, expectedExtended: false);
var method = dependency.MainModule.Types.Single(t => t.Name == "Math").Methods.Single();
var reference = output.ImportReference(method, dependencyCoreIdentity);
var main = output.AddFunction("Main");
main.LoadConstant(21);
main.Call(reference);
main.Return();
output.EntryPoint = main;
var nativeImage = output.WriteNativeAssembly();
```

Twenty-two C# contract groups now pass. They cover owned imports after producer/input
mutation, exact PE method resolution, native global no-result calls, owner/core/identity
rejections, opaque unsupported signatures and stack errors. The native C# gate imports
its cross-assembly global from a PE snapshot and returns 42. The Raven consumer also
imports from the read-only snapshot and returns 42 in neoCLR.

### Native declaration reader and compiler reference projection (development)

```csharp
namespace NeoCLR.Metadata.Experimental.Model;
public sealed class NativeAssemblyDefinition
{
    public AssemblyIdentity Identity { get; }
    public IReadOnlyList<AssemblyIdentity> References { get; }
    public static NativeAssemblyDefinition ReadAssembly(ReadOnlySpan<byte> image);
    public byte[] CreateReferenceAssembly(AssemblyIdentity coreLibrary);
}
```

`ReadAssembly` copies an owned declaration snapshot from the bounded writer's native
format-5 UTF-8 JSON. `Identity` retains the exact unsigned assembly identity.
`References` retains exact direct native dependency identities in manifest order,
including implementation-only references omitted from the PE projection. The list is
owned and read-only; identities include version/culture/key/flags. It is bounded to
256 entries, performs no resolution or file/runtime loading, and is not a transitive
closure. Hosts must supply the required native dependency graph to the runtime.

This is a metadata reader, not a native verifier, arbitrary format-5 reader, or body
translator. Bodies remain opaque; the original native artifact must pass neoCLR's
verifier before execution. Disposing JSON parsing state or changing the input buffer
has no effect on the snapshot. No native file is loaded into a runtime by either API.

Supported declarations are public static Int32/Int64/Boolean/String/no-result functions (including globals)
and public/internal static classes with no fields. The reader checks canonical identity tuples,
encoded module/type/function names, references, origins, tokens, owner order, entry
point signature, duplicate declarations and unsupported declaration fields. Unknown
root/declaration fields and duplicate JSON properties are rejected. It accepts at most
4 MiB of input, depth 64, one manifest/module, 256 references, 4,095 types, 4096 methods,
256 methods per owner (including global scope), and 256 parameters per method.
Unsupported, inconsistent or malformed metadata throws `InvalidDataException`.
These consistency checks are not authentication and do not validate instruction bodies.

`CreateReferenceAssembly` creates owned PE bytes for the existing .NET semantic-loader
bootstrap. `coreLibrary` is explicit and must supply System.Object and
System.Runtime.CompilerServices.ReferenceAssemblyAttribute; null throws
`ArgumentNullException`. The PE contains callable/type declarations, the reference-only
attribute, and throwing placeholder bodies. Native bodies are never translated or
replaced with executable behavior. Entry points and native body dependency references
are omitted; this supported signature subset needs only primitive/core types. Exceeded
writer limits or incompatible assembly identities throw `InvalidDataException`.
Each projection can receive a fresh MVID; callers should reuse one projection/snapshot
for a compilation instead of treating repeated projections as the same module scope.

```csharp
var native = NativeAssemblyDefinition.ReadAssembly(nativeBytes);
var referencePe = native.CreateReferenceAssembly(explicitCoreIdentity);
var definitions = AssemblyDefinition.ReadAssembly(referencePe, expectedExtended: false);
// Register referencePe with the existing compiler loader and import definitions
// through AssemblyBuilder.ImportReference. Execute the original nativeBytes only.
```

This follows the .NET distinction between implementation and reference assemblies,
including ReferenceAssemblyAttribute and throwing placeholder bodies. The temporary
bridge adds a projection and cannot preserve arbitrary native semantics; a native
semantic-data provider should replace it. See Microsoft's
[reference assembly contract](https://learn.microsoft.com/en-us/dotnet/standard/assembly/reference-assemblies).

Twenty-four C# contract groups pass, including native Unicode/global/no-result
roundtrips, input ownership, malformed/unsupported metadata and count limits, reference
marker inspection, and .NET execution-load rejection with BadImageFormatException.
The Raven consumer starts from the native dependency, binds the projected declarations,
and emits applications that neoCLR executes with the original dependency to 42.

Native reference tests additionally cover exact same-name/different-version identities,
manifest order, repeated-call deduplication, collection immutability, source-buffer
ownership, omission of implementation-only PE references, and inconsistent/duplicate
native references. The compiler consumer now executes a three-assembly chain in neoCLR.

## RuntimeAssemblyContainer

Development-only host type in `NeoCLR.Metadata.Experimental`:

```csharp
public static class RuntimeAssemblyContainer
{
    public static byte[] Write(ReadOnlySpan<byte> nativeImage, AssemblyIdentity coreLibrary);
    public static byte[] WriteBinary(ReadOnlySpan<byte> nativeImage, AssemblyIdentity coreLibrary);
    public static byte[] Read(ReadOnlySpan<byte> image);
    public static AssemblyDefinition ReadCliProjection(ReadOnlySpan<byte> image);
}
```

`Write` accepts the bounded writer's native format-5 JSON and explicit core-library
identity supplying Object and ReferenceAssemblyAttribute. It validates declarations,
creates a reference-only CLI projection, then embeds a required NEOX execution section
(kind 256, schema 1) in a new PE32 `.neometa` section. Both metadata views come from
the same native snapshot. The native image plus its 32-byte envelope header/directory
must fit 1 MiB; the PE must fit 4 MiB. Null core throws ArgumentNullException;
unsupported declarations, exceeded bounds or malformed containers throw InvalidDataException.
Each call creates a fresh projection MVID; whole-PE byte determinism is not promised.

`Read` validates unsigned PE32 framing, recognition marker/digest, envelope and native
declarations. It returns an owned copy of native UTF-8 JSON; mutation cannot affect
the input or future reads. Ordinary PEs, unknown required schemas, optional/unsupported
execution sections, missing/changed bindings and malformed data throw InvalidDataException.
Optional unknown sections are ignored. Bodies are opaque to this host API: successful
inspection is not runtime admission, dependency resolution or typed verification.

`ReadCliProjection` performs the same checks and reads the physical CLI declarations
as a Cecil-style `AssemblyDefinition`. Its `Profile` is null (no structural reference
profile), `EntryPointToken` is zero and `Write()` preserves the complete original PE.
It additionally rejects unsupported/malformed CLI metadata. The native payload is the
runtime authority; this method does not prove semantic equality of arbitrary native
and CLI views. Consumers should use the aware writer and keep their compiler-reference
file and read snapshot identical. General rewriting and automatic resolution are absent.

```csharp
byte[] native = builder.WriteNativeAssembly();
byte[] pe = RuntimeAssemblyContainer.Write(native, builder.CoreLibrary);
File.WriteAllBytes("Program.dll", pe);
var declarations = RuntimeAssemblyContainer.ReadCliProjection(pe);
var nativeDeclarations = NativeAssemblyDefinition.ReadAssembly(RuntimeAssemblyContainer.Read(pe));
```

neoCLR `verify Program.dll` / `run Program.dll` accepts this container, including
`--module Library.dll` dependencies. The Rust host API is
`metadata_container::native_json(&[u8]) -> Result<&str, Fault>` for borrowed transport
extraction and `metadata_container::load(&[u8]) -> Result<Module, Fault>` for legacy validation, including bundled System linking. `assembler::ModuleInput::MetadataPe(&[u8])` supports mixed
explicit module sets; `LoadedProgram` remains responsible for linking and execution.
There is no guest loader API yet. CLI bodies are throwing reference stubs, never
executed by neoCLR. The payload still uses JSON; direct loading does not yet eliminate
text parsing or establish a speedup. See the [profile/design](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/design/extended-cli-metadata.md#direct-runtime-container-checkpoint--2026-09-30).

### Native console literal output

`MethodBuilder.WriteConsoleLine(string text)` appends native console output without
changing the surrounding Int32 stack. Text is limited to 64 KiB of valid UTF-8; null
throws ArgumentNullException, unpaired UTF-16 surrogates and exceeded literal bounds
throw ArgumentException. The assembly's aggregate console-literal bytes are limited
to 4 MiB before serialization (InvalidDataException); encoded output/container limits
still apply. `WriteNativeAssembly` lowers the operation to `ldstr`,
`System.Console.WriteLine(String)` and `pop` for the bundled System's Void-valued result.
System is the runtime's implicit platform dependency. CLI `Write()` rejects this
native-only operation with InvalidDataException; reference-only projections preserve
declarations and continue emitting throwing stubs. General string signatures and
Console overload import are not introduced.

The compiled Raven acceptance case prints Hello World directly from Main, then from
a separate Greet function called by Main. Both emitted containers load/verify/run
in neoCLR and exit zero. The API remains independent of Raven; compiler-side mapping
requires an explicit registered Console reference and only accepts string literals.

### Binary execution payloads (schema 2)

```csharp
public static byte[] RuntimeAssemblyContainer.WriteBinary(
    ReadOnlySpan<byte> nativeImage, AssemblyIdentity coreLibrary);
```

`WriteBinary` accepts the bounded writer's format-5 JSON (at most 4 MiB) as a host-side
intermediate and writes section 256/schema 2 using a definite-length CBOR profile.
The binary payload plus its 32-byte directory must fit 1 MiB; the PE still fits 4 MiB.
It has the same core/reference projection, ownership and null-core contract as Write.
InvalidDataException covers unsupported declarations, non-integer numbers, invalid
encoding, duplicate fields and exceeded bounds. Only signed Int64 numbers, valid UTF-8
text, arrays, text-keyed maps, booleans and null are encoded; tags, bytes and floats
are not part of this profile. Decode is limited to depth 64 and 262,144 items (including
keys), and rejects duplicate keys, nonminimal encodings and trailing bytes.

`Read` and `ReadCliProjection` now accept schemas 1 and 2. For schema 1, Read returns
the original JSON bytes; for schema 2 it reconstructs owned format-5 JSON with equivalent
values for the existing host reader. Original whitespace/numeric spelling are not
preserved. Reconstructed JSON remains bounded to 4 MiB. Snapshot Write still preserves
the full original PE byte-for-byte. The writer continues to use fresh projection MVIDs.
The original `RuntimeAssemblyContainer.Write` keeps emitting schema 1 for compatibility.

Rust `metadata_container::decode(&[u8]) -> Result<Module, Fault>` validates the container
and decodes either payload without linking; `load` applies the legacy validation,
including bundled System linking. Use ModuleInput::MetadataPe for explicit dependency sets.
Schema 2 is deserialized directly to the runtime model, with no JSON roundtrip.
`native_json` remains a borrowed schema-1-only extractor and faults for schema 2.
The CLI and ModuleInput::MetadataPe use decode/load and accept both encodings.
Binary containers have no embedded text for the debugger's source pane.
See the [binary profile and tradeoffs](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/design/extended-cli-metadata.md#binary-native-execution-profile--2026-09-30).


## NativeModuleContainer

Public static host class in `NeoCLR.Metadata.Experimental`:

```csharp
byte[] NativeModuleContainer.WriteBinary(ReadOnlySpan<byte> nativeImage);
byte[] NativeModuleContainer.Read(ReadOnlySpan<byte> image);
byte[] NativeModuleContainer.WriteLibraryBinary(ReadOnlySpan<byte> nativeImage);
```

`WriteBinary` translates existing format-5 JSON values to a standalone NEOX envelope
with required section 256/schema 2. Unlike RuntimeAssemblyContainer it does not require
canonical API-writer declaration names and does not construct a PE or CLI projection.
`Read` reconstructs equivalent owned JSON values. Neither call checks full declaration
schemas, resolves references or verifies bodies; the runtime performs those checks.
Header validation requires numeric format 5, nonempty module name and a functions array.
Members, origins, identities, native names, dependencies and instruction operands are
preserved as values. No tokens or references are remapped. Unknown optional envelope
sections are accepted but omitted from the returned JSON; retain original bytes if
opaque section preservation is required.

All calls throw InvalidDataException for malformed headers, framing, unsupported
binary values or exceeded limits. WriteBinary retains schema 2: input JSON is limited
to 4 MiB, the envelope to 1 MiB; the same integer-only, Unicode, depth and item limits as the PE binary profile
apply. Lexical JSON spellings are not preserved. There is no CLI stream binding digest
in this standalone form and no authenticity guarantee. It cannot be passed to Raven's
.NET MetadataReference loader.

```csharp
var image = NativeModuleContainer.WriteBinary(File.ReadAllBytes("System.neo.json"));
File.WriteAllBytes("System.neox", image);
```

Rust `metadata_container::decode_envelope(&[u8]) -> Result<Module, Fault>` accepts a
standalone execution envelope (schema 1, 2 or 3), deserializes the native model and rejects
invalid framing/required schemas. `load_envelope` additionally applies legacy validation,
including bundled-System linking for non-System modules. `ModuleInput::NativeEnvelope`
uses decode for explicit dependency sets. The CLI detects NEOX magic for the root,
`--module` and `--system`; existing PE and JSON handling is unchanged. No embedded
source text is available to the debugger.


`WriteLibraryBinary` emits required execution schema 3 in a standalone envelope.
It accepts negative Int64 values and nonnegative UInt64 values; Double operands retain
their exact unsigned IEEE-754 bits, including negative zero and NaN payloads. CBOR
floating-point values themselves remain unsupported. Input/reconstructed JSON is limited
to 32 MiB, the envelope to 8 MiB, and item count (including map keys) to 2,097,152.
Depth remains 64. `Read` accepts schema 2 with its original limits and schema 3 with
these larger budgets. The general MetadataEnvelope/MetadataSection public APIs retain
their 1 MiB limits; the larger envelope is scoped to NativeModuleContainer. The PE
RuntimeAssemblyContainer APIs continue to support schemas 1/2 only.

Older runtimes reject required schema 3; choose WriteBinary when the old profile's
bounds and signed-only numbers suffice. The translator tool now emits schema 3 by
default. This is an explicit compatibility change for that experimental tool, not a
silent relaxation of schema 2. Semantic admission, linking and verification remain
runtime responsibilities. Larger budgets increase possible memory/CPU costs; they
are finite limits, not streaming or lazy-loading guarantees.


### Rust host native assembly writer

`metadata_container::write_module(module: &Module) -> Result<Vec<u8>, Fault>` encodes
an immutable borrowed native module into owned standalone schema-3 NEOX bytes. It
serializes the model directly, without JSON, and preserves its definitions/references.
Unsupported semantic format (anything except 5), empty/whitespace module name, encoding
failures, payload/item/depth
budget violations return Fault. The byte sink rejects output beyond 8 MiB minus envelope
framing before extending its payload buffer; the completed encoding is checked by the
same schema guard as the reader. It does not link dependencies, validate declarations
or type-verify bodies. Callers perform those steps separately.

The CLI `assemble --format neox` uses the existing assembler/resolver and verifies the
complete load set before invoking this API and creating the output file. Only the root
module is serialized. This Rust-host function is not a new guest introspection API or
an addition to the .NET Cecil-style object model. The independent .NET NativeModuleContainer
reader validates the emitted wire format in the cross-reader consumer tests.


## NativeLibraryDefinition and NativeFunctionDefinition

Development APIs in `NeoCLR.Metadata.Experimental.Model`. These .NET-host types
are not guest Introspection APIs. They adapt native declarations to the existing
Raven .NET semantic importer; they do not constitute a complete native symbol provider.

```csharp
public sealed class NativeLibraryDefinition {
    public string ModuleName { get; }
    public IReadOnlyList<string> TypeNames { get; }
    public IReadOnlyList<NativeFunctionDefinition> Functions { get; }
    public static NativeLibraryDefinition ReadAssembly(ReadOnlySpan<byte> image);
    public byte[] CreateStaticInt32ReferenceAssembly(
        AssemblyIdentity projectionIdentity, AssemblyIdentity coreLibrary,
        IEnumerable<NativeFunctionDefinition> functions);
}
public sealed class NativeFunctionDefinition {
    public NativeLibraryDefinition Library { get; }
    public int TableIndex { get; }
    public string Name { get; }
    public string? DeclaringTypeName { get; }
    public bool TryGetStaticInt32Signature(out int parameterCount);
}
// Additional MethodBuilder overload:
public void Call(NativeFunctionDefinition target);
```

`ReadAssembly` accepts standalone binary schemas 2/3 through NativeModuleContainer,
including the translated System library. It owns the decoded data and inventories all
functions, even unsupported signatures. `ModuleName` and `Name` retain native spellings;
`TypeNames` retains table order and can repeat for differing generic arities. Duplicate
name/arity types fail. Inventories are limited to 65,536 types/functions, descriptive
names to 4,096 characters, and transport retains its existing byte/depth/item limits.
Malformed framing/inventory raises InvalidDataException. This is not runtime body or
complete declaration validation. `TableIndex` is snapshot-local position, not an inferred
CLI token or persistent native identity. `DeclaringTypeName` is a nominal Named owner;
null also covers generic/primitive owner forms that this callable view does not support.

`TryGetStaticInt32Signature` accepts static nongeneric nominal-owner functions with
0–256 Int32 parameters and an Int32 result. Private, instance, virtual, abstract,
byref-receiver, no-result, Result, generic and other signatures return false and count
zero. Native visibility defaults to public when omitted; explicit nonpublic visibility or
nonpublic origin access is rejected. It does not promise the owning type is
projectable; the projection separately checks owner existence, public visibility and
nongeneric shape.

The projection requires an explicit **partial** selection of 1–4096 owned function
objects and an unsigned synthetic identity distinct from the supplied core identity.
At most 256 distinct owners are projected. Empty, duplicate, foreign or unsupported
selections, duplicate signatures and invalid owners fail with InvalidDataException.
Null arguments fail with ArgumentNullException. The returned ordinary reference-only
PE preserves selected method names and Int32 signatures under original type names,
using static containers and throwing bodies. It does **not** preserve native assembly
identity, type instance/field/property shape, generic contracts, parameter names or
attributes. No methods outside the explicit selection are claimed to be available.
Treating the projection as a complete core library or as an executable implementation
is unsupported. Native/native-host type-name collisions still require a proper core
provider; the current command removes host facade references in selected-native mode and the
consumer verifies that Math.Min binds to the native view, not the host implementation.

`MethodBuilder.Call(NativeFunctionDefinition)` accepts only a recognized Int32
callable from module `System`; null raises ArgumentNullException, other modules or
signatures raise InvalidDataException. The native writer retains the original owner,
function name and parameter signature; it does not synthesize writer-specific hashed
identities for System methods. Stack underflow is rejected during writing. Ordinary
CLI writing rejects this native-only operation. Runtime verification and execution must
use the matching explicit System assembly. No revision or image digest is encoded for
that implicit System dependency; this is a bootstrap limit to replace with a general
native assembly binding contract.

Compiled examples and failures: `NativeLibrarySymbolChecks.cs` in the metadata C# tests
and Raven's `SystemSymbolChecks.cs` consumer. The builder now provides the bounded opcode/operand Emit API below. An editable
instruction collection and full Cecil body-editing support remain future work.


## OpCode and MethodBuilder.Emit

Development APIs in `NeoCLR.Metadata.Experimental.Model`. The enum describes logical
instructions supported by both writer backends; its numeric values are **not** physical
CLI or native opcode bytes.

```csharp
public enum OpCode { Ldc_I4, Ldarg, Add, Sub, Mul, Call, Ret, Ldloc, Stloc, Ceq, Clt, Cgt, Br, Brtrue, Brfalse, Ldc_Bool, Pop, Ldc_I8, Conv_I8, Conv_I4, Neg, Not, Ldstr, Starg, Div, Rem, And, Or, Xor, Shl, Shr }
public sealed partial class MethodBuilder {
    public void Emit(OpCode opCode);
    public void Emit(OpCode opCode, int operand);
    public void Emit(OpCode opCode, MethodBuilder operand);
    public void Emit(OpCode opCode, ImportedMethodReference operand);
    public void Emit(OpCode opCode, NativeFunctionDefinition operand);
}
```

| Opcode | Operand | Contract |
| --- | --- | --- |
| Ldc_I4 | int | Push a signed Int32 constant. |
| Pop | none | Discard one value of either supported primitive type; empty stack rejects when writing. |
| Ldarg | int | Load the zero-based declared primitive argument; bounds checked when writing. |
| Add, Sub, Mul, Div, Rem | none | Consume matching Int32/Int64 values and push the same-width arithmetic result. |
| Call | MethodBuilder | Use the target signature; external core/identity constraints checked when writing. |
| Call | ImportedMethodReference | Must belong to the consuming assembly builder. |
| Call | NativeFunctionDefinition | Native-only static Int32 System callable; matching System must be supplied at runtime. |
| Ret | none | Must finish the body with its exact declared return stack. |

Unknown enum values, missing/unexpected operands and wrong opcode/operand overloads
raise ArgumentException before appending. Null call operands raise ArgumentNullException;
foreign imported references raise ArgumentException. Unsupported native module/signature
raises InvalidDataException, as does exceeding the existing 4096-instruction method limit.
Failed Emit calls leave the body unchanged. Stack/argument/return validation remains
at Write/WriteNativeAssembly; raw emission does not bypass it. Ordinary CLI writing still
rejects native System calls. No caller-supplied object, byte array or metadata token
operand is accepted.

LoadConstant, LoadArgument, Add, Subtract, Multiply, all Call overloads and Return now
call Emit. Their supported semantics are unchanged. WriteConsoleLine remains a native
convenience expansion; general string operands, non-primitive local variables, exception
regions and editable instruction collections are unsupported. Future ILProcessor-like
editing must define instruction ownership and branch/exception target repair separately.

Example from the C# consumer, used with an Int32 method and a two-parameter Int32 helper:

```csharp
main.Emit(OpCode.Ldc_I4, 10);
main.Emit(OpCode.Ldc_I4, 4);
main.Emit(OpCode.Call, helper);
main.Emit(OpCode.Ret);
```

`EmitChecks.cs` proves helper/raw byte equivalence on the same graph, executes a CLI
arithmetic/call entry point to 42, and checks imported/native operands and failures.
Raven's native emitter now uses this surface; native runtime and translated-System
integration remain its executable consumer evidence.

## Int32 local slots (development, 2026-10-01)

The host-only `NeoCLR.Metadata.Experimental.Model` API adds:

```csharp
public sealed class LocalDefinition {
    public MethodBuilder Method { get; }
    public int Index { get; }
}
public sealed partial class MethodBuilder {
    public IReadOnlyList<LocalDefinition> Locals { get; }
    public LocalDefinition DeclareInt32Local();
    public void LoadLocal(LocalDefinition local);
    public void StoreLocal(LocalDefinition local);
    public void Emit(OpCode opCode, LocalDefinition local);
}
```

`DeclareInt32Local` allocates a stable zero-based slot, at most 256 per method;
exceeding that bound throws `InvalidDataException`. `Locals` is a read-only ordered
view. `ClearBody` clears instructions and retains locals/handles; rebuilding the method
is required to discard its declarations. The typed Emit overload accepts only `Ldloc`
and `Stloc`. Null throws `ArgumentNullException`, a different method owner or incorrect
opcode throws `ArgumentException`, and the existing instruction bound throws
`InvalidDataException`. Rejected instructions do not alter the body.

The existing `Emit(OpCode, int)` overload also accepts raw `Ldloc`/`Stloc` slot indices.
Writes reject out-of-range slots, stack underflow and loads before a store in the linear
body with `InvalidDataException`. Each write recomputes initialization; a previous body
or successful write does not initialize a rebuilt body. CLI emission writes Int32 local
signatures and init-locals method headers; native emission writes format-5 `locals` and
native local instructions. The reader accepts absent locals in older producer artifacts
and validates declared Int32/Int64/Boolean/String local lists; reference projections still omit executable
body details. Older experimental readers may reject the added `locals` field.

Unlike unrestricted Cecil bodies, this bounded API enforces initialization and stack
contracts when writing. Typed owner handles prevent accidental cross-method use; raw
indices support assembler consumers. General local types, address-taking and
scope/debug metadata remain outside this slice. The compiled C# `LocalChecks` consumer
executes CLI locals and checks native roundtrips and rejected contracts; Raven's local
assignment probe additionally verifies and executes the binary artifact in neoCLR.

## Branch labels and control flow (development, 2026-10-01)

```csharp
public sealed class BranchLabel { public MethodBuilder Method { get; } }
public sealed partial class MethodBuilder {
    public BranchLabel DefineLabel();
    public void MarkLabel(BranchLabel label);
    public void Emit(OpCode opCode, BranchLabel label);
    public void Emit(OpCode opCode, bool operand);
}
```

`DefineLabel` allocates a method-owned symbolic destination (maximum 4096;
`InvalidDataException` beyond that). `MarkLabel` marks its current position once.
The label Emit overload accepts `Br`, `Brtrue` and `Brfalse`; the Boolean overload
accepts only `Ldc_Bool`. Null labels throw `ArgumentNullException`; wrong owners,
repeated marks and incompatible opcodes throw `ArgumentException` without changing
instructions. Instruction-limit violations throw `InvalidDataException`. `ClearBody`
retains handles but removes marks, so reused destinations must be marked again.

Operand-free `Ceq` pops two matching Int32, Int64 or Boolean values and pushes Boolean;
mixed operand types reject. `Clt` and `Cgt` pop matching Int32/Int64 values and push Boolean,
using signed comparisons. Boolean equality permits logical negation (`false; ceq`)
without treating native Boolean as Int32. Conditional branches pop Boolean, not Int32; conditions may
also use `Ldc_Bool`. General Boolean locals/signatures are not introduced here.

Writing computes typed stack states and definitely stored locals over the control-flow
graph, including backward edges. Unmarked targets, incompatible stack joins, wrong
operand types, uninitialized loads on any incoming path, reachable fallthrough and
unreachable executable instructions fail with `InvalidDataException`. Return instructions
may occur on multiple reachable paths, each with the declared result and no extra stack
values. Labels emit no runtime instruction. Unused/unreachable label marks are harmless.
The earlier linear-only validation contract is superseded for branch-capable bodies.

CLI destinations become byte displacements; native destinations become instruction
indices after convenience-operation expansion (including Console output). Labels do not
encode byte offsets themselves, so backend layouts remain independent. The native
runtime already supports these format-5 instructions; no binary schema changes are
needed. Compared with Cecil's general ILProcessor, this supports forward/backward branches
with owned labels and checked typed control flow, but still has no insertion/removal API,
exception regions or arbitrary opcode surface. C# FlowChecks covers executable loops and
invalid joins/initialization; Raven's executable consumer includes Console inside a loop.


## Primitive signatures (development, 2026-10-01)

All types below are in `NeoCLR.Metadata.Experimental.Model`.

```csharp
public enum PrimitiveType { Void, Int32, Boolean, Int64, String, Byte, Single, Double }
public sealed class PrimitiveMethodSignature
{
    public PrimitiveMethodSignature(PrimitiveType returnType,
                                    IEnumerable<PrimitiveType> parameterTypes);
    public PrimitiveType ReturnType { get; }
    public IReadOnlyList<PrimitiveType> ParameterTypes { get; }
}
MethodBuilder AssemblyBuilder.AddFunction(string name, MethodSignature signature);
MethodBuilder TypeBuilder.AddMethod(string name, MethodSignature signature);
MethodSignature MethodBuilder.Signature { get; }
MethodSignature ImportedMethodReference.Signature { get; }
bool MethodDefinition.TryGetStaticPrimitiveSignature(out PrimitiveMethodSignature? decoded);
```

The signature constructor copies up to 256 ordered Int32/Int64/Boolean/String parameters. Results
may also be Void, meaning no result, not an inhabited native Void value. Null parameters
throw ArgumentNullException; invalid enum values, Void parameters or excessive counts
throw ArgumentException. ParameterTypes is an immutable view of the copied array.

The typed declaration overloads retain existing name/owner/count bounds. A null
signature throws ArgumentNullException; duplicate name plus ordered parameter types
throws ArgumentException, regardless of result type. Thus `Identity(int)` and
`Identity(bool)` coexist, while return-only overloads reject. Legacy count/result
overloads still mean Int32 parameters and Int32/no-result return.

MethodBuilder.Signature and ImportedMethodReference.Signature expose the immutable
contract. ParameterCount includes both types; ReturnsValue includes Boolean. Calls
consume declared parameter types in reverse stack order, and Return checks the declared
result. Int32 is not implicitly interchangeable with Boolean. Entrypoints remain
parameterless Int32/no-result; Boolean entrypoints reject at write time.

The primitive recognizer returns a fresh signature on success or null/false for
unsupported or malformed encodings. It accepts only static nongeneric default CLI
calling convention, canonical parameter counts and exact primitive encodings.
TryGetStaticInt32Signature keeps its earlier stricter behavior and rejects Boolean.
MemberReference.ResolveMethod and AssemblyBuilder.ImportReference match full primitive/vector
signatures, including ordered parameter types and result. Existing explicit resolver,
identity, core-contract and snapshot checks remain in force.

Native writers, readers and reference-only projections preserve these types without
a format/schema change. Older experimental readers may reject Boolean declarations.
Local declarations support Int32 and Boolean; selected System inventory imports remain the
separate Int32-only contract. Native bodies are still verified by neoCLR.

```csharp
var predicate = output.AddFunction("IsPositive",
    new PrimitiveMethodSignature(PrimitiveType.Boolean, [PrimitiveType.Int32]));
predicate.LoadArgument(0);
predicate.LoadConstant(0);
predicate.Emit(OpCode.Cgt);
predicate.Return();
```

C# contract tests cover mixed arguments, overloads, CLI execution, native projection,
MemberRef resolution, immutable signatures and invalid calls. Raven's integration
probe also executes the emitted Boolean calls in neoCLR, including a separately
compiled library referenced through its CLI declaration projection.


### Typed local declarations (development, 2026-10-01)

```csharp
LocalDefinition MethodBuilder.DeclareLocal(PrimitiveType type);
PrimitiveType LocalDefinition.Type { get; }
```

DeclareLocal accepts Int32, Int64, Boolean or String, returning a stable method-owned slot with an
immutable Type. Void and unknown enum values throw ArgumentException; the shared
256-local limit throws InvalidDataException. DeclareInt32Local remains shorthand for
DeclareLocal(PrimitiveType.Int32). ClearBody preserves the slot and type, but resets
body-derived initialization: a load must follow stores on all reachable paths.

StoreLocal and raw Stloc require the value's type to match the slot, including Boolean;
LoadLocal/Ldloc push that type. Wrong types reject before either writer emits an image.
CLI local signatures and native local lists retain the type. Native declaration reading
accepts both primitive local types, including old artifacts with no local-list field.
Reference-only projections omit native executable bodies and locals as before.

The C# local contracts execute Boolean locals in CLI, inspect their reflected type,
roundtrip native projections and reject cross-type stores and uninitialized loads.
Raven's shared planner now carries local types to each backend and validates a program
that stores a predicate result, reassigns it and compares Boolean locals on both runtimes.


### Discarding call results (development, 2026-10-01)

`MethodBuilder.Emit(OpCode.Pop)` removes one evaluation-stack value, preserving all
values below it. It accepts no operand and works for Int32, Int64, Boolean or String. Emission with
an operand throws ArgumentException without appending an instruction. Writer flow
validation throws InvalidDataException on underflow, including popping after a
no-result call. A no-result call has no value to discard. CLI emission uses `pop`;
native emission uses neoIL `pop`, with no schema change. Existing limits and branch
join checks continue to apply. Tests execute discarded Boolean and Int32 calls in
CLI and verify native writer rejection of underflow; Raven's binary-assembly consumer
also executes local and imported statement calls in neoCLR.


### Int64 and integer conversions (development, 2026-10-01)

The primitive signature/local API also accepts `PrimitiveType.Int64` in parameters,
results and local slots. `MethodDefinition.TryGetStaticPrimitiveSignature`, typed
imports, MemberRef resolution, native reading and reference-only projections preserve
Int64. The legacy Int32 recognizer still rejects every Int64 signature. Entrypoints
remain parameterless Int32/no-result; Int64 entrypoints reject before writing.

```csharp
void MethodBuilder.Emit(OpCode opCode, long operand); // Ldc_I8 only
// Operand-free Emit also accepts Conv_I8 and Conv_I4.
```

Ldc_I8 carries the exact signed 64-bit operand, including both extrema. Other opcodes
on this overload throw ArgumentException; instruction limits throw InvalidDataException.
Pass a long operand (`42L`), since the int overload continues to recognize only its
existing operands. No implicit opcode/operand widening is performed.

Conv_I8 accepts Int32/Int64, sign-extending Int32; Conv_I4 accepts the same types and
retains the low 32 bits when narrowing. Both operations are unchecked. Boolean and
empty-stack conversions reject at write time with InvalidDataException. Add/Sub/Mul
and signed Clt/Cgt accept matching Int32 or Int64 operands; Ceq additionally accepts
matching Boolean. Arithmetic preserves width, comparisons produce Boolean. There is
no implicit mixed-width arithmetic or Boolean/integer interchange. Pop also discards
Int64 values. Typed calls, returns, local stores and branch joins preserve exact types.

CLI emission uses Int64 signatures, ldc.i8 and conv.i4/conv.i8. Native emission uses
the corresponding existing neoIL operations and Int64 type names, without a schema
change. Older experimental host readers may reject these newly admitted declarations.
Checked and user-defined conversions remain unsupported. See the floating-point
profile below for Single/Double conversions. Native verification is still required before execution.

C# tests cover extrema, sign extension, truncation, local types, native projection,
imports, strict legacy recognition and invalid conversions/mixed arithmetic. Raven's
consumer executes long locals and signed conversion boundaries in neoCLR and imports
an Int64 callable from a separately compiled binary library.


### Signed unary integer operations (development, 2026-10-01)

`MethodBuilder.Emit(OpCode.Neg)` negates the top Int32/Int64 value without changing
its width. Negating the minimum signed value wraps to that same value, matching the
CLI neg operation. `Emit(OpCode.Not)` complements every bit at the operand's width;
it is not Boolean logical negation. Neither opcode accepts an operand: other Emit
overloads reject with ArgumentException without appending an instruction. Existing
instruction bounds throw InvalidDataException.

Both writers reject empty stacks and Boolean operands with InvalidDataException
during flow validation. CLI emission uses neg/not; native emission uses the existing
neoIL neg/not operations without a format change. Checked and unsigned unary support is not implied; the floating-point profile below
also admits Single/Double negation. Native verification remains required. C# contracts check
both widths, extrema, invalid stack operands and native projections. Raven's shared
body path handles built-in signed unary + (identity), - and ~; the native consumer
executes minimum-value wrapping and complements in binary assemblies.

### String values (development, 2026-10-01)

```csharp
// NeoCLR.Metadata.Experimental.Model
// PrimitiveType.String and OpCode.Ldstr are appended enum members.
void MethodBuilder.Emit(OpCode opCode, string operand);
void MethodBuilder.WriteConsoleLine();
```

`String` is supported in PrimitiveMethodSignature parameters/results, DeclareLocal,
MethodDefinition.TryGetStaticPrimitiveSignature, imported method references and native
reference projections. It is a built-in text contract, not general nominal type support.
The legacy Int32 recognizer still rejects String; String entrypoints are invalid.

`Emit(Ldstr, text)` pushes a string. Text may be empty and contain embedded NUL or
supplementary Unicode characters; it must be non-null, valid Unicode and at most
65,536 UTF-8 bytes. Null throws ArgumentNullException; an incorrect opcode, unpaired
UTF-16 surrogate or oversized literal throws ArgumentException without appending an
instruction. The assembly-wide string/instruction/image limits still apply.
The CLI writer uses a user-string token and native output uses UTF-8 text with ldstr.

`WriteConsoleLine()` consumes one String value through the existing native System
console bootstrap and discards its inhabited Void result. It is native-only; CLI
Write rejects it. Stack mismatch/underflow fail at write time with InvalidDataException.
The existing string-argument overload remains a constant-output convenience.

String calls, returns, local stores, loads, Pop and control-flow joins retain exact
type checks. Arithmetic, numeric conversions, Ceq/Clt/Cgt and conditional branches do
not accept String. Null literals, string equality/concatenation and String instance
members are outside this bounded writer API; no interning/identity guarantee is made.

The C# StringChecks consumer covers CLI execution, native projection and imports,
Unicode/empty/NUL literals, exact UTF-8 bounds and rejected operations. Raven's native
probe additionally executes computed Unicode text and a separately compiled library.

### Argument stores (development, 2026-10-01)

```csharp
// NeoCLR.Metadata.Experimental.Model
void MethodBuilder.StoreArgument(int index);
void MethodBuilder.Emit(OpCode opCode, int operand); // now also accepts Starg
```

`Starg` consumes a value of the parameter's exact declared Int32/Int64/Boolean/String
type and replaces that by-value argument slot in the current invocation. StoreArgument
is a convenience for the same instruction. Parameters begin initialized; subsequent
Ldarg observes the replacement. Caller storage is not changed. Indices are zero-based
with no implicit receiver, bounded by the method signature (at most 256 parameters).

Invalid indices, including negative values, are rejected at writing even in unreachable
code. Stack underflow and mismatched stored types throw InvalidDataException before
bytes are returned. The general instruction limit applies at emission. The int Emit
overload retains its existing opcode/operand validation; Starg with other operand
kinds rejects with ArgumentException.

CLI output uses standard starg (FE 0B plus UInt16 slot); native output uses existing
format-5 starg. No signature or container schema changes are required. Mutability is
a source binding rule, not additional parameter metadata. Ref/out/in and receiver
assignment are outside this bounded API. C# contracts cover all four admitted types,
caller isolation, the last slot, bounds, empty stack and type mismatch.


### Signed division (development 2026-10-01)

`OpCode.Div` and `void MethodBuilder.Divide()` append operand-free signed integer
division. `Emit(OpCode.Div)` is equivalent to the helper. Writing requires two
matching Int32 or Int64 values and leaves one value of that width; stack underflow,
mixed widths, Boolean and String operands raise InvalidDataException before an image
is returned. Operand-bearing Emit overloads reject Div with ArgumentException.
The new enum member is appended; numeric enum values are not serialized opcodes.

Both CLI div and native div truncate toward zero. Dividing by zero or the minimum
signed value by -1 is legal to emit but faults when executed (CLR arithmetic
exceptions; neoCLR DivideByZero/ArithmeticOverflow faults). Neither helper performs
constant evaluation. No unsigned, floating or checked-context overload is provided.
Existing signature metadata and native transport are unchanged.


### Signed remainder (development 2026-10-01)

`OpCode.Rem` and `void MethodBuilder.Remainder()` append operand-free signed remainder;
`Emit(OpCode.Rem)` is equivalent. The typed stack rules and failures match Div: two
matching Int32/Int64 operands become one value of the same width. Invalid stacks
raise InvalidDataException when writing; operand-bearing overloads reject Rem with
ArgumentException. Rem is appended to the enum without renumbering previous members.

Ordinary nonzero results have the dividend's sign and smaller magnitude than the
divisor. Both writers use existing rem instructions. Zero divisors fault at execution;
neoCLR also faults on minimum/-1. CLI output follows the host CLR's rem behavior
(the tested host faults for minimum/-1 too; .NET documents this edge as platform
sensitive). No unsigned/floating remainder or new exception handling is added.


### Integer and Boolean bitwise operations (development 2026-10-01)

`OpCode.And`, `Or`, `Xor` and the operand-free `MethodBuilder.BitwiseAnd()`,
`BitwiseOr()`, `BitwiseXor()` helpers consume two matching Int32/Int64 or Boolean
values and produce the same type. Boolean operands use eager AND/OR/XOR truth tables;
these instructions do not short-circuit. `Emit(opCode)` is equivalent to the corresponding helper.
Sign bits participate normally; these operations do not overflow. CLI and native
output use existing and/or/xor instructions, with no metadata extension.

Writing rejects underflow, mixed types/widths or String operands with
InvalidDataException. Operand-bearing Emit overloads reject these opcodes with
ArgumentException; the instruction limit still applies. New enum values are appended.
Enum and nullable Boolean semantics are not added. Native Boolean operands require
the runtime Boolean bit-operation support introduced on `codex/extended-cli-metadata`
in `fa25609d`; older runtimes reject them. CLI keeps ordinary and/or/xor encoding.


### Integer shifts (development 2026-10-01)

`OpCode.Shl`/`Shr` and `void MethodBuilder.ShiftLeft()`/`ShiftRight()` consume an
Int32 count above an Int32 or Int64 value and leave a result with the value's width.
`Emit(opCode)` is equivalent. Left shifts discard high bits; signed right shifts
replicate the sign bit. No unsigned shift is included. Counts are stack values,
not Emit operands. Operand-bearing overloads reject Shl/Shr with ArgumentException.
Writing rejects missing operands, an Int64 count or a Boolean/String value with
InvalidDataException; count values are not range-checked by the writer.

These map directly to standard CLI/native shl/shr. For counts 0–31 or 0–63 respectively,
results agree. CLI results for negative/oversized counts are unspecified; native
execution keeps its existing count masking (low 5 or 6 bits). Callers needing portable
masked semantics can emit an explicit AND on the count before shifting. The API does
not insert that policy or change ordinary .NET codegen. The enum values are appended.


### MethodVisibility (development 2026-10-01)

```csharp
public enum MethodVisibility { Public, Internal, Private }
public MethodBuilder TypeBuilder.AddMethod(string name,
    MethodSignature signature, MethodVisibility visibility);
public MethodVisibility MethodBuilder.Visibility { get; }
```

The overload creates a static method with explicit access; existing AddMethod overloads
remain public. Public is subject to owner visibility, Internal restricts access to the
assembly, and Private to the declaring type. Invalid enum values raise
ArgumentOutOfRangeException; null signatures raise ArgumentNullException; name,
duplicate-signature and method-count rules are unchanged. Visibility is immutable.
Assembly functions retain public metadata representation; protected/friend access is
not added. MethodDefinition.Attributes already exposes the corresponding CLI bits.

CLI output uses standard Public/Assembly/Private MethodAttributes. Native output reuses
public/internal/private visibility and matching origin member_access (Internal maps to
Assembly). Public rows keep the omitted visibility default. The bounded native reader
accepts old public rows, validates access/origin consistency and preserves all three
values in the reference projection. It rejects unknown access, inconsistent flags and
nonpublic global functions with InvalidDataException. Older readers reject new nonpublic
rows. Bodies/references can describe forbidden calls: the writer is not an access checker;
Raven binding and runtime verification enforce access. ImportReference retains that rule.


### Assembly function access (development 2026-10-01)

`AssemblyBuilder.AddFunction(string name, MethodSignature signature,
MethodVisibility visibility)` creates an ownerless function with Public or Internal
access. Existing overloads remain Public. Name/signature bounds and duplicate checks
are unchanged. A null signature raises ArgumentNullException; invalid names, duplicate
signatures or exceeded limits raise ArgumentException. Private/unknown visibility
raises ArgumentOutOfRangeException before graph mutation: private access requires a
declaring type in the current native access model.

`MethodBuilder.Visibility` retains the selected access. CLI globals use ordinary
Public/Assembly MethodAttributes; native functions retain no owner and use existing
public/internal visibility. The bounded native reader preserves internal function
access in its CLI reference projection and rejects private ownerless definitions.
Older bounded readers reject internal global rows; no schema version changes. The
writer does not authorize calls; native verification checks resolved module identity.
Explicit entry selection may run an internal function in its defining module.

## Assembly function namespaces

Development host APIs (2026-10-01):

```csharp
MethodBuilder AssemblyBuilder.AddFunction(string @namespace, string name,
    MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public);
string MethodBuilder.Namespace { get; }
```

Adds an assembly-owned function with no declaring type. The namespace is empty for
the global namespace; otherwise it consists of nonblank dot-separated segments,
valid Unicode, and no control characters. Combined namespace/name length is at most
1024 UTF-16 code units. Duplicate namespace/name/parameter-type signatures and more
than 256 assembly functions are rejected. Null namespace/signature throws
ArgumentNullException; invalid namespace/name/duplicate/limit throws ArgumentException;
unsupported visibility throws ArgumentOutOfRangeException. Rejection leaves the graph
unchanged. Existing AddFunction overloads use the empty namespace.

MethodBuilder.Namespace returns the declared function namespace or type-owner
namespace. Name remains the simple source name. ImportedMethodReference.Namespace
now also exposes a namespaced function's namespace (null for global-namespace
functions); DeclaringTypeName remains null for all ownerless functions.

Native metadata stores the namespace separately and includes it in the executable
name encoding. The temporary CLI projection uses global MethodDefs named
`<NeoFunction>{uppercase UTF8 namespace hex}_{uppercase UTF8 name hex}`. The prefix
is reserved by AddFunction even in the global namespace to avoid collisions. Existing
unnamespaced function encoding is unchanged. Native snapshot/reference creation and
ImportReference preserve the namespace through that encoding. Raw CLI snapshots
expose the encoded physical Name; ordinary .NET source loaders do not gain native
namespace-function lookup. Native bodies remain authoritative in PE/#Neo artifacts.

```csharp
var min = assembly.AddFunction("System.Math", "Min",
    new PrimitiveMethodSignature(PrimitiveType.Int32,
        new[] { PrimitiveType.Int32, PrimitiveType.Int32 }));
// min.Namespace == "System.Math"; min.DeclaringType == null
```

The native reader rejects inconsistent namespace/executable names and namespaces
on type-owned methods. The runtime retains the namespace and rejects malformed or
type-owned namespace annotations. Calls use exact executable names; source namespace
lookup belongs to the compiler. New output requires the matching reader/runtime;
older artifacts omit the field and retain their global-namespace behavior.

## Root classes and primitive instance fields

Development host API (2026-10-01), additional to existing static AddType:

```csharp
TypeBuilder AssemblyBuilder.AddClass(string @namespace, string name,
    TypeVisibility visibility = TypeVisibility.Public);
bool TypeBuilder.IsStatic { get; }
IReadOnlyList<FieldBuilder> TypeBuilder.Fields { get; }
FieldBuilder TypeBuilder.AddField(string name, SignatureType type,
    FieldVisibility visibility = FieldVisibility.Private, bool isReadOnly = false);
public enum FieldVisibility { Public, Internal, Private }
```

AddClass creates a nonabstract, nonsealed root reference class. CLI base is
System.Object; native metadata has no explicit base. No constructor is synthesized.
AddType continues to create static classes. Both share the 4,095-type limit and
namespace/name uniqueness. Invalid names/visibility/duplicates throw ArgumentException
(including ArgumentOutOfRangeException for visibility). Namespaces and type names
retain the existing AddType contract.

AddField declares instance storage, never a property; storage is mutable by default. It accepts Int32,
Int64, Boolean, String or an owned nonstatic root class, a unique nonblank name without controls or invalid Unicode,
up to 1024 characters, and defined FieldVisibility values. Invalid declarations or
more than 256 fields per type throw ArgumentException before mutation. Static owners
throw InvalidOperationException. Writing enforces at most 4096 assembly fields.
Literal and static fields remain unsupported here. Later generic and imported reference-type
field support uses the signature contracts documented below.
Null types throw ArgumentNullException; Void and foreign classes throw ArgumentException
before mutation. Forward and self references use exact output-builder identity.
FieldBuilder exposes read-only DeclaringType (TypeBuilder), Name (string), FieldType
(SignatureType), Visibility (FieldVisibility) and IsReadOnly (bool). Field handles are owned by their type.
Development API migration: rebuild consumers and inspect FieldType.Primitive or
FieldType.ClassType; primitive AddField calls continue through implicit conversion.
CLI fields use ordinary CLASS TypeDef signatures; native fields use existing Named
records, preserved by reference projection. This introduces no new binary schema.
Setting isReadOnly emits ordinary CLI InitOnly and the existing native field_readonly
flag. Executable writes fail with InvalidDataException when an ordinary method or a
constructor of another type stores the field. Declaration/reference projection preserves
these flags. The updated runtime enforces direct stores and returns readonly managed
field addresses outside the declaring constructor; reads and mutation of an object
referenced by a readonly field remain legal. This is shallow storage protection, not
deep immutability or an unsafe-memory sandbox. Raw unmanaged pointers remain outside
managed readonly guarantees. Constructors can acquire writable initialization references.
Development migration: rebuild AddField consumers for the added optional argument and
use the matching runtime; older runtimes do not enforce these flags during execution.
No new binary schema is introduced by this bridge extension.

Nominal stores require the exact declared class; null literals, nullable source
contracts and external class imports are not added by this slice.

Read-only snapshot additions:

```csharp
uint TypeDefinition.Attributes { get; } // physical TypeAttributes
IReadOnlyList<FieldDefinition> TypeDefinition.Fields { get; }
IReadOnlyList<FieldDefinition> ModuleDefinition.Fields { get; }
FieldDefinition? ModuleDefinition.GetFieldDefinition(uint metadataToken);
```

Fields are ordered physical Field rows, with at most 4096 rows and aggregate
signature bytes bounded by the reader's existing 4 MiB limit. GetFieldDefinition
returns null for absent/wrong-kind tokens. FieldDefinition exposes Module,
MetadataToken, DeclaringType, Name and ushort Attributes (FieldAttributes).
GetSignature() returns a new byte array; unsupported encodings remain opaque.
TryGetPrimitiveType(out PrimitiveType type) recognizes exact Int32/Int64/Boolean/
String field signatures, returning false and Void otherwise. No resolution or code
loading occurs. CLI global fields retain the physical module pseudo-type as owner.

```csharp
var order = assembly.AddClass("Example", "Order");
var number = order.AddField("Number", PrimitiveType.Int32);
var pending = order.AddField("Pending", PrimitiveType.Boolean, FieldVisibility.Internal);
```

Native/reference projection preserves class flags, field order/types/access and
module-scoped Field tokens in origin metadata. The bounded native reader rejects
unsupported shapes and inconsistent origin rows. Old readers cannot read new class/
field output. Construction and field instructions are described below.

## Root construction and instance bodies

Development host API (2026-10-01):

```csharp
MethodBuilder TypeBuilder.AddInstanceMethod(string name, MethodSignature signature,
    MethodVisibility visibility = MethodVisibility.Public);
MethodBuilder TypeBuilder.AddConstructor(IEnumerable<PrimitiveType> parameterTypes,
    MethodVisibility visibility = MethodVisibility.Public);
bool MethodBuilder.IsStatic { get; }
bool MethodBuilder.IsConstructor { get; }
void MethodBuilder.Duplicate();
void MethodBuilder.NewObject(MethodBuilder constructor);
void MethodBuilder.LoadField(FieldBuilder field);
void MethodBuilder.StoreField(FieldBuilder field);
void MethodBuilder.Emit(OpCode opcode, FieldBuilder field);
// Added enum values: Dup, Newobj, Ldfld, Stfld.
```

AddInstanceMethod creates a nonvirtual method on a root class. AddConstructor creates
an instance .ctor with Void result. Signatures contain only declared primitive
parameters (maximum 256), excluding the receiver. Argument slot zero holds the exact
declaring-class receiver; declared parameters start at one. Static methods keep their
existing indexing. LoadArgument supports both kinds; StoreArgument rejects receiver
stores when writing. Entry points must be static. Names .ctor/.cctor are reserved in
ordinary AddMethod/AddInstanceMethod; use AddConstructor. Null signatures/parameter
sequences throw ArgumentNullException. Invalid names, access values, duplicate
name/parameter signatures or limits throw ArgumentException; static owners reject
instance declarations with InvalidOperationException before mutation.

Emit(Dup) and Duplicate copy the top stack value, preserving reference identity.
Emit(Newobj, constructor) and NewObject consume declared constructor arguments and
push an owned class reference. Emit(Call, method) and Call consume declared parameters
and, for instance methods, the receiver below them. Direct Call to a constructor and
Newobj to an ordinary method throw ArgumentException. Builder method references retain
the existing external assembly identity/core contract; imported snapshot references
remain static-only.

Emit(Ldfld, field)/LoadField consume the exact declaring-class receiver and push the
primitive or owned nominal field value. Emit(Stfld, field)/StoreField consume receiver then value.
Field handles must belong to the output assembly; foreign fields/wrong opcodes throw
ArgumentException and null operands throw ArgumentNullException before mutation.
Writing rejects stores to readonly fields outside a declaring constructor, stack underflow, mismatched receivers/values/joins and receiver stores
with InvalidDataException; existing instruction/body bounds apply. Access control
remains enforced by the executing target, not a new writer-level access checker.

CLI output uses ordinary HasThis signatures, constructor flags, newobj/dup/ldfld/stfld
and Field tokens. A root constructor's CLI body receives an automatic six-byte
ldarg.0/call System.Object::.ctor prologue; branches target the declared body, never
re-run that prologue. Native roots need no base call, and use the existing instance,
newobj.ctor and field-slot contracts. This is an explicit backend initialization
contract, not arbitrary identical IL bodies or general constructor chaining.
Reference projections preserve instance signatures and constructor flags with throwing
bodies; executable native bodies still reside in the required #Neo payload.

No constructor is synthesized. External inheritance, virtual dispatch, arbitrary constructor chaining,
indexed properties and external instance snapshot imports remain outside this bounded
producer. Owned nominal parameters/results/locals, fields and properties are supported. The C# fixture constructs an Order,
mutates it through one alias and reads through another; both targets return 42.
The separate Raven Order integration now compiles the unchanged declaration; see the
[bridge evidence](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/raven-cli-bridge.md).

## Primitive property associations

Development host API (2026-10-01):

```csharp
IReadOnlyList<PropertyBuilder> TypeBuilder.Properties { get; }
PropertyBuilder TypeBuilder.AddProperty(string name, SignatureType type,
    MethodBuilder? getter = null, MethodBuilder? setter = null);
```

PropertyBuilder exposes DeclaringType (TypeBuilder), Name (string), PropertyType
(SignatureType), GetMethod/SetMethod (nullable MethodBuilder), and IsStatic (bool).
These immutable associations add no storage or bodies. Accessors must already belong
to the same type, be ordinary methods and agree on static/instance shape. The getter
has no declared parameters and returns the property type; the setter takes one value
of that type and returns Void. At least one accessor is required. Accessor visibility
is preserved independently, including private setters. Ordinary accessor names are
allowed; CLI emission adds SpecialName to associated methods.

Non-indexed Int32/Int64/Boolean/String and owned root-class properties are supported.
Nominal types must belong to the output assembly; null throws ArgumentNullException,
and Void/foreign types throw ArgumentException before mutation. Accessor signatures
must use the exact same owned class identity. A unique nonblank
name has at most 1024 characters, no control characters or invalid Unicode. Invalid
contracts, duplicate names, accessors already associated with another property and
more than 256 properties per type throw ArgumentException before mutation. Writing
rejects more than 4096 assembly properties with InvalidDataException. The accessor
reuse/name limits are bounded producer restrictions, not permanent native rules.

CLI output contains Property, PropertyMap and MethodSemantics rows with ordinary
static/HasThis signatures. Native metadata uses existing property/accessor references
and exact origin Property tokens. The reader checks signature, owner, instance shape,
accessor existence, uniqueness and origins before constructing a throwing reference
projection. Outputs without properties retain their previous encoding. New property
outputs require the matching bounded reader. Properties do not imply backing fields;
Indexed and generic-owner properties are supported by the later slices below; external
nominal properties, attributes and default values remain unsupported.

Development API migration (2026-10-01): rebuild consumers of AddProperty and
PropertyBuilder.PropertyType and inspect SignatureType.Primitive/ClassType. Existing
primitive construction calls use implicit conversion. Nominal properties retain standard
CLI CLASS/TypeDef signatures and existing native Named records, including setter
parameter references. Static, read-only, write-only and private accessor associations
are supported without inventing storage or changing the native binary schema.

### Owned property snapshots

```csharp
IReadOnlyList<PropertyDefinition> ModuleDefinition.Properties { get; }
PropertyDefinition? ModuleDefinition.GetPropertyDefinition(uint metadataToken);
IReadOnlyList<PropertyDefinition> TypeDefinition.Properties { get; }
```

The module lists physical Property rows in metadata order; types list their declared
properties. Lookup returns null for absent/wrong-kind tokens. PropertyDefinition
exposes Module, MetadataToken, DeclaringType, Name, ushort Attributes (PropertyAttributes),
nullable GetMethod/SetMethod and `IReadOnlyList<MethodDefinition>` OtherMethods. Accessors
are the same owned objects returned by Module.GetMethodDefinition, with no assembly
loading or resolution. OtherMethods preserves ordinary Other associations.

GetSignature() returns new owned bytes, including unsupported signature encodings.
TryGetPrimitiveSignature(out PrimitiveType type, out bool isStatic) recognizes exact
non-indexed Int32/Int64/Boolean/String signatures; failure returns Void/false. This
helper decodes the property signature, not accessor compatibility. The general CLI
snapshot retains broader signatures opaquely; full CLI verification is not claimed.
Default values/custom attributes are not exposed by this snapshot.

Reading enforces at most 4096 Property rows, 16384 MethodSemantics rows and the shared
4 MiB aggregate signature budget. Missing/ambiguous owners, absent signatures and
accessors outside the declaring type fail with InvalidDataException. Input buffers
and returned signature arrays can be changed without affecting the snapshot.

## Root-class locals

Development host API (2026-10-01):

```csharp
LocalDefinition MethodBuilder.DeclareLocal(TypeBuilder type);
PrimitiveType? LocalDefinition.Type { get; }
TypeBuilder? LocalDefinition.ClassType { get; }
```

A local is either primitive (Type has a value, ClassType is null) or nominal (Type is
null, ClassType is the exact declared root class). **Development API migration:**
LocalDefinition.Type is now nullable; callers that assumed every local was primitive
must branch on Type/ClassType. Existing primitive overloads and slot ordering remain.
No Void sentinel or System.Type handle represents a nominal slot.

The class overload accepts only nonstatic root classes in the same output graph.
Null throws ArgumentNullException; static/foreign classes throw ArgumentException
before mutation. All locals share the existing 256-slot limit (InvalidDataException).
ClearBody preserves local declarations. Ldloc/Stloc and existing helpers preserve
reference identity and require exact declared class identity. Writing rejects wrong
primitive/class stores, different nominal classes and loads before a store on every
reachable path. Inheritance conversions, null constants and external nominal locals
are not admitted.

CLI local signatures use CLASS plus the TypeDef coded index; native locals use the
existing Named type contract. The bounded native reader validates local class ownership
and shape, then omits implementation locals from reference projections as before.
A matching reader is required for these new bodies. API C# tests and direct binary
neoCLR execution validate local aliasing/mutation to 42. The subsequent Raven
consumer validates source-level aliasing too; see the [integration record](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/raven-cli-bridge.md#raven-object-locals-and-aliasing--2026-10-01).


## Nominal signatures

**Development 2026-10-01.** `NeoCLR.Metadata.Experimental.Model` now exposes:

```csharp
public sealed record SignatureType {
    public PrimitiveType? Primitive { get; }
    public TypeBuilder? ClassType { get; }
    public static implicit operator SignatureType(PrimitiveType type);
    public static implicit operator SignatureType(TypeBuilder type);
    public override string ToString();
}
public class MethodSignature {
    public MethodSignature(SignatureType returnType, IEnumerable<SignatureType> parameterTypes);
    public MethodSignature(PrimitiveType returnType, IEnumerable<PrimitiveType> parameterTypes);
    public SignatureType ReturnType { get; }
    public IReadOnlyList<SignatureType> ParameterTypes { get; }
}
MethodSignature MethodBuilder.Signature { get; }
MethodSignature ImportedMethodReference.Signature { get; }
MethodBuilder TypeBuilder.AddConstructor(MethodSignature signature,
    MethodVisibility visibility = MethodVisibility.Public);
```

SignatureType has exactly one representation: a defined primitive (including Void only
for results), an exact nonstatic TypeBuilder identity, or a bounded vector (see
[vector declarations](#vector-declarations-development-2026-10-01)). Invalid primitive values and
static classes throw ArgumentException; a null class throws ArgumentNullException.
Its diagnostic ToString is not a persistent identity. Record equality retains exact
builder identity. MethodSignature copies at most 256 parameters; null input throws
ArgumentNullException and null/Void parameters or too many parameters throw
ArgumentException. Returned lists cannot be modified. Constructors require Void results.

AddFunction (both global and namespaced overloads), AddMethod and AddInstanceMethod
now take MethodSignature instead of PrimitiveMethodSignature. Before mutating declarations
they reject class types from another builder, including another builder with the same
assembly identity. Existing visibility, duplicate-parameter-signature and count limits
remain. Method return type does not distinguish overloads. PrimitiveMethodSignature now
derives from MethodSignature and retains its primitive-typed ReturnType/ParameterTypes
properties; primitive consumers can continue constructing it. **Development API migration:**
rebuild consumers against the changed method signatures. Code inspecting MethodBuilder
or ImportedMethodReference signatures must use `.Primitive` or `.ClassType`, rather than
assuming a primitive enum. Imported read-only method contracts accept primitives and primitive vectors; nominal imports remain unsupported.

CLI output uses CLASS TypeDef signatures; native output uses existing Named type records.
Calls, argument stores and returns enforce exact class identity, just like nominal locals.
No implicit base conversion, null literal or structural
signature support is added. Cross-assembly builder calls with nominal signatures reject
at write rather than fabricating a TypeRef. Entries remain parameterless Int32/Void.
NativeAssemblyDefinition accepts owned nonstatic Named signature references and remaps
them into each reference projection. MethodDefinition preserves CLI blobs, while its
primitive recognizers correctly decline nominal signatures. Reference-only output still
contains throwing placeholder bodies and must not be executed as the native implementation.

Compiled consumer pattern (covered by NominalSignatureChecks):

```csharp
var identity = graph.AddFunction("Identity", new MethodSignature(order, [order]));
identity.LoadArgument(0);
identity.Return();
var self = order.AddInstanceMethod("Self", new MethodSignature(order, []));
self.LoadArgument(0);
self.Return();
```

Here `order` is graph.AddClass's owned result. The C# contract test passes an allocated
instance through both calls, mutates it through a nominal parameter and reads 42 through
its alias on .NET and binary neoCLR. Wrong-class arguments/results and foreign signatures
reject. Raven additionally validates owned nominal constructor parameters and overloads.

## Vector declarations (development, 2026-10-01)

```csharp
SignatureType SignatureType.ArrayOf(SignatureType elementType);
SignatureType? SignatureType.ArrayElement { get; }
LocalDefinition MethodBuilder.DeclareLocal(SignatureType type);
SignatureType LocalDefinition.SignatureType { get; }
```

`ArrayOf` creates a one-dimensional zero-based vector of Int32, Int64, Boolean,
String, an owned non-static root class or another vector (jagged arrays). Null throws
ArgumentNullException; Void, by-reference elements and nesting beyond the existing
16-level signature bound throw ArgumentException. Declaration APIs reject foreign element
owners before mutation. Method, field and property signatures share this contract;
`DeclareLocal` also rejects Void and enforces the existing 256-slot limit.
`SignatureType.ArrayElement` is null for scalar types. `LocalDefinition.Type` and
`ClassType` are both null for vectors; use `SignatureType` to inspect all slot kinds.
For example, `method.DeclareLocal(SignatureType.ArrayOf(PrimitiveType.Int32))`
creates an integer-array slot. Stack joins, calls and stores require exact element
identity; covariance is not admitted by this bounded writer.

CLI output uses standard SZARRAY signatures; native output uses ArrayRef, preserving
reference identity. The native reader validates elements and reconstructs the same
CLI signatures for reference assemblies. These host-only C# APIs remain covered by
this manual reference, not the RavenDoc runtime snapshot. Allocation/indexing APIs are described below. C# contract tests execute array
parameter/result/local and property/field aliasing on .NET and compare native projections.

### Vector body operations

```csharp
void MethodBuilder.Emit(OpCode opCode, SignatureType elementType);
void MethodBuilder.NewArray(SignatureType elementType);
void MethodBuilder.LoadArrayElement(SignatureType elementType);
void MethodBuilder.StoreArrayElement(SignatureType elementType);
void MethodBuilder.LoadArrayLength();
```

The typed Emit overload accepts Newarr, Ldelem and Stelem with scalar element types
accepted by ArrayOf. Wrong opcode/element/ownership throws ArgumentException; null
throws ArgumentNullException, with no body mutation. Helpers have the same validation.
Allocation consumes Int32 length and pushes a reference array with default-initialized
elements. Loads consume array/index and push an element; stores consume array/index/value.
Exact element identity is checked at write time (InvalidDataException); bounds and
negative lengths fault at execution. New arrays of reference elements contain null
references until initialized. The current API does not expose null literals.

Operand-free Emit(OpCode.Ldlen) consumes an array and pushes native unsigned length,
which the bounded flow checker admits into Conv_I4, duplication or discard. Returning
it as Int32 without conversion rejects. LoadArrayLength appends ldlen/conv.i4, checks
the instruction limit before appending either, and produces Int32. CLI uses standard
newarr/ldelem/stelem type tokens and ldlen; native uses the corresponding typed operations.
One scalar primitive TypeRef is cached per output kind. No nested arrays, spans,
covariance, element addresses or imported nominal elements are admitted. C# tests
verify CLI execution and API-produced binary verification/execution on neoCLR.

## Indexed property associations (development, 2026-10-01)

`TypeBuilder.AddProperty` now also accepts indexed getter/setter methods. Its existing
signature is unchanged. `PropertyBuilder.ParameterTypes : IReadOnlyList<SignatureType>`
exposes a copied immutable list of index types, excluding receiver and setter value.
For a getter, all parameters are indices; for a setter-only property, all but its last
parameter are indices. The final setter parameter must equal PropertyType and its
result must be Void. Both accessors must agree on index types and instance/static
shape. Primitive, owned-class and vector index types follow MethodSignature rules.
A getter may have 256 indices; a setter leaves at most 255 within the method limit.

Properties overload by name plus exact index parameter sequence; return type alone
does not distinguish overloads. Duplicate signatures, reused accessors and inconsistent
getter/setter signatures throw ArgumentException without adding a property. This
relaxes the former name-only uniqueness check. Rebuild development host consumers to
use ParameterTypes. Ordinary non-indexed properties have an empty list.

CLI output encodes standard Property signatures/MethodSemantics; native output uses
the existing property parameters list and accessor function references. The native
reader validates these lists and reproduces CLI reference signatures. PropertyDefinition
retains the full signature through GetSignature; its non-indexed TryGetPrimitiveSignature
helper deliberately returns false for indexers. No DefaultMemberAttribute is synthesized:
Raven source binding already identifies its indexers, while other CLI compilers may need
that attribute to recognize source-level indexing. Native introspection's no-index
GetValue/SetValue convenience calls are not expanded by this producer change.

Example: define `GetItem(Int32) -> Int32` and `SetItem(Int32, Int32) -> Void`, then
`owner.AddProperty("Item", PrimitiveType.Int32, get, set)`. The resulting ParameterTypes
contains one Int32. The same name may have a separate Int64-index getter. C# tests
execute these accessor associations on .NET and inspect native reference projections.

The C# `--indexer-integration <runtime> <fresh-output>` check also writes a binary
assembly with overloaded getters and a two-index setter-only association. neoCLR
loads and verifies its property contracts and executes the accessor calls to 42.
This uses existing runtime metadata validation and call instructions, not an indexer
opcode or a new runtime introspection invocation API.

## Generic method declarations (development, 2026-10-01)

```csharp
SignatureType SignatureType.MethodParameter(int index);
int? SignatureType.MethodParameterIndex { get; }
MethodSignature(SignatureType returnType, IEnumerable<SignatureType> parameterTypes,
    IEnumerable<string>? genericParameterNames = null);
IReadOnlyList<string> MethodSignature.GenericParameterNames { get; }
```

MethodParameter creates a positional method type reference (MVAR); indices outside
0–31 throw ArgumentOutOfRangeException. Declaration/local/instruction use checks the
index against the current method's declared arity and throws ArgumentException for
invalid scope. Fields and property accessors cannot use method generic parameters.
The signature constructor copies up to 32 unique nonblank valid-Unicode names of at
most 256 characters without controls; invalid names/counts throw ArgumentException.
Names are immutable and preserved in CLI GenericParam rows and native generic_parameters.
Unconstrained ownerless functions and static/ordinary instance methods on owned root
classes are admitted. Generic constructors, generic types and constraints remain outside
this API slice.

For example, `new MethodSignature(SignatureType.MethodParameter(0),
[SignatureType.MethodParameter(0)], ["T"])` defines `Identity<T>(T) -> T`.
Method generic parameters also work in locals and vector elements. CLI uses MVAR and
TypeSpec operands for typed array instructions; native uses MethodTypeParameter.
Direct Call/Emit(Call, MethodBuilder) on an open generic definition throws
ArgumentException. Generic entries are invalid. Overload uniqueness includes generic
arity; return type still does not distinguish methods. MethodDefinition.GenericArity
and raw signatures retain generic declarations in native CLI reference projections;
primitive-only recognizers continue to decline them. This extends the development
MethodSignature constructor; rebuild host consumers.


### Instantiated generic calls (development)

`MethodBuilder.MakeGenericInstance(params SignatureType[] typeArguments)` returns an
immutable `GenericMethodInstance` exposing `Definition`, copied `TypeArguments` and
substituted `Signature`. `Call(GenericMethodInstance)` and
`Emit(OpCode.Call, GenericMethodInstance)` append an owned call. Null arguments throw
ArgumentNullException; wrong arity, Void, foreign class/definition, wrong opcode,
out-of-scope caller parameters and nested-array substitution throw ArgumentException.
Body stack compatibility is validated on write. Only unconstrained static or ordinary instance definitions
in the current output are supported. Bounded static imported methods now use the separate
[imported generic contract](#imported-generic-methods-development-2026-10-01).
For example, `body.Call(identity.MakeGenericInstance(PrimitiveType.Int32))` consumes
one Int32 for `Identity<T>(T)->T` and produces Int32. Forwarding may instead pass
`SignatureType.MethodParameter(0)` from a caller that declares that parameter.
CLI output uses cached MethodSpec records; native calls carry explicit generic arguments
and substituted parameter types. The binary integration test verifies forwarding,
Int32/Int64/Boolean instantiations, owned-object identity and generic vector creation/access.

Generic-call regression checks also confirm that caller mutation of the argument array
cannot alter an existing instance, and both writers reject a mismatched substituted
stack argument before producing an image. Raw `Emit(Call, instance)` supports forwarded
parameters with the same scope and stack contract as `Call(instance)`.

Instance generic calls consume the exact owned declaring-class receiver before explicit
arguments and return the substituted result. Receiver slot zero remains separate from
MVAR parameter indices. `AddInstanceMethod(name, signature)` accepts named method
parameters; `MakeGenericInstance`, typed `Call` and raw `Emit(Call, instance)` work as
for static calls. No virtual-dispatch contract is introduced. Both writers validate
receiver stack shape; native reference projection preserves instance/generic flags.
Constructors and property accessors cannot declare their own method generic parameters.

### Typed local initialization (development)

`OpCode.Ldloca` accepts an owned `LocalDefinition` or local index; `OpCode.Initobj`
accepts `SignatureType`. `MethodBuilder.LoadLocalAddress(LocalDefinition)` and
`InitializeObject(SignatureType)` expose the same operations. The address may designate
an uninitialized local; initialization establishes definite assignment only for that
exact local. Both writers reject a mismatched type, non-address operand, an address
escaping through value storage/calls/returns, and later loads not initialized on every
reachable path. Writable managed-reference parameters are supported as described below. Address values at joins must identify the same local.

`OpCode.Ldobj` and `OpCode.Stobj` accept an exact non-Void `SignatureType` through
`Emit(OpCode, SignatureType)`. The corresponding helpers are
`MethodBuilder.LoadObject(SignatureType type)` and `StoreObject(SignatureType type)`.
Load consumes an owned local address and pushes its value; store consumes an address
followed by the value and leaves no result. Store establishes definite assignment for
that local; load requires assignment on every incoming path. Primitive, nominal,
vector and scoped generic local types follow the existing signature rules. A null
operand throws ArgumentNullException; Void, invalid scope or foreign ownership throws
ArgumentException before appending. Instruction limits and invalid stack/address/type
or assignment contracts throw InvalidDataException (the latter on writing).
CLI uses standard ldobj/stobj tokens, including TypeSpec for generic/vector operands;
native emits its existing typed ldobj/stobj. This does not admit pointers, field/array
addresses or escaping references. Ldobj/Stobj also accept managed-reference parameters. See `LocalObjectChecks.cs` for an
executable generic copy and branch-merged local update.

`MethodBuilder.LoadDefault(SignatureType)` declares one scratch local, initializes it
and loads its value (three instructions). Types may be primitives, owned root classes,
vectors or scoped method parameters; Void, foreign owners and invalid method scope
throw ArgumentException. Null arguments throw ArgumentNullException. Local/instruction
limits throw InvalidDataException before this helper changes the body; ClearBody retains
scratch declarations. Invalid stack/definite-assignment contracts fail during writing.
CLI uses ldloca/initobj/ldloc with TypeSpec for generic/vector operands; native uses the
same logical operations. Numeric defaults are zero, Boolean false, and reference
(string/class/vector) defaults are typed null references. This does not add nullable
source syntax or general pointer/byref APIs.

### Static generic owners (development)

`AssemblyBuilder.AddGenericType(string namespace, string name,
IEnumerable<string> genericParameterNames, TypeVisibility visibility = Public)` creates
an owned static class with one through 32 copied, unique parameter names. It appends
the CLI arity suffix to Name (for example Helpers`1); the input simple name must not
contain a backtick. Invalid/duplicate names or limits throw ArgumentException; null
parameter sequences throw ArgumentNullException. `TypeBuilder.GenericParameterNames`
is an immutable ordinal list. AddGenericType creates static owners; AddGenericClass
(below) creates instance owners. Both can associate properties using scoped VAR.

`SignatureType.TypeParameter(int index)` and nullable `TypeParameterIndex` represent
VAR independently from method MVAR. Index bounds are 0–31 (ArgumentOutOfRangeException);
method signatures, locals and typed instruction operands validate the declaring-type
scope. Assembly functions cannot use VAR. Entries cannot belong to a generic owner.

`MethodBuilder.MakeConstructedReference(IEnumerable<SignatureType> declaringTypeArguments,
IEnumerable<SignatureType>? methodArguments = null)` binds a generic owner and
all method parameters together. It returns immutable `ConstructedMethodReference`
with `Definition`, copied `DeclaringTypeArguments`, copied `MethodArguments` and the
simultaneously substituted `Signature`. Null owner argument sequences throw
ArgumentNullException. Wrong arity, Void/null/foreign arguments, nongeneric
owners or nested array substitution throw ArgumentException. Supplied caller parameters
are validated in the caller scope when emitted; substitution does not capture them.
`Call(ConstructedMethodReference)` and `Emit(OpCode.Call, ConstructedMethodReference)`
require an owned target and supported caller scope; null throws ArgumentNullException,
invalid opcode/ownership/scope throws ArgumentException, and instruction/stack failures
use InvalidDataException. Open-owner Call and MakeGenericInstance are rejected: bind
owner and method arguments together even when the method itself is nongeneric.

CLI uses GenericParam/VAR and a MemberRef on a constructed TypeSpec, optionally wrapped
in MethodSpec. Native definitions use open Constructed owners and explicit TypeParameter
ordinals; calls substitute owner and method arguments independently. The native reader
retains arities/names and validates open ownership before creating a reference projection.
These are owned-output references; external method imports use the separate bounded
[imported generic contract](#imported-generic-methods-development-2026-10-01).


### Generic reference classes (development)

`AssemblyBuilder.AddGenericClass(string namespace, string name,
IEnumerable<string> genericParameterNames, TypeVisibility visibility = Public)` uses
AddGenericType's naming, ownership, arity and error rules, but creates a nonabstract,
nonsealed reference class rooted at System.Object in CLI. No constructor is synthesized.

`TypeBuilder.MakeGenericInstance(params SignatureType[] typeArguments)` returns an
immutable `GenericTypeInstance` with `Definition` and copied `TypeArguments`.
`Equals(GenericTypeInstance?)`, `Equals(object?)` and `GetHashCode()` compare the exact
owned definition and arguments structurally; `ToString()` is diagnostic only.
Null arrays throw ArgumentNullException; static/nongeneric owners, arity mismatch,
null/Void/foreign arguments or nesting beyond 16 levels throw ArgumentException.
Implicit conversion to SignatureType and `SignatureType.GenericInstance` preserve
this identity in parameters, results, fields, locals, arrays and typed initialization.
Bare generic TypeBuilder-to-SignatureType conversion is rejected: construct the owner,
including with VAR arguments for an open self-reference. Scope is checked on use.

AddField now accepts declaring-type VAR and constructed owned classes (including
nested constructions). MVAR in fields is rejected. Generic definition field handles
can be emitted only inside their declaring type; constructed external field references
are deferred. This keeps CLI/native field access equivalent without losing the owner
instantiation. Readonly fields retain declaring-constructor store checks.

MakeConstructedReference now accepts instance methods and constructors as well as
static methods. `NewObject(ConstructedMethodReference)` and
`Emit(OpCode.Newobj, ConstructedMethodReference)` consume substituted constructor
parameters and produce the exact constructed class. Call is required for other
methods, Newobj for constructors; wrong opcodes, ownership or caller scope throw
ArgumentException, null throws ArgumentNullException, and instruction/stack failures
throw InvalidDataException. Instance calls require an exactly matching constructed
receiver. Constructor chaining remains unsupported. Generic method parameters remain
independent of the declaring type, including nested signature substitution.

CLI emits standard GENERICINST, VAR and MemberRef/MethodSpec; native output uses the
existing Constructed type and field contracts. The native declaration reader validates
field scope and preserves generic fields/constructed signatures in its non-executable
CLI reference projection. Constraints, inheritance, external
owners and external constructed field handles remain outside this bounded API.


### Properties on generic owners (development)

`TypeBuilder.AddProperty` now accepts declaring-type parameters in the value and
index signatures, including vectors and constructed class signatures containing VAR.
Both static and instance owners retain their ordinary accessor shape. Accessors cannot
declare method generic parameters; MVAR, out-of-range VAR, mismatched owner/result/index
signatures and reused accessors throw ArgumentException before adding the association.
No storage or bodies are synthesized by AddProperty; index signatures still come from
the associated methods and visibility stays on each accessor.

CLI keeps Property/PropertyMap/MethodSemantics and VAR signatures. Native metadata uses
canonical open Constructed owners on accessor references. The reader validates exact
owner identity, arity and ordered VAR arguments, resolves each accessor under the
property's declaring-type scope, and preserves both property signatures and associations
in its reference projection. Malformed owner references or scope violations throw
InvalidDataException. Use the matching reader for these new producer outputs; the
runtime already supports the native representation, so no runtime schema migration is
required. This adds no external property import or generic constraints.


### Constructed fields (development)

`FieldBuilder.MakeConstructedReference(params SignatureType[] typeArguments)` returns
an immutable `ConstructedFieldReference` with `Definition`, `DeclaringType` (a
GenericTypeInstance) and substituted `FieldType`. Arguments are copied; original
field signatures stay open. Constructor arguments follow MakeGenericInstance's bounds,
ownership and 16-level nesting rules. Null throws ArgumentNullException; wrong arity,
invalid/foreign types or unsupported substitution throws ArgumentException.

`MethodBuilder.LoadField(ConstructedFieldReference)`, `StoreField(...)` and
`Emit(OpCode, ConstructedFieldReference)` accept Ldfld/Stfld. Caller VAR/MVAR scope
and ownership are checked before appending; invalid opcode/scope throws ArgumentException,
null throws ArgumentNullException, and instruction limits throw InvalidDataException.
Writing checks the exact constructed receiver, substituted value type and existing
readonly constructor rule. CLI emits a Field MemberRef on a constructed TypeSpec with
the open field signature. Native field indices retain the runtime's constructed receiver
identity and layout. External *assembly* imports remain unsupported; this API accesses
owned fields from outside their declaring type and supersedes that earlier restriction.


### Nominal type constraints (development)

`TypeBuilder.AddBaseTypeConstraint(int parameterIndex, TypeBuilder baseType)` adds one
owned nongeneric nonstatic root-class bound to a declared type parameter. Null bounds
throw ArgumentNullException; invalid indices, repeated parameters, foreign/static/generic
bounds throw ArgumentException. `GenericConstraints` exposes read-only
`GenericTypeConstraint` snapshots with `ParameterIndex` and `BaseType`; the public record
constructor creates a value only and does not add a constraint to a definition.

CLI uses GenericParamConstraint; native uses TypeBound. Within this root-class producer
subset, concrete arguments must be the bound class itself (inheritance/interfaces are
not representable yet). Symbolic VAR/MVAR arguments are deferred to runtime substitution.
Constructing an invalid concrete reference throws ArgumentException; writing revalidates
previous uses after added bounds and throws InvalidDataException. The native reader
validates bound ordinals/identity and preserves bounds in the CLI reference projection.
Malformed bound metadata throws InvalidDataException. Existing unconstrained outputs
retain their encoding; constrained outputs need the matching reader.

This is a nominal bound, not a class/struct/new()/notnull flag. Those categories,
method constraints, interface/dependent bounds and dispatch through an open constrained
parameter remain separate contracts. The runtime already enforces nominal TypeBound;
no schema change is needed for this slice.


### Special type-parameter requirements (development)

`NeoCLR.Metadata.Experimental.Model.TypeParameterConstraints` is a flags enum:
`None = 0`, `ReferenceType = 4`, `ValueType = 8`, `DefaultConstructor = 16`.
`ReferenceType` requires a managed reference (including String and vectors);
`ValueType` requires a supported nonnullable value. `DefaultConstructor` accepts a
value type or a concrete class with a public parameterless instance constructor.
The producer currently admits Int32, Int64 and Boolean value arguments.

```csharp
public IReadOnlyDictionary<int, TypeParameterConstraints> TypeBuilder.SpecialConstraints { get; }
public void TypeBuilder.SetSpecialConstraints(int parameterIndex, TypeParameterConstraints constraints);
```

The read-only view maps zero-based declared parameter ordinals to requirements.
`SetSpecialConstraints` replaces that ordinal's special flags; `None` clears them
without removing a nominal bound. Invalid ordinals, unknown bits, reference/value
combinations and value/nominal-class-bound combinations throw `ArgumentException`.
A reference or constructor flag may coexist with a nominal bound. Set both ValueType
and DefaultConstructor for Raven's `struct` declaration. No method flags are exposed.

```csharp
var owner = assembly.AddGenericClass("Example", "Box", ["T"]);
owner.SetSpecialConstraints(0, TypeParameterConstraints.ReferenceType |
    TypeParameterConstraints.DefaultConstructor);
```

Concrete arguments are checked when constructed, except class constructor existence
is deferred until writing so definition order does not matter. Final graph validation
rejects an unsatisfied requirement with `InvalidDataException`, including requirements
added after an earlier use. Symbolic arguments defer concrete checks to runtime
substitution. Flags do not enable construction or constrained dispatch on symbolic T.

CLI uses ordinary GenericParam attributes; native metadata has distinct ReferenceType,
ValueType and DefaultConstructor constraint kinds. The declaration reader rejects
unknown, duplicate or conflicting flags and preserves accepted flags in reference
projections. Native execution needs the matching feature-branch runtime; old runtimes
cannot decode these new kinds. Native notvoid/notreference keep their prior meaning.
See the [integration assessment](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/state-assessment-2026-10-01.md).


### Interface declarations (development)

`NeoCLR.Metadata.Experimental.Model` exposes:

```csharp
public TypeBuilder AssemblyBuilder.AddInterface(string @namespace, string name,
    TypeVisibility visibility = TypeVisibility.Public);
public TypeBuilder AssemblyBuilder.AddGenericInterface(string @namespace, string name,
    IEnumerable<string> genericParameterNames, TypeVisibility visibility = TypeVisibility.Public);
public bool TypeBuilder.IsInterface { get; }
public IReadOnlyList<TypeBuilder> TypeBuilder.BaseInterfaces { get; }
public void TypeBuilder.AddBaseInterface(TypeBuilder baseInterface);
public MethodBuilder TypeBuilder.AddInterfaceMethod(string name, MethodSignature signature);
public bool MethodBuilder.IsAbstract { get; }
```

The factories return owned interfaces with public/internal visibility. Generic names
are copied (1–32 distinct names); CLI arity is appended to the supplied simple name.
Ordinary type identity/4,095-type limits apply. Invalid identities, visibility, names or
duplicates throw ArgumentException; null generic names throw ArgumentNullException.
Interfaces are invariant in this API. AddBaseInterface admits directly inherited,
owned nongeneric interfaces; generic base instantiations are not exposed yet.

AddInterfaceMethod creates a public abstract instance contract, with supported
primitive/owned-class/array/declaring-type-parameter signatures. A null signature throws
ArgumentNullException; duplicate/invalid signatures, reserved constructor names,
method-level generics or per-owner 256-method limits throw ArgumentException.
Calling it on a class, or ordinary AddMethod/AddInstanceMethod/AddConstructor on an
interface, throws InvalidOperationException. Interfaces cannot own storage fields; AddField throws InvalidOperationException.
AddProperty now associates abstract getter/setter declarations, including declaring-type
parameter values and index signatures, using the same validation as class properties.
Owned interface references and constructed interface values are admitted as storage,
parameter and result signatures, including vector elements and typed defaults. The
existing SignatureType.ClassType property denotes a CLI CLASS identity and can refer
to a class or interface. MakeGenericInstance now accepts invariant generic interfaces
with the existing arity, scope, ownership and argument validation. Interfaces remain
invalid nominal class bounds and allocation targets. A signature alone does not declare an implementation; the bounded dispatch API is
described below.

```csharp
var comparer = assembly.AddGenericInterface("Example", "Comparer", ["T"]);
var t = SignatureType.TypeParameter(0);
var compare = comparer.AddInterfaceMethod("Compare",
    new MethodSignature(PrimitiveType.Int32, [t, t]));
// compare.IsAbstract is true; no instructions or locals belong to this declaration.
```

Writing rejects abstract methods with instructions/locals and direct calls to them
with InvalidDataException. Direct interface dispatch and implicit nongeneric implementations are now supported
as described below; default/static interface methods remain subsequent work. An abstract method has no CLI
body (RVA zero), including in a reference projection: no throwing placeholder is used.
CLI uses Interface/Abstract type flags with no base class, and public abstract virtual
new-slot method flags. These are ordinary CLI contracts, not extensions; see
[ECMA-335](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
Partition II, interface and method-definition rules.

The existing native Interface representation carries contract identity. Its separate
type is_abstract field applies to records and remains false; methods carry explicit
abstract/virtual flags and empty bodies. No runtime format or instruction change is
needed. The native reader validates this shape and preserves interfaces through CLI
reference projection. Unknown/mismatched flags, bodies/locals, storage and invalid
owner contracts reject with InvalidDataException. Matching reader/producers are
required; older experimental readers reject the additional declaration category.


Interface inheritance and properties use standard InterfaceImpl, Property and
MethodSemantics metadata. Abstract accessor methods also carry SpecialName and no
body. AddBaseInterface returns no value; it adds an edge visible through the read-only
BaseInterfaces view. Null throws ArgumentNullException; a noninterface owner throws
InvalidOperationException. Foreign, noninterface or generic bases, duplicate edges,
cycles and more than 256 direct bases throw ArgumentException. Interface declaration
order does not matter. No class-implements-interface contract is implied by this API.

```csharp
var disposable = assembly.AddInterface("Example", "Disposable");
disposable.AddInterfaceMethod("Dispose", new MethodSignature(PrimitiveType.Void, []));
var iterator = assembly.AddGenericInterface("Example", "Iterator", ["T"]);
iterator.AddBaseInterface(disposable);
var current = iterator.AddInterfaceMethod("get_Current",
    new MethodSignature(SignatureType.TypeParameter(0), []));
iterator.AddProperty("Current", SignatureType.TypeParameter(0), current);
```

Native reading validates inherited identities, cycles and accessor associations and
preserves them in reference projection. Property-only declarations with zero fields
may include the writer's empty field-origin arrays; all three arrays must be present
and empty when that form is used. Partial/inconsistent field-origin metadata rejects.


### Interface implementation and dispatch (development)

```csharp
IReadOnlyList<TypeBuilder> TypeBuilder.ImplementedInterfaces { get; }
void TypeBuilder.AddInterfaceImplementation(TypeBuilder contract);
void MethodBuilder.CallVirtual(MethodBuilder target);
void MethodBuilder.Emit(OpCode.Callvirt, MethodBuilder target);
```

AddInterfaceImplementation declares an owned nongeneric interface on a nongeneric
root class. Its inherited nongeneric contracts are included. The list is read-only;
null throws ArgumentNullException. Static/interface/generic owners throw
InvalidOperationException; foreign/generic/noninterface contracts, duplicates or more
than 256 direct contracts throw ArgumentException. Writing requires every inherited
contract to have a public instance method with the same name, parameter and result
signature; missing or incompatible members throw InvalidDataException. Member order
does not matter. Explicit MethodImpl mappings are not yet exposed. CLI implementations
are virtual/final/new-slot; native implicit matching uses the existing runtime rules.

CallVirtual and raw Emit(Callvirt, target) consume a receiver followed by declared
arguments, dispatch to its implementation and push any result. Targets must be owned
nongeneric abstract interface methods. Null targets throw ArgumentNullException;
foreign/generic/noninterface targets throw ArgumentException without appending an
instruction. The existing instruction limit throws InvalidDataException. Stack checking
accepts exact references and declared interface upcasts, including inherited contracts;
unrelated receivers reject when writing. Arrays remain invariant. Null references can
be stored but fault when dispatched; no exception-handling instructions are introduced.
Direct Call to an abstract contract still rejects. CLI uses callvirt 0x6f; native output
uses the existing callvirt opcode. Reference projections preserve implementations but
retain placeholder bodies and must not be executed as native code.

```csharp
var contract = assembly.AddInterface("Example", "Value");
var get = contract.AddInterfaceMethod("Get", new MethodSignature(PrimitiveType.Int32, []));
var concrete = assembly.AddClass("Example", "Answer");
concrete.AddInterfaceImplementation(contract);
var implementation = concrete.AddInstanceMethod("Get", new MethodSignature(PrimitiveType.Int32, []));
implementation.LoadConstant(42);
implementation.Return();
// In a body with a Value-typed receiver on the stack:
body.CallVirtual(get);
```

C# contract/integration tests exercise two implementations, inherited contracts,
reference arguments and null faults on .NET and the binary neoCLR loader. Generic
interface instances are legal signatures but not yet legal targets of this dispatch
API. Default/static interface members and class virtual overrides remain separate work.


## Imported primitive vectors (development, 2026-10-01)

```csharp
bool MethodDefinition.TryGetStaticValueSignature(out MethodSignature? decoded);
```

Recognizes static nongeneric methods with Int32, Int64, Boolean or String scalar or
one-dimensional zero-based vector parameters/results, plus Void only as a scalar
result. Success returns an immutable copied `MethodSignature`; malformed or unsupported
encodings return false with null. Parameter counts are canonical CLI compressed integers,
bounded to 256. Exact consumption rejects trailing/truncated bytes, Void elements,
jagged/multidimensional arrays, byrefs, nominal/generic types and other conventions.
This reads declarations only; it neither resolves types nor validates bodies.

`AssemblyBuilder.ImportReference` and `MemberReference.ResolveMethod` now use this
contract and distinguish full vector element signatures when resolving overloads.
The existing explicit dependency/core identity, ownership and resolver restrictions
remain. For example, a read-only `Identity(int[]) -> int[]` definition can be imported,
called with `MethodBuilder.Call`, and emitted as ordinary CLI AssemblyRef/TypeRef/MemberRef
metadata or as a native assembly. Primitive-only recognition APIs remain narrower.

Vectors retain .NET CLI `SZARRAY` signatures; no extension or semantic divergence is
introduced. Imported nominal and generic signatures still need a separate identity
contract. The C# vector checks execute ordinary CLR library/application images and
check exact overload resolution, native declaration projection and malformed encodings.
The Raven [runtime probe](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/vector-library-validation.json)
executes independently emitted native library/application images with the neoCLR profile.


## Imported generic methods (development, 2026-10-01)

```csharp
bool MethodDefinition.TryGetStaticGenericValueSignature(out MethodSignature? decoded);
ImportedGenericMethodReference ImportedMethodReference.MakeGenericInstance(params SignatureType[] typeArguments);

public sealed class ImportedGenericMethodReference {
    public ImportedMethodReference Definition { get; }
    public IReadOnlyList<SignatureType> TypeArguments { get; }
    public MethodSignature Signature { get; }
}
void MethodBuilder.Call(ImportedGenericMethodReference method);
void MethodBuilder.Emit(OpCode opCode, ImportedGenericMethodReference operand);
```

The recognizer returns an immutable signature for static unconstrained methods with
1–32 method parameters, primitives, method parameters (`MVAR`) and single vectors of
those types. Generic parameter names in this callable view are positional `T0`, `T1`,
etc.; the original image remains unchanged. The generic arity must agree with the
GenericParam table, whose indices must be consecutive and whose flags/constraints
must be empty. Nongeneric, constrained, out-of-scope, noncanonical or malformed
signatures return false/null. Declaring-type parameters and nominal types are outside
this subset. Existing nongeneric recognizers keep their narrower behavior.

ImportReference retains this declaration contract; MemberReference.ResolveMethod
also matches its generic arity and exact parameter/result types. Instantiation copies
one non-Void consumer-scoped signature type per method parameter, including owned or
imported reference types, constructions, supported vectors and caller generic parameters.
Null arrays throw ArgumentNullException; wrong arity, foreign type ownership, a
nongeneric definition or substitutions producing a nested vector throw ArgumentException.
Definition exposes the immutable imported reference, never a mutable dependency builder.
TypeArguments is the copied ordered sequence; Signature substitutes method parameters
and may retain caller-scoped parameters until emission.

Call and raw Emit(Call, reference) append the same generic call. Null references throw
ArgumentNullException; wrong opcodes, consuming owners or caller generic scopes throw
ArgumentException. Method/declaring-type parameter indices are checked against the
caller at emission. An uninstantiated generic ImportedMethodReference cannot be called. Instruction limits
and stack/type mismatches continue to throw InvalidDataException on append/write.
CLI output uses a standard MethodSpec targeting an external MemberRef. Native output
uses the existing generic_arguments call contract; there is no format or opcode change.

For example, import `First<T>(T[]) -> T`, instantiate with PrimitiveType.Int32, then
pass an Int32 vector to Call. The C# tests execute the resulting separate library and
application on the CLR. Raven's native-profile [generic library probe](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/generic-library-validation.json)
verifies and runs the equivalent binary boundary in neoCLR, including consumer-owned
nominal arguments, alias mutation, external constructions and caller method/owner
parameter forwarding. Imported generic declaring types, cross-dependency nominal types in the imported
definition signature and constraints remain unsupported; this is not general collection
import support.

### Lifecycle direction reaffirmed 2026-10-01

.NET metadata remains the baseline; neoCLR extensions are explicit. This Cecil-like
library is intended to support inspection, modification, creation from nothing and
writing through familiar definition/reference concepts. Current loaded snapshots are
immutable and preserve original bytes; producer graphs are editable. General editing
of arbitrary loaded assemblies through one model is not implemented yet. The bounded
import APIs described above must not be read as a permanent split or a new metadata
format. Assembly-level functions already exist, with a CLI `<Module>` compatibility
representation; new generic imports add no further format extension.


## Imported type signatures (development, 2026-10-01)

```csharp
ImportedTypeReference AssemblyBuilder.ImportReference(TypeDefinition definition, AssemblyIdentity dependencyCoreLibrary);

public sealed class ImportedTypeReference : IEquatable<ImportedTypeReference> {
    public AssemblyBuilder Owner { get; }
    public AssemblyIdentity AssemblyIdentity { get; }
    public string Namespace { get; }
    public string Name { get; }
    public int GenericArity { get; }
    public IReadOnlyList<SignatureType> TypeArguments { get; }
    public ImportedTypeReference MakeGenericInstance(params SignatureType[] arguments);
    public bool Equals(ImportedTypeReference? other);
    public override bool Equals(object? obj);
    public override int GetHashCode();
    public override string ToString();
}
ImportedTypeReference? SignatureType.ImportedType { get; }
public static implicit operator SignatureType(ImportedTypeReference type);
```

ImportReference copies the exact assembly identity, namespace, metadata name and arity
from a public top-level reference class/interface declaration. It interns definition
references per consumer and rejects conflicting dependency MVIDs. The explicit core
identity must match the consumer. Value types, nested/static/nonpublic types, signed
or flagged dependencies, self imports, constrained/variant generic definitions and
arities above 32 are unsupported. Generic indices must be consecutive. Unsupported
contracts and limits throw InvalidDataException; null inputs throw ArgumentNullException.
The consumer admits at most 4096 imported type names and 256 dependency identities.
The overload addition means a null literal in ImportReference needs an explicit
MethodDefinition or TypeDefinition cast to select the intended overload.

A nongeneric reference converts directly to SignatureType. A generic definition first
needs MakeGenericInstance, which copies one non-Void consumer-scoped argument per
parameter. Arguments may include the consumer's own classes, other imported reference
types and scoped generic parameters; scope is checked at signature use. Depth is limited
to 16. Null arrays throw ArgumentNullException. Wrong arity, already constructed/open
signature use, foreign ownership, Void/null arguments and excessive depth throw
ArgumentException. Generic substitution recurses into imported constructions.

Owner identifies the consumer; AssemblyIdentity/Namespace/Name identify the external
definition. GenericArity belongs to the definition; TypeArguments is empty for definition
references and copied/read-only for constructions. Equality compares consumer ownership,
exact external identity and ordered arguments; GetHashCode agrees. ToString is diagnostic
only. No mutable dependency TypeBuilder is exposed. Imported constructions are accessed
through SignatureType.ImportedType, while GenericInstance still describes owned types.
LocalDefinition.SignatureType also carries these types; ClassType remains null for them.

Example: import a dependency's `Box<T>` TypeDefinition, create an Order class in the
consumer, and call `box.MakeGenericInstance(order)`. Use the resulting reference in
MethodSignature, fields, locals, vectors and default-value initialization. Standard CLI
output uses AssemblyRef/TypeRef and CLASS/GENERICINST signatures (TypeSpec for typed IL
operands). The native writer uses existing Named/Constructed identities scoped to the
dependency manifest. Its reference projection preserves external scope rather than
inventing local definitions. Unknown scopes, noncanonical names and arity mismatches
reject. No native format, instruction or schema extension was added. Earlier experimental
metadata readers may reject these external signatures; rebuild consumers with this API.

Native dependencies must use the metadata writer's existing format-5 naming contract;
importing an arbitrary CLI declaration does not translate its executable implementation.
Imported references are recorded as native dependencies even when not subsequently used.
The reader resolves no dependencies and verifies no external constraints/layouts.
Method/constructor imports containing these signatures and mappings to translated
System's original native identities are **not yet implemented**. This slice supplies
the signature contract required for that work; Raven's collections sample remains blocked.

Validation: C# tests inspect actual CLR generic scopes and execute the output, then
check native declaration projection and malformed dependency rejection. The
`--imported-type-integration <neoclr> <fresh-output>` test mode loads a separate native
library/application, verifies and returns 42, including external interface and
`Box<consumer Order>` fields/defaults and generic forwarding.

## Imported nominal method signatures (development, 2026-10-01)

The existing `AssemblyBuilder.ImportReference(MethodDefinition, AssemblyIdentity)`
overload decodes dependency-local CLASS and GENERICINST signatures into consumer-owned
`ImportedTypeReference` values, including supported vectors and method parameters. A
method can return `Box<T>`, accept it, or accept a vector of a dependency interface.
The method owner still must be nongeneric and top-level, and the method static. The
exact dependency definition snapshot supplies type name, arity and reference category;
module-local tokens are never compared across assemblies.

The immutable `ImportedMethodReference.Signature` exposes the remapped contract. Import
interning still checks exact identity/MVID/token and signature equality. Scoped generic
arguments and substituted signatures use the existing `MakeGenericInstance` API.
Malformed/truncated/noncanonical encodings, missing definitions, excessive nesting,
value-type encodings, nested vectors and unsupported TypeRef signatures throw
`InvalidDataException`. Null inputs retain `ArgumentNullException` behavior.

Standard CLI TypeRef/MemberRef/MethodSpec output and native dependency-scoped named
signatures are retained, without runtime format changes. Direct calls through foreign
mutable producer methods do not gain nominal support; use the explicit snapshot import.
Primitive-only recognizers and `MemberReference.ResolveMethod` retain their documented
narrower subset. General nominal MemberRef resolution is still pending.

The C# contract tests execute a library factory returning an object and a library reader
accepting it on the CLR. Raven's native probe additionally tests a generic factory,
consumer-owned payload aliases, interface-vector overload matching and missing-method
rejection. Nullable annotations are not retained by the current declaration projection;
these cross-assembly factory contracts use nonnullable references. Imported value/union
contracts, constructors/instance calls and translated-System native identities remain
unsupported. [Native evidence](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/nominal-method-validation.json).

## Owned value types (development, 2026-10-01)

```csharp
TypeBuilder AssemblyBuilder.AddValueType(string @namespace, string name,
    TypeVisibility visibility = TypeVisibility.Public);
TypeBuilder AssemblyBuilder.AddGenericValueType(string @namespace, string name,
    IEnumerable<string> genericParameterNames,
    TypeVisibility visibility = TypeVisibility.Public);
bool TypeBuilder.IsValueType { get; }
bool TypeDefinition.IsValueType { get; }
```

AddValueType creates an owned, sealed sequential-layout CLI type deriving from the
configured core's System.ValueType. AddGenericValueType adds one through 32 invariant,
unconstrained parameters and appends the normal metadata arity suffix to the name.
Names/visibility/duplicate and resource checks match AddClass/AddGenericClass; invalid
inputs throw ArgumentException and null generic parameter collections throw
ArgumentNullException. Neither method creates an instance constructor. The definition
supports static methods, supported inline instance fields (see the union payload foundation update), defaults, parameters/results,
locals, vectors and generic construction/arguments. Public/internal visibility is
preserved in both output formats.

TypeBuilder.IsValueType reports the producer category. TypeDefinition.IsValueType
reports whether the input directly extends a type named System.ValueType or System.Enum;
this is metadata classification, not runtime loading or validation of the base assembly.
SignatureType.ClassType is retained as the compatibility property name for an owned
nongeneric nominal identity, including a value type. GenericTypeInstance.Definition
likewise distinguishes classes, interfaces and value types using the definition flags.
CLI encoding uses VALUETYPE and GENERICINST VALUETYPE for these types. Native output
uses the existing non-reference, sealed type shape; native reading and CLI projection
preserve it without a new instruction or format revision.

ValueType and DefaultConstructor generic requirements admit these definitions;
ReferenceType requirements and nominal class bounds reject them. Instance methods,
constructors and interface implementation are not yet supported for producer value
types (InvalidOperationException). Direct nominal field storage is rejected with ArgumentException, avoiding unsupported
recursive inline layouts. Declaring-type parameters are allowed, with types substituted
at construction. StoreField/Emit(Stfld, ...) on a value-type owner require an address
obtained from LoadLocalAddress for an initialized local of the exact owner type.
Reads accept either that address or a value copy. Uninitialized, mismatched-owner and
value-copy stores throw InvalidDataException on write; partial field assignment does
not initialize a local. Readonly field restrictions remain unchanged. Static properties may use static accessors as elsewhere.

Imported value-type definitions still reject: importing them as reference types would
corrupt signature/category semantics. Native reader checks the bounded shape and rejects
unsupported field storage before projection. These are current producer limits, not
permanent neoCLR rules; runtime value types already exist beyond this subset.

The C# checks execute defaults, field reads, static/generic forwarding and vector
storage on CLR and native neoCLR, both returning 42. They also test CLR reflection,
read snapshots, native projection, constraints and invalid shape/storage rejection.
[Evidence](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/value-type-validation.json).
This is a prerequisite for Raven union emission, not a completed collections gate.
Generic payload storage and addressed local field mutation are now tested: value
payloads and arrays retain copies, while reference payloads retain their aliases.
The legacy native managed-reference store returns an inhabited Void value; the native
writer appends a pop (and adjusts branch offsets) to preserve CLI stfld stack behavior.
No runtime opcode or format change is introduced.

## Definition-first model (author direction, 2026-10-01)

**Planned refactor, not implemented API:** definitions are the canonical editable
metadata model. Builders are optional convenience facades over those exact objects;
writers consume definitions. A caller must be able to construct an assembly/module,
a TypeDefinition with attributes and an explicit base-type reference, and field/method
members, then write it without using a builder. A builder must expose its definition
and preserve object identity: editing that definition affects subsequent output without
copying or synchronizing a second graph.

The current implementation instead uses immutable loaded definition snapshots and a
separate mutable producer graph. This is an architectural gap. The earlier bounded
builder APIs are compatibility surfaces, not the long-term source of truth. The
Cecil comparison is about the editable graph and ownership model, not dependence on
host reflection or an identical API spelling. Value-type classification follows the
base-type/category contract; sealed/layout attributes alone do not establish that a
type is a value type. Explicit target core identities remain essential for neoCLR.

The next implementation slices are:

1. Establish constructible assembly/module/type/field definitions and explicit type
   references with ownership-aware member collections. Use the author's manual struct
   example as a C# contract test. Encode a sealed sequential value type based on the
   configured core's ValueType and inspect/load the produced CLI/native assembly.
2. Move existing producer state into definitions and adapt builders into facades.
   Keep existing Raven-facing calls compatible where practical. Check that a builder
   and direct edits see the same definition objects and encode equivalent output;
   temporary adapters must not become a second authoritative graph.
3. Move method signatures, locals and instructions into definition/body objects, retain
   type-safe/raw Emit helpers, and make both writers consume that representation.
   Preserve symbolic branch targets and recalculate offsets on encoding; an ILProcessor
   can then provide insertion helpers without requiring a separate body model.
4. Materialize supported read metadata and bodies into the same definitions for
   read–edit–write. Preserve opaque unchanged input where possible and explicitly reject
   edits that would discard unsupported metadata. Do not claim arbitrary loaded assembly
   editing until preservation and executable roundtrip tests establish it.

Validation must cover manual construction, ownership/identity, edits made through both
surfaces, repeated writing and supported loaded roundtrips, plus existing C# tests and
Raven-to-native consumers. The roadmap's unchanged collections/runtime-library acceptance
remains the overall goal; this author-directed refactor precedes more builder-only
feature expansion. No new constructor or mutation API in this plan is shipped yet.

### Cecil alignment reference (reviewed 2026-10-01)

The author endorsed alignment with [Mono.Cecil](https://github.com/jbevain/cecil)
where it matters and supplied that repository as the reference. Review of its current
source supports the following direction; these are refactor requirements, not claims
that our API already implements them:

- AssemblyDefinition creation, reading and writing address the same object model.
  Follow that lifecycle and familiar definition/reference terminology.
  [AssemblyDefinition source](https://github.com/jbevain/cecil/blob/master/Mono.Cecil/AssemblyDefinition.cs).
- Type definitions expose attributes, base types and editable members. Follow that
  shape; derive value-type classification from the base/category contract rather than
  a collection of independent flags. Cecil's IsValueType checks System.ValueType/Enum
  and does not support setting that property directly.
  [TypeDefinition source](https://github.com/jbevain/cecil/blob/master/Mono.Cecil/TypeDefinition.cs).
- Member collections maintain declaring-owner relationships when adding/removing members.
  Adopt explicit ownership invariants to prevent accidentally sharing a definition
  between modules/types. Exact reparenting behavior remains a design/validation choice.
  [MemberDefinitionCollection source](https://github.com/jbevain/cecil/blob/master/Mono.Cecil/MemberDefinitionCollection.cs).
- Module-scoped importing and type-system access provide reference context. Preserve
  this separation from runtime loading; neoCLR must retain explicit core/dependency
  identity rather than infer its target from host reflection.
  [ModuleDefinition source](https://github.com/jbevain/cecil/blob/master/Mono.Cecil/ModuleDefinition.cs).
- An ILProcessor operates on a method body's instruction collection and supports typed
  raw emission plus insertion/removal/replacement. Adopt this relationship; convenient
  Call/LoadArgument helpers may sit above it. Retargeting branches and maintaining valid
  bodies need explicit rules and tests, not an assumption that list replacement does all
  repairs automatically.
  [ILProcessor source](https://github.com/jbevain/cecil/blob/master/Mono.Cecil.Cil/ILProcessor.cs).

The benefit is one familiar editable graph for manual callers, compiler emission and
inspection. The costs are migration of existing builder-owned state, ownership/cache
invalidation rules and preservation testing for loaded content. We are aligning the
model, not promising drop-in Cecil API/binary compatibility or adding a Cecil runtime
dependency. neoCLR assembly-level functions and future metadata categories remain
explicit extensions to that shared model. Different encoders consume it and validate
supported target capabilities; they do not determine the public graph's shape.


## Authored definitions: first migration slice

Development API, 2026-10-01. CLI metadata and CIL remain the baseline; this slice
changes the authoring model without changing the wire format or instruction set.
It implements only the assembly/type/field part of the definition-first plan above.

| API | Contract |
| --- | --- |
| `AssemblyDefinition.CreateAssembly(AssemblyIdentity identity, AssemblyIdentity coreLibrary)` | Creates an authored assembly with explicit target identity and core contract. Existing producer identity validation applies. |
| `AssemblyDefinition.Write()` | Encodes authored definitions; loaded snapshots still return owned original bytes. |
| `AssemblyDefinition.WriteNativeAssembly()` | Encodes authored definitions through the existing native writer. Loaded snapshots throw `InvalidOperationException`. |
| `AssemblyBuilder.Definition` / `AssemblyBuilder.ForDefinition(AssemblyDefinition)` | Returns the same assembly/facade without copying declarations. Null input throws `ArgumentNullException`; loaded input throws `InvalidOperationException`. |
| `ModuleDefinition.ImportReference(AssemblyIdentity scope, string namespace, string name)` | Creates an explicitly scoped reference; does not load a dependency or consult host reflection. Invalid/null arguments reject. Explicit resolution requires matching assembly identity. |
| `TypeDefinition(string namespace, string name, uint attributes, TypeReference? baseType)` | Creates a detached declaration. Attachment admits nongeneric root classes, static classes or sealed sequential value types with the same module's explicitly imported core Object/ValueType base. Unsupported shape, foreign ownership or duplicate name throws `ArgumentException`. |
| `TypeDefinition.BaseType` | Authored base reference or a loaded local native base; CLI base decoding remains pending. |
| `FieldDefinition(string name, ushort attributes, SignatureType fieldType)` | Creates a detached instance field. Supports Private, Assembly or Public access and optional InitOnly; nonvoid signature required. Value-type storage retains existing primitive/generic-payload restrictions. |
| `FieldDefinition.FieldType` | Authored signature; null for opaque loaded signatures. `GetSignature()` rejects authored fields until encoded/read; use FieldType instead. |
| `FieldDefinition.Name` | Authored fields can be renamed with Unicode/name and duplicate checks. Loaded edits throw `InvalidOperationException`. |
| `TypeBuilder.Definition` / `FieldBuilder.Definition` | Exact declaration object consumed by the facade. Builder field creation appends to the definition collection. |

`ModuleDefinition.Types` and `TypeDefinition.Fields` now return `IList<T>` instead
of `IReadOnlyList<T>`: a development API source compatibility change. Authored lists
support appending only. Insertion elsewhere, replacement, removal and clearing throw
`NotSupportedException`. Ownership is exclusive; attach a type to its module and fields
to their type. Detached Module properties are unset until attachment. Physical tokens
remain zero until encoding and rereading; token lookup is a loaded-image operation.
`ModuleDefinition.Fields` reflects current authored fields.

Method bodies, property associations and generic parameter declarations still use the
existing builder representation. Authored method/function views and EntryPoint now expose canonical declarations;
use the builder facade to create type methods and edit bodies. Other
snapshot metadata views are not materialized authored views. Loaded definitions remain
read-only, byte-preserving snapshots. Writers currently use builder encoding adapters;
this is not completion of canonical method/body or reader/editor migration.

```csharp
var assembly = AssemblyDefinition.CreateAssembly(identity, targetCoreIdentity);
var module = assembly.MainModule;
var valueType = module.ImportReference(targetCoreIdentity, "System", "ValueType");
var type = new TypeDefinition("Example", "MyStruct", 0x109, valueType);
type.Fields.Add(new FieldDefinition("MyField", 0x6, PrimitiveType.Int32));
module.Types.Add(type);
byte[] pe = assembly.Write();
```

Validation: `AuthoredDefinitionChecks` covers exact facade identity, direct declarations,
renaming, ownership rejection, CLR shape/execution and native verify/run (42).
`--authored-definition-integration <runtime> <fresh-directory>` reproduces the native
case. The 76 C# contract groups and Raven external-signature/generic-library runtime
probes pass. The broader collections `Option<Order>` gate remains unchanged.


### Shared method declarations

The next migration slice adds `MethodBuilder.Definition`, with the declaration owning
its name, authored namespace, access/static flags and immutable signature.
`MethodDefinition.AuthoredSignature` returns that same `MethodSignature` (null for
loaded declarations); `MethodDefinition.Namespace` returns the authored namespace
(null for loaded physical rows). `ModuleDefinition.Methods`, `.Functions`,
`TypeDefinition.Methods` and `AssemblyDefinition.EntryPoint` return those exact
objects. Global functions retain a null declaring type. Type-method and aggregate method collections remain read-only views. The following
slice enables direct assembly-function construction; mutable body definitions remain pending.

`MethodDefinition.Attributes` includes context-derived CLI flags for constructors,
property accessors, interface contracts and implementations. Inspection and PE writing
share flag calculation; writer-side accessor classification stays precomputed.
Authored `GetSignature()` throws `InvalidOperationException`; raw signature recognizers
remain loaded-image operations. Write and reread for physical tokens/signatures.
EntryPointToken remains zero before serialization. Existing typed helpers and raw
Emit overloads continue to operate through the method builder.

Validated by 76 C# groups, including declaration identity and accessor/interface flag
round trips; direct-struct CLR/native execution returns 42. Raven's external-signature
native probe also passes with the rebuilt metadata dependency.


### Direct assembly-level function construction

`new MethodDefinition(string name, MethodSignature signature,
MethodVisibility visibility = MethodVisibility.Public, string @namespace = "")`
creates a detached static function. Public/Internal access is supported; other access
throws `ArgumentOutOfRangeException`. Null signatures throw `ArgumentNullException`;
invalid/reserved names and namespaces throw `ArgumentException`. Final Unicode/body
validation remains at writing, preserving existing builder behavior.

Append to `assembly.MainModule.Functions` to attach and validate signature ownership,
uniqueness and the 256-function limit. This collection now returns `IList<MethodDefinition>`
instead of `IReadOnlyList<MethodDefinition>` (development API change). Loaded collections
reject appends; authored collections reject replacement/removal/reordering. Attached or
loaded declarations cannot be attached again. A failed foreign-signature attachment does
not transfer ownership. Type-owned methods still require TypeBuilder construction.

`MethodBuilder.ForDefinition(MethodDefinition)` returns the existing body helper facade.
Null throws `ArgumentNullException`; detached or loaded input throws
`InvalidOperationException`. `AssemblyDefinition.EntryPoint` can now be set on authored
assemblies, including null for a library. Foreign/detached entries throw
`ArgumentException`; loaded assignment throws `InvalidOperationException`. Writing
still validates the entry signature/body. Detached Module is unset until attachment.

```csharp
var answer = new MethodDefinition("Answer", PrimitiveMethodSignature.Int32(0, true),
    @namespace: "Example");
assembly.MainModule.Functions.Add(answer);
var answerBody = MethodBuilder.ForDefinition(answer);
answerBody.LoadConstant(42);
answerBody.Return();
var main = new MethodDefinition("Main", PrimitiveMethodSignature.Int32(0, true));
assembly.MainModule.Functions.Add(main);
assembly.EntryPoint = main;
var mainBody = MethodBuilder.ForDefinition(main);
mainBody.Call(answerBody);
mainBody.Return();
byte[] image = assembly.Write();
```

This follows Cecil's declaration/body-helper separation while preserving neoCLR's
assembly-level function category. Existing CLI global-function projection and native
encoding remain unchanged. The executable test combines this call with direct struct
field storage; CLR/native return 42. All 76 C# contract groups pass. Arbitrary instruction
editing, canonical body definitions and loaded read/edit/write remain future slices.


### Direct static type-method construction

`new MethodDefinition(string name, ushort attributes, MethodSignature signature)`
creates a detached static type method. Attributes accept Static plus Public, Assembly
or Private, and optional HideBySig. The subsequent instance/constructor extension is described below. Abstract, virtual
and other flags remain unsupported by this constructor; existing builders retain
their previous bounded contracts. Null signatures throw `ArgumentNullException`; unsupported
flags, empty/overlong names and `.cctor` throw `ArgumentException`.

Attach the declaring type to its authored module, then append the method to
`TypeDefinition.Methods`. This property now returns `IList<MethodDefinition>` instead
of `IReadOnlyList<MethodDefinition>` (development source compatibility change). It is
append-only for authored types and read-only for loaded types. Adding to a detached
type throws `InvalidOperationException`. Attachment checks signature ownership and
scope, duplicate name/parameter/generic-arity signatures and the 256-method limit.
Foreign or already attached declarations reject without moving ownership. Explicit
function declarations and type-method declarations cannot be interchanged. Interface
methods continue to require the abstract-contract builder API.

`MethodBuilder.ForDefinition` supplies existing typed helpers and raw Emit overloads.
The method's namespace follows its declaring type, and its declaring type/module are
the exact authored objects. Existing builder-created instance methods, constructors
and interface contracts also enter this same collection. Method bodies remain in the
builder representation. CLI attributes and encodings are unchanged.

```csharp
var answer = new MethodDefinition("Answer", 0x96,
    PrimitiveMethodSignature.Int32(0, true)); // Public | Static | HideBySig
attachedType.Methods.Add(answer);
var body = MethodBuilder.ForDefinition(answer);
body.LoadConstant(42);
body.Return();
```

Validation extends the manual executable to call a directly declared static type method
through an assembly-level helper. CLR and neoCLR return 42. C# contracts cover shared
identity, access flags, duplicate/foreign attachment, detached-owner rejection and
unsupported method categories. The broader collections `Option<Order>` gate is unchanged.


### Direct instance methods and constructors

The CLI-attribute `MethodDefinition(string, ushort, MethodSignature)` constructor now
also accepts nonstatic instance methods and `.ctor` declarations. Public/Assembly/Private
and optional HideBySig apply as before. A `.ctor` must be nonstatic, return Void and
declare no method generic parameters. It may specify SpecialName and RTSpecialName
together; writing supplies both flags as it does for existing builder constructors.
Those flags on ordinary methods, partial constructor flag pairs, `.cctor`, arbitrary
virtual/abstract flags and invalid constructor signatures throw `ArgumentException`.

Attachment requires a reference-class owner for instance methods/constructors; static
and value-type owners throw `InvalidOperationException` before ownership transfer.
Direct interface contracts are covered by the subsequent interface slice below. Signature ownership, generic scopes,
access and readonly-field rules reuse the existing method/body contracts. Detached
method bodies still require attachment before `MethodBuilder.ForDefinition` succeeds.

```csharp
var constructor = new MethodDefinition(".ctor", 0x1806,
    new MethodSignature(PrimitiveType.Void, new[] { PrimitiveType.Int32 }));
attachedClass.Methods.Add(constructor);
var body = MethodBuilder.ForDefinition(constructor);
body.LoadArgument(0); // receiver
body.LoadArgument(1);
body.StoreField(ownedFieldBuilder);
body.Return();
```

As with the existing producer, root-class CLI constructors initialize System.Object;
native construction uses the existing root-object path. Constructor chaining, value-type
instance methods and class virtual dispatch are not added by this authoring change.
No wire format or CIL changes are introduced. This uses the Cecil-like definition/body
helper separation while retaining the current bounded execution contract.

The C# executable fixture constructs a manually declared reference class, initializes
its private readonly Int32 field in a manual constructor, calls a manual instance reader,
and carries the result through the function/type-method/struct path. CLR and neoCLR
return 42. Tests also compare emitted constructor/instance flags and reject invalid
constructor signatures and owner categories. Canonical body definitions, direct interface
contracts and loaded editing remain pending; the collections `Option<Order>` gate is unchanged.


### Direct interface declarations and dispatch

A nongeneric `TypeDefinition` with Interface | Abstract and optional Public attributes
can now be attached directly. Its base reference must be null and it cannot declare
fields. Unsupported bases/storage reject before module ownership is transferred.

The CLI-attribute `MethodDefinition` constructor admits public instance contracts with
Abstract | Virtual | NewSlot together (optional HideBySig). Partial flag combinations,
static/private contracts, constructors and method-generic contracts reject with
`ArgumentException`. Attach these declarations to an authored interface; an abstract
contract on a class or concrete declaration on an interface throws
`InvalidOperationException`. These bounds match existing interface builder support.

```csharp
var contract = new TypeDefinition("Example", "IRead", 0xa1, null);
assembly.MainModule.Types.Add(contract);
var read = new MethodDefinition("Read", 0x5c6,
    PrimitiveMethodSignature.Int32(0, true));
contract.Methods.Add(read);
// Use MethodBuilder.ForDefinition(read) as the target of CallVirtual.
```

The contract remains bodyless. Writing rejects instructions or locals on it through
existing abstract-method validation. Relationships (base interfaces and class
implementations) still use TypeBuilder.AddBaseInterface/AddInterfaceImplementation;
they have not yet moved to directly mutable definition collections. Generic interface
definitions still require builders. No class virtual dispatch, static interface methods,
default implementations or new instruction encodings are introduced.

The manual executable declares IRead and Read directly, registers the class implementation
through the existing helper and calls it with CallVirtual. CLR and neoCLR return 42.
All 76 C# groups pass, including existing multi-implementation/inherited interface tests
and new invalid-owner/flags/interface-storage checks. Raven's rebuilt external-signature
probe also passes. Canonical bodies, relationship definitions and loaded editing remain
pending; this does not resolve the collections `Option<Order>` import gap.


### Authored interface relationships

`InterfaceImplementation(TypeReference interfaceType)` creates an unattached relationship;
null throws `ArgumentNullException`. `InterfaceType` retains the exact immutable reference.
`DeclaringType` is null until successful attachment. Append to the new
`TypeDefinition.Interfaces : IList<InterfaceImplementation>` on an attached authored type.
For interfaces this declares inheritance; for root classes it declares implicit
implementation. Builders add to this same collection; their existing handle lists are
encoding caches over these immutable edges.

The original local overload requires attached, same-assembly nongeneric interfaces.
External overloads are documented under [external interface declarations](#external-interface-declarations-development-2026-10-03). Foreign,
noninterface, detached, unresolved, duplicate or cyclic targets and reused edge objects
throw `ArgumentException`; unsupported/detached owners throw `InvalidOperationException`.
Replacement, removal and clearing throw `NotSupportedException`. Loaded `Interfaces`
access also throws `NotSupportedException` until reader materialization is implemented.
No resolver/host loading occurs during attachment. Existing implementation completeness,
256-edge limits and cycle checks remain in force.

```csharp
derivedInterface.Interfaces.Add(new InterfaceImplementation(baseInterface.ToReference()));
implementingClass.Interfaces.Add(new InterfaceImplementation(derivedInterface.ToReference()));
```

The manual inherited-interface dispatch case executes on CLR/neoCLR (42). All 76 C#
groups pass, including duplicate/cycle/category rejection. CLI InterfaceImpl and native
encoding remain unchanged. Generic relationships and loaded editing remain unsupported.


### Definition-owned method bodies

`MethodDefinition.Body : MethodBodyDefinition` now owns authored instruction, local
and symbolic-label storage. Existing MethodBuilder helpers and raw Emit overloads
operate on this same storage, as do the CLI/native writer adapters. The definition
returns one stable body object. Loaded Body access throws `NotSupportedException`;
loaded-body materialization and editing remain pending. An abstract declaration may
have an empty Body, but writing rejects instructions or locals on it.

| Member | Contract |
| --- | --- |
| `MethodBodyDefinition.Method` | Exact owning MethodDefinition. |
| `Locals : IReadOnlyList<LocalDefinition>` | Cached, live read-only view of declared slots; existing builder-owned handles remain compatible. |
| `Labels : IReadOnlyList<BranchLabel>` | Cached, live read-only view of allocated label handles. |
| `ClearInstructions()` | Removes instructions and label marks, preserving locals and label handles. Referenced labels must be marked again. Writers revalidate the result. |

Use `MethodBuilder.ForDefinition(method)` for emission after attachment. ClearBody
now delegates to Body.ClearInstructions. Read-only local views no longer allocate a
wrapper on each getter call; no broader performance improvement is claimed. This
moves canonical storage without introducing a second instruction graph or changing
CLI/CIL/native lowering. Public arbitrary instruction lists, insert-before/after,
body decoding, exception regions and independent local construction are not added.

C# tests verify method/body/local/label identity, clear-and-rebuild with a retained
branch label and locals, CLR execution (42), and rejection of loaded Body access.
The native manual object/interface/struct case replaces a body through the definition
API before emission and returns 42. All 76 contract groups and the rebuilt Raven
external-signature probe pass. Property/generic-definition migration and loaded editing
remain open, as does the collections `Option<Order>` target gap.


### Shared authored property declarations

`PropertyDefinition(string name, SignatureType propertyType, MethodDefinition? getter = null,
MethodDefinition? setter = null)` creates a detached association. It requires a nonnull
value type and at least one authored accessor; null type throws `ArgumentNullException`,
missing/loaded accessors throw `ArgumentException`. Append to the attached owner's
`TypeDefinition.Properties : IList<PropertyDefinition>` after attaching accessors.
The collection is append-only; loaded collections remain read-only. Changing its former
IReadOnlyList return type is a development API compatibility change.

Attachment reuses name/Unicode, non-Void signature, scope/ownership, accessor reuse,
getter/setter shape, index parameters, duplicate and limit validation. Invalid associations
throw `ArgumentException` without attaching; detached owners throw `InvalidOperationException`.
Accessors must be nongeneric ordinary methods on that owner. Their access and staticness
remain on method definitions. Authored property flags are zero; custom attributes, defaults
and arbitrary PropertyAttributes authoring are not introduced.

`PropertyType` and `ParameterTypes` expose authored value/index signatures (null for loaded
opaque signatures). GetMethod/SetMethod return the exact authored declarations. Accessing
DeclaringType before attachment throws `InvalidOperationException`. Authored GetSignature
throws `InvalidOperationException`; encode and read for physical blobs. Raw primitive
recognition remains a loaded-signature operation. PropertyBuilder.Definition returns this
same object; the builder delegates its state. ModuleDefinition.Properties includes authored
properties. Loaded raw rows and their preservation behavior remain unchanged.

```csharp
var property = new PropertyDefinition("Value", PrimitiveType.Int32, getterDefinition);
attachedType.Properties.Add(property);
```

The manual executable associates its instance reader with Value. CLR reflection constructs
the class and reads Value as 42; native execution still returns 42. All 76 contract groups,
including existing getter/setter/indexer tests, pass; Raven's rebuilt external-signature
probe passes. No property wire encoding or compiler admission change is introduced.

### Builder direction clarified by the author (2026-10-01)

Definitions/references and inspection/manipulation remain Cecil-like. Builders are a
separate convenience layer over those same definitions and should increasingly follow
Reflection.Emit-style generation patterns. They are not a replacement or drop-in
implementation of System.Reflection.Emit, and this direction does not require a host
Reflection.Emit dependency. Typed helpers and raw emits remain complementary.

This is an architectural direction, not a completed API rename or compatibility claim.
Future Define-style construction and body-helper decisions should follow that direction
while preserving target-specific capability checks and explicit identities. The property
migration here shares declarations; existing Add-style entry points remain supported.


### Direct generic types and shared constraint storage

`TypeDefinition(string namespace, string name, uint attributes, TypeReference? baseType,
IEnumerable<string> genericParameterNames)` creates a detached generic root class,
value type or interface. Supply a simple name without backtick/arity; the constructor
appends the CLI arity suffix. One through 32 unique valid names are copied using the
existing signature-name rules. Null input throws `ArgumentNullException`; invalid names,
empty/duplicate/over-limit parameters throw `ArgumentException`. Attachment applies the
existing category/base/ownership restrictions; generic static classes remain unsupported.
Pending fields validate VAR references against the declaring arity, including supported
generic value-type payload fields.

`TypeDefinition.GenericParameterNames` is an immutable authored list shared by the builder;
it is null for loaded snapshots whose names are not materialized. Existing GenericArity
continues to report loaded arity. `SpecialConstraints` and `GenericConstraints` expose
cached live read-only views of definition-owned storage. Loaded access throws
`NotSupportedException`. Existing TypeBuilder.SetSpecialConstraints/AddBaseTypeConstraint
helpers validate and update that storage. Nominal constraints retain their existing
GenericTypeConstraint builder bound handles; independent generic-parameter/constraint
reference objects and loaded constraint editing are future work.

The manual executable declares `Identity<T>` directly, adds Pass(T):T directly, constrains T
to a value type through the helper, and calls `Identity<Int32>`.Pass. CLR reflection confirms
the parameter name and special constraint; CLR/native execution returns 42. A String
instantiation rejects. All 76 C# groups and Raven's rebuilt native probe pass. Existing
GenericParam/GenericParamConstraint and native encodings are unchanged.

### Layered reader/writer direction (author clarification, 2026-10-01)

The intended write pipeline is **builders → definitions → encoded metadata → PE**.
Builders are optional generation helpers over the shared definitions. Metadata encoding
owns CLI tables/heaps, signatures, method bodies and explicit neoCLR extensions; the PE
layer packages the result. The read path reverses these boundaries: PE extraction,
metadata decoding and definition materialization. Builders may then wrap editable loaded
definitions. Readers and writers should expose complementary responsibilities at these
boundaries, rather than making the object model depend on a particular packaging writer.

This is the target architecture, not a claim that the separation is complete. Current
AssemblyBuilder.WriteImage still combines metadata/body encoding and PE packaging;
AssemblyDefinition reading returns bounded immutable snapshots. Native container handling
also has its existing adapter path. No format change or new binary layer API is claimed
by this record. The definition migration provides shared authoring state; extracting the
encoding/packaging boundary and materializing editable loaded definitions remain work.


### Integration priority after architecture checkpoint

Author direction, 2026-10-01: the current API is good enough to resume the working
Raven/native case. Further object-model migration, builder naming and encoding/PE
separation are deferred unless required by an observed integration blocker. This
supersedes the earlier migration-first sequencing, not the architectural direction.
The current blocker remains imported `Option<Order>` value-category support; the
reference-only import contract must not be widened by treating it as a class.


### Imported value signatures (integration slice, 2026-10-01)

AssemblyBuilder.ImportReference(TypeDefinition, core) now admits public top-level value
definitions in addition to reference classes/interfaces. Values must directly extend
System.ValueType through an AssemblyRef matching the explicit dependency core identity.
Enums, local/fake ValueType bases, nested/nonpublic declarations and constrained/variant
generic definitions remain unsupported. ImportedTypeReference.IsValueType is immutable
and participates in identity, interning consistency and generic substitution. Special
constraint checks distinguish imported values from references.

CLI signatures encode imported values with VALUETYPE and GENERICINST VALUETYPE. The
static method signature decoder accepts dependency-local value TypeDefs and rejects
CLASS/VALUETYPE disagreement with the declaration. Cross-dependency TypeRef signatures,
imported instance/constructor calls and generic declaring-owner member calls remain
unsupported. Importing a type does not import its members or implement Raven unions.

Native format-5 signatures still use Named/Constructed types resolved by the loader.
To retain the CLI category in metadata-only projection, the assembly manifest now has
an optional `value_type_references` array of canonical external native definition names.
The writer emits it only for imported values. The reader bounds it to 4096 unique names,
requires declared dependency scopes and restores value-category signatures in the PE
projection. Native validation requires each listed name to resolve to a nonreference,
noninterface loaded type, rejecting missing declarations/category mismatches.

This is a temporary native projection annotation for information represented directly
in CLI signatures. It adds no opcode or runtime representation. Existing payloads without
the field retain their reference-only import interpretation. Payloads containing it
require the updated runtime; older strict readers reject the new manifest field.
The metadata library owns emission/projection and neoCLR owns dependency validation.
The future native indexed metadata reader/writer should encode the category directly
in signatures, removing the need for this annotation.

C# tests (77 groups) cover cross-assembly nominal/generic value signatures, CLR factory/read
and default/forward execution (42), projection category retention, category disagreement
and explicit core rejection. `--imported-value-integration <runtime> <fresh-directory>`
emits separate native library/consumer containers, verifies/runs (42), and checks that a
reference-class substitution for an imported value rejects. Raven's dedicated consumer
probe also returns 42 with imported returns, parameters, locals and forwarding. The
unchanged collections sample advances to an unsupported lowered invocation; it does not
execute yet.

## Imported member dispatch and closed generic interfaces (2026-10-02)

These are development host C# APIs in `NeoCLR.Metadata.Experimental.Model`. They
use the existing CLI/CIL format: HAS_THIS and VAR in signatures, TypeSpec parents
for constructed MemberRefs and callvirt for interface/final virtual dispatch. Native
format-5 uses its existing constructed-owner function references and interface
relationships. No new opcode is introduced. Reuse the CLI/Cecil comparison and
research above; the benefit is one exact imported contract for both writers, with
a bounded supported subset instead of host reflection loading. The cost is explicit
dependency registration and rejection of currently unsupported declarations.

| Type/member | Contract |
| --- | --- |
| `AssemblyBuilder.ImportReference(MethodDefinition, AssemblyIdentity)` | Also imports public nongeneric instance methods of public top-level reference classes/interfaces. Concrete class methods must be nonvirtual or final; interface methods must be abstract/virtual. Owners may be invariant, unconstrained generic reference types. Core/identity/MVID, access/category and scoped signature checks remain mandatory. |
| `ImportedMethodReference.IsStatic : bool` | Whether the reference excludes a receiver. |
| `ImportedMethodReference.IsInterfaceMethod : bool` | Whether the declaring contract is an interface. |
| `ImportedMethodReference.RequiresVirtualDispatch : bool` | Whether Callvirt is required, including final virtual class members; Call is rejected for these references. |
| `ImportedMethodReference.MakeConstructedReference(IEnumerable<SignatureType> declaringTypeArguments, IEnumerable<SignatureType>? methodArguments = null)` | Returns `ImportedConstructedMethodReference`. Copies arguments, requires exact arities and consumer ownership; caller VAR/MVAR scope is checked when emitted. Instance method generics remain unsupported; static generics may have method arguments. |
| `ImportedConstructedMethodReference.Definition` | The consumer-owned open imported member. |
| `ImportedConstructedMethodReference.DeclaringType` | Consumer-owned constructed `ImportedTypeReference`. |
| `ImportedConstructedMethodReference.Signature` | Substituted `MethodSignature`, with declaring and method parameters replaced simultaneously. |
| `ImportedConstructedMethodReference.MethodArguments` | Read-only copied method argument list. |
| `MethodBuilder.Call(ImportedConstructedMethodReference)` | Emits a direct Call for a concrete/static contract. |
| `MethodBuilder.CallVirtual(ImportedMethodReference)` and `CallVirtual(ImportedConstructedMethodReference)` | Emit Callvirt for an imported interface/final virtual contract. |
| `MethodBuilder.Emit(OpCode, ImportedMethodReference)` and `Emit(OpCode, ImportedConstructedMethodReference)` | Raw typed operands retain the same ownership, dispatch and generic-scope checks as helpers. No unrestricted integer/token escape hatch. |
| `InterfaceImplementation(TypeReference, IEnumerable<SignatureType>)` | Definition-owned relationship with copied arguments; attachment validates ownership and shape. |
| `InterfaceImplementation.TypeArguments : IReadOnlyList<SignatureType>` | Empty for nongeneric relationships, copied ordered arguments for a construction. |
| `TypeBuilder.AddInterfaceImplementation(GenericTypeInstance)` | Appends the same definition relationship. Supports root classes, including generic owners, implementing owned generic interfaces with inherited edges and declaring-type arguments. |

Null inputs throw `ArgumentNullException`; invalid construction/ownership/opcodes
throw `ArgumentException` (an invalid implementation owner throws
`InvalidOperationException`). Unsupported imported declarations, signature bytes,
identity conflicts, limits, missing public interface implementations and invalid
body stacks throw `InvalidDataException`. Instructions retain the existing count
limits. Generic arities are at most 32; constructed relationships are bounded to
256. The target interface must be in the same assembly as an authored implementation;
imported dispatch references themselves can cross assemblies. Neither API infers
implementation conformance from matching method names alone.

```csharp
var imported = consumer.ImportReference(interfaceMethodDefinition, explicitCore);
var call = imported.MakeConstructedReference(new SignatureType[] { PrimitiveType.Int32 });
body.Call(factoryReference); // Returns exactly the imported constructed interface.
body.LoadConstant(42);
body.CallVirtual(call);
body.Return();
```

Readers preserve closed generic interface relationships in reference projections;
older experimental host readers reject this expanded subset. The existing native
runtime already supports these relationships. Imported constructors/value-instance
members, generic instance methods, nonfinal virtual class slots, generic interface
inheritance, cross-dependency TypeRef signatures and native System identity mapping
remain outside this increment. `ImportedInterfaceChecks` executes a separately
written library/consumer on CLR and neoCLR (42), checks native null-receiver failure,
wrong arity/opcode/owner rejection and reference projection. Raven's corresponding
probe executes constructed interface and final class calls (42); the unchanged
collections sample advances to propagation-expression lowering.


### Writable managed-reference parameters (development, 2026-10-02)

`SignatureType.ByReference(SignatureType elementType)` creates a managed-reference
parameter signature; `SignatureType.ByReferenceElement` returns its target or null.
The target is a non-Void supported type, including vectors and scoped generic parameters.
Null throws ArgumentNullException; Void, nested byrefs or exceeding the 16-level nesting
bound throws ArgumentException. Ownership and generic scope are checked on use.
Only method parameters admit this type: byref returns, fields, locals, array elements
and generic arguments reject. ToString appends `&` to the target diagnostic name.

MethodSignature, owned and imported method calls, generic substitution and native CLI
projection preserve BYREF. CLI uses ELEMENT_TYPE_BYREF in parameter signatures; native
uses its existing ByRef type. The bounded static value/generic signature readers recognize
byref primitive/vector/MVAR parameters; primitive-only readers continue to return false
for those signatures. Out flags are emitted only for explicit OutParameters contracts described below; no In flag or readonly modifier is inferred.

Call consumes an initialized exact local address or a matching forwarded ref parameter.
It cannot establish assignment for an uninitialized caller local. `LoadArgument` loads
the reference; `LoadObject`/`StoreObject` (or raw Ldobj/Stobj) read/write the referenced
value. Argument rebinding with Starg rejects; Initobj remains local-address-only.
The body validator rejects mismatched targets, non-address operands and uninitialized
ref calls when either writer runs. References cannot escape via results or value locals.
This slice does not admit ref receiver methods or Raven `out` propagation calls. Explicit metadata out calls are described below.

`ByReferenceChecks.cs` demonstrates generic replacement and reference forwarding, with
independent library/consumer execution from both CLI snapshots and native projections.
Run `--byref-integration <runtime> <fresh-output>` for native binary verification and
execution (42); the regular C# tests include positive and rejection checks.


### Output parameter contracts (development, 2026-10-02)

`MethodSignature(SignatureType returnType, IEnumerable<SignatureType> parameterTypes,
IEnumerable<string>? genericParameterNames = null, IEnumerable<int>? outParameters = null)`
accepts optional zero-based declared parameter indices (excluding the instance receiver).
`IReadOnlyList<int> OutParameters` exposes a copied, sorted list. Every index must be
unique, in bounds and designate a ByReference parameter; invalid or oversized lists
throw ArgumentException. Omission means ordinary ref semantics. The distinction is a
parameter contract, not an overload identity; methods cannot overload ref versus out.
Interface implementations must match the declared out contract.

Callers may pass uninitialized local addresses to declared outputs. On normal return,
those locals become definitely assigned. Ref inputs, including aliases of outputs,
are checked before any output assignment is published. Callees cannot read an output
before writing it and must assign every output on every normal return path; forwarding
an output to another out parameter also establishes assignment. Ldobj/Stobj use the
same parameter addresses as ref calls. Invalid flow raises InvalidDataException when
writing either representation. Initobj still accepts only local addresses.

CLI output uses ordinary BYREF signatures plus Param Out flags. Snapshot signature
readers and imports retain those flags in OutParameters. Native output uses the existing
out_parameters function member, which runtime loading checks and execution enforces;
no wire-format extension or runtime opcode is added. Native-to-CLI projection writes
matching Param rows. Generic method/owner substitution retains output indices.
C# definite-assignment is a language rule; the CLI flag alone does not prove an external
body assigns its outputs. Imported contracts require a trusted/validated implementation;
the metadata reader does not inspect method bodies.

`OutParameterChecks.cs` covers generic assignment, output forwarding, ordinary CLR Out
reflection, native projection/import and library/consumer execution. Negative checks
cover invalid indices, missing/partial assignment, reads before writes, interface
contract mismatch and aliased ref/out preconditions. Run `--out-integration <runtime>
<fresh-output>` for native binary verify/run (42). Conditional out_when_true,
readonly/in contracts and Raven admission remain unsupported by this producer API.


### Managed value receivers (development, 2026-10-02)

`TypeBuilder.AddInstanceMethod` and attached instance `MethodDefinition` declarations
now admit value types as well as reference classes. On a value type, argument zero is
an initialized managed reference to the exact open declaring type. Field reads/writes
and typed LoadObject/StoreObject may use that receiver; field mutation affects caller
storage. Static-type instance declarations and value constructors remain rejected.
CLI uses ordinary instance method signatures with implicit byref `this`; native writes
`receiver_byref: true`. Native readers require that flag for value instance methods and
preserve it when creating a CLI reference assembly. No new opcode or format extension.

`ImportedMethodReference.RequiresManagedReceiver` is true for value instance members.
ImportReference accepts public nongeneric nonvirtual or final concrete value methods,
including invariant unconstrained generic owners; ordinary imported signature limits
still apply. RequiresVirtualDispatch is false for these concrete value calls, including
final virtual implementations. Use Call (or Emit(Call, reference)) with an initialized
exact managed receiver; value Callvirt is rejected because constrained/boxed dispatch
is not represented by this contract. Constructed imported references retain their value
category and receiver requirement. No receiver flag is added to ordinary parameter lists.

Receiver preconditions are checked before publishing any out assignments. Passing an
uninitialized receiver also as an output does not make the call valid. Mismatched or
value-copy receivers and invalid field/indirect access fail on writing with
InvalidDataException. Import/dispatch argument failures retain existing exceptions.
`ValueReceiverChecks.cs` demonstrates separate library/consumer mutation and generic
TryGet(out T), CLI/native projection and rejection cases. Run
`--value-receiver-integration <runtime> <fresh-output>` for CLR and native verify/run 42.
Raven source value declarations and constrained interface calls are not implied.


### Literal terminal failure (development, 2026-10-02)

`MethodBuilder.Fail(string message)` and `Emit(OpCode.Fail, string message)` append a
terminal instruction with no stack operand or result. The body stack must be empty.
The native writer emits the existing `fault` instruction: execution ends with a host
`Fault` classified as `UserFault`, preserving the message. No guest catch/finally
semantics or host-process abort is introduced. The executable CLI writer instead
constructs and throws `System.InvalidOperationException(message)` using ordinary CLI
metadata/CIL; CLR callers can catch it. This is an explicit target difference, not a
claim that CLR exceptions and native faults are interchangeable. The CLI core identity
must supply that standard exception type. Native reference projections keep their
existing nonexecutable throwing stubs.

Messages follow `Ldstr` validation: null throws `ArgumentNullException`; unpaired UTF-16
surrogates or text exceeding 64 KiB UTF-8 throw `ArgumentException`; empty text is valid.
Instruction limits throw `InvalidDataException`. Writers reject a nonempty failure stack,
reachable fallthrough and unreachable instructions after failure. Out parameters need
assignment only on normal returns; a terminal-only path need not assign them. Labels
and branch joins retain ordinary flow validation. Definitions can use the same logical
opcode/string operand through the existing body model. Dynamic diagnostics remain the
runtime library's `System.Fail(message)` contract; this bounded builder instruction
accepts a literal only.


### Value and imported constructors (development, 2026-10-02)

The existing `TypeBuilder.AddConstructor` overloads and authored `.ctor` definitions
now admit value types, including unconstrained generic owners. CLI emission uses
ordinary instance constructor metadata/CIL without injecting an Object base call into
a value constructor. Native emission uses the existing managed construction receiver.
Every own field must be assigned on every normal return. Reading a field before its
assignment, using the whole construction receiver through ldobj/stobj, passing it to
ordinary calls or storing it elsewhere is rejected when writing. A terminal failure
path need not complete initialization. Constructor chaining remains unsupported.
These conservative producer restrictions preserve the runtime's construction contract;
they are not a claim that every CLR-valid constructor body is admitted.

`AssemblyBuilder.ImportReference(MethodDefinition, AssemblyIdentity)` now admits public
constructors of supported top-level classes and value types. The definition must have
the CLI special-name/runtime-special-name flags, a nonstatic/nonvirtual/nongeneric
Void signature and no byref parameters. Type initializers, nested owners and nested
nominal parameter types remain unsupported. Imports still require exact dependency/core
identity and the native writer's naming contract.

`ImportedMethodReference.IsConstructor` identifies allocation references.
`MethodBuilder.NewObject(ImportedMethodReference)` and
`NewObject(ImportedConstructedMethodReference)` consume the substituted constructor
arguments and push the resulting declaring reference or value. Raw `Emit(OpCode.Newobj,
reference)` is equivalent. Null references throw `ArgumentNullException`; foreign
references, wrong dispatch opcodes and uninstantiated/invalid generic scopes throw
`ArgumentException`. Instruction limits and invalid stack/initialization flow fail with
`InvalidDataException`. `Call`/`Callvirt` reject constructor operands. Existing
`RequiresManagedReceiver` describes the value member's implicit receiver; Newobj supplies
its construction address rather than requiring a caller-supplied initialized address.


## Nested declarations (development, 2026-10-02)

`TypeDefinition.NestedTypes : IList<TypeDefinition>` exposes immediate children.
Loaded snapshots are read-only. An attached authored owner accepts detached children
with an empty namespace, no generic parameters and CLI NestedPublic (2) or
NestedAssembly (5) visibility. The enclosing owner must also be nongeneric. Ownership
is immutable after attachment; duplicate names are scoped to the enclosing definition.
Cycles, reattachment, generic nesting and depth beyond 16 fail with ArgumentException;
adding to a detached owner fails with InvalidOperationException. Module.Types continues
to enumerate every physical declaration, including nested types. DeclaringType and
ToReference().Resolve() preserve the same authored definition identity.

`TypeBuilder.AddNestedClass(string name, TypeVisibility visibility = Public)` and
`AddNestedValueType(string name, TypeVisibility visibility = Public)` attach definitions
through that collection and return their builders. Internal maps to NestedAssembly.
Neither helper synthesizes constructors. Existing AddConstructor and body APIs apply.
These helpers reject duplicate/invalid names, generic owners, unsupported visibility
and declaration/depth limits with ArgumentException.

CLI writers emit NestedClass rows and nested visibility. Native writers retain both
explicit module-local declaring_type and matching source declaring_type_token; readers
validate that they identify the same preceding owner before reconstructing the reference
projection. Runtime loading/execution is tested with a nested value constructor (42),
alongside CLR execution, duplicate short names under different owners, deep navigation
and malformed ownership rejection. Imported nested types and captured generic owner
parameters are not yet supported. Existing top-level identities remain unchanged.


### Nested imports and generic children (development, 2026-10-02)

`TypeBuilder.AddNestedGenericValueType(string name, IEnumerable<string> parameters,
TypeVisibility visibility = Public)` adds a generic value child under a nongeneric
owner. The name excludes the arity suffix; one to 32 unique parameter names are copied.
Null parameters throw ArgumentNullException; invalid identity, arity, visibility or
ownership throws ArgumentException. NestedTypes now also accepts corresponding manually
created generic value definitions. Captured generic outer parameters remain unsupported.

`ImportedTypeReference.DeclaringType` exposes the immutable enclosing definition reference,
or null. ImportReference accepts public nested class/value declarations under public
nongeneric containers, retaining scope through equality, hashing and generic substitution.
Parents may be static containers. CLI TypeRefs use enclosing TypeRef resolution scopes;
native identity paths are deterministic and preserve all owners. Private/internal enclosing
scopes, generic captured owners and unsupported base/core contracts reject with
InvalidDataException. NewObject, Call and constructed imported methods retain the nested
owner; their earlier stack and ownership checks still apply.

C# fixtures execute nested imported nongeneric/generic value and class constructors on
both CLR and neoCLR (42). Native reference projection remaps owned types by declaration
identity instead of namespace/name, preventing collisions between equal child short names.


## Structural Function bodies (development, 2026-10-02)

`SignatureType.Function(MethodSignature signature)` creates a structural callable type.
`SignatureType.FunctionSignature` is null for other categories, otherwise exposes
`FunctionSignature.ParameterTypes`, `ReturnType` and `NoResult`. Shape equality/hash use
ordered parameter/result identities, including scoped generic parameters and nominal
owners; target methods and parameter names do not participate. The signature is immutable.
Up to sixteen value parameters are supported; Void means explicitly no stack result.
Byref/out callback contracts, generic callback declarations and inhabited Void results
are outside this initial slice. Generic enclosing method/type parameters can occur in
shapes and are substituted recursively. Nesting is limited to sixteen levels.
Null signatures throw ArgumentNullException; unsupported contracts throw ArgumentException.

`FunctionBinding(SignatureType functionType, MethodBuilder target)` checks a nongeneric
static, final/nonvirtual reference-instance or abstract interface target against the exact
shape. The overload `FunctionBinding(SignatureType, ConstructedMethodReference)` accepts
an owned constructed generic class/interface method after exact signature substitution.
`ConstructedTarget` returns that reference (null for the MethodBuilder overload);
`FunctionType` and `Target` expose the shape and method definition. Constructors,
uninstantiated generic methods, nonfinal virtual class methods and value receivers are rejected.
`FunctionBinding(SignatureType, GenericMethodInstance)` accepts an owned generic method
with all method arguments supplied. `GenericTarget` returns that instance, or null.
Constructed owner references may also supply method arguments. Both forms validate
the substituted shape, caller scopes and generic constraints before writing.
Null arguments throw ArgumentNullException; incompatible signatures throw ArgumentException.
Emission additionally validates output ownership and caller generic scopes.
`MethodBuilder.BindFunction(functionType, target)` and
`Emit(OpCode.BindFunction, FunctionBinding)` push a callable value; the target must belong
to the output assembly. Instance binding consumes the object receiver already on the
stack; static binding consumes no receiver. `InvokeFunction(functionType)` and
`Emit(OpCode.Callvirt, SignatureType)` consume the Function receiver and ordered arguments,
then push the result unless NoResult. Invalid operands, target kinds, ownership or scope
throw ArgumentException, null operands ArgumentNullException. Instruction limits and
invalid evaluation stacks fail with InvalidDataException (stack validation on write).
Bound instance receivers retain shared object identity. Compiler-generated capture frames
and external binding targets remain future work; ordinary imported methods can accept/return
Function values.

Native output uses the runtime's structural Function shape, function.bind and ordinary
instance Invoke. `GetILGenerator().BindFunction` supports definition, constructed-owner and generic-method target overloads;
interface binding selects dispatch from the retained receiver. CLI output uses
core-scoped Func/Action carriers with ldftn (ldvirtftn for interfaces)/newobj and
callvirt, including generic signature variables on Invoke MemberRefs. This is a bridge
representation; native Function identity is independent of a nominal delegate declaration.
The imported-signature reader recognizes those carriers only in the explicit core scope.
Ordinary CLI delegates retain their .NET behavior; native static/instance binding follows the
existing Function runtime contract. This API does not expose native method pointers.

C# validation executes static bindings across an assembly boundary on CLR and native
binary PE, including generic higher-order calls and a no-result callback (42). Exact
shape equality, wrong results, wrong receivers and reference projection are checked.
Raven consumes this transport for owned static and concrete instance method groups.
Captured lambdas still require compiler-generated closure frames. Instance-binding C#
tests execute two bindings against the same mutable receiver on CLR and NeoCLR and
reject missing/wrong stack receivers before writing.

### Concrete imported value overrides (development, 2026-10-02)

`AssemblyBuilder.ImportReference(MethodDefinition, AssemblyIdentity)` accepts public,
nongeneric concrete instance overrides declared on imported value types. They consume
an exact managed receiver and use direct `Call`; `RequiresManagedReceiver` is true and
`RequiresVirtualDispatch` is false. This follows CLR's concrete value-call behavior,
without boxing or reference dispatch. Abstract methods remain unsupported, as do
nonfinal virtual/override methods on reference classes. The same explicit owner/core
and signature validation applies. C# fixtures execute an actual CLR override and a
matching native implementation emitted by the API, returning the expected string and
42. This does not add general class virtual dispatch or boxed value receivers.

### Reference conversion bodies (development, 2026-10-02)

`MethodBuilder.CastReference(SignatureType target)` and
`Emit(OpCode.Castclass, SignatureType)` consume a reference and push the same object
through the target reference signature. Targets are owned/imported nonstatic nominal
reference types or vectors, including constructed generic reference types. Value types,
managed addresses, Function shapes and unbounded generic parameters are not admitted.
The operand must belong to the output assembly; invalid target/owner throws
`ArgumentException`, null throws `ArgumentNullException`, and invalid source stacks or
instruction limits produce `InvalidDataException` when validated. No boxing occurs.
CLI output uses ECMA `castclass`; native output uses its existing `castclass` verifier
and runtime contract, which may reject unsupported conversions. These methods do not
promise general .NET downcast coverage. A C# fixture checks inherited interface dispatch
through two implementations on CLR and neoCLR (42), and rejects value operands.

### Explicit translated-library linkage (development, 2026-10-02)

`AssemblyBuilder.BindNativeLibrary(AssemblyDefinition reference,
NativeLibraryDefinition implementation, AssemblyIdentity coreLibrary)` registers one
explicit declaration/implementation pair before importing any types or members from
that dependency. All parameters are required (`ArgumentNullException`); the core must
match the output core, the dependency must differ from the output, prior imports or a
duplicate binding are rejected, and at most 256 dependencies are admitted
(`InvalidDataException`). Registration returns no value. It does not execute or verify
library bodies. Supply the implementation module separately when loading the consumer.

Subsequent `ImportReference` calls validate the selected public native declaration's
name, arity, value/reference/interface category, instance/static shape, generic arity,
parameter/result signatures, managed receiver and output contract. Unsupported or
ambiguous matches fail with `InvalidDataException`. Translated names remove the CLI
backtick arity suffix and join nesting with dots; this is an explicit temporary bridge,
not a universal resolver or a general name-remapping API. Generic constraints and
unsupported imported shapes retain their existing restrictions. Native interface
members must have empty bodies. Static namespace carriers may use the translator's
legacy value-like representation. Unused declarations are not individually validated.

CLI output preserves original AssemblyRef/TypeRef scopes. Native output uses the bound
module name and exact revision when present (otherwise a name-only module reference),
and records `native_module_bindings`/`native_type_bindings` in its manifest so the PE
reference projection can recover the CLI identities. Readers reject undeclared scopes,
duplicate aliases, invalid categories and cyclic declaring scopes. Older experimental
runtimes do not understand these added manifest fields; use a matching compiler/library/
runtime bundle. No forwarding, dependency download or host assembly probing occurs.

Dependency-local TypeRefs, including local-core values extending a TypeDef
`System.ValueType`, are accepted without an external resolver. Func/Action carriers
also recognize the explicitly selected local core. Its inhabited nominal `System.Void`
is mapped to native Void storage while remaining a nominal value in the CLI projection;
CLI ELEMENT_TYPE_VOID still means no result. A CLI no-result call to an inhabited-Void
native method emits a result discard, with branch offsets adjusted. This bridge does
not erase an out parameter or add general CLI inhabited-void execution support.

The C# linkage fixture executes on CLR and neoCLR (42), tests versioned/unversioned
bindings and malformed mappings, and provides the checked-in binary runtime fixture.
`LocalCoreSignatureChecks` covers local nominal/core Function signatures and cyclic
TypeRef rejection. The unchanged Raven collections application additionally exercises
inhabited Void through Option residual propagation.

The Rust host metadata model exposes `metadata_origin::AssemblyMetadata`'s new
`native_module_bindings: Vec<NativeModuleBinding>` and
`native_type_bindings: Vec<NativeTypeBinding>` fields. Both default to empty on read and
are omitted when empty on write. `NativeModuleBinding` has `assembly: String`,
`module: String`, `revision: Option<String>`; `NativeTypeBinding` has
`native_name`, `assembly`, `namespace`, `name` (`String`), `arity: usize`,
`value_type: bool`, and `declaring: Option<String>`. They derive Debug, Clone,
PartialEq, Eq, Serialize and Deserialize; unknown fields are rejected. Runtime admission
checks declared scopes, text, duplicates and limits (256 modules, 4096 types, arity 32).
These retain projection provenance; they do not authorize calls or override runtime
resolution/access checks. Hosts constructing AssemblyMetadata literals must initialize
the new vectors. These host-only APIs remain outside the guest RavenDoc assembly.

### Constructed interface inheritance (development, 2026-10-02)

`TypeBuilder.AddBaseInterface(GenericTypeInstance baseInterface)` adds an owned
constructed generic base to an interface. Arguments may reference the declaring
interface's type parameters. The definition graph remains authoritative:
`Definition.Interfaces` retains both ordinary and constructed edges; `BaseInterfaces`
lists only directly inherited nongeneric definitions. Null throws
`ArgumentNullException`; a noninterface owner throws `InvalidOperationException`.
Foreign/noninterface bases, duplicate edges, definition cycles, out-of-scope arguments
and the existing 256-constructed-edge limit throw `ArgumentException`. Mutating a later declaration
cannot introduce a cycle through a constructed edge.

Root classes, including generic owners, may implement inherited generic interfaces using the
existing `AddInterfaceImplementation(GenericTypeInstance)` overload. Required methods
are collected transitively with positional substitution before exact public instance
implementation validation. Missing methods fail writing with `InvalidDataException`;
inheritance expansion is bounded to 4096 distinct constructed contracts. This does not
add variance, interface default bodies or MethodImpl
mappings. CLI uses ordinary InterfaceImpl/TypeSpec signatures; native metadata uses
constructed interface signatures with the existing runtime dispatch semantics. No new
instruction or runtime metadata category is introduced.

`MethodBuilder.CallVirtual(ConstructedMethodReference method)` and
`Emit(OpCode.Callvirt, ConstructedMethodReference)` dispatch owned constructed interface
contracts. Constructors require Newobj, interface contracts require Callvirt, and other
constructed methods retain Call. Null throws `ArgumentNullException`; invalid opcode,
foreign target or invalid scope throws `ArgumentException`; instruction limits and
invalid stacks fail with `InvalidDataException`. Both encodings preserve the same
method signature and owner arguments. A C# fixture checks transitive positional
substitution, CLI/native execution (42), PE reference projection, missing implementations,
duplicate edges, cycles and invalid argument scopes.


### Generic class interface implementations (development, 2026-10-02)

`TypeBuilder.AddInterfaceImplementation(TypeBuilder contract)` now also accepts a
nongeneric interface on a generic root class. The `GenericTypeInstance` overload
accepts constructed interfaces whose arguments refer to that class's type parameters.
The same relationships can be attached manually through `Definition.Interfaces`.
Static/value/interface owners remain invalid implementation owners
(`InvalidOperationException`); foreign/noninterface/duplicate edges and invalid owner
or method parameter scopes remain `ArgumentException`. Existing edge limits apply.

Required inherited methods are substituted positionally and must have matching public
instance implementations when writing (`InvalidDataException` otherwise). Stack checking
also substitutes the actual generic receiver's arguments before permitting interface
assignment or dispatch; it does not admit arbitrary generic conversions or variance.
CLI output uses standard InterfaceImpl TypeSpecs and callvirt, and native metadata
retains the same owner arguments for its existing runtime dispatch. No opcode changed.

C# tests execute `Implementation<Unused,T> : Middle<T> : Root<T>` with an Int32 Echo
through `Root<int>` on CLR and neoCLR (42), reproject native metadata to CLI, and reject
missing/mismatched implementations and out-of-scope type/method parameters. Raven's
unchanged collection interfaces additionally execute with generic provider and iterator
implementations, including constructor fields and inherited property/indexer calls.
Loaded CLI `TypeDefinition.Interfaces` enumeration remains an explicit reader limitation;
this change expands authored definitions and native round trips, not that reader view.


### Checked uninitialized reservation (development, 2026-10-02)

`MethodBuilder.ReserveArray(SignatureType elementType)` and
`Emit(OpCode.ReserveArray, SignatureType elementType)` consume an Int32 length and push
an ordinary vector of that exact element type. Supported scalar elements include
caller-scoped type/method parameters. Null throws `ArgumentNullException`; unsupported
Void/vector elements or invalid owner/parameter scope throw `ArgumentException`.
Instruction limits and invalid stack shapes throw `InvalidDataException` on construction
or writing as appropriate. `OpCode.ReserveArray` is appended to the public opcode enum.

This is a native-only extension. Native writing encodes existing `array.reserve`, and
PE/#Neo loading retains checked uninitialized slots. Stores publish typed values; direct
or addressed reads before publication fault in the runtime. Negative lengths and runtime
allocation limits retain the existing array-reservation checks. `NewArray`/`Newarr` keep
ordinary default initialization. See [the storage contract](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/reserved-array-capacity.md).

`AssemblyBuilder.Write()` (and executable CLI writing through definitions) rejects a
body containing reservation with `InvalidDataException`: CLI newarr is not an equivalent
encoding. Native container reference projections remain available and do not promise
executable CLI bodies. No native format version, runtime opcode or guest public API
changes are required. This host-only builder API is covered by this manual reference,
not the guest RavenDoc assembly.

C# checks cover generic raw emission, native verification/execution (stored slot 42,
unread slot fault), projection loading, executable CLI refusal, invalid element and
parameter scope, and an invalid length stack type. Raven separately tests the matched
explicit authoring-seed mapping and its default-disabled configuration.

### Native namespace-function dependency binding (development, 2026-10-02)

BindNativeLibrary also recognizes public, nongeneric, abstract sealed top-level CLI
containers carrying the exact configured core's System.Runtime.CompilerServices.TopLevelAttribute.
The marker must have a parameterless constructor and no named arguments. Public static
concrete methods bind to ownerless native functions named namespace.method, with existing
arity, parameter, result and access validation. Container spelling is irrelevant.
CLI output preserves the container reference; native output uses the bound function.
Unmarked or incorrectly scoped containers do not receive namespace treatment. Raw CLI
global-method imports remain unsupported. This is temporary CLI bridge interpretation,
not a general public custom-attribute API or native semantic importer.

### Direct native declaration reading (development, 2026-10-02)

`AssemblyDefinition.ReadNativeAssembly(ReadOnlySpan<byte> image)` reads API-produced
PE/#Neo schema-1/2 containers into the existing definition model. This first profile
admits nongeneric namespace functions and fieldless top-level classes with
Int32, Int64, Boolean, String or no-result method signatures. It rejects other types and unsupported
signatures with InvalidDataException rather than silently returning a partial assembly.
Input/container limits and required-schema/binding checks still apply. No dependency
is loaded, CLI assembly generated, method body translated or host reflection used.

`AssemblyDefinition.IsNative` identifies snapshots populated from authoritative native
metadata. Identity and MainModule.AssemblyReferences retain exact declared scopes.
MainModule.Functions/Methods contain the same canonical MethodDefinition objects,
including namespace ownership via MethodDefinition.Namespace. MetadataToken and
EntryPointToken retain validated native origin identifiers, local to the module.
MainModule.Mvid is Guid.Empty because the native manifest declares no MVID; consumers
must not use it alone as a snapshot identity. Profile is null for this execution profile.

`MethodDefinition.TryGetSignature(out MethodSignature? decoded)` returns an immutable
logical signature for native functions, authored methods and the existing bounded
static CLI primitive/vector/generic profiles. It returns false for other loaded CLI
signatures requiring contextual decoding. Void means no result, not an inhabited value.
Existing static primitive/value signature helpers also recognize these native functions.
GetSignature throws NotSupportedException for native methods; there is no CLI blob.
Body remains unsupported for loaded methods; no empty executable body is fabricated.

Write returns a fresh copy of the original complete PE, preserving opaque bodies.
Loaded mutation, builder attachment and native rewriting remain unsupported. The
existing IAssemblyResolver/AssemblyReference.Resolve contract accepts these snapshots
and rechecks exact identity, including version; missing/mismatched dependencies fail.
It supplies metadata definitions, not runtime reflection objects.

```csharp
var assembly = AssemblyDefinition.ReadNativeAssembly(File.ReadAllBytes("Library.dll"));
foreach (var function in assembly.MainModule.Functions) {
    if (function.TryGetSignature(out var signature)) {
        Console.WriteLine($"{function.Namespace}.{function.Name}: {signature!.ParameterTypes.Count} parameters");
    }
}
```

Raven primitive function loading and call imports are implemented in the bounded
development path below. Instance type/member materialization and structural signatures remain pending. Standalone translated System inventory is not
admitted by this PE function-only entry point. Existing CLI readers/writers remain
unchanged. These host C# APIs are covered here rather than the guest RavenDoc selection.


### Native callable imports (development, 2026-10-02)

`AssemblyBuilder.ImportReference(MethodDefinition definition, AssemblyIdentity dependencyCoreLibrary)`
now accepts primitive nongeneric native namespace-function definitions obtained from
ReadNativeAssembly. It returns the existing immutable ImportedMethodReference; Call
and raw call operands use the same native reference encoding. Namespace, overload
signature and exact dependency identity are retained. No producer body is copied.

The explicit dependency core must match the output core. Existing identity/resource
limits apply. InvalidDataException reports unsupported contracts, incompatible cores
or conflicting snapshots. Native snapshots are compared by a cached SHA-256 of the
complete owned PE, since their Mvid is empty; identical rereads share an import, while
byte-different images under one identity are rejected even if logically equivalent.
CLI snapshots retain their MVID comparison. Snapshot fingerprints are not public or
persistent assembly identity and do not change the encoded metadata format.

```csharp
var dependency = AssemblyDefinition.ReadNativeAssembly(File.ReadAllBytes("Library.dll"));
var target = dependency.MainModule.Functions.Single(f => f.Name == "Twice");
var imported = output.ImportReference(target, explicitCoreIdentity);
var main = output.AddFunction("Main");
main.LoadConstant(21);
main.Call(imported);
main.Return();
output.EntryPoint = main;
```

Global/native namespace calls support native emission, not ordinary CLI output.
Import does not perform source accessibility checking or discover/load dependencies;
Raven performs semantic access checks and neoCLR resolves explicit runtime modules.
Generic/richer-signature native callable imports remain outside this read profile. These host
C# APIs continue to use this manual reference instead of the guest RavenDoc inventory.


### Direct native static types (development, 2026-10-02)

ReadNativeAssembly additionally admits fieldless, nongeneric top-level static classes
with primitive static method signatures. Module.Types contains canonical TypeDefinition
objects; each TypeDefinition.Methods member links back through DeclaringType. These
members do not appear in Module.Functions. Public/internal type visibility and
public/internal/private member visibility are preserved using the existing CLI-shaped
attributes. Type tokens retain validated native origin tokens (the first is 0x02000002).
TypeReference.Resolve returns the same owned definition. Loaded collections remain
read-only and Write preserves the original complete image.

AssemblyBuilder.ImportReference accepts these primitive static methods through the
existing callable overload. Value/interface/nested/generic types, fields,
properties and nonprimitive signatures still reject the entire read with
InvalidDataException; no partial type inventory is silently exposed. This adds no new
encoding or public API surface. Compared with .NET metadata, ownership and access flags
use the same shape; the supported native declaration profile remains narrower.


### Direct native instance classes (development, 2026-10-02)

ReadNativeAssembly also admits fieldless nongeneric top-level instance classes. Existing
TypeDefinition.Attributes retains the native class's nonabstract/nonsealed category;
MethodDefinition.IsStatic reflects receiver presence. Constructor definitions use .ctor,
SpecialName/RTSpecialName flags and a no-result logical signature. The explicit signature
parameters exclude the receiver. TryGetSignature accepts these declarations, while all
TryGetStatic* signature helpers reject instance methods and constructors.

ImportReference(TypeDefinition, core) admits the public instance-class identity for
locals/receivers. ImportReference(MethodDefinition, core) accepts public primitive
instance methods and constructors through its existing contract. ImportedMethodReference
IsConstructor/IsStatic and its declaring identity govern NewObject/Call emission; no
reflection object or CLI signature blob is synthesized. Existing core, exact identity,
snapshot and accessibility restrictions remain. Raven performs source access checks.

The loaded definition graph remains immutable. Fields/properties, value/interface/
nested/generic classes and nominal parameter/result signatures are rejected by the
current direct reader. Existing writer/runtime support is broader than this read profile.
No additional guest API or RavenDoc type is introduced; this manual covers the host APIs.


### Direct native primitive fields (development, 2026-10-02)

ReadNativeAssembly now also admits Int32, Int64, Boolean and String instance fields on
nongeneric top-level classes. TypeDefinition.Fields and Module.Fields expose canonical
FieldDefinition instances; GetFieldDefinition(originToken) returns that same instance.
DeclaringType, name, access and InitOnly/readonly flags retain their native contracts.
Field tokens retain validated origin identities, starting at 0x04000001.

FieldDefinition.TryGetPrimitiveType(out PrimitiveType type) recognizes these native
fields without inventing CLI signature bytes. GetSignature throws NotSupportedException
for native fields. FieldType remains the authored-only signature property (null for
loaded declarations); use the existing primitive query in this bounded read profile.
Loaded field names and collections remain immutable, and Write preserves the input PE.
Nominal/vector/structural field signatures still reject the entire direct read.

Raven exposes these native fields as compiler symbols, retaining type/access/readonly
information. A native library's constructor and instance methods can initialize/read its
own fields using existing source emission. Direct consumer field emission across the
assembly boundary is not implemented: it reports NEOMETA001 and leaves output empty.
The stateful method-call consumer is tested separately and returns 42. No guest API,
encoded layout change or new public host member is introduced by this reader slice.


### Imported primitive field operands (development, 2026-10-02)

`AssemblyBuilder.ImportReference(FieldDefinition definition, AssemblyIdentity dependencyCoreLibrary)`
returns an interned `ImportedFieldReference` for a public Int32/Int64/Boolean/String
instance field on a public nongeneric top-level reference class. The owner must contain
only instance fields; constructed/value/interface/static owners and translated layout
bindings are unsupported. Null arguments throw ArgumentNullException. Unsupported
contracts, core/snapshot conflicts and the 4096-import limit throw InvalidDataException.
Existing exact type identity and snapshot checks apply before a cached reference is reused.
Authored/detached mutable field definitions reject; read an immutable snapshot first.

ImportedFieldReference exposes read-only Owner (consuming AssemblyBuilder), DeclaringType
(ImportedTypeReference), Name, FieldType (primitive SignatureType) and IsReadOnly. It owns
no mutable producer graph or executable body. Native field ordinals remain internal.

MethodBuilder.LoadField(ImportedFieldReference), StoreField(ImportedFieldReference) and
Emit(OpCode, ImportedFieldReference) support Ldfld/Stfld. Null operands throw
ArgumentNullException; wrong opcodes/foreign output ownership throw ArgumentException.
Write validates the exact receiver, primitive value stack types and readonly restriction;
violations throw InvalidDataException. Imported readonly fields may be loaded but never
stored by the external consumer, including its constructors.

CLI writing emits a TypeRef-scoped field MemberRef and ordinary ldfld/stfld instructions.
Native writing requires an imported **native** definition snapshot so the field ordinal
is established by validated native metadata. A CLI field snapshot can emit CLI but
native writing rejects it: no CLI-to-native layout equivalence is guessed. Supply the
exact matching native library artifact to the runtime. The existing native field opcode
format is unchanged.

```csharp
var library = AssemblyDefinition.ReadNativeAssembly(File.ReadAllBytes("Library.dll"));
var field = library.MainModule.Types.Single(t => t.Name == "Calculator")
    .Fields.Single(f => f.Name == "Visible");
var imported = output.ImportReference(field, explicitCoreIdentity);
body.LoadLocal(receiver); // local uses imported.DeclaringType
body.LoadConstant(42);
body.StoreField(imported);
body.LoadLocal(receiver);
body.Emit(OpCode.Ldfld, imported);
```

C# contract tests execute CLI field accesses on the CLR from both CLI and native
read-model inputs, and cover readonly/receiver/opcode/ownership/core/snapshot rejection.
Raven consumers execute direct native field stores and loads across assemblies (42).
These host C# APIs use this manual reference and remain outside guest RavenDoc selection.

### Native local nominal signatures (development, 2026-10-02)

`SignatureType.ReferencedType { get; }` returns `TypeReference?`: a nominal reference
owned by a loaded definition snapshot, or null for other signature categories.
For the direct native reader's current profile, `ReferencedType.Resolve()` requires no
resolver and returns the canonical local TypeDefinition. `ClassType` and `ImportedType`
remain null for this category; no mutable builder is retained. `ToString()` provides a
diagnostic namespace/name, not serialized identity. The getter performs no code loading.

`AssemblyDefinition.ReadNativeAssembly` and `MethodDefinition.TryGetSignature` now support
nongeneric local root class parameter/result types as well as primitives, on namespace
functions, class methods and constructors. The native format and opaque Write roundtrip
are unchanged. Cross-dependency, generic, interface/value, array/byref signature categories,
nominal fields and properties remain outside this read profile and throw InvalidDataException.

`AssemblyBuilder.ImportReference(MethodDefinition, core)` imports each referenced local
class through the existing exact identity/snapshot/core checks and produces output-owned
ImportedTypeReference signature operands. Existing public-type/access restrictions apply;
unsupported imports throw InvalidDataException. Passing a loaded nominal SignatureType
directly to a builder definition throws ArgumentException: import its declaring method
or resolve and explicitly import its type first. This separation prevents a loaded
snapshot from acquiring an output builder owner.

The static Int32, primitive and primitive-vector signature helpers continue to return
false for nominal signatures. Use TryGetSignature to inspect the richer native profile.
C# tests read both native container variants and execute imported factory/instance calls
on the CLR; Raven tests compile and execute factories, nominal function/method signatures
and constructor arguments in neoCLR. These are host development APIs, not guest APIs.

### Native local nominal fields (development, 2026-10-02)

`FieldDefinition.TryGetSignature(out SignatureType? type)` returns true for authored
field types, direct native primitive/local class fields and primitive CLI field blobs.
It returns false with null for other loaded CLI signatures; unsupported does not mean
absent. No body or dependency is loaded. A native nominal signature's ReferencedType
resolves to its snapshot's canonical TypeDefinition, shared with method signature types.
Forward/cyclic local references are supported. Loaded FieldType remains the existing
authored-only property; use TryGetSignature for a loaded field's logical type.

ReadNativeAssembly now admits fields referring to local nongeneric root classes.
Unsupported external, array, generic, value/interface field shapes still reject the
read with InvalidDataException. Native GetSignature still throws NotSupportedException
because no CLI blob exists; TryGetPrimitiveType returns false/Void for nominal fields.
Loaded declarations remain immutable and Write still copies the original native image.

`AssemblyBuilder.ImportReference(FieldDefinition, core)` additionally accepts these
native nominal fields when both the owner and referenced storage type satisfy existing
public class import rules. ImportedFieldReference.FieldType contains an output-owned
ImportedTypeReference signature. LoadField, StoreField and raw Ldfld/Stfld operands use
the existing exact receiver/value checks and readonly-store rejection. Null/foreign
operands retain their existing errors; unsupported signatures/core/snapshot/layout
contracts throw InvalidDataException. CLI writing emits a nominal field MemberRef;
native writing uses the existing validated ordinal. CLI nominal field decoding/import
remains outside this bounded profile. No PE/#Neo schema or opcode change is introduced.

C# tests cover forward/cyclic references, field/method signature identity, both native
containers, immutable fields, incorrect primitive/nominal stores and readonly stores.
They execute the emitted equivalent on .NET (42); Raven consumers execute replacement,
nested mutation and original-object independence in neoCLR (42).

### External native nominal signatures (development, 2026-10-02)

ReadNativeAssembly now admits nongeneric top-level reference classes from declared
native dependencies in method/function/constructor and field signatures. The reader
copies their exact assembly identity, namespace and name into TypeReference rows in
ModuleDefinition.TypeReferences. ReferencedType points to that same immutable reference.
Native TypeRef tokens are reader-assigned, module-local identifiers, not claimed CLI
rows or native origin tokens. ResolutionScopeToken identifies the declared AssemblyRef.
Read/inspection does not load dependencies or resolve types. Write preserves the original
PE/#Neo image and introduces no format or instruction change.

`TypeReference.Resolve(IAssemblyResolver? resolver = null)` retains its existing contract:
external references require an explicit resolver returning an exact-identity snapshot;
missing/wrong dependencies or missing/ambiguous types throw InvalidDataException.
No filesystem probing, simple-name fallback or runtime reflection loading occurs.
Local references still resolve without a resolver.

New overloads:

```csharp
ImportedMethodReference AssemblyBuilder.ImportReference(
    MethodDefinition definition, AssemblyIdentity dependencyCoreLibrary,
    IAssemblyResolver? resolver);
ImportedFieldReference AssemblyBuilder.ImportReference(
    FieldDefinition definition, AssemblyIdentity dependencyCoreLibrary,
    IAssemblyResolver? resolver);
```

These import native nominal signature types through the supplied resolver into the
output's existing interned nominal references. All resolved types must be public,
nongeneric top-level reference classes, and the supplied core contract applies to all
of them. Exact snapshot/core checks remain in force. Existing two-argument overloads
remain available and behave as before; external native signatures require a resolver.
Null definition/core throws ArgumentNullException; a null resolver is permitted for
local-only contracts. Missing/mismatched/unsupported dependencies or conflicting snapshots
throw InvalidDataException. Resolver exceptions retain host meaning. Primitive helpers
continue to reject nominal signatures. CLI cross-dependency signature decoding is not
expanded by these overloads.

C# tests inspect both native containers, check scoped reference/field/method identity,
reject missing resolver/version/type/snapshot conflicts, and execute a three-assembly
consumer on .NET (42). Raven compiles the same dependency shape and executes it in neoCLR.
Generic, constructed, value/interface and array signature profiles remain unsupported.

### Native vector signatures (development, 2026-10-02)

ReadNativeAssembly, MethodDefinition.TryGetSignature and FieldDefinition.TryGetSignature
now admit one-dimensional zero-based vectors of supported primitives or nongeneric root
reference classes. SignatureType.ArrayElement retains the primitive kind or immutable
nominal reference; external class elements use the existing explicit resolver contract.
Field and method signatures share the same nominal element identity. Opaque Write still
copies the original image. No format or public API signature change is introduced.

Method/field ImportReference overloads recursively map array elements into output-owned
signatures and retain exact dependency/snapshot checks. Loaded nominal array signatures
must be imported before builder use; using one directly throws ArgumentException.
Array operands continue to use existing exact element-type validation. Wrong array
arguments or field stores throw InvalidDataException on write. No covariance, jagged,
multidimensional, generic, value/interface element or byref-array support is added.
Unsupported native declaration shapes still reject the entire read.

TryGetStaticValueSignature recognizes native primitive vectors, matching its existing
bounded CLI profile; nominal vectors return false. TryGetStaticPrimitiveSignature and
TryGetPrimitiveType continue to reject all vectors. Use the logical signature APIs to
inspect nominal arrays. CLI field decoding remains limited to its existing primitive
profile; native array field imports can emit either CLI MemberRefs or native ordinals.

C# checks cover both native containers, primitive vector helper behavior, nominal element
identity, field/method signature equality, wrong-element arguments/stores and .NET execution
of native array imports (42). Raven's three-assembly consumer stores an external-class
array in a field, replaces an element through an alias and passes a primitive array
across the same boundary; neoCLR returns 42.

### Native non-indexed properties (development, 2026-10-02)

ReadNativeAssembly now materializes non-indexed properties on the supported root
class profile. ModuleDefinition.Properties, TypeDefinition.Properties and
GetPropertyDefinition return the same snapshot-owned PropertyDefinition objects.
DeclaringType, GetMethod and SetMethod refer to canonical definitions; accessor
visibility and staticness are preserved, with SpecialName on accessor methods.
MetadataToken is the validated native property origin token. Missing accessors remain
null, including read-only and write-only properties; OtherMethods is empty.

```csharp
public bool TryGetSignature(out SignatureType? type, out bool isStatic);
```

PropertyDefinition.TryGetSignature returns the logical property value type and
staticness for supported native properties, authored non-indexed properties and
primitive non-indexed CLI properties. It returns false, null and false for other
loaded CLI signatures and indexers. Native types include supported primitives,
local/external nominal classes and their vectors. References belong to the immutable
snapshot; dependencies resolve explicitly through the existing resolver contract.
GetSignature throws NotSupportedException for native properties because they have no
CLI blob. PropertyType and ParameterTypes remain authored-only APIs; use the logical
reader for loaded native properties. TryGetPrimitiveSignature recognizes native
primitive properties and rejects nominal/vector properties with Void/false outputs.

Import the canonical accessor methods through AssemblyBuilder.ImportReference to emit
property calls; properties themselves are associations rather than CIL call operands.
Existing exact identity, resolver and snapshot checks apply. An unchanged native
assembly still writes its original bytes. Indexed properties, generic/value/interface
owners and unsupported signatures fail the whole direct read with InvalidDataException.
No property schema or runtime opcode changes are made. C# checks cover both containers,
ownership, signatures, static/read-only/write-only accessors, CLR accessor execution,
opaque roundtrip and indexer rejection. Raven consumes separately compiled native
properties through the same accessor definitions.

### Native indexed property signatures (development, 2026-10-02)

ReadNativeAssembly now also admits indexed properties in the existing supported
root-class profile. Ordered index types are immutable signature shapes, with the same
primitive/nominal/vector support and exact dependency resolution as value types.
Accessor associations retain canonical method identity and their existing visibility.

```csharp
public bool TryGetSignature(out SignatureType? type,
    out IReadOnlyList<SignatureType> parameters, out bool isStatic);
```

The new PropertyDefinition overload returns the value type, ordered index parameters
(excluding the setter value), and accessor staticness. It supports authored/native
properties, including getter-only and setter-only indexers, and the existing bounded
non-indexed primitive CLI profile. On failure outputs are null, an empty list and false.
The returned parameter collection is read-only. It does not resolve dependencies or
materialize bodies. The existing two-output overload and TryGetPrimitiveSignature
continue to reject indexers. PropertyType/ParameterTypes remain authored-only; native
GetSignature still throws NotSupportedException rather than inventing a CLI blob.

C# tests cover both containers, authored/native signatures, immutable parameter lists,
setter-value exclusion, accessor identity, byte-preserving roundtrip and CLR execution
of imported indexer methods. Raven now consumes setter-only indexers as well as
read/write overloads through the same property parameter contract. Source reads and
compound assignments require an accessible getter. No CLI/native property encoding
or runtime opcode changes.

### Direct native interface definitions (development, 2026-10-02)

ReadNativeAssembly now admits nongeneric top-level interfaces and local interface
inheritance/root-class implementation relationships. TypeDefinition.Attributes retain
CLI Interface/Abstract flags; interface methods retain Abstract/Virtual/NewSlot flags.
Existing property/accessor and primitive/nominal/vector signature APIs apply.

TypeDefinition.Interfaces returns a cached, read-only `IList<InterfaceImplementation>`
for native snapshots, including an empty list for owners with no contracts. Each
relationship has the exact DeclaringType and a reference resolving to the canonical
local interface definition; TypeArguments is empty in this profile. Mutations throw
NotSupportedException. Authored collections remain append-only; loaded CLI relationship
materialization still throws NotSupportedException. Concurrent reads share relationship
identity. Unchanged native images still write their original bytes.

AssemblyBuilder.ImportReference now accepts interface-valued native signatures.
Imported abstract interface methods require CallVirtual. Body validation admits native
class/interface-to-interface conversions only when exact imported snapshots prove an
inheritance/implementation path; unrelated interfaces still reject. This is cached
nominal conformance, not duck typing, generic variance or an unrestricted reference cast.
Existing core/identity/snapshot checks apply. Generic/value/nested owners, external
interface implementation edges and loaded CLI relationship decoding remain unsupported.

C# tests cover both native containers, canonical immutable relationships, abstract
flags, byte-preserving roundtrip, unrelated-interface rejection and CLR execution of
accessors imported from native definitions. Raven separately compiles interface
factories and a consumer; neoCLR executes inherited method/property dispatch (42).

Interface-valued fields and arrays are also covered by C# import/emission tests.
A concrete native class with a proven local implementation path can be stored in
an imported interface field or exact-interface array and dispatched on CLR. Raven's
three-assembly consumer verifies equivalent native storage and alias behavior. These
use existing FieldDefinition and MethodDefinition signature/import APIs; no new public
API or array covariance rule is introduced.

### Direct native static generic methods (2026-10-02 development)

`AssemblyDefinition.ReadNativeAssembly` now also admits unconstrained static generic
methods and namespace functions on nongeneric owners. `MethodDefinition.GenericArity`
and `TryGetSignature` preserve parameter names, positional `SignatureType.MethodParameter`
references and their one-dimensional vectors. `TryGetStaticGenericValueSignature`
recognizes the primitive/parameter/vector subset; nongeneric helpers reject even unused
generic parameters. `GetSignature` still rejects native CLI-blob requests.

`AssemblyBuilder.ImportReference(MethodDefinition, AssemblyIdentity)` retains this
signature; `ImportedMethodReference.MakeGenericInstance` uses the existing arity and
consumer-ownership checks. `MethodBuilder.Call` accepts that instantiated reference.
Static methods can emit CLI MemberRef/MethodSpec or native generic calls; namespace
functions remain native-only imports. Both native container schemas are covered.

Generic owners, generic instance methods, constraints, byrefs and nested vectors remain
outside this direct-reading profile and throw InvalidDataException. Loaded definitions
are immutable; Write preserves the original image. Raven's native symbol layer now consumes this same static generic profile through
method-owned parameters and shared constructed-method substitution.
Native parameter names must also satisfy runtime slot-name rules; the existing writer
can accept reserved names such as `Value` that runtime verification rejects. This writer
validation mismatch remains a follow-up, not an accepted runtime contract.

### Direct native generic root classes (2026-10-02 development)

`AssemblyDefinition.ReadNativeAssembly` now also admits unconstrained generic root
classes with bounded members. `TypeDefinition.GenericArity` and the read-only
`GenericParameterNames` retain the metadata arity/name suffix and declared parameter
names. CLI snapshots still return null for GenericParameterNames. `SpecialConstraints`
and `GenericConstraints` return empty read-only views for the supported native profile;
constrained native declarations remain rejected, not silently stripped.

`MethodDefinition.TryGetSignature`, `FieldDefinition.TryGetSignature` and property
signature queries preserve `SignatureType.TypeParameter(ordinal)` and its vectors.
`AssemblyBuilder.ImportReference(TypeDefinition, core)` creates the open reference;
`MakeGenericInstance` constructs it. Imported constructors and methods use the existing
`ImportedMethodReference.MakeConstructedReference` API and shared substitution. They
emit CLI or native calls without projecting native definitions to CLI metadata.

The bounded profile does not yet materialize constructed nominal signatures such as
`Box<int>` in a declaration, generic interface/static owners or generic inheritance.
Native generic constraints, nested/value owners and instance generic methods still
reject. Direct imported fields on constructed owners remain unsupported; use supported
methods/properties. Both container schemas execute `Box<int>` construction, Set and Get
on neoCLR; the equivalent CLI consumer executes on CLR (42).

### ReferencedGenericType (development 2026-10-02)

Namespace: `NeoCLR.Metadata.Experimental.Model`. The new read-only
`SignatureType.ReferencedGenericInstance` property returns a `ReferencedGenericType`
for loaded constructed signatures such as `Box<int>`; otherwise null. Its public members:

- `TypeReference Definition`: canonical snapshot-scoped generic definition reference.
- `IReadOnlyList<SignatureType> TypeArguments`: copied, immutable arguments in ordinal order.
- `Equals(ReferencedGenericType?)` / `Equals(object?)`: definition reference identity plus
  structural ordered argument equality; null and other categories are unequal.
- `GetHashCode()`: agrees with that equality; it is not a stable serialized identity.
- `ToString()`: diagnostic display, not a serialized type name.

`ReadNativeAssembly` now retains local closed generic root-class constructions in the
supported field/property/method signature categories, including vectors. Arguments can
include bounded primitive, nominal, vector and nested closed construction signatures.
Scoped parameters inside a local construction (`Box<T>`) are now retained, including
method and owner parameters and vectors of constructions. Ordinals are validated against
the declaring scope by the native reader. External generic constructions now retain exact assembly-scoped definition references
and ordered arguments too. Resolving/importing them requires the explicit resolver
overload, just like external nongeneric signatures. Missing or wrong-version dependencies
throw InvalidDataException. Constraints and generic inheritance remain unsupported.

Loaded construction signatures are not builder operands: using one directly in an
authored method fails ownership validation with ArgumentException. Import the containing
declaration using AssemblyBuilder.ImportReference; that recursively imports the owner
and arguments into output-owned references. Existing CLI/native constructed-call
encoding is reused. No synthetic CLI blobs or reflection types are introduced.


## Authored function references (development, 2026-10-02)

`AssemblyBuilder.CreateFunctionReference(AssemblyIdentity dependency,
AssemblyIdentity dependencyCoreLibrary, string artifactSha256, string namespace,
string name, MethodSignature signature) -> ImportedMethodReference` creates a native
assembly/namespace-function reference from resolved values. It needs no reader,
MethodDefinition or resolver. The returned reference is owned by the output builder;
matching name/parameter/arity contracts are interned. Use Call or MakeGenericInstance
on the result through the existing emission API.

The dependency must be unsigned, distinct from the output, and use the same explicit
core identity. The digest must contain 64 hexadecimal SHA-256 digits; its comparison
is case-insensitive. It identifies the host-selected native image for conflict checks
within the output, not an encoded runtime integrity guarantee. The caller is responsible
for the truth and completeness of the contract and for supplying the matching runtime
artifact. No image loading, accessibility checking or dependency verification occurs.

Supported signatures contain Int32, Int64, Boolean, String, a Void result, method-owned
generic parameters, output-owned external top-level nominal signatures (reference and value types, including
closed/open constructions), and single-dimensional vectors of supported non-Void scalars.
Construction arguments are checked recursively, including method-parameter scope and
output ownership. Nested types, owner parameters, byrefs and out parameters are
rejected. The caller must supply the nominal declaration facts; this does not import
inheritance or interface conversion information. Function names
and namespaces obey the existing authored declaration rules. A required null argument
throws ArgumentNullException; malformed digests, names/namespaces or parameter scope
throw ArgumentException. Unsupported contracts, incompatible core/identity, snapshot
conflicts, inconsistent result signatures and resource limits throw InvalidDataException.
Native output retains the existing format-5 name/signature linking contract; ordinary
CLI output rejects references to assembly-level functions.

```csharp
var signature = new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]);
var call = output.CreateFunctionReference(dependencyIdentity, coreIdentity,
    dependencyImageSha256, "Example", "Identity", signature);
main.LoadConstant(42);
main.Call(call);
main.Return();
```

The C# AuthoredFunctionReferenceChecks exercises creation without a reader and rejection
contracts. NativeGenericMethodChecks reconstructs a generic vector reference from
values and executes it in both native containers. The separate library IILGenerator
API remains planned; the snippet uses the current MethodBuilder operations.


## Authored type references (development, 2026-10-02)

`AssemblyBuilder.CreateTypeReference(AssemblyIdentity dependency,
AssemblyIdentity dependencyCoreLibrary, string artifactSha256, string namespace,
string name, int genericArity = 0) -> ImportedTypeReference` authors an output-owned
public top-level reference-class identity from resolved values, without a reader or
resolver. `name` is the metadata name: a generic definition includes its backtick arity
suffix. Arity is 0–32. Parameters are asserted invariant and unconstrained. Construct
an open generic reference with `MakeGenericInstance` before using it in a signature.
Matching references are interned, including subsequent imports from the same snapshot.

Identity/core/digest rules match CreateFunctionReference: unsigned external identity,
matching explicit core, and 64 hexadecimal SHA-256 digits. This does not verify the
image, accessibility, members or generic constraints. It carries no inheritance or
interface conversion information; value/nested/interface declarations are outside this
API's contract. The digest detects conflicting selected snapshots within the output,
not runtime integrity. Mixed reader imports and authored references share the checks.

Null arguments throw ArgumentNullException; a malformed digest throws ArgumentException.
Unsupported identity, metadata name/arity, core mismatch, snapshot conflict and resource
limits throw InvalidDataException. The current limits are 4,096 nominal references and
256 dependency identities. Returned construction arguments are copied and checked against
the consuming output's scope by MakeGenericInstance.

```csharp
var box = output.CreateTypeReference(dependencyIdentity, coreIdentity,
    dependencyImageSha256, "Example", "Box`1", 1);
SignatureType integerBox = box.MakeGenericInstance(PrimitiveType.Int32);
```

AuthoredFunctionReferenceChecks covers reader-free nominal identity, interning, argument
copying and negative contracts. NativeGenericOwnerChecks uses an authored Box reference
alongside imported members, checking canonical identity and execution on CLR and both
native containers. This does not imply that member references are reader-independent.


## Authored method references (development, 2026-10-02)

`AssemblyBuilder.CreateMethodReference(ImportedTypeReference declaringType, string name,
MethodSignature signature, bool isStatic = false, bool isOverride = false) -> ImportedMethodReference` authors a
public concrete nonvirtual root-class member contract without a reader definition.
The declaring type must be an output-owned, top-level reference-class definition whose
dependency/core/artifact contract has already been registered. Pass the generic definition,
not its construction; use `MakeConstructedReference` on the returned member to bind owner
arguments. Static generic methods are supported; instance generic methods are not.

Signatures admit primitive values, a Void result, scoped owner/method parameters,
output-owned external reference-class constructions and vectors. `.ctor` requires an
instance nongeneric Void signature. Later extensions documented below admit ref/out parameters, nested/value owners and bounded value overrides. General class virtual contracts remain unsupported. Authored interface dispatch is supported as
described below. No members or access rules are resolved
from the dependency: the caller asserts the semantic contract. This API does not imply
that an arbitrary interface reference can be treated as a reference-class owner.

Matching authored contracts are interned. Conflicting static/result contracts with the
same owner/name/arity/parameter signature reject. Null arguments throw
ArgumentNullException; invalid owner/name/constructor or parameter scope throws
ArgumentException; unsupported signature, conflicting contract or the shared 4,096
callable-reference limit throws InvalidDataException. Foreign output signatures reject.
Existing artifact checks and writer signature/stack validation still apply.

```csharp
var box = output.CreateTypeReference(dependencyIdentity, coreIdentity,
    dependencyImageSha256, "Example", "Box`1", 1);
var constructor = output.CreateMethodReference(box, ".ctor",
    new MethodSignature(PrimitiveType.Void, [SignatureType.TypeParameter(0)]));
main.LoadConstant(42);
main.NewObject(constructor.MakeConstructedReference([PrimitiveType.Int32]));
```

NativeGenericOwnerChecks reconstructs constructor/Get/Set/Same contracts from values,
then executes construction, mutation and self-typed calls on CLR and both native
containers. AuthoredFunctionReferenceChecks covers interning and invalid owner/scope/
constructor/conflicting contracts. The separate library IILGenerator remains planned.


## Authored field references (development, 2026-10-02)

`AssemblyBuilder.CreateFieldReference(ImportedTypeReference declaringType, string name,
SignatureType fieldType, int instanceStorageOrdinal, bool isReadOnly = false)
-> ImportedFieldReference` authors a public instance-field contract without a reader.
The owner must be an output-owned top-level root reference-class definition with a
registered native artifact digest. The caller supplies the native zero-based instance
slot, including private fields in the declaration order, and must assert that this
layout matches the selected artifact. The library cannot verify a supplied ordinal
against bytes it has not read. CLI output uses name/type MemberRef; native output uses
the ordinal. This does not change the instruction set or encode the artifact digest.

Storage supports primitive non-Void values, external reference classes (including
generic constructions) and single vectors thereof. Owner type parameters are permitted
within the declaring definition's arity. Construction arguments are checked recursively;
method parameters, out-of-scope owner parameters, bare generic definitions and foreign
output signatures reject. Static/inherited fields, byrefs and value
profiles are unsupported. Readonly loads work; stores reject during body validation.
Matching contracts intern; conflicting names, ordinals, storage or readonly flags on
the same owner reject. Null arguments throw ArgumentNullException; invalid owner,
name, negative ordinal, unsupported type or foreign output signatures throw
ArgumentException. Conflicts or the shared 4,096-field limit throw InvalidDataException.

```csharp
var value = output.CreateFieldReference(itemReference, "Value", PrimitiveType.Int32,
    instanceStorageOrdinal: 2);
method.LoadArgument(0);
method.LoadField(value);
method.Return();
```

C# authored-reference checks cover body writing, interning, slot conflicts, invalid
owners/ordinals and readonly-store rejection. Raven's native consumers exercise direct
field load/store and alias mutation with private fields preceding public fields.


## Authored interface contracts (development, 2026-10-02)

`AssemblyBuilder.CreateInterfaceReference(AssemblyIdentity dependency,
AssemblyIdentity dependencyCoreLibrary, string artifactSha256, string namespace,
string name) -> ImportedTypeReference` authors a public nongeneric top-level interface.
The overload with a final `int genericArity` authors an unconstrained invariant generic
interface; the metadata name includes its arity suffix. The original overload remains.
Identity/core/digest/name validation, errors and limits follow CreateTypeReference.
Class/interface classification conflicts throw InvalidDataException, including when
mixing authored references and reader imports. The caller asserts truthful declaration
facts; the dependency is not loaded or inspected.

`AssemblyBuilder.AddInterfaceConversion(ImportedTypeReference source,
ImportedTypeReference target) -> void` registers direct interface inheritance or class/value
implementation. Both references belong to this output; the source is a top-level
nominal definition (possibly generic), and the target is an interface identity or
construction from CreateInterfaceReference. Target arguments may refer to the source's
type parameters; out-of-scope arguments reject. Traversal substitutes arguments
simultaneously, preserving distinct constructions and rejecting declaration cycles.
Edges are idempotent and transitive conversions are derived using graph traversal.
Null endpoints throw ArgumentNullException; foreign/unsupported endpoints, noninterface
targets and cycles throw ArgumentException. More than 4096 distinct direct edges throws
InvalidDataException. Register relationships before writing bodies that depend on them.
This records conversions; it does not generate implementations in the runtime dependency.

CreateMethodReference on an authored interface now creates a nongeneric abstract instance
contract requiring virtual dispatch. Static members, constructors and generic methods
reject. IsInterfaceMethod and RequiresVirtualDispatch are true, and CallVirtual emits the
existing dispatch instruction. Classes retain the nonvirtual contract. Native field
ordinals and artifact checks are unchanged.

C# checks cover transitive class-to-interface return conversion, inherited interface
calls, cyclic/invalid edges and conflicting nominal classification. Raven supplies
relationships from symbols and executes its interface inheritance, alias storage and
method/property dispatch samples. Generic interfaces and class inheritance remain
outside this authoring slice. The library instruction-generator API is still pending.


## IILGenerator (development, 2026-10-02)

`MethodBuilder.GetILGenerator() -> IILGenerator` returns a stable generator for the
method's authored definition. `MethodDefinition.GetILGenerator() -> IILGenerator`
returns that same instance for an attached authored definition; detached or loaded
opaque definitions throw InvalidOperationException. Body generation does not turn a
loaded snapshot into an editable authored body.

The public library interface is independent of Raven's compiler emission interfaces.
Raven's NeoCLR adapter translates its portable instruction stream into this library API.
Builders describe declarations; the generator authors the existing definition-owned
body. The generator owns append operations, locals/labels and immediate operand validation.
Old public builder instruction methods forward to that generator as compatibility entry
points. Write-time graph/stack validation still uses the existing writer path and internal
operation representation; relocating those is separate from body authoring. There is no
duplicate instruction buffer or changed encoding, opcode semantics or validation timing.

`Locals: IReadOnlyList<LocalDefinition>` exposes the existing slot-ordered local view.
All operations below preserve the corresponding MethodBuilder contract documented in
this reference and in their interface XML documentation: exact typed operand validation,
method-local label/local ownership, write-time stack checking and target capability
limits. Wrong opcode/operand or foreign handles throw ArgumentException; null required
operands throw ArgumentNullException; write-time validation/resource failures throw
InvalidDataException. Per-overload exceptions and limits remain as documented for the
matching operation; the interface does not admit new opcodes or operand categories.
ClearBody removes instructions and retains locals, as before. Authoring is not thread-safe.
Instruction insertion/reordering and loaded-body editing are not implemented.

```csharp
var entry = output.AddFunction("Main");
output.EntryPoint = entry;
IILGenerator il = entry.GetILGenerator();
il.Emit(OpCode.Ldc_I4, 40);
il.LoadConstant(2);
il.Add();
il.Return();
```

The complete operation signatures are listed below. These use library metadata handles,
not System.Reflection.Emit or Raven interface types.

```csharp
void LoadConstant(int value);
void WriteConsoleLine(string text);
void WriteConsoleLine();
void LoadArgument(int index);
void StoreArgument(int index);
void Add();
void Subtract();
void Multiply();
void Divide();
void Remainder();
void BitwiseAnd();
void BitwiseOr();
void BitwiseXor();
void ShiftLeft();
void ShiftRight();
void Call(MethodBuilder target);
void Call(ImportedMethodReference target);
void Call(NativeFunctionDefinition target);
void Return();
void ClearBody();
void LoadField(ConstructedFieldReference field);
void StoreField(ConstructedFieldReference field);
void Emit(OpCode opCode, ConstructedFieldReference operand);
void NewObject(ConstructedMethodReference constructor);
void Call(ConstructedMethodReference method);
void CallVirtual(ConstructedMethodReference method);
void Emit(OpCode opCode, ConstructedMethodReference operand);
void BindFunction(SignatureType functionType, MethodBuilder target);
void BindFunction(SignatureType functionType, ConstructedMethodReference target);
void BindFunction(SignatureType functionType, GenericMethodInstance target);
void Emit(OpCode opCode, FunctionBinding operand);
void InvokeFunction(SignatureType functionType);
void Call(GenericMethodInstance method);
void Emit(OpCode opCode, GenericMethodInstance operand);
void NewObject(ImportedMethodReference constructor);
void NewObject(ImportedConstructedMethodReference constructor);
void Call(ImportedConstructedMethodReference method);
void CallVirtual(ImportedConstructedMethodReference method);
void CallVirtual(ImportedMethodReference method);
void Emit(OpCode opCode, ImportedConstructedMethodReference operand);
void LoadField(ImportedFieldReference field);
void StoreField(ImportedFieldReference field);
void Emit(OpCode opCode, ImportedFieldReference operand);
void Call(ImportedGenericMethodReference method);
void Emit(OpCode opCode, ImportedGenericMethodReference operand);
void CallVirtual(MethodBuilder target);
void Emit(OpCode opCode, SignatureType elementType);
void CastReference(SignatureType target);
void NewArray(SignatureType elementType);
void ReserveArray(SignatureType elementType);
void LoadArrayElement(SignatureType elementType);
void StoreArrayElement(SignatureType elementType);
void LoadArrayLength();
void Emit(OpCode opCode);
void Emit(OpCode opCode, int operand);
void Emit(OpCode opCode, long operand);
void Emit(OpCode opCode, string operand);
void Fail(string message);
void Emit(OpCode opCode, MethodBuilder operand);
void Emit(OpCode opCode, ImportedMethodReference operand);
void Emit(OpCode opCode, NativeFunctionDefinition operand);
void Emit(OpCode opCode, bool operand);
void Emit(OpCode opCode, FieldBuilder operand);
void Duplicate();
void NewObject(MethodBuilder constructor);
void LoadField(FieldBuilder field);
void StoreField(FieldBuilder field);
BranchLabel DefineLabel();
void MarkLabel(BranchLabel label);
void Emit(OpCode opCode, BranchLabel label);
void LoadLocalAddress(LocalDefinition local);
void InitializeObject(SignatureType type);
void LoadDefault(SignatureType type);
LocalDefinition DeclareInt32Local();
LocalDefinition DeclareLocal(PrimitiveType type);
LocalDefinition DeclareLocal(TypeBuilder type);
LocalDefinition DeclareLocal(SignatureType type);
void LoadLocal(LocalDefinition local);
void StoreLocal(LocalDefinition local);
void Emit(OpCode opCode, LocalDefinition local);
void LoadObject(SignatureType type);
void StoreObject(SignatureType type);
```

GeneratorChecks verifies identity, definition access, scope/operand rejection, local
preservation and CLR execution (42). NativeGenericOwnerChecks uses the interface for
construction/mutation/calls and executes on CLR and both native containers. All seven
Raven native consumers use the new interface through the backend adapter.


Static container validation (2026-10-02): CreateTypeReference may supply declaration
identity for an authored static method reference on a static class. This does not assert
that the container is an instantiable signature value. Raven keeps these admissions
separate. NativeGenericMethodChecks now authors static generic references from values
for both CLR and native consumers, without passing reader definitions.


Closed generic field validation (development, 2026-10-02):
NativeGenericOwnerChecks authors scalar/vector fields containing an imported `Box<Int32>`,
loads and stores them, and executes the resulting CLI assembly on .NET and native
assembly on both containers. AuthoredFunctionReferenceChecks rejects open parameters,
unconstructed generic definitions and foreign references. This extends storage
signatures only; the field's declaring owner must still be nongeneric. No new API
signature or metadata encoding is introduced.


### ImportedConstructedFieldReference (development, 2026-10-02)

`ImportedFieldReference.MakeConstructedReference(params SignatureType[] typeArguments)`
returns an immutable constructed field with `Definition`, `DeclaringType` and substituted
`FieldType` properties. The definition may now contain owner type parameters (including
vectors and nominal constructions). Arguments are copied; wrong arity, nongeneric owner,
Void or foreign arguments throw ArgumentException, and null arguments throw
ArgumentNullException. Substitution is simultaneous and preserves caller parameters.

`IILGenerator` and the forwarding `MethodBuilder` offer `LoadField`, `StoreField` and
`Emit(OpCode, ImportedConstructedFieldReference)`. Only Ldfld/Stfld are admitted. Null
operands throw ArgumentNullException; foreign, unconstructed generic, wrong-opcode or
out-of-scope operands throw ArgumentException. Writing validates receiver, storage and
readonly rules and throws InvalidDataException for invalid bodies. CLI writes a MemberRef
with open storage signature and a constructed parent; native writes the existing slot.
These host APIs are covered by this manual reference, not the guest RavenDoc snapshot.


### Generic native relationship materialization (development, 2026-10-02)

ReadNativeAssembly now admits unconstrained invariant top-level generic interfaces and
classes implementing them. TypeDefinition.Interfaces exposes immutable relationships;
InterfaceImplementation.TypeArguments preserves open owner parameter ordinals or closed
arguments, and InterfaceType resolves the canonical open definition. Relationships
currently target interfaces declared in the same native assembly. Cyclic/invalid
relationships reject during reader validation. Loaded CLI relationship materialization
remains pending; no reflection facade or projection is introduced.

ImportReference(MethodDefinition, core) supports abstract generic-owner interface
methods; bind owner arguments with MakeConstructedReference and emit CallVirtual.
ImportReference(FieldDefinition, core) now supports native generic-owner fields; bind
arguments with MakeConstructedReference and emit LoadField/StoreField. Native storage
and dispatch encodings are unchanged. Reader-derived and explicitly authored interface
conversion paths both substitute and validate invariant arguments.

Validation: NativeGenericOwnerChecks runs reader and authored imports on .NET and
executes the authored field/interface consumer with both native containers. Raven tests
scope identity, both dependency orders, incompatible constructions and primitive/nominal
runtime dispatch. Constraints, variance and cross-assembly relationship declarations
are not added by this slice. Host APIs remain documented manually here; the guest
RavenDoc assembly is unchanged.


## Metadata-only Introspection facade (development, 2026-10-02)

Namespace `NeoCLR.Metadata.Experimental.Introspection`, in the existing .NET metadata
library. These are C# prototype types shaped after the runtime System.Introspection
model, not runtime Reflection types or a complete replacement for that model.

| Type | Implemented public surface |
| --- | --- |
| MetadataLoadContext | Constructor `(IEnumerable<AssemblyDefinition> snapshots)`; `IReadOnlyList<AssemblyInfo> Assemblies`; `Resolve(AssemblyIdentity)` and `Resolve(AssemblyReference)` returning AssemblyInfo; `Resolve(TypeReference)` returning NominalTypeInfo |
| AssemblyInfo | `string Name`, `AssemblyIdentity Identity`, `IReadOnlyList<AssemblyInfo> ReferencedAssemblies`, `GetModules(): IReadOnlyList<ModuleInfo>`, `GetTypes(): IReadOnlyList<NominalTypeInfo>` |
| ModuleInfo | `string Name`, `AssemblyInfo Assembly`, `GetTypes(): IReadOnlyList<NominalTypeInfo>` |
| TypeInfo | Abstract read-only `string DisplayName`, `bool IsNominalType`; library-controlled construction |
| NominalTypeInfo | TypeInfo plus `string Name`, `string Namespace`, `string FullName`, `uint MetadataToken`, `ModuleInfo Module`, nullable `NominalTypeInfo DeclaringType`, `int GenericArity`, `bool IsInterface`, `bool IsValueType` |

The constructor registers at most 4096 already-read CLI/native snapshots in input order.
It does not read files or expand dependencies. Null catalog/elements throw
ArgumentNullException; mutable builder definitions or excessive input throw
ArgumentException. Repeating the same snapshot object is idempotent; different objects
with one exact identity throw InvalidDataException even when their bytes might match.
No implicit image hashing, version fallback, assembly loading or code execution occurs.

Resolve(identity) requires the full identity, including version/culture/token/flags.
Resolve(reference) also requires its consuming snapshot to be the registered object.
Missing/conflicting scopes, missing or ambiguous types and unsupported reader scopes
throw InvalidDataException. Null inputs throw ArgumentNullException. Exported forwarders,
multimodule scopes and constructed TypeSpec resolution retain reader limitations.
ReferencedAssemblies lazily resolves direct edges and throws when a dependency is
unregistered; legal assembly cycles reuse views rather than recursively loading graphs.

Collections are read-only and views canonical within a context, including concurrent
nominal lookups. Reference equality is context identity, not equality across contexts.
GetTypes includes nested definitions and excludes the CLI `<Module>` pseudo-type.
Names are display/declaration strings; Module.Assembly.Identity supplies binding scope.
GenericArity describes a definition, not constructed arguments. MetadataToken is the
reader's module-local token (native origin tokens included), not an execution handle.

No assembly/module token or FullName is fabricated for native inputs without that
contract. The exact AssemblyIdentity is exposed instead. Additional runtime TypeInfo
properties, member enumeration, constructed/array/function/parameter views and binding
flags remain unimplemented. This is an explicit subset; unsupported capabilities are
not represented as empty member collections. The context retains snapshots through its
views and owns no disposable runtime loader; ordinary managed lifetime applies.

```csharp
var context = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext(
    new[] { consumerSnapshot, dependencySnapshot });
var consumer = context.Resolve(consumerSnapshot.Identity);
var externalType = context.Resolve(consumerSnapshot.MainModule.TypeReferences[0]);
var declaringAssembly = externalType.Module.Assembly;
```

MetadataLoadContextChecks covers CLI/native navigation, exact versions, conflicts,
missing dependencies, diamonds, legal cycles, foreign snapshot rejection, concurrency,
context isolation and collection immutability. Raven now uses this context for nominal
resolution; its emission remains based on symbols. This host-only namespace is manually
documented here and is not included in the guest RavenDoc reference assembly.


### Constructed and field views (development, 2026-10-02)

In `NeoCLR.Metadata.Experimental.Introspection`:

| Type | Public members added |
| --- | --- |
| MetadataLoadContext | `ResolveSignature(SignatureType signature, IReadOnlyList<TypeInfo>? typeArguments = null): TypeInfo` |
| NominalTypeInfo | `GetGenericArguments(): IReadOnlyList<TypeInfo>`, `MakeGenericType(params TypeInfo[] arguments): ConstructedTypeInfo`, `GetFields(): IReadOnlyList<FieldInfo>` |
| PrimitiveTypeInfo | `PrimitiveType Kind`, DisplayName, IsNominalType=false |
| ArrayTypeInfo | `TypeInfo ElementType`, DisplayName, IsNominalType=false |
| GenericParameterTypeInfo | `NominalTypeInfo DeclaringType`, `int Position`, `string Name`, DisplayName, IsNominalType=false |
| ConstructedTypeInfo | `NominalTypeInfo Definition`, `IReadOnlyList<TypeInfo> TypeArguments`, `GetFields(): IReadOnlyList<FieldInfo>`, DisplayName, IsNominalType=true |
| FieldInfo | `string Name`, `uint MetadataToken`, `TypeInfo DeclaringType`, `TypeInfo FieldType`, `bool IsStatic`, `bool IsReadOnly` |

Views are library-created and interned by context/type structure. Field collections are
stable and read-only per owner view. GetFields returns all declared fields in metadata
order, with no inherited lookup or visibility filtering. The token always identifies
the original declaration in Definition.Module. Primitive views describe signature
categories, not invented core-library declarations. Open parameter identity includes
the declaring nominal definition; DisplayName (`!0`, etc.) is not identity.

MakeGenericType copies exactly one same-context argument per parameter. Null arguments
throw ArgumentNullException; wrong arity, foreign/null/Void/bare-generic arguments,
nongeneric owners and excessive nesting throw ArgumentException. Scoped caller parameters
are allowed. This projects metadata, not language-level generic constraint satisfaction.

ResolveSignature accepts reader primitives, nominal references, constructions, vectors
and owner parameters. Up to 32 owner arguments are copied before projection; null
signature throws ArgumentNullException, foreign/null/Void/excessive arguments throw
ArgumentException. Missing dependencies, unsupported signatures or absent parameter
scope throw InvalidDataException. Output-builder types and method-scoped parameters are
not supported. Nesting is bounded to 16. Substitution is simultaneous: supplying another
owner's !0 returns that parameter unchanged rather than substituting it again.

GetFields decodes through the current reader profile and throws InvalidDataException
for unavailable signatures/dependencies. Broader CLI signatures retain reader limitations.
For a recursive `Box<T>`.Next: `Box<T>`, projecting `Box<Int32>` returns the existing constructed
view without expanding its fields recursively. No runtime objects are inspected or mutated.

The names follow the runtime model; this prototype distinguishes nominal definitions
from constructed views explicitly. MethodInfo/ParameterInfo, properties, full access
flags, function/byref views and open method generic scopes remain follow-up work.
The new host types are covered here, not added to the guest RavenDoc assembly.


### Method and parameter views (development, 2026-10-02)

Namespace `NeoCLR.Metadata.Experimental.Introspection`:

| Type | Public surface added |
| --- | --- |
| MetadataLoadContext | `Resolve(MethodDefinition): MethodInfo`; `ResolveSignature(SignatureType, IReadOnlyList<TypeInfo>? typeArguments, IReadOnlyList<TypeInfo>? methodArguments): TypeInfo` |
| ModuleInfo | `GetFunctions(): IReadOnlyList<MethodInfo>` |
| NominalTypeInfo / ConstructedTypeInfo | `GetMethods(): IReadOnlyList<MethodInfo>` |
| MethodInfo | `Name`, `Namespace` (strings), `MetadataToken` (uint), `Module` (ModuleInfo), nullable `DeclaringType` (TypeInfo), `ReturnType` (TypeInfo), `GenericParameterNames` (`IReadOnlyList<string>`), `IsStatic`, `IsAbstract`, `IsVirtual` (bool); `GetParameters(): IReadOnlyList<ParameterInfo>`; `GetGenericArguments(): IReadOnlyList<TypeInfo>` |
| ParameterInfo | `DeclaringMethod` (MethodInfo), `Position` (int), `ParameterType` (TypeInfo) |
| MethodGenericParameterTypeInfo | `DeclaringMethod` (MethodInfo), `Position` (int), DisplayName (`!!ordinal`), IsNominalType=false |

Resolve requires a method definition from an exact registered snapshot. Null input throws
ArgumentNullException; missing snapshots/undecodable signatures throw InvalidDataException.
Methods are interned by original definition and declaring owner view. GetMethods returns
all declared non-constructor methods including accessors, without inherited lookup or
visibility filtering. Module.GetFunctions enumerates namespace functions. Enumeration
collections are read-only; repeated calls preserve element identities. Constructor
metadata can be resolved directly as MethodInfo; a dedicated ConstructorInfo facade is
not added. Names/tokens describe declarations, never runtime handles.

Returns/parameters resolve lazily, throwing InvalidDataException for unavailable
signature/dependency profiles. GetParameters preserves positions and canonical projected
types. Parameter names/defaults/attributes are not fabricated when the current reader
has not provided them. GenericParameterNames copies declared names. Method arguments
have declaring-method identity distinct from owner parameters even at the same ordinal.
Constructed owners substitute only their own arguments; method parameters remain open.

The three-argument ResolveSignature accepts explicit copied type and method scopes,
each at most 32 entries. Null signature throws ArgumentNullException; null/foreign/Void
arguments or excessive scope length throw ArgumentException. Missing parameter slots,
dependencies or unsupported signature shapes throw InvalidDataException. Supplied
arguments are substituted simultaneously without recursively rebinding caller scopes.
The existing overload remains; a method parameter with no supplied method scope rejects.

Raven consumes method views for signature projection. Constraint/overload/access policy
and compiler symbols remain Raven responsibilities. Method instantiation as a dedicated
view, full parameter metadata, custom attributes, property views and broad CLI decoding
remain pending. There is no Invoke API. Current MethodInfo/ParameterInfo names follow
System.Introspection; this host prototype does not change guest APIs or commit a future
identical implementation. All public host members are documented here outside RavenDoc.


## Property and direct interface views (development 2026-10-03)

Namespace: `NeoCLR.Metadata.Experimental.Introspection`. Host C# development APIs;
not guest APIs or runtime Reflection. Existing context/snapshot lifetime rules apply.

Both `NominalTypeInfo` and `ConstructedTypeInfo` expose:

| Member | Contract |
| --- | --- |
| `IReadOnlyList<PropertyInfo> GetProperties()` | Stable read-only list of declared properties in metadata order, including non-public/static/indexed properties. No inherited lookup or visibility filter. |
| `IReadOnlyList<TypeInfo> GetDeclaredInterfaces()` | Stable read-only direct interface edges in metadata order; arguments substitute in this owner's scope. Does not include transitive ancestors. |

`PropertyInfo` is obtained from a type view; it has no public constructor:

| Member | Contract |
| --- | --- |
| `string Name` | Metadata declaration name. |
| `uint MetadataToken` | Original property token, local to the declaring module. |
| `TypeInfo DeclaringType` | Open or constructed owner through which it was selected. |
| `TypeInfo PropertyType` | Resolved property result/storage type after owner substitution. |
| `IReadOnlyList<TypeInfo> IndexParameterTypes` | Read-only ordered index types; empty for non-indexed properties. Setter value is excluded, including setter-only properties. No parameter names/defaults/attributes are invented. |
| `MethodInfo? GetMethod` | Canonical getter on the same owner, including non-public accessors; null if absent. |
| `MethodInfo? SetMethod` | Canonical setter on the same owner, including non-public accessors; null if absent. |
| `bool IsStatic` | Staticness recorded by the metadata accessors. |

```csharp
var box = context.Resolve(identity).GetTypes().Single(t => t.Name == "Box`1");
var closed = box.MakeGenericType(context.ResolveSignature(PrimitiveType.Int32));
var current = closed.GetProperties().Single(p => p.Name == "Current");
// current.PropertyType is the canonical Int32 view; its getter is also in closed.GetMethods().
var directContracts = closed.GetDeclaredInterfaces();
```

The C# contract fixture validates this inspection pattern. Open definitions preserve
their scoped parameters; caller-provided parameters retain caller identity. Constructed
relationships can be inspected edge by edge to compose substitutions, without eagerly
expanding a graph. Properties and lists are cached per owner; accessor/type identity is
canonical in the context. There is no invocation, property value access or runtime loading.

`GetProperties()` throws `InvalidDataException` for unsupported signatures or missing
catalog dependencies. Signature scope/construction bounds from `ResolveSignature` still
apply. `GetDeclaredInterfaces()` throws `InvalidDataException` for invalid relationships
or missing dependencies and `NotSupportedException` when reader relationship materialization
is unavailable (currently CLI snapshots). The native reader currently supports
local and external interface declarations; exact dependencies are resolved through the context catalog.

Unlike .NET Reflection property access, this reports metadata only. Index types are
exposed directly instead of manufacturing method-owned ParameterInfo objects for a
property. Other-method semantics, full parameter metadata, transitive interface queries,
constructor-specific views and generic method construction are not added here.


## Inherited interface views (development 2026-10-03)

`NominalTypeInfo.GetInterfaces()` and `ConstructedTypeInfo.GetInterfaces()` return
`IReadOnlyList<TypeInfo>` containing direct and inherited interfaces, excluding the owner.
Owner arguments are substituted at each edge; diamond duplicates collapse by canonical
constructed identity, so different arguments remain distinct. Results are stable,
read-only and in depth-first metadata order. GetDeclaredInterfaces remains direct-only.

Invalid relationships, missing dependencies, cyclic declaration paths, more than 4,096
distinct interface views or 65,536 visited edges throw InvalidDataException. Unsupported
reader relationship materialization (currently CLI snapshots) throws NotSupportedException.
There is no partial success result, recursive member expansion or runtime loading.
This covers the native root-class/interface profile; general class inheritance and
parameter-constraint queries are not claimed. The result is cached per owner.


## Generic method construction (development 2026-10-03)

Host `Introspection.MethodInfo` adds these members:

| Member | Contract |
| --- | --- |
| `bool IsGenericMethodDefinition` | True for a generic declaration without supplied method arguments, even on a constructed owner. |
| `MethodInfo GetGenericMethodDefinition()` | Returns this definition or the construction's definition on the same declaring owner. Throws InvalidOperationException for nongeneric methods. |
| `MethodInfo MakeGenericMethod(params TypeInfo[] arguments)` | Copies same-context arguments and returns a canonical constructed view. Only callable on a generic definition. |

`GetGenericArguments()` returns scoped parameters on a definition and copied supplied
arguments on a construction. GenericParameterNames remains declaration metadata.
ReturnType and GetParameters apply owner and method arguments simultaneously; each
ParameterInfo.DeclaringMethod is the constructed method that exposes it. DeclaringType,
Module and MetadataToken retain original provenance. No declaration mutation occurs.

```csharp
var mixed = closed.GetMethods().Single(m => m.Name == "Mixed");
var call = mixed.MakeGenericMethod(context.ResolveSignature(PrimitiveType.Boolean));
// For Box<Int32>.Mixed<U>(U, T) -> T: parameters are Boolean/Int32, result Int32.
```

This pattern is tested by the C# fixture against equivalent CLR metadata inspection.
Namespace functions support construction too. Supplied caller type/method parameters
remain caller-scoped and are not recursively rebound. Repeated equal requests return
the same view within one context; differing owners or contexts remain distinct.

MakeGenericMethod throws ArgumentNullException for a null argument array;
InvalidOperationException for a nongeneric or already constructed method; ArgumentException
for wrong arity, null elements, foreign-context types, Void, bare generic definitions,
or argument nesting of 16 or more. Result signature projection may later throw
InvalidDataException for unsupported/unavailable metadata or nesting bounds, as on open
methods. Collections remain read-only. This supports the reader's unconstrained generic
signature profile, not general constraint validation, runtime invocation or code emission.


## Declaration facts (development 2026-10-03)

Host namespace: `NeoCLR.Metadata.Experimental.Introspection`.

| API | Contract |
| --- | --- |
| `MetadataAccessibility` | CLI-shaped values: CompilerControlled (PrivateScope), Private, FamilyAndAssembly, Assembly, Family, FamilyOrAssembly, Public. Consumers apply language access rules. |
| `NominalTypeInfo.Accessibility` | Declared type access, including distinct nested visibility categories on supported CLI definitions. |
| `NominalTypeInfo.IsAbstract`, `IsSealed`, `IsStatic` | Metadata flags; static denotes abstract plus sealed. |
| `MethodInfo.Accessibility`, `FieldInfo.Accessibility` | Declared member access; not filtered by caller. |
| `MethodInfo.IsConstructor` | Instance `.ctor` classification. |
| `MethodInfo.IsStaticConstructor` | Static `.cctor` classification. |
| `NominalTypeInfo.GetConstructors()`, `ConstructedTypeInfo.GetConstructors()` | Read-only `IReadOnlyList<MethodInfo>` of declared constructors/type initializers in metadata order. Same canonical method elements as direct resolution; constructed owners substitute signatures. |

GetMethods still excludes constructors. Unsupported callable signatures/dependencies throw
InvalidDataException, including CLI instance signatures outside the current decoder.
Enumeration does not execute initializers or load runtime types. The C# fixture checks
public constructors/private fields, internal declarations, readonly/static facts and
closed-owner constructor parameters. Raven supports public/internal/private native access;
other categories fail explicitly rather than being treated as private.


## External interface declarations (development, 2026-10-03)

These additions extend the earlier authored relationship profile to external definitions;
existing local overloads and encodings remain unchanged.

- `AssemblyBuilder.CompleteInterfaceReference(ImportedTypeReference reference) -> void`
  asserts that all direct `CreateMethodReference` contracts (including accessor methods)
  and `AddInterfaceConversion` base edges have been supplied. The reference must be an
  output-owned open interface definition. Null throws `ArgumentNullException`; foreign,
  constructed or noninterface references throw `ArgumentException`. Completion is
  idempotent and permits explicitly empty interfaces. Adding a new method or base edge
  afterwards throws `InvalidOperationException`; reusing existing methods/edges remains valid.
  This is a caller assertion; it does not inspect the dependency or replace runtime linking.
- `TypeBuilder.AddBaseInterface(ImportedTypeReference contract) -> void` declares an
  external base on an interface. `AddInterfaceImplementation(ImportedTypeReference contract)`
  declares an external implementation on a root class. Arguments can use owner generic
  parameters. Null throws `ArgumentNullException`; foreign/unregistered targets, invalid
  scope, duplicate edges or limits throw `ArgumentException`; invalid owners throw
  `InvalidOperationException`. Each external interface and inherited base must be completed
  before writing. Incomplete contracts and missing/mismatched implementations throw
  `InvalidDataException`. Only matching public instance methods acquire CLI implementation flags.
- Direct definitions use `new InterfaceImplementation(module.ImportReference(identity,
  namespace, metadataName), arguments)` appended to `TypeDefinition.Interfaces`, after
  registering the matching output-owned interface contract. It shares builder validation.
- `RuntimeAssemblyContainer.WriteBinary(AssemblyBuilder assembly) -> byte[]` validates and
  encodes the authored graph with its CLI reference projection. Null throws
  `ArgumentNullException`; invalid contracts/encoding throw `InvalidDataException`.
  No reader/importer is consulted. Native payloads remain authoritative and CLI bodies
  remain non-executable. Existing native-bytes container overloads cannot reconstruct
  external interface method contracts and throw `NotSupportedException` for these
  declarations; use the graph overload. This restriction also applies to
  `NativeAssemblyDefinition.CreateReferenceAssembly` without resolved external contracts.

Reader snapshots preserve exact scoped interface identities and constructed arguments;
`GetDeclaredInterfaces`/`GetInterfaces` resolve them through the explicit metadata context.
Missing or wrongly classified dependencies reject during resolution. Generic diamonds
retain canonical views and deduplicate the common construction.

```csharp
var value = output.CreateInterfaceReference(dependency, core, digest, "Example", "Value`1", 1);
output.CreateMethodReference(value, "Get", new(SignatureType.TypeParameter(0), []));
output.CompleteInterfaceReference(value);
var box = output.AddClass("Example", "Box");
box.AddInterfaceImplementation(value.MakeGenericInstance(PrimitiveType.Int32));
var get = box.AddInstanceMethod("Get", new(PrimitiveType.Int32, []));
get.GetILGenerator().LoadConstant(42);
get.GetILGenerator().Return();
var image = RuntimeAssemblyContainer.WriteBinary(output);
```

`ExternalInterfaceChecks` validates definition/builder parity, CLI flags and execution,
PE/native round trips, external generic diamond resolution and native linked dispatch.
No format version change is required. The C# development API remains documented here
rather than through guest RavenDoc.


## Native Self signatures (development, 2026-10-03)

`Model.SignatureType.Self` is an immutable singleton signature node; `bool IsSelf`
identifies it. It is neither a primitive nor a positional type/method parameter.
`ToString()` returns `Self`. It consumes no generic arity. Attaching it to a bodyless
instance interface method (directly or through `AddInterfaceMethod`) or its associated
property supplies its scope. Vectors and unconstrained nominal constructions can contain
Self. Class signatures must use the actual nominal declaring type instead.

Native writing preserves the existing `SelfType` encoding; the binary payload and format
versions do not change. Native readers reject unresolved Self in fields, locals, free
functions and class method signatures with `InvalidDataException`. Builder attachment
rejects those declarations with `ArgumentException`. Standalone constructions may retain
Self, like an open parameter, but their eventual use must provide the interface scope.

`Introspection.SelfTypeInfo : TypeInfo` exposes:

- `TypeInfo DeclaringType`: canonical open or constructed interface view supplying scope.
- `string DisplayName`: the interface display label followed by `.Self`, not an identity.
- `bool IsNominalType`: always false.

Method results/parameters and property types project to the same context-canonical Self
view for the same owner. Constructing `I<T>` substitutes T while retaining a distinct Self
scope for `I<int>`. No generic ordinal or runtime handle is exposed. Calling public
`MetadataLoadContext.ResolveSignature(SignatureType.Self)` without a member scope rejects
with `InvalidDataException`; use the declared member views. Contexts do not share identity.

Executable CLI `AssemblyBuilder.Write()` rejects Self with `InvalidDataException`.
The reference-only CLI projection in PE/#Neo uses the existing value-type marker
`System.Runtime.CompilerServices.Self`, scoped to the explicitly supplied core identity.
It is only a transport representation; it has no CLR implementing-type semantics.
Native materialization reads the authoritative payload, never the marker. Both container
variants and immutable snapshot writing preserve the native signature.

This slice supports contract declarations and inspection. Implementation substitution,
typed `callself` authoring, static Self interface methods, Self-containing inheritance,
Raven native symbol projection/emission and general byref/function introspection remain
subsequent work. Existing runtime Self execution does not imply these host APIs are ready.
Tests: `SelfSignatureChecks` (C#), with a generated PE/#Neo loaded and verified by NeoCLR;
its entry point returns 42 independently of Self dispatch.


## Parameter passing modes (development, 2026-10-03)

`Introspection.ParameterInfo.PassingMode : ParameterPassingMode` preserves the declared
calling convention. `ParameterPassingMode.Value` means a copied value, `Ref` means a
writable reference initialized by the caller, and `Out` means a writable reference assigned
by the callee before normal return. `ParameterType` exposes the value/element type, not a
byref wrapper. Position and canonical DeclaringMethod remain unchanged. There are no
runtime objects, argument addresses or invocation operations in these views.

Native method materialization retains existing `ByRef` nodes and `out_parameters` indices
in MethodSignature. Parameter views substitute owner/method arguments within the referenced
element, including vectors. Copying a snapshot preserves the same modes. Neither native
format 5 nor the binary container schema changes. Unsupported readonly `In` metadata is
rejected by signature recognition and introspection instead of fabricated as writable ref;
byref constructors, byref returns and general readonly signatures remain unsupported.
Existing supported CLI static signatures expose the same ref/out distinction.

`AssemblyBuilder.CreateMethodReference` accepts writable ref/out parameter signatures on
supported class/interface methods. Its MethodSignature carries `ByReference(element)` and
zero-based `outParameters`; conflicting ref/out contracts reject with InvalidDataException.
Constructed external interface traversal substitutes elements and retains output indices.
Constructor byrefs still reject. Namespace-function symbol-authored references retain their
previous bounded signature profile. No dependency definitions are reopened to discover modes.

Validation: C# `ParameterModeChecks` covers manual/builder declarations, both native
containers, open/constructed scopes, CLI mode parity, readonly rejection and conflicting
contracts. The Raven three-assembly driver case executes inherited generic ref/out dispatch
on both targets with library sources removed; incompatible implementations publish nothing.

## Union payload foundation (development, 2026-10-03)

The metadata producer now accepts direct nominal and constructed fields in owned
value types, including a `Payload<T>` embedded in a `Carrier<T>`. Builder calls and
manually attached field definitions share validation. Writing rejects recursive inline
storage and limits owned layout traversal to depth 64 and 4096 visited constructions;
references and vectors terminate inline traversal. Native input validates the same
owned layouts. External dependency layouts still require explicit dependency resolution
and runtime linking; this check does not load dependencies implicitly.

`AssemblyDefinition.ReadNativeAssembly` now materializes unconstrained top-level
value declarations, signatures, fields and supported constructors/methods. `IsValueType`
reflects the native category; sealed/sequential flags are preserved without inventing
a CLI `System.ValueType` dependency. ImportReference overloads retain the explicit
matching output core requirement. Nested declarations, constrained owners and generic
instance methods remain outside this native snapshot profile. Native snapshots remain
immutable and preserve their original bytes on Write.

This matches CLR inline value storage and copy semantics: an executable tag/payload
fixture returns 42 on both runtimes after mutating an independent copy. Direct native
imports of nongeneric/generic value constructors and methods also execute on both.
The fixture uses an Int32 tag and is a metadata contract test, not a replacement for
Raven union lowering or proof that source Option compiles.

Raven maps the introspection value category to Struct and its semantic ValueType base;
generic field substitution remains in introspection. It does not reopen imported
metadata in emission. Source value declarations, nested union cases, the Byte tag,
synthesized members and symbol-authored external value operands remain the next
compiler work. Runtime-library union sources are unchanged. No format version change,
CLI projection fallback, runtime implementation change or performance claim is needed.

## Native nested case metadata (development, 2026-10-03)

`AssemblyDefinition.ReadNativeAssembly` now retains supported nested class/value
ownership beneath nongeneric declaring types, including generic nested value cases.
Local signature lookup keys include the declaring token; external TypeRef rows use
nested TypeRef scopes. Same-named cases beneath different companions remain distinct.
Nested public/internal accessibility, canonical declaring views and constructor/member
signatures survive direct native reading without a CLI projection. ImportReference
accepts these scoped native definitions with the existing explicit matching core policy.

Raven publishes nested symbols as members of their declaring type, preserving their
namespace and containing-type identities; namespace member and simple-name lookup do
not flatten them. Generic nested field substitution stays in introspection. Emission
continues to reject native nested operands outside its symbol-authored capability profile.

This follows CLI NestedClass/TypeRef identity semantics using existing native relationship
encoding; no schema change or implicit dependency loading is required. C# checks cover
same-named local/external payloads, multiple nesting levels, internal visibility, missing
dependencies and cyclic owners. Direct native nested generic/nongeneric constructor
imports execute on CLR and NeoCLR (42). Native semantic probes retain owner/field identity.

This is a reader/importer prerequisite for union cases, not source union completion.
Nested types that capture generic enclosing parameters remain unsupported. Source union
declaration collection, nested definition emission, Byte discriminators and complete
synthesized union contracts remain pending. Existing immutable snapshot behavior is unchanged.

## Byte signatures and IL generation (development, 2026-10-03)

`PrimitiveType.Byte` denotes unsigned 8-bit storage, encoded as CLI `ELEMENT_TYPE_U1`
and native `Byte`. It is available in primitive method signatures, fields, locals,
array elements and imported signatures. Readers and introspection retain the Byte
identity. As in CLI, loading Byte produces an Int32 evaluation-stack value; storing
into Byte truncates to eight bits, and loading zero-extends. `ref Byte` remains distinct
from `ref Int32`; stack normalization does not erase storage or managed-pointer identity.

`method.GetILGenerator().Emit(OpCode.Conv_U1)` consumes an Int32 or Int64 and produces
an Int32 containing the low unsigned eight bits. It encodes CLI/native `conv.u1` and
performs unchecked truncation, including -1 → 255 and 298 → 42. Other input categories
reject during body validation; floating-point conversion is outside this writer profile.
The legacy `MethodBuilder.Emit` forwarding API accepts the same opcode, but new code
should use `IILGenerator`. No new native format version or runtime operation is required.

This matches CLI small-integer storage/evaluation behavior; the explicit Byte signature
costs another supported primitive category but avoids widening union tags in metadata.
C# coverage checks CLI/native round trips, field/method imports, local/array/field storage,
conversion boundaries, CLR execution and incompatible by-reference rejection. The native
artifact executes using the runtime's existing Byte storage and conversion implementation.

## Value interfaces and constrained calls (development, 2026-10-03)

`TypeBuilder.AddInterfaceImplementation(TypeBuilder|GenericTypeInstance|ImportedTypeReference)`
and `TypeDefinition.Interfaces.Add` now admit value-type owners as well as root classes.
Owned, constructed and registered external relationships retain their existing ownership,
completeness, signature and duplicate validation. The native reader retains these edges;
CLI emission writes ordinary InterfaceImpl rows and implementation method flags.
Declaring a relationship does not introduce implicit boxing in the body API.

New `IILGenerator` members:

```csharp
void CallConstrained(TypeBuilder receiverType, MethodBuilder target);
void Emit(OpCode opCode, TypeBuilder receiverType, MethodBuilder target);
```

The call profile requires an owned nongeneric implementing type and an owned nongeneric
interface contract. Instance calls require a value-type implementation; static calls
admit class or value-type implementations and consume no receiver. The raw overload
accepts `OpCode.Callvirt` for instance contracts and `OpCode.Call` for static contracts.
Null operands throw ArgumentNullException; foreign, generic, nonconforming, nonvalue
instance receivers or invalid targets/opcodes throw ArgumentException
before appending. The existing instruction bound applies. Writing validates complete
implementations, parameters and the exact managed receiver address; invalid bodies throw
InvalidDataException. Arguments follow the receiver on the stack.

```csharp
var il = method.GetILGenerator();
il.LoadLocalAddress(counter);
il.CallConstrained(counterType, nextInterfaceMethod);
// Equivalent atomic typed emission:
// il.Emit(OpCode.Callvirt, counterType, nextInterfaceMethod);
```

For instance contracts, CLI encoding is `constrained.` plus `callvirt`; native encoding
uses the existing borrowed `callself` instruction. Both dispatch using the addressed storage,
so mutations affect that instance and leave prior copies independent. This implements
the value-implementation branch of [.NET constrained dispatch](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.constrained?view=net-10.0).
Reference receivers, external/constructed call targets, open generic receivers, and boxing
fallbacks remain outside this initial authoring profile. The bounded API costs a separate
validated operand form but prevents an incomplete prefix from being left in a body.
No native format version or runtime code change is required. This operation is exposed
on the metadata IL generator, not added to method builders or Raven's shared IL interface.


## Value override authoring (development, 2026-10-03)

```csharp
MethodBuilder TypeBuilder.AddOverride(string name, MethodSignature signature);
```

The initial profile accepts public instance `ToString() -> String` on a value type,
including a generic value owner. The method itself cannot be generic. The result is
an attached definition with a managed receiver; use `GetILGenerator()` for its body.
Null signatures throw ArgumentNullException. Unsupported names/signatures, duplicate
methods and method limits throw ArgumentException. Class, interface and static owners
throw InvalidOperationException before attachment.

Manual authoring uses the same path: construct `MethodDefinition` with CLI
Public | Virtual (0x46), append it to `TypeDefinition.Methods`, then obtain its builder
with `MethodBuilder.ForDefinition`. HideBySig is added by the writer. Do not specify
Abstract or NewSlot. Interface implementation inference preserves this override rather
than changing it into a new virtual slot. Existing nonvirtual AddInstanceMethod behavior
is unchanged.

```csharp
var display = valueType.AddOverride("ToString", new(PrimitiveType.String, []));
var il = display.GetILGenerator();
il.Emit(OpCode.Ldstr, "Example value");
il.Return();
var cliImage = assembly.Write();
```

This follows [.NET CLI Virtual/ReuseSlot semantics](https://source.dot.net/system.private.corelib/src/runtime/src/libraries/System.Private.CoreLib/src/System/Reflection/MethodAttributes.cs.html).
C# execution verifies boxed Object.ToString dispatch, generic value construction,
interface dispatch to the same implementation, definition/builder parity and PE flags.
The restricted profile avoids inventing general inheritance resolution; its cost is
that other Object slots and class overrides remain unsupported.

**Native binding:** before native writing, register exactly one explicit System library
with `BindNativeLibrary(referenceSnapshot, nativeLibrary, coreIdentity)`. The existing
retained CLI bootstrap snapshot must declare public virtual Object.ToString() -> String;
the native library must be module System and provide the matching public virtual instance
slot, with no generic parameters, byref receiver, output modes or no-result semantics.
Missing, ambiguous or incompatible bindings throw InvalidDataException. No library is
loaded implicitly. This remains an explicit bootstrap bridge, not an application-reference
fallback or a general Object/Reflection API.

Native writing includes the registered assembly identity and native module/revision binding
even without a call to the dependency in the body. Override names use the runtime's
owner.ToString slot convention, with is_virtual/is_override and receiver_byref. Ordinary
method naming and interface encodings are unchanged. These are existing format-5/runtime
fields; no format bump or runtime change is required.

Native readers retain the CLI Virtual/ReuseSlot flags, validate the bounded shape and System
binding, and native imports preserve the call name. Both native semantic snapshots and
reference-only CLI projections retain the declaration meaning. Execution additionally
checks the linked dependency's actual definitions. Reader validation is not proof that
runtime bodies or external artifacts have been verified.

The C# `--native-value-override` integration mode writes a native library, rereads it,
emits a separate direct-call consumer and executes it. A neoIL harness calls an API-produced generic boxing method with ordinary
and generic values, then calls Object.ToString; it also checks reference identity and
primitive display, returning 42 using the real retained System bundle. This does not
claim Raven union execution.
Raven now consumes the bounded override with explicit runtime-seed binding. Generated
display conversions/formatting operations and union/case metadata preservation remain pending.


### Typed boxing (development, 2026-10-03)

`AssemblyBuilder.CoreObjectType : ImportedTypeReference` returns an interned,
output-owned System.Object reference in the explicitly supplied CoreLibrary. It performs
no loading or dependency discovery. A signed core identity is supported for this exact
reference; other imported signed nominal identities remain outside the bounded profile.

`IILGenerator.Box(SignatureType type)` and `Emit(OpCode.Box, SignatureType type)`
consume the exact storage signature and push CoreObjectType. Types may include primitives,
owned/imported nominal types, vectors and in-scope owner/method generic parameters.
Void, managed references, Self, Function, foreign owners and out-of-scope parameters reject.
Operand errors throw ArgumentException (null throws ArgumentNullException); stack errors
and missing native bindings throw InvalidDataException before an image is returned.
The new operation belongs to the IL generator, not the method builder.

CLI encoding is standard `box` (0x8c) with a TypeDef/TypeRef/TypeSpec operand. Native
encoding uses the existing `box` instruction and nominal System.Object result. Value
storage is copied; reference instantiations preserve object identity, including null on
CLR. Native String uses the runtime's object representation. Native writing requires a
BindNativeLibrary mapping for the exact core identity, module System, and a matching public
System.Object class. No runtime format change, automatic dependency load or application
reference projection is introduced. C# tests execute method/owner generic scopes on CLR;
the native integration harness exercises generic value boxing, primitive display and
reference identity against the retained System seed.

```csharp
var t = SignatureType.MethodParameter(0);
var method = assembly.AddFunction("Box", new(assembly.CoreObjectType, [t], ["T"]));
var il = method.GetILGenerator();
il.LoadArgument(0);
il.Box(t);
il.Return();
```


### Owned field addresses (development, 2026-10-03)

`IILGenerator.LoadFieldAddress(FieldBuilder field)` and
`LoadFieldAddress(ConstructedFieldReference field)` consume an initialized receiver and
push a managed reference to its actual field storage. The equivalent raw operations are
`Emit(OpCode.Ldflda, field)` on these two operand categories. Existing forwarding Emit
methods on MethodBuilder retain parity; no new builder-body convenience API is added.

The field must belong to this output, be an instance field and be mutable. Constructed
references substitute the exact owner arguments; open definition operands require the
matching owner scope. A reference-class receiver is consumed directly. A value receiver
must already be addressed; a temporary value on the stack is rejected instead of silently
mutating a copy. Uninitialized locals/out parameters/constructor fields reject on write.
Readonly and imported field addresses remain explicitly unsupported in this profile.

Null arguments throw ArgumentNullException; foreign/readonly/out-of-scope operands throw
ArgumentException without appending instructions. Instruction limits and incorrect stack
or initialization contracts throw InvalidDataException before producing output. Reads,
stores and calls through the resulting managed reference reuse their existing exact-type
checks. CLI uses standard ldflda (0x7c) with a FieldDef/MemberRef; native uses the existing
ldflda layout ordinal. No format or runtime behavior change is required.

C# tests execute nested field mutation through a constructed generic holder and verify
that its object alias sees 42 on CLR and NeoCLR. They also reject readonly and foreign
operands, temporary value receivers and uninitialized local receivers.


### Reference tests (development, 2026-10-03)

`IILGenerator.IsNull()` / `Emit(OpCode.ReferenceIsNull)` consume an ordinary nominal,
interface, vector or String reference and push Boolean. They perform a null identity test,
not a user-defined equality operation. CLI uses ldnull/ceq and accounts for its temporary
stack slot; native uses ref.isnull. Managed references and unboxed values reject on write.

`IsInstance(SignatureType target)` / `Emit(OpCode.Isinst, target)` implement standard
isinst: a matching reference or null, with no unboxing or exception on type mismatch.
Reference targets retain their signature (including String). Value and scoped generic
targets produce CoreObjectType, representing a box or null. Void, Self, Function, managed
references, foreign owners and out-of-scope parameters reject before instruction append.
A native value/generic type test requires the same explicit System core binding as boxing.
No target or dependency is inferred from the executing host.

`CastReference` / `Emit(Castclass, target)` additionally accept String targets and String
inputs. Failed checked casts retain runtime InvalidCast behavior; null remains null.
ArgumentNullException/ArgumentException describe null or invalid operands; stack, limit
and native-binding failures throw InvalidDataException before an image is returned.
These operations add no metadata format or runtime behavior. C# tests exercise null and
matching/nonmatching primitive/reference type tests, generic scopes and String identity
on CLR; native binary tests verify/run 42 with the explicit retained-System bootstrap.

Core display dispatch continuation (2026-10-03): imported public instance
System.Object.ToString() -> String from the exact explicit CLI core snapshot now
supports CallVirtual. Native output requires the validated System slot binding;
missing, ambiguous, nongeneric/signature-mismatched or nonvirtual slots reject.
This reuses CLI callvirt and native virtual dispatch without a format/runtime change.
Raven opts into a bounded semantic Object display capability; other virtual class
calls remain unsupported. Runtime Contract and importer/emitter boundaries are unchanged.
API-authored boxed value overrides execute through Object; ordinary Raven commands
print `42` and `text` on both targets. The union preflight now reaches a synthesized
get_Value null literal. Full native union/case metadata and execution remain pending.


### Type custom attributes (development, 2026-10-03)

Host-only namespace `NeoCLR.Metadata.Experimental.Model`:

- `CustomAttributeArgument(PrimitiveType type, object? value)` stores immutable `Type`
  and `Value`. Supported fixed arguments are String (including null), Int32 and Boolean.
  Other type/value combinations throw ArgumentException. Strings are strict UTF-8 and
  bounded to 65,536 UTF-16 code units.
- `CustomAttributeDefinition(TypeReference attributeType, IEnumerable<CustomAttributeArgument> arguments)`
  authors an instance `.ctor` reference without implicitly loading dependencies.
  Overloads taking `MethodDefinition constructor` or `ImportedMethodReference constructor`
  additionally validate the fixed arguments against the constructor signature. Owned
  constructors must be public; references must belong to the output module.
- `AttributeType` exposes the nominal owner; `GetConstructorSignature()` and `GetValue()`
  return owned copies of the CLI constructor signature and custom-attribute blob.
  `GetArguments()` returns read-only decoded fixed arguments. Inspection never runs a
  constructor. Unsupported argument/signature categories throw NotSupportedException;
  malformed supported data throws InvalidDataException.
- `TypeDefinition.CustomAttributes : IList<CustomAttributeDefinition>` is append-only on
  authored, attached definitions and read-only on loaded snapshots. The corresponding
  `TypeBuilder.AddCustomAttribute(CustomAttributeDefinition attribute)` uses the same
  ownership/limit validation. Foreign references throw ArgumentException; missing local
  constructors or exceeded graph limits reject on write. At most 256 attributes per type
  and 256 fixed arguments per attribute are supported.

Host-only namespace `NeoCLR.Metadata.Experimental.Introspection`:

- `NominalTypeInfo.GetCustomAttributes() : IReadOnlyList<CustomAttributeInfo>` returns
  cached metadata-only views in declaration order.
- `CustomAttributeInfo.Namespace` and `Name` inspect the stored owner identity;
  `GetArguments()` decodes data without resolving dependencies.
- `CustomAttributeInfo.GetAttributeType() : NominalTypeInfo` resolves the canonical
  owner through the explicit MetadataLoadContext catalog. Missing or mismatched
  dependencies throw InvalidDataException. No runtime reflection or constructor execution
  is involved.

Authoring is limited to top-level nongeneric nominal owners and type-level attributes.
Named arguments, enum/array/System.Type arguments and other parent categories are not
newly supported. Loaded CLI blobs remain available as raw copies even when typed decoding
is unsupported. This does not add mutable loaded-assembly rewriting.

CLI output uses the existing CustomAttribute table, constructor MemberRef and standard
prolog/fixed-argument/named-count blob. Native PE/#Neo output uses the runtime's existing
custom_attributes records; its CLI projection contains equivalent standard blobs. This
is an interim dual representation, not a CLI-authoritative runtime format or a new union
wire category. Native constructor-bearing records currently do not enforce CLI
System.Attribute inheritance; the .NET execution control uses System.ObsoleteAttribute.
C# tests cover actual CLR decoding, native round trips, explicit dependency resolution,
malformed records and runtime verification/execution without invoking an attribute ctor.


### Intrinsic bootstrap mappings (development, 2026-10-03)

`AssemblyBuilder.BindNativeLibrary` now permits static methods on the exact explicitly
bound core System.String owner when the native module is System. Selected methods still
require matching public signatures, results and owner identity. Native calls encode their
owner as primitive String, while CLI output retains its TypeRef/MemberRef identity.
This does not admit String as an ordinary nominal imported class or add instance members.
An imported value System.Char from that same core binding encodes native type operands as
Char, including generated formatting type tests; it is not a new public Char primitive
signature API. Missing/wrong bindings fail before returning an image.

These are existing runtime primitive representations, not new union semantics or wire
categories. An executable C# fixture checks concatenation, a nonmatching boxed Int32/Char
test, rejected owner/result/module changes and continued rejection of nominal String import.
Method-body validation failures now include the declaring method for actionable diagnostics.


## Parameter names and authored value references (development, 2026-10-03)

`MethodDefinition.ParameterNames : IReadOnlyDictionary<int, string>` exposes sparse,
zero-based declared names. `MethodDefinition.SetParameterName(int position, string? name)`
and the forwarding `MethodBuilder.SetParameterName` set a name or clear it with null.
The position must be inside the signature; names must contain 1–1024 characters and
no control characters. Loaded snapshots reject mutation. Names are descriptive and
do not change method identity or parameter modes.

CLI output uses ordinary Param names; native output uses the existing aligned
`parameter_names` array. Readers reject names outside the signature and malformed
native array lengths. `Introspection.ParameterInfo.Name : string?` returns null for
an unnamed parameter and retains names through constructed generic views.

`AssemblyBuilder.CreateValueTypeReference(AssemblyIdentity dependency,
AssemblyIdentity dependencyCoreLibrary, string artifactSha256, string namespace,
string name, int genericArity = 0)` authors a public unconstrained value identity.
It has the exact identity, core, digest and resource validation of `CreateTypeReference`.
It neither loads the dependency nor proves its storage layout.

`AssemblyBuilder.CreateNestedTypeReference(ImportedTypeReference declaringType,
string name, int genericArity = 0, bool isValueType = false)` authors a public nested
identity. The declaring reference must belong to this output and carry a native
artifact contract; generic/constructed declaring scopes are unsupported. The nested
namespace is empty and its arity is its own. Foreign scopes throw `ArgumentException`;
unsupported identities or scope categories reject through normal import validation.

`CreateMethodReference` now also accepts these value and nested owners and their
supported nominal signature operands. Instance value calls retain managed receivers;
physical nesting survives encoding. The caller supplies semantic facts and an explicit
artifact digest. This does not introduce dependency loading, virtual override authoring,
byref constructors or generic instance methods.

C# contracts exercise CLI/native name round trips, constructed parameter views, readonly
snapshots, invalid names/positions/array lengths, and symbol-authored nested generic value
constructors executed by the CLR. The native Raven union consumer additionally executes
these references on neoCLR. No metadata version change is required.

Development validation, 2026-10-03: authored value-interface edges now retain the
relationships needed by imported source Option/Result carriers. Unboxed value-to-interface
assignment remains rejected; boxing or constrained dispatch is still required. C# tests
cover idempotent value edges, wrong target kinds and foreign ownership. The unchanged
source-library consumer executes separately on neoCLR with the bounded union bootstrap.


## Function signature views (development, 2026-10-03)

`Introspection.FunctionTypeInfo : TypeInfo` is a context-owned metadata facade for an
existing native callback shape. It exposes no callable instance, invocation or reflection.

| Member | Contract |
| --- | --- |
| `TypeInfo ReturnType` | Canonical result view; primitive Void for an explicit no-result signature. |
| `IReadOnlyList<TypeInfo> ParameterTypes` | Immutable ordered value parameter views. Names and callable targets are not signature identity. |
| `bool NoResult` | True only when invocation leaves no result; an inhabited unit type is distinct. |
| `bool IsNominalType` | Always false. |
| `string DisplayName` | Diagnostic signature text, not a serialized identity or resolver key. |

`MetadataLoadContext.ResolveSignature` and member views now project function shapes,
recursively substituting owner and method parameters and resolving nominal dependencies.
Equivalent shapes share a view in one context; contexts never share identity. Missing
catalog dependencies and unscoped parameters throw InvalidDataException. Existing foreign
argument and nesting limits remain enforced. There are at most sixteen value parameters;
byref callback parameters and generic callback declarations remain unsupported. Outer
method/type parameters can appear inside a callback.

`AssemblyDefinition.ReadNativeAssembly` preserves these shapes in supported method,
field and property signatures, using the existing function encoding without a format
version change. `AssemblyBuilder.CreateMethodReference` also accepts bounded callback
operands authored from semantic facts; dependency resolution is not added to emission.
The earlier test rejecting all callback fields now checks their retained signature.

C# tests cover open and constructed owner/method substitution, canonical views, external
identity, missing dependencies, explicit no-result signatures and context isolation.
Raven currently imports value-returning callbacks through its existing callable symbols
and explicit primitive bootstrap. Explicit no-result callback import still rejects rather
than being silently changed to an inhabited-unit result. This completes ArrayList's Boolean
predicate case; it does not integrate the separate structural Function language experiments.

### Exact unboxing and generic conversions (development)

`IILGenerator.UnboxAny(SignatureType target)` and
`IILGenerator.Emit(OpCode.UnboxAny, SignatureType target)` consume an object reference
and push the requested storage type. MethodBuilder's existing raw `Emit` forwarding
overload accepts the same opcode; the convenience operation lives on the IL generator.
The target may be a primitive value, nominal value/reference, vector, or caller-scoped
method/type parameter. Void, managed references, Self and function signatures reject
with ArgumentException, as do foreign owners and unbound generic parameters. Stack
validation on write rejects non-reference input with InvalidDataException.

CLI output uses `unbox.any` (0xA5); native output uses the existing `unbox.any` operation.
Value targets require an exact boxed type and reject null or a different box at execution.
Reference targets preserve object identity and admit null under the runtime's existing
checked-reference rules. This is not a numeric conversion and creates no new native
format or semantic category. Unlike `CastReference`, a generic target may instantiate
as either a value or reference type.

```csharp
var il = method.GetILGenerator();
il.LoadArgument(0); // object parameter
il.UnboxAny(SignatureType.MethodParameter(0));
il.Return();
```

Validation includes CLR execution for primitive/method/owner generic scopes, reference
identity/null, incorrect boxes and invalid authoring/stack inputs. The unchanged native
query library separately imports and executes OfType over heterogeneous boxed values
and reference payloads; see the bootstrap query acceptance workflow.

### AssemblyBuilder.SetArrayBacking(TypeBuilder type)

Development API: selects the output-owned nominal class backing native vectors. `type`
must be a nonabstract, nonstatic, nonnested generic class with one unconstrained parameter
and exactly one private field of type `T[]`. Its local base may be absent or the
output-owned native Object root (development, 2026-10-07). Reading preserves that
relationship; runtime linking requires the exact host-selected, fieldless root.
Other bases remain unsupported. Repeating the same selection is idempotent;
a different selection throws InvalidOperationException. Null throws ArgumentNullException;
foreign/incompatible descriptors throw ArgumentException. Native writing revalidates the
selection after subsequent definition edits.

This host execution policy is encoded beside ordinary definitions in the native assembly
manifest; it does not alter CLI array signatures or make CLI arrays implement custom
interfaces. The runtime requires exactly one selected descriptor across linked assemblies.
Its storage field aliases vector identity; replacing/addressing that field and allocating
the descriptor as a nominal object are unsupported. Older readers reject this optional
format-5 field. NativeAssemblyDefinition.ReadAssembly validates its identity and storage
shape and retains it in the immutable native snapshot. See the integration documentation
for source `Array<T>`, iteration configuration and executable validation.

### Imported value overrides and slot facts

`Introspection.MethodInfo.IsNewSlot` exposes the CLI NewSlot declaration bit, including
on constructed views. A virtual method with this bit clear reuses an inherited slot;
this is metadata information, not a language-level override-resolution service.

`AssemblyBuilder.CreateMethodReference(ImportedTypeReference declaringType, string name,
MethodSignature signature, bool isStatic = false, bool isOverride = false)` additionally
accepts the bounded value override contract: an instance, nongeneric `ToString(): String`
on a value owner (including a generic value definition). Other override shapes throw
ArgumentException. Conflicting reuse-slot/ordinary references throw InvalidDataException.
Repeated equivalent references intern normally. The reference retains a managed receiver
and native override identity; it does not request class/interface virtual dispatch.
No dependency metadata is reopened by this authoring API. CLI MemberRef signatures remain
unchanged. Wider overrides remain unsupported.

The optional isOverride parameter preserves source calls but changes the experimental host
method signature; rebuild binary consumers with the matching metadata library.

### .NET acceptance service adapters (development tooling only)

The separate `NeoCLR.DotNetServices` test assembly under the bootstrap tooling exposes:

- `System.Runtime.CompilerServices.CheckedStorage.Reserve<T>(int length) -> T[]`: allocates
  a real CLR array. Negative lengths and allocation failures propagate CLR exceptions.
  Slots have CLR default initialization; native tracked-uninitialized storage is not emulated.
- `System.Runtime.CompilerServices.RuntimeFailure.Terminate(string message) -> void`:
  writes the message to stderr and exits the process with status 1. Marked DoesNotReturn.

These executable host adapters contain no reader/writer APIs or source library types and
are not selected for guest RavenDoc publication. They support the paired .NET assessment,
which currently fails later on unit storage mapping.

The development .NET acceptance service additionally exposes the public readonly empty
`System.Runtime.CompilerServices.UnitValue` struct. It is the inhabited unit storage type
selected by the explicit .NET bootstrap manifest; default initialization is its sole value.
It is not CLR System.Void and adds no metadata reader/writer or guest runtime API. The host
compiler contract and ordinary CLI signatures carry this mapping without a native format
extension. See the integration documentation for the remaining array-adapter limitation.


### Declared generic type-parameter names (development, 2026-10-03)

`GenericParameterTypeInfo.Name: string` returns the declared name from a supported native
snapshot, for example `TItem`. It has no parameters and does not resolve dependencies or
load runtime types. Identity remains the owning declaration plus Position; DisplayName
remains `!ordinal`. Equal names on different owners do not merge parameters. CLI snapshots
currently do not materialize these names: accessing Name throws NotSupportedException,
rather than inventing a name. No generic constraints or variance facts are added here.

```csharp
var parameter = (GenericParameterTypeInfo)nativeType.GetGenericArguments()[0];
string declaredName = parameter.Name;
```

This host-only member is outside the guest RavenDoc selection and is documented here.
C# contracts cover native name/ordinal/canonical identity and explicit CLI rejection.

### Primitive comparer bootstrap imports (development, 2026-10-04)

`AssemblyBuilder.ImportReference(MethodDefinition, AssemblyIdentity)` additionally
supports explicit CLI core String/Int32/Int64 members bound to an executable native System
seed. It validates snapshot/core identity, primitive owner, public concrete nonvirtual
method, parameters/results and receiver passing mode. String instance calls consume a
String value; Int32/Int64 instance calls consume a managed address of their exact width. Generic owner or
nominal layout inference is not introduced. String still cannot be imported as an
ordinary nominal class through this bootstrap mapping. `ImportedMethodReference` keeps
its original CLI owner identity and reports `RequiresManagedReceiver` for Int32 and Int64.
`IILGenerator.Call`/`Emit` validate the actual primitive receiver and preserve the ordinary
CLI member reference encoding; native output uses the existing primitive owner encoding.

The exact public virtual core `System.Object.GetHashCode() -> Int32` slot is now admitted
by `ImportReference` and `IILGenerator.CallVirtual`/`Emit(Callvirt, ...)`. Native output
requires an explicit System binding with matching virtual/result/receiver flags. Missing,
wrong or nonvirtual slots throw `InvalidDataException` before an image is returned.
This does not enable arbitrary class virtual imports. No public signature, metadata
version or runtime instruction changes are required.

C# native comparer binding checks execute String.Equals, Int32/Int64.CompareTo and Object
hash calls and reject wrong receiver modes, virtual primitive methods, wrong hash result
and unbound native hash dispatch. Int64 checks also reject wrong primitive owner, missing
method identity and wrong result width, and execute both signed extrema and equality.
The source-built StringComparer consumer tests existing
UTF-8 scalar ordering and Unicode simple folding; .NET ordinal UTF-16 ordering is not
substituted. This host API is documented manually; the separate guest RavenDoc snapshot
check remains stale and was not regenerated for this host-only change.

### Argument addresses and CLI Object signatures (development, 2026-10-04)

`IILGenerator.LoadArgumentAddress(int index)` and `Emit(OpCode.Ldarga, int index)`
load a managed address to an ordinary by-value argument. Indexing includes the implicit
instance receiver: static parameter zero uses index 0; an instance method's first
parameter uses index 1. Changes through the address affect that invocation's parameter
storage, preserving normal call-by-value isolation from the caller. Use Ldarg for an
already managed-reference parameter. Receiver slots, byref parameter slots and negative
or out-of-range indices reject with InvalidDataException during writing, even when the
instruction is unreachable. Indirect operations and calls require the exact addressed
type. Instruction-limit failures retain existing emission-time validation. Raw forwarding
MethodBuilder.Emit also accepts Ldarga; new code should use GetILGenerator().

Both writers share validation: CLI writes the standard ldarga instruction (FE 0A and a
16-bit argument index); native writes the runtime's existing ldarga operation. No runtime
or format-version change is needed. C# tests execute mutation through static and instance
parameter addresses on CLR and NeoCLR, and reject invalid slots and mismatched stores.

Imported CLI ELEMENT_TYPE_OBJECT signatures (including vector elements) now map to the
consumer's explicit CoreObjectType after compatible core validation. Primitive-only
signature recognizers retain their narrower contract; original signature bytes remain
unchanged. The native comparer binding test imports and executes Object.ReferenceEquals.
This does not add implicit dependency loading or nominal layout inference.

These host C# APIs remain covered by this manual reference, separate from guest RavenDoc
selection. The known stale guest documentation snapshot is not regenerated by this slice.

### Explicit core character signatures (development, 2026-10-04)

`AssemblyBuilder.ImportReference(MethodDefinition, AssemblyIdentity)` decodes CLI
ELEMENT_TYPE_CHAR (0x03), including vector elements, as an output-owned value reference
to System.Char in the supplied core identity. The CLI writer emits canonical 0x03 for
that exact top-level, nongeneric core value type. Other nominal types are unchanged;
the signed-core exception is limited to the existing Object and this Char identity.
CLI execution retains UTF-16 code units, including surrogate values.

With an explicit native System binding, that core Char reference uses the existing
native grapheme Char storage. Imported Char methods must be public, concrete and
nonvirtual with a managed receiver matching the seed. String.get_Item can return Char;
Char.ToString preserves the grapheme text. Calls encode the intrinsic Char owner instead
of Named(System.Char). This is a target representation difference, not reinterpretation
of .NET characters as graphemes during CLR execution.

`AssemblyDefinition.ReadNativeAssembly` materializes Char signatures using the exact
native type alias. Missing aliases, wrong namespace/category or nested character aliases
reject with InvalidDataException. `NativeAssemblyDefinition.CreateReferenceAssembly`
requires that alias to match the explicit core and writes canonical CLI Char signatures.
This declaration projection cannot represent native multi-scalar grapheme values as CLR
char values; it is not an executable value conversion. Direct native imports retain the
metadata identity without generating a CLI projection.

C# checks cover CLI scalar/vector signatures and execution, native materialization and
reimport, CLI projection, grapheme execution and invalid alias/receiver/owner/result
contracts. No metadata version or VM instruction changed. This host API remains covered
by this manual reference; the separate guest RavenDoc snapshot is still stale.


### Native erased System.Value signatures (development, 2026-10-04)

`AssemblyBuilder.ImportReference` retains System.Value as an ordinary scoped value-type
reference in the CLI projection. With an explicit native binding for the selected core
into the System module, native writing maps that exact nongeneric, top-level System.Value
identity to the existing `Value` signature. `AssemblyDefinition.ReadNativeAssembly`,
`NativeAssemblyDefinition.ReadAssembly` and signature reimport recover the scoped
System.Value identity; no runtime reflection or implicit dependency lookup is involved.
Same-named non-core types are not intrinsic carriers. Native Value requires the explicit
value alias; malformed/missing aliases and a different projection core are rejected with
InvalidDataException. This is an extension of existing signature APIs, not a new API.

The native carrier is independent of the System.Object hierarchy: erased payload type
identity must be retained, including for future structural/nominal introspection inputs.
The current integration test covers existing Int64 and Byte host outcomes and wrong-kind
unpacking. It does not establish all payload categories. Existing runtime storage/depth
and lifetime checks still apply; frame-backed references cannot escape by erasure. The
CLI reference projection is not a CLR implementation of the erased carrier.

Validation: C# `--native-erased-value <seed.neox> <core.dll>` checks signature encoding,
round-trip import and invalid aliases. The 130 metadata groups remain passing. Raven's
`bootstrap/verify_erased_values.py` independently compiles and imports a generic wrapper,
executes real ParseInt64 calls and checks runtime wrong-kind failure. No guest Raven API
snapshot signature changed; existing snapshot maintenance blockers remain recorded.


### RuntimeAssemblyContainer library PE profile (development, 2026-10-04)

- `public const int MaxLibraryImageSize = 16 * 1024 * 1024`: maximum complete schema-3
  library PE image. This is distinct from MetadataArtifactReader.MaxImageSize (4 MiB)
  and the ordinary structural-reference profile.
- `public static byte[] WriteLibraryBinary(AssemblyBuilder assembly)`: validates the
  authored graph and complete external contracts, builds its CLI reference projection,
  and writes authoritative native metadata in a required schema-3 section. Returns
  owned unsigned PE32 bytes. Null throws ArgumentNullException; invalid declarations,
  graph, dependencies, encoding or exceeded bounds throw InvalidDataException.
- `RuntimeAssemblyContainer.Read`, `ReadCliProjection` and
  `AssemblyDefinition.ReadNativeAssembly` now accept library schema 3. Native semantic
  import reads the native declaration graph; ReadCliProjection remains reference-only.

Library transport is bounded to 16 MiB PE, 8 MiB envelope, 32 MiB host JSON, 2,097,152
binary nodes and depth 64. Existing declaration, signature, body and aggregate literal
limits remain in force. The ordinary Write/WriteBinary APIs and schema-1/2 readers keep
4 MiB PE/1 MiB envelope bounds. Public NativeAssemblyDefinition.ReadAssembly(JSON)
retains its 4 MiB input bound; library container reading uses its bounded internal path.
Older runtimes reject the required schema-3 section. Standalone schema-3 transport
already existed; this change admits that profile inside the bound PE container.

```csharp
byte[] image = RuntimeAssemblyContainer.WriteLibraryBinary(builder);
var declarations = AssemblyDefinition.ReadNativeAssembly(image);
```

C# tests prove legacy writer rejection, >4 MiB library read/projection/native reimport,
owned snapshots, over-budget rejection and linked execution. The malformed-schema
fixture now uses unsupported schema 4, because schema 3 is deliberately supported.
131 metadata groups pass. The combined-source compiler gate separately proves a
53-source native library, unchanged broad application and MemoryStream execution.
No public guest Raven signature changed; the manual development reference covers these
host C# APIs. No website build or performance claim is part of this change.

### Int32 enum authoring and introspection (development, 2026-10-04)

`AssemblyBuilder.AddEnum(string namespace, string name, TypeVisibility visibility = Public)`
returns a `TypeBuilder` for a nongeneric top-level enum. It creates the selected core
`System.Enum` base and a Public/SpecialName/RTSpecialName Int32 `value__` field.
`TypeBuilder.AddEnumMember(string name, int value)` returns a literal `FieldBuilder`;
its definition has Public/Static/Literal/HasDefault flags and the owning enum signature.
Aliases and unnamed integer values are valid. Names must be unique ASCII identifiers.
`TypeBuilder.Fields` retains its existing instance-storage meaning; all declarations,
including literals, are available through `TypeBuilder.Definition.Fields`.

Manual definitions use the same validation path: attach a sealed `TypeDefinition`
extending the explicit core System.Enum, with `value__` attributes 0x606 and Int32
signature, then add fields with attributes 0x8056, the attached enum signature and an
Int32 constant. The existing three-argument `FieldDefinition` constructor is retained;
the four-argument overload accepts `int? constant`. Constants require literal flags;
ordinary fields cannot carry one. Duplicate or invalid declarations throw
`ArgumentException`; invalid final enum shapes throw `InvalidDataException` on writing.
Flags, other widths, generic/nested enums, interfaces and enum-owned methods/properties
are unsupported by this authoring profile. Existing runtime/frontend flags support
is unaffected.

`TypeDefinition.IsEnum`, `TypeBuilder.IsEnum` and introspection
`NominalTypeInfo.IsEnum` report enum declarations. `FieldDefinition.IsLiteral` and
introspection `FieldInfo.IsLiteral` identify static literals; their `int? Constant`
returns a supported Int32 constant or null. CLI readers retain other raw signature
bytes; this does not expand CLI nominal field-signature decoding. Native readers
project canonical nominal literal signatures and reject unsupported enum shapes.

`AssemblyBuilder.CreateEnumReference(AssemblyIdentity dependency,
AssemblyIdentity dependencyCoreLibrary, string artifactSha256, string namespace,
string name)` authors a public output-owned enum identity from symbol/host facts.
It has the same identity, digest and dependency validation as CreateValueTypeReference.
It does not load a dependency. `ImportReference` of a native enum definition registers
its enum category as well. Actual native dependency linking/verification remains required.

`IILGenerator.ConvertToEnum(SignatureType enumType)` consumes Int32 and produces the
nominal enum; `ConvertFromEnum` performs the inverse. MethodILGenerator implements both.
Foreign operands and non-enum signatures reject; stack validation checks operands.
CLI emission requires no instruction because the evaluation stack carries the underlying
integer. Native emission uses existing newobj and conv.i4 with nominal enum storage.
These are semantic helpers, not new raw opcodes or a format-version change.

Validation: the C# enum contract group covers builder/manual parity, CLI execution,
native/CLI declaration round trips, facade facts and malformed metadata rejection.
`--enum-runtime` additionally loads a separately encoded native library and consumer.
The paired Raven `bootstrap/verify_enums.py` driver compiles the unchanged TaskState
source and exercises mutation, array storage, comparisons and integer conversions on
both targets. No guest public API is added; the existing guest snapshot maintenance
blocker is unchanged.

### Native function identity clarification (2026-10-04)

Native metadata encodes a function type as `Function { parameters, returns, no_result }`.
It does not identify a nominal delegate class. The introspection load context interns
resolved shapes: equal ordered parameter/result identities and the return convention
share a `FunctionTypeInfo` instance within that context. Different contexts retain
separate views. Function objects carry callable targets and optional bound receivers;
neither participates in type identity, and the metadata facade cannot invoke them.
Raven's current Func/Action symbol transport is a compiler adapter, not native identity.
Inhabited unit and no-result invocation remain distinct. Where Raven converts a
no-result method to a unit-producing callback, the native emitter generates an
output-owned adapter that invokes the original callback and produces the configured
unit value. This allocates an adapter/closure; it is not runtime signature coercion.


### Authored Self implementation contracts (2026-10-04)

`AssemblyBuilder.CreateMethodReference` accepts `SignatureType.Self` for an authored
interface's instance contract, including nested supported signature shapes. A nominal
class method contract containing Self is rejected with InvalidDataException. Existing
ownership, completeness and signature-scope validation still applies.

When writing an implementing class/value type, required interface methods substitute
Self with that implementing type (the open construction for a generic owner). This
applies to local and output-owned external interface contracts, including inherited
requirements and arrays/constructions. Wrong concrete parameter/result signatures fail
with InvalidDataException before output publication. The interface metadata retains
Self; the implementation's declared methods use concrete signatures. This is conformance
validation, not an implicit conversion or a new dispatch opcode. Executable CLI output
with symbolic Self remains unsupported; native SelfType encoding is unchanged.


### Floating-point signatures and IL generation (development, 2026-10-04)

`PrimitiveType.Single` and `PrimitiveType.Double` represent CLI binary32/binary64
signature types. Readers retain these identities in methods, fields and properties;
value-type generic constraints admit both. No nominal wrapper type is introduced.

The public `IILGenerator` returned by `MethodBuilder.GetILGenerator()` provides:

```csharp
void LoadConstant(float value);
void LoadConstant(double value);
void Emit(OpCode opCode, float operand);
void Emit(OpCode opCode, double operand);
```

The raw overloads require `Ldc_R4` and `Ldc_R8`, respectively; an incompatible opcode
throws `ArgumentException` without appending an instruction. Constants retain their
IEEE bits, including signed zero, infinities and NaNs. Operandless `Emit` accepts
`Conv_R4` and `Conv_R8`; the existing integer conversions also accept floating inputs.
This is unchecked numeric conversion, not a guarantee for out-of-range conversions.

Flow validation admits matching Single/Double arithmetic, remainder, negation and
Ceq/Clt/Cgt comparisons. Mixed numeric types require explicit conversions. Bitwise
operations and shifts still reject floating operands with `InvalidDataException`
before output is returned. CLI output uses the standard signatures and opcodes;
native output uses the existing Single/Double types and neoIL instructions.

Native literals store integer bit patterns, not JSON/CBOR floating-point values.
`RuntimeAssemblyContainer.WriteBinary(AssemblyBuilder)` chooses existing schema 3
when a Double literal needs the full UInt64 range (including negative zero).
Otherwise it retains schema 2. The application host-image size limit is unchanged.
The native-bytes overload continues to require schema-2-compatible input;
`WriteLibraryBinary` always uses schema 3. Older schema-2-only readers reject schema 3.
No runtime or format-version change is introduced by this writer extension.

`FloatingPointChecks` exercises CLI and native signature round trips, literal overload
validation, invalid bitwise operations and .NET execution. Set `NEOCLR_FLOAT_ARTIFACT`
when running the C# tests to save the matching executable native assembly, then run it
with neoCLR and the explicit System seed. Success exits 42 after arithmetic, NaN and
signed-zero checks. This does not establish Raven floating-point code generation or
complete compilation of the Single/Double class-library sources.


### Unordered floating comparisons (development, 2026-10-04)

`IILGenerator.Emit(OpCode.Clt_Un)` and `Emit(OpCode.Cgt_Un)` consume two matching
numeric operands and produce Boolean: unsigned integer less/greater, or floating
less/greater respectively, or true if either floating operand is NaN. CLI output uses
`clt.un`/`cgt.un`; native output uses the same existing instructions.
Operand-bearing overloads reject these opcodes with `ArgumentException`.

Combining `Cgt_Un` with Boolean negation implements ordered `<=`, and `Clt_Un`
with Boolean negation implements ordered `>=`. The floating metadata C# contract
executes NaN in both Double operand positions and Single NaN on .NET and NeoCLR.
The Raven driver gate separately validates all six source comparison operators.


### Static interface contracts and inherited Self (development, 2026-10-04)

`TypeBuilder.AddInterfaceMethod(string name, MethodSignature signature, bool isStatic)`
adds a public bodyless contract, allowing nongeneric static members alongside existing
instance members. The original two-argument overload retains instance behavior.
Static properties reuse `AddProperty` with static accessors. Invalid owners throw
InvalidOperationException; invalid/duplicate signatures throw ArgumentException.
Instructions or locals on abstract methods fail write validation with InvalidDataException.

Detached `MethodDefinition` accepts Public | Static | Abstract | Virtual | NewSlot,
with the same attachment and signature validation. `CreateMethodReference` accepts
static authored interface contracts; the host still supplies and completes the entire
contract explicitly. Required implementations match static/instance classification,
visibility, name and substituted parameter/result types. An instance method cannot
satisfy a static requirement, nor the reverse. Missing/wrong implementations fail
before writing output. CLI output includes MethodImpl rows for static implementations;
native output uses existing bodyless static interface declarations, without virtual
instance dispatch flags. Symbolic Self still requires native output or a CLI reference
projection, not executable CLI Self semantics.

An interface may inherit a local or explicitly scoped external construction such as
`ComparableTo<Self>`. Classes cannot declare unbound Self inheritance. Reader and
introspection views retain that scoped Self argument; declaration/constructed owner
views supply the scope, and conformance substitutes the concrete implementer.
No generic parameter is added. The facade does not load code or invoke contracts.

`StaticInterfaceChecks` verifies definition/builder parity, local/external static
conformance and rejection, inherited Self views, native round trips, and observable
.NET constrained static dispatch. The API-authored native program exits 42. The
Raven Number gate additionally executes actual source contracts across three native
assemblies. Compiler generic callself emission and intrinsic primitive ownership
remain separate work; this authoring support does not claim they are complete.


## Fixed-width integer signatures and unsigned instructions (development, 2026-10-04)

`PrimitiveType.SByte`, `Int16`, `UInt16`, `UInt32` and `UInt64` complete the eight
fixed-width integer signatures alongside Byte, Int32 and Int64. They can be used in
method results/parameters, fields, properties, vectors, generic arguments and
`IILGenerator.DeclareLocal(PrimitiveType)`. The local overload also accepts Single
and Double; Void and undefined enum values reject with ArgumentException.
Readers preserve exact storage types. CLI signatures use the standard element codes;
native signatures use existing primitive names without a new format version.

Evaluation normalizes SByte/Byte/Int16/UInt16/UInt32 to Int32 and UInt64 to Int64 bits,
as CLI does. Storage and by-reference element identities remain exact. Unsignedness
is an instruction property, not inferred from a stack value's storage signature.

Operand-free `IILGenerator.Emit(OpCode)` additionally accepts:

| Opcode | Input and result |
| --- | --- |
| Conv_I1, Conv_I2 | Numeric input, low signed 8/16 bits sign-extended to Int32 |
| Conv_U2, Conv_U4 | Numeric input, unsigned 16/32-bit conversion, Int32 evaluation bits |
| Conv_U8 | Numeric input, unsigned 64-bit conversion; zero-extends Int32 bits |
| Conv_R_Un | Int32/Int64 bits interpreted unsigned, converted to Double |
| Div_Un, Rem_Un | Two matching integer evaluation types; unsigned quotient/remainder |
| Shr_Un | Integer and Int32 count; zero-filled right shift |
| Clt_Un, Cgt_Un | Matching integers compared unsigned, or matching floats with unordered semantics |

These emit standard CLI opcodes and existing native instructions. Floating operands
for unsigned division, remainder, shift or Conv_R_Un reject during body validation;
integer width mismatches likewise reject. Conversion to UInt64 from a signed Int32
requires sign extension with Conv_I8 when that is the source-language contract.
Use `Emit(OpCode.Ldc_I8, longBits)` for 64-bit integer constants.

`IntegerWidthChecks.cs` executes the same authored program on .NET and NeoCLR,
including high-bit arithmetic, conversion and signature round trips. The separately
compiled Raven integer gate covers native import, fields/properties, arrays and generic
calls. This does not yet make source-owned primitive Number implementations or generic
Number-constrained dispatch complete.


## Native numeric declaration representation (development, 2026-10-04)

`TypeDefinition.NativePrimitive`, `TypeBuilder.NativePrimitive` and introspection
`NominalTypeInfo.NativePrimitive` return `PrimitiveType?`. Non-null means that the
native declaration implements that canonical runtime scalar. Null means ordinary
metadata; a CLI System name alone never establishes native ownership.

`TypeDefinition.SetNativePrimitive(PrimitiveType primitive)` works on authored
(including detached) definitions. `TypeBuilder.SetNativePrimitive` forwards to the
same validation. The declaration must be the matching `System.<primitive>` sealed,
sequential, nongeneric, top-level value type with no record fields or constructors.
Only the ten fixed-width integer/floating primitives are admitted. Other categories,
wrong names, conflicting designations or incompatible storage throw ArgumentException;
loaded snapshots throw InvalidOperationException. The writer revalidates after edits,
so adding fields or constructors later cannot bypass the contract.

```csharp
var scalar = assembly.AddValueType("System", "Double");
scalar.SetNativePrimitive(PrimitiveType.Double);
var read = scalar.AddInstanceMethod("Identity", new(PrimitiveType.Double, []));
var il = read.GetILGenerator();
il.LoadArgument(0); // managed address of Double, not an ordinary record
il.LoadObject(PrimitiveType.Double);
il.Return();
```

An owned scalar's signature and implementing Self are the primitive signature. Native
output uses existing canonical System names, scalar owners, Runtime representation
and managed receiver instructions. No new native instruction or metadata version is
introduced. The snapshot and introspection preserve this fact; importing a native
method preserves its exact assembly dependency and scalar receiver. No runtime code
or reflection API is loaded during import.

Executable CLI output rejects native primitive declarations and calls to their
imported members with InvalidDataException: these are not ordinary structs hosted in
another .NET assembly. Reference-only CLI projections remain available for diagnostics;
only authoritative native snapshots retain NativePrimitive. This follows the CLR's
separation of intrinsic scalar storage from ordinary value layouts without claiming
that an arbitrary .NET library can replace core numeric types.

The designation does not supply runtime storage fields, resolve dependency ownership,
or rewrite a source `m_value` field. A compiler must select ownership explicitly and
translate its checked intrinsic-storage contract into receiver operations. Duplicate
canonical primitive owners remain invalid at runtime linking. Raven source primitive
integration and generic Number constraints are not established by this API alone.

`PrimitiveRepresentationChecks` covers detached/builder authoring, strict malformed
input, immutable snapshots, introspection, late invalid edits and independent native
method import. With `NEOCLR_PRIMITIVE_ARTIFACT` and `NEOCLR_PRIMITIVE_CONSUMER` set,
the C# suite saves a primitive library and consumer; running the consumer with that
module and the explicit seed returns 42 after mutation through its primitive receiver.


`AssemblyBuilder.CreateMethodReference` also accepts optional
`PrimitiveType? nativePrimitive = null`. Supply the same explicit numeric designation
for every member of an output-owned canonical System value reference. This authors
scalar receivers and dependency records using only identity/signature facts; it does
not reopen reader definitions or validate the dependency's implementation. Invalid
names, nesting, arity, constructors or overrides throw ArgumentException; conflicting
primitive/nominal owner contracts throw InvalidDataException. Ordinary references keep
the null default. Runtime linking verifies the supplied dependency. Executable CLI
output rejects these native calls. `PrimitiveRepresentationChecks` now also saves
`NEOCLR_PRIMITIVE_CONSUMER + ".authored.neox"`, an independently executable consumer
created through this symbol-facts-only path.


Native primitive definitions and authored member references now use canonical runtime
member names, such as `System.Int32.CompareTo`, so seed-facing primitive calls retain
their ABI spelling. The native reader accepts the preceding encoded member spelling
as well and retains each artifact's actual callable/accessor names. This does not
change signatures, opcodes or container version. Numeric interface resolution uses
the declaration origin name when needed; exact type, visibility and parameter-mode
contracts are still validated. Existing libraries must be rebuilt when ownership
moves from the implicit System seed to a separately referenced source library.

Earlier experimental metadata readers that require encoded primitive member names
must be upgraded before reading newly emitted canonical members. Existing runtime
container and instruction formats are unchanged.

Raven’s symbol-authored primitive references use the new canonical ABI. Rebuild older
experimental primitive-provider artifacts for that path; reader-mediated ImportReference
continues to preserve the older artifact’s executable name.


### Static constrained interface dispatch (development, 2026-10-04)

`IILGenerator.CallConstrained(implementingType, staticInterfaceMethod)` and
`Emit(OpCode.Call, implementingType, staticInterfaceMethod)` now author the static
counterpart of the instance operation above. The implementing type is an output-owned,
nongeneric class or value type, including explicitly designated native numeric types.
The interface and its nongeneric method must be owned by the same output assembly;
external/constructed contracts and open method/type parameters remain unsupported.

```csharp
var il = main.GetILGenerator();
il.LoadConstant(42);
il.CallConstrained(implementation, interfaceEcho);
il.Return();
```

This consumes the declared arguments and produces the declared result, without a
receiver value or address. The writer checks the complete interface implementation.
`Self` inside parameter/result signatures substitutes the implementing type, including
nested signature shapes already supported by implementation conformance. Invalid stack
operands fail writing with InvalidDataException. Invalid opcodes or ownership/conformance
fail before the instruction is appended with ArgumentException; null operands use
ArgumentNullException. The current instruction-count limit remains in force.

The CLI writer emits standard `constrained.` followed by `call`; the native writer uses
existing nonborrowed `callself`. Native Self signatures still cannot be emitted as an
executable CLI assembly. C# tests execute ordinary static contracts on .NET and save
native ordinary/Self artifacts via `NEOCLR_STATIC_ARTIFACT` (the Self image appends
`.self.neox`). Both native artifacts return 42. This is a metadata-generator capability,
not yet support for generic Number-constrained Raven programs. No schema or runtime
instruction change is introduced.


### Method interface bounds (development, 2026-10-04)

`Model.GenericMethodInterfaceConstraint(int ParameterIndex, TypeReference InterfaceType)`
records a zero-based method type parameter ordinal and a local or external nongeneric interface reference.
`MethodDefinition.InterfaceConstraints` and `MethodBuilder.InterfaceConstraints` expose
read-only lists of these records. Define bounds using:

```csharp
void MethodDefinition.AddInterfaceConstraint(int parameterIndex, TypeDefinition interfaceType);
void MethodDefinition.AddInterfaceConstraint(int parameterIndex, TypeReference interfaceType);
void MethodBuilder.AddInterfaceConstraint(int parameterIndex, TypeBuilder interfaceType);
void MethodBuilder.AddInterfaceConstraint(int parameterIndex, ImportedTypeReference interfaceType);
```

The builder delegates to the canonical definition. Detached definitions may be attached
later; writing revalidates module ownership. A null interface throws ArgumentNullException;
invalid ordinal, duplicate, foreign attached owner, noninterface, generic interface or
more than 128 bounds throws ArgumentException. Loaded snapshots are immutable and reject
mutation with InvalidOperationException. Unsupported loaded constraint categories cause
InterfaceConstraints to throw InvalidDataException rather than return a partial contract.

`Introspection.MethodGenericParameterTypeInfo.GetInterfaceConstraints()` returns
`IReadOnlyList<NominalTypeInfo>` using canonical views in the same metadata load context.
External bounds resolve through the explicit dependency catalog and must resolve to nongeneric
interfaces. Missing/wrong dependencies or noninterface bounds throw InvalidDataException.
No implicit dependency loading occurs. Method construction in introspection substitutes
signatures; language admission and constraint satisfaction remain compiler responsibilities.

`MethodBuilder.MakeGenericInstance` checks concrete owned arguments against every bound;
writing repeats this validation so later edits cannot invalidate an existing call silently.
Method-parameter forwarding is now checked against the caller bounds at emission; see the external dispatch update below. Constructed argument satisfaction remains bounded by supported explicit interface conversions.
`AssemblyBuilder.ImportReference(MethodDefinition, ...)` rejects bounded methods with
NotSupportedException until its contract can preserve their bounds. Unconstrained imports
retain their previous behavior.

CLI output uses GenericParamConstraint rows referencing interface TypeDefs or scoped TypeRefs. Native
output uses existing function generic_constraints / TypeBound records. Native snapshots
retain both bound kinds. The legacy reference-only CLI projection preserves local bounds
and rejects external bounds with NotSupportedException; use direct native import. No runtime
schema change is required; older experimental readers may reject the added function field.
C# tests cover builder/definition parity, executable CLI output, native/projection round
trips, canonical views, invalid ordinals/duplicates and late graph mutation. A native
API-authored bounded method executes with exit 42. This does not yet enable Raven's
Number-constrained generic bodies or calls.


External method bounds require a reference registered through CreateInterfaceReference,
with an explicit dependency identity/core/digest and CompleteInterfaceReference before
writing. Both definition and builder overloads use this output-owned contract; they do
not reopen importer objects. Definition-level external bounds require an attached method.
Foreign/unregistered references, duplicates and generic bounds reject. Concrete local
arguments may satisfy external bounds through their declared interface relationships;
method-parameter forwarding is now checked at emission. Direct snapshot import of bounded methods remains unsupported; see the symbol-authored call-reference distinction below.

Migration (development only, 2026-10-04): GenericMethodInterfaceConstraint.InterfaceType
is now TypeReference. Use MetadataLoadContext.Resolve for canonical views or an explicit
resolver for TypeReference.Resolve. The TypeDefinition authoring overload is retained.
Separate contract fixtures execute on .NET and NeoCLR; the direct Raven importer checks
valid/invalid arguments and canonical external symbol identity without a CLI projection.


### Open constrained static calls (development, 2026-10-04)

The independent metadata `IILGenerator`, returned by `MethodBuilder.GetILGenerator()`,
now exposes these additional overloads:

```csharp
void CallConstrained(SignatureType implementingType, MethodBuilder target);
void Emit(OpCode opCode, SignatureType implementingType, MethodBuilder target);
```

`implementingType` must be `SignatureType.MethodParameter(index)` in the body's method
scope. The parameter must have an owned nongeneric interface bound admitting the target
interface, directly or through inheritance. `target` must be an owned nongeneric static
abstract interface method. Raw Emit accepts only OpCode.Call. Null operands throw
ArgumentNullException; missing bounds, invalid scope, foreign targets, instance methods
and unsupported opcodes throw ArgumentException before adding an instruction. Writing
revalidates the operands and the body stack.

Self in the target signature substitutes the method parameter for argument/result
verification, including supported nested signature shapes. CLI emits standard constrained.
with an MVAR TypeSpec followed by call. Native output uses the existing nonborrowed
callself operation with MethodTypeParameter. No runtime or metadata format change is
required. The earlier TypeBuilder receiver overloads retain their existing behavior.

Tests execute a generic static-interface call on both .NET and NeoCLR. A separate native
Self fixture instantiates the method with Double, adds 20 and 22 and returns exit 42.
Typed/raw calls and inherited bounds are covered. Type-owner parameters and open instance calls are outside this owned-target overload.
The subsequent external overloads and Raven integration are documented below. CLI Self representation is not introduced by
this API; the portable CLI fixture uses ordinary scalar signatures.


### External constrained dispatch and Number integration (development, 2026-10-04)

Additional `IILGenerator` overloads are available beside the owned-target overloads:

```csharp
void CallConstrained(SignatureType implementingType, ImportedMethodReference target);
void Emit(OpCode opCode, SignatureType implementingType, ImportedMethodReference target);
void CallConstrained(SignatureType implementingType, ImportedConstructedMethodReference target);
void Emit(OpCode opCode, SignatureType implementingType, ImportedConstructedMethodReference target);
```

The implementing operand is an in-scope method parameter. The nongeneric external
reference overload accepts static nongeneric abstract interface contracts and Call.
The constructed reference overload accepts a completed constructed interface contract,
with no method arguments, using Call for static methods and Callvirt for instance methods.
Instance calls consume a managed receiver address followed by their explicit arguments.
The caller's method bounds must entail the target interface. Traversal substitutes Self
with the implementing parameter, so Number's `ComparableTo<Self>` relationship admits
CompareTo on ComparableTo<!!0>. Contract identities, declaring arguments and methods
remain output-owned. No importer or runtime reflection object is retained.

Null operands throw ArgumentNullException. Unsupported scope, foreign references,
missing bounds, wrong opcode or target category throw ArgumentException. Incomplete
contracts or excessive relationship traversal throw InvalidDataException. Validation
repeats before writing; stack argument/result mismatches reject before publication.
Standard CLI constrained./call or constrained./callvirt and existing native callself
encode these calls. No native format or runtime behavior change is required.

`MethodBuilder.MakeGenericInstance` now permits provisional method-parameter arguments.
Call emission validates the forwarding caller's scope and bounds; writing rechecks them.
Creating a reference alone does not prove that an arbitrary caller can use it. Concrete
arguments still validate at reference creation. Generic owner-parameter forwarding and
special method constraints remain unsupported. The direct snapshot convenience overload
ImportReference(MethodDefinition, ...) still rejects bounded methods; Raven authors
ordinary call references from validated symbol contracts, retaining the bounds in the
producer's method definitions as CLI metadata does.

```csharp
void AssemblyBuilder.SetNativePrimitive(ImportedTypeReference type, PrimitiveType primitive);
```

This explicit host assertion binds an output-owned external numeric value declaration
to its canonical scalar category. It requires a nongeneric top-level System type whose
name matches one of the ten numeric PrimitiveType values. The reference already carries
its exact dependency identity/core/digest. A null reference throws ArgumentNullException;
a foreign or incompatible reference throws ArgumentException; a second dependency owning
the same scalar throws InvalidDataException. Repeating the same designation is idempotent.
Declared interface conversions can then prove scalar generic arguments satisfy bounds.
This does not infer primitive ownership merely from an arbitrary metadata name.

C# tests execute external static and constructed instance calls plus bounded forwarding
on .NET and NeoCLR. The native compiler gate independently rebuilds numeric sources,
imports them into a generic algorithms library, and imports both emitted libraries into
a source-free consumer covering every Number member across all ten numeric types.


## Explicit interface bodies (development, 2026-10-04)

```csharp
public sealed record ExplicitInterfaceImplementation(
    InterfaceImplementation Interface, string MemberName);
IReadOnlyList<ExplicitInterfaceImplementation> MethodDefinition.ExplicitInterfaceImplementations { get; }
void MethodDefinition.AddExplicitInterfaceImplementation(InterfaceImplementation contract, string memberName);
void MethodBuilder.AddExplicitInterfaceImplementation(TypeBuilder contract, string memberName, params SignatureType[] arguments);
void MethodBuilder.AddExplicitInterfaceImplementation(ImportedTypeReference contract, string memberName);
```

These APIs map a private concrete nongeneric instance body to a member of an interface
implemented by its declaring type, including inherited and constructed external contracts.
`Interface` preserves the output-owned declaration and copied generic arguments;
`MemberName` is the declared method name (`get_Count` for a property getter). The body's
signature selects the overload. Builders delegate to the same definition validation.
External references require the existing complete, explicitly supplied interface contract.
No importer or runtime reflection object is reused and no dependency is implicitly loaded.

Null contracts throw ArgumentNullException. Invalid visibility/category, foreign references,
duplicate mappings and invalid scopes throw ArgumentException. Detached or loaded definitions
cannot be edited (InvalidOperationException). Writing rejects missing members, incompatible
signatures or multiple bodies for the same interface slot with InvalidDataException.
Mappings are limited to 128 per body; generic explicit methods and static explicit bodies
are outside this profile. Call `method.GetILGenerator()` to emit the body separately.

CLI output uses standard MethodImpl rows and private/final/virtual/newslot bodies. Native
output uses the existing scoped `interface_implementations` contract and runtime dispatch;
there is no new metadata version. Native snapshots expose mappings and method flags.
CLI snapshot MethodImpl materialization remains unsupported: reading this property throws
NotSupportedException. Native reference-only CLI projection also rejects these mappings
rather than silently discarding them; direct native semantic import is supported.

Qualified explicit property names are descriptive metadata names; actual accessor identity,
visibility and signatures remain validated independently. The runtime permits bounded ASCII
qualification used by Raven, not arbitrary new identifier syntax. C# contracts cover definition/
builder parity, two same-named interface slots, native round trips, private CLI dispatch,
external generic diamond contracts and incompatible implementations.


## Runtime-owned String declaration (development, 2026-10-04)

`TypeDefinition.SetNativePrimitive(PrimitiveType.String)` and the corresponding
TypeBuilder convenience method now designate the canonical nongeneric top-level
System.String **reference** declaration. Numeric designations still require value types.
String must be a nonabstract, noninterface class with no fields or static constructors.
Instance constructors are supported through AddConstructor and ordinary .ctor definitions. The
same validation repeats at write time. Wrong category, identity, storage or conflicting
designation throws ArgumentException; loaded definitions remain immutable.

`AssemblyBuilder.SetNativePrimitive(ImportedTypeReference, PrimitiveType.String)` and
`CreateMethodReference(..., nativePrimitive: PrimitiveType.String)` accept an output-owned
external String reference. They retain the existing explicit dependency identity, core and
digest requirements. No declaration is inferred from its name alone. Foreign/wrong-category
references reject; conflicting owners reject with InvalidDataException.

String instance bodies receive the string reference directly: LoadArgument(0) is the
storage value. Do not use the numeric managed receiver's ldobj/stobj convention. Native
encoding uses the existing Runtime representation, String signature and System.String
identity. Its is_reference_type field describes ordinary record-class semantics, so is
false for this runtime-owned representation; the reader derives String's reference category
from its canonical Runtime identity. No format version or runtime instruction changed.
Native snapshots retain NativePrimitive=String and IsValueType=false. Executable CLI
writing still rejects native primitive implementations; this is not a host System.String
replacement. Char remains outside this API and retains its grapheme representation.

C# tests cover definition/builder category parity, native round trips, invalid value-type
String designation and explicit external method references. Native local and separately
encoded consumer calls execute with reference receivers. The fixture uses an empty System
seed to exclude competing String ownership; it is a metadata contract test, not the full
Raven class-library bootstrap gate.


## Owned native grapheme declaration (development, 2026-10-04)

```csharp
bool TypeDefinition.NativeGrapheme { get; }
void TypeDefinition.SetNativeGrapheme();
bool TypeBuilder.NativeGrapheme { get; }
void TypeBuilder.SetNativeGrapheme();
void AssemblyBuilder.SetNativeGrapheme(ImportedTypeReference type);
bool NominalTypeInfo.NativeGrapheme { get; }
```

SetNativeGrapheme designates the canonical top-level, nongeneric System.Char value
declaration as runtime-owned Unicode grapheme storage. It must have sealed sequential
value-type attributes, no fields and no constructors. Builder convenience delegates to
exactly the definition validation. Wrong identity, category or storage throws
ArgumentException. Loaded definitions throw InvalidOperationException. Writing revalidates
so adding a field after designation cannot bypass the contract. Repeated designation is
idempotent. NativePrimitive remains null: no numeric or UTF-16-code-unit category is added.

A local Char signature uses the owning nominal declaration in the API and the existing
native Char signature in the artifact. Instance bodies receive a managed value address:
LoadArgument(0), LoadObject(characterDefinition), Return() preserves the whole grapheme.
The native reader and introspection facade retain NativeGrapheme, IsValueType and canonical
signature identity. Native Runtime representation and System.Char callable names are reused;
no format-version change is needed.

Executable CLI output rejects this native storage designation. Ordinary imported .NET Char
and its UTF-16 code-unit signature remain unchanged. This does not replace the .NET char
contract. Native snapshot ImportReference(TypeDefinition/MethodDefinition) preserves the
storage designation. For emission from independent semantic facts, create an output-owned
System.Char value reference with the dependency identity, core and artifact digest, call
AssemblyBuilder.SetNativeGrapheme(reference), then CreateMethodReference. This loads no
metadata. Repeated designation of the same reference is harmless. Null throws
ArgumentNullException; foreign references and incompatible identity/category throw
ArgumentException. Conflicting owners and designation after ordinary method authoring
throw InvalidDataException. Native writing also rejects competing local/external owners.
Constructors, overrides and numeric primitive reinterpretation are unsupported on authored
grapheme methods. The caller supplies complete signature facts; runtime linking validates
them against the actual dependency. Native binding tables retain the external assembly
scope, and introspection resolves it through the explicit metadata catalog.

Raven source-owned Char provider selection now uses this designation. This API does not silently
reinterpret an ordinary .NET Char reference as a grapheme.

The API contract test authors the type and a managed receiver method, round-trips its
metadata and executes a separate neoIL caller with combining-mark and ZWJ emoji graphemes.
The caller uses an empty System seed without a competing Char declaration. It is not proof
of the full source-built Char class library. Unicode text is the text model; grapheme values
preserve complete text, while UTF-8 is its native encoding.


### Runtime String construction and bootstrap Char signatures

Owned String constructors return Void and retain standard instance .ctor flags and
parameter signatures. Their IL generator may StoreArgument(0) to replace the private
construction receiver with a String. Other instance methods still reject receiver stores.
NewObject returns the completed immutable String; empty construction starts with empty
text. Numeric primitive and grapheme constructors remain rejected. Native snapshots retain
constructor classification; both imported and independently authored String constructor
references are supported. Executable CLI output remains unsupported for native storage.

When a System native bootstrap is explicitly bound, imported CLI Char signature elements
resolve to the graph's designated local/external grapheme owner, including inside vectors.
With no such binding, Char preserves its original CLI core scope even if the output
also declares a native grapheme type. This is a bridge signature rule, not implicit
loading or a new .NET Char representation.

## Native class-base reader views (development, 2026-10-04)

C# namespace `NeoCLR.Metadata.Experimental.Introspection`:

- `NominalTypeInfo.BaseType: TypeInfo?` resolves a recorded native class base to the
  canonical same-context view. Null means the native declaration has no explicit base.
  The initial supported shape is a local, top-level, nongeneric reference class base
  of a local, top-level, nongeneric reference class. This does not enumerate inherited
  members, synthesize System.Object, load files or instantiate runtime objects.
- Reading this property on CLI snapshots throws `NotSupportedException`: their base
  relationships are not yet materialized. Do not interpret absent reader support as
  proof of no inheritance. Constructed/external native bases remain unsupported and
  reject when the snapshot is read.

C# namespace `NeoCLR.Metadata.Experimental.Model`:

- Existing `TypeDefinition.BaseType: TypeReference?` now retains those loaded native
  bases as owned definition references. `Resolve()` returns the original parent
  definition. Authored behavior is unchanged; loaded CLI base decoding remains pending.
- `AssemblyDefinition.ReadNativeAssembly(ReadOnlySpan<byte>)` also accepts standalone
  schema-2/3 NEOX images containing the supported assembly manifest, in addition to
  PE/#Neo. General module inventories without that manifest remain unsupported.
  `Write()` on an immutable snapshot copies the original container; it does not convert
  NEOX to PE or reconstruct an editable graph. Existing bounds and validation apply.
- Missing targets, cycles, static/value/interface/primitive bases and unsupported
  generic/external relationships throw `InvalidDataException`. The native reader does
  not validate opaque method bodies; runtime verification remains required.
- `NativeAssemblyDefinition.CreateReferenceAssembly` throws `NotSupportedException`
  for these inherited declarations until the writer can preserve them. Container
  writers that require that projection also reject; none silently flatten the hierarchy.

Validation: ClassBaseReaderChecks covers definition/facade identity, snapshot copying,
missing/self/cyclic/unsupported bases and explicit rejection of lossy projection and
unmaterialized CLI base views. These are .NET-host APIs documented manually here; no
guest RavenDoc signature was added. Builder authoring, external/generic base projection,
Raven symbols and emission remain subsequent work.

## Local class-base authoring (development, 2026-10-04)

```csharp
TypeBuilder AssemblyBuilder.AddClass(string @namespace, string name,
    TypeBuilder baseType, TypeVisibility visibility = TypeVisibility.Public);
```

This overload creates an ordinary nongeneric top-level reference class derived from
an already attached class in the same output. Both owners must be ordinary reference
classes, without intrinsic representation, generic parameters or nesting. Null bases
throw `ArgumentNullException`; unsupported/foreign bases and invalid declarations
throw `ArgumentException` before attachment. Representation changes that invalidate
an existing base relationship reject at write time.

Manual `TypeDefinition` construction with `parent.ToReference()` followed by
`module.Types.Add(definition)` uses the same validation. Bases are immutable and must
already be attached, so authoring cannot create inheritance cycles. CLI output uses
TypeDef.Extends; native output uses the existing `base` Named relationship. No new
format version is introduced. External/constructed base authoring remains unsupported.

`IILGenerator.Call(MethodBuilder)` and `Emit(OpCode.Call, MethodBuilder)` now accept
a direct base constructor from a derived constructor. Load argument zero, then the
constructor arguments, before calling. A normal return requires exactly one base
initialization on every path. Receiver escape, field access before base initialization,
repeated initialization and incompatible initialization at branch joins reject with
`InvalidDataException` during writing. Constructors of unrelated types and calls from
ordinary methods reject at emission with `ArgumentException`. Allocation continues to
use `NewObject`/`Newobj`. Native root construction remains implicit; CLI root constructors
still receive Object::.ctor, while derived constructors receive only their explicit call.

An initialized derived receiver can be used for inherited field and method access.
C# tests execute both manual and builder definitions on .NET, round-trip native PE
metadata and reject invalid initialization. `--class-base-runtime <runtime> <fresh-dir>`
executes a generated native PE that returns 42. This does not add virtual methods,
protected visibility, closed-hierarchy declarations, or external class import.

### Runtime protected constructors and closed class families (2026-10-04)

Runtime `metadata::Visibility::Protected` serializes as `protected` in the native
method row. It is admitted only for instance `.ctor` methods with a declaring type.
The verifier and interpreter allow calls from that type or its descendants, using
resolved definition identities. Unrelated callers and ordinary protected methods,
fields, module functions or types reject. The C# authoring API now exposes `MethodVisibility.Protected` for both
`TypeBuilder.AddConstructor` overloads. Manual `MethodDefinition` declarations accept
CLI access bits `Family` (4) only on instance `.ctor` declarations. Both paths share
signature/owner checks; unsupported protected members throw `ArgumentException` before
attachment. Writer validation rejects unrelated-family constructor operands with
`InvalidDataException` before producing an image. Existing numeric enum values are
unchanged; Protected is appended.

CLI output uses ordinary MethodAttributes.Family; native output records `protected`
with origin access `Family`. Native reader materialization and `MethodInfo.Accessibility`
preserve `MetadataAccessibility.Family`, including `GetConstructors()` views. There is
no runtime reflection or importer dependency involved in authoring. External class
base authoring and general protected methods/fields remain unsupported.

Native record type rows with `is_closed_hierarchy: true` must describe an abstract,
nonsealed reference class. Direct children must share the root's defining assembly
and revision. Descendants through an open local child may be external. The existing
false default is unchanged; source permits lists remain Raven validation. These
checks do not add closed-interface or general sealed-leaf enforcement. See the
[runtime access contract](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/accessibility.md) and
[executable scope](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/class-hierarchy-foundation-2026-10-04.md).

### Native closed-class authoring and facade (2026-10-04)

```csharp
TypeBuilder AssemblyBuilder.AddClosedClass(string @namespace, string name,
    TypeVisibility visibility = TypeVisibility.Public);
TypeBuilder AssemblyBuilder.AddClosedClass(string @namespace, string name,
    TypeBuilder baseType, TypeVisibility visibility = TypeVisibility.Public);
TypeDefinition(string @namespace, string name, uint attributes,
    TypeReference? baseType, bool isClosedHierarchy);
bool TypeDefinition.IsClosedHierarchy { get; }
bool TypeBuilder.IsClosedHierarchy { get; }
bool TypeBuilder.IsAbstract { get; }
bool NominalTypeInfo.IsClosedHierarchy { get; }
IReadOnlyList<NominalTypeInfo> NominalTypeInfo.GetPermittedDirectSubtypes();
void AssemblyBuilder.DeclareClassBase(ImportedTypeReference type,
    ImportedTypeReference baseType);
```

The original four-argument TypeDefinition constructor is preserved. Closed declarations
require Abstract without Sealed and a nongeneric top-level reference class; invalid
manual definitions throw ArgumentException on attachment before modifying the module.
The overload without a base creates a root using the explicitly selected core Object
reference. The base-taking overload (development, 2026-10-07) uses an already attached
ordinary nongeneric reference class, including the native Object root. It shares
manual-definition validation: null throws ArgumentNullException; foreign, generic,
interface, static and value bases throw ArgumentException. Invalid visibility/name
or duplicate types also reject. Constructors must initialize their direct base;
missing initialization or a sealed base rejects during writer validation. Native
reader and introspection retain the canonical base independently of family closure.
Its constructors may be Protected. Ordinary local children use AddClass(baseType);
abstract allocation fails writer validation with InvalidDataException.

Native closed-family flags survive PE/native reader materialization. Facade children
are canonical direct BaseType matches in defining-module metadata order; an ordinary
native class returns an empty list. CLI snapshots throw NotSupportedException for
closure queries because CLI closed-family attributes are not materialized.

Ordinary executable Write rejects native closed-family declarations. Native PE
transport retains Abstract in its nonexecutable CLI projection and closure in the
authoritative native payload; there is currently no projected closed-family attribute.
Generic/nested closed roots and class virtual/abstract methods remain unsupported.

DeclareClassBase records host-provided signature conversion facts for two output-owned
nongeneric top-level class references in the same exact dependency assembly. It emits
no new dependency declaration and opens no metadata reader. Repeating the same edge is
idempotent; foreign/value/interface/generic/nested/cross-assembly, cyclic or conflicting
edges throw ArgumentException. Null inputs throw ArgumentNullException; over 4096 edges
throw InvalidDataException. Runtime verification still checks the actual dependency
definitions; hosts must supply truthful symbol facts. It does not enable authoring a
new class that derives from a dependency.

See [the executable compiler gate](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/closed-family-2026-10-04.md).

## Runtime type handles (development, 2026-10-04)

`PrimitiveType.RuntimeTypeHandle` is an opaque runtime type identity in signatures,
not a user-authored empty struct. CLI encoding is a value-type reference to
`System.RuntimeTypeHandle` in the explicitly selected core assembly; native encoding
uses the existing `RuntimeTypeHandle` category. CLI imports recognize that exact core
AssemblyRef identity or a value-type definition inside that exact selected core.
The library does not load dependencies or expose host Type objects.

`IILGenerator.LoadTypeToken(SignatureType type)` and
`Emit(OpCode.Ldtoken, SignatureType type)` push one RuntimeTypeHandle. They accept
owned nominal types, constructed nominal types, primitives, vectors and in-scope
method/owner generic parameters. Encoding uses CLI `ldtoken` or native `ldtoken`.
Null throws ArgumentNullException; Void, direct function signatures, byrefs, Self,
foreign owners and out-of-scope parameters throw ArgumentException. Token authoring
uses the same generator for builders and attached definitions. It does not construct
runtime introspection descriptors or promise default-handle/comparison operations.

C# tests check CLI local, constructed, method-generic and owner-generic identity,
CLI signature import, native signature round trips and rejected operands. The generated
native PE executes local, constructed, vector and instantiated method-parameter tokens,
verifies successfully and returns 42. External dependency-token identity and runtime
handle-service comparisons remain integration gates.


## Runtime internal calls (development, 2026-10-05)

`MethodDefinition.SetInternalCall() -> void` marks an authored, nongeneric assembly
function with implementation attributes `0x1000`. `MethodBuilder.SetInternalCall()`
is a convenience method over the same canonical definition. Both require an empty
body, no locals or labels and no type owner. Loaded snapshots, generic declarations,
type methods and nonempty declarations throw InvalidOperationException. Setting the
same valid declaration again is idempotent. There is no implicit P/Invoke library or
entry-point lookup.

`MethodDefinition.ImplementationAttributes` returns the flag for authored and loaded
CLI/native definitions. CLI writing emits MethodDef.ImplFlags and no method body;
this does not make an arbitrary InternalCall executable by the desktop CLR. Native
writing uses the existing `impl_flags` field and the exact namespace-qualified service
name, rather than an ordinary generated function name. Existing functions keep their
encoding. Readers and native reference projections preserve the flag, and imported
assembly-function references preserve the runtime service name.

The declaration's supported signature may reference types defined in the same output
assembly. That enables a service contract to share ownership with source descriptors;
it does not implement their runtime factory. The runtime still checks the exact binding
name/signature. A structurally valid declaration with an unknown service writes, then
runtime verify/run rejects it. There is no implicit dependency loading or service fallback.

Writing rejects instructions, locals or labels added after marking, and rejects an
internal-call entry point with InvalidDataException. Native reading rejects unsupported
implementation bits or an internal-call declaration with a body, locals, owner, generic
parameters or incompatible abstract/virtual/instance flags. Type-owned and generic
internal-call authoring remain outside this bounded profile.

```csharp
var query = assembly.AddFunction("neoCLR.Runtime", "TypeArgumentCount",
    new MethodSignature(PrimitiveType.Int32, [PrimitiveType.RuntimeTypeHandle]));
query.SetInternalCall();
// Emit calls from ordinary method bodies through their separate IILGenerator.
```

C# tests cover definitions/builders, CLI/native/reference-projection flags, output-owned
nominal result signatures and invalid graphs. Generated native PE executes with an
explicit empty test seed, both directly and through a separate native metadata consumer.
Unknown runtime services reject. See the [integration record](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/internal-call-authoring-2026-10-05.md).


Runtime execution checkpoint (2026-10-05): native internal-call declarations can now
return an output-owned System.Introspection.ModuleInfo interface through TypeModule.
The runtime validates and materializes its source-owned provider; see the
[scope, layout and executable contract](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/source-module-descriptors-2026-10-05.md).
No new C# authoring member is required. This bounded runtime capability does not imply
that arbitrary descriptor-returning services or the full production descriptor library
are supported.

## Closed interface families (development, 2026-10-05)

`AssemblyBuilder.AddClosedInterface(string @namespace, string name, TypeVisibility visibility = TypeVisibility.Public)`
returns an attached nongeneric top-level interface. Manual definitions use
`new TypeDefinition(ns, name, 0xa1, null, isClosedHierarchy: true)` and the same validation.
Invalid names, visibility, duplicates or unsupported categories throw ArgumentException.
Native round trips preserve IsClosedHierarchy independently of CLI Sealed; CLI reference
flags remain Interface/Abstract. Executable CLI writing rejects the native extension.
`NominalTypeInfo.GetPermittedDirectSubtypes()` includes directly declared interface
implementations and derived interfaces, excluding indirect descendants. Runtime linking
rejects direct children outside the defining module/revision. Open local branches remain
extensible. Generic closed interfaces are not yet authored. Existing local relationship
encoding and closed-family flag are reused without a format revision.

## Reference Object overrides (development, 2026-10-05)

`TypeBuilder.AddOverride(name, signature)` and manual `MethodDefinition(name, 0x46, signature)`
admit nongeneric reference owners alongside value owners. Supported exact slots:
`String ToString()`, `Int32 GetHashCode()`, `Boolean Equals(assembly.CoreObjectType)`.
The flags are Public/Virtual without NewSlot or Abstract. Generic reference owners and
generic override methods remain unsupported. Static/interface owners, wrong core Object
identity, or incompatible signatures reject on attachment; duplicate members reject.

`IILGenerator.CallVirtual(MethodBuilder)` and `Emit(OpCode.Callvirt, MethodBuilder)` also
admit these owned reference overrides. Imported references retain RequiresVirtualDispatch.
`CreateMethodReference(..., isOverride: true)` supports the same slots and exact core
identity. Value overrides retain managed receivers and direct calls.

Native writes require an explicit System binding with the actual matching public virtual
Object slot, and native readers preserve flags. Runtime linking validates the contract;
reference dispatch traverses the actual class lineage, including an implicit Object view
for rootless native classes. No Object storage fields are introduced. The CLI writer
uses ELEMENT_TYPE_OBJECT for the configured core Object signature, including nested
arrays/parameters, rather than a nominal CLASS encoding that fails CLR override matching.
C# tests execute both manual and builder definitions on CLR; native compiler consumers
exercise separate-assembly class, Object-view and inherited dispatch.

## Flags enums (development, 2026-10-05)

`TypeDefinition.IsFlagsEnum`, `TypeBuilder.IsFlagsEnum` and
`Introspection.NominalTypeInfo.IsFlagsEnum` report combinable enum values. CLI snapshots
recognize the exact core FlagsAttribute without loading dependencies; native snapshots
read the existing enum-info flag. Ordinary enums/non-enums report false. Native raw
custom-attribute collections do not synthesize a FlagsAttribute: use IsFlagsEnum for the
portable declaration fact. Reference projections reconstruct the ordinary core marker.

`TypeDefinition.SetEnumFlags()` and `TypeBuilder.SetEnumFlags()` mark attached authored
enums through a standard core FlagsAttribute. Calls are idempotent. Loaded, detached or
non-enum definitions reject with InvalidOperationException. A manually added
`CustomAttributeDefinition(module.ImportReference(core, "System", "FlagsAttribute"), [])`
uses the same writer path. Duplicate/malformed core markers reject with InvalidDataException.
Another assembly's similarly named attribute does not set the flags classification.
Native writing consumes the standard marker into existing enum-info metadata, without
an executable attribute-constructor dependency. No native format revision is required.
Supported storage remains Int32; flag names and values retain the normal enum contracts.


## Parameter arrays (development, 2026-10-05)

`MethodDefinition.SetParameterArray(int position)` and `MethodBuilder.SetParameterArray(int position)`
mark the final by-value vector parameter. Definitions must be attached and authored;
loaded/detached methods throw InvalidOperationException, and invalid positions or physical
signatures throw ArgumentException. Repeating the same marker is harmless.
`MethodDefinition.ParameterArrayIndex` returns its zero-based position or null.
`Introspection.ParameterInfo.IsParameterArray` exposes the same fact for CLI/native snapshots.
The physical signature and calling convention remain ordinary array parameters.

CLI writing uses the standard core ParamArrayAttribute on a Param row. Native writing uses
existing custom_attributes with the parameter's metadata target_token, retaining exact
native System marker identity through an explicit bootstrap binding. No format extension
or implicit dependency loading is introduced. Missing/wrong marker declarations, malformed
marker signatures/values and invalid parameter targets reject before publication.
The bootstrap marker constructor is executable but is not invoked during metadata import.
This marker is recognized by its qualified standard name in CLI metadata; native dependency
resolution additionally checks the explicit System alias. Other method attribute categories
remain explicitly unsupported in this reader profile.

## Inherited interface implementations (development, 2026-10-05)

`TypeBuilder.AddInterfaceImplementation` and manually authored `TypeDefinition.Interfaces`
now accept exact public implementations inherited from a local nongeneric base class.
Search selects the nearest matching declaration, after an explicitly mapped local method.
Only methods actually selected for interface implementation receive CLI virtual/final/newslot
flags; unrelated methods retain their existing attributes. Existing exact parameter/return
matching and visibility validation apply. Missing implementations reject before writing.
Inherited explicit reimplementation is rejected rather than authoring an invalid MethodImpl
whose body is outside the declaring type. External class inheritance remains unsupported.

## Final classes and Boolean storage (development, 2026-10-05)

`TypeDefinition.SetSealedClass()` and `TypeBuilder.SetSealedClass()` set the ordinary
CLI Sealed flag on an attached concrete reference class. They are idempotent and return
void. Loaded, detached, static, abstract, value and closed-family definitions reject
with InvalidOperationException. Manual `TypeDefinition` attributes may use the same
`0x100` flag; the writer validates both paths. CLI and native readers preserve the
flag, and `Introspection.NominalTypeInfo.IsSealed` reports it. Deriving from a sealed
local class rejects before writing. This is finality, distinct from a closed hierarchy.

`SetNativePrimitive(PrimitiveType.Boolean)` now supports the canonical fieldless
`System.Boolean` value declaration. It follows numeric primitive validation and
ldobj/stobj scalar storage; native import and introspection preserve NativePrimitive.
A name alone does not claim ownership. Executable CLI writing of runtime-owned
primitive implementations still rejects; explicit host ownership selects the provider.

## Authored type-row budget (development, 2026-10-05)

AssemblyBuilder type authoring and manual ModuleDefinition.Types attachment now admit
4,095 declarations instead of 256. This shares the existing CLI reader budget of 4,096
TypeDef rows, reserving row one for `<Module>`. Native rows omit that synthetic row and
admit the same 4,095 declarations. Nested and top-level declarations share the budget.
Authoring beyond it raises ArgumentException; native reading beyond it raises
InvalidDataException before materialization. Failed attachment leaves the collection
unchanged. No signature, token representation or metadata version changes.

All byte, field, method, property and dependency limits still apply. Large libraries
may need RuntimeAssemblyContainer.WriteLibraryBinary rather than the smaller application
profile. The limit is a bounded host-library policy, not a CLI format maximum. Older
host library versions retain their lower native/authoring cap and reject larger inputs.

### Native enum Object behavior (development, 2026-10-05)

Int32-backed native enums now participate in inherited Object.ToString, Equals and
GetHashCode calls through ordinary boxing. ToString uses the existing enum metadata
formatter (named values, flags composition and numeric fallback); equality also requires
the same nominal enum type, and hashing uses the Int32 payload. This adds no metadata
category or guest member signature. The existing enum alias choice remains unspecified
for .NET parity; the current formatter prefers the first declared exact alias.

### Managed entry arguments (development, 2026-10-05)

AssemblyBuilder.EntryPoint and authored AssemblyDefinition.EntryPoint admit an optional
single `SignatureType.ArrayOf(PrimitiveType.String)` parameter. Writer and native reader
validate the same contract; CLI output retains the normal String[] signature and entry
MethodDef token. The native runtime supplies a bounded managed String array at startup.
No additional instruction, metadata version, runtime service or compiler-generated
wrapper is introduced. Current native name-based entry selection rejects ambiguity
between parameterless and String[] overloads.

Rust `ExecutionOptions.arguments` and Environment retain argv[0]; String[] entry points
receive the remaining elements. Hosts should include the program name/path first.
Empty options produce an empty array. CLI `--` arguments retain their order and Unicode
text. Allocation respects array and heap limits and can fail before the entry body.
Direct host method invocation continues to use the explicitly supplied arguments.
C# EntryArgumentChecks covers CLI invocation and native readback; runtime tests cover
empty/nonempty vectors, limits, unsupported shapes and ambiguity. The emitted C# test
image executes under neoCLR with two arguments, and the Raven upload sample consumes
its URL/mode parameters unchanged.

### Source-owned runtime handles (development, 2026-10-05)

`TypeDefinition.SetNativePrimitive(PrimitiveType.RuntimeTypeHandle)` and the matching
builder method now accept the canonical `System.RuntimeTypeHandle` declaration: sealed,
sequential, nongeneric, top-level value type without fields or constructors. The runtime
owns handle storage; do not author an IntPtr payload or an m_value field. Wrong names,
reference categories, storage or constructors reject with ArgumentException. Writers
revalidate edits. Native reader and `NominalTypeInfo.NativePrimitive` preserve the
RuntimeTypeHandle designation; loaded definitions remain immutable.

This extends the existing primitive contract rather than adding a new signature category.
Signatures still use PrimitiveType.RuntimeTypeHandle, encoded as the ordinary CLI core
handle reference and the existing native RuntimeTypeHandle category. Executable CLI
output of native primitive implementations remains unsupported. Dependency catalogs must
select one owner; a source handle cannot compete with a retained seed handle. C# tests
cover definition/builder authoring, native round trip, introspection and invalid storage.

### Internal-call declarations in separate assemblies (development, 2026-10-05)

Native loading now admits matching runtime-service InternalCall declarations in distinct
assemblies. Each retains its definition identity. Symbolic calls with a local service
declaration bind locally; explicit external identities remain exact, including access
checks. Duplicate declarations within one module, incompatible service signatures and
ambiguous unqualified calls without a local declaration still reject. Registry validation
is unchanged: a matching name alone never grants a native service. See the
[Object service integration gate](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/object-services-2026-10-05.md).

## Native Object root authoring (development, 2026-10-05)

`AssemblyBuilder.AddNativeObjectRoot() : TypeBuilder` attaches a public abstract,
nongeneric, baseless `System.Object`. For manual authoring, construct
`new TypeDefinition("System", "Object", 0x81, null)`, call
`TypeDefinition.SetNativeObjectRoot() : void`, then add it to the module's `Types`.
`TypeBuilder.SetNativeObjectRoot() : void` uses the same validation.
`TypeDefinition.IsNativeObjectRoot : bool` and `TypeBuilder.IsNativeObjectRoot : bool`
report the authoring designation. Loaded snapshots return false: this flag is not
runtime selection. Readers preserve the declaration's name, abstract flags and base.

Designation rejects incompatible names, visibility, generic/nested/value/static/closed
shapes and record fields with `ArgumentException`; loaded definitions reject mutation
with `InvalidOperationException`. Attachment rejects duplicates. Writers revalidate
shape and reject later invalid mutations with `InvalidDataException`.

Native output uses canonical `System.Object` in the existing format, with no base and
no new representation category. Its reference-only CLI projection uses a nil `Extends`
and `ELEMENT_TYPE_OBJECT` for signatures referring to the authored root. Executable
CLI `Write()` rejects the native designation. Unlike ordinary .NET Object, the current
NeoCLR source root is abstract; this API preserves that existing platform contract.
The benefit is explicit root identity without a competing bootstrap base; the cost is
that this artifact needs NeoCLR host admission and cannot be executed by the CLR.

This is declaration support, not complete core-library production: callers still need
boxing/root reference selection and compiler/driver wiring. Root-slot authoring is
provided below.
`CoreObjectType` continues to mean the explicit bootstrap reference. No implicit bases
or bootstrap references are retargeted by designation. The runtime host must separately
select the exact root identity and validate its required slots. Ordinary names do not
claim root ownership. See `ObjectRootChecks.cs` for builder/definition parity, native
round trips, CLI signatures, mutation rejection and ordinary lookalike controls.

### Native Object virtual slots

`TypeBuilder.AddNativeObjectSlot(string name, MethodSignature signature) : MethodBuilder`
adds a concrete public virtual **new slot** to the designated root. Supported signatures
are `String ToString()`, `Int32 GetHashCode()` and `Boolean Equals(root)`, where the last
parameter is the exact authored root, not `CoreObjectType` or another output's root.
Use `GetILGenerator()` on the returned builder for the body. Manual definitions use
`new MethodDefinition(name, 0x146, signature)` followed by attachment to root.Methods.
Signature/name errors throw `ArgumentException`; an incompatible owner or foreign root
throws `InvalidOperationException` during attachment. Duplicate signatures are rejected.

Readers, introspection and CLI reference projections retain Virtual and NewSlot without
Abstract or override semantics. Native encoding uses existing `is_virtual`/`is_override`
fields; no format revision. `CallVirtual`/raw `Callvirt` accept owned root slots.
An ordinary class or value type cannot claim these new root slots. The existing Object
override API continues to reuse inherited slots. As on CLR, declaring a virtual slot is
distinct from overriding one; NeoCLR's explicit root admission remains host configuration.

Validation: `ObjectSlotChecks` covers manual/builder parity, native and CLI round trips,
introspection flags and exact Equals identity, malformed declarations and wrong owners.
The generated fixture is loaded and executed by Rust `object_root_identity`; production
source Object/compiler integration is still pending.

### Selected Object signatures and boxing

`AssemblyBuilder.ObjectType : SignatureType` returns the explicitly authored native
root when present, otherwise the explicitly selected external root or existing bootstrap Object signature.
`CoreObjectType : ImportedTypeReference` keeps its original bootstrap meaning.
Declare the root before creating signatures; existing signatures are not rewritten.
No dependency is loaded and this property does not select runtime host ownership.

`Box` and value-type `Isinst` stack results use `ObjectType`. Native emission with an
authored root validates all three concrete root slots instead of demanding a separate
legacy System binding. An incomplete root or a body expecting the unrelated bootstrap
Object fails with `InvalidDataException` before bytes are returned. With an external root, author all three exact slot references through
`CreateObjectSlotReference` before native writing: ToString, Equals and GetHashCode.
Missing slots reject with `InvalidDataException`; a selected name alone is insufficient.
The host supplies the contract and the runtime checks the actual linked definitions.
Without either selected root, the existing explicit System binding requirement is unchanged.

The API-produced fixture's `BoxedDisplay()` now emits boxing and virtual dispatch and
executes with result `"42"` under explicit runtime root selection. This tests executable
instructions from the API, in addition to the existing source-assembled runtime callers.

### Overrides of an authored Object root

`TypeBuilder.AddOverride(name, signature)` and manually attached
`MethodDefinition(name, 0x46, signature)` now support the selected local Object root.
Equals must take the exact `AssemblyBuilder.ObjectType`; a bootstrap or foreign root
argument throws `InvalidOperationException` on attachment. The root itself cannot
claim an override of its own slot. Signature selection is revalidated on write, so
adding a root after authoring bootstrap-based Equals fails with `InvalidDataException`.

Native emission requires a complete local root slot contract, or the existing explicit
legacy System binding when no root is authored. Overrides preserve CLI Virtual without
NewSlot or Abstract. Native readers and introspection retain the Equals parameter's
canonical local identity. No new native format fields or runtime dispatch rules are used.

The supported direct builder-to-PE path executes constructors and all three overrides
through root slots. Regenerating a CLI projection from a native snapshot with class
inheritance remains explicitly unsupported (`NotSupportedException`); use the original
authored graph for PE emission. The normal .NET override execution controls still pass.
Raven's source-root planner, virtual declarations and special-type mapping remain pending.


### Ordinary abstract classes (development, 2026-10-05)

`TypeDefinition.SetAbstractClass()` and `TypeBuilder.SetAbstractClass()` mark an
attached, nongeneric, top-level ordinary reference class with CLI `Abstract` (0x80).
The builder delegates to its canonical definition. Repeated calls are idempotent.
They return void and throw `InvalidOperationException` for detached or loaded
snapshots, value/interface/static/sealed classes, nested or generic owners, native
Object roots and closed families. Rejected calls leave attributes unchanged.

Manual authoring accepts `new TypeDefinition(ns, name, 0x81, baseReference)` through
the same attachment validation. The base must be the explicitly selected core Object
or an attached supported local class. Public/internal visibility is supported.
Concrete methods, fields and public/protected constructors retain existing contracts.
Direct construction of an abstract owner fails writer validation with
`InvalidDataException`; a concrete subclass can call its base constructor normally.

Native reads preserve Abstract without inventing Sealed, static classification or
closed-family semantics. Introspection exposes `IsAbstract == true`, `IsStatic ==
false`, `IsSealed == false`, and `IsClosedHierarchy == false`. Static classes continue
to require both Abstract and Sealed. The existing payload fields and CLI flags suffice;
there is no format-version change. General virtual/abstract method authoring remains
unsupported pending the next inheritance slice. No guest class-library API changed.


### Local class virtual slots (development, 2026-10-05)

`TypeBuilder.AddVirtualMethod(string name, MethodSignature signature)` introduces a
public, nongeneric instance slot on a nongeneric reference class. It returns the
canonical `MethodBuilder`; emit its body through `GetILGenerator()`.
`AddAbstractMethod(string name, MethodSignature signature)` introduces the same slot
without a body and requires an abstract owner. Manual definitions use CLI attributes
`0x146` (Public | Virtual | NewSlot) and `0x546` (also Abstract), respectively.
Invalid names/signatures or duplicates throw `ArgumentException`; incompatible owners
throw `InvalidOperationException`. Abstract methods reject bodies/locals on write.

`AddOverride` and manual `MethodDefinition(name, 0x46, signature)` also support exact
local inherited class slots. Writer validation requires matching return/parameter
signatures and parameter modes; concrete classes must implement inherited abstract
members. Incompatible overrides, hiding an inherited member, or missing implementations
throw `InvalidDataException` before output. General new-slot hiding, generic class
slots, re-abstraction, nonpublic virtual slots and external class overrides remain
unsupported. Existing value/Object overrides retain their root identity checks.

`IILGenerator.Emit(OpCode.Callvirt, method)` / `CallVirtual(method)` dispatch these
reference-class slots. `Call(method)` invokes a concrete base body directly; direct
abstract calls are rejected. Reader/introspection flags preserve Virtual, Abstract and
NewSlot. Interface conformance includes the local base chain. Explicit virtual slots
implementing interfaces are not marked Final. Runtime resolution uses validated
metadata member origins to bridge encoded interface names and native slot spellings,
while retaining exact owner/signature matching. Existing CLI flags and native payload
fields suffice; no version change or implicit dependency loading is introduced.


### Raven host catalog boundary (development, 2026-10-05)

Raven's separate compiler adapter now provides `NeoClrReferenceCatalog` to read an
explicit primitive core, native PE references and optional retained runtime seed.
This is a compiler-host API, not a member of the metadata library or guest System API.
It shares immutable reference instances between semantic import and emission and owns
no runtime reflection, implicit dependency search or editor file watcher. The native
CLI uses it; the subsequent native project/editor slices below also consume it. See the
[Raven host API contract](https://github.com/marinasundstrom/raven/blob/codex/metadata-consumer/docs/compiler/neoclr-cli-bridge.md#shared-native-reference-catalog-2026-10-05)
for signatures, input limits, failures and snapshot lifetime. No guest API snapshot changes.


### Raven native project provider (development, 2026-10-05)

Raven's optional `NeoClrProjectMetadataProvider` adapts the native catalog to evaluated
projects through the target-neutral `IProjectMetadataProvider` and immutable
`ProjectMetadataConfiguration` host contracts. These are .NET compiler-host APIs,
not guest System.Introspection APIs, so the RavenDoc guest snapshot is unchanged.
Their selection, parameters/results, errors and limits are documented in the
[public host API contract](https://github.com/marinasundstrom/raven/blob/codex/metadata-consumer/docs/compiler/neoclr-cli-bridge.md#explicit-native-project-metadata-2026-10-05).
The metadata reader/writer and importer/emitter boundaries remain independent.


### Native project emission and editor lifetime (development, 2026-10-05)

The host provider's `GetConfiguration(projectFilePath)` returns a successfully loaded
`NeoClrProjectConfiguration` with `Catalog`, `ReferencePaths`, `RuntimeSeedPath`,
`Validate(compilation)` and `CreateEmissionBackend(assemblyName)`. Target-neutral
project/provider interfaces expose explicit metadata input paths for host file watching.
Ownership and async contracts are evaluated once per load; emission uses host artifact
identities and compiler symbols. Invalid reloads retain the last successful editor
snapshot and report failure; builds independently revalidate before atomic publication.

See the [complete host API contract](https://github.com/marinasundstrom/raven/blob/codex/metadata-consumer/docs/compiler/neoclr-cli-bridge.md#native-vs-code-poc-acceptance-2026-10-05)
for signatures, exceptions, limits and commands. These are compiler-host APIs, not guest
System APIs; no RavenDoc guest snapshot or native metadata encoding changes are needed.

### Native compiler-host documentation sidecars (development, 2026-10-05)

File-backed native catalog references expose existing Raven XML/Markdown sidecars
through symbol documentation, without changing native metadata or guest APIs.
Markdown member help takes precedence over XML; absent/malformed optional sidecars
do not invalidate an assembly. Image-only references have no implicit sidecar search.
See [host options, output rules and editor lifetime](https://github.com/marinasundstrom/raven/blob/codex/metadata-consumer/docs/compiler/neoclr-cli-bridge.md#native-ide-documentation-2026-10-05).
The guest RavenDoc snapshot is unchanged. Website guides are separate content.


### Explicit callable nullable annotations (development, 2026-10-06)

`Model.NullableAnnotation(IEnumerable<byte> flags, bool isUniform = false)` copies
1–4096 flags into immutable `Flags`. Values are 0 (oblivious), 1 (non-null) and
2 (nullable), in .NET nullable transform order. `IsUniform` distinguishes the
scalar-byte `NullableAttribute` constructor, which repeats its flag, from a
positional byte-array payload. Uniform annotations require exactly one flag.
Null input throws `ArgumentNullException`; invalid values or lengths throw
`ArgumentException`.

`MethodDefinition.SetNullableAnnotation(int position, NullableAnnotation? annotation)`
and the matching `MethodBuilder` convenience method share one implementation.
Position -1 identifies the return; nonnegative positions identify parameters.
Passing null removes the annotation. Invalid positions throw
`ArgumentOutOfRangeException`; detached or loaded definitions cannot be mutated
and throw `InvalidOperationException`. `NullableAnnotations` exposes the explicit
annotations as a read-only dictionary. Attach manually created definitions before
setting annotations, just as for parameter metadata.

```csharp
method.GetILGenerator().LoadArgument(0);
method.GetILGenerator().Return();
method.SetNullableAnnotation(0, new NullableAnnotation([1, 2]));
method.Definition.SetNullableAnnotation(-1, new NullableAnnotation([1, 2]));
// For a string[] signature: a non-null array with nullable string elements.
```

CLI writing emits ordinary parameter/return custom attributes referencing
`System.Runtime.CompilerServices.NullableAttribute` in the configured core assembly.
That core must supply the appropriate byte/byte[] constructors. The CLI reader
preserves these explicit payloads; malformed, duplicate or oversized payloads
reject. `Introspection.MethodInfo.ReturnNullableAnnotation` and
`ParameterInfo.NullableAnnotation` expose them without changing physical type views.
Absent metadata is not a non-null assertion.

This is raw explicit annotation preservation, not a completed nullable type system.
`NullableContextAttribute`, field/property annotations and general signature-shape
validation remain pending. Raven's native adapter now reconstructs supported callable
nullable reference, array and generic positions from the facade. Native writing preserves these facts in
`origin.nullable_annotations`; native readers and the compatibility CLI projection
retain both scalar and vector forms. Runtime metadata validation checks position,
uniqueness and payload bounds without executing an attribute constructor. Loaded-assembly rewriting remains subject to the library's existing
limits. There is no new runtime check, layout change or GC policy.

Validation: `dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests
-- --nullable-annotations` checks authored-definition/builder parity, defensive copying,
CLI round trips, facade exposure, malformed input and .NET `NullabilityInfoContext`
interpretation. The generated identity methods execute without changing object identity.
See the [design and integration boundary](https://github.com/marinasundstrom/neoCLR/blob/d622395e765b20d2f257e8c8a415db7bdb6743cd/docs/design/callable-nullability.md).

## Native-width integers (development, 2026-10-06)

`PrimitiveType.IntPtr` and `PrimitiveType.UIntPtr` represent signed and unsigned
native-width integers in method, field, property and local signatures. Use them in
`MethodSignature`, `SignatureType` and definition/builder APIs just like the fixed-width
integer categories. CLI writers emit standard ELEMENT_TYPE_I (0x18) and ELEMENT_TYPE_U
(0x19); readers and introspection preserve the categories. Native metadata uses the
existing runtime IntPtr/UIntPtr types. These are integer values, not unmanaged pointer
signatures or new inline-array representations.

`method.GetILGenerator().Emit(OpCode.Conv_I)` and `Emit(OpCode.Conv_U)` emit `conv.i`
and `conv.u`. Existing numeric conversions accept native-width integer inputs.
The writer rejects nonnumeric operands before publishing bytes. Target width is
chosen by the executing runtime; no fixed 64-bit signature is substituted. This support does not add unchecked pointer access or native integer arithmetic
validation. The source-provider follow-up also admits SetNativePrimitive(IntPtr/UIntPtr)
for their exact canonical System value declarations, with the same no-record-storage
validation as other primitive providers.

```csharp
var identity = owner.AddMethod("Identity", new(PrimitiveType.IntPtr, [PrimitiveType.IntPtr]));
var il = identity.GetILGenerator();
il.LoadArgument(0);
il.Emit(OpCode.Conv_I8);
il.Emit(OpCode.Conv_I);
il.Return();
```

C# tests compile and execute signed/unsigned conversions, locals and signatures on
.NET; the same native artifact verifies and exits 42 under neoCLR. Definition-authored
fields, builder-authored methods/properties and introspection retain both categories.
No metadata format version changes; older host readers may reject these newly admitted
signatures even though the runtime already implements their native categories.

### Source-owned erased Value storage (development, 2026-10-07)

`PrimitiveType.Value` designates the existing native erased carrier with
`TypeDefinition.SetNativePrimitive` or `TypeBuilder.SetNativePrimitive`. Create an
owned public System.Value value definition, then mark it; generic/nested definitions,
record fields and constructors reject as for RuntimeTypeHandle. Use that definition
or builder as a signature operand. Converting the bare enum to SignatureType throws
ArgumentOutOfRangeException because erased signatures require an explicit owner.

```csharp
var value = assembly.AddValueType("System", "Value");
value.SetNativePrimitive(PrimitiveType.Value);
var identity = assembly.AddFunction("Identity", new MethodSignature(value, new SignatureType[] { value }));
identity.GetILGenerator().LoadArgument(0);
identity.GetILGenerator().Return();
```

Native signatures encode Value and the declaration uses existing Runtime storage;
CLI signatures retain the owned value-type token. Executable CLI emission rejects
native primitive designations. Native reading preserves NativePrimitive and resolves
local Value signatures to the canonical local definition. The exact configured
core/System bootstrap alias can share evaluation storage with this selected carrier;
arbitrary foreign or unmarked local types do not acquire erased storage. No format
version change or runtime instruction was added. See the
[source bootstrap gate and remaining limits](https://github.com/marinasundstrom/neoCLR/blob/codex/native-system-bootstrap/docs/experiments/extended-cli-metadata/source-value-2026-10-07.md).

### Intrinsic String over an explicit Object root (development, 2026-10-07)

`AddClass("System", "String", objectRoot)` followed by
`SetNativePrimitive(PrimitiveType.String)` retains its canonical Object base when
read through `AssemblyDefinition.ReadNativeAssembly`. Runtime loading requires
explicit selection of that fieldless Object root. String retains intrinsic UTF-8
storage, not record fields; unrelated primitive bases remain unsupported. Use
`GetILGenerator()` for constructor bodies, including the direct base call.
[Runtime evidence](https://github.com/marinasundstrom/neoCLR/blob/d622395e765b20d2f257e8c8a415db7bdb6743cd/docs/experiments/extended-cli-metadata/string-root-2026-10-07.md).

### Expanded library envelope (development, 2026-10-07)

`NativeModuleContainer.WriteLibraryBinary`,
`RuntimeAssemblyContainer.WriteLibraryBinary` and runtime `write_module` select
required schema 4 when the encoded library exceeds schema 3's 8 MiB envelope.
Schema 4 allows a 16 MiB envelope. `Read`, `ReadNativeAssembly` and native runtime
loading accept it; older readers reject the required version. Small libraries keep
schema 3. Schema-1/2 bounds remain unchanged. The native CBOR/semantic model is unchanged.

All other library limits remain: 32 MiB JSON, 2,097,152 nodes, depth 64, and 16 MiB
**total PE** (`MaxLibraryImageSize`). Container/CLI overhead counts toward that PE
limit. Exceeding any applicable bound throws `InvalidDataException`; no partial
compiler artifact is published. No execution or complete bootstrap is implied by a
successful write. [Contracts and executable evidence](https://github.com/marinasundstrom/neoCLR/blob/d622395e765b20d2f257e8c8a415db7bdb6743cd/docs/experiments/extended-cli-metadata/expanded-library-2026-10-07.md).

### Canonical native unit (development, 2026-10-07)

`TypeDefinition.SetNativePrimitive`, `TypeBuilder.SetNativePrimitive` and
`AssemblyBuilder.SetNativePrimitive(ImportedTypeReference, PrimitiveType)` accept
`PrimitiveType.Void` for the explicitly selected canonical `System.Void` declaration.
It represents NeoCLR's inhabited unit, usable in parameters, generic arguments and
function results. There is no additional native Unit type.

Local declarations must be empty, nongeneric, top-level value types without
constructors; wrong identities and storage reject through the existing validation.
The external-reference API records host-supplied ownership without loading a
consumer's dependencies. Native encoding records the scoped canonical unit alias;
loading still requires its actual dependency. Manual definitions and builders share
validation. Existing CLI executable primitive-provider rejection remains unchanged.

A signature using the designated declaration denotes a unit **value**; a bare
`PrimitiveType.Void` return retains the API's CLI-compatible **no-result** convention.
`FunctionSignature.NoResult` preserves that stack distinction across encoding and
introspection. This is a low-level calling convention, not two Raven language types.
Nominal CLI signature transport is retained where CLI void is illegal. The compiler
owns target-specific lowering and supplies RuntimeUnitContract ownership; an arbitrary
empty struct is never inferred to be unit.

### External native Object authoring (development, 2026-10-07)

`AssemblyBuilder.SetNativeObjectRoot(ImportedTypeReference root) : void` selects an
output-owned nongeneric, nonnested reference named System.Object. Construct the reference
with `CreateTypeReference` from the exact dependency identity/core/artifact digest, then
select it before authoring signatures. Use `AssemblyBuilder.ForDefinition(definition)`
for the same selection when manually constructing definitions. `ObjectType` returns
that signature; `CoreObjectType` retains its explicit bootstrap identity.

The method rejects null/foreign/malformed references with argument exceptions and
conflicting local/external selections with InvalidOperationException. Repeating the
same selection is allowed. A caller supplies the host-validated root identity; this
API does not load or inspect the dependency and does not substitute runtime admission.
Unselected same-named identities do not satisfy Equals override validation.

Manual method definitions and builders share the exact Boolean Equals(selected Object)
validation; ToString/GetHashCode retain their existing signatures. Authored/imported
method references apply the same selected identity. Primitive-bootstrap Object type
references and ELEMENT_TYPE_OBJECT signatures map to the selected root during import.
Already-created signatures are not rewritten.

Native writing uses existing scoped module/type aliases for canonical System.Object,
with no new format version. Reading checks the alias and exact Equals owner. The CLI
reference image uses ELEMENT_TYPE_OBJECT; executable CLI output rejects a selected
native root. Bootstrap-only behavior remains unchanged.

[Validation and executable examples](https://github.com/marinasundstrom/neoCLR/blob/d622395e765b20d2f257e8c8a415db7bdb6743cd/docs/experiments/extended-cli-metadata/imported-object-authoring-2026-10-07.md)
cover manual/builder parity, wrong owners, conflicts, native round trips and a Raven
consumer against the source-built Runtime. These are host metadata APIs, documented
here rather than in the guest RavenDoc reference assembly.

### Imported erased Value representation (development, 2026-10-07)

The existing `AssemblyBuilder.SetNativePrimitive(reference, PrimitiveType.Value)`
selection now preserves the external carrier through native writing, reading and
introspection. It writes the Value storage tag with a scoped System.Value alias and
value-type reference, using the existing format. Competing explicit primitive owners
reject. A same-named unselected type does not acquire this representation.

When importing the primitive bootstrap's empty nongeneric System.Value definition,
an explicitly selected external carrier replaces that bootstrap reference. Retained
service signature matching recognizes the same explicit Value designation without
requiring a second seed-owned type. This does not replace System.Object or assign a
.NET SpecialType to the carrier. Existing seed-owned and local-source cases remain
supported; this adds no new public API or native instruction.

[Executable evidence](https://github.com/marinasundstrom/neoCLR/blob/d622395e765b20d2f257e8c8a415db7bdb6743cd/docs/experiments/extended-cli-metadata/imported-value-2026-10-07.md)
covers successful/error payloads through actual runtime services and canonical
introspection parameter/return identity.

### Imported Object slot references (development, 2026-10-07)

`AssemblyBuilder.CreateObjectSlotReference(string name, MethodSignature signature)`
returns an interned `ImportedMethodReference` for the explicitly selected external
Object root. Supported contracts are `String ToString()`, `Int32 GetHashCode()` and
`Boolean Equals(ObjectType)`, without generic or out parameters. Call
`SetNativeObjectRoot(reference)` first. No dependency is opened; the host provides
identity and runtime linking validates the actual root and slot.

The returned contract has `RequiresVirtualDispatch == true`; use
`generator.Emit(OpCode.Callvirt, reference)`. Ordinary Call rejects with ArgumentException.
This is an inherited slot declaration reference, not an override or interface method.
Missing selection throws InvalidOperationException, invalid signatures throw
ArgumentException, and conflicting ordinary/virtual references throw InvalidDataException.
The reference retains its external assembly scope in CLI metadata and canonical native
Object owner/name for runtime linking. No encoding category or version changes.

Body validation accepts reference-class receivers for the explicitly selected external
root; values still require explicit boxing or constrained dispatch. Selection does not
make arbitrary same-named types universal receivers.

[C# and executable Raven validation](https://github.com/marinasundstrom/neoCLR/blob/d622395e765b20d2f257e8c8a415db7bdb6743cd/docs/experiments/extended-cli-metadata/imported-object-slots-2026-10-07.md)
covers slot interning/category, wrong signatures, incompatible contracts, wrong dispatch
opcode and execution through a base-typed receiver. General imported virtual class
methods remain outside this bounded API. Host API documentation lives here; it is not
a new guest class-library API for the RavenDoc assembly snapshot.

### Ordinary selected-root method references (development, 2026-10-07)

`CreateMethodReference` on the explicitly selected external Object root now uses the
same canonical native owner/member names as authored root definitions. This includes
nonvirtual `GetType`; it remains an ordinary instance call. Only
`CreateObjectSlotReference` claims virtual-slot dispatch. Exact artifact identity and
CLI member scopes are retained; no dependency is opened by authoring. A C# regression
checks canonical names and ordinary-call classification, and the API fixture executes
GetType against the actual source-built Runtime. This closes a JSON mapping link failure.

## Native namespace Double constants (development, 2026-10-08)

`NeoCLR.Metadata.Experimental.Model.NamespaceConstantDefinition(string namespace,
string name, double value, MethodVisibility visibility = Public)` describes a
compile-time constant. Read-only `Namespace`, `Name`, `Value`, `Visibility` properties
return its exact declaration; `AssemblyBuilder.AddNamespaceConstant(definition)`
adds it, and `ModuleDefinition.NamespaceConstants` reads authored/native snapshots.
These host APIs are documented manually here, outside the guest RavenDoc selection.

Namespace/name are at most 1024 characters; namespace may be global/empty, otherwise
its dot-separated segments and the simple name must be nonblank and control-free.
Simple names contain no dot. Values must be finite Double, visibility Public or
Internal. Invalid declarations and duplicate namespace/name pairs throw
ArgumentException; null definitions throw ArgumentNullException. At most 4096
constants per assembly. Returned collections are read-only; loaded editing is absent.
Values are inlined by the compiler, with no field address/storage or runtime initializer.

Native JSON/NEOX/PE retains exact 16-digit lowercase binary64 bit strings in
`assemblies[].namespace_constants`, including negative zero. Native readers reject
unsupported types, malformed bits, nonfinite values, unknown members and duplicates.
Older readers reject the extension. Ordinary CLI reads expose no native constants;
standalone CLI writing/projection explicitly rejects a graph containing them.
The nonauthoritative CLI envelope accompanying a native PE does not project these
constants; native readers must use #Neo. No native execution or guest reflection API
is implied by this descriptive metadata. See [Math contracts](../docs/math.md).
