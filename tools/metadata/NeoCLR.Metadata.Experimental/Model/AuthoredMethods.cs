using System.Reflection;

namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class MethodDefinition
{
    /// <summary>Marks an authored nongeneric assembly function as a bodyless runtime internal call.</summary>
    /// <remarks>The runtime must bind the exact namespace, name and signature. This does not select P/Invoke or load a native library.</remarks>
    /// <exception cref="InvalidOperationException">Loaded, type-owned, generic or nonempty declaration.</exception>
    public void SetInternalCall()
    {
        if (AuthoredSignature is null || IsTypeDeclaration || GenericArity != 0 ||
            Body.Instructions.Count != 0 || Body.LocalStorage.Count != 0 || Body.LabelStorage.Count != 0)
            throw new InvalidOperationException("internal calls require an empty authored nongeneric assembly function");
        ImplementationAttributes = 0x1000;
    }

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
        declarationAttributes = (ushort)((visibility switch { MethodVisibility.Public => 6, MethodVisibility.Internal => 3, MethodVisibility.Protected => 4, _ => 1 }) | (isStatic ? 0x10 : 0));
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
    internal static bool IsObjectOverride(string name, MethodSignature signature, AssemblyIdentity? core = null, TypeBuilder? root = null, ImportedTypeReference? externalRoot = null)
    {
        if (signature.GenericParameterNames.Count != 0) return false;
        return name switch
        {
            "ToString" => signature.ReturnType == PrimitiveType.String && signature.ParameterTypes.Count == 0,
            "GetHashCode" => signature.ReturnType == PrimitiveType.Int32 && signature.ParameterTypes.Count == 0,
            "Equals" => signature.ReturnType == PrimitiveType.Boolean && signature.ParameterTypes.Count == 1 &&
                (signature.ParameterTypes[0].ClassType is { IsNativeObjectRoot: true } localRoot &&
                    (core is null || ReferenceEquals(localRoot, root)) ||
                root is null && signature.ParameterTypes[0].ImportedType is { Namespace: "System", Name: "Object", GenericArity: 0, DeclaringType: null, IsValueType: false } owner &&
                (core is null || (externalRoot is not null ? Equals(owner, externalRoot) : owner.AssemblyIdentity.Equals(core)))),
            _ => false
        };
    }
    /// <summary>Creates a detached type method, constructor, virtual slot, override or abstract contract with CLI attributes.</summary>
    /// <param name="name">Nonempty name or .ctor; .cctor is unsupported. Unique by signature on attachment.</param>
    /// <param name="attributes">Public, Assembly, Private or constructor-only Family; optional Static and HideBySig. Constructors may use SpecialName and RTSpecialName together; Public instance or static interface contracts require Abstract, Virtual and NewSlot together. Public Object overrides use Virtual without Abstract or NewSlot. Reference-class slots use Virtual and NewSlot; abstract slots also require Abstract.</param>
    /// <param name="signature">Supported signature validated against the destination type on attachment.</param>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Unsupported attributes or invalid name.</exception>
    /// <remarks>Append to an attached authored TypeDefinition.Methods collection. Instance methods require a class or value owner. Class overrides require an exact inherited virtual contract; value overrides retain the bounded Object contracts. Constructors require a nongeneric Void signature.</remarks>
    public MethodDefinition(string name, ushort attributes, MethodSignature signature)
    {
        ArgumentNullException.ThrowIfNull(signature);
        if (string.IsNullOrEmpty(name) || name.Length > 1024 || name == ".cctor" ||
            (attributes & ~0x1dd7) != 0 || (attributes & 7) is not (1 or 3 or 4 or 6))
            throw new ArgumentException("invalid type-method declaration");
        var slotFlags = attributes & 0x540;
        if (slotFlags is not (0 or 0x40 or 0x140 or 0x540) ||
            slotFlags != 0 && ((attributes & 7) != 6 || name == ".ctor" || signature.GenericParameterNames.Count != 0 ||
                (attributes & 0x10) != 0 && slotFlags != 0x540))
            throw new ArgumentException("invalid virtual method flags or signature");
        bool constructor = name == ".ctor";
        if ((attributes & 7) == 4 && !constructor) throw new ArgumentException("Family visibility currently requires a constructor");
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
    /// <summary>Marks this nongeneric assembly function as a bodyless runtime internal call.</summary>
    /// <exception cref="InvalidOperationException">Type-owned, generic or nonempty declaration.</exception>
    public void SetInternalCall() => Definition.SetInternalCall();

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
    internal bool IsVirtual => (Definition.DeclarationAttributes & 0x40) != 0 || IsAbstract;
    internal bool IsNativeObjectSlot => DeclaringType?.IsNativeObjectRoot == true && (Definition.DeclarationAttributes & 0x540) == 0x140;
    internal bool IsImplicitObjectOverride => IsOverride && DeclaringType?.FindInheritedMethod(Name, Signature) is null;
    internal bool IsOverride => (Definition.DeclarationAttributes & 0x140) == 0x40;
    internal ushort GetAttributes(bool? accessor = null)
    {
        bool special = accessor ?? DeclaringType?.Properties.Any(p => ReferenceEquals(p.GetMethod, this) || ReferenceEquals(p.SetMethod, this)) == true;
        var flags = (MethodAttributes)Definition.DeclarationAttributes | MethodAttributes.HideBySig;
        if (IsAbstract) flags |= MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot;
        else if (!IsVirtual && (DeclaringType?.Implements(this) == true || Definition.ExplicitInterfaceImplementations.Count != 0)) flags |= MethodAttributes.Virtual | MethodAttributes.Final | MethodAttributes.NewSlot;
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
    /// <param name="name">Inherited slot name; rootless owners support ToString, GetHashCode or Equals.</param>
    /// <param name="signature">Exact nongeneric inherited contract, including return and parameter passing modes.</param>
    /// <returns>An owned method with a class receiver or managed value receiver and a body generator.</returns>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Unsupported override contract, duplicate signature or method limit.</exception>
    /// <exception cref="InvalidOperationException">Owner is static, an interface, or the Equals argument has the wrong core identity.</exception>
    /// <remarks>CLI emission reuses the corresponding inherited virtual slot. Ordinary class overrides require a local nongeneric base and are validated on write. For implicit Object overrides, native encoding uses a complete explicitly authored Object root, or requires one explicit BindNativeLibrary System binding with a matching Object slot. Equals must use the selected ObjectType identity. Generic value owners are supported; generic override methods are not.</remarks>
    public MethodBuilder AddOverride(string name, MethodSignature signature)
    {
        ArgumentNullException.ThrowIfNull(signature);
        if (LocalBase is null && !MethodDefinition.IsObjectOverride(name, signature))
            throw new ArgumentException("unsupported Object override signature");
        var definition = new MethodDefinition(name, (ushort)(MethodAttributes.Public | MethodAttributes.Virtual), signature);
        Definition.Methods.Add(definition);
        return MethodBuilder.ForDefinition(definition);
    }

    /// <summary>Adds a public nongeneric instance virtual slot on a reference class.</summary>
    /// <exception cref="ArgumentException">Invalid signature, duplicate method or unsupported owner.</exception>
    public MethodBuilder AddVirtualMethod(string name, MethodSignature signature)
        => AddClassSlot(name, signature, 0x146);

    /// <summary>Adds a public bodyless instance virtual slot on an abstract reference class.</summary>
    /// <exception cref="InvalidOperationException">Owner is not an abstract reference class.</exception>
    public MethodBuilder AddAbstractMethod(string name, MethodSignature signature)
        => AddClassSlot(name, signature, 0x546);

    private MethodBuilder AddClassSlot(string name, MethodSignature signature, ushort attributes)
    {
        if (IsInterface || IsValueType || IsStatic) throw new InvalidOperationException("class slot requires a reference class");
        var definition = new MethodDefinition(name, attributes, signature);
        Definition.Methods.Add(definition);
        return MethodBuilder.ForDefinition(definition);
    }

    internal MethodBuilder? FindInheritedMethod(string name, MethodSignature signature)
    {
        for (var parent = LocalBase; parent is not null; parent = parent.LocalBase)
        {
            var method = parent.Methods.SingleOrDefault(m => !m.IsStatic && m.Name == name &&
                m.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes));
            if (method is not null) return method;
        }
        return null;
    }

    internal void ValidateClassSlots()
    {
        if (IsInterface || IsStatic) return;
        foreach (var method in Methods.Where(m => !m.IsStatic && !m.IsConstructor))
        {
            var inherited = FindInheritedMethod(method.Name, method.Signature);
            if (method.IsOverride)
            {
                if (inherited is not null ? !inherited.IsVirtual || !inherited.Signature.Matches(method.Signature) :
                    !MethodDefinition.IsObjectOverride(method.Name, method.Signature, Assembly.CoreLibrary, Assembly.NativeObjectRoot, Assembly.ExternalObjectRoot))
                    throw new InvalidDataException("override requires an exact inherited virtual contract");
            }
            else if (inherited is not null) throw new InvalidDataException("inherited method hiding is unsupported");
        }
        if (IsAbstract) return;
        for (var owner = LocalBase; owner is not null; owner = owner.LocalBase)
            foreach (var contract in owner.Methods.Where(m => m.IsAbstract))
            {
                var implementation = Methods.SingleOrDefault(m => m.Name == contract.Name && m.Signature.Matches(contract.Signature))
                    ?? FindInheritedMethod(contract.Name, contract.Signature);
                if (implementation is null || implementation.IsAbstract)
                    throw new InvalidDataException("concrete class must implement inherited abstract methods");
            }
    }

    internal void AttachMethod(MethodDefinition definition)
    {
        if (!definition.IsTypeDeclaration || definition.AuthoredSignature is not { } signature ||
            definition.Producer is { } producer && !ReferenceEquals(producer.DeclaringType, this))
            throw new ArgumentException("method must belong to this type or be a detached authored type method");
        var slotFlags = definition.DeclarationAttributes & 0x540;
        if (slotFlags == 0x40 && (IsInterface || IsStatic || IsNativeObjectRoot ||
            (LocalBase is null || MethodDefinition.IsObjectOverride(definition.Name, signature)) &&
            !MethodDefinition.IsObjectOverride(definition.Name, signature, Assembly.CoreLibrary, Assembly.NativeObjectRoot, Assembly.ExternalObjectRoot)))
            throw new InvalidOperationException("override requires a class owner and the exact inherited contract");
        if (IsNativeObjectRoot && slotFlags != 0 && (slotFlags != 0x140 || !MethodDefinition.IsNativeObjectSlot(definition.Name, signature, this)))
            throw new InvalidOperationException("native Object slots require their exact root signature");
        if (slotFlags != 0 && !IsInterface && !IsNativeObjectRoot &&
            (IsStatic || !IsValueType && GenericParameterNames.Count != 0 || IsValueType && (slotFlags != 0x40 || !MethodDefinition.IsObjectOverride(definition.Name, signature, Assembly.CoreLibrary, Assembly.NativeObjectRoot, Assembly.ExternalObjectRoot))))
            throw new InvalidOperationException("virtual slots require a nongeneric reference class or supported Object value override");
        if (definition.Producer is null && (IsInterface && slotFlags != 0x540 || !IsInterface && slotFlags == 0x540 && (!IsAbstract || definition.IsStatic)))
            throw new InvalidOperationException("abstract methods require an abstract class or interface");
        if (definition.Name == ".ctor" && definition.AuthoredSignature!.ParameterTypes.Any(p => p.ByReferenceElement is not null)) throw new InvalidOperationException("byref constructor parameters unsupported");
        if (!definition.IsStatic && IsStatic) throw new InvalidOperationException("instance methods require a nonstatic owner");
        signature.ValidateOwner(Assembly, GenericParameterNames.Count, allowSelf: IsInterface);
        if (methods.Count >= 256 || methods.Any(m => m.Name == definition.Name &&
            m.Signature.GenericParameterNames.Count == signature.GenericParameterNames.Count && m.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)))
            throw new ArgumentException("duplicate method or exceeded limit");
        methods.Add(definition.Producer ?? new MethodBuilder(Assembly, definition, this));
    }
}
