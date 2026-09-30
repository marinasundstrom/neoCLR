namespace NeoCLR.Metadata.Experimental;

/// <summary>An immutable owned section of an experimental NEOX envelope.</summary>
public sealed class MetadataSection
{
    private readonly byte[] payload;

    /// <summary>Creates a section, copying its payload. Does not decode payload semantics.</summary>
    /// <param name="kind">Nonzero section kind.</param>
    /// <param name="version">Nonzero schema version.</param>
    /// <param name="required">Whether the consumer must explicitly support this schema.</param>
    /// <param name="payload">Bytes copied into private storage, limited to MaxImageSize.</param>
    /// <exception cref="ArgumentOutOfRangeException">Kind/version is zero or payload exceeds the size limit.</exception>
    public MetadataSection(ushort kind, ushort version, bool required, ReadOnlySpan<byte> payload)
        : this(kind, version, required, payload, MetadataEnvelope.MaxImageSize) { }

    internal MetadataSection(ushort kind, ushort version, bool required, ReadOnlySpan<byte> payload, int maxImageSize)
    {
        if (kind == 0) throw new ArgumentOutOfRangeException(nameof(kind));
        if (version == 0) throw new ArgumentOutOfRangeException(nameof(version));
        if (payload.Length > maxImageSize) throw new ArgumentOutOfRangeException(nameof(payload));
        Kind = kind;
        Version = version;
        Required = required;
        this.payload = payload.ToArray();
    }

    /// <summary>Gets the section kind; unique within an envelope.</summary>
    public ushort Kind { get; }
    /// <summary>Gets the section schema version.</summary>
    public ushort Version { get; }
    /// <summary>Gets whether the schema must be supported by the consuming layer.</summary>
    public bool Required { get; }
    /// <summary>Gets the payload's byte length.</summary>
    public int PayloadLength => payload.Length;
    /// <summary>Returns a new caller-owned payload copy.</summary>
    /// <returns>A copy that can be modified without affecting this section.</returns>
    public byte[] GetPayload() => (byte[])payload.Clone();

    internal ReadOnlySpan<byte> Payload => payload;
}
