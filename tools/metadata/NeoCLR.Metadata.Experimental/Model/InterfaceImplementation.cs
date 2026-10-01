namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An authored interface inheritance or implementation relationship.</summary>
/// <remarks>Only same-assembly, nongeneric interface definitions are currently admitted.</remarks>
public sealed class InterfaceImplementation
{
    /// <summary>Creates an unattached relationship to an interface reference.</summary>
    /// <exception cref="ArgumentNullException">Interface type is null.</exception>
    public InterfaceImplementation(TypeReference interfaceType)
    {
        ArgumentNullException.ThrowIfNull(interfaceType);
        InterfaceType = interfaceType;
    }
    /// <summary>Gets the exact target reference supplied by the caller.</summary>
    public TypeReference InterfaceType { get; }
    /// <summary>Gets the declaring class or interface, or null before attachment.</summary>
    public TypeDefinition? DeclaringType { get; internal set; }
}

public sealed partial class TypeDefinition
{
    private readonly IList<InterfaceImplementation>? authoredInterfaces;
    /// <summary>Gets append-only authored interface relationships.</summary>
    /// <exception cref="NotSupportedException">Relationship materialization for loaded snapshots is pending.</exception>
    /// <remarks>Attach the owner to a module first. Appends validate ownership, target category, duplicates and cycles.</remarks>
    public IList<InterfaceImplementation> Interfaces => authoredInterfaces ?? throw new NotSupportedException("loaded interface relationships are not materialized yet");
}
