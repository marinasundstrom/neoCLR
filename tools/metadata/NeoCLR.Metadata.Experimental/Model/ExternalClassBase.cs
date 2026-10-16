namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly HashSet<ImportedTypeReference> fieldlessClassBases = [];
    private readonly Dictionary<ImportedTypeReference, ImportedMethodReference[]> externalClassContracts = [];
    private readonly Dictionary<ImportedTypeReference, int> externalClassFieldCounts = [];

    /// <summary>Completes a nongeneric external class base contract from resolved dependency member facts.</summary>
    /// <param name="type">Owned public, nonsealed, nongeneric top-level class reference.</param>
    /// <param name="methods">All declared public instance methods, including abstract/virtual slots and property accessors; excludes constructors.</param>
    /// <param name="declaredInstanceFieldCount">The dependency class’s own instance storage count, including private fields; 0–256.</param>
    /// <remarks>The host supplies validated symbol facts, including direct base and interface edges. Fields remain owned by the dependency and are laid out only after linking its exact artifact. This does not copy storage or infer it from names.</remarks>
    /// <exception cref="ArgumentException">Foreign, unsupported, duplicate or conflicting contracts.</exception>
    /// <exception cref="ArgumentOutOfRangeException">The declared instance storage count is outside 0–256.</exception>
    public void CompleteClassBaseReference(ImportedTypeReference type, IEnumerable<ImportedMethodReference> methods, int declaredInstanceFieldCount)
    {
        ArgumentNullException.ThrowIfNull(type); ArgumentNullException.ThrowIfNull(methods);
        if (!ReferenceEquals(type.Owner, this) || type.IsValueType || IsAuthoredInterface(type) ||
            type.GenericArity != 0 || type.TypeArguments.Count != 0 || type.DeclaringType is not null)
            throw new ArgumentException("external base requires a nongeneric top-level class reference", nameof(type));
        if (declaredInstanceFieldCount is < 0 or > 256) throw new ArgumentOutOfRangeException(nameof(declaredInstanceFieldCount));
        var members = methods.ToArray();
        if (members.Length > 4096 || members.Distinct().Count() != members.Length || members.Any(m =>
            m is null || !ReferenceEquals(m.Owner, this) || !Equals(m.DeclaringReference, type) || m.IsStatic || m.IsConstructor))
            throw new ArgumentException("invalid external class member contract", nameof(methods));
        if (externalClassContracts.TryGetValue(type, out var existing))
        {
            if (!existing.SequenceEqual(members) || externalClassFieldCounts[type] != declaredInstanceFieldCount) throw new ArgumentException("conflicting external class contract");
            return;
        }
        externalClassContracts.Add(type, members);
        externalClassFieldCounts.Add(type, declaredInstanceFieldCount);
    }

    internal bool HasClassBaseContract(ImportedTypeReference type) => fieldlessClassBases.Contains(type) || externalClassContracts.ContainsKey(type);

    internal int ExternalClassFieldCount(ImportedTypeReference type)
    {
        int count = 0;
        for (ImportedTypeReference? current = type; current is not null; current = DeclaredClassBase(current))
        {
            if (!HasClassBaseContract(current)) throw new InvalidDataException("external ancestor requires a complete layout contract");
            count = checked(count + externalClassFieldCounts.GetValueOrDefault(current));
        }
        return count;
    }

    internal IEnumerable<MethodBuilder> ExternalClassMethods(ImportedTypeReference type)
    {
        for (ImportedTypeReference? current = type; current is not null; current = DeclaredClassBase(current))
            if (externalClassContracts.TryGetValue(current, out var methods))
                foreach (var method in methods) yield return method.Target;
    }

    internal IEnumerable<SignatureType> ExternalClassInterfaces(ImportedTypeReference type)
    {
        for (ImportedTypeReference? current = type; current is not null; current = DeclaredClassBase(current))
            if (authoredInterfaceBases.TryGetValue(current, out var interfaces))
                foreach (var contract in interfaces) yield return contract;
    }

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

    /// <summary>Adds a class with an explicitly declared external base contract.</summary>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Unique metadata name.</param>
    /// <param name="baseType">An output-owned base registered with CompleteClassBaseReference or DeclareFieldlessClassBase.</param>
    /// <param name="visibility">Public or Internal visibility.</param>
    /// <returns>The owned derived class.</returns>
    /// <exception cref="ArgumentException">Invalid visibility, name, ownership or undeclared base contract.</exception>
    public TypeBuilder AddClass(string @namespace, string name, ImportedTypeReference baseType, TypeVisibility visibility = TypeVisibility.Public)
    {
        ArgumentNullException.ThrowIfNull(baseType);
        if (!Enum.IsDefined(visibility) || !HasClassBaseContract(baseType))
            throw new ArgumentException("external base requires a complete explicit contract", nameof(baseType));
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
