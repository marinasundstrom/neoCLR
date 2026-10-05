namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class TypeDefinition
{
    /// <summary>Prevents subclasses using the ordinary CLI Sealed flag.</summary>
    /// <exception cref="InvalidOperationException">The definition is loaded, detached, abstract or not an ordinary reference class.</exception>
    public void SetSealedClass()
    {
        if (Producer is null || Producer.IsStatic || Producer.IsInterface || IsValueType || Producer.IsAbstract || IsClosedHierarchy)
            throw new InvalidOperationException("only attached concrete reference classes can be sealed");
        Attributes |= 0x100;
    }
}
public sealed partial class TypeBuilder
{
    /// <summary>Seals an ordinary reference class through its definition.</summary>
    /// <exception cref="InvalidOperationException">The definition is loaded, detached, abstract or not an ordinary reference class.</exception>
    public void SetSealedClass() => Definition.SetSealedClass();
}
