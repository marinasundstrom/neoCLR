using Mono.Cecil;

// Experimental metadata-closure audit, not an IL verifier or production loader.
// Never use Cecil's default resolver: it searches outside the supplied artifact set.
static class ClosureAudit
{
    public static string[] Inspect(params string[] paths)
    {
        using var resolver = new SuppliedAssemblies();
        foreach (var path in paths) resolver.Add(path);
        // Resolving a member can make Cecil's TypeSystem synthesize a core AssemblyRef.
        // Snapshot every image's actual table first; lookup order must not change the audit.
        var assemblyReferences = resolver.Images.ToDictionary(a => a, a => a.MainModule.AssemblyReferences.ToArray());
        var errors = new SortedSet<string>(StringComparer.Ordinal);
        foreach (var assembly in resolver.Images)
        {
            var module = assembly.MainModule;
            if (assembly.Modules.Count != 1 || module.HasExportedTypes)
                errors.Add($"{assembly.Name.Name}: multiple modules/type forwarding unsupported");
            foreach (var reference in assemblyReferences[assembly])
                Check($"assembly {reference.FullName}", () => resolver.Resolve(reference));
            foreach (var reference in module.GetTypeReferences())
                Check($"type {reference.FullName} [{reference.Scope}]", () => reference.Resolve());
            foreach (var reference in module.GetMemberReferences())
                Check($"member {reference.FullName}", () => reference switch
                {
                    MethodReference method => (object?)method.Resolve(),
                    FieldReference field => field.Resolve(),
                    _ => null
                });

            void Check(string subject, Func<object?> resolve)
            {
                try
                {
                    if (resolve() is null) errors.Add($"{assembly.Name.Name}: unresolved {subject}");
                }
                catch (AssemblyResolutionException)
                {
                    errors.Add($"{assembly.Name.Name}: unresolved {subject}");
                }
            }
        }
        return errors.ToArray();
    }

    sealed class SuppliedAssemblies : IAssemblyResolver
    {
        readonly Dictionary<string, AssemblyDefinition> images = new(StringComparer.Ordinal);
        public IEnumerable<AssemblyDefinition> Images => images.Values;
        public void Add(string path)
        {
            var image = AssemblyDefinition.ReadAssembly(path, new ReaderParameters { AssemblyResolver = this });
            if (!images.TryAdd(image.Name.FullName, image))
            {
                image.Dispose();
                throw new InvalidDataException("Duplicate assembly identity in explicit input set.");
            }
        }
        public AssemblyDefinition Resolve(AssemblyNameReference name) => images.TryGetValue(name.FullName, out var image)
            ? image : throw new AssemblyResolutionException(name);
        public AssemblyDefinition Resolve(AssemblyNameReference name, ReaderParameters parameters) => Resolve(name);
        public void Dispose()
        {
            foreach (var image in images.Values) image.Dispose();
        }
    }
}
