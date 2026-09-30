using System.Text.Json;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned function inventory from a standalone native binary module, including translated libraries.</summary>
/// <remarks>Transport and inventory are validated, not method bodies or the complete native type system.
/// Explicit static Int32 callable projections are partial views, never complete replacement assemblies.</remarks>
public sealed class NativeLibraryDefinition
{
    private readonly JsonElement root;
    private NativeLibraryDefinition(JsonElement root)
    {
        this.root = root;
        ModuleName = Text(root, "name");
        var types = root.GetProperty("types").EnumerateArray().ToArray();
        var typeNames = types.Select(t => Text(t, "name")).ToArray();
        if (types.Select(t => (Text(t, "name"), t.TryGetProperty("generic_parameters", out var parameters) ? parameters.GetArrayLength() : 0)).Distinct().Count() != typeNames.Length) throw new InvalidDataException("duplicate native type");
        TypeNames = Array.AsReadOnly(typeNames);
        var functions = root.GetProperty("functions").EnumerateArray().ToArray();
        if (functions.Length > 65536 || types.Length > 65536) throw new InvalidDataException("native inventory limit exceeded");
        Functions = Array.AsReadOnly(functions.Select((f, i) => new NativeFunctionDefinition(this, f, i)).ToArray());
    }
    /// <summary>Gets the original native module name; this is not an inferred CLI assembly identity.</summary>
    public string ModuleName { get; }
    /// <summary>Gets native type names in table order; names may repeat for different generic arities. Instance/generic contracts are not projected.</summary>
    public IReadOnlyList<string> TypeNames { get; }
    /// <summary>Gets all functions in native table order, including unsupported callable signatures.</summary>
    public IReadOnlyList<NativeFunctionDefinition> Functions { get; }
    /// <summary>Reads a schema-2/3 standalone binary module into an owned declaration inventory.</summary>
    /// <param name="image">Complete NEOX module under NativeModuleContainer's bounds.</param>
    /// <returns>An immutable snapshot independent of the caller's buffer.</returns>
    /// <exception cref="InvalidDataException">Invalid transport/inventory, duplicate types or exceeded limits.</exception>
    /// <remarks>Functions retain opaque signature data internally; unsupported signatures are not approximated.</remarks>
    public static NativeLibraryDefinition ReadAssembly(ReadOnlySpan<byte> image)
    {
        var json = NativeModuleContainer.Read(image);
        try
        {
            using var document = JsonDocument.Parse(json);
            return new(document.RootElement.Clone());
        }
        catch (Exception error) when (error is JsonException or InvalidOperationException or KeyNotFoundException or ArgumentException)
        { throw new InvalidDataException("invalid native library inventory", error); }
    }
    /// <summary>Projects explicitly selected public static Int32 callables as a reference-only CLI view.</summary>
    /// <param name="projectionIdentity">Explicit unsigned synthetic identity for this partial view, distinct from the native assembly.</param>
    /// <param name="coreLibrary">Explicit host primitive/reference-attribute core identity.</param>
    /// <param name="functions">One or more owned functions, at most 4096; duplicates and unsupported selections fail.</param>
    /// <returns>Owned reference PE bytes with throwing bodies and selected types as static method containers.</returns>
    /// <exception cref="ArgumentNullException">A required argument is null.</exception>
    /// <exception cref="InvalidDataException">Unsupported or foreign function, empty/duplicate selection, owner/signature collision or limit.</exception>
    /// <remarks>Preserves callable names and Int32 signatures, not parameter names, instance shape, fields, properties,
    /// generics, attributes or assembly identity. No runtime implementation is emitted. Never use this view as a complete core library.</remarks>
    public byte[] CreateStaticInt32ReferenceAssembly(AssemblyIdentity projectionIdentity, AssemblyIdentity coreLibrary,
        IEnumerable<NativeFunctionDefinition> functions)
    {
        ArgumentNullException.ThrowIfNull(projectionIdentity);
        ArgumentNullException.ThrowIfNull(coreLibrary);
        ArgumentNullException.ThrowIfNull(functions);
        if (projectionIdentity.PublicKeyToken.Length != 0 || projectionIdentity.Flags != 0 || projectionIdentity.Equals(coreLibrary))
            throw new InvalidDataException("projection requires a distinct unsigned identity");
        var selected = functions.Take(4097).ToArray();
        if (selected.Length is 0 or > 4096 || selected.Distinct().Count() != selected.Length)
            throw new InvalidDataException("invalid static callable selection");
        var graph = new AssemblyBuilder(projectionIdentity, coreLibrary);
        var owners = new Dictionary<string, TypeBuilder>(StringComparer.Ordinal);
        var signatures = new HashSet<(string Owner, string Name, int Count)>();
        try
        {
            foreach (var function in selected)
            {
                if (function is null || !ReferenceEquals(function.Library, this) || !function.TryGetStaticInt32Signature(out var count))
                    throw new InvalidDataException("unsupported or foreign static callable");
                var owner = function.DeclaringTypeName!;
                if (!signatures.Add((owner, function.Name, count))) throw new InvalidDataException("duplicate callable signature");
                var type = root.GetProperty("types").EnumerateArray().SingleOrDefault(t => Text(t, "name") == owner && !HasItems(t, "generic_parameters"));
                if (type.ValueKind != JsonValueKind.Object || !IsPublic(type) || HasItems(type, "generic_parameters") || owner.Contains('`') || owner.Contains('+') ||
                    (type.TryGetProperty("origin", out var origin) && (origin.ValueKind != JsonValueKind.Object || origin.TryGetProperty("publicly_visible", out var visible) && visible.ValueKind != JsonValueKind.True)))
                    throw new InvalidDataException("unsupported callable owner");
                if (!owners.TryGetValue(owner, out var builder))
                {
                    if (owners.Count >= 256 || owner.Length > 1024) throw new InvalidDataException("callable owner limit exceeded");
                    var split = owner.LastIndexOf('.');
                    builder = graph.AddType(split < 0 ? "" : owner[..split], owner[(split + 1)..]);
                    owners.Add(owner, builder);
                }
                var method = builder.AddMethod(function.Name[(owner.Length + 1)..], count);
                method.LoadConstant(0);
                method.Return();
            }
            return graph.WriteReferenceImage();
        }
        catch (ArgumentException error) { throw new InvalidDataException("invalid callable projection", error); }
    }
    internal static bool IsPublic(JsonElement element)
        => !element.TryGetProperty("visibility", out var visibility) || visibility.ValueKind == JsonValueKind.String && visibility.GetString() == "public";
    internal static bool HasItems(JsonElement element, string name)
        => element.TryGetProperty(name, out var value) && (value.ValueKind != JsonValueKind.Array || value.GetArrayLength() != 0);
    internal static string Text(JsonElement element, string name)
    {
        var text = element.GetProperty(name).GetString();
        if (string.IsNullOrWhiteSpace(text) || text.Length > 4096 || text.Any(char.IsControl)) throw new InvalidDataException("invalid native name");
        return text;
    }
}

