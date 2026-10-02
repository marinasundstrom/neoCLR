namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class MethodBuilder
{
    private IILGenerator? generator;
    /// <summary>Gets this authored method's stable body generator.</summary>
    /// <returns>A generator writing the same definition body and local/label scope as legacy builder operations.</returns>
    /// <remarks>Builder instruction methods remain compatibility entry points; new clients should use this contract.</remarks>
    public IILGenerator GetILGenerator() => generator ??= new MethodILGenerator(this);
}

public sealed partial class MethodDefinition
{
    /// <summary>Gets the body generator for an attached authored definition.</summary>
    /// <returns>The same generator returned by the declaration's builder.</returns>
    /// <exception cref="InvalidOperationException">The definition is detached or loaded rather than attached and authored.</exception>
    public IILGenerator GetILGenerator() => MethodBuilder.ForDefinition(this).GetILGenerator();
}
