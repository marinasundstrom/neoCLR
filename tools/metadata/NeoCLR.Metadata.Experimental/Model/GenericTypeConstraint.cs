namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A nominal class bound for one declaring-type parameter.</summary>
public sealed record GenericTypeConstraint(int ParameterIndex, TypeBuilder BaseType);

/// <summary>Supported CLI-compatible special type-parameter requirements.</summary>
[Flags]
public enum TypeParameterConstraints
{
    /// <summary>No special requirement.</summary>
    None = 0,
    /// <summary>An ordinary managed reference type.</summary>
    ReferenceType = 4,
    /// <summary>A nonnullable value type.</summary>
    ValueType = 8,
    /// <summary>A value type or concrete class with a public parameterless constructor.</summary>
    DefaultConstructor = 16
}

public sealed partial class TypeBuilder
{
    private readonly Dictionary<int, TypeParameterConstraints> specialConstraints = [];
    /// <summary>Gets special requirements by parameter ordinal.</summary>
    public IReadOnlyDictionary<int, TypeParameterConstraints> SpecialConstraints => new System.Collections.ObjectModel.ReadOnlyDictionary<int, TypeParameterConstraints>(specialConstraints);
    /// <summary>Sets special requirements without changing nominal bounds.</summary>
    /// <param name="parameterIndex">Declared parameter ordinal.</param>
    /// <param name="constraints">ReferenceType, ValueType, DefaultConstructor, or compatible combinations; None clears flags.</param>
    /// <exception cref="ArgumentException">Invalid index/bits or conflicting reference/value/base requirements.</exception>
    public void SetSpecialConstraints(int parameterIndex, TypeParameterConstraints constraints)
    {
        if (parameterIndex < 0 || parameterIndex >= GenericParameterNames.Count || ((int)constraints & ~28) != 0 ||
            (constraints & (TypeParameterConstraints.ReferenceType | TypeParameterConstraints.ValueType)) == (TypeParameterConstraints.ReferenceType | TypeParameterConstraints.ValueType) ||
            constraints.HasFlag(TypeParameterConstraints.ValueType) && genericConstraints.Any(c => c.ParameterIndex == parameterIndex))
            throw new ArgumentException("invalid or conflicting special constraints");
        if (constraints == TypeParameterConstraints.None) specialConstraints.Remove(parameterIndex);
        else specialConstraints[parameterIndex] = constraints;
    }
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
        if (parameterIndex < 0 || parameterIndex >= GenericParameterNames.Count || genericConstraints.Any(c => c.ParameterIndex == parameterIndex) || specialConstraints.GetValueOrDefault(parameterIndex).HasFlag(TypeParameterConstraints.ValueType) ||
            !ReferenceEquals(baseType.Assembly, Assembly) || baseType.IsStatic || baseType.GenericParameterNames.Count != 0)
            throw new ArgumentException("one owned nongeneric class bound per declared type parameter required");
        genericConstraints.Add(new(parameterIndex, baseType));
        genericConstraints.Sort((a, b) => a.ParameterIndex.CompareTo(b.ParameterIndex));
    }
    internal void ValidateTypeArguments(IReadOnlyList<SignatureType> arguments, bool complete = false)
    {
        foreach (var (index, flags) in specialConstraints)
        {
            var type = arguments[index];
            if (type.TypeParameterIndex is not null || type.MethodParameterIndex is not null) continue;
            bool value = type.Primitive is PrimitiveType.Int32 or PrimitiveType.Int64 or PrimitiveType.Boolean;
            bool reference = type.Primitive == PrimitiveType.String || type.ClassType is not null || type.GenericInstance is not null || type.ArrayElement is not null;
            var definition = type.ClassType ?? type.GenericInstance?.Definition;
            bool constructible = value || definition is not null && (!complete || definition.Methods.Any(m => m.IsConstructor && m.ParameterCount == 0 && m.Visibility == MethodVisibility.Public));
            if (flags.HasFlag(TypeParameterConstraints.ReferenceType) && !reference || flags.HasFlag(TypeParameterConstraints.ValueType) && !value ||
                flags.HasFlag(TypeParameterConstraints.DefaultConstructor) && !constructible)
                throw new ArgumentException("type argument does not satisfy special constraints");
        }
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
