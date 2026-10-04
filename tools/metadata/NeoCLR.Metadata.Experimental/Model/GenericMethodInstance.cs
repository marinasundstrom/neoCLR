namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable instantiation of an owned generic method or function.</summary>
public sealed class GenericMethodInstance
{
    internal GenericMethodInstance(MethodBuilder definition, SignatureType[] arguments)
    {
        Definition = definition;
        TypeArguments = Array.AsReadOnly(arguments);
        SignatureType Substitute(SignatureType type) => type.FunctionSignature is { } function ? function.Substitute(Substitute) : type.ByReferenceElement is { } target ? SignatureType.ByReference(Substitute(target)) : type.ImportedType is { } imported ? imported.Substitute(Substitute) : type.MethodParameterIndex is { } index ? arguments[index]
            : type.GenericInstance is { } instance ? instance.Definition.MakeGenericInstance(instance.TypeArguments.Select(Substitute).ToArray())
            : type.ArrayElement is { } element ? SignatureType.ArrayOf(Substitute(element)) : type;
        Signature = new(Substitute(definition.Signature.ReturnType), definition.Signature.ParameterTypes.Select(Substitute), outParameters: definition.Signature.OutParameters);
    }
    /// <summary>Gets the generic definition, retaining its declaring assembly and owner.</summary>
    public MethodBuilder Definition { get; }
    /// <summary>Gets copied type arguments in generic parameter order.</summary>
    public IReadOnlyList<SignatureType> TypeArguments { get; }
    /// <summary>Gets the substituted call signature. Caller-scoped method parameters may remain.</summary>
    public MethodSignature Signature { get; }
}

public sealed partial class MethodBuilder
{
    /// <summary>Constructs a typed generic call reference without changing the definition.</summary>
    /// <param name="typeArguments">Exactly one non-Void supported type per generic parameter; copied.</param>
    /// <returns>An immutable instantiation. Caller-scoped MVAR arguments are checked on emission.</returns>
    /// <exception cref="ArgumentNullException">The argument array is null.</exception>
    /// <exception cref="ArgumentException">Wrong arity, nongeneric definition, null/Void/foreign argument or nested array substitution.</exception>
    public GenericMethodInstance MakeGenericInstance(params SignatureType[] typeArguments)
    {
        ArgumentNullException.ThrowIfNull(typeArguments);
        if (DeclaringType?.GenericParameterNames.Count > 0 || IsConstructor || Signature.GenericParameterNames.Count == 0 || typeArguments.Length != Signature.GenericParameterNames.Count ||
            typeArguments.Any(t => t is null || t.Primitive == PrimitiveType.Void))
            throw new ArgumentException("generic type arguments must match the definition", nameof(typeArguments));
        foreach (var type in typeArguments) type.ValidateOwner(Assembly, 32, 32);
        ValidateMethodArguments(typeArguments, deferOpen: true);
        return new(this, (SignatureType[])typeArguments.Clone());
    }
    /// <summary>Appends a call to an instantiated generic method in this output.</summary>
    /// <param name="method">Owned instantiation with arguments valid in the caller's generic scope.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Foreign definition or out-of-scope type arguments.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack shape is validated on write.</exception>
    public void Call(GenericMethodInstance method) => GetILGenerator().Call(method);
    /// <summary>Appends a generic call using an explicit typed reference.</summary>
    /// <param name="opCode">Call only.</param>
    /// <param name="operand">Instantiation; see Call(GenericMethodInstance).</param>
    /// <exception cref="ArgumentException">Wrong opcode, foreign definition or invalid generic scope.</exception>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, GenericMethodInstance operand) => GetILGenerator().Emit(opCode, operand);
}
