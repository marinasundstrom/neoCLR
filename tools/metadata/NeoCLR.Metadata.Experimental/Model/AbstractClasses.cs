namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class TypeDefinition
{
    /// <summary>Marks an ordinary class abstract using the CLI Abstract flag.</summary>
    /// <remarks>Abstract classes may contain concrete methods and protected constructors. This does not close the hierarchy or add virtual slots.</remarks>
    /// <exception cref="InvalidOperationException">The definition is detached, loaded, sealed, generic, nested or not an ordinary reference class.</exception>
    public void SetAbstractClass()
    {
        if (Producer is null || Producer.IsStatic || Producer.IsInterface || IsValueType ||
            (Attributes & 0x100) != 0 || GenericArity != 0 || DeclaringType is not null || IsNativeObjectRoot || IsClosedHierarchy)
            throw new InvalidOperationException("only attached nongeneric top-level ordinary reference classes can be marked abstract");
        Attributes |= 0x80;
    }
}

public sealed partial class TypeBuilder
{
    /// <summary>Marks an ordinary class abstract through its canonical definition.</summary>
    /// <exception cref="InvalidOperationException">The definition is detached, loaded, sealed, generic, nested or not an ordinary reference class.</exception>
    public void SetAbstractClass() => Definition.SetAbstractClass();
}
