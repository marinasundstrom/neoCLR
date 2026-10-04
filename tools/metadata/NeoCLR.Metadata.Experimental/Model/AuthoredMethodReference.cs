namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<ImportedTypeReference, PrimitiveType> authoredPrimitiveOwners = [];
    /// <summary>Declares the scalar representation of an output-owned external numeric value type or String reference type from host semantic facts.</summary>
    /// <exception cref="ArgumentException">The reference is foreign or does not name the matching nongeneric System primitive category.</exception>
    /// <exception cref="InvalidDataException">Another dependency already owns the scalar representation.</exception>
    public void SetNativePrimitive(ImportedTypeReference type, PrimitiveType primitive)
    {
        ArgumentNullException.ThrowIfNull(type);
        if (!ReferenceEquals(type.Owner, this) || !TypeDefinition.IsSupportedNativePrimitive(primitive) || type.IsValueType != (primitive != PrimitiveType.String) ||
            type.Namespace != "System" || type.Name != primitive.ToString() || type.GenericArity != 0 || type.DeclaringType is not null)
            throw new ArgumentException("invalid external primitive designation");
        if (authoredPrimitiveOwners.Any(p => p.Value == primitive && !Equals(p.Key, type)))
            throw new InvalidDataException("conflicting external primitive ownership");
        authoredPrimitiveOwners[type] = primitive;
    }
    internal ImportedTypeReference? ExternalPrimitive(PrimitiveType primitive) => authoredPrimitiveOwners.SingleOrDefault(p => p.Value == primitive).Key;
    internal PrimitiveType? AuthoredPrimitiveOwner(ImportedTypeReference type) => authoredPrimitiveOwners.TryGetValue(type, out var primitive) ? primitive : null;
    /// <summary>Authors a public nonvirtual member, bounded value override or abstract interface method reference without a reader definition.</summary>
    /// <param name="declaringType">Output-owned class/value/interface definition, not a construction.</param>
    /// <param name="name">Simple member name, or .ctor for a constructor.</param>
    /// <param name="signature">Primitive, scoped parameter, external nominal construction, vector or bounded function signature; Self is admitted only for an interface contract.</param>
    /// <param name="isStatic">Whether the member has no receiver. Constructors must be instance members.</param>
    /// <param name="isOverride">Reuse the inherited slot; currently only instance value-type ToString(): String is supported.</param>
    /// <param name="nativePrimitive">Explicit canonical numeric or String owner representation, or null for an ordinary owner.</param>
    /// <returns>An interned output-owned method contract. Construct generic owners before calling.</returns>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="ArgumentException">Invalid owner, name, constructor or signature scope.</exception>
    /// <exception cref="InvalidDataException">Unsupported signature, conflicting contract or reference limit.</exception>
    /// <exception cref="InvalidOperationException">A new method is added after interface completion.</exception>
    /// <remarks>The caller supplies public nonvirtual class semantics or an abstract interface contract. No dependency is loaded or verified.
    /// Authored interfaces require nongeneric abstract contracts. Instance contracts use virtual dispatch; static contracts have no receiver. Writable ref/out parameters are supported; byref constructors, instance generic methods are unsupported. Value/nested owners retain managed receiver and physical scope semantics.
    /// Dependency identity, core and artifact checks are established by the declaring type reference.</remarks>
    public ImportedMethodReference CreateMethodReference(ImportedTypeReference declaringType, string name,
        MethodSignature signature, bool isStatic = false, bool isOverride = false, PrimitiveType? nativePrimitive = null)
    {
        ArgumentNullException.ThrowIfNull(declaringType);
        ArgumentNullException.ThrowIfNull(name);
        ArgumentNullException.ThrowIfNull(signature);
        if (!ReferenceEquals(declaringType.Owner, this) ||
            declaringType.TypeArguments.Count != 0 || !importedGraphs.TryGetValue(declaringType.AssemblyIdentity, out var graph))
            throw new ArgumentException("method requires an owned nominal definition", nameof(declaringType));
        if (nativePrimitive is { } primitive && (!TypeDefinition.IsSupportedNativePrimitive(primitive) ||
            declaringType.IsValueType != (primitive != PrimitiveType.String) || declaringType.Namespace != "System" || declaringType.Name != primitive.ToString() ||
            declaringType.GenericArity != 0 || declaringType.DeclaringType is not null || name == ".cctor" || name == ".ctor" && primitive != PrimitiveType.String || isOverride))
            throw new ArgumentException("invalid native primitive member owner", nameof(nativePrimitive));
        if (isOverride && (!declaringType.IsValueType || isStatic || name != "ToString" ||
            signature.ReturnType != PrimitiveType.String || signature.ParameterTypes.Count != 0 || signature.GenericParameterNames.Count != 0))
            throw new ArgumentException("override reference requires instance value ToString(): String");
        if (IsNativeGrapheme(declaringType) && (name is ".ctor" or ".cctor" || isOverride || nativePrimitive is not null))
            throw new ArgumentException("grapheme members require ordinary methods without primitive reinterpretation");
        bool constructor = name == ".ctor";
        bool isInterface = authoredInterfaces.Contains(declaringType);
        if (isInterface && (constructor || signature.GenericParameterNames.Count != 0))
            throw new ArgumentException("interface contract requires a nongeneric method");
        if (string.IsNullOrEmpty(name) || name.Length > 1024 || name == ".cctor" || name.Any(char.IsControl) ||
            constructor && (isStatic || signature.ReturnType != PrimitiveType.Void || signature.GenericParameterNames.Count != 0) ||
            !isStatic && signature.GenericParameterNames.Count != 0)
            throw new ArgumentException("unsupported method or constructor contract", nameof(name));
        if (!Supported(signature.ReturnType, true) || signature.ParameterTypes.Any(t => !Supported(t.ByReferenceElement ?? t, false)) || constructor && signature.ParameterTypes.Any(t => t.ByReferenceElement is not null))
            throw new InvalidDataException("unsupported authored method signature");
        signature.ValidateOwner(this, declaringType.GenericArity, allowSelf: isInterface);
        if (authoredCallableReferences.Any(m => Equals(m.DeclaringReference, declaringType) && m.Target.DeclaringType?.NativePrimitive != nativePrimitive))
            throw new InvalidDataException("conflicting primitive owner contract");
        foreach (var existing in authoredCallableReferences)
        {
            if (!Equals(existing.DeclaringReference, declaringType) || existing.Name != name ||
                existing.Signature.GenericParameterNames.Count != signature.GenericParameterNames.Count ||
                !existing.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)) continue;
            if (existing.Target.DeclaringType?.NativePrimitive != nativePrimitive || existing.Target.NativeValueOverride != isOverride || existing.IsStatic != isStatic || !existing.Signature.Matches(signature))
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
        if (nativePrimitive is { } declaredPrimitive) SetNativePrimitive(declaringType, declaredPrimitive);
        var owner = MaterializeOwner(declaringType);
        if (nativePrimitive is { } scalar) owner.SetNativePrimitive(scalar);
        if (IsNativeGrapheme(declaringType)) owner.SetNativeGrapheme();
        var reference = new ImportedMethodReference(this, new MethodBuilder(graph.Graph, owner, name, signature, isStatic: isStatic))
        { DeclaringReference = declaringType, RequiresVirtualDispatch = isInterface && !isStatic };
        reference.Target.NativeValueOverride = isOverride;
        reference.Target.NativeImportPrimitiveOwner = nativePrimitive;
        reference.Target.NativeImportCharOwner = IsNativeGrapheme(declaringType);
        authoredCallableReferences.Add(reference);
        return reference;

        bool Supported(SignatureType type, bool result) =>
            type.IsSelf ? isInterface :
            type.FunctionSignature is { } function ? Supported(function.ReturnType, true) && function.ParameterTypes.All(p => Supported(p, false)) :
            type.Primitive is { } primitive ? primitive != PrimitiveType.Void || result :
            type.ImportedType is { } nominal ? nominal.TypeArguments.All(t => Supported(t, false)) :
            type.MethodParameterIndex is not null || type.TypeParameterIndex is not null ||
            type.ArrayElement is { ArrayElement: null } element && Supported(element, false);
    }
}
