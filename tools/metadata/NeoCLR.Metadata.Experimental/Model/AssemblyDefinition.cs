using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned read-only assembly view, separate from runtime reflection and mutable emission.</summary>
public sealed class AssemblyDefinition
{
    private AssemblyDefinition(string name, Version version, string moduleName, Guid mvid,
        IReadOnlyList<TypeRow> rows, MetadataProfileDocument? profile)
    {
        Name = name; Version = version; Profile = profile;
        MainModule = new ModuleDefinition(this, moduleName, mvid, rows);
    }
    /// <summary>Gets the assembly's simple name, not a complete binding identity.</summary>
    public string Name { get; }
    /// <summary>Gets the declared assembly version.</summary>
    public Version Version { get; }
    /// <summary>Gets the manifest module; other modules are not loaded.</summary>
    public ModuleDefinition MainModule { get; }
    /// <summary>Gets locally validated extended metadata, or null for explicitly admitted ordinary input.</summary>
    public MetadataProfileDocument? Profile { get; }

    /// <summary>Recognizes bounded PE input and snapshots its assembly/module/TypeDef declarations.</summary>
    /// <param name="image">Complete image; do not mutate during the call.</param>
    /// <param name="expectedExtended">Require the neoCLR marker/profile by default; false admits ordinary CLI input.</param>
    /// <returns>An owned snapshot with no open stream or loaded runtime assembly.</returns>
    /// <exception cref="InvalidDataException">Invalid/unsupported artifact, missing assembly metadata, malformed declarations more than 4096 TypeDefs, or excessive decoded names.</exception>
    /// <remarks>Does not bind dependencies, interpret physical TypeRef/TypeSpec rows, decode member signatures or emit assemblies.</remarks>
    public static AssemblyDefinition ReadAssembly(ReadOnlySpan<byte> image, bool expectedExtended = true)
    {
        if (image.Length > MetadataArtifactReader.MaxImageSize) throw new InvalidDataException("image exceeds limit");
        var owned = image.ToArray();
        var artifact = MetadataArtifactReader.Read(owned, expectedExtended);
        try
        {
            using var stream = new MemoryStream(owned, writable: false);
            using var pe = new PEReader(stream);
            var reader = pe.GetMetadataReader();
            if (!reader.IsAssembly) throw new InvalidDataException("assembly manifest required");
            if (reader.TypeDefinitions.Count > 4096) throw new InvalidDataException("too many type definitions");
            int nameCharacters = 0;
            string ReadName(StringHandle handle)
            {
                var value = reader.GetString(handle);
                if (value.Length > MetadataArtifactReader.MaxImageSize - nameCharacters)
                    throw new InvalidDataException("decoded declaration names exceed limit");
                nameCharacters += value.Length;
                return value;
            }
            var assembly = reader.GetAssemblyDefinition();
            var module = reader.GetModuleDefinition();
            var rows = new List<TypeRow>();
            foreach (var handle in reader.TypeDefinitions)
            {
                var type = reader.GetTypeDefinition(handle);
                var declaring = type.GetDeclaringType();
                rows.Add(new((uint)MetadataTokens.GetToken(handle), ReadName(type.Namespace), ReadName(type.Name),
                    type.GetGenericParameters().Count, declaring.IsNil ? 0 : (uint)MetadataTokens.GetToken(declaring)));
            }
            var parents = rows.ToDictionary(row => row.Token, row => row.DeclaringToken);
            foreach (var row in rows)
            {
                var seen = new HashSet<uint>();
                uint token = row.Token;
                while (token != 0)
                {
                    if (!seen.Add(token) || !parents.TryGetValue(token, out token)) throw new InvalidDataException("invalid or cyclic declaring type");
                }
            }
            return new(ReadName(assembly.Name), assembly.Version, ReadName(module.Name),
                reader.GetGuid(module.Mvid), rows, artifact.Profile);
        }
        catch (BadImageFormatException error)
        {
            throw new InvalidDataException("malformed CLI declaration metadata", error);
        }
    }
    internal sealed record TypeRow(uint Token, string Namespace, string Name, int Arity, uint DeclaringToken);
}

