namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class TypeDefinition
{
    /// <summary>Gets whether this authored declaration explicitly supplies the native Object root.</summary>
    /// <remarks>This authoring flag is not runtime admission and is not inferred on loaded snapshots.</remarks>
    public bool IsNativeObjectRoot { get; private set; }

    /// <summary>Designates a public, abstract, nongeneric System.Object as a baseless native root.</summary>
    /// <remarks>Call before attachment for manual definitions, or through an attached class builder.
    /// The declaration must have no storage or closed-family designation. Its ordinary bootstrap Object
    /// base is removed. The host must separately select and validate this root when loading assemblies.
    /// CLI output is a reference-only projection; executable .NET output is rejected.</remarks>
    /// <exception cref="InvalidOperationException">The declaration is a loaded snapshot.</exception>
    /// <exception cref="ArgumentException">The declaration has an incompatible shape or base.</exception>
    public void SetNativeObjectRoot()
    {
        ValidateNativeObjectRoot();
        if (BaseType is { } parent && (Producer is null || !Equals(parent.ExplicitScope, Producer.Assembly.CoreLibrary) ||
            parent.Namespace != "System" || parent.Name != "Object"))
            throw new ArgumentException("native Object root cannot replace an explicit class base");
        authoredBaseType = null;
        IsNativeObjectRoot = true;
    }

    internal void ValidateNativeObjectRoot()
    {
        if (authoredFields is null) throw new InvalidOperationException("loaded Object declarations are immutable");
        if (Namespace != "System" || Name != "Object" || Attributes != 0x81 || GenericArity != 0 ||
            DeclaringType is not null || IsValueType || NativePrimitive is not null || IsClosedHierarchy || Fields.Count != 0)
            throw new ArgumentException("native Object root requires a public abstract nongeneric top-level System.Object without storage");
    }
}

public sealed partial class AssemblyBuilder
{
    /// <summary>Adds the explicit baseless native System.Object declaration to this output.</summary>
    /// <returns>The attached abstract root builder; methods must be supplied by the caller.</returns>
    /// <exception cref="ArgumentException">The output already declares System.Object or exceeds declaration limits.</exception>
    /// <remarks>Uses the same validation as manually authored definitions. This does not select a runtime root.</remarks>
    public TypeBuilder AddNativeObjectRoot()
    {
        var definition = new TypeDefinition("System", "Object", 0x81, null);
        definition.SetNativeObjectRoot();
        Definition.MainModule.Types.Add(definition);
        return definition.Producer!;
    }
}

public sealed partial class TypeBuilder
{
    /// <summary>Gets the explicit native Object-root authoring designation.</summary>
    public bool IsNativeObjectRoot => Definition.IsNativeObjectRoot;

    /// <summary>Designates an eligible abstract System.Object through definition validation.</summary>
    /// <exception cref="ArgumentException">The declaration's shape or base is incompatible.</exception>
    public void SetNativeObjectRoot() => Definition.SetNativeObjectRoot();
}
