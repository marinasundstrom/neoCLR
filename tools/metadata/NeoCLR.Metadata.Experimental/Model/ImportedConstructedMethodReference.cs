namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable imported call on a constructed reference class or interface.</summary>
public sealed class ImportedConstructedMethodReference
{
    internal ImportedConstructedMethodReference(ImportedMethodReference definition, ConstructedMethodReference target, ImportedTypeReference declaringType)
    { Definition = definition; Target = target; DeclaringType = declaringType; }
    /// <summary>Gets the consumer-owned open member reference.</summary>
    public ImportedMethodReference Definition { get; }
    /// <summary>Gets the constructed consumer-owned receiver/declaring type.</summary>
    public ImportedTypeReference DeclaringType { get; }
    /// <summary>Gets the substituted argument and return signature.</summary>
    public MethodSignature Signature => Target.Signature;
    /// <summary>Gets copied method arguments, empty for a nongeneric method.</summary>
    public IReadOnlyList<SignatureType> MethodArguments => Target.MethodArguments;
    internal ConstructedMethodReference Target { get; }
}

public sealed partial class ImportedMethodReference
{
    /// <summary>Binds an imported generic reference owner and optional method parameters.</summary>
    /// <param name="declaringTypeArguments">Exactly one non-Void consumer-scoped type per owner parameter.</param>
    /// <param name="methodArguments">Exactly one per method parameter; null means none.</param>
    /// <returns>An immutable reference with copied arguments.</returns>
    /// <exception cref="ArgumentNullException">Owner arguments are null.</exception>
    /// <exception cref="ArgumentException">Wrong arity, foreign arguments or unsupported substitution.</exception>
    public ImportedConstructedMethodReference MakeConstructedReference(IEnumerable<SignatureType> declaringTypeArguments, IEnumerable<SignatureType>? methodArguments = null)
    {
        ArgumentNullException.ThrowIfNull(declaringTypeArguments);
        var owners = declaringTypeArguments.Take(33).ToArray();
        var methods = (methodArguments ?? []).Take(33).ToArray();
        if (DeclaringReference is not { GenericArity: > 0 } declaring || owners.Length != declaring.GenericArity || methods.Length != Signature.GenericParameterNames.Count)
            throw new ArgumentException("constructed import requires matching owner and method arities");
        foreach (var type in owners.Concat(methods)) {
            if (type is null || type.Primitive == PrimitiveType.Void) throw new ArgumentException("invalid generic argument");
            type.ValidateOwner(Owner, 32, 32);
        }
        return new(this, new ConstructedMethodReference(Target, owners, methods), declaring.MakeGenericInstance(owners));
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Appends a direct imported call on a constructed reference owner.</summary>
    public void Call(ImportedConstructedMethodReference method) => Emit(OpCode.Call, method);
    /// <summary>Appends interface dispatch to an imported constructed contract.</summary>
    public void CallVirtual(ImportedConstructedMethodReference method) => Emit(OpCode.Callvirt, method);
    /// <summary>Appends interface dispatch to an imported nongeneric contract.</summary>
    public void CallVirtual(ImportedMethodReference method) => Emit(OpCode.Callvirt, method);
    /// <summary>Emits Call for concrete/static methods or Callvirt for interface contracts.</summary>
    /// <param name="opCode">Opcode matching the imported member dispatch kind.</param>
    /// <param name="operand">Reference owned by the current output.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, foreign consumer, or invalid caller generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack checked on write.</exception>
    public void Emit(OpCode opCode, ImportedConstructedMethodReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (!ReferenceEquals(operand.Definition.Owner, Assembly) || opCode != (operand.Definition.RequiresVirtualDispatch ? OpCode.Callvirt : OpCode.Call))
            throw new ArgumentException("incorrect imported owner or dispatch opcode");
        foreach (var type in operand.DeclaringType.TypeArguments.Concat(operand.MethodArguments))
            type.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new(operand.Definition.RequiresVirtualDispatch ? "call.virtual.constructed" : "call.constructed", Target: operand.Definition.Target,
            ConstructedTarget: operand.Target, Type: operand.Definition.IsStatic ? null : (SignatureType)operand.DeclaringType));
    }
}
