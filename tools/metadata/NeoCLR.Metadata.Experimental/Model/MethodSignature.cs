namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable primitive, owned class/construction, scoped generic parameter or vector signature type.</summary>
public sealed record SignatureType
{
    private SignatureType(PrimitiveType? primitive, TypeBuilder? classType, SignatureType? arrayElement = null, int? methodParameter = null, int? typeParameter = null, GenericTypeInstance? genericInstance = null) { GenericInstance = genericInstance; Primitive = primitive; ClassType = classType; ArrayElement = arrayElement; MethodParameterIndex = methodParameter; TypeParameterIndex = typeParameter; }
    /// <summary>Gets the primitive kind, or null for a class or vector reference.</summary>
    public PrimitiveType? Primitive { get; }
    /// <summary>Gets the exact owned class identity, or null for a primitive or vector.</summary>
    public TypeBuilder? ClassType { get; }
    /// <summary>Gets the element type for a zero-based vector, or null for a scalar.</summary>
    public SignatureType? ArrayElement { get; }
    /// <summary>Creates a one-dimensional zero-based vector of primitives, owned root classes or method parameters.</summary>
    /// <param name="elementType">Non-Void scalar type; nested and multidimensional arrays are not admitted.</param>
    /// <exception cref="ArgumentNullException">Element is null.</exception>
    /// <exception cref="ArgumentException">Element is Void, another array, or exceeds the 16-level nesting bound.</exception>
    public static SignatureType ArrayOf(SignatureType elementType)
    {
        ArgumentNullException.ThrowIfNull(elementType);
        if (elementType.Primitive == PrimitiveType.Void || elementType.ArrayElement is not null || elementType.NestingDepth >= 16)
            throw new ArgumentException("array element must be a supported scalar", nameof(elementType));
        return new(null, null, elementType);
    }
    /// <summary>Gets the positional method generic parameter, or null for other types.</summary>
    public int? MethodParameterIndex { get; }
    /// <summary>Creates an MVAR reference, scoped by the containing method signature.</summary>
    /// <param name="index">Zero-based index, from 0 through 31; checked against the declaring arity on use.</param>
    /// <exception cref="ArgumentOutOfRangeException">Index is outside the supported range.</exception>
    public static SignatureType MethodParameter(int index)
        => index is >= 0 and < 32 ? new(null, null, methodParameter: index) : throw new ArgumentOutOfRangeException(nameof(index));
    /// <summary>Gets the positional declaring-type parameter, or null.</summary>
    public int? TypeParameterIndex { get; }
    /// <summary>Creates a VAR reference scoped to a generic declaring type.</summary>
    /// <param name="index">Zero-based index, 0 through 31.</param>
    /// <exception cref="ArgumentOutOfRangeException">Index outside the supported range.</exception>
    public static SignatureType TypeParameter(int index)
        => index is >= 0 and < 32 ? new(null, null, typeParameter: index) : throw new ArgumentOutOfRangeException(nameof(index));
    /// <summary>Gets the constructed class identity, or null for other signatures.</summary>
    public GenericTypeInstance? GenericInstance { get; }
    /// <summary>Creates a signature for an owned constructed generic reference class.</summary>
    public static implicit operator SignatureType(GenericTypeInstance type)
        => new(null, null, genericInstance: type ?? throw new ArgumentNullException(nameof(type)));
    internal int NestingDepth => GenericInstance is { } instance ? 1 + instance.TypeArguments.Max(t => t.NestingDepth) : ArrayElement is { } element ? 1 + element.NestingDepth : 0;
    internal void ValidateOwner(AssemblyBuilder assembly, int genericArity = 0, int typeArity = 0)
    {
        if (MethodParameterIndex is { } index && index >= genericArity) throw new ArgumentException("method type parameter outside declared scope");
        if (TypeParameterIndex is { } ordinal && ordinal >= typeArity) throw new ArgumentException("type parameter outside declared scope");
        ArrayElement?.ValidateOwner(assembly, genericArity, typeArity);
        if (GenericInstance is { } instance)
        {
            if (!ReferenceEquals(instance.Definition.Assembly, assembly)) throw new ArgumentException("foreign constructed class");
            foreach (var argument in instance.TypeArguments) argument.ValidateOwner(assembly, genericArity, typeArity);
        }
        if ((ArrayElement?.ClassType ?? ClassType) is { } owner && !ReferenceEquals(owner.Assembly, assembly))
            throw new ArgumentException("signature requires a class owned by the output assembly");
    }
    /// <summary>Creates a primitive signature type, including Void for results only.</summary>
    public static implicit operator SignatureType(PrimitiveType type)
        => Enum.IsDefined(type) ? new(type, null) : throw new ArgumentOutOfRangeException(nameof(type));
    /// <summary>Creates a reference to a nongeneric nonstatic root class; generic definitions require construction.</summary>
    public static implicit operator SignatureType(TypeBuilder type)
    {
        ArgumentNullException.ThrowIfNull(type);
        if (type.IsStatic || type.GenericParameterNames.Count > 0) throw new ArgumentException("signature class must be a nonstatic root", nameof(type));
        return new(null, type);
    }
    /// <summary>Returns a diagnostic name; it is not a serialized type identity.</summary>
    public override string ToString() => GenericInstance is { } instance ? instance.ToString() : ArrayElement is { } element ? element + "[]" : MethodParameterIndex is { } index ? "!!" + index : TypeParameterIndex is { } ordinal ? "!" + ordinal : Primitive?.ToString() ?? ClassType!.Namespace + "." + ClassType.Name;
}

