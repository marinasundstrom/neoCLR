namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable concrete instantiation of an imported static generic method.</summary>
public sealed class ImportedGenericMethodReference
{
    internal ImportedGenericMethodReference(ImportedMethodReference definition, GenericMethodInstance target)
    { Definition = definition; Target = target; }
    internal GenericMethodInstance Target { get; }
    /// <summary>Gets the immutable imported definition and consuming owner.</summary>
    public ImportedMethodReference Definition { get; }
    /// <summary>Gets copied concrete primitive/vector arguments in generic parameter order.</summary>
    public IReadOnlyList<SignatureType> TypeArguments => Target.TypeArguments;
    /// <summary>Gets the substituted parameter/result signature.</summary>
    public MethodSignature Signature => Target.Signature;
}

public sealed partial class ImportedMethodReference
{
    /// <summary>Instantiates an imported unconstrained method with concrete primitive/vector arguments.</summary>
    /// <param name="typeArguments">Exactly one non-Void primitive or primitive vector per method parameter; copied.</param>
    /// <returns>An immutable reference owned by the same consumer.</returns>
    /// <exception cref="ArgumentNullException">Argument array is null.</exception>
    /// <exception cref="ArgumentException">Wrong arity, nongeneric definition, unsupported argument or nested-vector substitution.</exception>
    public ImportedGenericMethodReference MakeGenericInstance(params SignatureType[] typeArguments)
    {
        ArgumentNullException.ThrowIfNull(typeArguments);
        if (typeArguments.Any(t => t is null || t.Primitive == PrimitiveType.Void ||
            t.Primitive is null && t.ArrayElement?.Primitive is null))
            throw new ArgumentException("imported generic calls require concrete primitive or primitive-vector arguments", nameof(typeArguments));
        return new(this, Target.MakeGenericInstance(typeArguments));
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Appends a call to an instantiated imported generic method.</summary>
    /// <param name="method">An instantiation owned by this output.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Reference belongs to another output.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validation occurs on write.</exception>
    public void Call(ImportedGenericMethodReference method) => Emit(OpCode.Call, method);
    /// <summary>Appends a generic imported call using the raw typed operand API.</summary>
    /// <param name="opCode">Call only.</param>
    /// <param name="operand">An instantiation owned by this output.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode or consuming owner.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, ImportedGenericMethodReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        RequireCall(opCode);
        if (!ReferenceEquals(operand.Definition.Owner, Assembly)) throw new ArgumentException("reference belongs to another output builder", nameof(operand));
        Append(new("call.generic", Target: operand.Target.Definition, GenericTarget: operand.Target));
    }
}
