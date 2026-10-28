namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class TypeDefinition
{
    private readonly Dictionary<int, IList<CustomAttributeDefinition>> genericParameterAttributes = [];
    internal IEnumerable<KeyValuePair<int, IList<CustomAttributeDefinition>>> GenericParameterAttributes => genericParameterAttributes;
    /// <summary>Gets attributes on a zero-based generic parameter. Loaded lists are read-only.</summary>
    /// <exception cref="ArgumentOutOfRangeException">The ordinal is outside the declaration's generic arity.</exception>
    public IList<CustomAttributeDefinition> GetGenericParameterCustomAttributes(int position)
    {
        if (position < 0 || position >= GenericArity) throw new ArgumentOutOfRangeException(nameof(position));
        if (!genericParameterAttributes.TryGetValue(position, out var attributes))
        {
            attributes = authoredFields is not null ? new DefinitionCollection<CustomAttributeDefinition>([], attribute =>
            {
                if (genericParameterAttributes[position].Count >= 256) throw new InvalidDataException("too many generic parameter attributes");
                attribute.ValidateOwner(Module);
            }) : Array.AsReadOnly(Array.Empty<CustomAttributeDefinition>());
            genericParameterAttributes.Add(position, attributes);
        }
        return attributes;
    }
    internal void SetLoadedGenericParameterAttributes(int position, IEnumerable<CustomAttributeDefinition> attributes)
        => genericParameterAttributes[position] = Array.AsReadOnly(attributes.ToArray());
}

public sealed partial class MethodDefinition
{
    private readonly Dictionary<int, IList<CustomAttributeDefinition>> genericParameterAttributes = [];
    internal IEnumerable<KeyValuePair<int, IList<CustomAttributeDefinition>>> GenericParameterAttributes => genericParameterAttributes;
    /// <summary>Gets attributes on a zero-based generic parameter. Loaded lists are read-only.</summary>
    /// <exception cref="ArgumentOutOfRangeException">The ordinal is outside the declaration's generic arity.</exception>
    public IList<CustomAttributeDefinition> GetGenericParameterCustomAttributes(int position)
    {
        if (position < 0 || position >= GenericArity) throw new ArgumentOutOfRangeException(nameof(position));
        if (!genericParameterAttributes.TryGetValue(position, out var attributes))
        {
            attributes = AuthoredSignature is not null ? new DefinitionCollection<CustomAttributeDefinition>([], attribute =>
            {
                if (genericParameterAttributes[position].Count >= 256) throw new InvalidDataException("too many generic parameter attributes");
                attribute.ValidateOwner(Module);
            }) : Array.AsReadOnly(Array.Empty<CustomAttributeDefinition>());
            genericParameterAttributes.Add(position, attributes);
        }
        return attributes;
    }
    internal void SetLoadedGenericParameterAttributes(int position, IEnumerable<CustomAttributeDefinition> attributes)
        => genericParameterAttributes[position] = Array.AsReadOnly(attributes.ToArray());
}
