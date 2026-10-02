namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class MethodBuilder
{
    /// <summary>Consumes an initialized owned local address or managed-reference parameter and loads its value.</summary>
    /// <param name="type">Exact non-Void local type, including scoped generic parameters.</param>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="ArgumentException">Invalid type, ownership or generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; address, type and definite-assignment checks run on write.</exception>
    public void LoadObject(SignatureType type) => GetILGenerator().LoadObject(type);

    /// <summary>Consumes an owned local address or managed-reference parameter followed by its value; local stores establish definite assignment.</summary>
    /// <param name="type">Exact non-Void local type, including scoped generic parameters.</param>
    /// <exception cref="ArgumentNullException">Type is null.</exception>
    /// <exception cref="ArgumentException">Invalid type, ownership or generic scope.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; address and type checks run on write.</exception>
    public void StoreObject(SignatureType type) => GetILGenerator().StoreObject(type);
}
