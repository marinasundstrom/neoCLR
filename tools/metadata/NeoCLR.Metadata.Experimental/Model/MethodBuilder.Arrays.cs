namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class MethodBuilder
{
    /// <summary>Appends a typed vector allocation, element load or element store.</summary>
    /// <param name="opCode">Newarr, Ldelem or Stelem.</param>
    /// <param name="elementType">Non-Void scalar primitive or root class owned by this output.</param>
    /// <exception cref="ArgumentNullException">Element is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, unsupported element or foreign owner.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validation occurs on write.</exception>
    public void Emit(OpCode opCode, SignatureType elementType)
    {
        _ = SignatureType.ArrayOf(elementType);
        elementType.ValidateOwner(Assembly);
        Append(new(opCode switch {
            OpCode.Newarr => "array.new", OpCode.Ldelem => "array.load", OpCode.Stelem => "array.store",
            _ => throw OperandError(opCode)
        }, Type: elementType));
    }

    /// <summary>Consumes an Int32 length and creates a default-initialized reference vector.</summary>
    /// <param name="elementType">Supported scalar element; see Emit(OpCode, SignatureType).</param>
    public void NewArray(SignatureType elementType) => Emit(OpCode.Newarr, elementType);
    /// <summary>Consumes an array and Int32 index, then pushes the element.</summary>
    /// <param name="elementType">Exact element identity.</param>
    public void LoadArrayElement(SignatureType elementType) => Emit(OpCode.Ldelem, elementType);
    /// <summary>Consumes an array, Int32 index and element, then stores the element.</summary>
    /// <param name="elementType">Exact element identity.</param>
    public void StoreArrayElement(SignatureType elementType) => Emit(OpCode.Stelem, elementType);
    /// <summary>Consumes an array and loads its length normalized to Int32 using ldlen and conv.i4.</summary>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validation occurs on write.</exception>
    public void LoadArrayLength()
    {
        if (Instructions.Count > 4094) throw new InvalidDataException("instruction limit exceeded");
        Emit(OpCode.Ldlen); Emit(OpCode.Conv_I4);
    }
}
