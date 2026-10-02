namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class TypeDefinition
{
    internal TypeDefinition? AuthoredDeclaringType { get; set; }
    private IList<TypeDefinition>? nestedTypes;
    /// <summary>Gets immediate nested definitions. Authored owners accept append-only detached definitions; loaded snapshots are read-only.</summary>
    /// <remarks>Authored nesting currently requires nongeneric owners, empty child namespaces, and NestedPublic or NestedAssembly visibility. Module.Types remains the complete physical declaration list.</remarks>
    public IList<TypeDefinition> NestedTypes => nestedTypes ??= MetadataToken != 0
        ? Array.AsReadOnly(Module.Types.Where(t => ReferenceEquals(t.DeclaringType, this)).ToArray())
        : new DefinitionCollection<TypeDefinition>([], child =>
        {
            if (Producer is null) throw new InvalidOperationException("attach the enclosing type before adding nested definitions");
            if (child.Producer is not null || child.MetadataToken != 0 || child.AuthoredDeclaringType is not null ||
                child.Namespace.Length != 0 || GenericArity != 0 ||
                (child.Attributes & 7) is not (2 or 5))
                throw new ArgumentException("nested definition requires detached type, empty namespace and supported nested visibility");
            int depth = 1;
            for (var owner = this; owner is not null; owner = owner.DeclaringType)
                if (++depth > 16 || ReferenceEquals(owner, child)) throw new ArgumentException("cyclic or excessive nested ownership");
            child.AuthoredDeclaringType = this;
            try { Module.Types.Add(child); }
            catch { child.AuthoredDeclaringType = null; throw; }
        });
}

public sealed partial class TypeBuilder
{
    /// <summary>Adds a nongeneric nested value type with explicit lexical ownership.</summary>
    /// <param name="name">Simple metadata name, unique within this owner.</param>
    /// <param name="visibility">Public or Internal (CLI NestedPublic or NestedAssembly).</param>
    /// <returns>The builder over the attached nested definition.</returns>
    /// <exception cref="ArgumentException">Invalid name, generic owner, duplicate, unsupported visibility or nesting/type limit.</exception>
    public TypeBuilder AddNestedValueType(string name, TypeVisibility visibility = TypeVisibility.Public) => AddNested(name, visibility, true);
    /// <summary>Adds a nongeneric nested root class with explicit lexical ownership.</summary>
    /// <param name="name">Simple metadata name, unique within this owner.</param>
    /// <param name="visibility">Public or Internal (CLI NestedPublic or NestedAssembly).</param>
    /// <returns>The builder over the attached nested definition.</returns>
    /// <exception cref="ArgumentException">Invalid name, generic owner, duplicate, unsupported visibility or nesting/type limit.</exception>
    public TypeBuilder AddNestedClass(string name, TypeVisibility visibility = TypeVisibility.Public) => AddNested(name, visibility, false);
    /// <summary>Adds a generic nested value type under a nongeneric enclosing owner.</summary>
    /// <param name="name">Simple name without an arity suffix.</param>
    /// <param name="parameters">One to 32 unique generic parameter names; copied.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>The attached value-type builder.</returns>
    /// <exception cref="ArgumentException">Invalid parameters, name, ownership or visibility.</exception>
    /// <exception cref="ArgumentNullException">Parameters are null.</exception>
    public TypeBuilder AddNestedGenericValueType(string name, IEnumerable<string> parameters, TypeVisibility visibility = TypeVisibility.Public)
        => AddNested(name, visibility, true, parameters ?? throw new ArgumentNullException(nameof(parameters)));
    private TypeBuilder AddNested(string name, TypeVisibility visibility, bool value, IEnumerable<string>? parameters = null)
    {
        if (visibility is not (TypeVisibility.Public or TypeVisibility.Internal)) throw new ArgumentException("unsupported nested visibility", nameof(visibility));
        var attributes = (visibility == TypeVisibility.Public ? 2u : 5u) | (value ? 0x108u : 0u);
        var baseType = Assembly.Definition.MainModule.ImportReference(Assembly.CoreLibrary, "System", value ? "ValueType" : "Object");
        var definition = parameters is null ? new TypeDefinition("", name, attributes, baseType)
            : new TypeDefinition("", name, attributes, baseType, parameters);
        Definition.NestedTypes.Add(definition);
        return definition.Producer!;
    }
}
