namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    /// <summary>Authors a top-level native reference-class identity without a reader definition.</summary>
    /// <param name="dependency">Exact unsigned external assembly identity.</param>
    /// <param name="dependencyCoreLibrary">Explicit core identity, equal to this output's core.</param>
    /// <param name="artifactSha256">64 hexadecimal SHA-256 digits for the selected native image.</param>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Metadata name, including the arity suffix for a generic definition.</param>
    /// <param name="genericArity">Number of invariant, unconstrained parameters, from zero to 32.</param>
    /// <returns>An interned output-owned definition reference; construct generic references before signature use.</returns>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="ArgumentException">The digest is malformed.</exception>
    /// <exception cref="InvalidDataException">Unsupported identity/name/arity/core, snapshot conflict or resource limit.</exception>
    /// <remarks>The caller asserts a public top-level reference class with no required inheritance or interface conversions.
    /// This does not load or check the dependency, import members, or preserve inheritance/constraint information.
    /// The digest is an output-local consistency check, not an encoded runtime integrity guarantee.</remarks>
    public ImportedTypeReference CreateTypeReference(AssemblyIdentity dependency, AssemblyIdentity dependencyCoreLibrary,
        string artifactSha256, string @namespace, string name, int genericArity = 0)
        => CreateNominalReference(dependency, dependencyCoreLibrary, artifactSha256, @namespace, name, genericArity, false);

    /// <summary>Authors a public unconstrained value-type identity from explicit dependency facts.</summary>
    /// <param name="dependency">Exact unsigned external assembly identity.</param>
    /// <param name="dependencyCoreLibrary">Core identity, equal to the output core.</param>
    /// <param name="artifactSha256">64 hexadecimal SHA-256 digits for the selected native image.</param>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Metadata name including any generic arity suffix.</param>
    /// <param name="genericArity">Number of invariant unconstrained parameters, zero to 32.</param>
    /// <returns>An interned output-owned value definition reference.</returns>
    /// <exception cref="ArgumentException">The digest is malformed.</exception>
    /// <exception cref="InvalidDataException">Unsupported identity, core, name, arity, conflicting snapshot or resource limit.</exception>
    /// <remarks>Core, digest, ownership and limits match CreateTypeReference. No dependency is loaded.</remarks>
    public ImportedTypeReference CreateValueTypeReference(AssemblyIdentity dependency, AssemblyIdentity dependencyCoreLibrary,
        string artifactSha256, string @namespace, string name, int genericArity = 0)
        => CreateNominalReference(dependency, dependencyCoreLibrary, artifactSha256, @namespace, name, genericArity, false, true);

    /// <summary>Authors a public nested reference/value identity beneath an output-owned nongeneric scope.</summary>
    /// <param name="declaringType">An output-owned, unconstructed nongeneric native scope.</param>
    /// <param name="name">Simple nested metadata name including any arity suffix.</param>
    /// <param name="genericArity">The nested type's own generic arity.</param>
    /// <param name="isValueType">Whether the identity denotes a value type.</param>
    /// <returns>An interned scoped reference.</returns>
    /// <exception cref="ArgumentException">The scope is not owned by this output or lacks a native artifact contract.</exception>
    /// <exception cref="InvalidDataException">Unsupported identity, nesting or resource limit.</exception>
    /// <remarks>The declaring scope must carry a native artifact contract. Namespace is empty; generic arity belongs to the nested type alone.</remarks>
    public ImportedTypeReference CreateNestedTypeReference(ImportedTypeReference declaringType, string name,
        int genericArity = 0, bool isValueType = false)
    {
        ArgumentNullException.ThrowIfNull(declaringType);
        if (!ReferenceEquals(declaringType.Owner, this) || !importedGraphs.TryGetValue(declaringType.AssemblyIdentity, out var graph) ||
            !graph.Snapshot.StartsWith("native:", StringComparison.Ordinal))
            throw new ArgumentException("nested identity requires an output-owned native scope", nameof(declaringType));
        var reference = ImportTypeIdentity(declaringType.AssemblyIdentity, "", name, genericArity, isValueType, declaringType);
        RegisterNominalKind(reference, false);
        return reference;
    }

    private ImportedTypeReference CreateNominalReference(AssemblyIdentity dependency, AssemblyIdentity dependencyCoreLibrary,
        string artifactSha256, string @namespace, string name, int genericArity, bool isInterface, bool isValueType = false)
    {
        ArgumentNullException.ThrowIfNull(dependency);
        ArgumentNullException.ThrowIfNull(dependencyCoreLibrary);
        ArgumentNullException.ThrowIfNull(artifactSha256);
        ArgumentNullException.ThrowIfNull(@namespace);
        ArgumentNullException.ThrowIfNull(name);
        if (artifactSha256.Length != 64 || !artifactSha256.All(Uri.IsHexDigit))
            throw new ArgumentException("expected SHA-256 hexadecimal digest", nameof(artifactSha256));
        if (!CoreLibrary.Equals(dependencyCoreLibrary)) throw new InvalidDataException("incompatible core contract");
        var snapshot = "native:" + artifactSha256.ToUpperInvariant();
        if (importedGraphs.TryGetValue(dependency, out var prior) && prior.Snapshot != snapshot)
            throw new InvalidDataException("conflicting dependency module snapshots");
        var reference = ImportTypeIdentity(dependency, @namespace, name, genericArity, isValueType);
        RegisterNominalKind(reference, isInterface);
        if (!importedGraphs.ContainsKey(dependency))
            importedGraphs.Add(dependency, (snapshot, new AssemblyBuilder(dependency, dependencyCoreLibrary)));
        return reference;
    }
    private readonly Dictionary<ImportedTypeReference, bool> nominalKinds = [];
    private void RegisterNominalKind(ImportedTypeReference reference, bool isInterface)
    {
        if (nominalKinds.TryGetValue(reference, out var prior) && prior != isInterface)
            throw new InvalidDataException("conflicting nominal type classification");
        nominalKinds[reference] = isInterface;
    }
}
