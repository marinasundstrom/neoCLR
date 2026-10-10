using NeoCLR.Metadata.Experimental.Model;

namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>A metadata-only assembly-level member with assembly ownership and a namespace-qualified name.</summary>
/// <remarks>Type and Function views are resolved lazily. Constants currently support finite Double only.
/// No runtime member invocation, storage or assembly loading is performed.</remarks>
public sealed class AssemblyMemberInfo
{
    private readonly MetadataLoadContext context;
    private readonly AssemblyMemberDefinition definition;
    internal AssemblyMemberInfo(MetadataLoadContext context, AssemblyInfo assembly, AssemblyMemberDefinition definition)
    {
        this.context = context;
        this.definition = definition;
        Assembly = assembly;
    }
    /// <summary>Gets the member kind.</summary>
    public AssemblyMemberKind Kind => definition.Kind;
    /// <summary>Gets the declaring assembly facade.</summary>
    public AssemblyInfo Assembly { get; }
    /// <summary>Gets the canonical logical module directly owning this member.</summary>
    public ModuleInfo Module => context.Resolve(definition.DeclaringModule);
    /// <summary>Gets the namespace portion of the member name.</summary>
    public string Namespace => definition.Namespace;
    /// <summary>Gets the simple metadata name.</summary>
    public string Name => definition.Name;
    /// <summary>Gets the qualified member name, excluding assembly identity and overload signature.</summary>
    public string FullName => definition.FullName;
    /// <summary>Gets the nominal type view for a Type member; otherwise null.</summary>
    public NominalTypeInfo? Type => definition.Type is { } type ? context.GetType(type) : null;
    /// <summary>Gets the method view for a Function member; otherwise null.</summary>
    public MethodInfo? Function => definition.Function is { } function ? context.Resolve(function) : null;
    /// <summary>Gets the exact Double value for a Constant member; otherwise null.</summary>
    public double? ConstantValue => definition.Constant?.Value;
}
