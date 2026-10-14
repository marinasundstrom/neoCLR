namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A local or external nongeneric interface requirement on a method type parameter.</summary>
/// <param name="ParameterIndex">Zero-based method type parameter ordinal.</param>
/// <param name="InterfaceType">Nongeneric interface reference scoped to the declaring module.</param>
public sealed record GenericMethodInterfaceConstraint(int ParameterIndex, TypeReference InterfaceType);

/// <summary>A method type-parameter bound represented by its complete metadata signature.</summary>
/// <param name="ParameterIndex">Zero-based method type parameter ordinal.</param>
/// <param name="Bound">Nominal or explicitly admitted structural type signature.</param>
public sealed record GenericMethodTypeConstraint(int ParameterIndex, SignatureType Bound);

public sealed partial class MethodDefinition
{
    private readonly List<GenericMethodInterfaceConstraint> authoredInterfaceConstraints = [];
    internal readonly List<GenericMethodTypeConstraint> AuthoredStructuralConstraints = [];
    internal IReadOnlyList<GenericMethodInterfaceConstraint> NominalInterfaceConstraints => authoredInterfaceConstraints;
    private AssemblyDefinition.MethodConstraintRow[] loadedInterfaceConstraints = [];
    private AssemblyDefinition.StructuralMethodConstraintRow[] loadedStructuralConstraints = [];

    /// <summary>Gets all supported method type bounds without losing structural grouping.</summary>
    /// <remarks>Structural bounds require an explicitly opted-in native snapshot. This query does not validate type arguments.</remarks>
    /// <exception cref="InvalidDataException">The method has unsupported generic constraints.</exception>
    public IReadOnlyList<GenericMethodTypeConstraint> TypeConstraints => unsupportedGenericParameters
        ? throw new InvalidDataException("method has unsupported generic parameter constraints")
        : Array.AsReadOnly((AuthoredSignature is not null
            ? authoredInterfaceConstraints.Select(c => new GenericMethodTypeConstraint(c.ParameterIndex, ConstraintSignature(c.InterfaceType))).Concat(AuthoredStructuralConstraints)
            : loadedInterfaceConstraints.Select(c => new GenericMethodTypeConstraint(c.Parameter, Module.GetNativeSignatureType(c.Interface)))
                .Concat(loadedStructuralConstraints.Select(c => new GenericMethodTypeConstraint(c.Parameter, c.Bound.Materialize(Module)))))
            .ToArray());
    /// <summary>Gets method-scoped interface bounds, preserving parameter ordinals.</summary>
    /// <remarks>Supports nominal nongeneric interfaces only; use TypeConstraints for structural bounds. A structural bound causes this query to throw rather than return an incomplete set.</remarks>
    /// <exception cref="InvalidDataException">Loaded metadata contains unsupported constraints; no partial constraint set is returned.</exception>
    public IReadOnlyList<GenericMethodInterfaceConstraint> InterfaceConstraints => unsupportedGenericParameters || loadedStructuralConstraints.Length != 0 || AuthoredStructuralConstraints.Count != 0
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
        if (authoredInterfaceConstraints.Count + AuthoredStructuralConstraints.Count >= 128 || parameterIndex < 0 || parameterIndex >= GenericArity ||
            TypeConstraints.Any(c => c.ParameterIndex == parameterIndex && c.Bound == signature))
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

    /// <summary>Adds a native-only closed structural bound on a method type parameter.</summary>
    /// <remarks>Constituents must be output-owned local reference types. CLI emission, external bounds and open structural operands are unsupported.</remarks>
    /// <exception cref="ArgumentException">Invalid ordinal, kind, duplicate, foreign/open/value/final intersection operand or excessive constraints.</exception>
    /// <exception cref="ArgumentNullException">Constituents are null.</exception>
    public void AddExperimentalStructuralConstraint(int parameterIndex, ReferenceCompositionKind kind, params SignatureType[] constituents)
    {
        ArgumentNullException.ThrowIfNull(constituents);
        if (parameterIndex < 0 || parameterIndex >= Signature.GenericParameterNames.Count || Definition.TypeConstraints.Count >= 128)
            throw new ArgumentException("invalid method constraint ordinal or count");
        bool Local(SignatureType type) => type.Composition is { } c ? c.Constituents.All(Local)
            : type.ClassType is not null || type.GenericInstance is not null;
        if (constituents.Any(t => t is null || !Local(t)))
            throw new ArgumentException("structural method bounds require local reference constituents");
        var bound = Assembly.CreateExperimentalReferenceComposition(kind, constituents);
        if (bound.Composition is null)
            throw new ArgumentException("structural method bounds require at least two distinct constituents");
        if (Definition.TypeConstraints.Any(c => c.ParameterIndex == parameterIndex && c.Bound == bound))
            throw new ArgumentException("duplicate method type bound");
        Definition.AuthoredStructuralConstraints.Add(new(parameterIndex, bound));
    }

