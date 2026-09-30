namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable callable reference imported into one output assembly.</summary>
/// <remarks>Supports static primitive signatures only. Import does not load code or verify access or native dependency availability.</remarks>
public sealed class ImportedMethodReference
{
    internal ImportedMethodReference(AssemblyBuilder owner, MethodBuilder target) { Owner = owner; Target = target; }
    internal MethodBuilder Target { get; }
    /// <summary>Gets the consuming assembly builder.</summary>
    public AssemblyBuilder Owner { get; }
    /// <summary>Gets the exact dependency identity.</summary>
    public AssemblyIdentity AssemblyIdentity => Target.Assembly.Identity;
    /// <summary>Gets the declaring namespace, or null for a global function.</summary>
    public string? Namespace => Target.DeclaringType?.Namespace;
    /// <summary>Gets the top-level declaring type name, or null for a global function.</summary>
    public string? DeclaringTypeName => Target.DeclaringType?.Name;
    /// <summary>Gets the method name.</summary>
    public string Name => Target.Name;
    /// <summary>Gets the number of parameters.</summary>
    public int ParameterCount => Target.ParameterCount;
    /// <summary>Gets whether a result is present.</summary>
    public bool ReturnsValue => Target.ReturnsValue;
    /// <summary>Gets the immutable imported primitive signature.</summary>
    public PrimitiveMethodSignature Signature => Target.Signature;
}

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<(AssemblyIdentity Identity, uint Token), ImportedMethodReference> importedReferences = [];
    private readonly Dictionary<AssemblyIdentity, (Guid Mvid, AssemblyBuilder Graph)> importedGraphs = [];

    /// <summary>Imports an immutable callable contract from a read-only definition.</summary>
    /// <param name="definition">External static nongeneric primitive method or global function.</param>
    /// <param name="dependencyCoreLibrary">Host-asserted dependency core contract; must equal this output's explicit core identity.</param>
    /// <returns>A reference owned by this output builder, independent of the producer's mutable graph.</returns>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="InvalidDataException">Unsupported signature/owner, conflicting identity or module snapshot, incompatible core contract, or resource limit.</exception>
    /// <remarks>No core identity is inferred from the host or from primitive signature bytes. Nested/generic owners and signed dependencies are currently unsupported.
    /// Global references support native emission only. The native dependency must use the same format-5 naming contract as this writer.</remarks>
    public ImportedMethodReference ImportReference(MethodDefinition definition, AssemblyIdentity dependencyCoreLibrary)
    {
        ArgumentNullException.ThrowIfNull(definition);
        ArgumentNullException.ThrowIfNull(dependencyCoreLibrary);
        var identity = definition.Module.Assembly.Identity;
        var type = definition.DeclaringType;
        if (!CoreLibrary.Equals(dependencyCoreLibrary)) throw new InvalidDataException("cross-target call requires compatible core identity");
        if (identity.Equals(Identity) || identity.PublicKeyToken.Length != 0 || identity.Flags != 0)
            throw new InvalidDataException("unsupported external assembly identity");
        if (!definition.TryGetStaticPrimitiveSignature(out var signature) ||
            type is { GenericArity: not 0 } || type?.DeclaringType is not null)
            throw new InvalidDataException("unsupported imported method signature or owner");
        if (!importedGraphs.TryGetValue(identity, out var imported))
        {
            if (importedGraphs.Count >= 256) throw new InvalidDataException("too many imported assemblies");
            imported = (definition.Module.Mvid, new AssemblyBuilder(identity, dependencyCoreLibrary));
            importedGraphs.Add(identity, imported);
        }
        else if (imported.Mvid != definition.Module.Mvid) throw new InvalidDataException("conflicting dependency module snapshots");
        var key = (identity, definition.MetadataToken);
        if (importedReferences.TryGetValue(key, out var existing))
        {
            if (existing.Name != definition.Name || existing.Namespace != type?.Namespace || existing.DeclaringTypeName != type?.Name ||
                !existing.Signature.Matches(signature!))
                throw new InvalidDataException("conflicting imported method contract");
            return existing;
        }
        if (importedReferences.Count >= 4096) throw new InvalidDataException("too many imported methods");
        // Private reference-only nodes reuse both backends' existing exact-identity call encoding.
        // No producer bodies or mutable definition graph are retained or exposed.
        var owner = type is null ? null : new TypeBuilder(imported.Graph, type.Namespace, type.Name);
        var reference = new ImportedMethodReference(this, new MethodBuilder(imported.Graph, owner, definition.Name, signature!));
        importedReferences.Add(key, reference);
        return reference;
    }
}
