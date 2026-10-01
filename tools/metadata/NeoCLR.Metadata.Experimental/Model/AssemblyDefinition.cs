using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned read-only assembly view, separate from runtime reflection and mutable emission.</summary>
public sealed class AssemblyDefinition
{
    private readonly byte[] image;
    private AssemblyDefinition(AssemblyIdentity identity, string moduleName, Guid mvid,
        IReadOnlyList<TypeRow> rows, IReadOnlyList<FieldRow> fields, IReadOnlyList<PropertyRow> properties, IReadOnlyList<MethodRow> methods, IReadOnlyList<MemberReferenceRow> memberReferences, IReadOnlyList<ReferenceRow> references, IReadOnlyList<TypeReferenceRow> typeReferences, MetadataProfileDocument? profile, byte[] image, uint entryPointToken)
    {
        Identity = identity; Profile = profile; this.image = image; EntryPointToken = entryPointToken;
        MainModule = new ModuleDefinition(this, moduleName, mvid, rows, fields, properties, methods, memberReferences, references, typeReferences);
    }
    /// <summary>Gets the assembly's simple name, not a complete binding identity.</summary>
    public string Name => Identity.Name;
    /// <summary>Gets the declared assembly version.</summary>
    public Version Version => Identity.Version;
    /// <summary>Gets the exact metadata identity used for explicit dependency matching.</summary>
    public AssemblyIdentity Identity { get; }
    /// <summary>Gets the manifest module; other modules are not loaded.</summary>
    public ModuleDefinition MainModule { get; }
    /// <summary>Gets locally validated extended metadata, or null for explicitly admitted ordinary input.</summary>
    public MetadataProfileDocument? Profile { get; }

    /// <summary>Gets the managed MethodDef entry token, or zero for a library.</summary>
    public uint EntryPointToken { get; }
    /// <summary>Gets the owned entry-point definition, or null for a library.</summary>
    public MethodDefinition? EntryPoint => MainModule.GetMethodDefinition(EntryPointToken);
    /// <summary>Returns a new byte-for-byte copy of this unchanged read snapshot.</summary>
    /// <returns>Owned original PE bytes, preserving all validated and opaque data.</returns>
    /// <remarks>This is preservation, not rebuilding or applying edits. Use AssemblyBuilder for controlled new output.</remarks>
    public byte[] Write() => (byte[])image.Clone();

    /// <summary>Recognizes bounded PE input and snapshots its assembly, type and callable declarations.</summary>
    /// <param name="image">Complete image; do not mutate during the call.</param>
    /// <param name="expectedExtended">Require the neoCLR marker/profile by default; false admits ordinary CLI input.</param>
    /// <returns>An owned snapshot with no open stream or loaded runtime assembly.</returns>
    /// <exception cref="InvalidDataException">Invalid/unsupported artifact, missing assembly metadata, malformed declarations/identities, exceeded row limits or excessive decoded names/key data.</exception>
    /// <remarks>Reads physical nominal references and callable signature blobs without automatically resolving dependencies. General signature decoding, bodies and TypeSpec interpretation remain separate.</remarks>
    public static AssemblyDefinition ReadAssembly(ReadOnlySpan<byte> image, bool expectedExtended = true)
        => ReadCore(image, expectedExtended, runtimeProjection: false);

    internal static AssemblyDefinition ReadRuntimeProjection(ReadOnlySpan<byte> image)
        => ReadCore(image, expectedExtended: true, runtimeProjection: true);

