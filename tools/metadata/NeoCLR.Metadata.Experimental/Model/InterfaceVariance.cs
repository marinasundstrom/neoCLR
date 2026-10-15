namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Declared generic interface parameter variance in native metadata.</summary>
/// <remarks>Annotations do not establish assignability or prove signature safety; the runtime validates declarations.</remarks>
public enum GenericVariance
{
    /// <summary>Requires the same type argument.</summary>
    Invariant,
    /// <summary>Permits admitted reference conversions in the output direction.</summary>
    Covariant,
    /// <summary>Permits admitted reference conversions in the input direction.</summary>
    Contravariant
}

public sealed partial class TypeDefinition
{
    /// <summary>Gets native generic parameter variance in ordinal order, or null when this snapshot does not materialize it.</summary>
    /// <remarks>Native snapshots return an immutable arity-sized list, normalizing omitted annotations to Invariant.
    /// CLI snapshots and authored generic definitions also retain annotations. Null means unavailable.
    /// This is declaration metadata, not a conversion API.
    /// Signature safety and executable conversion support remain runtime validation responsibilities.</remarks>
    public IReadOnlyList<GenericVariance>? GenericParameterVariance { get; internal set; }
}

public sealed partial class TypeBuilder
{
    /// <summary>Declares the variance of one generic interface parameter; unannotated parameters are invariant.</summary>
    /// <param name="parameterIndex">Zero-based declaring-type parameter ordinal.</param>
    /// <param name="variance">Invariant, Covariant or Contravariant.</param>
    /// <exception cref="ArgumentException">Invalid ordinal, annotation or noninterface variant owner.</exception>
    /// <remarks>Writing validates nested signature positions. Writable aliases remain invariant.</remarks>
    public void SetVariance(int parameterIndex, GenericVariance variance)
    {
        if (parameterIndex < 0 || parameterIndex >= GenericParameterNames.Count || !Enum.IsDefined(variance) ||
            !IsInterface && variance != GenericVariance.Invariant)
            throw new ArgumentException("variance requires a valid generic interface parameter");
        var values = Definition.GenericParameterVariance?.ToArray() ?? new GenericVariance[GenericParameterNames.Count];
        values[parameterIndex] = variance;
        Definition.GenericParameterVariance = Array.AsReadOnly(values);
    }

    internal void ValidateVariance()
    {
        var annotations = Definition.GenericParameterVariance;
        if (annotations is null || annotations.All(v => v == GenericVariance.Invariant)) return;
        if (!IsInterface || annotations.Count != GenericParameterNames.Count)
            throw new InvalidDataException("invalid interface variance declaration");
        void Position(SignatureType type, int polarity)
        {
            if (type.IsSelf) throw new InvalidDataException("variant interfaces cannot use Self");
            if (type.TypeParameterIndex is { } ordinal)
            {
                var annotation = annotations[ordinal];
                if (annotation == GenericVariance.Covariant && polarity != 1 ||
                    annotation == GenericVariance.Contravariant && polarity != -1)
                    throw new InvalidDataException("unsafe interface variance position");
            }
            if (type.GenericInstance is { } instance)
                for (int i = 0; i < instance.TypeArguments.Count; i++)
                {
                    var direction = instance.Definition.Definition.GenericParameterVariance?[i] ?? GenericVariance.Invariant;
                    Position(instance.TypeArguments[i], direction == GenericVariance.Covariant ? polarity : direction == GenericVariance.Contravariant ? -polarity : 0);
                }
            if (type.ImportedType is { } imported)
                for (int i = 0; i < imported.TypeArguments.Count; i++)
                {
                    var direction = Assembly.ImportedVariance(imported, i);
                    Position(imported.TypeArguments[i], direction == GenericVariance.Covariant ? polarity : direction == GenericVariance.Contravariant ? -polarity : 0);
                }
            if (type.ArrayElement is { } element) Position(element, 0);
            if (type.ByReferenceElement is { } reference) Position(reference, 0);
            if (type.PointerElement is { } pointer) Position(pointer, 0);
            if (type.FunctionSignature is { } function)
                foreach (var part in function.ParameterTypes.Append(function.ReturnType)) Position(part, 0);
            if (type.Composition is { } composition)
                foreach (var part in composition.Constituents) Position(part, 0);
            if (type.Tuple is { } tuple)
                foreach (var part in tuple.ElementTypes) Position(part, 0);
        }
        foreach (var parent in InterfaceSignatures) Position(parent, 1);
        foreach (var method in Methods)
        {
            if (method.IsStatic || !method.IsAbstract || method.Signature.GenericParameterNames.Count != 0)
                throw new InvalidDataException("variant interfaces require bodyless instance contracts without method generics");
            foreach (var argument in method.Signature.ParameterTypes) Position(argument, -1);
            Position(method.Signature.ReturnType, 1);
        }
        foreach (var field in MetadataFields) Position(field.FieldType, 0);
    }
}

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<ImportedTypeReference, IReadOnlyList<GenericVariance>> importedVariance = [];
    internal GenericVariance ImportedVariance(ImportedTypeReference type, int ordinal)
        => importedVariance.TryGetValue(InterfaceDefinition(type), out var annotations)
            ? annotations[ordinal] : GenericVariance.Invariant;
}
