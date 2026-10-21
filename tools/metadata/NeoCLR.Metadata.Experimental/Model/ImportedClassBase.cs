namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<ImportedTypeReference, ImportedTypeReference> declaredClassBases = [];

    /// <summary>Declares an existing dependency's direct class base for output-side reference conversion validation.</summary>
    /// <param name="type">An output-owned, unconstructed top-level external class definition.</param>
    /// <param name="baseType">Its direct base signature, with arguments scoped to the derived definition.</param>
    /// <exception cref="ArgumentNullException">Either reference is null.</exception>
    /// <exception cref="ArgumentException">Foreign, value, interface, unbound base, nested, cyclic, out-of-scope or conflicting relationship.</exception>
    /// <exception cref="InvalidDataException">More than 4096 relationships are declared.</exception>
    /// <remarks>The host supplies truthful symbol facts; no reader is reopened. This does not author a new inherited definition or change dependency metadata. Native linking/verifying uses the actual dependency definitions. Generic base arguments are substituted through each declared ancestry edge; constructed classes remain invariant. This contract does not enable authoring generic class layouts.</remarks>
    public void DeclareClassBase(ImportedTypeReference type, ImportedTypeReference baseType)
    {
        ArgumentNullException.ThrowIfNull(type); ArgumentNullException.ThrowIfNull(baseType);
        bool Class(ImportedTypeReference value) => ReferenceEquals(value.Owner, this) && !value.IsValueType &&
            !IsAuthoredInterface(value) && !nominalKinds.GetValueOrDefault(InterfaceDefinition(value)) && value.DeclaringType is null;
        bool Argument(SignatureType value) => value.TypeParameterIndex is { } index ? index < type.GenericArity
            : value.Primitive is { } primitive ? primitive != PrimitiveType.Void
            : value.ArrayElement is { } element ? Argument(element)
            : value.ClassType is { GenericParameterNames.Count: 0, IsStatic: false } ? true
            : value.GenericInstance is { } local ? local.TypeArguments.All(Argument)
            : value.ImportedType is { } imported && imported.TypeArguments.Count == imported.GenericArity && imported.TypeArguments.All(Argument);
        if (!Class(type) || type.TypeArguments.Count != 0 || !Class(baseType) ||
            baseType.TypeArguments.Count != baseType.GenericArity || !baseType.TypeArguments.All(Argument))
            throw new ArgumentException("invalid external class base signature");
        ((SignatureType)baseType).ValidateOwner(this, typeArity: type.GenericArity);
        if (declaredClassBases.TryGetValue(type, out var existing))
        {
            if (!existing.Equals(baseType)) throw new ArgumentException("conflicting external class base");
            return;
        }
        // Compare definitions, not constructions: A<T> : A<T[]> is a cycle too.
        for (var ancestor = InterfaceDefinition(baseType); ;)
        {
            if (ancestor.Equals(type)) throw new ArgumentException("cyclic external class base");
            if (!declaredClassBases.TryGetValue(ancestor, out var parent)) break;
            ancestor = InterfaceDefinition(parent);
        }
        if (declaredClassBases.Count >= 4096) throw new InvalidDataException("too many external class bases");
        declaredClassBases.Add(type, baseType);
    }

    internal ImportedTypeReference? DeclaredClassBase(ImportedTypeReference type)
    {
        if (!declaredClassBases.TryGetValue(InterfaceDefinition(type), out var parent)) return null;
        if (type.TypeArguments.Count == 0) return parent;
        return parent.Substitute(part => SubstituteClassArgument(type, part));
    }

    internal static SignatureType SubstituteClassArgument(ImportedTypeReference owner, SignatureType part)
    {
        if (owner.TypeArguments.Count == 0) return part;
        SignatureType Substitute(SignatureType value) => SubstituteClassArgument(owner, value);
        return part.PointerElement is { } pointer ? SignatureType.PointerTo(Substitute(pointer))
            : part.TypeParameterIndex is { } index ? owner.TypeArguments[index]
            : part.ArrayElement is { } element ? SignatureType.ArrayOf(Substitute(element))
            : part.ByReferenceElement is { } reference ? SignatureType.ByReference(Substitute(reference))
            : part.GenericInstance is { } local ? local.Definition.MakeGenericInstance(local.TypeArguments.Select(Substitute).ToArray())
            : part.ImportedType is { } imported ? imported.Substitute(Substitute) : part;
    }

    internal bool HasDeclaredClassBase(ImportedTypeReference type, ImportedTypeReference target)
    {
        while (DeclaredClassBase(type) is { } parent)
        {
            if (parent.Equals(target)) return true;
            type = parent;
        }
        return false;
    }
}
