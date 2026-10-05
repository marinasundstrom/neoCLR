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
    /// <summary>Gets the output's designated native Object root signature, or its explicit bootstrap Object reference.</summary>
    /// <remarks>Declare a native root before constructing signatures. CoreObjectType always retains its bootstrap meaning.
    /// This selects authoring identity only; runtime admission still requires explicit host configuration.</remarks>
    public SignatureType ObjectType => NativeObjectRoot is { } root ? root : CoreObjectType;

    internal TypeBuilder? NativeObjectRoot => types.SingleOrDefault(t => t.IsNativeObjectRoot);

    internal void ValidateNativeObjectSlots()
    {
        var root = NativeObjectRoot ?? throw new InvalidDataException("no authored native Object root");
        foreach (var name in new[] { "ToString", "Equals", "GetHashCode" })
            if (!root.Methods.Any(m => m.IsNativeObjectSlot && m.Name == name && MethodDefinition.IsNativeObjectSlot(name, m.Signature, root)))
                throw new InvalidDataException("native Object operation requires a complete root slot contract: " + name);
    }

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

public sealed partial class MethodDefinition
{
    internal static bool IsNativeObjectSlot(string name, MethodSignature signature, TypeBuilder? root = null)
    {
        if (signature.GenericParameterNames.Count != 0 || signature.OutParameters.Count != 0) return false;
        return name switch
        {
            "ToString" => signature.ReturnType == PrimitiveType.String && signature.ParameterTypes.Count == 0,
            "GetHashCode" => signature.ReturnType == PrimitiveType.Int32 && signature.ParameterTypes.Count == 0,
            "Equals" => signature.ReturnType == PrimitiveType.Boolean && signature.ParameterTypes.Count == 1 &&
                signature.ParameterTypes[0].ClassType is { IsNativeObjectRoot: true } owner && (root is null || ReferenceEquals(owner, root)),
            _ => false
        };
    }
}

public sealed partial class TypeBuilder
{
    /// <summary>Adds a concrete public virtual new slot to the explicit native Object root.</summary>
    /// <param name="name">ToString, Equals or GetHashCode.</param>
    /// <param name="signature">String ToString(), Boolean Equals(this root), or Int32 GetHashCode().</param>
    /// <returns>The method builder; use GetILGenerator to supply its body.</returns>
    /// <exception cref="ArgumentException">Invalid signature, duplicate method or declaration limit.</exception>
    /// <exception cref="InvalidOperationException">The owner is not the designated root.</exception>
    /// <remarks>Equivalent to attaching a MethodDefinition with Public, Virtual and NewSlot flags.
    /// These are declarations of slots, not overrides. Runtime admission still validates the complete root.</remarks>
    public MethodBuilder AddNativeObjectSlot(string name, MethodSignature signature)
    {
        if (!IsNativeObjectRoot) throw new InvalidOperationException("Object slots require the explicit native root");
        if (!MethodDefinition.IsNativeObjectSlot(name, signature)) throw new ArgumentException("unsupported native Object slot signature");
        var definition = new MethodDefinition(name, 0x146, signature);
        Definition.Methods.Add(definition);
        return MethodBuilder.ForDefinition(definition);
    }
}