/// <summary>An owned manifest-module definition with local TypeDef lookup.</summary>
public sealed class ModuleDefinition
{
    private readonly Dictionary<uint, TypeDefinition> definitions;
    internal ModuleDefinition(AssemblyDefinition assembly, string name, Guid mvid, IReadOnlyList<AssemblyDefinition.TypeRow> rows)
    {
        Assembly = assembly; Name = name; Mvid = mvid;
        var types = rows.Select(row => new TypeDefinition(this, row)).ToArray();
        Types = Array.AsReadOnly(types);
        definitions = types.ToDictionary(type => type.MetadataToken);
    }
    /// <summary>Gets the owning assembly snapshot.</summary>
    public AssemblyDefinition Assembly { get; }
    /// <summary>Gets the declared module name.</summary>
    public string Name { get; }
    /// <summary>Gets the declared MVID; this is not the experimental catalog's module scope UUID.</summary>
    public Guid Mvid { get; }
    /// <summary>Gets all TypeDefs in metadata row order, including nested types and the module pseudo-type.</summary>
    public IReadOnlyList<TypeDefinition> Types { get; }
    /// <summary>Looks up a module-local TypeDef without loading any dependencies.</summary>
    /// <param name="metadataToken">A physical TypeDef token; other kinds and absent rows return null.</param>
    /// <returns>The owned definition, or null if this snapshot has no such TypeDef.</returns>
    public TypeDefinition? GetTypeDefinition(uint metadataToken) => definitions.GetValueOrDefault(metadataToken);
}

/// <summary>A nominal declaration read from a physical TypeDef row.</summary>
public sealed class TypeDefinition
{
    private readonly uint declaringToken;
    internal TypeDefinition(ModuleDefinition module, AssemblyDefinition.TypeRow row)
    {
        Module = module; MetadataToken = row.Token; Namespace = row.Namespace;
        Name = row.Name; GenericArity = row.Arity; declaringToken = row.DeclaringToken;
    }
    /// <summary>Gets the owning module snapshot.</summary>
    public ModuleDefinition Module { get; }
    /// <summary>Gets this image's TypeDef token; tokens are not cross-module identity.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the stored namespace, without constructing a display identity.</summary>
    public string Namespace { get; }
    /// <summary>Gets the stored metadata name, including any generic arity suffix.</summary>
    public string Name { get; }
    /// <summary>Gets the number of declared GenericParam rows, including captured outer parameters where encoded.</summary>
    public int GenericArity { get; }
    /// <summary>Gets the enclosing definition, or null for a top-level type.</summary>
    public TypeDefinition? DeclaringType => Module.GetTypeDefinition(declaringToken);
    /// <summary>Creates a nominal reference scoped to this module snapshot.</summary>
    /// <returns>A reference that resolves to this exact owned definition.</returns>
    public TypeReference ToReference() => new(this);
}

/// <summary>A definition-backed nominal reference scoped to one module snapshot.</summary>
/// <remarks>Not yet a physical CLI TypeRef/TypeSpec or cross-module imported reference. Object equality is reference equality.</remarks>
public sealed class TypeReference
{
    internal TypeReference(TypeDefinition definition) { Module = definition.Module; MetadataToken = definition.MetadataToken; }
    /// <summary>Gets the target module snapshot; no implicit dependency search occurs.</summary>
    public ModuleDefinition Module { get; }
    /// <summary>Gets the target TypeDef token within Module.</summary>
    public uint MetadataToken { get; }
    /// <summary>Resolves within the immutable target module.</summary>
    /// <returns>The original TypeDefinition object.</returns>
    public TypeDefinition Resolve() => Module.GetTypeDefinition(MetadataToken)!;
}
