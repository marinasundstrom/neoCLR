namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable Field row with an owned signature blob; no code is loaded.</summary>
public sealed class FieldDefinition
{
    private readonly byte[] signature;
    private readonly uint owner;
    internal FieldDefinition(ModuleDefinition module, AssemblyDefinition.FieldRow row)
    { Module = module; MetadataToken = row.Token; Name = row.Name; Attributes = row.Attributes; owner = row.DeclaringToken; signature = (byte[])row.Signature.Clone(); }
    /// <summary>Gets the owning snapshot.</summary>
    public ModuleDefinition Module { get; }
    /// <summary>Gets the physical Field token.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the declaring type, including the physical module pseudo-type for CLI globals.</summary>
    public TypeDefinition DeclaringType => Module.GetTypeDefinition(owner)!;
    /// <summary>Gets the metadata name.</summary>
    public string Name { get; }
    /// <summary>Gets physical FieldAttributes flags.</summary>
    public ushort Attributes { get; }
    /// <summary>Copies the raw signature, retaining unsupported encodings.</summary>
    /// <returns>New owned signature bytes.</returns>
    public byte[] GetSignature() => (byte[])signature.Clone();
    /// <summary>Recognizes an exact primitive field signature.</summary>
    /// <param name="type">Int32, Int64, Boolean or String on success; Void on failure.</param>
    /// <returns>False for unsupported or malformed encodings.</returns>
    public bool TryGetPrimitiveType(out PrimitiveType type)
    {
        type = signature.Length == 2 && signature[0] == 0x06 ? signature[1] switch {
            0x08 => PrimitiveType.Int32, 0x0a => PrimitiveType.Int64, 0x02 => PrimitiveType.Boolean, 0x0e => PrimitiveType.String,
            _ => PrimitiveType.Void } : PrimitiveType.Void;
        return type != PrimitiveType.Void;
    }
}
