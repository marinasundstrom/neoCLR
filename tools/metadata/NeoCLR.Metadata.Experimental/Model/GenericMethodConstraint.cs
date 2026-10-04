namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned nongeneric interface requirement on a method type parameter.</summary>
/// <param name="ParameterIndex">Zero-based method type parameter ordinal.</param>
/// <param name="InterfaceType">Nongeneric interface in the declaring module.</param>
public sealed record GenericMethodInterfaceConstraint(int ParameterIndex, TypeDefinition InterfaceType);

public sealed partial class MethodDefinition
{
    private readonly List<GenericMethodInterfaceConstraint> authoredInterfaceConstraints = [];
    private AssemblyDefinition.MethodConstraintRow[] loadedInterfaceConstraints = [];
    /// <summary>Gets method-scoped interface bounds, preserving parameter ordinals.</summary>
    /// <remarks>The initial contract supports owned nongeneric interfaces only.</remarks>
    /// <exception cref="InvalidDataException">Loaded metadata contains unsupported constraints; no partial constraint set is returned.</exception>
    public IReadOnlyList<GenericMethodInterfaceConstraint> InterfaceConstraints => unsupportedGenericParameters
        ? throw new InvalidDataException("method has unsupported generic parameter constraints")
        : AuthoredSignature is not null
        ? authoredInterfaceConstraints.AsReadOnly()
        : Array.AsReadOnly(loadedInterfaceConstraints.Select(c => new GenericMethodInterfaceConstraint(c.Parameter,
            Module.GetTypeDefinition(c.Interface) ?? throw new InvalidDataException("missing method constraint interface"))).ToArray());

    /// <summary>Adds an owned nongeneric interface bound to a method generic parameter.</summary>
    /// <exception cref="ArgumentException">Invalid ordinal, noninterface/generic bound, duplicate or foreign owner.</exception>
    /// <exception cref="ArgumentNullException">Interface is null.</exception>
    /// <exception cref="InvalidOperationException">The definition is a loaded immutable snapshot.</exception>
    public void AddInterfaceConstraint(int parameterIndex, TypeDefinition interfaceType)
    {
        ArgumentNullException.ThrowIfNull(interfaceType);
        if (AuthoredSignature is null) throw new InvalidOperationException("loaded method constraints are immutable");
        if (authoredInterfaceConstraints.Count >= 128 || parameterIndex < 0 || parameterIndex >= GenericArity || (interfaceType.Attributes & 0x20) == 0 || interfaceType.GenericArity != 0 ||
            Module is not null && !ReferenceEquals(Module, interfaceType.Module) ||
            authoredInterfaceConstraints.Any(c => c.ParameterIndex == parameterIndex && ReferenceEquals(c.InterfaceType, interfaceType)))
            throw new ArgumentException("invalid method interface constraint");
        authoredInterfaceConstraints.Add(new(parameterIndex, interfaceType));
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Gets the canonical definition's method interface bounds.</summary>
    public IReadOnlyList<GenericMethodInterfaceConstraint> InterfaceConstraints => Definition.InterfaceConstraints;
    /// <summary>Adds an owned nongeneric interface bound; definition and builder authoring share validation.</summary>
    public void AddInterfaceConstraint(int parameterIndex, TypeBuilder interfaceType)
        => Definition.AddInterfaceConstraint(parameterIndex, (interfaceType ?? throw new ArgumentNullException(nameof(interfaceType))).Definition);

    internal void ValidateMethodArguments(IReadOnlyList<SignatureType> arguments)
    {
        foreach (var constraint in InterfaceConstraints)
        {
            var argument = arguments[constraint.ParameterIndex];
            var type = argument.ClassType ?? (argument.Primitive is { } primitive
                ? Assembly.Types.SingleOrDefault(t => t.NativePrimitive == primitive) : null);
            if (type is null || !type.ConformsTo(constraint.InterfaceType.Producer!))
                throw new ArgumentException("generic method argument does not satisfy its interface bound");
        }
    }
}