/// <summary>An immutable signature with supported value types and optional unconstrained method parameters.</summary>
public class MethodSignature
{
    /// <summary>Copies up to 256 non-Void parameters. Ownership is checked when defining a method.</summary>
    /// <param name="returnType">Primitive, Void or nonstatic owned class.</param>
    /// <param name="parameterTypes">Copied non-Void parameter sequence.</param>
    /// <exception cref="ArgumentNullException">Result or parameters are null.</exception>
    /// <exception cref="ArgumentException">Null/Void parameter or more than 256 parameters.</exception>
    /// <param name="genericParameterNames">Copied unique method parameter names, at most 32; omitted for nongeneric signatures.</param>
    public MethodSignature(SignatureType returnType, IEnumerable<SignatureType> parameterTypes, IEnumerable<string>? genericParameterNames = null)
    {
        ArgumentNullException.ThrowIfNull(returnType);
        ArgumentNullException.ThrowIfNull(parameterTypes);
        var names = (genericParameterNames ?? []).Take(33).ToArray();
        if (names.Length > 32 || names.Any(n => string.IsNullOrWhiteSpace(n) || n.Length > 256 || n.Any(char.IsControl)) || names.Distinct().Count() != names.Length)
            throw new ArgumentException("invalid generic parameter names", nameof(genericParameterNames));
        foreach (var name in names)
            try { _ = new System.Text.UTF8Encoding(false, true).GetByteCount(name); }
            catch (System.Text.EncoderFallbackException error) { throw new ArgumentException("invalid generic parameter Unicode", nameof(genericParameterNames), error); }
        GenericParameterNames = Array.AsReadOnly(names);
        var parameters = parameterTypes.Take(257).ToArray();
        if (parameters.Length > 256 || parameters.Any(p => p is null || p.Primitive == PrimitiveType.Void))
            throw new ArgumentException("invalid parameter signature", nameof(parameterTypes));
        ReturnType = returnType; ParameterTypes = Array.AsReadOnly(parameters);
    }
    /// <summary>Creates a primitive-only signature.</summary>
    public MethodSignature(PrimitiveType returnType, IEnumerable<PrimitiveType> parameterTypes)
        : this((SignatureType)returnType, (parameterTypes ?? throw new ArgumentNullException(nameof(parameterTypes))).Select(p => (SignatureType)p)) { }
    /// <summary>Gets copied names of unconstrained method generic parameters, in ordinal order.</summary>
    public IReadOnlyList<string> GenericParameterNames { get; }
    /// <summary>Gets the result type; Void denotes no result.</summary>
    public SignatureType ReturnType { get; }
    /// <summary>Gets declared parameters, excluding the receiver.</summary>
    public IReadOnlyList<SignatureType> ParameterTypes { get; }
    internal void ValidateOwner(AssemblyBuilder assembly, int typeArity = 0)
    {
        foreach (var type in ParameterTypes.Append(ReturnType)) type.ValidateOwner(assembly, GenericParameterNames.Count, typeArity);
    }
    internal bool Matches(MethodSignature other) => GenericParameterNames.Count == other.GenericParameterNames.Count && ReturnType == other.ReturnType && ParameterTypes.SequenceEqual(other.ParameterTypes);
}
