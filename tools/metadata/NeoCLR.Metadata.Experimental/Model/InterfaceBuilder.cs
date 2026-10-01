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
        var seen = new HashSet<TypeBuilder>();
        bool ReachesOwner(TypeBuilder current) => ReferenceEquals(current, this) || seen.Add(current) && current.BaseInterfaces.Any(ReachesOwner);
        if (!baseInterface.IsInterface || baseInterface.GenericParameterNames.Count != 0 || !ReferenceEquals(baseInterface.Assembly, Assembly) ||
            baseInterfaces.Count >= 256 || baseInterfaces.Contains(baseInterface) || ReachesOwner(baseInterface))
            throw new ArgumentException("invalid, duplicate or cyclic base interface", nameof(baseInterface));
        baseInterfaces.Add(baseInterface);
    }

    /// <summary>Gets whether this is an interface declaration rather than a class.</summary>
    public bool IsInterface { get; }

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
    /// <summary>Gets whether this is a bodyless abstract interface method.</summary>
    public bool IsAbstract => DeclaringType?.IsInterface == true;
}
