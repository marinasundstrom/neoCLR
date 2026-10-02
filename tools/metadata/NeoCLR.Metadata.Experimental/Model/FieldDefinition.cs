namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable field declaration from CLI or native metadata; no code is loaded.</summary>
public sealed partial class FieldDefinition
{
    private readonly byte[] signature;
    private readonly PrimitiveType? nativeType;
    private readonly uint owner;
    internal FieldDefinition(ModuleDefinition module, AssemblyDefinition.FieldRow row)
    { Module = module; MetadataToken = row.Token; name = row.Name; Attributes = row.Attributes; owner = row.DeclaringToken; signature = (byte[])row.Signature.Clone(); nativeType = row.NativeType; }
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
    /// <exception cref="NotSupportedException">Native fields have no CLI signature blob; use TryGetPrimitiveType.</exception>
    public byte[] GetSignature() => nativeType is not null ? throw new NotSupportedException("native fields have no CLI signature blob; use TryGetPrimitiveType") : FieldType is null ? (byte[])signature.Clone() : throw new InvalidOperationException("authored signature tokens are assigned when writing; use FieldType");
    /// <summary>Recognizes an exact primitive field signature.</summary>
    /// <param name="type">Int32, Int64, Boolean or String on success; Void on failure.</param>
    /// <returns>False for unsupported or malformed encodings.</returns>
    public bool TryGetPrimitiveType(out PrimitiveType type)
    {
        if (nativeType is { } native) { type = native; return true; }
        if (FieldType is { } authored) { type = authored.Primitive ?? PrimitiveType.Void; return type != PrimitiveType.Void; }
        type = signature.Length == 2 && signature[0] == 0x06 ? signature[1] switch {
            0x08 => PrimitiveType.Int32, 0x0a => PrimitiveType.Int64, 0x02 => PrimitiveType.Boolean, 0x0e => PrimitiveType.String,
            _ => PrimitiveType.Void } : PrimitiveType.Void;
        return type != PrimitiveType.Void;
    }
}
