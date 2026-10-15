namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>The nominal membership operation of an experimental reference composition.</summary>
public enum ReferenceCompositionKind
{
    /// <summary>Membership in at least one constituent.</summary>
    Union,
    /// <summary>Membership in every constituent.</summary>
    Intersection
}

/// <summary>An immutable experimental native signature; storage requires explicit authoring or imported-contract admission.</summary>
public sealed class ReferenceCompositionSignature : IEquatable<ReferenceCompositionSignature>
{
    internal ReferenceCompositionSignature(ReferenceCompositionKind kind, SignatureType[] constituents)
    {
        Kind = kind;
        Constituents = Array.AsReadOnly(constituents);
    }
    /// <summary>Gets either-membership (Union) or all-membership (Intersection).</summary>
    public ReferenceCompositionKind Kind { get; }
    /// <summary>Gets normalized bounds in first-occurrence order; identity ignores this order.</summary>
    public IReadOnlyList<SignatureType> Constituents { get; }
    /// <summary>Compares kind and constituent identity without nominal-subtype reduction.</summary>
    public bool Equals(ReferenceCompositionSignature? other) => other is not null && Kind == other.Kind &&
        Constituents.Count == other.Constituents.Count && Constituents.ToHashSet().SetEquals(other.Constituents);
    /// <inheritdoc/>
    public override bool Equals(object? obj) => obj is ReferenceCompositionSignature other && Equals(other);
    /// <inheritdoc/>
    public override int GetHashCode()
    {
        var hash = 0;
        foreach (var type in Constituents) hash = unchecked(hash + type.GetHashCode());
        return HashCode.Combine(Kind, Constituents.Count, hash);
    }
    /// <summary>Returns a diagnostic display, not a serialized identity.</summary>
    public override string ToString() => Kind + "<" + string.Join(", ", Constituents) + ">";
}

public sealed partial record SignatureType
{
    /// <summary>Gets the opt-in native reference composition, or null. Storage requires explicit authoring or imported-contract admission.</summary>
    public ReferenceCompositionSignature? Composition { get; }
    internal bool ContainsComposition => Composition is not null ||
        GenericInstance?.TypeArguments.Any(t => t.ContainsComposition) == true ||
        ReferencedGenericInstance?.TypeArguments.Any(t => t.ContainsComposition) == true ||
        ImportedType?.TypeArguments.Any(t => t.ContainsComposition) == true ||
        ArrayElement?.ContainsComposition == true || ByReferenceElement?.ContainsComposition == true ||
        PointerElement?.ContainsComposition == true || FunctionSignature is { } function &&
        function.ParameterTypes.Append(function.ReturnType).Any(t => t.ContainsComposition);

    internal static SignatureType FromComposition(ReferenceCompositionKind kind, IEnumerable<SignatureType> types)
    {
        var flattened = types.SelectMany(t => t.Composition is { } c && c.Kind == kind ? c.Constituents : new[] { t }).Distinct().ToArray();
        return flattened.Length == 1 ? flattened[0] : new(null, null, composition: new(kind, flattened));
    }
}


