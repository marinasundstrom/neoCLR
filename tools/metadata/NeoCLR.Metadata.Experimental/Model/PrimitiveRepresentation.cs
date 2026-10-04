namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class TypeDefinition
{
    /// <summary>Gets the canonical numeric scalar implemented by this native declaration, or null for ordinary declarations.</summary>
    /// <remarks>CLI snapshots do not imply native primitive ownership merely from a System name.</remarks>
    public PrimitiveType? NativePrimitive { get; private set; }

    /// <summary>Designates a canonical System numeric value declaration as native runtime-owned scalar storage.</summary>
    /// <param name="primitive">One of the ten fixed-width integer or floating numeric primitives.</param>
    /// <exception cref="InvalidOperationException">The declaration is a loaded snapshot.</exception>
    /// <exception cref="ArgumentException">The identity/category/storage is incompatible or the designation conflicts.</exception>
    /// <remarks>No record fields, constructors, nesting or generic parameters are allowed. Bodies access the scalar through
    /// the managed receiver using ldobj/stobj. This does not authorize duplicate primitive ownership across dependencies.
    /// Native emission preserves existing Runtime representation; executable CLI emission rejects this designation.</remarks>
    public void SetNativePrimitive(PrimitiveType primitive)
    {
        if (authoredFields is null) throw new InvalidOperationException("loaded primitive declarations are immutable");
        if (!IsNumericPrimitive(primitive) || NativePrimitive is { } previous && previous != primitive ||
            Namespace != "System" || Name != primitive.ToString() || !IsValueType || GenericArity != 0 || DeclaringType is not null ||
            (Attributes & 0x1b8) != 0x108 || Fields.Count != 0 || Methods.Any(m => m.Name is ".ctor" or ".cctor"))
            throw new ArgumentException("native numeric declaration requires its canonical sealed sequential System value type with no record storage or constructors", nameof(primitive));
        NativePrimitive = primitive;
    }

    internal static bool IsNumericPrimitive(PrimitiveType primitive) => primitive is PrimitiveType.SByte or PrimitiveType.Byte or
        PrimitiveType.Int16 or PrimitiveType.UInt16 or PrimitiveType.Int32 or PrimitiveType.UInt32 or PrimitiveType.Int64 or
        PrimitiveType.UInt64 or PrimitiveType.Single or PrimitiveType.Double;
}

public sealed partial class TypeBuilder
{
    /// <summary>Gets the canonical numeric scalar implemented by this native declaration.</summary>
    public PrimitiveType? NativePrimitive => Definition.NativePrimitive;
    /// <summary>Designates runtime scalar storage through the definition's validation path.</summary>
    /// <param name="primitive">The canonical numeric primitive matching this System declaration.</param>
    /// <exception cref="ArgumentException">See TypeDefinition.SetNativePrimitive.</exception>
    public void SetNativePrimitive(PrimitiveType primitive) => Definition.SetNativePrimitive(primitive);
    internal void ValidatePrimitiveRepresentation()
    {
        if (NativePrimitive is { } primitive) Definition.SetNativePrimitive(primitive);
    }
}
