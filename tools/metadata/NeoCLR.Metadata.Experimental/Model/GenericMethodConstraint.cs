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
    private readonly Dictionary<int, TypeParameterConstraints> specialConstraints = [];
    /// <summary>Gets native method parameter category constraints, keyed by ordinal.</summary>
    /// <exception cref="InvalidDataException">Loaded metadata contains unsupported generic constraints.</exception>
    public IReadOnlyDictionary<int, TypeParameterConstraints> SpecialConstraints => unsupportedGenericParameters
        ? throw new InvalidDataException("method has unsupported generic parameter constraints")
        : new System.Collections.ObjectModel.ReadOnlyDictionary<int, TypeParameterConstraints>(specialConstraints);

    /// <summary>Sets ordinary reference/value/constructor requirements or a standalone native FunctionType requirement; None clears flags.</summary>
    /// <exception cref="ArgumentException">Invalid ordinal, category or combination with a type bound.</exception>
    /// <exception cref="InvalidOperationException">The definition is a loaded immutable snapshot.</exception>
    public void SetSpecialConstraints(int parameterIndex, TypeParameterConstraints constraints)
    {
        if (AuthoredSignature is null) throw new InvalidOperationException("loaded method constraints are immutable");
        if (parameterIndex < 0 || parameterIndex >= GenericArity ||
            !TypeBuilder.ValidSpecialConstraintFlags(constraints) ||
            constraints != TypeParameterConstraints.None && (constraints == TypeParameterConstraints.FunctionType && TypeConstraints.Any(c => c.ParameterIndex == parameterIndex) ||
                !specialConstraints.ContainsKey(parameterIndex) && specialConstraints.Count + TypeConstraints.Count >= 128))
            throw new ArgumentException("invalid method category constraint");
        if (constraints == TypeParameterConstraints.None) specialConstraints.Remove(parameterIndex);
        else specialConstraints[parameterIndex] = constraints;
    }

    private readonly List<GenericMethodInterfaceConstraint> authoredInterfaceConstraints = [];
    internal readonly List<GenericMethodTypeConstraint> AuthoredTypeConstraints = [];
    internal IReadOnlyList<GenericMethodInterfaceConstraint> NominalInterfaceConstraints => authoredInterfaceConstraints;
    private AssemblyDefinition.MethodConstraintRow[] loadedInterfaceConstraints = [];
    private AssemblyDefinition.StructuralMethodConstraintRow[] loadedStructuralConstraints = [];

    /// <summary>Gets all supported method type bounds without losing structural grouping.</summary>
    /// <remarks>Structural bounds require an explicitly opted-in native snapshot. This query does not validate type arguments.</remarks>
    /// <exception cref="InvalidDataException">The method has unsupported generic constraints.</exception>
    public IReadOnlyList<GenericMethodTypeConstraint> TypeConstraints => unsupportedGenericParameters
        ? throw new InvalidDataException("method has unsupported generic parameter constraints")
        : Array.AsReadOnly((AuthoredSignature is not null
            ? authoredInterfaceConstraints.Select(c => new GenericMethodTypeConstraint(c.ParameterIndex, ConstraintSignature(c.InterfaceType))).Concat(AuthoredTypeConstraints)
            : loadedInterfaceConstraints.Select(c => new GenericMethodTypeConstraint(c.Parameter, Module.GetNativeSignatureType(c.Interface)))
                .Concat(loadedStructuralConstraints.Select(c => new GenericMethodTypeConstraint(c.Parameter, c.Bound.Materialize(Module)))))
            .ToArray());
    /// <summary>Gets method-scoped interface bounds, preserving parameter ordinals.</summary>
    /// <remarks>Supports nominal nongeneric interfaces only; use TypeConstraints for structural bounds. A structural bound causes this query to throw rather than return an incomplete set.</remarks>
    /// <exception cref="InvalidDataException">Loaded metadata contains unsupported constraints; no partial constraint set is returned.</exception>
    public IReadOnlyList<GenericMethodInterfaceConstraint> InterfaceConstraints => loadedInterfaceConstraints.Any(c => (c.Interface >> 24) == 2 && (Module.GetTypeDefinition(c.Interface)!.Attributes & 0x20) == 0) || specialConstraints.Count != 0 || unsupportedGenericParameters || loadedStructuralConstraints.Length != 0 || AuthoredTypeConstraints.Count != 0
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
        if (specialConstraints.GetValueOrDefault(parameterIndex) == TypeParameterConstraints.FunctionType || specialConstraints.Count + authoredInterfaceConstraints.Count + AuthoredTypeConstraints.Count >= 128 || parameterIndex < 0 || parameterIndex >= GenericArity ||
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
    /// <summary>Sets ordinary reference/value/constructor requirements or a standalone native FunctionType requirement; None clears flags.</summary>
    /// <exception cref="ArgumentException">Invalid ordinal, category or existing type bound.</exception>
    public void SetSpecialConstraints(int parameterIndex, TypeParameterConstraints constraints)
        => Definition.SetSpecialConstraints(parameterIndex, constraints);

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
    /// <remarks>Constituents must be output-owned closed reference types. External intersection constituents require imported definition snapshots proving extension eligibility. CLI emission and open structural operands are unsupported.</remarks>
    /// <exception cref="ArgumentException">Invalid ordinal, kind, duplicate, foreign/open/value/final intersection operand or excessive constraints.</exception>
    /// <exception cref="ArgumentNullException">Constituents are null.</exception>
    public void AddExperimentalStructuralConstraint(int parameterIndex, ReferenceCompositionKind kind, params SignatureType[] constituents)
    {
        ArgumentNullException.ThrowIfNull(constituents);
        if (Definition.SpecialConstraints.ContainsKey(parameterIndex) || parameterIndex < 0 || parameterIndex >= Signature.GenericParameterNames.Count || Definition.SpecialConstraints.Count + Definition.TypeConstraints.Count >= 128)
            throw new ArgumentException("invalid method constraint ordinal or count");
        var bound = Assembly.CreateExperimentalReferenceComposition(kind, constituents);
        if (bound.Composition is null)
            throw new ArgumentException("structural method bounds require at least two distinct constituents");
        if (Definition.TypeConstraints.Any(c => c.ParameterIndex == parameterIndex && c.Bound == bound))
            throw new ArgumentException("duplicate method type bound");
        Definition.AuthoredTypeConstraints.Add(new(parameterIndex, bound));
    }

    /// <summary>Adds one owned nongeneric nominal reference-class bound per method parameter.</summary>
    /// <remarks>Preserved by CLI and native metadata. Interface and structural bounds have separate authoring APIs.</remarks>
    /// <exception cref="ArgumentException">The ordinal, category, ownership or duplicate class bound is invalid.</exception>
    public void AddBaseTypeConstraint(int parameterIndex, SignatureType bound)
    {
        ArgumentNullException.ThrowIfNull(bound);
        bool valid = bound.ClassType is { IsValueType: false, IsInterface: false, IsStatic: false } local && local.GenericParameterNames.Count == 0 && ReferenceEquals(local.Assembly, Assembly)
            || bound.ImportedType is { IsValueType: false, GenericArity: 0 } imported && ReferenceEquals(imported.Owner, Assembly) && Assembly.IsKnownMethodClassBound(imported);
        if (!valid || parameterIndex < 0 || parameterIndex >= Signature.GenericParameterNames.Count ||
            Definition.SpecialConstraints.GetValueOrDefault(parameterIndex) == TypeParameterConstraints.FunctionType ||
            Definition.AuthoredTypeConstraints.Any(c => c.ParameterIndex == parameterIndex && c.Bound.Composition is null) ||
            Definition.SpecialConstraints.Count + Definition.TypeConstraints.Count >= 128)
            throw new ArgumentException("invalid method class bound");
        Definition.AuthoredTypeConstraints.Add(new(parameterIndex, bound));
    }

    internal AssemblyBuilder? ConstraintOwner { get; set; }

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
        // Scalar signatures retain the capabilities of their explicitly selected nominal owner.
        // Imported callable definitions live in a private graph, so use the consuming owner.
        if (actual.Primitive is { } primitive && (bound.ImportedType is not null || bound.ClassType is not null))
        {
            var owner = ConstraintOwner ?? Assembly;
            if (owner.Types.SingleOrDefault(t => t.NativePrimitive == primitive) is { } localPrimitive)
                actual = localPrimitive;
            else if (owner.ExternalPrimitive(primitive) is { } importedPrimitive)
                actual = importedPrimitive;
        }
        if (bound.ImportedType is { } external)
        {
            var owner = ConstraintOwner ?? Assembly;
            if (actual.ImportedType is { } imported)
                return owner.HasNativeInterfaceConversion(imported, external) || owner.HasDeclaredClassBase(imported, external);
            if (actual.ClassType is { } authored)
                return authored.HasExternalBase(external) || authored.InheritedContracts().Any(c =>
                    c == bound || c.ImportedType is { } contract && owner.HasNativeInterfaceConversion(contract, external));
        }
        if (actual.GenericInstance is { } instance)
            return bound.ClassType is { IsInterface: false } classBound ? instance.Definition.DerivesFrom(classBound)
                : bound.ImportedType is { } externalBound && instance.Definition.HasExternalBase(externalBound) || instance.ConformsTo(bound);
        if (actual.ClassType is not { } local) return false;
        return bound.ClassType is { } nominal ? nominal.IsInterface ? local.ConformsTo(nominal) : local.DerivesFrom(nominal)
            : bound.GenericInstance is { Definition.IsInterface: true } constructed && local.ConformsTo(constructed);
    }

    internal bool ForwardedBoundsSatisfy(MethodBuilder? caller, int parameter, SignatureType required)
    {
        if (caller is null || parameter < 0 || parameter >= caller.Signature.GenericParameterNames.Count) return false;
        var bounds = caller.Definition.TypeConstraints.Where(c => c.ParameterIndex == parameter).Select(c => c.Bound).ToArray();
        if (bounds.Length == 0) return false;
        var guarantee = bounds.Length == 1 ? bounds[0] : SignatureType.FromComposition(ReferenceCompositionKind.Intersection, bounds);
        return SatisfiesStructuralBound(guarantee, required);
    }

    internal void ValidateMethodArguments(IReadOnlyList<SignatureType> arguments, MethodBuilder? caller = null, bool deferOpen = false)
    {
        foreach (var constraint in Definition.SpecialConstraints)
        {
            var argument = arguments[constraint.Key];
            if (argument.MethodParameterIndex is not null || argument.TypeParameterIndex is not null)
            {
                if (caller is null && deferOpen) continue;
                var supplied = argument.MethodParameterIndex is { } parameter
                    ? caller?.Definition.SpecialConstraints.GetValueOrDefault(parameter) ?? TypeParameterConstraints.None
                    : caller?.DeclaringType?.SpecialConstraints.GetValueOrDefault(argument.TypeParameterIndex!.Value) ?? TypeParameterConstraints.None;
                if (supplied.HasFlag(TypeParameterConstraints.ValueType)) supplied |= TypeParameterConstraints.DefaultConstructor;
                if ((supplied & constraint.Value) == constraint.Value) continue;
                throw new ArgumentException("forwarded generic argument lacks required category constraints");
            }
            TypeBuilder.ValidateSpecialTypeArgument(argument, constraint.Value, complete: true);
        }
        foreach (var constraint in Definition.AuthoredTypeConstraints)
        {
            var argument = arguments[constraint.ParameterIndex];
            if (argument.MethodParameterIndex is { } parameter)
            {
                if (caller is null && deferOpen) continue;
                if (ForwardedBoundsSatisfy(caller, parameter, constraint.Bound)) continue;
                throw new ArgumentException("forwarded method argument lacks required structural bounds");
            }
            if (argument.TypeParameterIndex is { } ownerParameter)
            {
                if (caller is null && deferOpen) continue;
                if (caller?.DeclaringType?.GenericConstraints.Any(c => c.ParameterIndex == ownerParameter && SatisfiesStructuralBound(c.BaseType, constraint.Bound)) == true) continue;
                throw new ArgumentException("forwarded declaring-type argument lacks required nominal bound");
            }
            if (!SatisfiesStructuralBound(argument, constraint.Bound))
                throw new ArgumentException("generic method argument does not satisfy its structural bound");
        }
        foreach (var constraint in Definition.NominalInterfaceConstraints)
        {
            var argument = arguments[constraint.ParameterIndex];
            if (argument.MethodParameterIndex is { } parameter)
            {
                if (caller is null && deferOpen) continue;
                var required = Definition.ConstraintSignature(constraint.InterfaceType);
                if (caller?.Definition.AuthoredTypeConstraints.Any(c => c.ParameterIndex == parameter) == true)
                {
                    if (ForwardedBoundsSatisfy(caller, parameter, required)) continue;
                    throw new ArgumentException("forwarded method argument lacks required interface bounds");
                }
                if (caller is not null && parameter < caller.Signature.GenericParameterNames.Count &&
                    caller.Definition.NominalInterfaceConstraints.Any(c => c.ParameterIndex == parameter &&
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

public sealed partial class AssemblyBuilder
{
    internal bool IsKnownMethodClassBound(ImportedTypeReference type)
        => nominalKinds.TryGetValue(InterfaceDefinition(type), out var isInterface) && !isInterface;
}
