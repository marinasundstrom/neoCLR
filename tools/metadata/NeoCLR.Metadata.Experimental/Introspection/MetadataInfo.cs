using NeoCLR.Metadata.Experimental.Model;

namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>A metadata-only assembly facade, shaped after System.Introspection.AssemblyInfo.</summary>
public sealed class AssemblyInfo
{
    private readonly IReadOnlyList<ModuleInfo> modules;
    private readonly Lazy<IReadOnlyList<AssemblyInfo>> references;
    internal AssemblyInfo(MetadataLoadContext context, AssemblyDefinition definition)
    {
        Definition = definition;
        modules = Array.AsReadOnly(new[] { new ModuleInfo(context, this, definition.MainModule) });
        references = new(() => Array.AsReadOnly(definition.MainModule.AssemblyReferences.Select(context.Resolve).ToArray()));
    }
    internal AssemblyDefinition Definition { get; }
    /// <summary>Gets the simple metadata name.</summary>
    public string Name => Definition.Name;
    /// <summary>Gets the exact identity, including version, culture, key token and flags.</summary>
    public AssemblyIdentity Identity => Definition.Identity;
    /// <summary>Gets resolved direct dependencies; missing catalog entries throw InvalidDataException on access.</summary>
    /// <remarks>Traversal is one edge at a time. Legal dependency cycles return the existing assembly views.</remarks>
    public IReadOnlyList<AssemblyInfo> ReferencedAssemblies => references.Value;
    /// <summary>Gets modules. The current reader supports one main module only.</summary>
    public IReadOnlyList<ModuleInfo> GetModules() => modules;
    /// <summary>Gets nominal definitions including nested types, excluding the CLI module pseudo-type.</summary>
    public IReadOnlyList<NominalTypeInfo> GetTypes() => modules[0].GetTypes();
}

/// <summary>A metadata-only module facade, with its stable owning assembly.</summary>
public sealed class ModuleInfo
{
    private readonly ModuleDefinition definition;
    private readonly MetadataLoadContext context;
    private readonly Lazy<IReadOnlyList<NominalTypeInfo>> types;
    internal ModuleInfo(MetadataLoadContext context, AssemblyInfo assembly, ModuleDefinition definition)
    {
        this.context = context;
        Assembly = assembly;
        this.definition = definition;
        types = new(() => Array.AsReadOnly(definition.Types.Where(t => t.Name != "<Module>").Select(context.GetType).ToArray()));
    }
    /// <summary>Gets the metadata module name.</summary>
    public string Name => definition.Name;
    /// <summary>Gets the canonical owning assembly view.</summary>
    public AssemblyInfo Assembly { get; }
    /// <summary>Gets declared nominal types in metadata order, including nested types.</summary>
    public IReadOnlyList<NominalTypeInfo> GetTypes() => types.Value;
    /// <summary>Gets namespace-level functions in metadata order.</summary>
    public IReadOnlyList<MethodInfo> GetFunctions() => Array.AsReadOnly(definition.Functions.Select(context.Resolve).ToArray());
}

/// <summary>The initial metadata-only type facade; additional type families will extend this model.</summary>
/// <remarks>This prototype deliberately exposes a subset of runtime System.Introspection.TypeInfo.
/// Missing capabilities are not fabricated as empty collections or runtime handles.</remarks>
public abstract class TypeInfo
{
    private protected TypeInfo(MetadataLoadContext context) { Context = context; }
    internal MetadataLoadContext Context { get; }
    internal virtual int Depth => 0;
    /// <summary>Gets a display label, not a binding identity.</summary>
    public abstract string DisplayName { get; }
    /// <summary>Gets whether this represents a nominal declaration.</summary>
    public abstract bool IsNominalType { get; }
}

