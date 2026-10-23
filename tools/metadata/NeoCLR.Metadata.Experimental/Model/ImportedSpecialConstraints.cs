using System.Collections.ObjectModel;

namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<ImportedTypeReference, bool> defaultConstructors = [];
    private readonly Dictionary<ImportedTypeReference, Dictionary<int, SignatureType>> importedBounds = [];

    /// <summary>Registers whether a dependency class is concrete with a public parameterless constructor.</summary>
    /// <remarks>The host must inspect the declaration, including abstractness and constructor accessibility. Identical facts are idempotent.</remarks>
    /// <exception cref="ArgumentException">Foreign, constructed, nested, value or interface owner.</exception>
    /// <exception cref="InvalidDataException">Conflicting previously registered fact.</exception>
    public void DeclareDefaultConstructor(ImportedTypeReference reference, bool available)
    {
        ArgumentNullException.ThrowIfNull(reference);
        if (!ReferenceEquals(reference.Owner, this) || reference.TypeArguments.Count != 0 || reference.DeclaringType is not null || reference.IsValueType || IsAuthoredInterface(reference))
            throw new ArgumentException("requires an owned reference class definition");
        if (defaultConstructors.TryGetValue(reference, out var previous) && previous != available)
            throw new InvalidDataException("conflicting default constructor contract");
        defaultConstructors[reference] = available;
    }
    internal bool HasDefaultConstructor(ImportedTypeReference reference) => defaultConstructors.GetValueOrDefault(InterfaceDefinition(reference));
    internal IReadOnlyDictionary<int, SignatureType> ImportedBounds(ImportedTypeReference reference)
        => new ReadOnlyDictionary<int, SignatureType>(importedBounds.TryGetValue(InterfaceDefinition(reference), out var bounds) ? bounds : new());

    /// <summary>Registers a resolved nongeneric nominal class or interface bound on an imported generic reference class.</summary>
    /// <remarks>Dependency declarations remain authoritative. Closed arguments use nominal ancestry; symbolic base arguments must prove the bound.</remarks>
    /// <exception cref="ArgumentException">Invalid owner, ordinal, bound or conflicting value constraint.</exception>
    /// <exception cref="InvalidDataException">Conflicting previously registered bound.</exception>
    public void AddBaseTypeConstraint(ImportedTypeReference reference, int parameterIndex, ImportedTypeReference baseType)
    {
        ArgumentNullException.ThrowIfNull(reference); ArgumentNullException.ThrowIfNull(baseType);
        if (!ReferenceEquals(reference.Owner, this) || reference.TypeArguments.Count != 0 || reference.DeclaringType is not null || reference.IsValueType || IsAuthoredInterface(reference) ||
            parameterIndex < 0 || parameterIndex >= reference.GenericArity || !ReferenceEquals(baseType.Owner, this) || baseType.GenericArity != 0 || baseType.IsValueType ||
            (reference.SpecialConstraints.GetValueOrDefault(parameterIndex).HasFlag(TypeParameterConstraints.ValueType) && !nominalKinds.GetValueOrDefault(InterfaceDefinition(baseType)) || reference.SpecialConstraints.GetValueOrDefault(parameterIndex).HasFlag(TypeParameterConstraints.FunctionType)))
            throw new ArgumentException("requires a generic reference class and nominal class or interface bound");
        if (!importedBounds.TryGetValue(reference, out var bounds)) importedBounds.Add(reference, bounds = []);
        if (bounds.TryGetValue(parameterIndex, out var previous) && previous != (SignatureType)baseType)
            throw new InvalidDataException("conflicting imported nominal bound");
        bounds[parameterIndex] = baseType;
    }

    private readonly Dictionary<ImportedTypeReference, Dictionary<int, TypeParameterConstraints>> importedSpecialConstraints = [];
    internal IReadOnlyDictionary<int, TypeParameterConstraints> ImportedSpecialConstraints(ImportedTypeReference reference)
        => new ReadOnlyDictionary<int, TypeParameterConstraints>(importedSpecialConstraints.TryGetValue(InterfaceDefinition(reference), out var constraints) ? constraints : new());

    /// <summary>Registers a resolved special requirement on an imported generic class or value definition.</summary>
    /// <param name="reference">Output-owned unconstructed top-level generic class or value type.</param>
    /// <param name="parameterIndex">Definition parameter ordinal.</param>
    /// <param name="constraints">ReferenceType, ValueType, DefaultConstructor or compatible combinations; native FunctionType alone.</param>
    /// <exception cref="ArgumentException">Wrong owner, ordinal, category or unsupported flags.</exception>
    /// <exception cref="InvalidDataException">A different requirement was already registered for this parameter.</exception>
    /// <remarks>The host supplies dependency facts without reader access. Repeating an identical registration is allowed.
    /// Constructions are checked immediately and again when used; symbolic base arguments must prove the requirement.
    /// Constructor facts must be registered for external arguments. The provider retains the actual metadata declaration.</remarks>
    public void SetSpecialConstraints(ImportedTypeReference reference, int parameterIndex, TypeParameterConstraints constraints)
    {
        ArgumentNullException.ThrowIfNull(reference);
        if (!ReferenceEquals(reference.Owner, this) || reference.TypeArguments.Count != 0 || reference.DeclaringType is not null ||
            IsAuthoredInterface(reference) || parameterIndex < 0 || parameterIndex >= reference.GenericArity ||
            constraints == TypeParameterConstraints.None || ((int)constraints & ~60) != 0 ||
            constraints.HasFlag(TypeParameterConstraints.FunctionType) && constraints != TypeParameterConstraints.FunctionType ||
            (constraints & (TypeParameterConstraints.ReferenceType | TypeParameterConstraints.ValueType)) == (TypeParameterConstraints.ReferenceType | TypeParameterConstraints.ValueType))
            throw new ArgumentException("requires an owned generic class or value definition and compatible special constraints");
        if ((constraints.HasFlag(TypeParameterConstraints.ValueType) || constraints.HasFlag(TypeParameterConstraints.FunctionType)) && ImportedBounds(reference).TryGetValue(parameterIndex, out var bound) && (constraints.HasFlag(TypeParameterConstraints.FunctionType) || bound.ImportedType is not { } importedBound || !IsAuthoredInterface(importedBound)))
            throw new ArgumentException("value constraint conflicts with nominal class or interface bound");
        if (!importedSpecialConstraints.TryGetValue(reference, out var registered))
            importedSpecialConstraints.Add(reference, registered = []);
        if (registered.TryGetValue(parameterIndex, out var existing) && existing != constraints)
            throw new InvalidDataException("conflicting imported special constraint contract");
        registered[parameterIndex] = constraints;
    }
}
