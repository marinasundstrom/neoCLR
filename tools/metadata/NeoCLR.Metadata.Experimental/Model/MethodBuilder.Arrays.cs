namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class MethodBuilder
{
    /// <summary>Appends a typed vector operation or addressed-local operation.</summary>
    /// <param name="opCode">Newarr, ReserveArray, Ldelem, Stelem, Initobj, Ldobj, Stobj, Castclass, or Callvirt for a Function signature.</param>
    /// <param name="elementType">Supported non-Void signature type; vector operations require scalar elements.</param>
    /// <exception cref="ArgumentNullException">Element is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, unsupported element or foreign owner.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validation occurs on write.</exception>
    public void Emit(OpCode opCode, SignatureType elementType) => GetILGenerator().Emit(opCode, elementType);

    internal static bool IsReferenceSignature(SignatureType type) => type.ArrayElement is not null ||
        type.ClassType is { IsValueType: false, IsStatic: false } || type.GenericInstance?.Definition is { IsValueType: false, IsStatic: false } ||
        type.ImportedType is { IsValueType: false };
    /// <summary>Consumes a reference and pushes the same object through a checked target reference signature.</summary>
    /// <param name="target">Owned or imported nominal reference or vector type.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="ArgumentException">Target is not a supported reference or belongs to another assembly.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; input stack validation occurs on write.</exception>
    /// <remarks>CLI uses castclass; native uses castclass and its verifier/runtime conversion contract. This does not box values.</remarks>
    public void CastReference(SignatureType target) => GetILGenerator().CastReference(target);

    /// <summary>Consumes an Int32 length and creates a default-initialized reference vector.</summary>
    /// <param name="elementType">Supported scalar element; see Emit(OpCode, SignatureType).</param>
    public void NewArray(SignatureType elementType) => GetILGenerator().NewArray(elementType);
    /// <summary>Consumes an Int32 length and reserves checked uninitialized vector elements.</summary>
    /// <param name="elementType">Supported scalar element, including caller-scoped generic parameters.</param>
    /// <exception cref="ArgumentNullException">Element is null.</exception>
    /// <exception cref="ArgumentException">Unsupported element or invalid owner scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack checked on native write.</exception>
    /// <remarks>Native-only operation. Reads before writes fault at runtime. Executable CLI writing rejects this operation; PE/#Neo reference projections remain supported.</remarks>
    public void ReserveArray(SignatureType elementType) => GetILGenerator().ReserveArray(elementType);
    /// <summary>Consumes an array and Int32 index, then pushes the element.</summary>
    /// <param name="elementType">Exact element identity.</param>
    public void LoadArrayElement(SignatureType elementType) => GetILGenerator().LoadArrayElement(elementType);
    /// <summary>Consumes an array, Int32 index and element, then stores the element.</summary>
    /// <param name="elementType">Exact element identity.</param>
    public void StoreArrayElement(SignatureType elementType) => GetILGenerator().StoreArrayElement(elementType);
    /// <summary>Consumes an array and loads its length normalized to Int32 using ldlen and conv.i4.</summary>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack validation occurs on write.</exception>
    public void LoadArrayLength() => GetILGenerator().LoadArrayLength();
}