/// <summary>An owned native function declaration; unsupported signatures remain in the inventory.</summary>
public sealed class NativeFunctionDefinition
{
    private readonly JsonElement declaration;
    internal NativeFunctionDefinition(NativeLibraryDefinition library, JsonElement declaration, int index)
    {
        Library = library; this.declaration = declaration; TableIndex = index;
        Name = NativeLibraryDefinition.Text(declaration, "name");
        if (declaration.TryGetProperty("owner", out var owner) && owner.ValueKind == JsonValueKind.Object &&
            owner.TryGetProperty("Named", out var named) && named.ValueKind == JsonValueKind.String)
            DeclaringTypeName = named.GetString();
    }
    /// <summary>Gets the owning native inventory.</summary>
    public NativeLibraryDefinition Library { get; }
    /// <summary>Gets the zero-based position in this snapshot's function table, not a CLI token or persistent identity.</summary>
    public int TableIndex { get; }
    /// <summary>Gets the original fully qualified native function name.</summary>
    public string Name { get; }
    /// <summary>Gets a nominal Named owner, or null for free functions and other owner forms.</summary>
    public string? DeclaringTypeName { get; }
    /// <summary>Recognizes public static nongeneric Int32 parameters/result on a nominal owner.</summary>
    /// <param name="parameterCount">Int32 argument count on success; zero on failure.</param>
    /// <returns>False for unsupported or malformed signatures, including native Result, inhabited Void and instance calls.</returns>
    public bool TryGetStaticInt32Signature(out int parameterCount)
    {
        parameterCount = 0;
        if (!NativeLibraryDefinition.IsPublic(declaration)) return false;
        if (DeclaringTypeName is not { Length: > 0 } owner || !Name.StartsWith(owner + ".", StringComparison.Ordinal) ||
            Name[(owner.Length + 1)..] is not { Length: > 0 } shortName || shortName.Contains('.') ||
            NativeLibraryDefinition.HasItems(declaration, "generic_parameters")) return false;
        foreach (var flag in new[] { "instance", "is_abstract", "is_virtual", "is_override", "receiver_byref", "receiver_readonly", "no_result" })
            if (declaration.TryGetProperty(flag, out var value) && value.ValueKind != JsonValueKind.False) return false;
        if (declaration.TryGetProperty("origin", out var origin) && (origin.ValueKind != JsonValueKind.Object ||
            origin.TryGetProperty("member_access", out var access) && (access.ValueKind != JsonValueKind.String || access.GetString() != "Public"))) return false;
        if (!declaration.TryGetProperty("returns", out var returns) || returns.ValueKind != JsonValueKind.String || returns.GetString() != "Int32" ||
            !declaration.TryGetProperty("parameters", out var parameters) || parameters.ValueKind != JsonValueKind.Array || parameters.GetArrayLength() > 256 ||
            parameters.EnumerateArray().Any(p => p.ValueKind != JsonValueKind.String || p.GetString() != "Int32")) return false;
        parameterCount = parameters.GetArrayLength();
        return true;
    }
}
