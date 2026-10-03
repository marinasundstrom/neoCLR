namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable field declaration from CLI or native metadata; no code is loaded.</summary>
public sealed partial class FieldDefinition
{
    private readonly byte[] signature;
    private readonly SignatureType? nativeType;
    private readonly uint owner;
    internal FieldDefinition(ModuleDefinition module, AssemblyDefinition.FieldRow row)
    { Module = module; MetadataToken = row.Token; name = row.Name; Attributes = row.Attributes; owner = row.DeclaringToken; signature = (byte[])row.Signature.Clone(); nativeType = row.NativeType?.Materialize(module); }
    /// <summary>Gets the owning snapshot.</summary>
    public ModuleDefinition Module { get; internal set; } = null!;
    /// <summary>Gets the CLI Field or validated native origin token.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the declaring type, including the physical module pseudo-type for CLI globals.</summary>
    public TypeDefinition DeclaringType => AuthoredOwner ?? Module.GetTypeDefinition(owner)!;
    /// <summary>Gets the metadata name.</summary>
    private string name = "";
    /// <summary>Gets the name; authored fields may be renamed with duplicate/name validation.</summary>
    public string Name
    {
        get => name;
        set
        {
            if (FieldType is null) throw new InvalidOperationException("loaded fields are read-only");
            CheckName(value);
            if (AuthoredOwner?.Fields.Any(f => !ReferenceEquals(f, this) && f.Name == value) == true) throw new ArgumentException("duplicate field name");
            name = value;
        }
    }
    /// <summary>Gets CLI-shaped FieldAttributes flags.</summary>
    public ushort Attributes { get; }
    /// <summary>Copies the raw signature, retaining unsupported encodings.</summary>
    /// <returns>New owned signature bytes.</returns>
    /// <exception cref="NotSupportedException">Native fields have no CLI signature blob; use TryGetSignature.</exception>
    public byte[] GetSignature() => nativeType is not null ? throw new NotSupportedException("native fields have no CLI signature blob; use TryGetSignature") : FieldType is null ? (byte[])signature.Clone() : throw new InvalidOperationException("authored signature tokens are assigned when writing; use FieldType");
    /// <summary>Reads the supported logical field type without loading code or resolving dependencies.</summary>
    /// <param name="type">Snapshot-owned or authored signature on success; otherwise null.</param>
    /// <returns>True for authored signatures, native primitive/nominal-class/vector fields and primitive CLI fields. False for other loaded CLI signatures.</returns>
    /// <remarks>Loaded nominal references resolve to the canonical definition through its local module or an explicit resolver. Import them before use as builder operands.</remarks>
    public bool TryGetSignature(out SignatureType? type)
    {
        type = nativeType ?? FieldType;
        if (type is not null) return true;
        if (!TryGetPrimitiveType(out var primitive)) return false;
        type = primitive;
        return true;
    }
    /// <summary>Recognizes an exact primitive field signature.</summary>
    /// <param name="type">Int32, Int64, Boolean or String on success; Void on failure.</param>
    /// <returns>False for unsupported or malformed encodings.</returns>
    public bool TryGetPrimitiveType(out PrimitiveType type)
    {
        if (nativeType is { } native) { type = native.Primitive ?? PrimitiveType.Void; return type != PrimitiveType.Void; }
        if (FieldType is { } authored) { type = authored.Primitive ?? PrimitiveType.Void; return type != PrimitiveType.Void; }
        type = signature.Length == 2 && signature[0] == 0x06 ? signature[1] switch {
            0x08 => PrimitiveType.Int32, 0x05 => PrimitiveType.Byte, 0x0a => PrimitiveType.Int64, 0x02 => PrimitiveType.Boolean, 0x0e => PrimitiveType.String,
            _ => PrimitiveType.Void } : PrimitiveType.Void;
        return type != PrimitiveType.Void;
    }
}
