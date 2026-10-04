namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    /// <summary>Adds an owned interface declaration with no initial base interfaces or implementations.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Nonempty metadata name.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>An interface accepting abstract instance method contracts.</returns>
    /// <exception cref="ArgumentException">Invalid identity, duplicate type or exceeded limit.</exception>
    public TypeBuilder AddInterface(string @namespace, string name, TypeVisibility visibility = TypeVisibility.Public)
        => AddTypeCore(@namespace, name, visibility, false, isInterface: true);

    /// <summary>Adds an invariant generic interface declaration.</summary>
    /// <param name="namespace">Namespace, possibly empty.</param>
    /// <param name="name">Simple name without an arity suffix.</param>
    /// <param name="genericParameterNames">One through 32 distinct names, copied.</param>
    /// <param name="visibility">Public or Internal.</param>
    /// <returns>An interface whose method signatures can use declaring-type parameters.</returns>
    /// <exception cref="ArgumentNullException">Null parameter names.</exception>
    /// <exception cref="ArgumentException">Invalid identity/parameters, duplicate type or exceeded limit.</exception>
    public TypeBuilder AddGenericInterface(string @namespace, string name, IEnumerable<string> genericParameterNames, TypeVisibility visibility = TypeVisibility.Public)
        => AddGenericTypeCore(@namespace, name, genericParameterNames, visibility, false, isInterface: true);
}

public sealed partial class TypeBuilder
{
    private readonly List<TypeBuilder> baseInterfaces = [];
    /// <summary>Gets directly inherited nongeneric interface definitions in declaration order.</summary>
    /// <remarks>Definition.Interfaces retains all declared edges, including constructed generic bases.</remarks>
    public IReadOnlyList<TypeBuilder> BaseInterfaces => baseInterfaces.AsReadOnly();
    /// <summary>Adds an owned nongeneric base interface without introducing class implementation semantics.</summary>
    /// <param name="baseInterface">A nongeneric interface from the same assembly.</param>
    /// <exception cref="ArgumentNullException">The base is null.</exception>
    /// <exception cref="ArgumentException">Foreign/noninterface/generic base, duplicate edge, cycle or limit exceeded.</exception>
    /// <exception cref="InvalidOperationException">The owner is not an interface.</exception>
    public void AddBaseInterface(TypeBuilder baseInterface)
    {
        ArgumentNullException.ThrowIfNull(baseInterface);
        if (!IsInterface) throw new InvalidOperationException("base-interface declarations require an interface owner");
        Definition.Interfaces.Add(new InterfaceImplementation(baseInterface.Definition.ToReference()));
    }
    /// <summary>Adds an owned constructed generic base interface, including owner-parameter arguments.</summary>
    /// <param name="baseInterface">An owned interface construction scoped to this declaring interface.</param>
    /// <exception cref="ArgumentNullException">The base is null.</exception>
    /// <exception cref="ArgumentException">Foreign/noninterface base, invalid arguments, duplicate edge, cycle or limit.</exception>
    /// <exception cref="InvalidOperationException">The owner is not an interface.</exception>
    public void AddBaseInterface(GenericTypeInstance baseInterface)
    {
        ArgumentNullException.ThrowIfNull(baseInterface);
        if (!IsInterface) throw new InvalidOperationException("base-interface declarations require an interface owner");
        Definition.Interfaces.Add(new InterfaceImplementation(baseInterface.Definition.Definition.ToReference(), baseInterface.TypeArguments));
    }
    private bool Reaches(TypeBuilder target)
    {
        var seen = new HashSet<TypeBuilder>();
        bool Visit(TypeBuilder current) => ReferenceEquals(current, target) || seen.Add(current) && current.InterfaceSignatures.Any(t => t.ImportedType is null && Visit(t.GenericInstance?.Definition ?? t.ClassType!));
        return Visit(this);
    }
    internal void AttachBaseInterface(TypeBuilder baseInterface)
    {
        if (!baseInterface.IsInterface || baseInterface.GenericParameterNames.Count != 0 || !ReferenceEquals(baseInterface.Assembly, Assembly) ||
            baseInterfaces.Count >= 256 || baseInterfaces.Contains(baseInterface) || baseInterface.Reaches(this))
            throw new ArgumentException("invalid, duplicate or cyclic base interface", nameof(baseInterface));
        baseInterfaces.Add(baseInterface);
    }

