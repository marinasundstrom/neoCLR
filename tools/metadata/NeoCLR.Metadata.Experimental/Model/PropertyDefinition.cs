namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned Property row and local accessor associations; no code is loaded.</summary>
public sealed partial class PropertyDefinition
{
    private readonly byte[] signature;
    private readonly SignatureType? nativeType;
    private readonly IReadOnlyList<SignatureType>? nativeParameters;
    private readonly uint owner, getter, setter;
    internal PropertyDefinition(ModuleDefinition module, AssemblyDefinition.PropertyRow row)
    {
        Module = module; MetadataToken = row.Token; Name = row.Name; Attributes = row.Attributes;
        nativeType = row.NativeType?.Materialize(module);
        nativeParameters = row.NativeParameters is null ? null : Array.AsReadOnly(row.NativeParameters.Select(p => p.Materialize(module)).ToArray());
        signature = (byte[])row.Signature.Clone(); owner = row.DeclaringToken; getter = row.Getter; setter = row.Setter;
        OtherMethods = Array.AsReadOnly(row.Others.Select(token => module.GetMethodDefinition(token)!).ToArray());
    }
    /// <summary>Gets the owning module snapshot.</summary>
    public ModuleDefinition Module { get; internal set; } = null!;
    /// <summary>Gets the CLI Property token or validated native property origin token.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the exact declaring type in this snapshot.</summary>
    public TypeDefinition DeclaringType => PropertyType is null ? Module.GetTypeDefinition(owner)! : AuthoredOwner ?? throw new InvalidOperationException("property is detached");
    /// <summary>Gets the stored metadata name.</summary>
    public string Name { get; }
    /// <summary>Gets physical PropertyAttributes flags.</summary>
    public ushort Attributes { get; }
    /// <summary>Gets the getter from this snapshot, or null when absent.</summary>
    public MethodDefinition? GetMethod => PropertyType is null ? Module.GetMethodDefinition(getter) : authoredGetter;
    /// <summary>Gets the setter from this snapshot, or null when absent.</summary>
    public MethodDefinition? SetMethod => PropertyType is null ? Module.GetMethodDefinition(setter) : authoredSetter;
    /// <summary>Gets other associated methods in metadata order.</summary>
    public IReadOnlyList<MethodDefinition> OtherMethods { get; }
    /// <summary>Copies the signature, preserving unsupported encodings opaquely.</summary>
    /// <returns>New owned signature bytes.</returns>
    /// <exception cref="NotSupportedException">Native properties have no CLI signature blob; use TryGetSignature.</exception>
    public byte[] GetSignature() => nativeType is not null ? throw new NotSupportedException("native properties have no CLI signature blob; use TryGetSignature") : PropertyType is null ? (byte[])signature.Clone() : throw new InvalidOperationException("use authored PropertyType and ParameterTypes before encoding");
    /// <summary>Reads a supported non-indexed logical property type and staticness without resolving dependencies.</summary>
    /// <param name="type">Authored or snapshot-owned type on success; null otherwise.</param>
    /// <param name="isStatic">True for a static property on success; false otherwise.</param>
    /// <returns>True for authored non-indexed properties, supported native properties and primitive non-indexed CLI properties.</returns>
    public bool TryGetSignature(out SignatureType? type, out bool isStatic)
    {
        type = (nativeParameters is { Count: > 0 } ? null : nativeType) ?? (ParameterTypes is { Count: 0 } ? PropertyType : null);
        isStatic = false;
        if (type is not null) { isStatic = (GetMethod ?? SetMethod)!.IsStatic; return true; }
        if (!TryGetPrimitiveSignature(out var primitive, out isStatic)) return false;
        type = primitive;
        return true;
    }
    /// <summary>Reads a supported logical property signature including ordered index parameters.</summary>
    /// <param name="type">Property value type on success; null otherwise.</param>
    /// <param name="parameters">Immutable index types on success; empty otherwise. Excludes the setter value.</param>
    /// <param name="isStatic">True for static accessors on success; false otherwise.</param>
    /// <returns>True for authored/native properties and the bounded non-indexed primitive CLI profile.</returns>
    public bool TryGetSignature(out SignatureType? type, out IReadOnlyList<SignatureType> parameters, out bool isStatic)
    {
        type = nativeType ?? PropertyType;
        parameters = nativeParameters ?? ParameterTypes ?? Array.Empty<SignatureType>();
        isStatic = false;
        if (type is not null) { isStatic = (GetMethod ?? SetMethod)!.IsStatic; return true; }
        return TryGetSignature(out type, out isStatic);
    }
    /// <summary>Recognizes an exact non-indexed primitive property signature, without resolving types or validating accessor signatures.</summary>
    /// <param name="type">Int32/Int64/Boolean/String on success; Void otherwise.</param>
    /// <param name="isStatic">True for a recognized static signature; false for instance or unrecognized signatures.</param>
    /// <returns>False for malformed, indexed or other unsupported encodings.</returns>
    public bool TryGetPrimitiveSignature(out PrimitiveType type, out bool isStatic)
    {
        if (nativeType is { } native && nativeParameters is not { Count: > 0 })
        {
            type = native.Primitive ?? PrimitiveType.Void;
            isStatic = type != PrimitiveType.Void && (GetMethod ?? SetMethod)!.IsStatic;
            return type != PrimitiveType.Void;
        }
        type = signature.Length == 3 && signature[0] is 0x08 or 0x28 && signature[1] == 0 ? signature[2] switch {
            0x08 => PrimitiveType.Int32, 0x05 => PrimitiveType.Byte, 0x0a => PrimitiveType.Int64, 0x02 => PrimitiveType.Boolean, 0x0e => PrimitiveType.String,
            _ => PrimitiveType.Void } : PrimitiveType.Void;
        isStatic = type != PrimitiveType.Void && signature[0] == 0x08;
        return type != PrimitiveType.Void;
    }
}
