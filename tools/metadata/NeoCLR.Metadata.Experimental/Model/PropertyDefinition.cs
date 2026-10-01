namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned Property row and local accessor associations; no code is loaded.</summary>
public sealed class PropertyDefinition
{
    private readonly byte[] signature;
    private readonly uint owner, getter, setter;
    internal PropertyDefinition(ModuleDefinition module, AssemblyDefinition.PropertyRow row)
    {
        Module = module; MetadataToken = row.Token; Name = row.Name; Attributes = row.Attributes;
        signature = (byte[])row.Signature.Clone(); owner = row.DeclaringToken; getter = row.Getter; setter = row.Setter;
        OtherMethods = Array.AsReadOnly(row.Others.Select(token => module.GetMethodDefinition(token)!).ToArray());
    }
    /// <summary>Gets the owning module snapshot.</summary>
    public ModuleDefinition Module { get; }
    /// <summary>Gets the physical Property token.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the exact declaring type in this snapshot.</summary>
    public TypeDefinition DeclaringType => Module.GetTypeDefinition(owner)!;
    /// <summary>Gets the stored metadata name.</summary>
    public string Name { get; }
    /// <summary>Gets physical PropertyAttributes flags.</summary>
    public ushort Attributes { get; }
    /// <summary>Gets the getter from this snapshot, or null when absent.</summary>
    public MethodDefinition? GetMethod => Module.GetMethodDefinition(getter);
    /// <summary>Gets the setter from this snapshot, or null when absent.</summary>
    public MethodDefinition? SetMethod => Module.GetMethodDefinition(setter);
    /// <summary>Gets other associated methods in metadata order.</summary>
    public IReadOnlyList<MethodDefinition> OtherMethods { get; }
    /// <summary>Copies the signature, preserving unsupported encodings opaquely.</summary>
    /// <returns>New owned signature bytes.</returns>
    public byte[] GetSignature() => (byte[])signature.Clone();
    /// <summary>Recognizes an exact non-indexed primitive property signature, without resolving types or validating accessor signatures.</summary>
    /// <param name="type">Int32/Int64/Boolean/String on success; Void otherwise.</param>
    /// <param name="isStatic">True for a recognized static signature; false for instance or unrecognized signatures.</param>
    /// <returns>False for malformed, indexed or other unsupported encodings.</returns>
    public bool TryGetPrimitiveSignature(out PrimitiveType type, out bool isStatic)
    {
        type = signature.Length == 3 && signature[0] is 0x08 or 0x28 && signature[1] == 0 ? signature[2] switch {
            0x08 => PrimitiveType.Int32, 0x0a => PrimitiveType.Int64, 0x02 => PrimitiveType.Boolean, 0x0e => PrimitiveType.String,
            _ => PrimitiveType.Void } : PrimitiveType.Void;
        isStatic = type != PrimitiveType.Void && signature[0] == 0x08;
        return type != PrimitiveType.Void;
    }
}
