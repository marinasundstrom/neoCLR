namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Primitive signature types supported by the bounded metadata writer.</summary>
public enum PrimitiveType
{
    /// <summary>No result; invalid for parameters and locals.</summary>
    Void,
    /// <summary>A signed 32-bit integer.</summary>
    Int32,
    /// <summary>A Boolean value, distinct from Int32 in native metadata.</summary>
    Boolean
}

/// <summary>An immutable static nongeneric primitive method signature.</summary>
public sealed class PrimitiveMethodSignature
{
    /// <summary>Copies parameter types and validates the bounded signature.</summary>
    /// <param name="returnType">Void, Int32 or Boolean.</param>
    /// <param name="parameterTypes">At most 256 Int32/Boolean parameters in order.</param>
    /// <exception cref="ArgumentNullException">Parameters are null.</exception>
    /// <exception cref="ArgumentException">Invalid type or too many parameters.</exception>
    public PrimitiveMethodSignature(PrimitiveType returnType, IEnumerable<PrimitiveType> parameterTypes)
    {
        ArgumentNullException.ThrowIfNull(parameterTypes);
        var parameters = parameterTypes.Take(257).ToArray();
        if (!Enum.IsDefined(returnType) || parameters.Length > 256 || parameters.Any(p => p is not (PrimitiveType.Int32 or PrimitiveType.Boolean)))
            throw new ArgumentException("invalid primitive signature");
        ReturnType = returnType; ParameterTypes = Array.AsReadOnly(parameters);
    }
    /// <summary>Gets the declared result type, including Void for no result.</summary>
    public PrimitiveType ReturnType { get; }
    /// <summary>Gets the copied parameter types in declaration order.</summary>
    public IReadOnlyList<PrimitiveType> ParameterTypes { get; }
    internal bool Matches(PrimitiveMethodSignature other)
        => ReturnType == other.ReturnType && ParameterTypes.SequenceEqual(other.ParameterTypes);
    internal static PrimitiveMethodSignature Int32(int count, bool result)
    {
        if (count is < 0 or > 256) throw new ArgumentException("invalid parameter count");
        return new(result ? PrimitiveType.Int32 : PrimitiveType.Void, Enumerable.Repeat(PrimitiveType.Int32, count));
    }
}
