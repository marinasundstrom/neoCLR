namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A nominal class bound for one declaring-type parameter.</summary>
public sealed record GenericTypeConstraint(int ParameterIndex, TypeBuilder BaseType);

public sealed partial class TypeBuilder
{
    private readonly List<GenericTypeConstraint> genericConstraints = [];
    /// <summary>Gets nominal bounds in parameter order.</summary>
    public IReadOnlyList<GenericTypeConstraint> GenericConstraints => genericConstraints.AsReadOnly();
    /// <summary>Adds one owned nongeneric root-class bound for a declared parameter.</summary>
    /// <param name="parameterIndex">Zero-based declaring-type parameter index.</param>
    /// <param name="baseType">Owned nongeneric nonstatic class.</param>
    /// <exception cref="ArgumentNullException">Null bound.</exception>
    /// <exception cref="ArgumentException">Invalid index, duplicate parameter, static/generic or foreign bound.</exception>
    /// <remarks>CLI uses GenericParamConstraint, native uses TypeBound. No class/struct/new flags are implied. Existing uses are revalidated when writing.</remarks>
    public void AddBaseTypeConstraint(int parameterIndex, TypeBuilder baseType)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (parameterIndex < 0 || parameterIndex >= GenericParameterNames.Count || genericConstraints.Any(c => c.ParameterIndex == parameterIndex) ||
            !ReferenceEquals(baseType.Assembly, Assembly) || baseType.IsStatic || baseType.GenericParameterNames.Count != 0)
            throw new ArgumentException("one owned nongeneric class bound per declared type parameter required");
        genericConstraints.Add(new(parameterIndex, baseType));
        genericConstraints.Sort((a, b) => a.ParameterIndex.CompareTo(b.ParameterIndex));
    }
    internal void ValidateTypeArguments(IReadOnlyList<SignatureType> arguments)
    {
        foreach (var constraint in genericConstraints)
        {
            var type = arguments[constraint.ParameterIndex];
            // Symbolic arguments are checked after substitution by the runtime.
            if (type.TypeParameterIndex is not null || type.MethodParameterIndex is not null) continue;
            if (!ReferenceEquals(type.ClassType, constraint.BaseType))
                throw new ArgumentException("type argument does not satisfy the nominal class bound");
        }
    }
}