    private static AssemblyDefinition ReadCore(ReadOnlySpan<byte> image, bool expectedExtended, bool runtimeProjection)
    {
        if (image.Length > MetadataArtifactReader.MaxImageSize) throw new InvalidDataException("image exceeds limit");
        var owned = image.ToArray();
        var artifact = runtimeProjection ? null : MetadataArtifactReader.Read(owned, expectedExtended);
        try
        {
            using var stream = new MemoryStream(owned, writable: false);
            using var pe = new PEReader(stream);
            var reader = pe.GetMetadataReader();
            if (!reader.IsAssembly) throw new InvalidDataException("assembly manifest required");
            if (reader.TypeDefinitions.Count > 4096) throw new InvalidDataException("too many type definitions");
            if (reader.MemberReferences.Count > 4096) throw new InvalidDataException("too many member references");
            if (reader.PropertyDefinitions.Count > 4096) throw new InvalidDataException("too many property definitions");
            if (reader.GetTableRowCount(TableIndex.MethodSemantics) > 16384) throw new InvalidDataException("too many method semantics");
            if (reader.FieldDefinitions.Count > 4096) throw new InvalidDataException("too many field definitions");
            if (reader.MethodDefinitions.Count > 4096) throw new InvalidDataException("too many method definitions");
            if (reader.TypeReferences.Count > 4096) throw new InvalidDataException("too many type references");
            if (reader.AssemblyReferences.Count > 256) throw new InvalidDataException("too many assembly references");
            int keyBytes = 0;
            int nameCharacters = 0;
            string ReadName(StringHandle handle)
            {
                var value = reader.GetString(handle);
                if (value.Length > MetadataArtifactReader.MaxImageSize - nameCharacters)
                    throw new InvalidDataException("decoded declaration names exceed limit");
                nameCharacters += value.Length;
                return value;
            }
            AssemblyIdentity ReadIdentity(StringHandle name, Version version, StringHandle culture, BlobHandle key, uint flags, bool definition)
            {
                int length = key.IsNil ? 0 : reader.GetBlobReader(key).Length;
                if (length > MetadataArtifactReader.MaxImageSize - keyBytes) throw new InvalidDataException("assembly key data exceeds limit");
                keyBytes += length;
                bool fullKey = (flags & 1) != 0;
                if ((fullKey && length == 0) || (!fullKey && (definition ? length != 0 : length != 0 && length != 8)))
                    throw new InvalidDataException("invalid assembly key/token representation");
                var bytes = reader.GetBlobBytes(key);
                if (fullKey) bytes = SHA1.HashData(bytes)[^8..].Reverse().ToArray();
                try { return new(ReadName(name), version, ReadName(culture), Convert.ToHexString(bytes), flags & ~1u); }
                catch (ArgumentException error) { throw new InvalidDataException("invalid assembly identity", error); }
            }
            var assembly = reader.GetAssemblyDefinition();
            var module = reader.GetModuleDefinition();
            uint entryPointToken = (uint)pe.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress;
            if (entryPointToken != 0 && (entryPointToken >> 24 != 6 || (entryPointToken & 0xffffff) == 0 ||
                (entryPointToken & 0xffffff) > reader.MethodDefinitions.Count)) throw new InvalidDataException("invalid managed entry token");
            var identity = ReadIdentity(assembly.Name, assembly.Version, assembly.Culture, assembly.PublicKey, (uint)assembly.Flags, true);
            var references = new List<ReferenceRow>();
            foreach (var handle in reader.AssemblyReferences)
            {
                var reference = reader.GetAssemblyReference(handle);
                references.Add(new((uint)MetadataTokens.GetToken(handle), ReadIdentity(reference.Name, reference.Version,
                    reference.Culture, reference.PublicKeyOrToken, (uint)reference.Flags, false)));
            }
            var typeReferences = new List<TypeReferenceRow>();
            foreach (var handle in reader.TypeReferences)
            {
                var reference = reader.GetTypeReference(handle);
                typeReferences.Add(new((uint)MetadataTokens.GetToken(handle), ReadName(reference.Namespace), ReadName(reference.Name),
                    reference.ResolutionScope.IsNil ? 0 : (uint)MetadataTokens.GetToken(reference.ResolutionScope)));
            }
            var referenceScopes = typeReferences.ToDictionary(row => row.Token, row => row.Scope);
            foreach (var row in typeReferences)
            {
                var seen = new HashSet<uint>();
                uint scope = row.Token;
                while (scope >> 24 == 1)
                {
                    if (seen.Count >= 32 || !seen.Add(scope) || !referenceScopes.TryGetValue(scope, out scope))
                        throw new InvalidDataException("invalid, cyclic or excessive nested TypeRef scope");
                }
                if (scope >> 24 == 0x23 && !references.Any(reference => reference.Token == scope))
                    throw new InvalidDataException("TypeRef assembly scope outside table");
            }
            var rows = new List<TypeRow>();
            foreach (var handle in reader.TypeDefinitions)
            {
                var type = reader.GetTypeDefinition(handle);
                var declaring = type.GetDeclaringType();
                var baseReference = type.BaseType.Kind == HandleKind.TypeReference ? reader.GetTypeReference((TypeReferenceHandle)type.BaseType) : default;
                var baseDefinition = type.BaseType.Kind == HandleKind.TypeDefinition ? reader.GetTypeDefinition((TypeDefinitionHandle)type.BaseType) : default;
                bool valueType = !type.BaseType.IsNil && (type.BaseType.Kind == HandleKind.TypeReference
                    ? reader.GetString(baseReference.Namespace) == "System" && reader.GetString(baseReference.Name) is "ValueType" or "Enum"
                    : type.BaseType.Kind == HandleKind.TypeDefinition && reader.GetString(baseDefinition.Namespace) == "System" && reader.GetString(baseDefinition.Name) is "ValueType" or "Enum");
                bool unsupportedParameters = type.GetGenericParameters().Select(reader.GetGenericParameter).Where((p, i) => p.Index != i || p.Attributes != 0 || p.GetConstraints().Count != 0).Any();
                rows.Add(new((uint)MetadataTokens.GetToken(handle), ReadName(type.Namespace), ReadName(type.Name),
                    type.GetGenericParameters().Count, declaring.IsNil ? 0 : (uint)MetadataTokens.GetToken(declaring), (uint)type.Attributes, !valueType && !unsupportedParameters && ((uint)type.Attributes & 0x180) != 0x180));
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
            int signatureBytes = 0;
            var methods = new List<MethodRow>();
            var typeRows = rows.ToDictionary(row => row.Token);
            var fields = new List<FieldRow>();
            foreach (var handle in reader.FieldDefinitions)
            {
                var field = reader.GetFieldDefinition(handle);
                uint owner = (uint)MetadataTokens.GetToken(field.GetDeclaringType());
                if (!typeRows.ContainsKey(owner)) throw new InvalidDataException("field has no declaring definition");
                int length = field.Signature.IsNil ? 0 : reader.GetBlobReader(field.Signature).Length;
                if (length == 0 || length > MetadataArtifactReader.MaxImageSize - signatureBytes)
                    throw new InvalidDataException("missing or excessive field signature data");
                signatureBytes += length;
                fields.Add(new((uint)MetadataTokens.GetToken(handle), owner, ReadName(field.Name), (ushort)field.Attributes, reader.GetBlobBytes(field.Signature)));
            }
            foreach (var handle in reader.MethodDefinitions)
            {
                var method = reader.GetMethodDefinition(handle);
                uint declaring = (uint)MetadataTokens.GetToken(method.GetDeclaringType());
                if (!typeRows.TryGetValue(declaring, out var owner)) throw new InvalidDataException("method has no declaring definition");
                bool global = owner.Token == 0x02000001 && owner.Name == "<Module>" && owner.Namespace.Length == 0 && owner.DeclaringToken == 0;
                if (global && (method.Attributes & System.Reflection.MethodAttributes.Static) == 0)
                    throw new InvalidDataException("global function must be static");
                int length = method.Signature.IsNil ? 0 : reader.GetBlobReader(method.Signature).Length;
                if (length == 0 || length > MetadataArtifactReader.MaxImageSize - signatureBytes)
                    throw new InvalidDataException("missing or excessive method signature data");
                signatureBytes += length;
                methods.Add(new((uint)MetadataTokens.GetToken(handle), global ? 0 : declaring, ReadName(method.Name),
                    (ushort)method.Attributes, (ushort)method.ImplAttributes, method.GetGenericParameters().Count,
                    reader.GetBlobBytes(method.Signature),
                    method.GetGenericParameters().Select(reader.GetGenericParameter).Where((p, i) => p.Index != i || p.Attributes != 0 || p.GetConstraints().Count != 0).Any()));
            }
            var properties = new List<PropertyRow>();
            var methodRows = methods.ToDictionary(m => m.Token);
            var propertyOwners = new Dictionary<PropertyDefinitionHandle, uint>();
            foreach (var typeHandle in reader.TypeDefinitions)
                foreach (var propertyHandle in reader.GetTypeDefinition(typeHandle).GetProperties())
                    if (!propertyOwners.TryAdd(propertyHandle, (uint)MetadataTokens.GetToken(typeHandle)))
                        throw new InvalidDataException("property has multiple declaring types");
            foreach (var handle in reader.PropertyDefinitions)
            {
                if (!propertyOwners.TryGetValue(handle, out uint owner)) throw new InvalidDataException("property has no declaring type");
                var property = reader.GetPropertyDefinition(handle);
                int length = property.Signature.IsNil ? 0 : reader.GetBlobReader(property.Signature).Length;
                if (length == 0 || length > MetadataArtifactReader.MaxImageSize - signatureBytes)
                    throw new InvalidDataException("missing or excessive property signature data");
                signatureBytes += length;
                uint Accessor(MethodDefinitionHandle accessor)
                {
                    if (accessor.IsNil) return 0;
                    uint token = (uint)MetadataTokens.GetToken(accessor);
                    if (!methodRows.TryGetValue(token, out var method) || method.DeclaringToken != owner)
                        throw new InvalidDataException("property accessor outside declaring type");
                    return token;
                }
                var accessors = property.GetAccessors();
                properties.Add(new((uint)MetadataTokens.GetToken(handle), owner, ReadName(property.Name), (ushort)property.Attributes,
                    reader.GetBlobBytes(property.Signature), Accessor(accessors.Getter), Accessor(accessors.Setter), accessors.Others.Select(Accessor).ToArray()));
            }
            var memberReferences = new List<MemberReferenceRow>();
            foreach (var handle in reader.MemberReferences)
            {
                var member = reader.GetMemberReference(handle);
                uint parent = (uint)MetadataTokens.GetToken(member.Parent);
                int table = (int)(parent >> 24), row = (int)(parent & 0xffffff);
                if (table is not (0x01 or 0x02 or 0x06 or 0x1a or 0x1b) || row == 0 || row > reader.GetTableRowCount((TableIndex)table))
                    throw new InvalidDataException("member reference parent outside supported metadata tables");
                int length = member.Signature.IsNil ? 0 : reader.GetBlobReader(member.Signature).Length;
                if (length == 0 || length > MetadataArtifactReader.MaxImageSize - signatureBytes)
                    throw new InvalidDataException("missing or excessive member signature data");
                signatureBytes += length;
                memberReferences.Add(new((uint)MetadataTokens.GetToken(handle), parent, ReadName(member.Name), reader.GetBlobBytes(member.Signature)));
            }
            return new(identity, ReadName(module.Name), reader.GetGuid(module.Mvid), rows, fields, properties, methods, memberReferences, references, typeReferences, artifact?.Profile, owned, entryPointToken);
        }
        catch (BadImageFormatException error)
        {
            throw new InvalidDataException("malformed CLI declaration metadata", error);
        }
    }
    internal sealed record MemberReferenceRow(uint Token, uint ParentToken, string Name, byte[] Signature);
    internal sealed record PropertyRow(uint Token, uint DeclaringToken, string Name, ushort Attributes, byte[] Signature, uint Getter, uint Setter, uint[] Others);
    internal sealed record FieldRow(uint Token, uint DeclaringToken, string Name, ushort Attributes, byte[] Signature);
    internal sealed record MethodRow(uint Token, uint DeclaringToken, string Name, ushort Attributes, ushort ImplementationAttributes, int Arity, byte[] Signature, bool UnsupportedGenericParameters);
    internal sealed record TypeReferenceRow(uint Token, string Namespace, string Name, uint Scope);
    internal sealed record ReferenceRow(uint Token, AssemblyIdentity Identity);
    internal sealed record TypeRow(uint Token, string Namespace, string Name, int Arity, uint DeclaringToken, uint Attributes, bool CanImportReference);
}

/// <summary>An owned manifest-module definition with local TypeDef lookup.</summary>
public sealed class ModuleDefinition
{
    private readonly Dictionary<uint, TypeDefinition> definitions;
    private readonly Dictionary<uint, MethodDefinition> methods;
    private readonly Dictionary<uint, PropertyDefinition> properties;
    private readonly Dictionary<uint, IReadOnlyList<PropertyDefinition>> declaredProperties;
    private readonly Dictionary<uint, FieldDefinition> fields;
    private readonly Dictionary<uint, IReadOnlyList<FieldDefinition>> declaredFields;
    private readonly Dictionary<uint, IReadOnlyList<MethodDefinition>> declaredMethods;
    internal ModuleDefinition(AssemblyDefinition assembly, string name, Guid mvid, IReadOnlyList<AssemblyDefinition.TypeRow> rows,
        IReadOnlyList<AssemblyDefinition.FieldRow> fieldRows, IReadOnlyList<AssemblyDefinition.PropertyRow> propertyRows, IReadOnlyList<AssemblyDefinition.MethodRow> methodRows, IReadOnlyList<AssemblyDefinition.MemberReferenceRow> memberReferenceRows, IReadOnlyList<AssemblyDefinition.ReferenceRow> references, IReadOnlyList<AssemblyDefinition.TypeReferenceRow> typeReferences)
    {
        Assembly = assembly; Name = name; Mvid = mvid;
        var types = rows.Select(row => new TypeDefinition(this, row)).ToArray();
        Types = Array.AsReadOnly(types);
        definitions = types.ToDictionary(type => type.MetadataToken);
        var storage = fieldRows.Select(row => new FieldDefinition(this, row)).ToArray();
        Fields = Array.AsReadOnly(storage);
        fields = storage.ToDictionary(field => field.MetadataToken);
        declaredFields = storage.GroupBy(field => field.DeclaringType.MetadataToken)
            .ToDictionary(group => group.Key, group => (IReadOnlyList<FieldDefinition>)Array.AsReadOnly(group.ToArray()));
        var callables = methodRows.Select(row => new MethodDefinition(this, row)).ToArray();
        Methods = Array.AsReadOnly(callables);
        methods = callables.ToDictionary(method => method.MetadataToken);
        Functions = Array.AsReadOnly(callables.Where(method => method.DeclaringType is null).ToArray());
        declaredMethods = callables.Where(method => method.DeclaringType is not null)
            .GroupBy(method => method.DeclaringType!.MetadataToken)
            .ToDictionary(group => group.Key, group => (IReadOnlyList<MethodDefinition>)Array.AsReadOnly(group.ToArray()));
        var propertyDefinitions = propertyRows.Select(row => new PropertyDefinition(this, row)).ToArray();
        Properties = Array.AsReadOnly(propertyDefinitions);
        properties = propertyDefinitions.ToDictionary(property => property.MetadataToken);
        declaredProperties = propertyDefinitions.GroupBy(property => property.DeclaringType.MetadataToken)
            .ToDictionary(group => group.Key, group => (IReadOnlyList<PropertyDefinition>)Array.AsReadOnly(group.ToArray()));
        AssemblyReferences = Array.AsReadOnly(references.Select(row => new AssemblyReference(this, row.Token, row.Identity)).ToArray());
        TypeReferences = Array.AsReadOnly(typeReferences.Select(row => new TypeReference(this, row)).ToArray());
        MemberReferences = Array.AsReadOnly(memberReferenceRows.Select(row => new MemberReference(this, row)).ToArray());
    }
    /// <summary>Gets the owning assembly snapshot.</summary>
    public AssemblyDefinition Assembly { get; }
    /// <summary>Gets the declared module name.</summary>
    public string Name { get; }
    /// <summary>Gets the declared MVID; this is not the experimental catalog's module scope UUID.</summary>
    public Guid Mvid { get; }
    /// <summary>Gets all TypeDefs in metadata row order, including nested types and the module pseudo-type.</summary>
    public IReadOnlyList<TypeDefinition> Types { get; }
    /// <summary>Gets physical Property rows in metadata order.</summary>
    public IReadOnlyList<PropertyDefinition> Properties { get; }
    /// <summary>Looks up a property in this snapshot.</summary>
    /// <param name="metadataToken">Property token; wrong-kind or missing rows return null.</param>
    /// <returns>The owned property or null.</returns>
    public PropertyDefinition? GetPropertyDefinition(uint metadataToken) => properties.GetValueOrDefault(metadataToken);
    internal IReadOnlyList<PropertyDefinition> GetDeclaredProperties(uint token) => declaredProperties.GetValueOrDefault(token) ?? Array.Empty<PropertyDefinition>();
    /// <summary>Gets physical Field rows in metadata order.</summary>
    public IReadOnlyList<FieldDefinition> Fields { get; }
    /// <summary>Looks up a field in this snapshot.</summary>
    /// <param name="metadataToken">Field token; other kinds or missing rows return null.</param>
    /// <returns>The owned field or null.</returns>
    public FieldDefinition? GetFieldDefinition(uint metadataToken) => fields.GetValueOrDefault(metadataToken);
    internal IReadOnlyList<FieldDefinition> GetDeclaredFields(uint token) => declaredFields.GetValueOrDefault(token) ?? Array.Empty<FieldDefinition>();
    /// <summary>Gets all callable definitions in physical MethodDef order, including global functions.</summary>
    public IReadOnlyList<MethodDefinition> Methods { get; }
    /// <summary>Gets top-level functions with no declaring type, in metadata order.</summary>
    public IReadOnlyList<MethodDefinition> Functions { get; }
    /// <summary>Looks up an owned MethodDef token; other kinds or absent rows return null.</summary>
    /// <param name="metadataToken">Physical MethodDef token in this snapshot.</param>
    /// <returns>The owned callable or null.</returns>
    public MethodDefinition? GetMethodDefinition(uint metadataToken) => methods.GetValueOrDefault(metadataToken);
    internal IReadOnlyList<MethodDefinition> GetDeclaredMethods(uint token) => declaredMethods.GetValueOrDefault(token) ?? Array.Empty<MethodDefinition>();
    /// <summary>Gets physical MemberRef rows, including opaque field or unsupported method references.</summary>
    public IReadOnlyList<MemberReference> MemberReferences { get; }
    /// <summary>Looks up a physical MemberRef token in this snapshot.</summary>
    /// <param name="metadataToken">Physical MemberRef token; other kinds and missing rows return null.</param>
    /// <returns>The owned reference or null.</returns>
    public MemberReference? GetMemberReference(uint metadataToken) => MemberReferences.FirstOrDefault(member => member.MetadataToken == metadataToken);
    /// <summary>Gets physical AssemblyRef rows in metadata order, without resolving dependencies.</summary>
    public IReadOnlyList<AssemblyReference> AssemblyReferences { get; }
    /// <summary>Gets physical nominal TypeRef rows in metadata order.</summary>
    public IReadOnlyList<TypeReference> TypeReferences { get; }
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
        Name = row.Name; GenericArity = row.Arity; CanImportReference = row.CanImportReference; declaringToken = row.DeclaringToken; Attributes = row.Attributes;
    }
    internal bool CanImportReference { get; }
    /// <summary>Gets the physical TypeAttributes flags.</summary>
    public uint Attributes { get; }
    /// <summary>Gets properties declared directly by this type.</summary>
    public IReadOnlyList<PropertyDefinition> Properties => Module.GetDeclaredProperties(MetadataToken);
    /// <summary>Gets fields declared directly by this type.</summary>
    public IReadOnlyList<FieldDefinition> Fields => Module.GetDeclaredFields(MetadataToken);
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
    /// <summary>Gets methods declared directly by this type; global functions belong to Module.Functions.</summary>
    public IReadOnlyList<MethodDefinition> Methods => Module.GetDeclaredMethods(MetadataToken);
    /// <summary>Creates a nominal reference scoped to this module snapshot.</summary>
    /// <returns>A reference that resolves to this exact owned definition.</returns>
    public TypeReference ToReference() => new(this);
}

