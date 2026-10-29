namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial record SignatureType
{
    internal bool ContainsFunctionReturn => FunctionReturnOperand is not null ||
        ArrayElement?.ContainsFunctionReturn == true || ByReferenceElement?.ContainsFunctionReturn == true ||
        FunctionSignature is { } shape && shape.ParameterTypes.Append(shape.ReturnType).Any(t => t.ContainsFunctionReturn) ||
        GenericInstance?.TypeArguments.Any(t => t.ContainsFunctionReturn) == true ||
        ImportedType?.TypeArguments.Any(t => t.ContainsFunctionReturn) == true ||
        Tuple?.ElementTypes.Any(t => t.ContainsFunctionReturn) == true;

    /// <summary>Gets the function operand of an unresolved return-type projection.</summary>
    public SignatureType? FunctionReturnOperand { get; }

    /// <summary>Projects a structural function's return type, retaining open dependencies.</summary>
    /// <remarks>Generic operands must be constrained as functions by their declaration.
    /// A projection is a type expression, not a nominal generic type.</remarks>
    /// <exception cref="ArgumentException">The operand is not a function or an open type expression.</exception>
    public static SignatureType FunctionReturn(SignatureType functionType)
    {
        ArgumentNullException.ThrowIfNull(functionType);
        if (functionType.FunctionSignature is { } shape) return shape.ReturnType;
        if (functionType.TypeParameterIndex is null && functionType.MethodParameterIndex is null)
            throw new ArgumentException("return-type projection requires a function type", nameof(functionType));
        if (functionType.NestingDepth >= 16) throw new ArgumentException("type nesting exceeds limit", nameof(functionType));
        return new(null, null, functionReturn: functionType);
    }
}
