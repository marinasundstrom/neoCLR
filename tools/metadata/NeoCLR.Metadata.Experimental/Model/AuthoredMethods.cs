using System.Reflection;

namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class MethodDefinition
{
    internal MethodBuilder? Producer { get; set; }
    internal bool IsTypeDeclaration { get; }
    private readonly string? authoredNamespace;
    internal MethodDefinition(MethodBuilder producer, string name, string ns, MethodSignature methodSignature, MethodVisibility visibility, bool isStatic)
    {
        Producer = producer;
        Module = producer.Assembly.Definition.MainModule;
        Name = name;
        authoredNamespace = ns;
        IsTypeDeclaration = producer.DeclaringType is not null;
        AuthoredSignature = methodSignature;
        GenericArity = methodSignature.GenericParameterNames.Count;
        declarationAttributes = (ushort)((visibility switch { MethodVisibility.Public => 6, MethodVisibility.Internal => 3, _ => 1 }) | (isStatic ? 0x10 : 0));
        signature = [];
    }
    /// <summary>Creates a detached static assembly-level function declaration.</summary>
    /// <param name="name">Simple metadata name, unique by namespace and parameter signature on attachment.</param>
    /// <param name="signature">Immutable supported signature, validated against the destination assembly on attachment.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <param name="namespace">Explicit function namespace; empty by default.</param>
    /// <exception cref="ArgumentException">Invalid name, namespace or visibility.</exception>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    public MethodDefinition(string name, MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public, string @namespace = "")
    {
        ArgumentNullException.ThrowIfNull(signature);
        FunctionNamespaceEncoding.Validate(@namespace);
        if (visibility is not (MethodVisibility.Public or MethodVisibility.Internal)) throw new ArgumentOutOfRangeException(nameof(visibility));
        if (string.IsNullOrEmpty(name) ||
            name.StartsWith(FunctionNamespaceEncoding.Prefix, StringComparison.Ordinal) || @namespace.Length + name.Length > 1024)
            throw new ArgumentException("invalid assembly function declaration");
        Name = name; authoredNamespace = @namespace; AuthoredSignature = signature;
        GenericArity = signature.GenericParameterNames.Count;
        declarationAttributes = (ushort)((visibility == MethodVisibility.Public ? 6 : 3) | 0x10);
        this.signature = [];
    }
    /// <summary>Creates a detached type method, constructor, bounded value override or abstract interface contract with CLI attributes.</summary>
    /// <param name="name">Nonempty name or .ctor; .cctor is unsupported. Unique by signature on attachment.</param>
    /// <param name="attributes">Public, Assembly or Private; optional Static and HideBySig. Constructors may use SpecialName and RTSpecialName together; public instance interface contracts require Abstract, Virtual and NewSlot together. A public value-type ToString override uses Virtual without Abstract or NewSlot.</param>
    /// <param name="signature">Supported signature validated against the destination type on attachment.</param>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Unsupported attributes or invalid name.</exception>
    /// <remarks>Append to an attached authored TypeDefinition.Methods collection. Instance methods require a class or value owner. Overrides currently require a value owner and the nongeneric parameterless String-returning ToString contract. Constructors require a nongeneric Void signature.</remarks>
    public MethodDefinition(string name, ushort attributes, MethodSignature signature)
    {
        ArgumentNullException.ThrowIfNull(signature);
        if (string.IsNullOrEmpty(name) || name.Length > 1024 || name == ".cctor" ||
            (attributes & ~0x1dd7) != 0 || (attributes & 7) is not (1 or 3 or 6))
            throw new ArgumentException("invalid type-method declaration");
        bool valueOverride = (attributes & 0x540) == 0x40;
        bool contract = !valueOverride && (attributes & 0x540) != 0;
        if (valueOverride && ((attributes & 0x17) != 6 || name != "ToString" ||
            signature.ReturnType != PrimitiveType.String || signature.ParameterTypes.Count != 0 || signature.GenericParameterNames.Count != 0))
            throw new ArgumentException("only public parameterless String-returning ToString overrides are supported");
        if (contract && ((attributes & 0x540) != 0x540 || (attributes & 0x17) != 6 || signature.GenericParameterNames.Count != 0 || name == ".ctor"))
            throw new ArgumentException("invalid interface method flags or signature");
        bool constructor = name == ".ctor";
        if (constructor && ((attributes & 0x10) != 0 || signature.ReturnType != PrimitiveType.Void || signature.GenericParameterNames.Count != 0) ||
            (attributes & 0x1800) != 0 && (!constructor || (attributes & 0x1800) != 0x1800))
            throw new ArgumentException("invalid constructor signature or special-name flags");
        Name = name; authoredNamespace = ""; AuthoredSignature = signature;
        GenericArity = signature.GenericParameterNames.Count;
        declarationAttributes = attributes; this.signature = []; IsTypeDeclaration = true;
    }
    /// <summary>Gets the authored or native source namespace; null for a loaded physical CLI declaration.</summary>
    public string? Namespace => nativeNamespace ?? Producer?.DeclaringType?.Namespace ?? authoredNamespace;
    /// <summary>Gets the authored signature shared by the builder; null for a loaded physical signature.</summary>
    /// <remarks>Raw signature recognizers apply to loaded declarations. Encode and reread for physical tokens and signature blobs.</remarks>
    public MethodSignature? AuthoredSignature { get; }
}