/// <summary>A nominal reference backed by a TypeDef or a physical TypeRef row.</summary>
/// <remarks>Object equality is reference equality. Structural expressions and TypeSpec decoding are separate.</remarks>
public sealed class TypeReference
{
    private readonly bool definitionBacked;
    internal TypeReference(TypeDefinition definition)
    {
        Module = definition.Module; MetadataToken = definition.MetadataToken;
        Namespace = definition.Namespace; Name = definition.Name; definitionBacked = true;
    }
    internal TypeReference(ModuleDefinition module, AssemblyDefinition.TypeReferenceRow row)
    { Module = module; MetadataToken = row.Token; Namespace = row.Namespace; Name = row.Name; ResolutionScopeToken = row.Scope; }
    /// <summary>Gets the owning module snapshot; for a physical TypeRef this is the consuming module.</summary>
    public ModuleDefinition Module { get; }
    /// <summary>Gets the TypeDef or physical TypeRef token within Module.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the stored namespace.</summary>
    public string Namespace { get; }
    /// <summary>Gets the stored metadata name.</summary>
    public string Name { get; }
    /// <summary>Gets a physical resolution-scope token; zero for definition-backed references or a nil scope.</summary>
    public uint ResolutionScopeToken { get; }
    /// <summary>Resolves a nominal reference through its local module or explicit assembly resolver.</summary>
    /// <param name="resolver">Required for assembly-scoped references; unused for local definitions.</param>
    /// <returns>The unique matching owned definition.</returns>
    /// <exception cref="InvalidDataException">Missing/mismatched dependency, absent/ambiguous type, nil or unsupported multi-module scope.</exception>
    /// <remarks>Does not follow exported-type forwarders or bind constructed generic TypeSpecs.</remarks>
    public TypeDefinition Resolve(IAssemblyResolver? resolver = null)
    {
        if (definitionBacked) return Module.GetTypeDefinition(MetadataToken)!;
        ModuleDefinition target;
        TypeDefinition? parent = null;
        switch (ResolutionScopeToken >> 24)
        {
            case 0 when ResolutionScopeToken == 1:
                target = Module;
                break;
            case 0x23:
                if (resolver is null) throw new InvalidDataException("assembly resolver required for TypeRef");
                var assembly = Module.AssemblyReferences.SingleOrDefault(reference => reference.MetadataToken == ResolutionScopeToken)
                    ?? throw new InvalidDataException("missing AssemblyRef scope");
                target = assembly.Resolve(resolver).MainModule;
                break;
            case 1:
                var outer = Module.TypeReferences.SingleOrDefault(reference => reference.MetadataToken == ResolutionScopeToken)
                    ?? throw new InvalidDataException("missing declaring TypeRef scope");
                parent = outer.Resolve(resolver);
                target = parent.Module;
                break;
            default: throw new InvalidDataException("unsupported or nil nominal resolution scope");
        }
        var matches = target.Types.Where(type => type.Name == Name && type.Namespace == Namespace && ReferenceEquals(type.DeclaringType, parent)).Take(2).ToArray();
        if (matches.Length != 1) throw new InvalidDataException("nominal type missing or ambiguous: " + Namespace + "." + Name);
        return matches[0];
    }
}