    private bool SatisfiesStructuralBound(SignatureType actual, SignatureType bound, int depth = 0)
    {
        if (depth >= 64) return false;
        if (actual.Composition is { Kind: ReferenceCompositionKind.Union } sourceUnion)
            return sourceUnion.Constituents.All(t => SatisfiesStructuralBound(t, bound, depth + 1));
        if (bound.Composition is { Kind: ReferenceCompositionKind.Intersection } targetIntersection)
            return targetIntersection.Constituents.All(t => SatisfiesStructuralBound(actual, t, depth + 1));
        if (bound.Composition is { Kind: ReferenceCompositionKind.Union } targetUnion)
            return targetUnion.Constituents.Any(t => SatisfiesStructuralBound(actual, t, depth + 1));
        if (actual.Composition is { Kind: ReferenceCompositionKind.Intersection } sourceIntersection)
            return sourceIntersection.Constituents.Any(t => SatisfiesStructuralBound(t, bound, depth + 1));
        if (actual == bound) return true;
        if (actual.GenericInstance is { } instance) return instance.ConformsTo(bound);
        if (actual.ClassType is not { } local) return false;
        return bound.ClassType is { } nominal ? nominal.IsInterface ? local.ConformsTo(nominal) : local.DerivesFrom(nominal)
            : bound.GenericInstance is { Definition.IsInterface: true } constructed && local.ConformsTo(constructed);
    }

    internal void ValidateMethodArguments(IReadOnlyList<SignatureType> arguments, MethodBuilder? caller = null, bool deferOpen = false)
    {
        foreach (var constraint in Definition.AuthoredStructuralConstraints)
        {
            var argument = arguments[constraint.ParameterIndex];
            if (argument.MethodParameterIndex is not null && caller is null && deferOpen) continue;
            if (!SatisfiesStructuralBound(argument, constraint.Bound))
                throw new ArgumentException("generic method argument does not satisfy its structural bound; open forwarding is unsupported");
        }
        foreach (var constraint in Definition.NominalInterfaceConstraints)
        {
            var argument = arguments[constraint.ParameterIndex];
            if (argument.MethodParameterIndex is { } parameter)
            {
                if (caller is null && deferOpen) continue;
                var required = Definition.ConstraintSignature(constraint.InterfaceType);
                if (caller is not null && parameter < caller.Signature.GenericParameterNames.Count &&
                    caller.InterfaceConstraints.Any(c => c.ParameterIndex == parameter &&
                        (required.ClassType is { } requiredLocal ? caller.Definition.ConstraintSignature(c.InterfaceType).ClassType?.ConformsTo(requiredLocal) == true
                         : Assembly.SatisfiesConstrainedBound(caller.Definition.ConstraintSignature(c.InterfaceType), required.ImportedType!, argument)))) continue;
                throw new ArgumentException("forwarded method argument lacks required interface bounds");
            }
            var type = argument.ClassType ?? (argument.Primitive is { } primitive
                ? Assembly.Types.SingleOrDefault(t => t.NativePrimitive == primitive) : null);
            var bound = Definition.ConstraintSignature(constraint.InterfaceType);
            var actual = type is not null ? (SignatureType)type : argument.Primitive is { } scalar && Assembly.ExternalPrimitive(scalar) is { } external ? (SignatureType)external : argument;
            if (!(bound.ClassType is { } local ? type?.ConformsTo(local) == true : Assembly.SatisfiesInterfaceBound(actual, bound.ImportedType!)))
                throw new ArgumentException("generic method argument does not satisfy its interface bound");
        }
    }
}