public sealed partial class MethodBuilder
{
    /// <summary>Gets the canonical declaration shared with the authored assembly's method views.</summary>
    public MethodDefinition Definition { get; }
    /// <summary>Returns the existing body facade for an attached authored declaration.</summary>
    /// <exception cref="ArgumentNullException">Definition is null.</exception>
    /// <exception cref="InvalidOperationException">Definition is detached or loaded.</exception>
    public static MethodBuilder ForDefinition(MethodDefinition definition)
    {
        ArgumentNullException.ThrowIfNull(definition);
        return definition.Producer ?? throw new InvalidOperationException("method must be attached to an authored module");
    }
    internal MethodBuilder(AssemblyBuilder assembly, MethodDefinition definition, TypeBuilder? owner = null)
    { Assembly = assembly; DeclaringType = owner; Definition = definition; definition.Producer = this; definition.Module = assembly.Definition.MainModule; }
    // Keep access flags and context-derived CLI flags in one place for inspection and writing.
    internal bool IsOverride => (Definition.DeclarationAttributes & 0x140) == 0x40;
    internal ushort GetAttributes(bool? accessor = null)
    {
        bool special = accessor ?? DeclaringType?.Properties.Any(p => ReferenceEquals(p.GetMethod, this) || ReferenceEquals(p.SetMethod, this)) == true;
        var flags = (MethodAttributes)Definition.DeclarationAttributes | MethodAttributes.HideBySig;
        if (IsAbstract) flags |= MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot;
        else if (!IsOverride && DeclaringType?.Implements(this) == true) flags |= MethodAttributes.Virtual | MethodAttributes.Final | MethodAttributes.NewSlot;
        if (IsConstructor) flags |= MethodAttributes.SpecialName | MethodAttributes.RTSpecialName;
        else if (special) flags |= MethodAttributes.SpecialName;
        return (ushort)flags;
    }
}

public sealed partial class AssemblyBuilder
{
    internal void AttachFunction(MethodDefinition definition)
    {
        if (definition.AuthoredSignature is not { } signature || definition.Producer is not null || definition.IsTypeDeclaration)
            throw new ArgumentException("function must be an unattached authored declaration");
        signature.ValidateOwner(this);
        if (functions.Count >= 256 || functions.Any(m => m.Namespace == definition.Namespace && m.Name == definition.Name &&
            m.Signature.GenericParameterNames.Count == signature.GenericParameterNames.Count && m.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)))
            throw new ArgumentException("duplicate function or exceeded limit");
        functions.Add(new MethodBuilder(this, definition));
    }
}

public sealed partial class TypeBuilder
{
    /// <summary>Adds a public instance override using the same validation as a detached method definition.</summary>
    /// <param name="name">Currently ToString only.</param>
    /// <param name="signature">Nongeneric parameterless signature returning String.</param>
    /// <returns>An owned method with a managed value receiver and a body generator.</returns>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Unsupported override contract, duplicate signature or method limit.</exception>
    /// <exception cref="InvalidOperationException">Owner is not a value type.</exception>
    /// <remarks>CLI emission reuses the inherited Object.ToString slot. Native encoding requires one explicit BindNativeLibrary System binding with a matching Object.ToString slot. Generic value owners are supported; generic override methods are not.</remarks>
    public MethodBuilder AddOverride(string name, MethodSignature signature)
    {
        var definition = new MethodDefinition(name, (ushort)(MethodAttributes.Public | MethodAttributes.Virtual), signature);
        Definition.Methods.Add(definition);
        return MethodBuilder.ForDefinition(definition);
    }

    internal void AttachMethod(MethodDefinition definition)
    {
        if (!definition.IsTypeDeclaration || definition.AuthoredSignature is not { } signature ||
            definition.Producer is { } producer && !ReferenceEquals(producer.DeclaringType, this))
            throw new ArgumentException("method must belong to this type or be a detached authored type method");
        if ((definition.DeclarationAttributes & 0x540) == 0x40 && !IsValueType)
            throw new InvalidOperationException("Object overrides currently require a value owner");
        if (definition.Producer is null && IsInterface != ((definition.DeclarationAttributes & 0x400) != 0))
            throw new InvalidOperationException("abstract contracts require interface owners; concrete methods require class owners");
        if (definition.Name == ".ctor" && definition.AuthoredSignature!.ParameterTypes.Any(p => p.ByReferenceElement is not null)) throw new InvalidOperationException("byref constructor parameters unsupported");
        if (!definition.IsStatic && IsStatic) throw new InvalidOperationException("instance methods require a nonstatic owner");
        signature.ValidateOwner(Assembly, GenericParameterNames.Count, allowSelf: IsInterface);
        if (methods.Count >= 256 || methods.Any(m => m.Name == definition.Name &&
            m.Signature.GenericParameterNames.Count == signature.GenericParameterNames.Count && m.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)))
            throw new ArgumentException("duplicate method or exceeded limit");
        methods.Add(definition.Producer ?? new MethodBuilder(Assembly, definition, this));
    }
}
