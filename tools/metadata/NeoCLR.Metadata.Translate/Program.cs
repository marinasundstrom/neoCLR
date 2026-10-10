using System.Text.Json;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;

// Explicit temporary CLI bootstrap preparation; ordinary translation stays unchanged.
if (args.Length == 3 && args[0] == "--prepare-source-attribute-core")
{
    try
    {
        if (Path.GetFullPath(args[1]) == Path.GetFullPath(args[2]))
            throw new InvalidDataException("input and output paths must differ");
        using var image = Mono.Cecil.AssemblyDefinition.ReadAssembly(args[1]);
        var module = image.MainModule;
        var removedNames = new HashSet<string> { "System.AttributeTargets", "System.AttributeUsageAttribute" };
        foreach (var type in module.GetTypes())
            foreach (var attribute in type.CustomAttributes.ToArray())
                if (removedNames.Contains(attribute.AttributeType.FullName))
                    type.CustomAttributes.Remove(attribute);
        foreach (var type in module.Types.Where(t => removedNames.Contains(t.FullName)).ToArray())
            module.Types.Remove(type);
        // Retained declarations may not keep signatures referencing removed definitions.
        foreach (var type in module.GetTypes())
        {
            var signatures = type.Fields.Select(f => f.FieldType)
                .Concat(type.Properties.Select(p => p.PropertyType))
                .Concat(type.Methods.SelectMany(m => m.Parameters.Select(p => p.ParameterType).Append(m.ReturnType)));
            if (signatures.Any(t => removedNames.Contains(t.GetElementType().FullName)))
                throw new InvalidDataException("Retained bootstrap signature uses a source-owned attribute declaration.");
        }
        using var output = new FileStream(args[2], FileMode.CreateNew, FileAccess.Write);
        image.Write(output);
        Console.WriteLine("Prepared source-owned attribute bootstrap.");
        return 0;
    }
    catch (Exception error) when (error is IOException or InvalidDataException or UnauthorizedAccessException)
    {
        Console.Error.WriteLine(error.Message);
        return 1;
    }
}

if (args.Length < 2 || (args.Length - 2) % 2 != 0 ||
    Enumerable.Range(0, (args.Length - 2) / 2).Any(i => args[2 + i * 2] != "--reference"))
{
    Console.Error.WriteLine("Usage: NeoCLR.Metadata.Translate <module.neo.json> <module.neox> [--reference <native-artifact>]...");
    return 2;
}
try
{
    if (Path.GetFullPath(args[0]) == Path.GetFullPath(args[1]))
        throw new InvalidDataException("input and output paths must differ");
    if (new FileInfo(args[0]).Length > 32 * 1024 * 1024)
        throw new InvalidDataException("native JSON exceeds 32 MiB limit");
    var source = File.ReadAllBytes(args[0]);
    if (args.Length > 2)
    {
        var model = JsonNode.Parse(source)?.AsObject() ?? throw new InvalidDataException("missing native module");
        var references = model["references"] as JsonArray ?? throw new InvalidDataException("explicit reference list required");
        var names = new HashSet<string>(StringComparer.Ordinal);
        foreach (var reference in references)
            if (!names.Add(ReferenceName(reference)))
                throw new InvalidDataException("duplicate existing module reference");
        for (var i = 3; i < args.Length; i += 2)
        {
            if (new FileInfo(args[i]).Length > RuntimeAssemblyContainer.MaxLibraryImageSize)
                throw new InvalidDataException("reference exceeds native library image limit");
            var artifact = File.ReadAllBytes(args[i]);
            var json = artifact.AsSpan().StartsWith("MZ"u8)
                ? RuntimeAssemblyContainer.Read(artifact) : NativeModuleContainer.Read(artifact);
            using var dependency = JsonDocument.Parse(json);
            var root = dependency.RootElement;
            var name = root.GetProperty("name").GetString();
            var revision = root.TryGetProperty("revision", out var version) ? version.GetString() : null;
            if (string.IsNullOrEmpty(name) || string.IsNullOrEmpty(revision) ||
                name == ReferenceName(model) || !names.Add(name))
                throw new InvalidDataException("reference requires a distinct module with an explicit revision");
            references.Add(new JsonObject { ["name"] = name, ["revision"] = revision });
        }
        source = JsonSerializer.SerializeToUtf8Bytes(model);
    }
    var image = NativeModuleContainer.WriteLibraryBinary(source);
    using var output = new FileStream(args[1], FileMode.CreateNew, FileAccess.Write);
    output.Write(image);
    Console.WriteLine($"Wrote {image.Length} bytes. Runtime semantic validation is required.");
    return 0;
}
catch (Exception error) when (error is IOException or InvalidDataException or UnauthorizedAccessException or JsonException or InvalidOperationException or KeyNotFoundException)
{
    Console.Error.WriteLine(error.Message);
    return 1;
}

static string ReferenceName(JsonNode? node)
{
    if (node is JsonObject item) node = item["name"];
    if (node is not JsonValue value || !value.TryGetValue<string>(out var name) || string.IsNullOrEmpty(name))
        throw new InvalidDataException("invalid module reference name");
    return name;
}
