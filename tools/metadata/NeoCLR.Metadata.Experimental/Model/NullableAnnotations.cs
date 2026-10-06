using System.Collections.ObjectModel;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Immutable explicit NullableAttribute payload, separate from the physical type signature.</summary>
public sealed class NullableAnnotation
{
    /// <summary>Copies bounded CLI flags. Uniform scalar attributes apply one flag to all nullable positions.</summary>
    /// <exception cref="ArgumentException">Flags are empty, exceed 4096, contain values above 2, or a uniform payload is not scalar.</exception>
    public NullableAnnotation(IEnumerable<byte> flags, bool isUniform = false)
    {
        ArgumentNullException.ThrowIfNull(flags);
        var copy = flags.Take(4097).ToArray();
        if (copy.Length is 0 or > 4096 || copy.Any(f => f > 2) || isUniform && copy.Length != 1)
            throw new ArgumentException("invalid nullable annotation flags", nameof(flags));
        Flags = Array.AsReadOnly(copy); IsUniform = isUniform;
    }
    /// <summary>Gets copied flags in .NET nullable transform order: 0 oblivious, 1 non-null, 2 nullable.</summary>
    public IReadOnlyList<byte> Flags { get; }
    /// <summary>True for the scalar-byte constructor; false for the positional byte-array constructor.</summary>
    public bool IsUniform { get; }
}

public sealed partial class MethodDefinition
{
    private readonly Dictionary<int, NullableAnnotation> nullableAnnotations = [];
    /// <summary>Explicit CLI NullableAttribute flags by parameter index; -1 denotes the return value.</summary>
    /// <remarks>Flags use the .NET nullable transform order: 0 oblivious, 1 non-null, 2 nullable.
    /// These annotations do not change signatures, storage or runtime checks. Absence is not a non-null assertion.</remarks>
    public IReadOnlyDictionary<int, NullableAnnotation> NullableAnnotations => new ReadOnlyDictionary<int, NullableAnnotation>(nullableAnnotations);

    /// <summary>Sets explicit nullable type-use flags, or clears them with null. Use -1 for the return value.</summary>
    /// <remarks>The caller supplies the NullableAttribute transform vector, including nested reference positions.
    /// Shape interpretation belongs to the consuming type system. Context compression is not authored.</remarks>
    /// <exception cref="InvalidOperationException">The method is not an attached authored definition.</exception>
    /// <exception cref="ArgumentException">The position is outside the return and parameter range.</exception>
    public void SetNullableAnnotation(int position, NullableAnnotation? annotation)
    {
        if (Producer is null) throw new InvalidOperationException("attach an authored method before changing parameter metadata");
        if (position < -1 || position >= Producer.ParameterCount) throw new ArgumentOutOfRangeException(nameof(position));
        if (annotation is null) nullableAnnotations.Remove(position);
        else nullableAnnotations[position] = annotation;
    }

    internal void LoadNullableAnnotations(IReadOnlyDictionary<int, NullableAnnotation>? annotations)
    {
        if (annotations is null) return;
        foreach (var pair in annotations)
            nullableAnnotations.Add(pair.Key, pair.Value);
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Sets or clears explicit CLI nullable flags through the owned definition; -1 denotes the return value.</summary>
    public void SetNullableAnnotation(int position, NullableAnnotation? annotation) => Definition.SetNullableAnnotation(position, annotation);
}
