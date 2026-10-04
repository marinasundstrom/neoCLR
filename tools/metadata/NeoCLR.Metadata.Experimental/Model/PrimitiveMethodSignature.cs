namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Primitive signature types supported by the bounded metadata writer.</summary>
public enum PrimitiveType
{
    /// <summary>No result; invalid for parameters and locals.</summary>
    Void,
    /// <summary>A signed 32-bit integer.</summary>
    Int32,
    /// <summary>A Boolean value, distinct from Int32 in native metadata.</summary>
    Boolean,
    /// <summary>A signed 64-bit integer.</summary>
    Int64,
    /// <summary>Unicode text; native literals must be representable as UTF-8.</summary>
    String,
    /// <summary>An unsigned 8-bit storage/signature type; evaluation uses Int32.</summary>
    Byte,
    /// <summary>An IEEE 754 binary32 value.</summary>
    Single,
    /// <summary>An IEEE 754 binary64 value.</summary>
    Double,
    /// <summary>A signed 8-bit storage type; evaluation uses Int32.</summary>
    SByte,
    /// <summary>A signed 16-bit storage type; evaluation uses Int32.</summary>
    Int16,
    /// <summary>An unsigned 16-bit storage type; evaluation uses Int32.</summary>
    UInt16,
    /// <summary>An unsigned 32-bit storage type; evaluation uses Int32 bits.</summary>
    UInt32,
    /// <summary>An unsigned 64-bit storage type; evaluation uses Int64 bits.</summary>
    UInt64,
    /// <summary>An opaque runtime type identity; CLI uses System.RuntimeTypeHandle, native metadata uses RuntimeTypeHandle.</summary>
    RuntimeTypeHandle
}

/// <summary>An immutable nongeneric primitive signature whose declared parameters exclude any instance receiver.</summary>
public sealed class PrimitiveMethodSignature : MethodSignature
{
    /// <summary>Copies parameter types and validates the bounded signature.</summary>
    /// <param name="returnType">Any defined PrimitiveType, including Void for no result.</param>
    /// <param name="parameterTypes">At most 256 non-Void primitive parameters in order.</param>
    /// <exception cref="ArgumentNullException">Parameters are null.</exception>
    /// <exception cref="ArgumentException">Invalid type or too many parameters.</exception>
    public PrimitiveMethodSignature(PrimitiveType returnType, IEnumerable<PrimitiveType> parameterTypes)
        : base(returnType, parameterTypes)
    {
        ReturnType = returnType;
        ParameterTypes = Array.AsReadOnly(base.ParameterTypes.Select(p => p.Primitive!.Value).ToArray());
    }
    /// <summary>Gets the declared result type, including Void for no result.</summary>
    public new PrimitiveType ReturnType { get; }
    /// <summary>Gets the copied parameter types in declaration order.</summary>
    public new IReadOnlyList<PrimitiveType> ParameterTypes { get; }
    internal bool Matches(PrimitiveMethodSignature other)
        => ReturnType == other.ReturnType && ParameterTypes.SequenceEqual(other.ParameterTypes);
    internal static PrimitiveMethodSignature Int32(int count, bool result)
    {
        if (count is < 0 or > 256) throw new ArgumentException("invalid parameter count");
        return new(result ? PrimitiveType.Int32 : PrimitiveType.Void, Enumerable.Repeat(PrimitiveType.Int32, count));
    }
}
