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

/// <summary>A checked method binding operand. The target is instance data, not part of Function type identity.</summary>
public sealed class FunctionBinding
{
    /// <summary>Creates an operand binding a nongeneric static, reference-instance or interface method to an exact structural shape.</summary>
    /// <param name="functionType">A structural Function signature.</param>
    /// <param name="target">A nongeneric static, final/nonvirtual reference-instance or interface target with the exact parameter/result signature.</param>
    /// <exception cref="ArgumentNullException">Either operand is null.</exception>
    /// <exception cref="ArgumentException">Shape, target kind or signatures do not match.</exception>
    public FunctionBinding(SignatureType functionType, MethodBuilder target) : this(functionType, target, null, null) { }
    /// <summary>Binds an exact method on a constructed owned generic class or interface, including supplied method arguments.</summary>
    /// <param name="functionType">The structural signature after owner argument substitution.</param>
    /// <param name="target">An owned constructed method reference; instance binding consumes its receiver.</param>
    /// <exception cref="ArgumentNullException">Either operand is null.</exception>
    /// <exception cref="ArgumentException">The target kind or substituted signature is incompatible.</exception>
    public FunctionBinding(SignatureType functionType, ConstructedMethodReference target) : this(functionType,
        target?.Definition ?? throw new ArgumentNullException(nameof(target)), target, null) { }
    /// <summary>Binds an instantiated generic method to its substituted structural signature.</summary>
    /// <param name="functionType">The exact structural signature after method argument substitution.</param>
    /// <param name="target">An owned generic method instance with all arguments supplied.</param>
    /// <exception cref="ArgumentNullException">Either operand is null.</exception>
    /// <exception cref="ArgumentException">The target kind or substituted signature is incompatible.</exception>
    public FunctionBinding(SignatureType functionType, GenericMethodInstance target) : this(functionType,
        target?.Definition ?? throw new ArgumentNullException(nameof(target)), null, target) { }
    private FunctionBinding(SignatureType functionType, MethodBuilder target, ConstructedMethodReference? constructed, GenericMethodInstance? generic)
    {
        ArgumentNullException.ThrowIfNull(functionType); ArgumentNullException.ThrowIfNull(target);
        bool contract = target.DeclaringType?.IsInterface == true && target.IsAbstract;
        if (functionType.FunctionSignature is not { } shape || (!target.IsStatic && (target.DeclaringType is not { IsValueType: false } || (target.Definition.Attributes & 0x60) == 0x40 && !contract)) || target.IsAbstract && !contract || target.IsConstructor ||
            constructed is null && generic is null && target.Signature.GenericParameterNames.Count != 0 || constructed is null && target.DeclaringType?.GenericParameterNames.Count > 0 || !shape.Signature.Matches(constructed?.Signature ?? generic?.Signature ?? target.Signature))
            throw new ArgumentException("function binding requires an exact static, reference-instance or interface target");
        FunctionType = functionType; Target = target; ConstructedTarget = constructed; GenericTarget = generic;
    }
    /// <summary>Gets the substituted owner reference, or null for an unconstructed target.</summary>
    public ConstructedMethodReference? ConstructedTarget { get; }
    /// <summary>Gets the instantiated generic method, or null.</summary>
    public GenericMethodInstance? GenericTarget { get; }
    /// <summary>Gets the structural type of the bound value.</summary>
    public SignatureType FunctionType { get; }
    /// <summary>Gets the method selected for invocation.</summary>
    public MethodBuilder Target { get; }
}

public sealed partial class MethodBuilder
{
    /// <summary>Pushes a Function value bound to an owned method; consumes its object receiver for an instance target.</summary>
    /// <param name="functionType">The exact structural shape.</param>
    /// <param name="target">Owned nongeneric static, final/nonvirtual reference-instance or interface target.</param>
    /// <exception cref="ArgumentException">Invalid target, shape, foreign owner or scope.</exception>
    /// <exception cref="ArgumentNullException">An operand is null.</exception>
    public void BindFunction(SignatureType functionType, MethodBuilder target) => GetILGenerator().BindFunction(functionType, target);
    /// <summary>Emits a checked Function binding; consumes an instance receiver when required and pushes the callable value.</summary>
    /// <param name="opCode">BindFunction.</param>
    /// <param name="operand">The exact binding owned by this assembly.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, foreign target or invalid generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void Emit(OpCode opCode, FunctionBinding operand) => GetILGenerator().Emit(opCode, operand);
    /// <summary>Consumes a Function receiver followed by its arguments, then pushes its result if any.</summary>
    /// <param name="functionType">Exact structural Function type.</param>
    /// <exception cref="ArgumentException">Not a Function or invalid generic scope/owner.</exception>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validity is checked on write.</exception>
    public void InvokeFunction(SignatureType functionType) => GetILGenerator().InvokeFunction(functionType);
}
