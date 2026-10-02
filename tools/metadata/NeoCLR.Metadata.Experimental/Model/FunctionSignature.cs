namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable structural callable shape, independent of its bound target.</summary>
public sealed class FunctionSignature : IEquatable<FunctionSignature>
{
    internal FunctionSignature(MethodSignature signature) { Signature = signature; }
    internal MethodSignature Signature { get; }
    /// <summary>Gets the ordered parameter types; names do not participate in identity.</summary>
    public IReadOnlyList<SignatureType> ParameterTypes => Signature.ParameterTypes;
    /// <summary>Gets the result type, or Void for an explicit no-result function.</summary>
    public SignatureType ReturnType => Signature.ReturnType;
    /// <summary>Gets whether invocation leaves no result on the evaluation stack.</summary>
    public bool NoResult => ReturnType.Primitive == PrimitiveType.Void;
    /// <summary>Compares exact ordered parameter and result shapes.</summary>
    public bool Equals(FunctionSignature? other) => other is not null && ReturnType == other.ReturnType && ParameterTypes.SequenceEqual(other.ParameterTypes);
    /// <summary>Compares structural shapes.</summary>
    public override bool Equals(object? obj) => obj is FunctionSignature other && Equals(other);
    /// <summary>Gets a hash consistent with structural equality.</summary>
    public override int GetHashCode() { var hash = new HashCode(); hash.Add(ReturnType); foreach (var type in ParameterTypes) hash.Add(type); return hash.ToHashCode(); }
    internal SignatureType Substitute(Func<SignatureType, SignatureType> map) => SignatureType.Function(new MethodSignature(map(ReturnType), ParameterTypes.Select(map)));
    /// <summary>Returns a diagnostic shape, not an encoded identity.</summary>
    public override string ToString() => "(" + string.Join(",", ParameterTypes) + ") -> " + ReturnType;
}

public sealed partial record SignatureType
{
    /// <summary>Gets the structural callable shape, or null for another type category.</summary>
    public FunctionSignature? FunctionSignature { get; }
    /// <summary>Creates an exact structural Function shape; CLI output uses Func/Action transport.</summary>
    /// <param name="signature">Up to sixteen value parameters and a value or no-result return. Generic declarations and byref parameters are currently unsupported; caller-scoped type parameters are permitted.</param>
    /// <returns>An immutable shape independent of any callable target.</returns>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Unsupported parameters or excessive nesting.</exception>
    public static SignatureType Function(MethodSignature signature)
    {
        ArgumentNullException.ThrowIfNull(signature);
        if (signature.GenericParameterNames.Count != 0 || signature.ParameterTypes.Count > 16 ||
            signature.ParameterTypes.Any(t => t.ByReferenceElement is not null) ||
            signature.ParameterTypes.Append(signature.ReturnType).Any(t => t.NestingDepth >= 16)) throw new ArgumentException("unsupported function shape", nameof(signature));
        return new(null, null, functionSignature: new FunctionSignature(signature));
    }
}

/// <summary>A checked static method binding operand. The target is instance data, not part of Function type identity.</summary>
public sealed class FunctionBinding
{
    /// <summary>Creates an operand binding a nongeneric static method to an exact structural shape.</summary>
    /// <param name="functionType">A structural Function signature.</param>
    /// <param name="target">A nongeneric static target with the exact parameter/result signature.</param>
    /// <exception cref="ArgumentNullException">Either operand is null.</exception>
    /// <exception cref="ArgumentException">Shape, target kind or signatures do not match.</exception>
    public FunctionBinding(SignatureType functionType, MethodBuilder target)
    {
        ArgumentNullException.ThrowIfNull(functionType); ArgumentNullException.ThrowIfNull(target);
        if (functionType.FunctionSignature is not { } shape || !target.IsStatic || target.IsAbstract || target.IsConstructor ||
            target.Signature.GenericParameterNames.Count != 0 || target.DeclaringType?.GenericParameterNames.Count > 0 || !shape.Signature.Matches(target.Signature))
            throw new ArgumentException("function binding requires an exact nongeneric static target");
        FunctionType = functionType; Target = target;
    }
    /// <summary>Gets the structural type of the bound value.</summary>
    public SignatureType FunctionType { get; }
    /// <summary>Gets the static method selected for invocation.</summary>
    public MethodBuilder Target { get; }
}

public sealed partial class MethodBuilder
{
    /// <summary>Pushes a Function value bound to an owned static method.</summary>
    /// <param name="functionType">The exact structural shape.</param>
    /// <param name="target">Owned nongeneric static target.</param>
    /// <exception cref="ArgumentException">Invalid target, shape, foreign owner or scope.</exception>
    /// <exception cref="ArgumentNullException">An operand is null.</exception>
    public void BindFunction(SignatureType functionType, MethodBuilder target) => Emit(OpCode.BindFunction, new FunctionBinding(functionType, target));
    /// <summary>Emits a checked Function binding; consumes no receiver and pushes the callable value.</summary>
    /// <param name="opCode">BindFunction.</param>
    /// <param name="operand">The exact static binding owned by this assembly.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, foreign target or invalid generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, FunctionBinding operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (opCode != OpCode.BindFunction || !ReferenceEquals(operand.Target.Assembly, Assembly)) throw new ArgumentException("binding requires BindFunction and an owned target");
        operand.FunctionType.ValidateOwner(Assembly, Signature.GenericParameterNames.Count, DeclaringType?.GenericParameterNames.Count ?? 0);
        Append(new("function.bind", Target: operand.Target, Type: operand.FunctionType));
    }
    /// <summary>Consumes a Function receiver followed by its arguments, then pushes its result if any.</summary>
    /// <param name="functionType">Exact structural Function type.</param>
    /// <exception cref="ArgumentException">Not a Function or invalid generic scope/owner.</exception>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validity is checked on write.</exception>
    public void InvokeFunction(SignatureType functionType) => Emit(OpCode.Callvirt, functionType);
}