/// <summary>A canonical nominal definition view within one metadata load context.</summary>
/// <remarks>Reference equality expresses identity within a context. Equal names in other scopes or contexts remain distinct.</remarks>
public sealed class NominalTypeInfo : TypeInfo
{
    private readonly MetadataLoadContext context;
    private readonly TypeDefinition definition;
    internal NominalTypeInfo(MetadataLoadContext context, TypeDefinition definition) : base(context)
    {
        this.context = context; this.definition = definition;
        parameters = Array.AsReadOnly(Enumerable.Range(0, definition.GenericArity).Select(i => (TypeInfo)new GenericParameterTypeInfo(context, this, i)).ToArray());
        fields = new(() => context.ProjectFields(definition, this, parameters));
        properties = new(() => context.ProjectProperties(definition, this, parameters));
        interfaces = new(() => context.ProjectInterfaces(definition, parameters));
        allInterfaces = new(() => context.ProjectInterfaceClosure(this));
        attributes = new(() => Array.AsReadOnly(definition.CustomAttributes.Select(a => new CustomAttributeInfo(context, a)).ToArray()));
    }
    private readonly IReadOnlyList<TypeInfo> parameters;
    private readonly Lazy<IReadOnlyList<FieldInfo>> fields;
    private readonly Lazy<IReadOnlyList<PropertyInfo>> properties;
    private readonly Lazy<IReadOnlyList<TypeInfo>> interfaces;
    private readonly Lazy<IReadOnlyList<TypeInfo>> allInterfaces;
    private readonly Lazy<IReadOnlyList<CustomAttributeInfo>> attributes;
    internal TypeDefinition Definition => definition;
    /// <summary>Gets declared metadata-only attribute records without loading dependencies or executing constructors.</summary>
    /// <remarks>Fixed-argument decoding is bounded; raw CLI blobs preserve other loaded categories.</remarks>
    public IReadOnlyList<CustomAttributeInfo> GetCustomAttributes() => attributes.Value;
    /// <summary>Gets stable owner-scoped generic parameter views in declaration order.</summary>
    public IReadOnlyList<TypeInfo> GetGenericArguments() => parameters;
    /// <summary>Constructs this generic definition using copied, same-context arguments.</summary>
    /// <exception cref="ArgumentException">Wrong arity, foreign/Void arguments or excessive nesting.</exception>
    /// <exception cref="ArgumentNullException">Arguments are null.</exception>
    public ConstructedTypeInfo MakeGenericType(params TypeInfo[] arguments) => context.Construct(this, arguments);
    /// <summary>Gets all declared fields in metadata order, without inherited-member or visibility filtering.</summary>
    /// <exception cref="InvalidDataException">A field signature is unsupported or its dependency is missing.</exception>
    public IReadOnlyList<FieldInfo> GetFields() => fields.Value;
    /// <summary>Gets declared properties in metadata order, including non-public and indexed properties.</summary>
    /// <exception cref="InvalidDataException">A signature or dependency is unsupported or missing.</exception>
    public IReadOnlyList<PropertyInfo> GetProperties() => properties.Value;
    /// <summary>Gets directly declared interface relationships with owner arguments substituted; no transitive traversal.</summary>
    /// <exception cref="NotSupportedException">The reader cannot materialize interface relationships for this format.</exception>
    /// <exception cref="InvalidDataException">A relationship or dependency cannot be resolved.</exception>
    public IReadOnlyList<TypeInfo> GetDeclaredInterfaces() => interfaces.Value;
    /// <summary>Gets distinct direct and inherited interface views in depth-first metadata order, excluding this type.</summary>
    /// <exception cref="InvalidDataException">An invalid/cyclic relationship, missing dependency or traversal bound is encountered.</exception>
    /// <exception cref="NotSupportedException">The reader cannot materialize interface relationships.</exception>
    public IReadOnlyList<TypeInfo> GetInterfaces() => allInterfaces.Value;
    /// <summary>Gets declared non-constructor methods without inherited lookup or visibility filtering.</summary>
    public IReadOnlyList<MethodInfo> GetMethods() => context.GetMethods(definition, this);
    /// <summary>Gets declared instance constructors and type initializers, including non-public declarations.</summary>
    public IReadOnlyList<MethodInfo> GetConstructors() => context.GetConstructors(definition, this);
    /// <summary>Gets declared metadata accessibility.</summary>
    public MetadataAccessibility Accessibility => MetadataAccess.Type(definition.Attributes);
    /// <summary>Gets the metadata abstract flag.</summary>
    public bool IsAbstract => (definition.Attributes & 0x80) != 0;
    /// <summary>Gets the metadata sealed flag.</summary>
    public bool IsSealed => (definition.Attributes & 0x100) != 0;
    /// <summary>Gets whether the declaration is an abstract sealed static container.</summary>
    public bool IsStatic => IsAbstract && IsSealed;
    /// <summary>Gets the metadata name, including generic arity suffix.</summary>
    public string Name => definition.Name;
    /// <summary>Gets the declared namespace.</summary>
    public string Namespace => definition.Namespace;
    /// <summary>Gets the nested qualified name; assembly scope is provided separately by Module.Assembly.</summary>
    public string FullName => DeclaringType is { } parent ? parent.FullName + "+" + Name : Namespace.Length == 0 ? Name : Namespace + "." + Name;
    /// <inheritdoc/>
    public override string DisplayName => FullName;
    /// <inheritdoc/>
    public override bool IsNominalType => true;
    /// <summary>Gets the reader's module-local definition token, including native-origin tokens.</summary>
    public uint MetadataToken => definition.MetadataToken;
    /// <summary>Gets the canonical declaring module.</summary>
    public ModuleInfo Module => context.RequireSnapshot(definition.Module.Assembly).GetModules()[0];
    /// <summary>Gets the enclosing declaration, or null for top-level types.</summary>
    public NominalTypeInfo? DeclaringType => definition.DeclaringType is { } parent ? context.GetType(parent) : null;
    /// <summary>Gets the declaration's generic parameter count, not constructed argument count.</summary>
    public int GenericArity => definition.GenericArity;
    /// <summary>Gets the metadata interface classification.</summary>
    public bool IsInterface => (definition.Attributes & 0x20) != 0;
    /// <summary>Gets the reader's nominal value-type classification.</summary>
    public bool IsValueType => definition.IsValueType;
}
