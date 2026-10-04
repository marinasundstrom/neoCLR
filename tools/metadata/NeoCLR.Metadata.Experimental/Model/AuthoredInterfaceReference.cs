namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly HashSet<ImportedTypeReference> authoredInterfaces = [];
    private readonly Dictionary<ImportedTypeReference, ImportedMethodReference[]> completedInterfaceContracts = [];
    private int authoredInterfaceEdgeCount;

    /// <summary>Declares that all direct methods and base-interface conversions of an authored interface have been supplied.</summary>
    /// <param name="reference">An output-owned open interface definition created by CreateInterfaceReference.</param>
    /// <exception cref="ArgumentNullException">The reference is null.</exception>
    /// <exception cref="ArgumentException">The reference is foreign, constructed or not an authored interface.</exception>
    /// <remarks>This is a caller assertion, not dependency verification. Repeated completion is idempotent.
    /// Existing method references remain reusable; new methods and base edges are rejected after completion.
    /// An explicitly completed empty interface is valid. Inherited interfaces must also be completed before implementation validation.</remarks>
    public void CompleteInterfaceReference(ImportedTypeReference reference)
    {
        ArgumentNullException.ThrowIfNull(reference);
        if (!ReferenceEquals(reference.Owner, this) || reference.TypeArguments.Count != 0 || !IsAuthoredInterface(reference))
            throw new ArgumentException("completion requires an owned open interface reference", nameof(reference));
        if (completedInterfaceContracts.ContainsKey(reference)) return;
        completedInterfaceContracts.Add(reference, authoredCallableReferences.Where(m => Equals(m.DeclaringReference, reference)).ToArray());
    }

    internal IEnumerable<ImportedTypeReference> ExternalInterfaceBases(ImportedTypeReference reference)
    {
        var definition = InterfaceDefinition(reference);
        if (!completedInterfaceContracts.ContainsKey(definition))
            throw new InvalidDataException("external interface contract is incomplete: " + reference);
        if (!authoredInterfaceBases.TryGetValue(definition, out var bases)) return [];
        return bases.Select(b => b.Substitute(t => SubstituteExternalInterfaceType(t, reference)));
    }

    internal IEnumerable<(string Name, MethodSignature Signature, bool IsStatic, MethodBuilder Declaration)> ExternalInterfaceMethods(ImportedTypeReference reference)
    {
        if (!completedInterfaceContracts.TryGetValue(InterfaceDefinition(reference), out var methods))
            throw new InvalidDataException("external interface contract is incomplete: " + reference);
        return methods.Select(m => (m.Name, new MethodSignature(SubstituteExternalInterfaceType(m.Signature.ReturnType, reference),
            m.Signature.ParameterTypes.Select(t => SubstituteExternalInterfaceType(t, reference)), outParameters: m.Signature.OutParameters), m.IsStatic, m.Target));
    }

    private static SignatureType SubstituteExternalInterfaceType(SignatureType type, ImportedTypeReference owner) =>
        type.TypeParameterIndex is { } index && owner.TypeArguments.Count != 0 ? owner.TypeArguments[index]
        : type.ByReferenceElement is { } target ? SignatureType.ByReference(SubstituteExternalInterfaceType(target, owner))
        : type.ArrayElement is { } element ? SignatureType.ArrayOf(SubstituteExternalInterfaceType(element, owner))
        : type.ImportedType is { } imported ? imported.Substitute(t => SubstituteExternalInterfaceType(t, owner)) : type;

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

    /// <summary>Records a direct interface inheritance or class/value implementation edge from resolved semantic facts.</summary>
    /// <param name="source">Owned top-level class, value type or interface definition.</param>
    /// <param name="target">Owned interface identity or construction created from CreateInterfaceReference.</param>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="ArgumentException">Foreign/unsupported endpoint, noninterface target or cyclic edge.</exception>
    /// <exception cref="InvalidDataException">More than 4096 direct conversion edges.</exception>
    /// <exception cref="InvalidOperationException">A new edge is added after source interface completion.</exception>
    /// <remarks>No definition lookup occurs. Transitive conversions are derived from these edges.
    /// The caller is responsible for truthful declarations; this does not synthesize runtime implementations.</remarks>
    public void AddInterfaceConversion(ImportedTypeReference source, ImportedTypeReference target)
    {
        ArgumentNullException.ThrowIfNull(source); ArgumentNullException.ThrowIfNull(target);
        if (!ReferenceEquals(source.Owner, this) || !ReferenceEquals(target.Owner, this) || source.TypeArguments.Count != 0 ||
            source.DeclaringType is not null || !IsAuthoredInterface(target) ||
            InterfaceDefinition(source).Equals(InterfaceDefinition(target)) || HasAuthoredInterfaceConversion(target, source, definitionsOnly: true))
            throw new ArgumentException("invalid or cyclic interface conversion");
        ((SignatureType)target).ValidateOwner(this, typeArity: source.GenericArity, allowSelf: IsAuthoredInterface(source));
        authoredInterfaceBases.TryGetValue(source, out var edges);
        if (edges?.Contains(target) == true) return;
        if (completedInterfaceContracts.ContainsKey(source)) throw new InvalidOperationException("interface contract is complete");
        if (edges is null) authoredInterfaceBases.Add(source, edges = []);
        if (authoredInterfaceEdgeCount >= 4096) throw new InvalidDataException("too many interface conversion edges");
        edges.Add(target); authoredInterfaceEdgeCount++;
        nativeInterfaceConversions.Clear();
    }

    internal ImportedTypeReference FindAuthoredInterface(TypeReference reference)
    {
        if (!ReferenceEquals(reference.Module, Definition.MainModule) || reference.ExplicitScope is null)
            throw new ArgumentException("external relationship requires an output-owned scoped reference");
        return authoredInterfaces.SingleOrDefault(t => t.AssemblyIdentity.Equals(reference.ExplicitScope) &&
            t.Namespace == reference.Namespace && t.Name == reference.Name)
            ?? throw new ArgumentException("external relationship requires an authored interface contract");
    }

    private ImportedTypeReference InterfaceDefinition(ImportedTypeReference type) =>
        ImportTypeIdentity(type.AssemblyIdentity, type.Namespace, type.Name, type.GenericArity, type.IsValueType, type.DeclaringType);

    private bool IsAuthoredInterface(ImportedTypeReference type) => authoredInterfaces.Contains(InterfaceDefinition(type));

    internal bool SatisfiesConstrainedBound(SignatureType bound, ImportedTypeReference target, SignatureType implementing)
    {
        var pending = new Stack<SignatureType>(); pending.Push(bound);
        var seen = new HashSet<SignatureType>();
        while (pending.TryPop(out var current))
        {
            if (!seen.Add(current)) continue;
            if (seen.Count > 4096) throw new InvalidDataException("constraint interface traversal exceeds limit");
            if (Equals(current.ImportedType, target)) return true;
            IEnumerable<SignatureType> bases = current.ImportedType is { } external ? ExternalInterfaceBases(external).Select(b => (SignatureType)b)
                : current.ClassType is { } local ? local.InterfaceSignatures : [];
            foreach (var parent in bases) pending.Push(MethodILGenerator.SubstituteConstrainedSelf(parent, implementing));
        }
        return false;
    }

    internal bool SatisfiesInterfaceBound(SignatureType argument, ImportedTypeReference bound) =>
        argument.ImportedType is { } imported ? HasAuthoredInterfaceConversion(imported, bound)
        : argument.ClassType is { } local && local.InheritedContracts().Any(c => c.ImportedType is { } external && HasAuthoredInterfaceConversion(external, bound));

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
