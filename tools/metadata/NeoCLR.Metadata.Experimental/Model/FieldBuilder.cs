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

/// <summary>An owned field declaration with a supported value signature.</summary>
public sealed partial class FieldBuilder
{
    internal FieldBuilder(TypeBuilder owner, FieldDefinition definition, int index)
    { DeclaringType = owner; Definition = definition; Index = index; }
    /// <summary>Gets the same authored field object held by the declaring definition.</summary>
    public FieldDefinition Definition { get; }
    internal int Index { get; }
    /// <summary>Gets the declaring nominal type.</summary>
    public TypeBuilder DeclaringType { get; }
    /// <summary>Gets the simple metadata name.</summary>
    public string Name => Definition.Name;
    /// <summary>Gets the storage signature, including scoped VAR and constructed owned classes.</summary>
    public SignatureType FieldType => Definition.FieldType!;
    /// <summary>Gets declared accessibility.</summary>
    public FieldVisibility Visibility => (Definition.Attributes & 7) switch { 6 => FieldVisibility.Public, 3 => FieldVisibility.Internal, _ => FieldVisibility.Private };
    /// <summary>Gets whether storage belongs to the declaring type rather than an instance.</summary>
    public bool IsStatic => (Definition.Attributes & 0x10) != 0;
    /// <summary>Gets whether stores are restricted to constructors of the declaring type.</summary>
    public bool IsReadOnly => (Definition.Attributes & 0x20) != 0;
}

public sealed partial class TypeBuilder
{
    private readonly List<FieldBuilder> fields = [];
    private readonly List<FieldBuilder> staticFields = [];
    /// <summary>Gets static storage fields, separately from instance layout.</summary>
    public IReadOnlyList<FieldBuilder> StaticFields => staticFields.AsReadOnly();
    /// <summary>Gets owned instance fields in declaration order.</summary>
    public IReadOnlyList<FieldBuilder> Fields => fields.AsReadOnly();
    /// <summary>Adds an instance or static field to an owned type.</summary>
    /// <param name="name">Nonempty unique field name, at most 1024 characters.</param>
    /// <param name="type">Supported non-Void type, including vectors, owned constructed classes and declaring-type VAR. Method parameters are invalid.</param>
    /// <param name="visibility">Public, Internal or Private; defaults to Private.</param>
    /// <param name="isReadOnly">Restrict stores to declaring instance constructors; defaults to false.</param>
    /// <param name="isStatic">Allocate per closed declaring type instead of per instance.</param>
    /// <returns>A field handle owned by this type.</returns>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="ArgumentException">Invalid name/type/access, unsupported value-type storage, duplicate name or exceeded field limit.</exception>
    /// <exception cref="InvalidOperationException">This is a static class.</exception>
    /// <remarks>At most 256 fields per storage category per type and 4096 per assembly. Static storage is default-initialized; static readonly stores and type initializers are not admitted. Value-type fields support nominal and constructed payloads. Direct self storage rejects on attachment; indirect inline cycles and bounded layout expansion reject when writing.</remarks>
    public FieldBuilder AddField(string name, SignatureType type, FieldVisibility visibility = FieldVisibility.Private, bool isReadOnly = false, bool isStatic = false)
    {
        var definition = new FieldDefinition(name, (ushort)((visibility switch { FieldVisibility.Public => 6, FieldVisibility.Internal => 3, FieldVisibility.Private => 1, _ => throw new ArgumentOutOfRangeException(nameof(visibility)) }) | (isReadOnly ? 0x20 : 0) | (isStatic ? 0x10 : 0)), type);
        Definition.Fields.Add(definition);
        return definition.Producer!;
    }
    internal FieldBuilder AttachField(FieldDefinition definition)
    {
        var name = definition.Name; var type = definition.FieldType!;
        var visibility = (definition.Attributes & 7) switch { 6 => FieldVisibility.Public, 3 => FieldVisibility.Internal, _ => FieldVisibility.Private };
        ArgumentNullException.ThrowIfNull(type);
        type.ValidateOwner(Assembly, typeArity: GenericParameterNames.Count);
        if (definition.IsLiteral)
        {
            if (!IsEnum || !ReferenceEquals(type.ClassType, this) || definition.Constant is null || literalFields.Count >= 256)
                throw new ArgumentException("literal fields require their enum owner and an Int32 constant");
            var literal = new FieldBuilder(this, definition, -1); literalFields.Add(literal); definition.Producer = literal; return literal;
        }
        var isStatic = (definition.Attributes & 0x10) != 0;
        if (type.ContainsComposition && (isStatic || IsValueType || IsInterface))
            throw new ArgumentException("composition fields require reference-class instance storage", nameof(definition));
        var storage = isStatic ? staticFields : fields;
        if (!isStatic && IsValueType && ReferenceEquals(type.ClassType ?? type.GenericInstance?.Definition, this)) throw new ArgumentException("recursive value-type storage", nameof(type));
        if (!isStatic && IsStatic || IsInterface) throw new InvalidOperationException("instance fields require a reference class");
        if (string.IsNullOrWhiteSpace(name) || name.Length > 1024 || name.Any(char.IsControl) ||
            type.Primitive == PrimitiveType.Void ||
            !Enum.IsDefined(visibility) || storage.Count >= 256 || MetadataFields.Any(f => f.Name == name))
            throw new ArgumentException("invalid or duplicate instance field");
        try { _ = new System.Text.UTF8Encoding(false, true).GetByteCount(name); }
        catch (System.Text.EncoderFallbackException error) { throw new ArgumentException("invalid field Unicode", error); }
        var field = new FieldBuilder(this, definition, storage.Count); storage.Add(field); definition.Producer = field; return field;
    }
}
