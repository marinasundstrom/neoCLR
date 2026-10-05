namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly List<ImportedMethodReference> authoredCallableReferences = [];

    /// <summary>Authors a native namespace-function reference from a resolved contract, without a reader definition.</summary>
    /// <param name="dependency">Exact unsigned dependency identity, distinct from this output.</param>
    /// <param name="dependencyCoreLibrary">Explicit core identity, equal to this output's core.</param>
    /// <param name="artifactSha256">64 hexadecimal digits identifying the selected native dependency image.</param>
    /// <param name="namespace">Function namespace; empty for global functions.</param>
    /// <param name="name">Simple function name.</param>
    /// <param name="signature">Primitive, method-parameter, external top-level nominal construction and single-vector signature; no byrefs.</param>
    /// <returns>An interned output-owned callable reference.</returns>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="ArgumentException">Invalid name, namespace, digest or signature scope.</exception>
    /// <exception cref="InvalidDataException">Unsupported identity/core/signature, conflicting snapshot or callable contract, or resource limit.</exception>
    /// <remarks>The caller supplies resolved semantic facts. This API does not load or validate the dependency image.
    /// The digest detects conflicts within this output; it is not an encoded runtime digest binding.
    /// Native format-5 linking uses namespace/name and signature. Ordinary CLI output rejects global call references.</remarks>
    public ImportedMethodReference CreateFunctionReference(AssemblyIdentity dependency, AssemblyIdentity dependencyCoreLibrary,
        string artifactSha256, string @namespace, string name, MethodSignature signature)
    {
        ArgumentNullException.ThrowIfNull(dependency);
        ArgumentNullException.ThrowIfNull(dependencyCoreLibrary);
        ArgumentNullException.ThrowIfNull(artifactSha256);
        ArgumentNullException.ThrowIfNull(@namespace);
        ArgumentNullException.ThrowIfNull(name);
        ArgumentNullException.ThrowIfNull(signature);
        if (artifactSha256.Length != 64 || !artifactSha256.All(Uri.IsHexDigit))
            throw new ArgumentException("expected SHA-256 hexadecimal digest", nameof(artifactSha256));
        if (!CoreLibrary.Equals(dependencyCoreLibrary) || dependency.Equals(Identity) || dependency.PublicKeyToken.Length != 0 || dependency.Flags != 0)
            throw new InvalidDataException("unsupported dependency identity or core contract");
        if (!Supported(signature.ReturnType, true) || signature.ParameterTypes.Any(t => !Supported(t, false)) || signature.OutParameters.Count != 0)
            throw new InvalidDataException("unsupported authored function signature");
        signature.ReturnType.ValidateOwner(this, signature.GenericParameterNames.Count, 0);
        foreach (var parameter in signature.ParameterTypes) parameter.ValidateOwner(this, signature.GenericParameterNames.Count, 0);
        // Validate declaration spelling before adding anything to the output's reference state.
        _ = new MethodDefinition(name, signature, @namespace: @namespace);
        var snapshot = "native:" + artifactSha256.ToUpperInvariant();
        if (importedGraphs.TryGetValue(dependency, out var graph) && graph.Snapshot != snapshot)
            throw new InvalidDataException("conflicting dependency module snapshots");
        foreach (var existing in authoredCallableReferences.Concat(importedReferences.Values.Where(r => r.DeclaringTypeName is null)))
        {
            if (!existing.AssemblyIdentity.Equals(dependency) || existing.Namespace != (@namespace.Length == 0 ? null : @namespace) ||
                existing.DeclaringTypeName is not null || existing.Name != name || existing.Signature.GenericParameterNames.Count != signature.GenericParameterNames.Count ||
                !existing.Signature.ParameterTypes.SequenceEqual(signature.ParameterTypes)) continue;
            if (!existing.Signature.Matches(signature)) throw new InvalidDataException("conflicting function contract");
            return existing;
        }
        if (authoredCallableReferences.Count + importedReferences.Count >= 4096) throw new InvalidDataException("too many imported methods");
        if (graph.Graph is null)
        {
            if (importedGraphs.Count >= 256) throw new InvalidDataException("too many imported assemblies");
            graph = (snapshot, new AssemblyBuilder(dependency, dependencyCoreLibrary));
            importedGraphs.Add(dependency, graph);
        }
        var reference = new ImportedMethodReference(this, new MethodBuilder(graph.Graph, null, name, signature, @namespace: @namespace));
        authoredCallableReferences.Add(reference);
        return reference;

        static bool Supported(SignatureType type, bool result) =>
            type.Primitive is { } primitive ? primitive != PrimitiveType.Void || result :
            type.ImportedType is { DeclaringType: null } nominal
                ? nominal.TypeArguments.All(argument => Supported(argument, false)) :
            type.MethodParameterIndex is not null || type.ArrayElement is { ArrayElement: null } element && Supported(element, false);
    }
}