    private readonly List<TypeBuilder> implementedInterfaces = [];
    /// <summary>Gets the directly implemented nongeneric interfaces of a root class or value type.</summary>
    public IReadOnlyList<TypeBuilder> ImplementedInterfaces => implementedInterfaces.AsReadOnly();
    private readonly List<GenericTypeInstance> constructedInterfaces = [];
    private readonly List<ImportedTypeReference> externalInterfaces = [];

    /// <summary>Adds a base-interface relationship to an output-owned external interface contract.</summary>
    /// <remarks>Call AssemblyBuilder.CompleteInterfaceReference for the interface and its bases before writing.</remarks>
    /// <exception cref="ArgumentException">The reference is foreign, unregistered, duplicate or has invalid generic scope.</exception>
    /// <exception cref="InvalidOperationException">The owner is not an interface.</exception>
    public void AddBaseInterface(ImportedTypeReference contract)
    {
        if (!IsInterface) throw new InvalidOperationException("base-interface declarations require an interface owner");
        AddExternalInterface(contract);
    }

    /// <summary>Adds an external interface implementation to a root class or value type.</summary>
    /// <remarks>All required public methods, including static/instance classification, are checked against the completed external contracts when writing.</remarks>
    /// <exception cref="ArgumentException">The reference is foreign, unregistered, duplicate or has invalid generic scope.</exception>
    /// <exception cref="InvalidOperationException">The owner is not a root class or value type.</exception>
    public void AddInterfaceImplementation(ImportedTypeReference contract)
    {
        if (IsInterface || IsStatic) throw new InvalidOperationException("interface implementations require a root class or value type");
        AddExternalInterface(contract);
    }

    private void AddExternalInterface(ImportedTypeReference contract)
    {
        ArgumentNullException.ThrowIfNull(contract);
        if (!ReferenceEquals(contract.Owner, Assembly)) throw new ArgumentException("foreign interface reference", nameof(contract));
        var reference = Definition.Module.ImportReference(contract.AssemblyIdentity, contract.Namespace, contract.Name);
        Definition.Interfaces.Add(new InterfaceImplementation(reference, contract.TypeArguments));
    }

    internal void AttachExternalInterface(ImportedTypeReference contract)
    {
        if (!IsInterface && IsStatic) throw new InvalidOperationException("interface relationship requires an interface or root class or value type");
        SignatureType signature = contract;
        signature.ValidateOwner(Assembly, typeArity: GenericParameterNames.Count, allowSelf: IsInterface);
        if (externalInterfaces.Contains(contract) || externalInterfaces.Count >= 256) throw new ArgumentException("duplicate or excessive interface relationship");
        externalInterfaces.Add(contract);
    }
    internal IEnumerable<SignatureType> InterfaceSignatures => InterfaceContracts.Select(t => (SignatureType)t).Concat(constructedInterfaces.Select(t => (SignatureType)t)).Concat(externalInterfaces.Select(t => (SignatureType)t));
    internal IEnumerable<SignatureType> InheritedContracts()
    {
        var seen = new HashSet<SignatureType>();
        IEnumerable<SignatureType> Visit(SignatureType contract)
        {
            if (!seen.Add(contract)) yield break;
            if (seen.Count > 4096) throw new InvalidDataException("interface inheritance expansion limit");
            yield return contract;
            if (contract.ImportedType is { } external)
            {
                foreach (var parent in Assembly.ExternalInterfaceBases(external))
                    foreach (var inherited in Visit(parent)) yield return inherited;
                yield break;
            }
            var owner = contract.GenericInstance?.Definition ?? contract.ClassType!;
            SignatureType Substitute(SignatureType type) => type.FunctionSignature is { } function ? function.Substitute(Substitute) : type.ByReferenceElement is { } byref ? SignatureType.ByReference(Substitute(byref)) : type.ImportedType is { } imported ? imported.Substitute(Substitute) : type.TypeParameterIndex is { } index && contract.GenericInstance is { } instance ? instance.TypeArguments[index]
                : type.GenericInstance is { } nested ? nested.Definition.MakeGenericInstance(nested.TypeArguments.Select(Substitute).ToArray())
                : type.ArrayElement is { } element ? SignatureType.ArrayOf(Substitute(element)) : type;
            foreach (var parent in owner.InterfaceSignatures)
                foreach (var inherited in Visit(Substitute(parent))) yield return inherited;
        }
        return InterfaceSignatures.SelectMany(Visit);
    }
    internal IEnumerable<(string Name, MethodSignature Signature, bool IsStatic, MethodBuilder Declaration, SignatureType Owner)> RequiredInterfaceMethods =>
        InheritedContracts().SelectMany(owner =>
            (owner.ImportedType is { } external ? Assembly.ExternalInterfaceMethods(external) :
                (owner.GenericInstance?.Definition ?? owner.ClassType!).Methods.Select(method => (method.Name, Signature:
                    owner.GenericInstance is { } instance ? new ConstructedMethodReference(method, instance.TypeArguments.ToArray(), []).Signature : method.Signature,
                    method.IsStatic, Declaration: method)))
            .Select(contract => (contract.Name, new MethodSignature(ResolveImplementationSelf(contract.Signature.ReturnType),
                contract.Signature.ParameterTypes.Select(ResolveImplementationSelf), outParameters: contract.Signature.OutParameters),
                contract.IsStatic, contract.Declaration, owner)));

