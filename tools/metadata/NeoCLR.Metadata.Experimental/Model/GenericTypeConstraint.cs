namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A nominal class bound for one declaring-type parameter.</summary>
public sealed record GenericTypeConstraint(int ParameterIndex, TypeBuilder BaseType);

/// <summary>Special type-parameter requirements; FunctionType is native-only.</summary>
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
    DefaultConstructor = 16,
    /// <summary>A native structural function signature; not a nominal delegate. Native emission only.</summary>
    FunctionType = 32
}

public sealed partial class TypeBuilder
{
    internal static bool ValidSpecialConstraintFlags(TypeParameterConstraints flags)
        => ((int)flags & ~60) == 0 &&
            (!flags.HasFlag(TypeParameterConstraints.FunctionType) || flags == TypeParameterConstraints.FunctionType) &&
            (flags & (TypeParameterConstraints.ReferenceType | TypeParameterConstraints.ValueType)) != (TypeParameterConstraints.ReferenceType | TypeParameterConstraints.ValueType);

    private Dictionary<int, TypeParameterConstraints> specialConstraints => Definition.SpecialConstraintStorage;
    /// <summary>Gets special requirements by parameter ordinal.</summary>
    public IReadOnlyDictionary<int, TypeParameterConstraints> SpecialConstraints => Definition.SpecialConstraints;
    /// <summary>Sets special requirements without changing nominal bounds.</summary>
    /// <param name="parameterIndex">Declared parameter ordinal.</param>
    /// <param name="constraints">ReferenceType, ValueType, DefaultConstructor or compatible combinations; native FunctionType alone. None clears flags.</param>
    /// <exception cref="ArgumentException">Invalid index/bits or conflicting reference/value/base requirements.</exception>
    public void SetSpecialConstraints(int parameterIndex, TypeParameterConstraints constraints)
    {
        if (parameterIndex < 0 || parameterIndex >= GenericParameterNames.Count || !ValidSpecialConstraintFlags(constraints) ||
            (constraints.HasFlag(TypeParameterConstraints.ValueType) || constraints.HasFlag(TypeParameterConstraints.FunctionType)) && genericConstraints.Any(c => c.ParameterIndex == parameterIndex))
            throw new ArgumentException("invalid or conflicting special constraints");
        if (constraints == TypeParameterConstraints.None) specialConstraints.Remove(parameterIndex);
        else specialConstraints[parameterIndex] = constraints;
    }
    private List<GenericTypeConstraint> genericConstraints => Definition.GenericConstraintStorage;
    /// <summary>Gets nominal bounds in parameter order.</summary>
    public IReadOnlyList<GenericTypeConstraint> GenericConstraints => Definition.GenericConstraints;
    /// <summary>Adds one owned nongeneric root-class bound for a declared parameter.</summary>
    /// <param name="parameterIndex">Zero-based declaring-type parameter index.</param>
    /// <param name="baseType">Owned nongeneric nonstatic class.</param>
    /// <exception cref="ArgumentNullException">Null bound.</exception>
    /// <exception cref="ArgumentException">Invalid index, duplicate parameter, static/generic or foreign bound.</exception>
    /// <remarks>CLI uses GenericParamConstraint, native uses TypeBound. No class/struct/new flags are implied. Existing uses are revalidated when writing.</remarks>
    public void AddBaseTypeConstraint(int parameterIndex, TypeBuilder baseType)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (parameterIndex < 0 || parameterIndex >= GenericParameterNames.Count || genericConstraints.Any(c => c.ParameterIndex == parameterIndex) || (specialConstraints.GetValueOrDefault(parameterIndex).HasFlag(TypeParameterConstraints.ValueType) || specialConstraints.GetValueOrDefault(parameterIndex).HasFlag(TypeParameterConstraints.FunctionType)) ||
            !ReferenceEquals(baseType.Assembly, Assembly) || baseType.IsStatic || baseType.IsInterface || baseType.IsValueType || baseType.GenericParameterNames.Count != 0)
            throw new ArgumentException("one owned nongeneric class bound per declared type parameter required");
        genericConstraints.Add(new(parameterIndex, baseType));
        genericConstraints.Sort((a, b) => a.ParameterIndex.CompareTo(b.ParameterIndex));
    }
    // A base declaration must prove symbolic requirements before any closed use.
    // Checking only closed construction arguments would admit invalid CLR metadata.
    internal void ValidateLocalBaseConstraints()
    {
        void Visit(SignatureType type)
        {
            if (type.GenericInstance is { } instance)
            {
                foreach (var (index, required) in instance.Definition.SpecialConstraints)
                {
                    if (instance.TypeArguments[index].TypeParameterIndex is not { } parameter) continue;
                    var actual = specialConstraints.GetValueOrDefault(parameter);
                    if (actual.HasFlag(TypeParameterConstraints.ValueType)) actual |= TypeParameterConstraints.DefaultConstructor;
                    if (genericConstraints.Any(c => c.ParameterIndex == parameter && !c.BaseType.IsNativeObjectRoot)) actual |= TypeParameterConstraints.ReferenceType;
                    if ((actual & required) != required) throw new InvalidDataException("base argument does not prove required special constraints");
                }
                foreach (var required in instance.Definition.GenericConstraints)
                    if (instance.TypeArguments[required.ParameterIndex].TypeParameterIndex is { } parameter &&
                        !genericConstraints.Any(c => c.ParameterIndex == parameter && ReferenceEquals(c.BaseType, required.BaseType)))
                        throw new InvalidDataException("base argument does not prove required nominal constraint");
                foreach (var argument in instance.TypeArguments) Visit(argument);
            }
            if (type.ImportedType is { } external)
            {
                foreach (var (index, required) in external.SpecialConstraints)
                {
                    if (external.TypeArguments[index].TypeParameterIndex is not { } parameter) continue;
                    var actual = specialConstraints.GetValueOrDefault(parameter);
                    if (actual.HasFlag(TypeParameterConstraints.ValueType)) actual |= TypeParameterConstraints.DefaultConstructor;
                    if (genericConstraints.Any(c => c.ParameterIndex == parameter && !c.BaseType.IsNativeObjectRoot)) actual |= TypeParameterConstraints.ReferenceType;
                    if ((actual & required) != required) throw new InvalidDataException("external base argument does not prove required special constraints");
                }
                foreach (var (index, bound) in external.TypeConstraints)
                    if (external.TypeArguments[index].TypeParameterIndex is { } parameter &&
                        !genericConstraints.Any(c => c.ParameterIndex == parameter && Assembly.HasVariantReferenceConversion(c.BaseType, bound)))
                        throw new InvalidDataException("external base argument does not prove nominal bound");
                foreach (var argument in external.TypeArguments) Visit(argument);
            }
            if (type.ArrayElement is { } element) Visit(element);
        }
        if (LocalBaseSignature is { } signature) Visit(signature);
        if (ExternalBase is { } external) Visit(external);
    }

    internal void ValidateTypeArguments(IReadOnlyList<SignatureType> arguments, bool complete = false)
    {
        foreach (var (index, flags) in specialConstraints)
        {
            var type = arguments[index];
            if (type.TypeParameterIndex is not null || type.MethodParameterIndex is not null) continue;
            ValidateSpecialTypeArgument(type, flags, complete);
        }
        foreach (var constraint in genericConstraints)
        {
            var type = arguments[constraint.ParameterIndex];
            // Symbolic arguments are checked after substitution by the runtime.
            if (type.TypeParameterIndex is not null || type.MethodParameterIndex is not null) continue;
            if (!Assembly.HasVariantReferenceConversion(type, constraint.BaseType))
                throw new ArgumentException("type argument does not satisfy the nominal class bound");
        }
    }
    internal static void ValidateSpecialTypeArgument(SignatureType type, TypeParameterConstraints flags, bool complete)
    {
        if (type.TypeParameterIndex is not null || type.MethodParameterIndex is not null) return;
        if (flags.HasFlag(TypeParameterConstraints.FunctionType) && type.FunctionSignature is null)
            throw new ArgumentException("type argument must be a structural function type");
        bool value = type.Primitive is PrimitiveType.Int32 or PrimitiveType.Int64 or PrimitiveType.Boolean or PrimitiveType.Byte or PrimitiveType.SByte or PrimitiveType.Int16 or PrimitiveType.UInt16 or PrimitiveType.UInt32 or PrimitiveType.UInt64 or PrimitiveType.IntPtr or PrimitiveType.UIntPtr or PrimitiveType.Single or PrimitiveType.Double ||
            type.ImportedType?.IsValueType == true || type.ClassType?.IsValueType == true || type.GenericInstance?.Definition.IsValueType == true;
        bool reference = type.Primitive == PrimitiveType.String || type.ClassType is { IsValueType: false } || type.ImportedType is { IsValueType: false } || type.GenericInstance?.Definition is { IsValueType: false } || type.ArrayElement is not null;
        var definition = type.ClassType ?? type.GenericInstance?.Definition;
        bool constructible = value || definition is { IsAbstract: false, IsInterface: false, IsStatic: false } && (!complete || definition.Methods.Any(m => m.IsConstructor && m.ParameterCount == 0 && m.Visibility == MethodVisibility.Public)) ||
            type.ImportedType is { } imported && imported.Owner.HasDefaultConstructor(imported);
        if (flags.HasFlag(TypeParameterConstraints.ReferenceType) && !reference || flags.HasFlag(TypeParameterConstraints.ValueType) && !value ||
            flags.HasFlag(TypeParameterConstraints.DefaultConstructor) && !constructible)
            throw new ArgumentException("type argument does not satisfy special constraints");
    }

}
