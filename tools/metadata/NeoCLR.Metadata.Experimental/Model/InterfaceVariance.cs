namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Declared generic interface parameter variance in native metadata.</summary>
/// <remarks>Annotations do not establish assignability or prove signature safety; the runtime validates declarations.</remarks>
public enum GenericVariance
{
    /// <summary>Requires the same type argument.</summary>
    Invariant,
    /// <summary>Permits admitted reference conversions in the output direction.</summary>
    Covariant,
    /// <summary>Permits admitted reference conversions in the input direction.</summary>
    Contravariant
}

public sealed partial class TypeDefinition
{
    /// <summary>Gets native generic parameter variance in ordinal order, or null when this snapshot does not materialize it.</summary>
    /// <remarks>Native snapshots return an immutable arity-sized list, normalizing omitted annotations to Invariant.
    /// CLI snapshots and authored definitions currently return null. This is declaration metadata, not a conversion API.
    /// Variant native definitions cannot yet be imported for emission or projected to CLI without losing their contract.
    /// Signature safety and executable conversion support remain runtime validation responsibilities.</remarks>
    public IReadOnlyList<GenericVariance>? GenericParameterVariance { get; }
}
