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
        if (Definition.CustomAttributes.Count(Definition.IsFlagsAttribute) > 1) throw new InvalidDataException("duplicate core FlagsAttribute");
        if (GenericParameterNames.Count != 0 || Definition.DeclaringType is not null || InterfaceSignatures.Any() || Methods.Count != 0 ||
            fields.Count != 1 || fields[0].Name != "value__" || fields[0].FieldType.Primitive != PrimitiveType.Int32 || fields[0].Definition.Attributes != 0x606 ||
            literalFields.Any(f => !f.Name.All(c => char.IsAsciiLetterOrDigit(c) || c == '_') || char.IsDigit(f.Name[0]) || !ReferenceEquals(f.FieldType.ClassType, this) || f.Definition.Constant is not int))
            throw new InvalidDataException("enum requires Int32 value__ storage and owned Int32 literal members");
    }
}

public sealed partial class TypeDefinition
{
    private readonly bool nativeEnumFlags;
    /// <summary>Gets whether this enum declares combinable flag values.</summary>
    /// <remarks>Reads the native enum classification or the exact core CLI FlagsAttribute, without loading the core.</remarks>
    public bool IsFlagsEnum => IsEnum && (Module?.Assembly.IsNative == true && Producer is null ? nativeEnumFlags : CustomAttributes.Any(IsFlagsAttribute));

    /// <summary>Marks an attached authored enum as a flags enum through the ordinary core FlagsAttribute.</summary>
    /// <exception cref="InvalidOperationException">The definition is detached, read-only, or not an enum.</exception>
    public void SetEnumFlags()
    {
        if (Producer is null || !IsEnum) throw new InvalidOperationException("flags require an attached authored enum");
        if (!IsFlagsEnum) CustomAttributes.Add(new CustomAttributeDefinition(Module!.ImportReference(Producer.Assembly.CoreLibrary, "System", "FlagsAttribute"), []));
    }

    internal bool IsFlagsAttribute(CustomAttributeDefinition attribute)
    {
        var reference = attribute.AttributeType;
        if (!IsEnum || reference.Namespace != "System" || reference.Name != "FlagsAttribute") return false;
        var core = Producer?.Assembly.CoreLibrary ?? ValueTypeCore;
        var scope = reference.ExplicitScope ?? (reference.ResolutionScopeToken >> 24 == 0x23
            ? reference.Module.AssemblyReferences.SingleOrDefault(a => a.MetadataToken == reference.ResolutionScopeToken)?.Identity
            : reference.ResolutionScopeToken == 0 ? reference.Module.Assembly.Identity : null);
        if (core is null || !Equals(core, scope)) return false;
        if (!attribute.GetConstructorSignature().AsSpan().SequenceEqual(new byte[] { 0x20, 0, 1 }) ||
            !attribute.GetValue().AsSpan().SequenceEqual(new byte[] { 1, 0, 0, 0 }))
            throw new InvalidDataException("invalid core FlagsAttribute contract");
        return true;
    }
}

public sealed partial class TypeBuilder
{
    /// <summary>Gets the flags classification shared with the authored definition.</summary>
    public bool IsFlagsEnum => Definition.IsFlagsEnum;
    /// <summary>Marks this enum as flags using the definition's standard attribute authoring path.</summary>
    /// <exception cref="InvalidOperationException">The owner is not an enum.</exception>
    public void SetEnumFlags() => Definition.SetEnumFlags();
}
