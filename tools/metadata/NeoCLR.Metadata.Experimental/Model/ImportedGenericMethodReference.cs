namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable instantiation of an imported static generic method.</summary>
public sealed class ImportedGenericMethodReference
{
    internal ImportedGenericMethodReference(ImportedMethodReference definition, GenericMethodInstance target)
    { Definition = definition; Target = target; }
    internal GenericMethodInstance Target { get; }
    /// <summary>Gets the immutable imported definition and consuming owner.</summary>
    public ImportedMethodReference Definition { get; }
    /// <summary>Gets copied consumer-scoped arguments in generic parameter order.</summary>
    public IReadOnlyList<SignatureType> TypeArguments => Target.TypeArguments;
    /// <summary>Gets the substituted parameter/result signature.</summary>
    public MethodSignature Signature => Target.Signature;
}

public sealed partial class ImportedMethodReference
{
    /// <summary>Instantiates an imported unconstrained method with consumer-scoped signature arguments.</summary>
    /// <param name="typeArguments">Exactly one non-Void type per method parameter; copied. Caller-scoped parameters are checked on emission.</param>
    /// <returns>An immutable reference owned by the same consumer.</returns>
    /// <exception cref="ArgumentNullException">Argument array is null.</exception>
    /// <exception cref="ArgumentException">Wrong arity, nongeneric definition, foreign argument or nested-vector substitution.</exception>
    public ImportedGenericMethodReference MakeGenericInstance(params SignatureType[] typeArguments)
    {
        ArgumentNullException.ThrowIfNull(typeArguments);
        if (Target.DeclaringType?.GenericParameterNames.Count > 0 || !IsStatic || Target.Signature.GenericParameterNames.Count == 0 || typeArguments.Length != Target.Signature.GenericParameterNames.Count ||
            typeArguments.Any(t => t is null || t.Primitive == PrimitiveType.Void))
            throw new ArgumentException("generic type arguments must match the imported definition", nameof(typeArguments));
        // Arguments belong to the consumer, not the private dependency definition graph.
        foreach (var type in typeArguments) type.ValidateOwner(Owner, 32, 32);
        return new(this, new GenericMethodInstance(Target, (SignatureType[])typeArguments.Clone()));
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Appends a call to an instantiated imported generic method.</summary>
    /// <param name="method">An instantiation owned by this output.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Reference belongs to another output or arguments exceed caller generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validation occurs on write.</exception>
    public void Call(ImportedGenericMethodReference method) => Emit(OpCode.Call, method);
    /// <summary>Appends a generic imported call using the raw typed operand API.</summary>
    /// <param name="opCode">Call only.</param>
    /// <param name="operand">An instantiation owned by this output.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, consuming owner or caller generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, ImportedGenericMethodReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        RequireCall(opCode);
        if (!ReferenceEquals(operand.Definition.Owner, Assembly)) throw new ArgumentException("reference belongs to another output builder", nameof(operand));
        foreach (var type in operand.TypeArguments)
            type.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new("call.generic", Target: operand.Target.Definition, GenericTarget: operand.Target));
    }
}
