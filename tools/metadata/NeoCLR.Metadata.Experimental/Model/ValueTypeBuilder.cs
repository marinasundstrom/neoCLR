namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    /// <summary>Adds a sealed sequential-layout value type with primitive fields and static methods.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Nonempty metadata name.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>An owned value-type definition; no instance constructor is synthesized.</returns>
    /// <exception cref="ArgumentException">Invalid identity, duplicate type or exceeded limit.</exception>
    /// <remarks>Instance methods, field mutation through addresses, generic/nominal field storage and interface implementations are not yet supported.</remarks>
    public TypeBuilder AddValueType(string @namespace, string name, TypeVisibility visibility = TypeVisibility.Public)
        => AddTypeCore(@namespace, name, visibility, false, isValueType: true);

    /// <summary>Adds an invariant unconstrained generic value type.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Simple name without an arity suffix.</param>
    /// <param name="genericParameterNames">One through 32 unique names, copied.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>An owned value-type definition with primitive field storage only.</returns>
    /// <exception cref="ArgumentNullException">Parameter names are null.</exception>
    /// <exception cref="ArgumentException">Invalid identity/parameters, duplicate type or exceeded limit.</exception>
    public TypeBuilder AddGenericValueType(string @namespace, string name, IEnumerable<string> genericParameterNames, TypeVisibility visibility = TypeVisibility.Public)
        => AddGenericTypeCore(@namespace, name, genericParameterNames, visibility, false, isValueType: true);
}