    internal SignatureType ResolveImplementationSelf(SignatureType type) => type.IsSelf && !IsInterface
        ? GenericParameterNames.Count == 0 ? this : MakeGenericInstance(Enumerable.Range(0, GenericParameterNames.Count).Select(SignatureType.TypeParameter).ToArray())
        : type.FunctionSignature is { } function ? function.Substitute(ResolveImplementationSelf)
        : type.ByReferenceElement is { } byref ? SignatureType.ByReference(ResolveImplementationSelf(byref))
        : type.ArrayElement is { } element ? SignatureType.ArrayOf(ResolveImplementationSelf(element))
        : type.ImportedType is { } imported ? imported.Substitute(ResolveImplementationSelf)
        : type.GenericInstance is { } generic ? generic.Definition.MakeGenericInstance(generic.TypeArguments.Select(ResolveImplementationSelf).ToArray()) : type;
    /// <summary>Declares an owned generic interface implementation on a root class or value type, including owner-parameter arguments.</summary>
    /// <exception cref="ArgumentNullException">Contract is null.</exception>
    /// <exception cref="ArgumentException">Foreign, out-of-scope, duplicate, cyclic or noninterface contract.</exception>
    /// <exception cref="InvalidOperationException">Owner is not a root class or value type.</exception>
    public void AddInterfaceImplementation(GenericTypeInstance contract)
    {
        ArgumentNullException.ThrowIfNull(contract);
        if (IsInterface || IsStatic)
            throw new InvalidOperationException("interface implementations require a root class or value type");
        Definition.Interfaces.Add(new InterfaceImplementation(contract.Definition.Definition.ToReference(), contract.TypeArguments));
    }
    internal void AttachConstructedInterface(GenericTypeInstance contract)
    {
        if (!IsInterface && IsStatic) throw new InvalidOperationException("constructed interfaces require an interface or root class or value type");
        if (!contract.Definition.IsInterface || !ReferenceEquals(contract.Definition.Assembly, Assembly) ||
            contract.Definition.Reaches(this) || constructedInterfaces.Count >= 256 || constructedInterfaces.Contains(contract))
            throw new ArgumentException("unsupported or duplicate constructed interface");
        foreach (var argument in contract.TypeArguments) argument.ValidateOwner(Assembly, typeArity: GenericParameterNames.Count, allowSelf: IsInterface);
        constructedInterfaces.Add(contract);
    }
    internal bool ConformsTo(GenericTypeInstance contract) => InheritedContracts().Any(t => Equals(t.GenericInstance, contract));
    internal IEnumerable<TypeBuilder> InterfaceContracts => baseInterfaces.Concat(implementedInterfaces);
    internal bool ConformsTo(TypeBuilder contract) => ReferenceEquals(this, contract) || InheritedContracts().Any(t => ReferenceEquals(t.ClassType, contract));
    internal IEnumerable<MethodBuilder> InterfaceMethods => InterfaceContracts.SelectMany(i => i.Methods.Concat(i.InterfaceMethods)).Distinct();
    internal bool Implements(MethodBuilder method) => !method.IsStatic && method.Visibility == MethodVisibility.Public &&
        RequiredInterfaceMethods.Any(c => !c.IsStatic && c.Name == method.Name && c.Signature.Matches(method.Signature));

