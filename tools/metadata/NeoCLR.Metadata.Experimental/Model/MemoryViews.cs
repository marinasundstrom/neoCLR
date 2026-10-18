namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Opaque managed-memory view contracts understood by neoCLR.</summary>
public enum MemoryViewKind
{
    /// <summary>A borrowed writable view.</summary>
    Slice,
    /// <summary>A borrowed readonly view.</summary>
    ReadOnlySlice,
    /// <summary>An owner-backed writable view.</summary>
    Memory,
    /// <summary>An owner-backed readonly view.</summary>
    ReadOnlyMemory
}

public sealed partial class TypeDefinition
{
    /// <summary>Gets the explicit native memory-view contract, when present.</summary>
    public MemoryViewKind? MemoryView { get; internal set; }
}

public sealed partial class TypeBuilder
{
    /// <summary>Marks a canonical fieldless generic value type as an opaque memory view.</summary>
    /// <remarks>Native metadata only. The runtime owns representation and lifetime enforcement.</remarks>
    public void SetMemoryView(MemoryViewKind kind)
    {
        if (!Enum.IsDefined(kind)) throw new ArgumentOutOfRangeException(nameof(kind));
        if (Definition.MemoryView is { } previous && previous != kind)
            throw new InvalidOperationException("memory-view contract already selected");
        var original = Definition.MemoryView;
        Definition.MemoryView = kind;
        try { ValidateMemoryView(); }
        catch { Definition.MemoryView = original; throw; }
    }

    internal void ValidateMemoryView()
    {
        if (Definition.MemoryView is not { } kind) return;
        if (Namespace != "System" || (Name != kind.ToString() && Name != kind.ToString() + "`1") ||
            !IsValueType || IsInterface || IsStatic || GenericParameterNames.Count != 1 ||
            MetadataFields.Any() || GenericConstraints.Count != 0 ||
            implementedInterfaces.Count != 0 || constructedInterfaces.Count != 0 || externalInterfaces.Count != 0 ||
            Definition.GenericParameterVariance?.Any(v => v != GenericVariance.Invariant) == true ||
            SpecialConstraints.Values.Any(v => v != 0) || Definition.DeclaringType is not null)
            throw new InvalidOperationException("memory view requires a canonical fieldless generic value type");
    }
}
