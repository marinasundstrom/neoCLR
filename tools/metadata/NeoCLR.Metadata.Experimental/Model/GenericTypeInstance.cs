namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable owned generic nominal construction with structural argument identity.</summary>
public sealed class GenericTypeInstance : IEquatable<GenericTypeInstance>
{
    internal GenericTypeInstance(TypeBuilder definition, SignatureType[] arguments)
    { Definition = definition; TypeArguments = Array.AsReadOnly(arguments); }
    /// <summary>Gets the owned generic nominal definition.</summary>
    public TypeBuilder Definition { get; }
    /// <summary>Gets copied arguments in declaration order.</summary>
    public IReadOnlyList<SignatureType> TypeArguments { get; }
    internal bool ConformsTo(SignatureType contract)
    {
        SignatureType Substitute(SignatureType type) => type.FunctionSignature is { } function ? function.Substitute(Substitute)
            : type.ByReferenceElement is { } byref ? SignatureType.ByReference(Substitute(byref))
            : type.ImportedType is { } imported ? imported.Substitute(Substitute)
            : type.TypeParameterIndex is { } index ? TypeArguments[index]
            : type.GenericInstance is { } nested ? nested.Definition.MakeGenericInstance(nested.TypeArguments.Select(Substitute).ToArray())
            : type.ArrayElement is { } element ? SignatureType.ArrayOf(Substitute(element)) : type;
        return Definition.InheritedContracts().Any(inherited => Substitute(inherited) == contract);
    }
    /// <summary>Compares definition identity and argument values.</summary>
    public bool Equals(GenericTypeInstance? other) => other is not null && ReferenceEquals(Definition, other.Definition) && TypeArguments.SequenceEqual(other.TypeArguments);
    /// <summary>Compares construction identity.</summary>
    public override bool Equals(object? obj) => obj is GenericTypeInstance other && Equals(other);
    /// <summary>Gets a hash consistent with construction identity.</summary>
    public override int GetHashCode()
    { var hash = new HashCode(); hash.Add(Definition); foreach (var type in TypeArguments) hash.Add(type); return hash.ToHashCode(); }
    /// <summary>Returns a diagnostic name, not serialized metadata identity.</summary>
    public override string ToString() => Definition.Namespace + "." + Definition.Name + "<" + string.Join(",", TypeArguments) + ">";
}

public sealed partial class TypeBuilder
{
    /// <summary>Constructs an immutable nominal signature; caller scope is checked when used.</summary>
    /// <param name="typeArguments">One non-Void supported argument per parameter; copied.</param>
    /// <returns>A nominal construction with structural argument identity.</returns>
    /// <exception cref="ArgumentNullException">Null argument array.</exception>
    /// <exception cref="ArgumentException">Static/nongeneric owner, wrong arity, null/Void/foreign argument or nesting beyond 16 levels.</exception>
    public GenericTypeInstance MakeGenericInstance(params SignatureType[] typeArguments)
    {
        ArgumentNullException.ThrowIfNull(typeArguments);
        if (IsStatic || GenericParameterNames.Count == 0 || typeArguments.Length != GenericParameterNames.Count || typeArguments.Any(t => t is null || t.Primitive == PrimitiveType.Void || t.NestingDepth >= 16))
            throw new ArgumentException("generic type arguments must match the definition");
        foreach (var argument in typeArguments) argument.ValidateOwner(Assembly, 32, 32, allowSelf: true);
        ValidateTypeArguments(typeArguments);
        return new(this, (SignatureType[])typeArguments.Clone());
    }
    internal SignatureType OpenSignature => GenericParameterNames.Count == 0 ? (SignatureType)this : MakeGenericInstance(Enumerable.Range(0, GenericParameterNames.Count).Select(SignatureType.TypeParameter).ToArray());
}
