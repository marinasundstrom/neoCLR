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
    /// <summary>Creates a detached static type-method declaration with CLI attributes.</summary>
    /// <param name="name">Nonempty name, excluding constructor names; unique by signature on attachment.</param>
    /// <param name="attributes">Static plus Public, Assembly or Private; optional HideBySig. Other flags are unsupported.</param>
    /// <param name="signature">Supported signature validated against the destination type on attachment.</param>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Unsupported attributes or invalid name.</exception>
    /// <remarks>Append to an attached authored TypeDefinition.Methods collection. Instance methods and constructors still use builders.</remarks>
    public MethodDefinition(string name, ushort attributes, MethodSignature signature)
    {
        ArgumentNullException.ThrowIfNull(signature);
        if (string.IsNullOrEmpty(name) || name.Length > 1024 || name is ".ctor" or ".cctor" ||
            (attributes & ~0x97) != 0 || (attributes & 0x10) == 0 || (attributes & 7) is not (1 or 3 or 6))
            throw new ArgumentException("invalid static type-method declaration");
        Name = name; authoredNamespace = ""; AuthoredSignature = signature;
        GenericArity = signature.GenericParameterNames.Count;
        declarationAttributes = attributes; this.signature = []; IsTypeDeclaration = true;
    }
    /// <summary>Gets the authored source namespace; null for a loaded physical declaration.</summary>
    public string? Namespace => Producer?.DeclaringType?.Namespace ?? authoredNamespace;
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
    internal ushort GetAttributes(bool? accessor = null)
    {
        bool special = accessor ?? DeclaringType?.Properties.Any(p => ReferenceEquals(p.GetMethod, this) || ReferenceEquals(p.SetMethod, this)) == true;
        var flags = (MethodAttributes)Definition.DeclarationAttributes | MethodAttributes.HideBySig;
        if (IsAbstract) flags |= MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot;
        else if (DeclaringType?.Implements(this) == true) flags |= MethodAttributes.Virtual | MethodAttributes.Final | MethodAttributes.NewSlot;
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
    internal void AttachMethod(MethodDefinition definition)
    {
        if (!definition.IsTypeDeclaration || definition.AuthoredSignature is not { } signature ||
            definition.Producer is { } producer && !ReferenceEquals(producer.DeclaringType, this))
            throw new ArgumentException("method must belong to this type or be a detached authored type method");
        if (definition.Producer is null && IsInterface) throw new InvalidOperationException("interface methods require abstract contract builders");
        signature.ValidateOwner(Assembly, GenericParameterNames.Count);
        if (methods.Count >= 256 || methods.Any(m => m.Name == definition.Name &&
            m.Signature.GenericParameterNames.Count == signature.GenericParameterNames.Count && m.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)))
            throw new ArgumentException("duplicate method or exceeded limit");
        methods.Add(definition.Producer ?? new MethodBuilder(Assembly, definition, this));
    }
}
