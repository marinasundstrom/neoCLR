namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned non-indexed primitive or owned root-class property with explicit accessor associations.</summary>
public sealed class PropertyBuilder
{
    internal PropertyBuilder(TypeBuilder owner, string name, SignatureType type, MethodBuilder? getter, MethodBuilder? setter)
    { DeclaringType = owner; Name = name; PropertyType = type; GetMethod = getter; SetMethod = setter; }
    /// <summary>Gets the declaring type.</summary>
    public TypeBuilder DeclaringType { get; }
    /// <summary>Gets the simple property name.</summary>
    public string Name { get; }
    /// <summary>Gets the primitive or owned root-class property value type.</summary>
    public SignatureType PropertyType { get; }
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
    /// <summary>Associates existing methods with a non-indexed primitive or owned root-class property; adds no storage or bodies.</summary>
    /// <param name="name">Unique nonblank name, at most 1024 characters, without controls or invalid Unicode.</param>
    /// <param name="type">Int32, Int64, Boolean, String or a nonstatic class owned by this assembly.</param>
    /// <param name="getter">Owned ordinary method with no declared parameters and the property result, or null.</param>
    /// <param name="setter">Owned ordinary method with one property-value parameter and Void result, or null.</param>
    /// <returns>An immutable association owned by this type.</returns>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="ArgumentException">Invalid name/type, missing or incompatible accessors, duplicate name, reused accessor or exceeded limit.</exception>
    /// <remarks>At least one accessor is required. Both must agree on instance/static shape; visibility stays on each accessor. At most 256 properties per type and 4096 per assembly.</remarks>
    public PropertyBuilder AddProperty(string name, SignatureType type, MethodBuilder? getter = null, MethodBuilder? setter = null)
    {
        ArgumentNullException.ThrowIfNull(type);
        type.ValidateOwner(Assembly);
        if (string.IsNullOrWhiteSpace(name) || name.Length > 1024 || name.Any(char.IsControl) ||
            type.Primitive == PrimitiveType.Void ||
            properties.Count >= 256 || properties.Any(p => p.Name == name) || getter is null && setter is null)
            throw new ArgumentException("invalid or duplicate property");
        try { _ = new System.Text.UTF8Encoding(false, true).GetByteCount(name); }
        catch (System.Text.EncoderFallbackException error) { throw new ArgumentException("invalid property Unicode", error); }
        foreach (var accessor in new[] { getter, setter }.OfType<MethodBuilder>())
            if (!ReferenceEquals(accessor.DeclaringType, this) || accessor.IsConstructor ||
                properties.Any(p => ReferenceEquals(p.GetMethod, accessor) || ReferenceEquals(p.SetMethod, accessor)))
                throw new ArgumentException("accessor must be an unassociated ordinary method of this type");
        if (getter is not null && (getter.ParameterCount != 0 || getter.Signature.ReturnType != type) ||
            setter is not null && (setter.ParameterCount != 1 || setter.Signature.ParameterTypes[0] != type || setter.ReturnsValue) ||
            getter is not null && setter is not null && getter.IsStatic != setter.IsStatic)
            throw new ArgumentException("incompatible property accessor signature");
        var property = new PropertyBuilder(this, name, type, getter, setter);
        properties.Add(property);
        return property;
    }
}
