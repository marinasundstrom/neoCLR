namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable imported call on a constructed nominal type.</summary>
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
    /// <summary>Binds an imported generic nominal owner and optional method parameters.</summary>
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
    /// <summary>Allocates through an imported nongeneric constructor; consumes its parameters and returns the declaring value or reference type.</summary>
    /// <param name="constructor">A constructor reference owned by this output assembly.</param>
    /// <exception cref="ArgumentNullException">The reference is null.</exception>
    /// <exception cref="ArgumentException">The reference is not a constructor, belongs to another output, or has invalid generic scope.</exception>
    /// <exception cref="InvalidDataException">The instruction limit is exceeded; stack validity is checked on write.</exception>
    public void NewObject(ImportedMethodReference constructor) => GetILGenerator().NewObject(constructor);
    /// <summary>Allocates through an imported constructor on a constructed generic owner.</summary>
    /// <param name="constructor">A constructor reference owned by this output assembly.</param>
    /// <exception cref="ArgumentNullException">The reference is null.</exception>
    /// <exception cref="ArgumentException">The reference is not a constructor, belongs to another output, or has invalid generic scope.</exception>
    /// <exception cref="InvalidDataException">The instruction limit is exceeded; stack validity is checked on write.</exception>
    public void NewObject(ImportedConstructedMethodReference constructor) => GetILGenerator().NewObject(constructor);

    /// <summary>Appends a direct imported call on a constructed nominal owner.</summary>
    public void Call(ImportedConstructedMethodReference method) => GetILGenerator().Call(method);
    /// <summary>Appends interface dispatch to an imported constructed contract.</summary>
    public void CallVirtual(ImportedConstructedMethodReference method) => GetILGenerator().CallVirtual(method);
    /// <summary>Appends interface dispatch to an imported nongeneric contract.</summary>
    public void CallVirtual(ImportedMethodReference method) => GetILGenerator().CallVirtual(method);
    /// <summary>Emits Newobj for constructors, Call for concrete/static methods or Callvirt for interface contracts.</summary>
    /// <param name="opCode">Opcode matching the imported member dispatch kind.</param>
    /// <param name="operand">Reference owned by the current output.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, foreign consumer, or invalid caller generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack checked on write.</exception>
    public void Emit(OpCode opCode, ImportedConstructedMethodReference operand) => GetILGenerator().Emit(opCode, operand);
}
