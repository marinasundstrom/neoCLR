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
    private readonly Lazy<IReadOnlyList<NominalTypeInfo>> types;
    internal ModuleInfo(MetadataLoadContext context, AssemblyInfo assembly, ModuleDefinition definition)
    {
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
}

/// <summary>The initial metadata-only type facade; additional type families will extend this model.</summary>
/// <remarks>This prototype deliberately exposes a subset of runtime System.Introspection.TypeInfo.
/// Missing capabilities are not fabricated as empty collections or runtime handles.</remarks>
public abstract class TypeInfo
{
    private protected TypeInfo() { }
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
    internal NominalTypeInfo(MetadataLoadContext context, TypeDefinition definition)
    { this.context = context; this.definition = definition; }
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