public sealed partial class AssemblyBuilder
{
    /// <summary>Creates and admits an experimental union or intersection with nominal generic constituents for native signatures and storage.</summary>
    /// <param name="kind">Union or intersection membership.</param>
    /// <param name="types">Two to 256 output-owned constituents; copied and normalized.</param>
    /// <returns>The structural signature, or the surviving constituent after duplicate normalization.</returns>
    /// <exception cref="ArgumentException">A constituent is foreign, unsupported, a bare generic parameter, or a final intersection bound.</exception>
    /// <remarks>Native-only; CLI projection remains disabled. Union value alternatives use explicit boxing; intersections exclude values. Scoped parameters inside nominal generic constituents are checked at use sites. Function alternatives are admitted in unions with recursively checked signatures; function intersection bounds and pointer alternatives are not admitted.</remarks>
    public SignatureType CreateExperimentalReferenceComposition(ReferenceCompositionKind kind, params SignatureType[] types)
    {
        ArgumentNullException.ThrowIfNull(types);
        if (!Enum.IsDefined(kind) || types.Length is < 2 or > 256)
            throw new ArgumentException("composition requires a supported kind and two to 256 constituents", nameof(types));
        var nodes = 0;
        bool SupportedArgument(SignatureType type) => type.MethodParameterIndex is not null || type.TypeParameterIndex is not null || (type.Composition is { } composition ? composition.Constituents.All(SupportedArgument)
            : type.FunctionSignature is { } function ? function.ParameterTypes.All(SupportedArgument)
                && (function.ReturnType.Primitive == PrimitiveType.Void || SupportedArgument(function.ReturnType))
            : type.Primitive is { } primitive ? primitive != PrimitiveType.Void
            : type.ArrayElement is { } element ? SupportedArgument(element)
            : type.GenericInstance is { } instance ? instance.TypeArguments.All(SupportedArgument)
            : type.ClassType is { GenericParameterNames.Count: 0 }
            || type.ImportedType is { } imported && imported.GenericArity == imported.TypeArguments.Count && imported.TypeArguments.All(SupportedArgument));
        void Validate(SignatureType type, bool intersection, int depth)
        {
            if (type is null || ++nodes > 4096 || depth > 32 || !SupportedArgument(type))
                throw new ArgumentException("composition requires bounded nominal, function or scalar constituents", nameof(types));
            type.ValidateOwner(this, 32, 32);
            if (type.Composition is { } composition)
            {
                foreach (var bound in composition.Constituents)
                    Validate(bound, intersection || composition.Kind == ReferenceCompositionKind.Intersection, depth + 1);
                return;
            }
            var local = type.ClassType ?? type.GenericInstance?.Definition;
            var scalar = type.Primitive is { } primitive && primitive is not (PrimitiveType.Void or PrimitiveType.Value or PrimitiveType.RuntimeTypeHandle or PrimitiveType.String);
            if (intersection
                ? local is not { IsStatic: false, IsValueType: false } && type.ImportedType is not { IsValueType: false }
                : !scalar && type.FunctionSignature is null && local is not { IsStatic: false } && type.ImportedType is null)
                throw new ArgumentException("unsupported composition alternative; intersections require nominal references", nameof(types));
            if (intersection)
            {
                var extendable = local is not null ? (local.Definition.Attributes & 0x100) == 0
                    : nativeImportedDefinitions.TryGetValue(InterfaceDefinition(type.ImportedType!), out var definition)
                        && (definition.Attributes & 0x100) == 0;
                if (!extendable) throw new ArgumentException("intersection bounds must be resolved and extendable", nameof(types));
            }
        }
        foreach (var type in types) Validate(type, kind == ReferenceCompositionKind.Intersection, 0);
        var result = SignatureType.FromComposition(kind, types);
        RegisterCompositionStorage(result);
        return result;
    }

    private void ValidateAuthoredCompositionBounds()
    {
        void Check(SignatureType type, bool intersection)
        {
            if (type.Composition is { } composition)
            {
                foreach (var bound in composition.Constituents)
                    Check(bound, intersection || composition.Kind == ReferenceCompositionKind.Intersection);
            }
            else if (intersection && (type.ClassType ?? type.GenericInstance?.Definition) is { } local
                && (local.Definition.Attributes & 0x100) != 0)
                throw new InvalidDataException("intersection bound became final after composition construction");
        }
        foreach (var composition in experimentalStorageCompositions)
            foreach (var bound in composition.Constituents)
                Check(bound, composition.Kind == ReferenceCompositionKind.Intersection);
    }

    private readonly HashSet<ReferenceCompositionSignature> experimentalStorageCompositions = [];
    internal bool AdmitsCompositionStorage(ReferenceCompositionSignature composition) => experimentalStorageCompositions.Contains(composition);

    /// <summary>Admits the closed reference compositions in an owned imported result for native storage and generic substitution.</summary>
    /// <param name="producer">An experimental imported callable owned by this output.</param>
    /// <exception cref="ArgumentNullException">Producer is null.</exception>
    /// <exception cref="ArgumentException">Producer is foreign or its result contains no composition.</exception>
    /// <remarks>CLI projection remains disabled. The imported exact constituent contracts are retained; arbitrary structural authoring is not enabled.</remarks>
    public void EnableExperimentalCompositionStorage(ImportedMethodReference producer)
    {
        ArgumentNullException.ThrowIfNull(producer);
        if (!ReferenceEquals(producer.Owner, this) || !producer.Signature.ReturnType.ContainsComposition)
            throw new ArgumentException("requires an owned imported composition result", nameof(producer));
        RegisterCompositionStorage(producer.Signature.ReturnType);
    }
    internal void RegisterCompositionStorage(SignatureType type)
    {
        if (type.Composition is { } composition)
        {
            experimentalStorageCompositions.Add(composition);
            foreach (var bound in composition.Constituents) RegisterCompositionStorage(bound);
        }
        if (type.FunctionSignature is { } function)
            foreach (var part in function.ParameterTypes.Append(function.ReturnType)) RegisterCompositionStorage(part);
        if (type.ImportedType is { } imported) foreach (var argument in imported.TypeArguments) RegisterCompositionStorage(argument);
        if (type.GenericInstance is { } instance) foreach (var argument in instance.TypeArguments) RegisterCompositionStorage(argument);
    }
}
