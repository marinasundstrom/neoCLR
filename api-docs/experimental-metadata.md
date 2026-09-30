# Experimental .NET metadata library

**Feature-branch development only; not part of Preview 11 or the neoCLR guest API.**
Assembly and namespace: `NeoCLR.Metadata.Experimental`. Target: .NET 10.
Source project: `tools/metadata/NeoCLR.Metadata.Experimental`.
This is the first reusable reader/writer slice intended for Raven's future symbol
loader and code-generation adapters. It reads/writes **NEOX 0.1 framing and structural signature syntax**.
No PE/CLI loading, artifact recognition, reference resolution,
Introspection assembly loading or assembly emission is implemented by this library yet.

## Namespace and types

- [MetadataSection](#metadatasection): immutable, owned opaque payload and section metadata.
- [MetadataEnvelope](#metadataenvelope): bounded envelope read/write operations.
- [TypeExpression](#typeexpression): immutable raw signature syntax tree.
- [SignatureContext](#signaturecontext): local generic arities and Self permission.
- [StructuralSignature](#structuralsignature): validated signature read/write operations.

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
is a type or has matching generic arity; the future reference layer must perform those
checks. Standalone byref-result syntax, full primitive support, declaration resolution,
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
