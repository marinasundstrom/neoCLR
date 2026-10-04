namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class MethodDefinition
{
    private int? parameterArrayIndex;
    /// <summary>Gets the zero-based final parameter carrying ParamArrayAttribute, or null.</summary>
    public int? ParameterArrayIndex => parameterArrayIndex;
    /// <summary>Marks the final by-value vector parameter as a parameter array.</summary>
    /// <remarks>The declaration changes call-site expansion metadata, not the physical array signature or calling convention.</remarks>
    /// <exception cref="InvalidOperationException">This is a loaded or detached method.</exception>
    /// <exception cref="ArgumentException">The index is not the final by-value array parameter.</exception>
    public void SetParameterArray(int position)
    {
        if (Producer is null) throw new InvalidOperationException("attach an authored method before changing parameter metadata");
        if (position < 0 || position != Producer.ParameterCount - 1 || Producer.Signature.ParameterTypes[position].ArrayElement is null)
            throw new ArgumentException("parameter array requires the final by-value vector parameter", nameof(position));
        parameterArrayIndex = position;
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Marks the final by-value array parameter through its owned definition.</summary>
    /// <exception cref="ArgumentException">The index or signature is unsupported.</exception>
    public void SetParameterArray(int position) => Definition.SetParameterArray(position);
}
