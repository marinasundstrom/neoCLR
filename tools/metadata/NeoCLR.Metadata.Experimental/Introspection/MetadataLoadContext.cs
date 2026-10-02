using NeoCLR.Metadata.Experimental.Model;

namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>A fixed catalog of loaded metadata snapshots and context-owned introspection views.</summary>
/// <remarks>No filesystem probing, runtime loading or execution occurs. Views retain their snapshots.
/// Identity is scoped to this context; no global cache or disposal of shared snapshots is involved.</remarks>
public sealed class MetadataLoadContext
{
    private readonly Dictionary<AssemblyIdentity, AssemblyInfo> assemblies = [];
    private readonly Dictionary<TypeDefinition, NominalTypeInfo> types = [];
    private readonly Dictionary<TypeReference, NominalTypeInfo> references = [];
    private readonly object gate = new();
    private readonly IAssemblyResolver resolver;

    /// <summary>Copies a catalog of immutable CLI or native reader snapshots.</summary>
    /// <param name="snapshots">Already-read assemblies; repeated registration of the same object is idempotent.</param>
    /// <exception cref="ArgumentNullException">The catalog or an element is null.</exception>
    /// <exception cref="ArgumentException">An assembly is mutable or more than 4096 inputs are supplied.</exception>
    /// <exception cref="InvalidDataException">Different snapshots claim the same exact identity.</exception>
    public MetadataLoadContext(IEnumerable<AssemblyDefinition> snapshots)
    {
        ArgumentNullException.ThrowIfNull(snapshots);
        int count = 0;
        foreach (var snapshot in snapshots)
        {
            ArgumentNullException.ThrowIfNull(snapshot);
            if (++count > 4096) throw new ArgumentException("metadata catalog input limit exceeded", nameof(snapshots));
            if (snapshot.Producer is not null) throw new ArgumentException("metadata context requires immutable reader snapshots", nameof(snapshots));
            if (assemblies.TryGetValue(snapshot.Identity, out var existing))
            {
                if (!ReferenceEquals(existing.Definition, snapshot))
                    throw new InvalidDataException("conflicting metadata snapshots: " + snapshot.Identity.Name + ", " + snapshot.Identity.Version);
                continue;
            }
            assemblies.Add(snapshot.Identity, new AssemblyInfo(this, snapshot));
        }
        Assemblies = Array.AsReadOnly(assemblies.Values.ToArray());
        resolver = new CatalogResolver(assemblies);
    }

    /// <summary>Gets registered assembly views in input order, without expanding dependencies.</summary>
    public IReadOnlyList<AssemblyInfo> Assemblies { get; }

    /// <summary>Finds a registered assembly by full identity.</summary>
    /// <exception cref="ArgumentNullException">Identity is null.</exception>
    /// <exception cref="InvalidDataException">No exact identity is registered; there is no version fallback.</exception>
    public AssemblyInfo Resolve(AssemblyIdentity identity)
    {
        ArgumentNullException.ThrowIfNull(identity);
        return assemblies.GetValueOrDefault(identity) ?? throw new InvalidDataException("metadata dependency not registered: " + identity.Name + ", " + identity.Version);
    }

    /// <summary>Resolves a dependency from a registered consuming snapshot.</summary>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="InvalidDataException">The consuming snapshot or exact dependency is not registered.</exception>
    public AssemblyInfo Resolve(AssemblyReference reference)
    {
        ArgumentNullException.ThrowIfNull(reference);
        RequireSnapshot(reference.Module.Assembly);
        return Resolve(reference.Identity);
    }

    /// <summary>Resolves a local or external nominal reference to its canonical context-owned view.</summary>
    /// <exception cref="ArgumentNullException">Reference is null.</exception>
    /// <exception cref="InvalidDataException">A snapshot/dependency is missing, or the existing reader cannot resolve the scope/type uniquely.</exception>
    /// <remarks>Generic definition identity is supported; constructed type/member views are a later increment.
    /// Exported forwarders and multi-module scopes retain the reader's explicit unsupported behavior.</remarks>
    public NominalTypeInfo Resolve(TypeReference reference)
    {
        ArgumentNullException.ThrowIfNull(reference);
        RequireSnapshot(reference.Module.Assembly);
        lock (gate)
        {
            if (!references.TryGetValue(reference, out var type)) references.Add(reference, type = GetType(reference.Resolve(resolver)));
            return type;
        }
    }

    internal AssemblyInfo RequireSnapshot(AssemblyDefinition definition)
    {
        var assembly = Resolve(definition.Identity);
        if (!ReferenceEquals(assembly.Definition, definition)) throw new InvalidDataException("reference belongs to a different metadata snapshot: " + definition.Name);
        return assembly;
    }

    internal NominalTypeInfo GetType(TypeDefinition definition)
    {
        RequireSnapshot(definition.Module.Assembly);
        lock (gate)
        {
            if (!types.TryGetValue(definition, out var type)) types.Add(definition, type = new NominalTypeInfo(this, definition));
            return type;
        }
    }

    private sealed class CatalogResolver(Dictionary<AssemblyIdentity, AssemblyInfo> catalog) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => catalog.GetValueOrDefault(identity)?.Definition;
    }
}
