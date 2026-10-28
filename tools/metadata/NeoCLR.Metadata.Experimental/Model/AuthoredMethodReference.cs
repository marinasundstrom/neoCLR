namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<ImportedTypeReference, PrimitiveType> authoredPrimitiveOwners = [];
    /// <summary>Declares the scalar representation of an output-owned external canonical scalar (including unit Void) or String reference type from host semantic facts.</summary>
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
    /// <summary>Authors a public member, class virtual slot, bounded value override or abstract interface method reference without a reader definition.</summary>
    /// <param name="declaringType">Output-owned class/value/interface definition, not a construction.</param>
    /// <param name="name">Simple member name, or .ctor for a constructor.</param>
    /// <param name="signature">Primitive, scoped parameter, external nominal construction, native union, vector or bounded function signature; Self is admitted only for an interface contract.</param>
    /// <param name="isStatic">Whether the member has no receiver. Constructors must be instance members.</param>
    /// <param name="isOverride">Reuse an inherited class slot with isVirtual; value overrides support Object.ToString/GetHashCode/Equals.</param>
    /// <param name="nativePrimitive">Explicit canonical numeric or String owner representation, or null for an ordinary owner.</param>
    /// <param name="visibility">Public by default; Protected is admitted only for constructors and requires a direct derived constructor call.</param>
    /// <param name="isVirtual">True for an ordinary nongeneric method slot on a reference class, including a generic class.</param>
    /// <param name="isAbstract">True for a bodyless new class slot, including on a generic owner; requires isVirtual.</param>
    /// <param name="interfaceBounds">Optional method parameter bounds, limited to output-owned nongeneric nominal interface references. Requires a static generic method.</param>
    /// <returns>An interned output-owned method contract. Construct generic owners before calling.</returns>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="ArgumentException">Invalid owner, name, constructor or signature scope.</exception>
    /// <exception cref="InvalidDataException">Unsupported signature, conflicting contract or reference limit.</exception>
    /// <exception cref="InvalidOperationException">A new method is added after interface completion.</exception>
    /// <remarks>The caller supplies resolved class member semantics or an abstract interface contract. No dependency is loaded or verified.
    /// Authored interfaces require abstract contracts; instance contracts may have unconstrained method parameters. Virtual/interface instance contracts use virtual dispatch; static contracts have no receiver. Writable ref/out parameters are supported; byref constructors and generic class virtual and static interface methods are unsupported. Ordinary nonvirtual generic instance methods are supported. Value/nested owners retain managed receiver and physical scope semantics.
    /// Dependency identity, core and artifact checks are established by the declaring type reference.</remarks>
    public ImportedMethodReference CreateMethodReference(ImportedTypeReference declaringType, string name,
        MethodSignature signature, bool isStatic = false, bool isOverride = false, PrimitiveType? nativePrimitive = null, MethodVisibility visibility = MethodVisibility.Public, bool isVirtual = false, bool isAbstract = false, IReadOnlyList<GenericMethodTypeConstraint>? interfaceBounds = null)
        => CreateMethodReferenceCore(declaringType, name, signature, isStatic, isOverride, nativePrimitive, false, visibility, isVirtual, isAbstract, interfaceBounds);

    /// <summary>Authors a virtual ToString, GetHashCode or Equals reference on the explicitly selected external Object root.</summary>
    /// <remarks>No dependency is loaded. The host supplies the selected root; runtime linking validates its actual slot. Use Callvirt, not Call.</remarks>
    /// <exception cref="InvalidOperationException">No external native root is selected.</exception>
    /// <exception cref="ArgumentException">The name or signature is not an exact supported Object slot.</exception>
    /// <exception cref="InvalidDataException">The reference conflicts with an existing contract.</exception>
    public ImportedMethodReference CreateObjectSlotReference(string name, MethodSignature signature)
    {
        var root = ExternalObjectRoot ?? throw new InvalidOperationException("select an external Object root before authoring slots");
        ArgumentNullException.ThrowIfNull(signature);
        if (!MethodDefinition.IsObjectOverride(name, signature, CoreLibrary, null, root) || signature.OutParameters.Count != 0)
            throw new ArgumentException("Object slot reference requires an exact root signature");
        return CreateMethodReferenceCore(root, name, signature, false, false, null, true, isVirtual: true);
    }

    private ImportedMethodReference CreateMethodReferenceCore(ImportedTypeReference declaringType, string name,
        MethodSignature signature, bool isStatic, bool isOverride, PrimitiveType? nativePrimitive, bool isObjectSlot, MethodVisibility visibility = MethodVisibility.Public, bool isVirtual = false, bool isAbstract = false, IReadOnlyList<GenericMethodTypeConstraint>? interfaceBounds = null)
    {
        ArgumentNullException.ThrowIfNull(declaringType);
        ArgumentNullException.ThrowIfNull(name);
        ArgumentNullException.ThrowIfNull(signature);
        var bounds = ValidateAuthoredInterfaceBounds(signature, interfaceBounds);
        if (bounds.Length != 0 && !isStatic)
            throw new ArgumentException("invalid imported method interface bounds", nameof(interfaceBounds));
        if (visibility != MethodVisibility.Public && !(visibility == MethodVisibility.Protected && name == ".ctor" && !isStatic))
            throw new ArgumentException("only constructors may use protected reference visibility");
        if (!ReferenceEquals(declaringType.Owner, this) ||
            declaringType.TypeArguments.Count != 0 || !importedGraphs.TryGetValue(declaringType.AssemblyIdentity, out var graph))
            throw new ArgumentException("method requires an owned nominal definition", nameof(declaringType));
        if (nativePrimitive is { } primitive && (!TypeDefinition.IsSupportedNativePrimitive(primitive) ||
            declaringType.IsValueType != (primitive != PrimitiveType.String) || declaringType.Namespace != "System" || declaringType.Name != primitive.ToString() ||
            declaringType.GenericArity != 0 || declaringType.DeclaringType is not null || name == ".cctor" || name == ".ctor" && primitive != PrimitiveType.String ||
            isOverride && (primitive != PrimitiveType.String || isStatic || !MethodDefinition.IsObjectOverride(name, signature, CoreLibrary, NativeObjectRoot, ExternalObjectRoot))))
            throw new ArgumentException("invalid native primitive member owner", nameof(nativePrimitive));
        if ((isVirtual || isAbstract) && (isStatic || declaringType.IsValueType || IsAuthoredInterface(declaringType) ||
            declaringType.DeclaringType is not null && !(declaringType.GenericArity == 0 && Equals(DeclaredClassBase(declaringType), declaringType.DeclaringType)) || name is ".ctor" or ".cctor" ||
            signature.GenericParameterNames.Count != 0 || isAbstract && (!isVirtual || isOverride)))
            throw new ArgumentException("class slot requires an ordinary nongeneric instance method");
        if (isOverride && !isVirtual && (isStatic || !declaringType.IsValueType && declaringType.GenericArity != 0 || !MethodDefinition.IsObjectOverride(name, signature, CoreLibrary, NativeObjectRoot, ExternalObjectRoot)))
            throw new ArgumentException("override reference requires an exact instance Object slot signature");
        if (IsNativeGrapheme(declaringType) && (name is ".ctor" or ".cctor" || nativePrimitive is not null ||
            isOverride && (isStatic || !MethodDefinition.IsObjectOverride(name, signature, CoreLibrary, NativeObjectRoot, ExternalObjectRoot))))
            throw new ArgumentException("grapheme members require ordinary methods or exact Object overrides without primitive reinterpretation");
        bool constructor = name == ".ctor";
        bool isInterface = authoredInterfaces.Contains(declaringType);
        if (isInterface && (constructor || isStatic && signature.GenericParameterNames.Count != 0))
            throw new ArgumentException("interface contract requires an ordinary instance method or nongeneric static method");
        if (string.IsNullOrEmpty(name) || name.Length > 1024 || name == ".cctor" || name.Any(char.IsControl) ||
            constructor && (isStatic || signature.ReturnType != PrimitiveType.Void || signature.GenericParameterNames.Count != 0))
            throw new ArgumentException("unsupported method or constructor contract", nameof(name));
        if (!Supported(signature.ReturnType, true) || signature.ParameterTypes.Any(t => !Supported(t.ByReferenceElement ?? t, false)) || constructor && signature.ParameterTypes.Any(t => t.ByReferenceElement is not null))
            throw new InvalidDataException($"unsupported authored method signature: {declaringType}.{name}({string.Join(", ", signature.ParameterTypes)}) -> {signature.ReturnType}");
        signature.ValidateOwner(this, declaringType.GenericArity, allowSelf: isInterface);
        if (authoredCallableReferences.Any(m => Equals(m.DeclaringReference, declaringType) && m.Target.DeclaringType?.NativePrimitive != nativePrimitive))
            throw new InvalidDataException("conflicting primitive owner contract");
        foreach (var existing in authoredCallableReferences)
        {
            if (!Equals(existing.DeclaringReference, declaringType) || existing.Name != name ||
                existing.Signature.GenericParameterNames.Count != signature.GenericParameterNames.Count ||
                !existing.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)) continue;
            if (!existing.Target.Definition.TypeConstraints.SequenceEqual(bounds) || existing.Target.Visibility != visibility || existing.Target.NativeImportObjectSlot != isObjectSlot || existing.Target.DeclaringType?.NativePrimitive != nativePrimitive || existing.Target.NativeValueOverride != isOverride || existing.Target.IsAbstract != (isAbstract || isInterface) || (existing.Target.IsVirtual != (isVirtual || isInterface)) || existing.IsStatic != isStatic || !existing.Signature.Matches(signature))
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
            if (ImportedMemoryView(type) is { } view) result.SetMemoryView(view);
            return result;
        }
        if (nativePrimitive is { } declaredPrimitive) SetNativePrimitive(declaringType, declaredPrimitive);
        var owner = MaterializeOwner(declaringType);
        if (nativePrimitive is { } scalar) owner.SetNativePrimitive(scalar);
        if (IsNativeGrapheme(declaringType)) owner.SetNativeGrapheme();
        var reference = new ImportedMethodReference(this, isVirtual
            ? new MethodBuilder(graph.Graph, new MethodDefinition(name, (ushort)(6 | 0x40 | (isOverride ? 0 : 0x100) | (isAbstract ? 0x400 : 0)), signature), owner)
            : new MethodBuilder(graph.Graph, owner, name, signature, visibility: visibility, isStatic: isStatic))
        { DeclaringReference = declaringType, RequiresVirtualDispatch = (isObjectSlot || isInterface || isVirtual || isOverride && !declaringType.IsValueType) && !isStatic };
        reference.Target.ConstraintOwner = this;
        reference.Target.Definition.AuthoredTypeConstraints.AddRange(bounds);
        reference.Target.NativeValueOverride = isOverride;
        reference.Target.NativeImportObjectSlot = isObjectSlot;
        reference.Target.NativeImportObjectOwner = Equals(ExternalObjectRoot, declaringType);
        if (reference.Target.NativeImportObjectOwner) reference.Target.NativeImportName = "System.Object." + (constructor ? ".ctor" : name);
        reference.Target.NativeImportPrimitiveOwner = nativePrimitive;
        reference.Target.NativeImportCharOwner = IsNativeGrapheme(declaringType);
        authoredCallableReferences.Add(reference);
        return reference;

        bool Supported(SignatureType type, bool result) => type.PointerElement is not null ? true :
            type.IsSelf ? isInterface :
            type.Composition is { Kind: ReferenceCompositionKind.Union } union ? union.Constituents.All(t => Supported(t, false)) :
            type.Tuple is { } tuple ? tuple.ElementTypes.All(t => Supported(t, false)) :
            type.FunctionSignature is { } function ? Supported(function.ReturnType, true) && function.ParameterTypes.All(p => Supported(p, false)) :
            type.Primitive is { } primitive ? primitive != PrimitiveType.Void || result :
            type.ImportedType is { } nominal ? nominal.TypeArguments.All(t => Supported(t, false)) :
            type.MethodParameterIndex is not null || type.TypeParameterIndex is not null ||
            type.ArrayElement is { ArrayElement: null } element && Supported(element, false);
    }

    private GenericMethodTypeConstraint[] ValidateAuthoredInterfaceBounds(MethodSignature signature,
        IReadOnlyList<GenericMethodTypeConstraint>? interfaceBounds)
    {
        var bounds = interfaceBounds?.ToArray() ?? [];
        if (bounds.Length > 128 || bounds.Distinct().Count() != bounds.Length || bounds.Any(c => c is null ||
            c.ParameterIndex < 0 || c.ParameterIndex >= signature.GenericParameterNames.Count ||
            c.Bound?.ImportedType is not { } bound || !ReferenceEquals(bound.Owner, this) ||
            bound.GenericArity != 0 || bound.TypeArguments.Count != 0 || !IsAuthoredInterface(bound)))
            throw new ArgumentException("invalid imported callable interface bounds", nameof(interfaceBounds));
        return bounds;
    }

}
