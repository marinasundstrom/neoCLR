namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned supported value property with explicit accessor associations.</summary>
public sealed class PropertyBuilder
{
    internal PropertyBuilder(TypeBuilder owner, string name, SignatureType type, MethodBuilder? getter, MethodBuilder? setter, SignatureType[] parameters)
    { DeclaringType = owner; Name = name; PropertyType = type; GetMethod = getter; SetMethod = setter; ParameterTypes = Array.AsReadOnly(parameters); }
    /// <summary>Gets the declaring type.</summary>
    public TypeBuilder DeclaringType { get; }
    /// <summary>Gets the simple property name.</summary>
    public string Name { get; }
    /// <summary>Gets the value signature, including declaring-type parameters and constructed classes.</summary>
    public SignatureType PropertyType { get; }
    /// <summary>Gets copied index parameter types, excluding receiver and setter value.</summary>
    public IReadOnlyList<SignatureType> ParameterTypes { get; }
    /// <summary>Gets the getter, or null for a write-only property.</summary>
    public MethodBuilder? GetMethod { get; }
    /// <summary>Gets the setter, or null for a read-only property.</summary>
    public MethodBuilder? SetMethod { get; }
    /// <summary>Gets whether the accessor contract has no receiver.</summary>
    public bool IsStatic => (GetMethod ?? SetMethod)!.IsStatic;
}

public sealed partial class TypeBuilder
{
    private readonly List<PropertyBuilder> properties = [];
    /// <summary>Gets owned properties in declaration order.</summary>
    public IReadOnlyList<PropertyBuilder> Properties => properties.AsReadOnly();
    /// <summary>Associates existing methods with a supported value property; adds no storage or bodies.</summary>
    /// <param name="name">Nonblank name, unique together with its index parameter types, at most 1024 characters, without controls or invalid Unicode.</param>
    /// <param name="type">Supported non-Void value signature, including declaring-type parameters; method parameters are invalid.</param>
    /// <param name="getter">Owned ordinary method whose parameters define the indices and whose result is the property type, or null.</param>
    /// <param name="setter">Owned ordinary method with matching index parameters followed by a property-value parameter and Void result, or null.</param>
    /// <returns>An immutable association owned by this type.</returns>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="ArgumentException">Invalid name/type, missing or incompatible accessors, duplicate name, reused accessor or exceeded limit.</exception>
    /// <remarks>At least one accessor is required. Index parameters are inferred from accessors and copied. Both must agree on index types and instance/static shape; visibility stays on each accessor. At most 256 properties per type and 4096 per assembly.</remarks>
    /// <exception cref="InvalidOperationException">Interface property declarations are not supported by this builder yet.</exception>
    public PropertyBuilder AddProperty(string name, SignatureType type, MethodBuilder? getter = null, MethodBuilder? setter = null)
    {
        if (IsInterface) throw new InvalidOperationException("interface properties are not supported yet");
        ArgumentNullException.ThrowIfNull(type);
        type.ValidateOwner(Assembly, typeArity: GenericParameterNames.Count);
        if (string.IsNullOrWhiteSpace(name) || name.Length > 1024 || name.Any(char.IsControl) ||
            type.Primitive == PrimitiveType.Void ||
            properties.Count >= 256 || getter is null && setter is null)
            throw new ArgumentException("invalid or duplicate property");
        try { _ = new System.Text.UTF8Encoding(false, true).GetByteCount(name); }
        catch (System.Text.EncoderFallbackException error) { throw new ArgumentException("invalid property Unicode", error); }
        foreach (var accessor in new[] { getter, setter }.OfType<MethodBuilder>())
            if (!ReferenceEquals(accessor.DeclaringType, this) || accessor.IsConstructor || accessor.Signature.GenericParameterNames.Count != 0 ||
                properties.Any(p => ReferenceEquals(p.GetMethod, accessor) || ReferenceEquals(p.SetMethod, accessor)))
                throw new ArgumentException("accessor must be an unassociated ordinary method of this type");
        var indices = getter is not null ? getter.Signature.ParameterTypes.ToArray()
            : setter!.Signature.ParameterTypes.Take(Math.Max(0, setter.ParameterCount - 1)).ToArray();
        if (getter is not null && getter.Signature.ReturnType != type ||
            setter is not null && (setter.ParameterCount != indices.Length + 1 || setter.Signature.ParameterTypes[^1] != type ||
                !setter.Signature.ParameterTypes.Take(indices.Length).SequenceEqual(indices) || setter.ReturnsValue) ||
            getter is not null && setter is not null && getter.IsStatic != setter.IsStatic)
            throw new ArgumentException("incompatible property accessor signature");
        if (properties.Any(p => p.Name == name && p.ParameterTypes.SequenceEqual(indices)))
            throw new ArgumentException("duplicate property signature");
        var property = new PropertyBuilder(this, name, type, getter, setter, indices);
        properties.Add(property);
        return property;
    }
}
