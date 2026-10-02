namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable constructed nominal signature in a loaded definition snapshot.</summary>
/// <remarks>Import the containing declaration before emission. This is not an output-builder operand.</remarks>
public sealed class ReferencedGenericType : IEquatable<ReferencedGenericType>
{
    internal ReferencedGenericType(TypeReference definition, IEnumerable<SignatureType> arguments)
    { Definition = definition; TypeArguments = Array.AsReadOnly(arguments.ToArray()); }
    /// <summary>Gets the snapshot-scoped generic definition reference.</summary>
    public TypeReference Definition { get; }
    /// <summary>Gets immutable signature arguments in declaration order.</summary>
    public IReadOnlyList<SignatureType> TypeArguments { get; }
    /// <summary>Compares definition reference identity and argument values.</summary>
    /// <param name="other">Construction to compare; null is unequal.</param>
    /// <returns>True for the same snapshot reference and equal ordered arguments.</returns>
    public bool Equals(ReferencedGenericType? other) => other is not null && ReferenceEquals(Definition, other.Definition) && TypeArguments.SequenceEqual(other.TypeArguments);
    /// <summary>Compares construction identity.</summary>
    /// <param name="obj">Object to compare.</param>
    /// <returns>True only for an equal loaded construction.</returns>
    public override bool Equals(object? obj) => obj is ReferencedGenericType other && Equals(other);
    /// <summary>Combines definition identity and structural argument hashes.</summary>
    public override int GetHashCode()
    {
        var hash = new HashCode(); hash.Add(Definition);
        foreach (var argument in TypeArguments) hash.Add(argument);
        return hash.ToHashCode();
    }
    /// <summary>Returns a diagnostic name, not a serialized identity.</summary>
    public override string ToString() => Definition.Namespace + "." + Definition.Name + "<" + string.Join(", ", TypeArguments) + ">";
}