    /// <summary>Declares implicit public implementation of an owned nongeneric interface.</summary>
    /// <param name="contract">An interface from this assembly, including its inherited contracts.</param>
    /// <exception cref="ArgumentNullException">Contract is null.</exception>
    /// <exception cref="ArgumentException">Foreign, generic, duplicate or noninterface contract, or limit exceeded.</exception>
    /// <exception cref="InvalidOperationException">Owner is not a root class or value type.</exception>
    /// <remarks>Writing requires an exact public implementation, including static/instance classification, for every inherited method.</remarks>
    public void AddInterfaceImplementation(TypeBuilder contract)
    {
        ArgumentNullException.ThrowIfNull(contract);
        if (IsInterface || IsStatic)
            throw new InvalidOperationException("interface implementations require a root class or value type");
        Definition.Interfaces.Add(new InterfaceImplementation(contract.Definition.ToReference()));
    }
    internal void AttachInterfaceImplementation(TypeBuilder contract)
    {
        if (IsInterface || IsStatic)
            throw new InvalidOperationException("interface implementations require a root class or value type");
        if (!contract.IsInterface || contract.GenericParameterNames.Count != 0 || !ReferenceEquals(contract.Assembly, Assembly) ||
            implementedInterfaces.Contains(contract) || implementedInterfaces.Count >= 256)
            throw new ArgumentException("invalid or duplicate interface implementation", nameof(contract));
        implementedInterfaces.Add(contract);
    }

    /// <summary>Gets whether this is an interface declaration rather than a class.</summary>
    public bool IsInterface => (Definition.Attributes & 0x20) != 0;

    /// <summary>Adds a public abstract instance method to an interface.</summary>
    /// <param name="name">Nonempty simple method name; constructors are forbidden.</param>
    /// <param name="signature">Supported value parameters/result; declaring-type parameters and native Self are allowed, method parameters are not.</param>
    /// <returns>A bodyless declaration; adding instructions or locals makes writing fail.</returns>
    /// <exception cref="ArgumentNullException">Null signature.</exception>
    /// <exception cref="ArgumentException">Invalid/duplicate signature, generic method or exceeded limit.</exception>
    /// <exception cref="InvalidOperationException">Owner is not an interface.</exception>
    public MethodBuilder AddInterfaceMethod(string name, MethodSignature signature)
        => AddInterfaceMethod(name, signature, false);

    /// <summary>Adds a public abstract instance or static interface contract.</summary>
    /// <param name="name">Nonempty simple metadata name.</param>
    /// <param name="signature">Nongeneric signature; native Self is allowed.</param>
    /// <param name="isStatic">True for a contract with no receiver.</param>
    /// <returns>A bodyless declaration.</returns>
    /// <exception cref="ArgumentNullException">Null signature.</exception>
    /// <exception cref="ArgumentException">Invalid name/signature, duplicate method or exceeded limit.</exception>
    /// <exception cref="InvalidOperationException">Owner is not an interface.</exception>
    /// <remarks>Instructions and locals are forbidden. Static implementation matching is exact;
    /// native constrained Self calls are a separate emission category.</remarks>
    public MethodBuilder AddInterfaceMethod(string name, MethodSignature signature, bool isStatic)
    {
        ArgumentNullException.ThrowIfNull(signature);
        if (signature.GenericParameterNames.Count != 0) throw new ArgumentException("generic interface methods are not supported yet", nameof(signature));
        return AddMethodCore(name, signature, MethodVisibility.Public, isStatic, false, abstractContract: true);
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Appends virtual dispatch to an owned nongeneric interface method.</summary>
    /// <param name="target">A public abstract interface instance method.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="ArgumentException">Foreign, generic or noninterface target.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void CallVirtual(MethodBuilder target) => GetILGenerator().CallVirtual(target);

    /// <summary>Gets whether this is a bodyless abstract interface method.</summary>
    public bool IsAbstract => DeclaringType?.IsInterface == true;
}
