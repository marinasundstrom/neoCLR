namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An authored interface inheritance or implementation relationship.</summary>
/// <remarks>Same-assembly nongeneric inheritance and root-class implementations of nongeneric or closed generic interfaces are admitted.</remarks>
public sealed class InterfaceImplementation
{
    /// <summary>Creates an unattached relationship to an interface reference.</summary>
    /// <exception cref="ArgumentNullException">Interface type is null.</exception>
    public InterfaceImplementation(TypeReference interfaceType)
    {
        ArgumentNullException.ThrowIfNull(interfaceType);
        InterfaceType = interfaceType;
    }
    /// <summary>Creates a relationship to a constructed generic interface.</summary>
    /// <param name="interfaceType">Reference to the owned open interface definition.</param>
    /// <param name="typeArguments">Copied type arguments; validated when attached.</param>
    /// <exception cref="ArgumentNullException">A required argument is null.</exception>
    public InterfaceImplementation(TypeReference interfaceType, IEnumerable<SignatureType> typeArguments) : this(interfaceType)
    {
        ArgumentNullException.ThrowIfNull(typeArguments);
        TypeArguments = Array.AsReadOnly(typeArguments.Take(33).ToArray());
    }
    /// <summary>Gets copied constructed arguments, empty for a nongeneric relationship.</summary>
    public IReadOnlyList<SignatureType> TypeArguments { get; } = Array.Empty<SignatureType>();
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
