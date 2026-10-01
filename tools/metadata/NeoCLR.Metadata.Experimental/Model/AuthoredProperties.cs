namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class PropertyDefinition
{
    internal TypeDefinition? AuthoredOwner { get; set; }
    internal PropertyBuilder? Producer { get; set; }
    private readonly MethodDefinition? authoredGetter, authoredSetter;
    /// <summary>Creates a detached property association over authored accessor declarations.</summary>
    /// <param name="name">Nonblank metadata name, validated on attachment.</param>
    /// <param name="propertyType">Non-Void supported signature.</param>
    /// <param name="getter">Authored getter, or null for a write-only property.</param>
    /// <param name="setter">Authored setter, or null for a read-only property.</param>
    /// <exception cref="ArgumentNullException">Property type is null.</exception>
    /// <exception cref="ArgumentException">Missing or loaded accessors.</exception>
    /// <remarks>Attach to an attached authored type after attaching its accessors. Ownership and signatures are validated on attachment.</remarks>
    public PropertyDefinition(string name, SignatureType propertyType, MethodDefinition? getter = null, MethodDefinition? setter = null)
    {
        ArgumentNullException.ThrowIfNull(propertyType);
        if (getter is null && setter is null || getter is not null && getter.AuthoredSignature is null || setter is not null && setter.AuthoredSignature is null)
            throw new ArgumentException("property requires authored accessors");
        Name = name; PropertyType = propertyType; authoredGetter = getter; authoredSetter = setter;
        ParameterTypes = Array.AsReadOnly(getter is not null ? getter.AuthoredSignature!.ParameterTypes.ToArray()
            : setter!.AuthoredSignature!.ParameterTypes.Take(Math.Max(0, setter.AuthoredSignature.ParameterTypes.Count - 1)).ToArray());
        signature = []; OtherMethods = Array.Empty<MethodDefinition>();
    }
    /// <summary>Gets the authored value signature; null for an opaque loaded signature.</summary>
    public SignatureType? PropertyType { get; }
    /// <summary>Gets authored index parameter signatures; null for loaded signatures not decoded here.</summary>
    public IReadOnlyList<SignatureType>? ParameterTypes { get; }
}
