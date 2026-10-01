namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Access scopes for an instance field.</summary>
public enum FieldVisibility
{
    /// <summary>Accessible outside the assembly, subject to type visibility.</summary>
    Public,
    /// <summary>Accessible within the assembly.</summary>
    Internal,
    /// <summary>Accessible only within the declaring type.</summary>
    Private
}

/// <summary>An owned instance-field declaration with a supported value signature.</summary>
public sealed partial class FieldBuilder
{
    internal FieldBuilder(TypeBuilder owner, string name, SignatureType type, FieldVisibility visibility, int index, bool isReadOnly)
    { DeclaringType = owner; Name = name; FieldType = type; Visibility = visibility; Index = index; IsReadOnly = isReadOnly; }
    internal int Index { get; }
    /// <summary>Gets the declaring reference class.</summary>
    public TypeBuilder DeclaringType { get; }
    /// <summary>Gets the simple metadata name.</summary>
    public string Name { get; }
    /// <summary>Gets the storage signature, including scoped VAR and constructed owned classes.</summary>
    public SignatureType FieldType { get; }
    /// <summary>Gets declared accessibility.</summary>
    public FieldVisibility Visibility { get; }
    /// <summary>Gets whether stores are restricted to constructors of the declaring type.</summary>
    public bool IsReadOnly { get; }
}

public sealed partial class TypeBuilder
{
    private readonly List<FieldBuilder> fields = [];
    /// <summary>Gets owned instance fields in declaration order.</summary>
    public IReadOnlyList<FieldBuilder> Fields => fields.AsReadOnly();
    /// <summary>Adds an instance field to an owned nonstatic root class.</summary>
    /// <param name="name">Nonempty unique field name, at most 1024 characters.</param>
    /// <param name="type">Supported non-Void type, including vectors, owned constructed classes and declaring-type VAR. Method parameters are invalid.</param>
    /// <param name="visibility">Public, Internal or Private; defaults to Private.</param>
    /// <param name="isReadOnly">Restrict stores to declaring instance constructors; defaults to false.</param>
    /// <returns>A field handle owned by this type.</returns>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="ArgumentException">Invalid name/type/access, duplicate name or exceeded field limit.</exception>
    /// <exception cref="InvalidOperationException">This is a static class.</exception>
    /// <remarks>At most 256 fields per type and 4096 per assembly. No static/literal fields yet.</remarks>
    public FieldBuilder AddField(string name, SignatureType type, FieldVisibility visibility = FieldVisibility.Private, bool isReadOnly = false)
    {
        ArgumentNullException.ThrowIfNull(type);
        type.ValidateOwner(Assembly, typeArity: GenericParameterNames.Count);
        if (IsStatic || IsInterface) throw new InvalidOperationException("instance fields require a reference class");
        if (string.IsNullOrWhiteSpace(name) || name.Length > 1024 || name.Any(char.IsControl) ||
            type.Primitive == PrimitiveType.Void ||
            !Enum.IsDefined(visibility) || fields.Count >= 256 || fields.Any(f => f.Name == name))
            throw new ArgumentException("invalid or duplicate instance field");
        try { _ = new System.Text.UTF8Encoding(false, true).GetByteCount(name); }
        catch (System.Text.EncoderFallbackException error) { throw new ArgumentException("invalid field Unicode", error); }
        var field = new FieldBuilder(this, name, type, visibility, fields.Count, isReadOnly); fields.Add(field); return field;
    }
}
