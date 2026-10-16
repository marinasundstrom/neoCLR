namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class TypeDefinition
{
    /// <summary>Gets the CLI BeforeFieldInit flag; native initialization then occurs on first static-field access.</summary>
    public bool IsBeforeFieldInit => (Attributes & 0x100000) != 0;

    /// <summary>Sets relaxed field-triggered initialization, or strict method/construction timing when false.</summary>
    /// <exception cref="InvalidOperationException">The definition is not attached to an authored assembly.</exception>
    public void SetBeforeFieldInit(bool value = true)
    {
        if (Producer is null) throw new InvalidOperationException("initialization timing requires an attached authored definition");
        Attributes = value ? Attributes | 0x100000u : Attributes & ~0x100000u;
    }
}

public sealed partial class TypeBuilder
{
    /// <summary>Sets the owner’s initialization timing through its definition. Strict timing is the default.</summary>
    /// <param name="value">True for relaxed first-static-field-access initialization; false for strict timing.</param>
    public void SetBeforeFieldInit(bool value = true) => Definition.SetBeforeFieldInit(value);
}
