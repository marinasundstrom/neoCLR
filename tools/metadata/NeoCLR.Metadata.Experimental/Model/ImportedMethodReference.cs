namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable callable reference imported into one output assembly.</summary>
/// <remarks>Supports static calls, public reference/value-instance calls and interface dispatch with bounded generic owners and signatures. Import does not load code or verify access or native dependency availability.</remarks>
public sealed partial class ImportedMethodReference
{
    internal ImportedMethodReference(AssemblyBuilder owner, MethodBuilder target) { Owner = owner; Target = target; }
    internal MethodBuilder Target { get; }
    internal ImportedTypeReference? DeclaringReference { get; init; }
    /// <summary>Gets whether this reference allocates through Newobj rather than an ordinary call.</summary>
    public bool IsConstructor => Target.IsConstructor;
    /// <summary>Gets whether this member has no receiver.</summary>
    public bool IsStatic => Target.IsStatic;
    /// <summary>Gets whether this member requires interface dispatch.</summary>
    public bool IsInterfaceMethod => Target.IsAbstract;
    /// <summary>Gets whether the imported contract requires a null-checking virtual call.</summary>
    public bool RequiresVirtualDispatch { get; internal init; }
    /// <summary>Gets whether this value member's implicit receiver is a managed address; Newobj supplies the constructor receiver.</summary>
    public bool RequiresManagedReceiver => !IsStatic && Target.DeclaringType!.IsValueType;
    /// <summary>Gets the consuming assembly builder.</summary>
    public AssemblyBuilder Owner { get; }
    /// <summary>Gets the exact dependency identity.</summary>
    public AssemblyIdentity AssemblyIdentity => Target.Assembly.Identity;
    /// <summary>Gets the declaring/function namespace, or null for a global-namespace function.</summary>
    public string? Namespace => Target.DeclaringType?.Namespace ?? (Target.Namespace.Length == 0 ? null : Target.Namespace);
    /// <summary>Gets the top-level declaring type name, or null for a global function.</summary>
    public string? DeclaringTypeName => Target.DeclaringType?.Name;
    /// <summary>Gets the method name.</summary>
    public string Name => Target.Name;
    /// <summary>Gets the number of parameters.</summary>
    public int ParameterCount => Target.ParameterCount;
    /// <summary>Gets whether a result is present.</summary>
    public bool ReturnsValue => Target.ReturnsValue;
    /// <summary>Gets the immutable imported value signature.</summary>
    public MethodSignature Signature => Target.Signature;
}

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<(AssemblyIdentity Identity, uint Token), ImportedMethodReference> importedReferences = [];
    private readonly Dictionary<AssemblyIdentity, (string Snapshot, AssemblyBuilder Graph)> importedGraphs = [];

    /// <summary>Imports an immutable callable contract from a read-only definition.</summary>
    /// <param name="definition">External static method/global function, public nonvirtual/final class member or concrete value member, or public abstract interface member with bounded nominal signatures.</param>
    /// <param name="dependencyCoreLibrary">Host-asserted dependency core contract; must equal this output's explicit core identity.</param>
    /// <returns>A reference owned by this output builder, independent of the producer's mutable graph.</returns>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="InvalidDataException">Unsupported signature/owner, conflicting identity or module snapshot, incompatible core contract, or resource limit.</exception>
    /// <remarks>No core identity is inferred from the host or from primitive signature bytes. Nested public owners are supported; signed dependencies are unsupported. Generic nominal owners must be invariant and unconstrained; instance methods must be nongeneric.
    /// Nominal signature types must be public top-level unconstrained class/interface/value definitions in the same dependency; cross-dependency TypeRef signatures require further contracts.
    /// Native primitive namespace functions and bounded static-type methods are imported directly; native snapshots use an image fingerprint instead of a CLI MVID. Global references support native emission only. The native dependency must use the same format-5 naming contract as this writer.</remarks>
    public ImportedMethodReference ImportReference(MethodDefinition definition, AssemblyIdentity dependencyCoreLibrary)
    {
        ArgumentNullException.ThrowIfNull(definition);
        ArgumentNullException.ThrowIfNull(dependencyCoreLibrary);
        var identity = definition.Module.Assembly.Identity;
        var type = definition.DeclaringType;
        var function = type is not null
            ? (Namespace: type.Namespace, Name: definition.Name)
            : definition.Module.Assembly.IsNative
                ? (Namespace: definition.Namespace, Name: definition.Name)
                : FunctionNamespaceEncoding.Decode(definition.Name);
        if (!CoreLibrary.Equals(dependencyCoreLibrary)) throw new InvalidDataException("cross-target call requires compatible core identity");
        if (identity.Equals(Identity) || identity.PublicKeyToken.Length != 0 || identity.Flags != 0)
            throw new InvalidDataException("unsupported external assembly identity");
        if (definition.Name == ".cctor") throw new InvalidDataException("type initializer import unsupported");
        bool isInterface = type is not null && (type.Attributes & 0x20) != 0;
        bool isVirtual = (definition.Attributes & 0x40) != 0;
        bool isAbstract = (definition.Attributes & 0x400) != 0;
        bool isFinal = (definition.Attributes & 0x20) != 0;
        if (type is { GenericArity: > 0, CanImportReference: false })
            throw new InvalidDataException("unsupported imported method owner");
        if (!definition.IsStatic && (type is null || definition.GenericArity != 0 ||
            definition.Name == ".cctor" || (definition.Attributes & 7) != 6 ||
            (isInterface ? !isAbstract || !isVirtual : isAbstract || isVirtual && !isFinal && !type.IsValueType)))
            throw new InvalidDataException("unsupported imported instance method contract");
        if (definition.Name == ".ctor" && (definition.IsStatic || isInterface || isVirtual || isAbstract || (definition.Attributes & 0x1800) != 0x1800))
            throw new InvalidDataException("invalid imported constructor contract");
        var declaringReference = type is not null && (!definition.IsStatic || type.GenericArity > 0)
            ? ImportReference(type, dependencyCoreLibrary) : null;
        var signature = definition.DecodeImportedSignature(this, dependencyCoreLibrary);
        if (definition.Name == ".ctor" && (signature!.ReturnType != PrimitiveType.Void || signature.GenericParameterNames.Count != 0 || signature.ParameterTypes.Any(p => p.ByReferenceElement is not null)))
            throw new InvalidDataException("constructor requires a nongeneric void signature without byref parameters");
        if (!importedGraphs.TryGetValue(identity, out var imported))
        {
            if (importedGraphs.Count >= 256) throw new InvalidDataException("too many imported assemblies");
            imported = (definition.Module.Assembly.ImportSnapshotIdentity, new AssemblyBuilder(identity, dependencyCoreLibrary));
            importedGraphs.Add(identity, imported);
        }
        else if (imported.Snapshot != definition.Module.Assembly.ImportSnapshotIdentity) throw new InvalidDataException("conflicting dependency module snapshots");
        var key = (identity, definition.MetadataToken);
        if (importedReferences.TryGetValue(key, out var existing))
        {
            if (existing.Name != function.Name || existing.Namespace != (type?.Namespace ?? (function.Namespace.Length == 0 ? null : function.Namespace)) || existing.DeclaringTypeName != type?.Name ||
                !existing.Signature.Matches(signature!))
                throw new InvalidDataException("conflicting imported method contract");
            return existing;
        }
        if (importedReferences.Count >= 4096) throw new InvalidDataException("too many imported methods");
        // Private reference-only nodes reuse both backends' existing exact-identity call encoding.
        // No producer bodies or mutable definition graph are retained or exposed.
        TypeBuilder? MakeOwner(TypeDefinition? declaration)
        {
            if (declaration is null) return null;
            var result = new TypeBuilder(imported.Graph, declaration.Namespace, declaration.Name, isStatic: false,
                genericNames: Enumerable.Range(0, declaration.GenericArity).Select(i => "T" + i).ToArray(),
                isInterface: (declaration.Attributes & 0x20) != 0, isValueType: declaration.IsValueType);
            result.Definition.AuthoredDeclaringType = MakeOwner(declaration.DeclaringType)?.Definition;
            return result;
        }
        var owner = MakeOwner(type);
        var reference = new ImportedMethodReference(this, new MethodBuilder(imported.Graph, owner, function.Name, signature!, @namespace: function.Namespace, isStatic: definition.IsStatic)) { DeclaringReference = declaringReference, RequiresVirtualDispatch = !definition.IsStatic && !type!.IsValueType && isVirtual };
        NativeBindingFor(identity)?.ValidateMethod(definition, reference.Target);
        importedReferences.Add(key, reference);
        return reference;
    }
}
