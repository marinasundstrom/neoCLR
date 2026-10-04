namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A private method body's explicit implementation of an interface member.</summary>
/// <param name="Interface">Exact interface declaration and copied constructed arguments.</param>
/// <param name="MemberName">Declared member name, including get_/set_ for accessors.</param>
/// <remarks>The body signature selects the nongeneric interface overload. The owner must declare
/// the interface, directly or through inheritance. Writing validates the complete contract.</remarks>
public sealed record ExplicitInterfaceImplementation(InterfaceImplementation Interface, string MemberName);

public sealed partial class MethodDefinition
{
    private readonly List<ExplicitInterfaceImplementation> explicitInterfaces = [];
    /// <summary>Gets explicit interface mappings for authored or native-loaded methods.</summary>
    /// <exception cref="NotSupportedException">CLI MethodImpl materialization is not yet supported.</exception>
    public IReadOnlyList<ExplicitInterfaceImplementation> ExplicitInterfaceImplementations =>
        AuthoredSignature is null && nativeSignature is null
            ? throw new NotSupportedException("CLI MethodImpl materialization is not supported")
            : explicitInterfaces.AsReadOnly();

    /// <summary>Adds an explicit mapping to an attached private instance method.</summary>
    /// <exception cref="InvalidOperationException">The method is detached or loaded.</exception>
    /// <exception cref="ArgumentException">Mapping, owner, visibility or signature is incompatible.</exception>
    public void AddExplicitInterfaceImplementation(InterfaceImplementation contract, string memberName)
    {
        ArgumentNullException.ThrowIfNull(contract);
        if (Producer is not { } method) throw new InvalidOperationException("attach an authored method before mapping");
        if (string.IsNullOrEmpty(memberName) || memberName.Length > 1024 || memberName.Any(char.IsControl) ||
            method.IsStatic || method.IsAbstract || method.IsOverride || method.IsConstructor ||
            method.Visibility != MethodVisibility.Private || GenericArity != 0 || explicitInterfaces.Count >= 128)
            throw new ArgumentException("explicit implementation requires a private concrete nongeneric instance body");
        var mapping = new ExplicitInterfaceImplementation(contract, memberName);
        var signature = method.ExplicitOwner(mapping);
        signature.ValidateOwner(method.Assembly, typeArity: method.DeclaringType!.GenericParameterNames.Count);
        if (explicitInterfaces.Any(m => m.MemberName == memberName && Equals(method.ExplicitOwner(m), signature)))
            throw new ArgumentException("duplicate explicit interface mapping");
        explicitInterfaces.Add(mapping);
    }
    internal void SetLoadedExplicitInterfaces(IEnumerable<ExplicitInterfaceImplementation> mappings)
        => explicitInterfaces.AddRange(mappings);
}

public sealed partial class MethodBuilder
{
    /// <summary>Maps this private instance body to an output-owned local or constructed interface member.</summary>
    public void AddExplicitInterfaceImplementation(TypeBuilder contract, string memberName, params SignatureType[] arguments)
    {
        ArgumentNullException.ThrowIfNull(contract);
        Definition.AddExplicitInterfaceImplementation(new InterfaceImplementation(contract.Definition.ToReference(), arguments), memberName);
    }
    /// <summary>Maps this private instance body to an output-owned external interface member.</summary>
    public void AddExplicitInterfaceImplementation(ImportedTypeReference contract, string memberName)
    {
        ArgumentNullException.ThrowIfNull(contract);
        if (!ReferenceEquals(contract.Owner, Assembly)) throw new ArgumentException("foreign interface");
        Definition.AddExplicitInterfaceImplementation(new InterfaceImplementation(
            Assembly.Definition.MainModule.ImportReference(contract.AssemblyIdentity, contract.Namespace, contract.Name), contract.TypeArguments), memberName);
    }
    internal SignatureType ExplicitOwner(ExplicitInterfaceImplementation mapping)
    {
        var reference = mapping.Interface.InterfaceType;
        if (!ReferenceEquals(reference.Module, Definition.Module)) throw new ArgumentException("foreign interface reference");
        var args = mapping.Interface.TypeArguments.ToArray();
        if (reference.ExplicitScope is not null)
        {
            var imported = Assembly.FindAuthoredInterface(reference);
            return args.Length == 0 ? imported : imported.MakeGenericInstance(args);
        }
        var type = reference.Resolve().Producer ?? throw new ArgumentException("interface must be authored");
        if (!type.IsInterface) throw new ArgumentException("mapping target is not an interface");
        return args.Length == 0 ? type : type.MakeGenericInstance(args);
    }
}

public sealed partial class TypeBuilder
{
    internal MethodBuilder? FindInterfaceImplementation(string name, MethodSignature signature, bool isStatic, SignatureType owner)
    {
        var explicitBodies = Methods.Where(m => m.Definition.ExplicitInterfaceImplementations.Any(e =>
            e.MemberName == name && Equals(m.ExplicitOwner(e), owner)) && m.Signature.Matches(signature)).ToArray();
        if (explicitBodies.Length > 1) throw new InvalidDataException("ambiguous explicit interface implementation");
        if (explicitBodies.Length == 1)
        {
            var body = explicitBodies[0];
            if (isStatic || !body.Signature.Matches(signature)) throw new InvalidDataException("incompatible explicit interface implementation");
            return body;
        }
        return Methods.SingleOrDefault(m => m.IsStatic == isStatic && m.Visibility == MethodVisibility.Public &&
            m.Name == name && m.Signature.GenericParameterNames.Count == 0 && m.Signature.Matches(signature));
    }
    internal void ValidateExplicitImplementations()
    {
        foreach (var body in Methods)
            foreach (var mapping in body.Definition.ExplicitInterfaceImplementations)
                if (!RequiredInterfaceMethods.Any(c => !c.IsStatic && c.Name == mapping.MemberName &&
                    Equals(c.Owner, body.ExplicitOwner(mapping)) && c.Signature.Matches(body.Signature)))
                    throw new InvalidDataException("explicit mapping does not match a declared interface member");
    }
}
