using System.Reflection;

namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly HashSet<ImportedTypeReference> enumReferences = [];
    internal bool IsEnumSignature(SignatureType type) => type.ClassType?.IsEnum == true || type.ImportedType is { } imported && enumReferences.Contains(imported);
    /// <summary>Authors a public Int32 enum identity from compiler-owned facts and an explicit dependency digest.</summary>
    /// <remarks>Identity validation matches CreateValueTypeReference. No dependency is loaded; native linking verifies the actual storage definition.</remarks>
    public ImportedTypeReference CreateEnumReference(AssemblyIdentity dependency, AssemblyIdentity dependencyCoreLibrary,
        string artifactSha256, string @namespace, string name)
    {
        var reference = CreateValueTypeReference(dependency, dependencyCoreLibrary, artifactSha256, @namespace, name);
        enumReferences.Add(reference);
        return reference;
    }

    /// <summary>Adds a CLI-shaped Int32-backed enum with its value__ storage field.</summary>
    /// <remarks>Only nongeneric, top-level Int32 enums are supported. Members are literal fields.</remarks>
    public TypeBuilder AddEnum(string @namespace, string name, TypeVisibility visibility = TypeVisibility.Public)
    {
        if (!Enum.IsDefined(visibility)) throw new ArgumentOutOfRangeException(nameof(visibility));
        var definition = new TypeDefinition(@namespace, name,
            (uint)(TypeAttributes.Sealed | (visibility == TypeVisibility.Public ? TypeAttributes.Public : TypeAttributes.NotPublic)),
            Definition.MainModule.ImportReference(CoreLibrary, "System", "Enum"));
        definition.Fields.Add(new FieldDefinition("value__", 0x606, PrimitiveType.Int32));
        Definition.MainModule.Types.Add(definition);
        return definition.Producer!;
    }
}

public sealed partial class TypeBuilder
{
    private readonly List<FieldBuilder> literalFields = [];
    internal IEnumerable<FieldBuilder> MetadataFields => fields.Concat(literalFields);
    /// <summary>Gets whether this type directly extends the selected core System.Enum.</summary>
    public bool IsEnum => Definition.IsEnum;
    /// <summary>Adds a public static literal with this enum's nominal type and an Int32 constant.</summary>
    public FieldBuilder AddEnumMember(string name, int value)
    {
        if (!IsEnum) throw new InvalidOperationException("enum member requires an enum owner");
        var definition = new FieldDefinition(name, 0x8056, (SignatureType)this, value);
        Definition.Fields.Add(definition);
        return definition.Producer!;
    }
    internal void ValidateEnum()
    {
        if (!IsEnum) return;
        if (GenericParameterNames.Count != 0 || Definition.DeclaringType is not null || InterfaceSignatures.Any() || Methods.Count != 0 ||
            fields.Count != 1 || fields[0].Name != "value__" || fields[0].FieldType.Primitive != PrimitiveType.Int32 || fields[0].Definition.Attributes != 0x606 ||
            literalFields.Any(f => !f.Name.All(c => char.IsAsciiLetterOrDigit(c) || c == '_') || char.IsDigit(f.Name[0]) || !ReferenceEquals(f.FieldType.ClassType, this) || f.Definition.Constant is not int))
            throw new InvalidDataException("enum requires Int32 value__ storage and owned Int32 literal members");
    }
}
