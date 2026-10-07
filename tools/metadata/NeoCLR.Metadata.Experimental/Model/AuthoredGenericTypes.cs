namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class TypeDefinition
{
    internal Dictionary<int, TypeParameterConstraints> SpecialConstraintStorage { get; } = [];
    internal List<GenericTypeConstraint> GenericConstraintStorage { get; } = [];
    private IReadOnlyDictionary<int, TypeParameterConstraints>? specialConstraintView;
    private IReadOnlyList<GenericTypeConstraint>? genericConstraintView;
    /// <summary>Gets a read-only view of authored special constraints, or the validated empty constraints of a supported native snapshot.</summary>
    /// <exception cref="NotSupportedException">CLI and unsupported native constraint materialization is pending.</exception>
    public IReadOnlyDictionary<int, TypeParameterConstraints> SpecialConstraints => GenericParameterNames is null
        ? throw new NotSupportedException("loaded generic constraints are not materialized yet")
        : specialConstraintView ??= new System.Collections.ObjectModel.ReadOnlyDictionary<int, TypeParameterConstraints>(SpecialConstraintStorage);
    /// <summary>Gets a read-only view of authored nominal constraints, or the validated empty constraints of a supported native snapshot.</summary>
    /// <exception cref="NotSupportedException">CLI and unsupported native constraint materialization is pending.</exception>
    public IReadOnlyList<GenericTypeConstraint> GenericConstraints => GenericParameterNames is null
        ? throw new NotSupportedException("loaded generic constraints are not materialized yet")
        : genericConstraintView ??= GenericConstraintStorage.AsReadOnly();
    /// <summary>Creates a detached generic type using a simple name and copied parameter names.</summary>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Simple name without CLI arity suffix.</param>
    /// <param name="attributes">Supported root-class, value-type or interface CLI flags.</param>
    /// <param name="baseType">Explicit core base reference, local native Object root for a reference class, or null for an interface.</param>
    /// <param name="genericParameterNames">One through 32 unique names; copied.</param>
    /// <exception cref="ArgumentNullException">Parameter sequence is null.</exception>
    /// <exception cref="ArgumentException">Invalid simple name or parameter names.</exception>
    public TypeDefinition(string @namespace, string name, uint attributes, TypeReference? baseType, IEnumerable<string> genericParameterNames)
        : this(@namespace, GenericIdentity(name, genericParameterNames), attributes, baseType) { }
    private TypeDefinition(string ns, (string Name, IReadOnlyList<string> Parameters) identity, uint attributes, TypeReference? baseType)
        : this(ns, identity.Name, attributes, baseType)
    {
        GenericParameterNames = identity.Parameters;
        GenericArity = identity.Parameters.Count;
    }
    private static (string, IReadOnlyList<string>) GenericIdentity(string name, IEnumerable<string> parameters)
    {
        ArgumentNullException.ThrowIfNull(parameters);
        var names = new MethodSignature(PrimitiveType.Void, Array.Empty<SignatureType>(), parameters).GenericParameterNames;
        if (string.IsNullOrEmpty(name) || name.Contains('`') || names.Count == 0)
            throw new ArgumentException("generic type requires a simple name and parameters");
        return (name + "`" + names.Count, names);
    }
    /// <summary>Gets authored or supported native parameter names in ordinal order; null for CLI snapshots whose names are not materialized.</summary>
    public IReadOnlyList<string>? GenericParameterNames { get; internal set; }
}
