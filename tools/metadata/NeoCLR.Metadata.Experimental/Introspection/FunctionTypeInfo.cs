namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>A metadata-only callable signature, with owner/method arguments substituted.</summary>
public sealed class FunctionTypeInfo : TypeInfo
{
    internal FunctionTypeInfo(MetadataLoadContext context, TypeInfo result, TypeInfo[] parameters, bool noResult) : base(context)
    { ReturnType = result; ParameterTypes = Array.AsReadOnly(parameters); NoResult = noResult; }
    /// <summary>Gets the result type, or primitive Void for a no-result signature.</summary>
    public TypeInfo ReturnType { get; }
    /// <summary>Gets ordered value parameter types; does not expose callable targets or invocation.</summary>
    public IReadOnlyList<TypeInfo> ParameterTypes { get; }
    /// <summary>Gets whether invocation leaves no result, distinct from an inhabited unit result.</summary>
    public bool NoResult { get; }
    /// <inheritdoc/>
    public override bool IsNominalType => false;
    internal override int Depth => 1 + ParameterTypes.Append(ReturnType).Max(t => t.Depth);
    /// <inheritdoc/>
    public override string DisplayName => "(" + string.Join(", ", ParameterTypes.Select(t => t.DisplayName)) + ") -> " + (NoResult ? "noresult" : ReturnType.DisplayName);
}
