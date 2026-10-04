using NeoCLR.Metadata.Experimental.Model;

namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>A declared metadata property with projected signatures and canonical accessors; no value access is available.</summary>
public sealed class PropertyInfo
{
    internal PropertyInfo(MetadataLoadContext context, PropertyDefinition definition, TypeInfo owner, IReadOnlyList<TypeInfo> arguments)
    {
        if (!definition.TryGetSignature(out var type, out var indices, out var isStatic))
            throw new InvalidDataException("unsupported property metadata signature: " + definition.Name);
        Name = definition.Name;
        MetadataToken = definition.MetadataToken;
        DeclaringType = owner;
        IsStatic = isStatic;
        PropertyType = context.ResolveMemberSignature(type!, arguments, [], owner);
        IndexParameterTypes = Array.AsReadOnly(indices.Select(t => context.ResolveMemberSignature(t, arguments, [], owner)).ToArray());
        GetMethod = definition.GetMethod is { } getter ? context.GetMethod(getter, owner) : null;
        SetMethod = definition.SetMethod is { } setter ? context.GetMethod(setter, owner) : null;
    }
    /// <summary>Gets the metadata declaration name.</summary>
    public string Name { get; }
    /// <summary>Gets the original module-local property token.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the open or constructed owner through which the property was selected.</summary>
    public TypeInfo DeclaringType { get; }
    /// <summary>Gets the property type projected in the declaring owner's generic scope.</summary>
    public TypeInfo PropertyType { get; }
    /// <summary>Gets projected index types in declaration order, excluding a setter's value parameter.</summary>
    public IReadOnlyList<TypeInfo> IndexParameterTypes { get; }
    /// <summary>Gets the canonical getter, including non-public accessors; null when absent.</summary>
    public MethodInfo? GetMethod { get; }
    /// <summary>Gets the canonical setter, including non-public accessors; null when absent.</summary>
    public MethodInfo? SetMethod { get; }
    /// <summary>Gets whether the metadata accessors are static.</summary>
    public bool IsStatic { get; }
}

public sealed partial class MetadataLoadContext
{
    internal IReadOnlyList<PropertyInfo> ProjectProperties(TypeDefinition definition, TypeInfo owner, IReadOnlyList<TypeInfo> arguments)
        => Array.AsReadOnly(definition.Properties.Select(p => new PropertyInfo(this, p, owner, arguments)).ToArray());

    internal IReadOnlyList<TypeInfo> ProjectInterfaces(TypeDefinition definition, IReadOnlyList<TypeInfo> arguments, TypeInfo owner)
        => Array.AsReadOnly(definition.Interfaces.Select(relationship =>
        {
            var target = Resolve(relationship.InterfaceType);
            if (!target.IsInterface) throw new InvalidDataException("metadata interface relationship targets a non-interface");
            if (relationship.TypeArguments.Count == 0)
            {
                if (target.GenericArity != 0) throw new InvalidDataException("metadata interface relationship lacks generic arguments");
                return (TypeInfo)target;
            }
            try
            {
                return target.MakeGenericType(relationship.TypeArguments.Select(t => ResolveMemberSignature(t, arguments, [], owner)).ToArray());
            }
            catch (ArgumentException error)
            {
                throw new InvalidDataException("invalid metadata interface construction", error);
            }
        }).ToArray());
}
