namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A static method on a constructed owned generic type, optionally with method arguments.</summary>
public sealed class ConstructedMethodReference
{
    internal ConstructedMethodReference(MethodBuilder definition, SignatureType[] ownerArguments, SignatureType[] methodArguments)
    {
        Definition = definition;
        DeclaringTypeArguments = Array.AsReadOnly(ownerArguments);
        MethodArguments = Array.AsReadOnly(methodArguments);
        SignatureType Substitute(SignatureType type) => type.TypeParameterIndex is { } owner ? ownerArguments[owner]
            : type.MethodParameterIndex is { } method ? methodArguments[method]
            : type.ArrayElement is { } element ? SignatureType.ArrayOf(Substitute(element)) : type;
        Signature = new(Substitute(definition.Signature.ReturnType), definition.Signature.ParameterTypes.Select(Substitute));
    }
    /// <summary>Gets the owned open definition.</summary>
    public MethodBuilder Definition { get; }
    /// <summary>Gets copied declaring-type arguments.</summary>
    public IReadOnlyList<SignatureType> DeclaringTypeArguments { get; }
    /// <summary>Gets copied method arguments, empty for a nongeneric method.</summary>
    public IReadOnlyList<SignatureType> MethodArguments { get; }
    /// <summary>Gets the simultaneously substituted call signature.</summary>
    public MethodSignature Signature { get; }
}

public sealed partial class MethodBuilder
{
    /// <summary>Binds a static generic owner and all method parameters in one immutable reference.</summary>
    /// <param name="declaringTypeArguments">One supported non-Void value type per owner parameter.</param>
    /// <param name="methodArguments">One per method parameter; null denotes none.</param>
    /// <returns>A constructed call reference, with copied arguments.</returns>
    /// <exception cref="ArgumentNullException">Owner argument sequence is null.</exception>
    /// <exception cref="ArgumentException">Wrong arity/owner, invalid argument, foreign class or nested array substitution.</exception>
    public ConstructedMethodReference MakeConstructedReference(IEnumerable<SignatureType> declaringTypeArguments, IEnumerable<SignatureType>? methodArguments = null)
    {
        ArgumentNullException.ThrowIfNull(declaringTypeArguments);
        var owners = declaringTypeArguments.Take(33).ToArray();
        var methods = (methodArguments ?? []).Take(33).ToArray();
        if (!IsStatic || DeclaringType is not { GenericParameterNames.Count: > 0 } owner || owners.Length != owner.GenericParameterNames.Count || methods.Length != Signature.GenericParameterNames.Count)
            throw new ArgumentException("constructed reference requires matching static owner and method arities");
        foreach (var type in owners.Concat(methods))
        {
            if (type is null || type.Primitive == PrimitiveType.Void) throw new ArgumentException("invalid generic argument");
            type.ValidateOwner(Assembly, 32, 32);
        }
        return new(this, owners, methods);
    }
    /// <summary>Calls a static method on a constructed generic owner.</summary>
    public void Call(ConstructedMethodReference method) => Emit(OpCode.Call, method);
    /// <summary>Appends Call with a constructed owner/method reference.</summary>
    /// <param name="opCode">Call only.</param>
    /// <param name="operand">Owned reference, valid in the caller's type/method scope.</param>
    /// <exception cref="ArgumentNullException">Null reference.</exception>
    /// <exception cref="ArgumentException">Foreign owner, wrong opcode or invalid caller scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack checked on write.</exception>
    public void Emit(OpCode opCode, ConstructedMethodReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand); RequireCall(opCode);
        if (!ReferenceEquals(operand.Definition.Assembly, Assembly)) throw new ArgumentException("constructed calls require an owned definition");
        foreach (var type in operand.DeclaringTypeArguments.Concat(operand.MethodArguments))
            type.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new("call.constructed", Target: operand.Definition, ConstructedTarget: operand));
    }
}
