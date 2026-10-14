namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>The nominal membership operation of an experimental reference composition.</summary>
public enum ReferenceCompositionKind
{
    /// <summary>Membership in at least one constituent.</summary>
    Union,
    /// <summary>Membership in every constituent.</summary>
    Intersection
}

/// <summary>An immutable experimental native signature; storage requires explicit imported-contract admission.</summary>
public sealed class ReferenceCompositionSignature : IEquatable<ReferenceCompositionSignature>
{
    internal ReferenceCompositionSignature(ReferenceCompositionKind kind, SignatureType[] constituents)
    {
        Kind = kind;
        Constituents = Array.AsReadOnly(constituents);
    }
    /// <summary>Gets either-membership (Union) or all-membership (Intersection).</summary>
    public ReferenceCompositionKind Kind { get; }
    /// <summary>Gets normalized bounds in first-occurrence order; identity ignores this order.</summary>
    public IReadOnlyList<SignatureType> Constituents { get; }
    /// <summary>Compares kind and constituent identity without nominal-subtype reduction.</summary>
    public bool Equals(ReferenceCompositionSignature? other) => other is not null && Kind == other.Kind &&
        Constituents.Count == other.Constituents.Count && Constituents.ToHashSet().SetEquals(other.Constituents);
    /// <inheritdoc/>
    public override bool Equals(object? obj) => obj is ReferenceCompositionSignature other && Equals(other);
    /// <inheritdoc/>
    public override int GetHashCode()
    {
        var hash = 0;
        foreach (var type in Constituents) hash = unchecked(hash + type.GetHashCode());
        return HashCode.Combine(Kind, Constituents.Count, hash);
    }
    /// <summary>Returns a diagnostic display, not a serialized identity.</summary>
    public override string ToString() => Kind + "<" + string.Join(", ", Constituents) + ">";
}

public sealed partial record SignatureType
{
    /// <summary>Gets the opt-in native reference composition, or null. Storage requires explicit imported-contract admission.</summary>
    public ReferenceCompositionSignature? Composition { get; }
    internal bool ContainsComposition => Composition is not null ||
        GenericInstance?.TypeArguments.Any(t => t.ContainsComposition) == true ||
        ReferencedGenericInstance?.TypeArguments.Any(t => t.ContainsComposition) == true ||
        ImportedType?.TypeArguments.Any(t => t.ContainsComposition) == true ||
        ArrayElement?.ContainsComposition == true || ByReferenceElement?.ContainsComposition == true ||
        PointerElement?.ContainsComposition == true || FunctionSignature is { } function &&
        function.ParameterTypes.Append(function.ReturnType).Any(t => t.ContainsComposition);

    internal static SignatureType FromComposition(ReferenceCompositionKind kind, IEnumerable<SignatureType> types)
    {
        var flattened = types.SelectMany(t => t.Composition is { } c && c.Kind == kind ? c.Constituents : new[] { t }).Distinct().ToArray();
        return flattened.Length == 1 ? flattened[0] : new(null, null, composition: new(kind, flattened));
    }
}


public sealed partial class AssemblyBuilder
{
    private readonly HashSet<ReferenceCompositionSignature> experimentalStorageCompositions = [];
    internal bool AdmitsCompositionStorage(ReferenceCompositionSignature composition) => experimentalStorageCompositions.Contains(composition);

    /// <summary>Admits the closed reference compositions in an owned imported result for native storage and generic substitution.</summary>
    /// <param name="producer">An experimental imported callable owned by this output.</param>
    /// <exception cref="ArgumentNullException">Producer is null.</exception>
    /// <exception cref="ArgumentException">Producer is foreign or its result contains no composition.</exception>
    /// <remarks>CLI projection remains disabled. The imported exact constituent contracts are retained; arbitrary structural authoring is not enabled.</remarks>
    public void EnableExperimentalCompositionStorage(ImportedMethodReference producer)
    {
        ArgumentNullException.ThrowIfNull(producer);
        if (!ReferenceEquals(producer.Owner, this) || !producer.Signature.ReturnType.ContainsComposition)
            throw new ArgumentException("requires an owned imported composition result", nameof(producer));
        RegisterCompositionStorage(producer.Signature.ReturnType);
    }
    internal void RegisterCompositionStorage(SignatureType type)
    {
        if (type.Composition is { } composition)
        {
            experimentalStorageCompositions.Add(composition);
            foreach (var bound in composition.Constituents) RegisterCompositionStorage(bound);
        }
        if (type.ImportedType is { } imported) foreach (var argument in imported.TypeArguments) RegisterCompositionStorage(argument);
        if (type.GenericInstance is { } instance) foreach (var argument in instance.TypeArguments) RegisterCompositionStorage(argument);
    }
}
