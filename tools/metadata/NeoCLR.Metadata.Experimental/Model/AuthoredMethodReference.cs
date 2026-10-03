namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    /// <summary>Authors a public nonvirtual member, bounded value override or abstract interface method reference without a reader definition.</summary>
    /// <param name="declaringType">Output-owned class/value/interface definition, not a construction.</param>
    /// <param name="name">Simple member name, or .ctor for a constructor.</param>
    /// <param name="signature">Primitive, scoped parameter, external nominal construction, vector or bounded function signature.</param>
    /// <param name="isStatic">Whether the member has no receiver. Constructors must be instance members.</param>
    /// <param name="isOverride">Reuse the inherited slot; currently only instance value-type ToString(): String is supported.</param>
    /// <returns>An interned output-owned method contract. Construct generic owners before calling.</returns>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="ArgumentException">Invalid owner, name, constructor or signature scope.</exception>
    /// <exception cref="InvalidDataException">Unsupported signature, conflicting contract or reference limit.</exception>
    /// <exception cref="InvalidOperationException">A new method is added after interface completion.</exception>
    /// <remarks>The caller supplies public nonvirtual class semantics or an abstract interface contract. No dependency is loaded or verified.
    /// Authored interfaces require nongeneric abstract instance contracts and emit virtual dispatch. Writable ref/out parameters are supported; byref constructors, instance generic methods are unsupported. Value/nested owners retain managed receiver and physical scope semantics.
    /// Dependency identity, core and artifact checks are established by the declaring type reference.</remarks>
    public ImportedMethodReference CreateMethodReference(ImportedTypeReference declaringType, string name,
        MethodSignature signature, bool isStatic = false, bool isOverride = false)
    {
        ArgumentNullException.ThrowIfNull(declaringType);
        ArgumentNullException.ThrowIfNull(name);
        ArgumentNullException.ThrowIfNull(signature);
        if (!ReferenceEquals(declaringType.Owner, this) ||
            declaringType.TypeArguments.Count != 0 || !importedGraphs.TryGetValue(declaringType.AssemblyIdentity, out var graph))
            throw new ArgumentException("method requires an owned nominal definition", nameof(declaringType));
        if (isOverride && (!declaringType.IsValueType || isStatic || name != "ToString" ||
            signature.ReturnType != PrimitiveType.String || signature.ParameterTypes.Count != 0 || signature.GenericParameterNames.Count != 0))
            throw new ArgumentException("override reference requires instance value ToString(): String");
        bool constructor = name == ".ctor";
        bool isInterface = authoredInterfaces.Contains(declaringType);
        if (isInterface && (isStatic || constructor || signature.GenericParameterNames.Count != 0))
            throw new ArgumentException("interface contract requires a nongeneric instance method");
        if (string.IsNullOrEmpty(name) || name.Length > 1024 || name == ".cctor" || name.Any(char.IsControl) ||
            constructor && (isStatic || signature.ReturnType != PrimitiveType.Void || signature.GenericParameterNames.Count != 0) ||
            !isStatic && signature.GenericParameterNames.Count != 0)
            throw new ArgumentException("unsupported method or constructor contract", nameof(name));
        if (!Supported(signature.ReturnType, true) || signature.ParameterTypes.Any(t => !Supported(t.ByReferenceElement ?? t, false)) || constructor && signature.ParameterTypes.Any(t => t.ByReferenceElement is not null))
            throw new InvalidDataException("unsupported authored method signature");
        signature.ValidateOwner(this, declaringType.GenericArity);
        foreach (var existing in authoredCallableReferences)
        {
            if (!Equals(existing.DeclaringReference, declaringType) || existing.Name != name ||
                existing.Signature.GenericParameterNames.Count != signature.GenericParameterNames.Count ||
                !existing.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)) continue;
            if (existing.Target.NativeValueOverride != isOverride || existing.IsStatic != isStatic || !existing.Signature.Matches(signature))
                throw new InvalidDataException("conflicting method contract");
            return existing;
        }
        if (completedInterfaceContracts.ContainsKey(declaringType)) throw new InvalidOperationException("interface contract is complete");
        if (authoredCallableReferences.Count + importedReferences.Count >= 4096) throw new InvalidDataException("too many imported methods");
        TypeBuilder MaterializeOwner(ImportedTypeReference type)
        {
            var result = new TypeBuilder(graph.Graph, type.Namespace, type.Name, isStatic: false,
                genericNames: Enumerable.Range(0, type.GenericArity).Select(i => "T" + i).ToArray(),
                isInterface: authoredInterfaces.Contains(type), isValueType: type.IsValueType);
            result.Definition.AuthoredDeclaringType = type.DeclaringType is { } parent ? MaterializeOwner(parent).Definition : null;
            return result;
        }
        var owner = MaterializeOwner(declaringType);
        var reference = new ImportedMethodReference(this, new MethodBuilder(graph.Graph, owner, name, signature, isStatic: isStatic))
        { DeclaringReference = declaringType, RequiresVirtualDispatch = isInterface };
        reference.Target.NativeValueOverride = isOverride;
        authoredCallableReferences.Add(reference);
        return reference;

        static bool Supported(SignatureType type, bool result) =>
            type.FunctionSignature is { } function ? Supported(function.ReturnType, true) && function.ParameterTypes.All(p => Supported(p, false)) :
            type.Primitive is { } primitive ? primitive != PrimitiveType.Void || result :
            type.ImportedType is { } nominal ? nominal.TypeArguments.All(t => Supported(t, false)) :
            type.MethodParameterIndex is not null || type.TypeParameterIndex is not null ||
            type.ArrayElement is { ArrayElement: null } element && Supported(element, false);
    }
}
