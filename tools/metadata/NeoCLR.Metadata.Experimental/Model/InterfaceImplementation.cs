namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An interface inheritance or implementation relationship.</summary>
/// <remarks>Authored relationships admit same-assembly constructed arguments. Loaded native relationships retain same-assembly generic interface arguments and their declaring type scope.</remarks>
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
    private readonly Lazy<IList<InterfaceImplementation>>? nativeInterfaces;
    /// <summary>Gets append-only authored or read-only materialized native interface relationships.</summary>
    /// <exception cref="NotSupportedException">Relationship materialization for loaded CLI snapshots is pending.</exception>
    /// <remarks>Attach the owner to a module first. Authored appends validate ownership, target category, duplicates and cycles. Native collections are read-only with canonical declaring/target definitions.</remarks>
    public IList<InterfaceImplementation> Interfaces => authoredInterfaces ?? nativeInterfaces?.Value
        ?? throw new NotSupportedException("loaded CLI interface relationships are not materialized yet");
}
