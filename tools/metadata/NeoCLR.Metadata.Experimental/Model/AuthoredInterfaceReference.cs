namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly HashSet<ImportedTypeReference> authoredInterfaces = [];
    private int authoredInterfaceEdgeCount;
    private readonly Dictionary<ImportedTypeReference, HashSet<ImportedTypeReference>> authoredInterfaceBases = [];

    /// <summary>Authors a public nongeneric top-level native interface identity without a reader.</summary>
    /// <param name="dependency">Exact unsigned external identity.</param>
    /// <param name="dependencyCoreLibrary">Matching explicit core identity.</param>
    /// <param name="artifactSha256">Selected native image SHA-256 in hexadecimal.</param>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Nongeneric metadata name.</param>
    /// <returns>An output-owned interface reference.</returns>
    /// <remarks>Identity, digest, validation errors and limits follow CreateTypeReference.
    /// The caller asserts interface classification; no dependency is read.</remarks>
    public ImportedTypeReference CreateInterfaceReference(AssemblyIdentity dependency, AssemblyIdentity dependencyCoreLibrary,
        string artifactSha256, string @namespace, string name)
    {
        var reference = CreateNominalReference(dependency, dependencyCoreLibrary, artifactSha256, @namespace, name, 0, true);
        authoredInterfaces.Add(reference);
        return reference;
    }

    /// <summary>Records a direct interface inheritance or class implementation edge from resolved semantic facts.</summary>
    /// <param name="source">Owned nongeneric top-level reference class or interface.</param>
    /// <param name="target">Owned interface created by CreateInterfaceReference.</param>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="ArgumentException">Foreign/unsupported endpoint, noninterface target or cyclic edge.</exception>
    /// <exception cref="InvalidDataException">More than 4096 direct conversion edges.</exception>
    /// <remarks>No definition lookup occurs. Transitive conversions are derived from these edges.
    /// The caller is responsible for truthful declarations; this does not synthesize runtime implementations.</remarks>
    public void AddInterfaceConversion(ImportedTypeReference source, ImportedTypeReference target)
    {
        ArgumentNullException.ThrowIfNull(source); ArgumentNullException.ThrowIfNull(target);
        if (!ReferenceEquals(source.Owner, this) || !ReferenceEquals(target.Owner, this) || source.GenericArity != 0 ||
            source.IsValueType || source.DeclaringType is not null || !authoredInterfaces.Contains(target) ||
            source.Equals(target) || HasAuthoredInterfaceConversion(target, source))
            throw new ArgumentException("invalid or cyclic interface conversion");
        if (!authoredInterfaceBases.TryGetValue(source, out var edges)) authoredInterfaceBases.Add(source, edges = []);
        if (edges.Contains(target)) return;
        if (authoredInterfaceEdgeCount >= 4096) throw new InvalidDataException("too many interface conversion edges");
        edges.Add(target); authoredInterfaceEdgeCount++;
        nativeInterfaceConversions.Clear();
    }

    private bool HasAuthoredInterfaceConversion(ImportedTypeReference source, ImportedTypeReference target)
    {
        var seen = new HashSet<ImportedTypeReference>();
        var pending = new Stack<ImportedTypeReference>(); pending.Push(source);
        while (pending.TryPop(out var current))
        {
            if (!seen.Add(current)) continue;
            if (current.Equals(target)) return true;
            if (authoredInterfaceBases.TryGetValue(current, out var bases))
                foreach (var parent in bases) pending.Push(parent);
        }
        return false;
    }
}
