namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly HashSet<ImportedTypeReference> authoredInterfaces = [];
    private int authoredInterfaceEdgeCount;
    private readonly Dictionary<ImportedTypeReference, HashSet<ImportedTypeReference>> authoredInterfaceBases = [];

    /// <summary>Authors a public nongeneric interface; see the generic-arity overload for validation and ownership.</summary>
    public ImportedTypeReference CreateInterfaceReference(AssemblyIdentity dependency, AssemblyIdentity dependencyCoreLibrary,
        string artifactSha256, string @namespace, string name)
        => CreateInterfaceReference(dependency, dependencyCoreLibrary, artifactSha256, @namespace, name, 0);

    /// <summary>Authors a public top-level native interface identity without a reader.</summary>
    /// <param name="dependency">Exact unsigned external identity.</param>
    /// <param name="dependencyCoreLibrary">Matching explicit core identity.</param>
    /// <param name="artifactSha256">Selected native image SHA-256 in hexadecimal.</param>
    /// <param name="namespace">Metadata namespace.</param>
    /// <param name="name">Metadata name, including the arity suffix.</param>
    /// <param name="genericArity">Unconstrained invariant parameter count; zero for nongeneric.</param>
    /// <returns>An output-owned interface reference.</returns>
    /// <remarks>Identity, digest, validation errors and limits follow CreateTypeReference.
    /// The caller asserts interface classification; no dependency is read.</remarks>
    public ImportedTypeReference CreateInterfaceReference(AssemblyIdentity dependency, AssemblyIdentity dependencyCoreLibrary,
        string artifactSha256, string @namespace, string name, int genericArity)
    {
        var reference = CreateNominalReference(dependency, dependencyCoreLibrary, artifactSha256, @namespace, name, genericArity, true);
        authoredInterfaces.Add(reference);
        return reference;
    }

    /// <summary>Records a direct interface inheritance or class implementation edge from resolved semantic facts.</summary>
    /// <param name="source">Owned top-level reference class or interface definition.</param>
    /// <param name="target">Owned interface identity or construction created from CreateInterfaceReference.</param>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="ArgumentException">Foreign/unsupported endpoint, noninterface target or cyclic edge.</exception>
    /// <exception cref="InvalidDataException">More than 4096 direct conversion edges.</exception>
    /// <remarks>No definition lookup occurs. Transitive conversions are derived from these edges.
    /// The caller is responsible for truthful declarations; this does not synthesize runtime implementations.</remarks>
    public void AddInterfaceConversion(ImportedTypeReference source, ImportedTypeReference target)
    {
        ArgumentNullException.ThrowIfNull(source); ArgumentNullException.ThrowIfNull(target);
        if (!ReferenceEquals(source.Owner, this) || !ReferenceEquals(target.Owner, this) || source.TypeArguments.Count != 0 ||
            source.IsValueType || source.DeclaringType is not null || !IsAuthoredInterface(target) ||
            InterfaceDefinition(source).Equals(InterfaceDefinition(target)) || HasAuthoredInterfaceConversion(target, source, definitionsOnly: true))
            throw new ArgumentException("invalid or cyclic interface conversion");
        ((SignatureType)target).ValidateOwner(this, typeArity: source.GenericArity);
        if (!authoredInterfaceBases.TryGetValue(source, out var edges)) authoredInterfaceBases.Add(source, edges = []);
        if (edges.Contains(target)) return;
        if (authoredInterfaceEdgeCount >= 4096) throw new InvalidDataException("too many interface conversion edges");
        edges.Add(target); authoredInterfaceEdgeCount++;
        nativeInterfaceConversions.Clear();
    }

    private ImportedTypeReference InterfaceDefinition(ImportedTypeReference type) =>
        ImportTypeIdentity(type.AssemblyIdentity, type.Namespace, type.Name, type.GenericArity, type.IsValueType, type.DeclaringType);

    private bool IsAuthoredInterface(ImportedTypeReference type) => authoredInterfaces.Contains(InterfaceDefinition(type));

    private bool HasAuthoredInterfaceConversion(ImportedTypeReference source, ImportedTypeReference target, bool definitionsOnly = false)
    {
        var seen = new HashSet<ImportedTypeReference>();
        var pending = new Stack<ImportedTypeReference>(); pending.Push(source);
        while (pending.TryPop(out var current))
        {
            if (!seen.Add(current)) continue;
            if (current.Equals(target) || definitionsOnly && InterfaceDefinition(current).Equals(InterfaceDefinition(target))) return true;
            if (seen.Count > 4096) throw new InvalidDataException("interface conversion graph exceeds limit");
            if (authoredInterfaceBases.TryGetValue(InterfaceDefinition(current), out var bases))
                foreach (var parent in bases)
                {
                    SignatureType Substitute(SignatureType type) => type.TypeParameterIndex is { } index && current.TypeArguments.Count != 0
                        ? current.TypeArguments[index] : type.ArrayElement is { } element ? SignatureType.ArrayOf(Substitute(element))
                        : type.ImportedType is { } imported ? imported.Substitute(Substitute) : type;
                    pending.Push(definitionsOnly ? InterfaceDefinition(parent) : parent.Substitute(Substitute));
                }
        }
        return false;
    }
}
