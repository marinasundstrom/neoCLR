using System.Collections.ObjectModel;

namespace NeoCLR.Metadata.Experimental.Model;

// Append-only ownership collections for the first definition migration slice.
// Replacement/removal needs reference invalidation rules before it can be admitted.
internal sealed class DefinitionCollection<T>(IEnumerable<T> initial, Action<T> attach) : Collection<T>(initial.ToList())
{
    protected override void InsertItem(int index, T item)
    {
        ArgumentNullException.ThrowIfNull(item);
        if (index != Count) throw new NotSupportedException("only append is supported during definition migration");
        attach(item); base.InsertItem(index, item);
    }
    protected override void SetItem(int index, T item) => throw new NotSupportedException("definition replacement is not yet supported");
    protected override void RemoveItem(int index) => throw new NotSupportedException("definition removal is not yet supported");
    protected override void ClearItems() => throw new NotSupportedException("definition removal is not yet supported");
}

public sealed partial class AssemblyDefinition
{
    internal AssemblyBuilder? Producer { get; private set; }
    internal static AssemblyDefinition ForProducer(AssemblyBuilder builder, Guid mvid)
        => new(builder.Identity, builder.Identity.Name + ".dll", mvid, [], [], [], [], [], [], [], null, [], 0) { Producer = builder };
    /// <summary>Creates an authored assembly with explicit target core identity.</summary>
    /// <remarks>Types and fields support direct construction. Loaded editing and definition-owned method bodies remain pending.</remarks>
    public static AssemblyDefinition CreateAssembly(AssemblyIdentity identity, AssemblyIdentity coreLibrary)
        => new AssemblyBuilder(identity, coreLibrary).Definition;
    /// <summary>Writes the authored assembly using the existing native encoding.</summary>
    /// <exception cref="InvalidOperationException">This is a loaded snapshot.</exception>
    public byte[] WriteNativeAssembly() => (Producer ?? throw new InvalidOperationException("loaded snapshot native rewriting is not supported")).WriteNativeAssembly();
}

public sealed partial class ModuleDefinition
{
    /// <summary>Creates an explicit assembly-scoped reference without loading the dependency or consulting the host runtime.</summary>
    public TypeReference ImportReference(AssemblyIdentity scope, string @namespace, string name)
    {
        ArgumentNullException.ThrowIfNull(scope);
        if (@namespace is null || string.IsNullOrWhiteSpace(name)) throw new ArgumentException("invalid type reference name");
        return new TypeReference(this, scope, @namespace, name);
    }
}

public sealed partial class TypeReference
{
    internal AssemblyIdentity? ExplicitScope { get; }
    internal TypeReference(ModuleDefinition module, AssemblyIdentity scope, string ns, string name)
    { Module = module; ExplicitScope = scope; Namespace = ns; Name = name; }
}

public sealed partial class TypeDefinition
{
    internal TypeBuilder? Producer { get; set; }
    private readonly IList<FieldDefinition>? authoredFields;
    private readonly IList<MethodDefinition>? authoredMethods;
    private readonly IList<PropertyDefinition>? authoredProperties;
    /// <summary>Creates a detached type declaration with CLI attributes and an explicit base reference.</summary>
    /// <remarks>Attach to an authored module's Types collection. This slice admits nongeneric interfaces, static/root classes and sealed sequential value types.</remarks>
    public TypeDefinition(string @namespace, string name, uint attributes, TypeReference? baseType)
    {
        if (@namespace is null || string.IsNullOrEmpty(name) || name == "<Module>" || @namespace.Length + name.Length > 1024 || (name + @namespace).Any(char.IsControl))
            throw new ArgumentException("invalid type name");
        Namespace = @namespace; Name = name; Attributes = attributes; BaseType = baseType; GenericParameterNames = Array.Empty<string>();
        IsValueType = baseType is { Namespace: "System", Name: "ValueType" or "Enum" };
        authoredProperties = new DefinitionCollection<PropertyDefinition>([], property =>
        {
            if (Producer is null) throw new InvalidOperationException("attach the owner before adding properties");
            Producer.AttachProperty(property);
        });
        authoredInterfaces = new DefinitionCollection<InterfaceImplementation>([], relationship =>
        {
            if (Producer is null) throw new InvalidOperationException("attach the owner before adding interface relationships");
            if (relationship.DeclaringType is not null) throw new ArgumentException("interface relationship already attached");
            TypeDefinition target;
            try { target = relationship.InterfaceType.Resolve(); }
            catch (InvalidDataException error) { throw new ArgumentException("interface relationship requires an owned definition reference", error); }
            if (target.Producer is not { } targetBuilder) throw new ArgumentException("interface relationship requires an attached authored target");
            if (relationship.TypeArguments.Count > 0) Producer.AttachConstructedInterface(targetBuilder.MakeGenericInstance(relationship.TypeArguments.ToArray()));
            else if (Producer.IsInterface) Producer.AttachBaseInterface(targetBuilder);
            else Producer.AttachInterfaceImplementation(targetBuilder);
            relationship.DeclaringType = this;
        });
        authoredMethods = new DefinitionCollection<MethodDefinition>([], method =>
        {
            if (Producer is null) throw new InvalidOperationException("attach the declaring type before adding methods");
            Producer.AttachMethod(method);
        });
        authoredFields = new DefinitionCollection<FieldDefinition>([], field =>
        {
            if (field.FieldType is null || field.AuthoredOwner is not null) throw new ArgumentException("field must be an unattached authored definition");
            if (Fields.Any(f => f.Name == field.Name)) throw new ArgumentException("duplicate field name");
            Producer?.AttachField(field);
            field.AuthoredOwner = this; field.Module = Module;
        });
    }
    /// <summary>Gets the explicit authored base reference; loaded base decoding remains pending.</summary>
    public TypeReference? BaseType { get; }
}

