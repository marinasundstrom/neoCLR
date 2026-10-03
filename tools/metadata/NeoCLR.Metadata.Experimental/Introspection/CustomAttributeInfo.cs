using NeoCLR.Metadata.Experimental.Model;

namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>A metadata-only attribute view. No constructor is executed.</summary>
public sealed class CustomAttributeInfo
{
    private readonly MetadataLoadContext context;
    private readonly CustomAttributeDefinition definition;
    internal CustomAttributeInfo(MetadataLoadContext context, CustomAttributeDefinition definition)
    { this.context = context; this.definition = definition; }
    /// <summary>Gets the stored attribute type namespace without resolving its dependency.</summary>
    public string Namespace => definition.AttributeType.Namespace;
    /// <summary>Gets the stored attribute type name without resolving its dependency.</summary>
    public string Name => definition.AttributeType.Name;
    /// <summary>Resolves the exact attribute owner through this view's explicit metadata catalog.</summary>
    /// <exception cref="InvalidDataException">The declared dependency or type is absent or mismatched.</exception>
    public NominalTypeInfo GetAttributeType() => context.Resolve(definition.AttributeType);
    /// <summary>Gets supported immutable fixed arguments without invoking the constructor.</summary>
    /// <exception cref="NotSupportedException">A fixed/named argument category is unsupported.</exception>
    /// <exception cref="InvalidDataException">Supported metadata is malformed.</exception>
    public IReadOnlyList<CustomAttributeArgument> GetArguments() => definition.GetArguments();
}
