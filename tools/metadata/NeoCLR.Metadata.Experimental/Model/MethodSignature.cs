namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable primitive, owned nominal/imported reference type, scoped generic parameter, vector or managed-reference parameter signature type.</summary>
public sealed partial record SignatureType
{
    private SignatureType(PrimitiveType? primitive, TypeBuilder? classType, SignatureType? arrayElement = null, int? methodParameter = null, int? typeParameter = null, GenericTypeInstance? genericInstance = null, ImportedTypeReference? importedType = null, SignatureType? byReferenceElement = null, FunctionSignature? functionSignature = null) { FunctionSignature = functionSignature; ByReferenceElement = byReferenceElement; ImportedType = importedType; GenericInstance = genericInstance; Primitive = primitive; ClassType = classType; ArrayElement = arrayElement; MethodParameterIndex = methodParameter; TypeParameterIndex = typeParameter; }
    /// <summary>Gets the target of a writable managed-reference parameter, or null.</summary>
    public SignatureType? ByReferenceElement { get; }
    /// <summary>Creates a writable managed-reference parameter type. Ref arguments must be initialized before calls.</summary>
    /// <param name="elementType">Non-Void, non-byref type; scoped parameters are supported.</param>
    /// <exception cref="ArgumentNullException">Element is null.</exception>
    /// <exception cref="ArgumentException">Void, nested byref or nesting limit exceeded.</exception>
    public static SignatureType ByReference(SignatureType elementType)
    {
        ArgumentNullException.ThrowIfNull(elementType);
        if (elementType.Primitive == PrimitiveType.Void || elementType.ByReferenceElement is not null || elementType.NestingDepth >= 16)
            throw new ArgumentException("invalid managed-reference target", nameof(elementType));
        return new(null, null, byReferenceElement: elementType);
    }
    /// <summary>Gets the primitive kind, or null for a class or vector reference.</summary>
    public PrimitiveType? Primitive { get; }
    /// <summary>Gets the exact owned nominal identity (class, interface or value type), or null for other signatures.</summary>
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
        if (elementType.Primitive == PrimitiveType.Void || elementType.ArrayElement is not null || elementType.ByReferenceElement is not null || elementType.NestingDepth >= 16)
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
    /// <summary>Gets an owned constructed nominal type; imported constructions use ImportedType.</summary>
    public GenericTypeInstance? GenericInstance { get; }
    /// <summary>Creates a signature for an owned constructed generic type.</summary>
    public static implicit operator SignatureType(GenericTypeInstance type)
        => new(null, null, genericInstance: type ?? throw new ArgumentNullException(nameof(type)));
    /// <summary>Gets an external reference type or construction, or null for other signatures.</summary>
    public ImportedTypeReference? ImportedType { get; }
    /// <summary>Creates a signature for a nongeneric imported reference or a complete construction.</summary>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">An open generic definition requires construction.</exception>
    public static implicit operator SignatureType(ImportedTypeReference type)
    {
        ArgumentNullException.ThrowIfNull(type);
        if (type.GenericArity != type.TypeArguments.Count) throw new ArgumentException("imported generic definition requires construction", nameof(type));
        return new(null, null, importedType: type);
    }
    internal int NestingDepth => FunctionSignature is { } shape ? 1 + shape.ParameterTypes.Append(shape.ReturnType).Max(t => t.NestingDepth) : ByReferenceElement is { } target ? 1 + target.NestingDepth : ImportedType is { TypeArguments.Count: > 0 } imported ? 1 + imported.TypeArguments.Max(t => t.NestingDepth) : GenericInstance is { } instance ? 1 + instance.TypeArguments.Max(t => t.NestingDepth) : ArrayElement is { } element ? 1 + element.NestingDepth : 0;
    internal void ValidateOwner(AssemblyBuilder assembly, int genericArity = 0, int typeArity = 0, bool complete = false, bool allowByReference = false)
    {
        if (FunctionSignature is { } function) foreach (var type in function.ParameterTypes.Append(function.ReturnType)) type.ValidateOwner(assembly, genericArity, typeArity, complete);
        if (ByReferenceElement is { } target)
        {
            if (!allowByReference) throw new ArgumentException("managed references are supported only as method parameters");
            target.ValidateOwner(assembly, genericArity, typeArity, complete);
        }
        if (ImportedType is { } imported)
        {
            if (!ReferenceEquals(imported.Owner, assembly)) throw new ArgumentException("imported type belongs to another output");
            foreach (var argument in imported.TypeArguments) argument.ValidateOwner(assembly, genericArity, typeArity, complete);
        }
        if (MethodParameterIndex is { } index && index >= genericArity) throw new ArgumentException("method type parameter outside declared scope");
        if (TypeParameterIndex is { } ordinal && ordinal >= typeArity) throw new ArgumentException("type parameter outside declared scope");
        ArrayElement?.ValidateOwner(assembly, genericArity, typeArity, complete);
        if (GenericInstance is { } instance)
        {
            if (!ReferenceEquals(instance.Definition.Assembly, assembly)) throw new ArgumentException("foreign constructed class");
            instance.Definition.ValidateTypeArguments(instance.TypeArguments, complete);
            foreach (var argument in instance.TypeArguments) argument.ValidateOwner(assembly, genericArity, typeArity, complete);
        }
        if ((ArrayElement?.ClassType ?? ClassType) is { } owner && !ReferenceEquals(owner.Assembly, assembly))
            throw new ArgumentException("signature requires a class owned by the output assembly");
    }
    /// <summary>Creates a primitive signature type, including Void for results only.</summary>
    public static implicit operator SignatureType(PrimitiveType type)
        => Enum.IsDefined(type) ? new(type, null) : throw new ArgumentOutOfRangeException(nameof(type));
    /// <summary>Creates a signature for an owned nongeneric class, interface or value type; generic definitions require construction.</summary>
    public static implicit operator SignatureType(TypeBuilder type)
    {
        ArgumentNullException.ThrowIfNull(type);
        if (type.IsStatic || type.GenericParameterNames.Count > 0) throw new ArgumentException("signature requires a nonstatic class or interface", nameof(type));
        return new(null, type);
    }
    /// <summary>Returns a diagnostic name; it is not a serialized type identity.</summary>
    public override string ToString() => FunctionSignature is { } function ? function.ToString() : ByReferenceElement is { } target ? target + "&" : ImportedType is { } imported ? imported.ToString() : GenericInstance is { } instance ? instance.ToString() : ArrayElement is { } element ? element + "[]" : MethodParameterIndex is { } index ? "!!" + index : TypeParameterIndex is { } ordinal ? "!" + ordinal : Primitive?.ToString() ?? ClassType!.Namespace + "." + ClassType.Name;
}

