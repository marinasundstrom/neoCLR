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
    /// <summary>Gets the directly inherited interface definitions in declaration order.</summary>
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
    internal void AttachBaseInterface(TypeBuilder baseInterface)
    {
        var seen = new HashSet<TypeBuilder>();
        bool ReachesOwner(TypeBuilder current) => ReferenceEquals(current, this) || seen.Add(current) && current.BaseInterfaces.Any(ReachesOwner);
        if (!baseInterface.IsInterface || baseInterface.GenericParameterNames.Count != 0 || !ReferenceEquals(baseInterface.Assembly, Assembly) ||
            baseInterfaces.Count >= 256 || baseInterfaces.Contains(baseInterface) || ReachesOwner(baseInterface))
            throw new ArgumentException("invalid, duplicate or cyclic base interface", nameof(baseInterface));
        baseInterfaces.Add(baseInterface);
    }

    private readonly List<TypeBuilder> implementedInterfaces = [];
    /// <summary>Gets the directly implemented nongeneric interfaces of a root class.</summary>
    public IReadOnlyList<TypeBuilder> ImplementedInterfaces => implementedInterfaces.AsReadOnly();
    private readonly List<GenericTypeInstance> constructedInterfaces = [];
    internal IEnumerable<SignatureType> InterfaceSignatures => InterfaceContracts.Select(t => (SignatureType)t).Concat(constructedInterfaces.Select(t => (SignatureType)t));
    internal IEnumerable<(string Name, MethodSignature Signature)> RequiredInterfaceMethods => InterfaceMethods.Select(m => (m.Name, m.Signature)).Concat(
        constructedInterfaces.SelectMany(i => i.Definition.Methods.Select(m => (m.Name, new ConstructedMethodReference(m, i.TypeArguments.ToArray(), []).Signature))));
    /// <summary>Declares a closed owned generic interface implementation on a nongeneric root class.</summary>
    /// <exception cref="ArgumentNullException">Contract is null.</exception>
    /// <exception cref="ArgumentException">Foreign, open, duplicate, inherited or noninterface contract.</exception>
    /// <exception cref="InvalidOperationException">Owner is not a nongeneric root class.</exception>
    public void AddInterfaceImplementation(GenericTypeInstance contract)
    {
        ArgumentNullException.ThrowIfNull(contract);
        Definition.Interfaces.Add(new InterfaceImplementation(contract.Definition.Definition.ToReference(), contract.TypeArguments));
    }
    internal void AttachConstructedInterface(GenericTypeInstance contract)
    {
        if (IsInterface || IsStatic || IsValueType || GenericParameterNames.Count > 0) throw new InvalidOperationException("constructed interfaces require a nongeneric root class");
        if (!contract.Definition.IsInterface || !ReferenceEquals(contract.Definition.Assembly, Assembly) ||
            contract.Definition.BaseInterfaces.Count > 0 || constructedInterfaces.Count >= 256 || constructedInterfaces.Contains(contract))
            throw new ArgumentException("unsupported or duplicate constructed interface");
        foreach (var argument in contract.TypeArguments) argument.ValidateOwner(Assembly);
        constructedInterfaces.Add(contract);
    }
    internal bool ConformsTo(GenericTypeInstance contract) => constructedInterfaces.Contains(contract);
    internal IEnumerable<TypeBuilder> InterfaceContracts => baseInterfaces.Concat(implementedInterfaces);
    internal bool ConformsTo(TypeBuilder contract) => ReferenceEquals(this, contract) || InterfaceContracts.Any(i => i.ConformsTo(contract));
    internal IEnumerable<MethodBuilder> InterfaceMethods => InterfaceContracts.SelectMany(i => i.Methods.Concat(i.InterfaceMethods)).Distinct();
    internal bool Implements(MethodBuilder method) => !method.IsStatic && method.Visibility == MethodVisibility.Public &&
        RequiredInterfaceMethods.Any(c => c.Name == method.Name && c.Signature.ReturnType == method.Signature.ReturnType &&
            c.Signature.ParameterTypes.SequenceEqual(method.Signature.ParameterTypes));

    /// <summary>Declares implicit public implementation of an owned nongeneric interface.</summary>
    /// <param name="contract">An interface from this assembly, including its inherited contracts.</param>
    /// <exception cref="ArgumentNullException">Contract is null.</exception>
    /// <exception cref="ArgumentException">Foreign, generic, duplicate or noninterface contract, or limit exceeded.</exception>
    /// <exception cref="InvalidOperationException">Owner is not a nongeneric root class.</exception>
    /// <remarks>Writing requires an exact public instance implementation for every inherited method.</remarks>
    public void AddInterfaceImplementation(TypeBuilder contract)
    {
        ArgumentNullException.ThrowIfNull(contract);
        if (IsInterface || IsStatic || IsValueType || GenericParameterNames.Count != 0)
            throw new InvalidOperationException("interface implementations require a nongeneric root class");
        Definition.Interfaces.Add(new InterfaceImplementation(contract.Definition.ToReference()));
    }
    internal void AttachInterfaceImplementation(TypeBuilder contract)
    {
        if (IsInterface || IsStatic || IsValueType || GenericParameterNames.Count != 0)
            throw new InvalidOperationException("interface implementations require a nongeneric root class");
        if (!contract.IsInterface || contract.GenericParameterNames.Count != 0 || !ReferenceEquals(contract.Assembly, Assembly) ||
            implementedInterfaces.Contains(contract) || implementedInterfaces.Count >= 256)
            throw new ArgumentException("invalid or duplicate interface implementation", nameof(contract));
        implementedInterfaces.Add(contract);
    }

    /// <summary>Gets whether this is an interface declaration rather than a class.</summary>
    public bool IsInterface => (Definition.Attributes & 0x20) != 0;

    /// <summary>Adds a public abstract instance method to an interface.</summary>
    /// <param name="name">Nonempty simple method name; constructors are forbidden.</param>
    /// <param name="signature">Supported value parameters/result; declaring-type parameters are allowed, method parameters are not.</param>
    /// <returns>A bodyless declaration; adding instructions or locals makes writing fail.</returns>
    /// <exception cref="ArgumentNullException">Null signature.</exception>
    /// <exception cref="ArgumentException">Invalid/duplicate signature, generic method or exceeded limit.</exception>
    /// <exception cref="InvalidOperationException">Owner is not an interface.</exception>
    public MethodBuilder AddInterfaceMethod(string name, MethodSignature signature)
    {
        ArgumentNullException.ThrowIfNull(signature);
        if (signature.GenericParameterNames.Count != 0) throw new ArgumentException("generic interface methods are not supported yet", nameof(signature));
        return AddMethodCore(name, signature, MethodVisibility.Public, false, false, abstractContract: true);
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Appends virtual dispatch to an owned nongeneric interface method.</summary>
    /// <param name="target">A public abstract interface instance method.</param>
    /// <exception cref="ArgumentNullException">Target is null.</exception>
    /// <exception cref="ArgumentException">Foreign, generic or noninterface target.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void CallVirtual(MethodBuilder target) => Emit(OpCode.Callvirt, target);

    /// <summary>Gets whether this is a bodyless abstract interface method.</summary>
    public bool IsAbstract => DeclaringType?.IsInterface == true;
}
