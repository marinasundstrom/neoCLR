namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A method or constructor on a constructed owned generic type, optionally with method arguments.</summary>
public sealed class ConstructedMethodReference
{
    internal ConstructedMethodReference(MethodBuilder definition, SignatureType[] ownerArguments, SignatureType[] methodArguments)
    {
        Definition = definition;
        DeclaringTypeArguments = Array.AsReadOnly(ownerArguments);
        MethodArguments = Array.AsReadOnly(methodArguments);
        SignatureType Substitute(SignatureType type) => type.TypeParameterIndex is { } owner ? ownerArguments[owner]
            : type.MethodParameterIndex is { } method ? methodArguments[method]
            : type.GenericInstance is { } instance ? instance.Definition.MakeGenericInstance(instance.TypeArguments.Select(Substitute).ToArray())
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
    /// <summary>Binds a generic owner and all method parameters in one immutable reference.</summary>
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
        if (DeclaringType is not { GenericParameterNames.Count: > 0 } owner || owners.Length != owner.GenericParameterNames.Count || methods.Length != Signature.GenericParameterNames.Count)
            throw new ArgumentException("constructed reference requires matching owner and method arities");
        foreach (var type in owners.Concat(methods))
        {
            if (type is null || type.Primitive == PrimitiveType.Void) throw new ArgumentException("invalid generic argument");
            type.ValidateOwner(Assembly, 32, 32);
        }
        return new(this, owners, methods);
    }
    /// <summary>Allocates and invokes a constructor on a constructed generic class.</summary>
    /// <param name="constructor">Owned constructed constructor reference.</param>
    /// <exception cref="ArgumentException">Not a constructor, foreign owner or invalid caller scope.</exception>
    /// <exception cref="ArgumentNullException">Null reference.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void NewObject(ConstructedMethodReference constructor) => Emit(OpCode.Newobj, constructor);
    /// <summary>Calls a method on a constructed generic owner.</summary>
    public void Call(ConstructedMethodReference method) => Emit(OpCode.Call, method);
    /// <summary>Appends Call or Newobj with a constructed owner/method reference.</summary>
    /// <param name="opCode">Newobj for constructors; Call otherwise.</param>
    /// <param name="operand">Owned reference, valid in the caller's type/method scope.</param>
    /// <exception cref="ArgumentNullException">Null reference.</exception>
    /// <exception cref="ArgumentException">Foreign owner, wrong opcode or invalid caller scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack checked on write.</exception>
    public void Emit(OpCode opCode, ConstructedMethodReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (opCode != (operand.Definition.IsConstructor ? OpCode.Newobj : OpCode.Call)) throw new ArgumentException("constructor requires Newobj; other members require Call");
        if (!ReferenceEquals(operand.Definition.Assembly, Assembly)) throw new ArgumentException("constructed calls require an owned definition");
        foreach (var type in operand.DeclaringTypeArguments.Concat(operand.MethodArguments))
            type.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new(operand.Definition.IsConstructor ? "new.constructed" : "call.constructed", Target: operand.Definition, ConstructedTarget: operand));
    }
}
