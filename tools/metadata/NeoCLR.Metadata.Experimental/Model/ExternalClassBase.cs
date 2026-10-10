namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly HashSet<ImportedTypeReference> fieldlessClassBases = [];

    /// <summary>Declares an explicitly imported public, nonsealed, fieldless class hierarchy as an external base contract.</summary>
    /// <remarks>The compiler host supplies validated dependency facts: no instance storage, generic/nested owners,
    /// interfaces or virtual slots beyond Object. Native linking still verifies the actual hierarchy.
    /// This bounded contract does not infer layout from a name or load a dependency.</remarks>
    /// <param name="type">An output-owned top-level nongeneric reference class.</param>
    /// <exception cref="ArgumentException">Foreign, generic, nested, value or interface reference.</exception>
    public void DeclareFieldlessClassBase(ImportedTypeReference type)
    {
        ArgumentNullException.ThrowIfNull(type);
        if (!ReferenceEquals(type.Owner, this) || type.IsValueType || IsAuthoredInterface(type) ||
            type.GenericArity != 0 || type.TypeArguments.Count != 0 || type.DeclaringType is not null)
            throw new ArgumentException("external base requires a nongeneric top-level class reference", nameof(type));
        fieldlessClassBases.Add(type);
    }

    /// <summary>Adds a class with an explicitly declared fieldless external base.</summary>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Unique metadata name.</param>
    /// <param name="baseType">An output-owned base registered with DeclareFieldlessClassBase.</param>
    /// <param name="visibility">Public or Internal visibility.</param>
    /// <returns>The owned derived class.</returns>
    /// <exception cref="ArgumentException">Invalid visibility, name, ownership or undeclared base contract.</exception>
    public TypeBuilder AddClass(string @namespace, string name, ImportedTypeReference baseType, TypeVisibility visibility = TypeVisibility.Public)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (!Enum.IsDefined(visibility) || !fieldlessClassBases.Contains(baseType))
            throw new ArgumentException("external base requires an explicit fieldless contract", nameof(baseType));
        var definition = new TypeDefinition(@namespace, name, visibility == TypeVisibility.Public ? 1u : 0u,
            Definition.MainModule.ImportReference(baseType.AssemblyIdentity, baseType.Namespace, baseType.Name))
        { AuthoredExternalBase = baseType };
        Definition.MainModule.Types.Add(definition);
        return definition.Producer!;
    }
}

public sealed partial class TypeDefinition
{
    internal ImportedTypeReference? AuthoredExternalBase { get; init; }
}

public sealed partial class TypeBuilder
{
    internal ImportedTypeReference? ExternalBase => Definition.AuthoredExternalBase;
    internal bool HasExplicitClassBase => LocalBase is not null || ExternalBase is not null;
    internal bool HasExternalBase(ImportedTypeReference target) => ExternalBase is { } parent
        ? parent.Equals(target) || Assembly.HasDeclaredClassBase(parent, target)
        : LocalBase?.HasExternalBase(target) == true;
    internal bool IsDirectBaseConstructor(MethodBuilder target, SignatureType? receiver = null)
        => ReferenceEquals(LocalBase, target.DeclaringType) && LocalBase is not null ||
            ExternalBase is { } external && Equals(receiver?.ImportedType, external) && target.IsConstructor &&
            target.Assembly.Identity.Equals(external.AssemblyIdentity) && target.DeclaringType?.Namespace == external.Namespace &&
            target.DeclaringType?.Name == external.Name;
}
