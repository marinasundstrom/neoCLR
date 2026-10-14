namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>The nominal membership operation of an experimental reference composition.</summary>
public enum ReferenceCompositionKind
{
    /// <summary>Membership in at least one constituent.</summary>
    Union,
    /// <summary>Membership in every constituent.</summary>
    Intersection
}

/// <summary>A read-only experimental native signature; it does not authorize compiler storage or emission.</summary>
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
    /// <summary>Gets the opt-in native reference composition, or null. Such signatures are read-only.</summary>
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
