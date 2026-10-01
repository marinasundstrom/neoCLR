namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Access scopes for a mutable instance field.</summary>
public enum FieldVisibility
{
    /// <summary>Accessible outside the assembly, subject to type visibility.</summary>
    Public,
    /// <summary>Accessible within the assembly.</summary>
    Internal,
    /// <summary>Accessible only within the declaring type.</summary>
    Private
}

/// <summary>An owned, mutable primitive or owned root-class instance-field declaration.</summary>
public sealed class FieldBuilder
{
    internal FieldBuilder(TypeBuilder owner, string name, SignatureType type, FieldVisibility visibility, int index)
    { DeclaringType = owner; Name = name; FieldType = type; Visibility = visibility; Index = index; }
    internal int Index { get; }
    /// <summary>Gets the declaring reference class.</summary>
    public TypeBuilder DeclaringType { get; }
    /// <summary>Gets the simple metadata name.</summary>
    public string Name { get; }
    /// <summary>Gets the primitive or owned root-class storage type.</summary>
    public SignatureType FieldType { get; }
    /// <summary>Gets declared accessibility.</summary>
    public FieldVisibility Visibility { get; }
}

public sealed partial class TypeBuilder
{
    private readonly List<FieldBuilder> fields = [];
    /// <summary>Gets owned instance fields in declaration order.</summary>
    public IReadOnlyList<FieldBuilder> Fields => fields.AsReadOnly();
    /// <summary>Adds a mutable primitive or owned root-class instance field to a nonstatic root class.</summary>
    /// <param name="name">Nonempty unique field name, at most 1024 characters.</param>
    /// <param name="type">Int32, Int64, Boolean, String or a nonstatic class owned by this assembly.</param>
    /// <param name="visibility">Public, Internal or Private; defaults to Private.</param>
    /// <returns>A field handle owned by this type.</returns>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="ArgumentException">Invalid name/type/access, duplicate name or exceeded field limit.</exception>
    /// <exception cref="InvalidOperationException">This is a static class.</exception>
    /// <remarks>At most 256 fields per type and 4096 per assembly. No static/readonly/literal fields yet.</remarks>
    public FieldBuilder AddField(string name, SignatureType type, FieldVisibility visibility = FieldVisibility.Private)
    {
        ArgumentNullException.ThrowIfNull(type);
        if (type.ClassType is { } owner && !ReferenceEquals(owner.Assembly, Assembly))
            throw new ArgumentException("field class belongs to another output", nameof(type));
        if (IsStatic) throw new InvalidOperationException("instance fields require a reference class");
        if (string.IsNullOrWhiteSpace(name) || name.Length > 1024 || name.Any(char.IsControl) ||
            type.Primitive == PrimitiveType.Void ||
            !Enum.IsDefined(visibility) || fields.Count >= 256 || fields.Any(f => f.Name == name))
            throw new ArgumentException("invalid or duplicate instance field");
        try { _ = new System.Text.UTF8Encoding(false, true).GetByteCount(name); }
        catch (System.Text.EncoderFallbackException error) { throw new ArgumentException("invalid field Unicode", error); }
        var field = new FieldBuilder(this, name, type, visibility, fields.Count); fields.Add(field); return field;
    }
}