public sealed partial class FieldDefinition
{
    internal FieldBuilder? Producer { get; set; }
    internal TypeDefinition? AuthoredOwner { get; set; }
    /// <summary>Gets the authored signature; null for an opaque loaded field signature.</summary>
    public SignatureType? FieldType { get; }
    /// <summary>Creates a detached instance field using CLI access and optional InitOnly flags.</summary>
    public FieldDefinition(string name, ushort attributes, SignatureType fieldType)
    {
        ArgumentNullException.ThrowIfNull(fieldType); CheckName(name);
        if ((attributes & ~0x27) != 0 || (attributes & 7) is not (1 or 3 or 6) || fieldType.Primitive == PrimitiveType.Void)
            throw new ArgumentException("unsupported field attributes or signature");
        this.name = name; Attributes = attributes; FieldType = fieldType; signature = [];
    }
    private static void CheckName(string value)
    {
        if (string.IsNullOrWhiteSpace(value) || value.Length > 1024 || value.Any(char.IsControl)) throw new ArgumentException("invalid field name");
        try { _ = new System.Text.UTF8Encoding(false, true).GetByteCount(value); }
        catch (System.Text.EncoderFallbackException error) { throw new ArgumentException("invalid field Unicode", error); }
    }
}

public sealed partial class AssemblyBuilder
{
    /// <summary>Gets the existing facade over an authored definition without copying its graph.</summary>
    /// <exception cref="ArgumentNullException">Definition is null.</exception>
    /// <exception cref="InvalidOperationException">Definition is a loaded snapshot.</exception>
    public static AssemblyBuilder ForDefinition(AssemblyDefinition definition)
    {
        ArgumentNullException.ThrowIfNull(definition);
        return definition.Producer ?? throw new InvalidOperationException("loaded definition editing remains unsupported");
    }
    internal TypeDefinition AttachType(TypeDefinition definition)
    {
        if (types.Count >= 256 || types.Any(t => t.Namespace == definition.Namespace && t.Name == definition.Name) ||
            definition.MetadataToken != 0 || definition.Producer is { } existing && !ReferenceEquals(existing.Assembly, this))
            throw new ArgumentException("foreign, duplicate or excessive type definition");
        if (definition.Producer is null)
        {
            var attributes = definition.Attributes;
            var category = attributes & ~1u;
            if (definition.GenericArity == 0 && definition.Name.Contains('`') || definition.GenericArity > 0 && category == 0x180 || category is not (0 or 0x180 or 0x108 or 0xa0)) throw new ArgumentException("unsupported manual type shape");
            if (category == 0xa0)
            {
                if (definition.BaseType is not null) throw new ArgumentException("interfaces have no class base");
            }
            else if (definition.BaseType is not { } baseType || !ReferenceEquals(baseType.Module, Definition.MainModule) || !Equals(baseType.ExplicitScope, CoreLibrary) || baseType.Namespace != "System" ||
                baseType.Name != (definition.IsValueType ? "ValueType" : "Object") || definition.IsValueType != (category == 0x108))
                throw new ArgumentException("type base/category does not match the explicit core contract");
            // Validate pending fields before attaching any ownership or writer handles.
            foreach (var field in definition.Fields)
            {
                field.FieldType!.ValidateOwner(this, typeArity: definition.GenericArity);
                if (category is 0x180 or 0xa0 || definition.IsValueType && field.FieldType.Primitive is null && field.FieldType.TypeParameterIndex is null)
                    throw new ArgumentException("unsupported field storage on manual type");
            }
            if (definition.Fields.Count > 256) throw new ArgumentException("field limit exceeded");
            definition.Module = Definition.MainModule;
            var builder = new TypeBuilder(this, definition);
            foreach (var field in definition.Fields) { field.Module = Definition.MainModule; builder.AttachField(field); }
        }
        types.Add(definition.Producer!);
        return definition;
    }
}
