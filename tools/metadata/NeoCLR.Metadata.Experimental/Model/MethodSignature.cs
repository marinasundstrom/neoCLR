namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable primitive or owned root-class signature type.</summary>
public sealed record SignatureType
{
    private SignatureType(PrimitiveType? primitive, TypeBuilder? classType) { Primitive = primitive; ClassType = classType; }
    /// <summary>Gets the primitive kind, or null for a class reference.</summary>
    public PrimitiveType? Primitive { get; }
    /// <summary>Gets the exact owned class identity, or null for a primitive.</summary>
    public TypeBuilder? ClassType { get; }
    /// <summary>Creates a primitive signature type, including Void for results only.</summary>
    public static implicit operator SignatureType(PrimitiveType type)
        => Enum.IsDefined(type) ? new(type, null) : throw new ArgumentOutOfRangeException(nameof(type));
    /// <summary>Creates a reference to a nonstatic root class.</summary>
    public static implicit operator SignatureType(TypeBuilder type)
    {
        ArgumentNullException.ThrowIfNull(type);
        if (type.IsStatic) throw new ArgumentException("signature class must be a nonstatic root", nameof(type));
        return new(null, type);
    }
    /// <summary>Returns a diagnostic name; it is not a serialized type identity.</summary>
    public override string ToString() => Primitive?.ToString() ?? ClassType!.Namespace + "." + ClassType.Name;
}

/// <summary>An immutable nongeneric signature with primitive or owned nominal parameters/results.</summary>
public class MethodSignature
{
    /// <summary>Copies up to 256 non-Void parameters. Ownership is checked when defining a method.</summary>
    /// <param name="returnType">Primitive, Void or nonstatic owned class.</param>
    /// <param name="parameterTypes">Copied non-Void parameter sequence.</param>
    /// <exception cref="ArgumentNullException">Result or parameters are null.</exception>
    /// <exception cref="ArgumentException">Null/Void parameter or more than 256 parameters.</exception>
    public MethodSignature(SignatureType returnType, IEnumerable<SignatureType> parameterTypes)
    {
        ArgumentNullException.ThrowIfNull(returnType);
        ArgumentNullException.ThrowIfNull(parameterTypes);
        var parameters = parameterTypes.Take(257).ToArray();
        if (parameters.Length > 256 || parameters.Any(p => p is null || p.Primitive == PrimitiveType.Void))
            throw new ArgumentException("invalid parameter signature", nameof(parameterTypes));
        ReturnType = returnType; ParameterTypes = Array.AsReadOnly(parameters);
    }
    /// <summary>Creates a primitive-only signature.</summary>
    public MethodSignature(PrimitiveType returnType, IEnumerable<PrimitiveType> parameterTypes)
        : this((SignatureType)returnType, (parameterTypes ?? throw new ArgumentNullException(nameof(parameterTypes))).Select(p => (SignatureType)p)) { }
    /// <summary>Gets the result type; Void denotes no result.</summary>
    public SignatureType ReturnType { get; }
    /// <summary>Gets declared parameters, excluding the receiver.</summary>
    public IReadOnlyList<SignatureType> ParameterTypes { get; }
    internal void ValidateOwner(AssemblyBuilder assembly)
    {
        if (ParameterTypes.Append(ReturnType).Any(t => t.ClassType is { } c && !ReferenceEquals(c.Assembly, assembly)))
            throw new ArgumentException("nominal signatures must use classes owned by the output assembly");
    }
    internal bool Matches(MethodSignature other) => ReturnType == other.ReturnType && ParameterTypes.SequenceEqual(other.ParameterTypes);
}