/// <summary>An immutable signature with supported value types and optional unconstrained method parameters.</summary>
public class MethodSignature
{
    /// <summary>Copies up to 256 non-Void parameters. Ownership is checked when defining a method.</summary>
    /// <param name="returnType">Supported primitive, Void, owned nominal or imported reference type.</param>
    /// <param name="parameterTypes">Copied non-Void parameter sequence.</param>
    /// <exception cref="ArgumentNullException">Result or parameters are null.</exception>
    /// <exception cref="ArgumentException">Null/Void parameter, byref result, invalid out index, or more than 256 parameters.</exception>
    /// <param name="genericParameterNames">Copied unique method parameter names, at most 32; omitted for nongeneric signatures.</param>
    /// <param name="outParameters">Copied distinct zero-based byref parameter indices assigned before normal return; omitted for ordinary ref parameters.</param>
    public MethodSignature(SignatureType returnType, IEnumerable<SignatureType> parameterTypes, IEnumerable<string>? genericParameterNames = null, IEnumerable<int>? outParameters = null)
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
        if (returnType.ByReferenceElement is not null) throw new ArgumentException("byref returns are unsupported", nameof(returnType));
        var outputs = (outParameters ?? []).Take(257).ToArray();
        if (outputs.Length > 256 || outputs.Distinct().Count() != outputs.Length || outputs.Any(i => i < 0 || i >= parameters.Length || parameters[i].ByReferenceElement is null))
            throw new ArgumentException("out parameters must be distinct byref parameter indices", nameof(outParameters));
        OutParameters = Array.AsReadOnly(outputs.Order().ToArray());
        ReturnType = returnType; ParameterTypes = Array.AsReadOnly(parameters);
    }
    /// <summary>Creates a primitive-only signature.</summary>
    public MethodSignature(PrimitiveType returnType, IEnumerable<PrimitiveType> parameterTypes)
        : this((SignatureType)returnType, (parameterTypes ?? throw new ArgumentNullException(nameof(parameterTypes))).Select(p => (SignatureType)p)) { }
    /// <summary>Gets copied names of unconstrained method generic parameters, in ordinal order.</summary>
    public IReadOnlyList<string> GenericParameterNames { get; }
    /// <summary>Gets the result type; Void denotes no result.</summary>
    public SignatureType ReturnType { get; }
    /// <summary>Gets zero-based declared parameter indices that must be assigned before every normal return.</summary>
    public IReadOnlyList<int> OutParameters { get; }
    /// <summary>Gets declared parameters, excluding the receiver.</summary>
    public IReadOnlyList<SignatureType> ParameterTypes { get; }
    internal void ValidateOwner(AssemblyBuilder assembly, int typeArity = 0, bool complete = false)
    {
        ReturnType.ValidateOwner(assembly, GenericParameterNames.Count, typeArity, complete);
        foreach (var type in ParameterTypes) type.ValidateOwner(assembly, GenericParameterNames.Count, typeArity, complete, allowByReference: true);
    }
    internal bool Matches(MethodSignature other) => GenericParameterNames.Count == other.GenericParameterNames.Count && ReturnType == other.ReturnType && ParameterTypes.SequenceEqual(other.ParameterTypes) && OutParameters.SequenceEqual(other.OutParameters);
}
