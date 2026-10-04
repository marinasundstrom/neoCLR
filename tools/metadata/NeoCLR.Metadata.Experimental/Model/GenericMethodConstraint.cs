namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A local or external nongeneric interface requirement on a method type parameter.</summary>
/// <param name="ParameterIndex">Zero-based method type parameter ordinal.</param>
/// <param name="InterfaceType">Nongeneric interface reference scoped to the declaring module.</param>
public sealed record GenericMethodInterfaceConstraint(int ParameterIndex, TypeReference InterfaceType);

public sealed partial class MethodDefinition
{
    private readonly List<GenericMethodInterfaceConstraint> authoredInterfaceConstraints = [];
    private AssemblyDefinition.MethodConstraintRow[] loadedInterfaceConstraints = [];
    /// <summary>Gets method-scoped interface bounds, preserving parameter ordinals.</summary>
    /// <remarks>The initial contract supports local or external nongeneric interfaces only.</remarks>
    /// <exception cref="InvalidDataException">Loaded metadata contains unsupported constraints; no partial constraint set is returned.</exception>
    public IReadOnlyList<GenericMethodInterfaceConstraint> InterfaceConstraints => unsupportedGenericParameters
        ? throw new InvalidDataException("method has unsupported generic parameter constraints")
        : AuthoredSignature is not null
        ? authoredInterfaceConstraints.AsReadOnly()
        : Array.AsReadOnly(loadedInterfaceConstraints.Select(c => new GenericMethodInterfaceConstraint(c.Parameter,
            Module.GetNativeSignatureType(c.Interface).ReferencedType!)).ToArray());

    /// <summary>Adds an owned nongeneric interface bound to a method generic parameter.</summary>
    /// <exception cref="ArgumentException">Invalid ordinal, noninterface/generic bound, duplicate or foreign owner.</exception>
    /// <exception cref="ArgumentNullException">Interface is null.</exception>
    /// <exception cref="InvalidOperationException">The definition is a loaded immutable snapshot.</exception>
    public void AddInterfaceConstraint(int parameterIndex, TypeDefinition interfaceType)
    {
        ArgumentNullException.ThrowIfNull(interfaceType);
        AddInterfaceConstraint(parameterIndex, interfaceType.ToReference());
    }

    /// <summary>Adds a local or explicitly registered external nongeneric interface bound.</summary>
    /// <remarks>External references require a completed output-owned interface contract when writing.</remarks>
    /// <param name="parameterIndex">Zero-based method type parameter ordinal.</param>
    /// <param name="interfaceType">Local or explicitly scoped external interface reference.</param>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="ArgumentException">Ordinal, ownership, category or duplicate constraint is invalid.</exception>
    /// <exception cref="InvalidOperationException">The definition is a loaded snapshot.</exception>
    public void AddInterfaceConstraint(int parameterIndex, TypeReference interfaceType)
    {
        ArgumentNullException.ThrowIfNull(interfaceType);
        if (AuthoredSignature is null) throw new InvalidOperationException("loaded method constraints are immutable");
        var signature = ConstraintSignature(interfaceType);
        if (authoredInterfaceConstraints.Count >= 128 || parameterIndex < 0 || parameterIndex >= GenericArity ||
            authoredInterfaceConstraints.Any(c => c.ParameterIndex == parameterIndex && Equals(ConstraintSignature(c.InterfaceType), signature)))
            throw new ArgumentException("invalid method interface constraint");
        authoredInterfaceConstraints.Add(new(parameterIndex, interfaceType));
    }
    internal SignatureType ConstraintSignature(TypeReference reference)
    {
        if (reference.ExplicitScope is not null)
        {
            var imported = (Module?.Assembly.Producer ?? throw new ArgumentException("attach method before adding external bounds")).FindAuthoredInterface(reference);
            if (imported.GenericArity != 0) throw new ArgumentException("generic method interface bounds are unsupported");
            return imported;
        }
        var definition = reference.Resolve();
        if ((definition.Attributes & 0x20) == 0 || definition.GenericArity != 0 ||
            Module is not null && !ReferenceEquals(Module, definition.Module) || definition.Producer is null)
            throw new ArgumentException("invalid method interface bound");
        return definition.Producer;
    }

}

public sealed partial class MethodBuilder
{
    /// <summary>Gets the canonical definition's method interface bounds.</summary>
    public IReadOnlyList<GenericMethodInterfaceConstraint> InterfaceConstraints => Definition.InterfaceConstraints;
    /// <summary>Adds an owned nongeneric interface bound; definition and builder authoring share validation.</summary>
    public void AddInterfaceConstraint(int parameterIndex, TypeBuilder interfaceType)
        => Definition.AddInterfaceConstraint(parameterIndex, (interfaceType ?? throw new ArgumentNullException(nameof(interfaceType))).Definition);

    /// <summary>Adds a nongeneric output-owned external interface bound.</summary>
    public void AddInterfaceConstraint(int parameterIndex, ImportedTypeReference interfaceType)
    {
        ArgumentNullException.ThrowIfNull(interfaceType);
        if (!ReferenceEquals(interfaceType.Owner, Assembly)) throw new ArgumentException("foreign interface bound");
        Definition.AddInterfaceConstraint(parameterIndex, Assembly.Definition.MainModule.ImportReference(interfaceType.AssemblyIdentity, interfaceType.Namespace, interfaceType.Name));
    }

    internal void ValidateMethodArguments(IReadOnlyList<SignatureType> arguments)
    {
        foreach (var constraint in InterfaceConstraints)
        {
            var argument = arguments[constraint.ParameterIndex];
            var type = argument.ClassType ?? (argument.Primitive is { } primitive
                ? Assembly.Types.SingleOrDefault(t => t.NativePrimitive == primitive) : null);
            var bound = Definition.ConstraintSignature(constraint.InterfaceType);
            if (type is null || !(bound.ClassType is { } local ? type.ConformsTo(local) : type.InheritedContracts().Any(c => Equals(c, bound))))
                throw new ArgumentException("generic method argument does not satisfy its interface bound");
        }
    }
}
