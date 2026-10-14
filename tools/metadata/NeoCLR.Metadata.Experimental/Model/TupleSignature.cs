namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An ordered, immutable native structural tuple signature with value semantics.</summary>
public sealed class TupleSignature : IEquatable<TupleSignature>
{
    internal TupleSignature(SignatureType[] elements) => ElementTypes = Array.AsReadOnly(elements.ToArray());
    /// <summary>Gets positional element signatures, preserving duplicates and nesting.</summary>
    public IReadOnlyList<SignatureType> ElementTypes { get; }
    /// <summary>Compares elements in order; a one-element tuple is not its element.</summary>
    public bool Equals(TupleSignature? other) => other is not null && ElementTypes.SequenceEqual(other.ElementTypes);
    /// <inheritdoc/>
    public override bool Equals(object? obj) => obj is TupleSignature other && Equals(other);
    /// <inheritdoc/>
    public override int GetHashCode()
    {
        var hash = new HashCode();
        foreach (var element in ElementTypes) hash.Add(element);
        return hash.ToHashCode();
    }
    /// <summary>Returns a diagnostic display, not a serialized identity.</summary>
    public override string ToString() => "tuple<" + string.Join(", ", ElementTypes) + ">";
}

public sealed partial record SignatureType
{
    /// <summary>Gets the native structural tuple shape, or null for another signature kind.</summary>
    public TupleSignature? Tuple { get; }
}

public sealed partial class AssemblyBuilder
{
    private readonly HashSet<TupleSignature> experimentalTuples = [];
    internal bool AdmitsTuple(TupleSignature tuple) => experimentalTuples.Contains(tuple);
    private void ValidateAuthoredTuples()
    {
        foreach (var tuple in experimentalTuples.ToArray()) CreateExperimentalTuple(tuple.ElementTypes.ToArray());
    }

    /// <summary>Creates and admits a closed structural tuple for native signatures and storage.</summary>
    /// <param name="elements">One to 256 ordered elements, copied by this method.</param>
    /// <returns>A structural value signature without a nominal declaration.</returns>
    /// <exception cref="ArgumentNullException">The element array is null.</exception>
    /// <exception cref="ArgumentException">An element is foreign, open, unsupported, or exceeds resource limits.</exception>
    /// <remarks>Native-only. Supports scalars, closed nominal types and nested tuples; excludes Void, erased Value, handles, borrows, arrays, pointers, functions and compositions. Maximum nesting is 16 and the validation budget is 4096 nodes.</remarks>
    public SignatureType CreateExperimentalTuple(params SignatureType[] elements)
    {
        ArgumentNullException.ThrowIfNull(elements);
        if (elements.Length is < 1 or > 256) throw new ArgumentException("tuple requires one to 256 elements", nameof(elements));
        var nodes = 0;
        void Validate(SignatureType type)
        {
            if (type is null || ++nodes > 4096 || type.NestingDepth >= 16)
                throw new ArgumentException("tuple requires bounded closed elements", nameof(elements));
            type.ValidateOwner(this);
            if (type.Tuple is { } tuple) { foreach (var element in tuple.ElementTypes) Validate(element); return; }
            if (type.Primitive is { } primitive && Enum.IsDefined(primitive) && primitive is not (PrimitiveType.Void or PrimitiveType.Value or PrimitiveType.RuntimeTypeHandle)) return;
            if (type.ClassType is { IsStatic: false, GenericParameterNames.Count: 0 } local && local.NativePrimitive is not (PrimitiveType.Void or PrimitiveType.Value or PrimitiveType.RuntimeTypeHandle)) return;
            if (type.GenericInstance is { Definition.IsStatic: false } instance) { foreach (var argument in instance.TypeArguments) Validate(argument); return; }
            if (type.ImportedType is { } imported && imported.GenericArity == imported.TypeArguments.Count && !NativeImportBinding.IsErasedValue(imported) && AuthoredPrimitiveOwner(imported) is not (PrimitiveType.Void or PrimitiveType.RuntimeTypeHandle)) { foreach (var argument in imported.TypeArguments) Validate(argument); return; }
            throw new ArgumentException("tuple requires scalar, closed nominal or nested tuple elements", nameof(elements));
        }
        foreach (var element in elements) Validate(element);
        var shape = new TupleSignature(elements);
        experimentalTuples.Add(shape);
        return SignatureType.FromTuple(shape);
    }
}
