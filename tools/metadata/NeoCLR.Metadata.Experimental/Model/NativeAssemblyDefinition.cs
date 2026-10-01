using System.Globalization;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Read-only declaration snapshot of the bounded metadata writer's native format-5 output.</summary>
/// <remarks>Reads metadata only. Native bodies are opaque and must still be verified by neoCLR. General format-5 assemblies and structural types are unsupported.</remarks>
public sealed class NativeAssemblyDefinition
{
    private sealed record TypeRow(string Namespace, string Name, string NativeName, TypeVisibility Visibility, bool IsStatic, FieldRow[] Fields);
    private sealed record FieldRow(string Name, JsonElement Type, FieldVisibility Visibility, bool IsReadOnly = false);
    private sealed record MethodRow(string Namespace, string Name, int Owner, MethodSignature Signature, MethodVisibility Visibility, bool Instance);
    private sealed record PropertyRow(int Owner, string Name, SignatureType Type, int Getter, int Setter);
    private readonly PropertyRow[] properties;
    private readonly TypeRow[] types;
    private readonly MethodRow[] methods;
    private NativeAssemblyDefinition(AssemblyIdentity identity, TypeRow[] types, MethodRow[] methods, PropertyRow[] properties, AssemblyIdentity[] references)
    { Identity = identity; this.types = types; this.methods = methods; this.properties = properties; References = System.Array.AsReadOnly(references); }
    /// <summary>Gets the exact unsigned assembly identity retained from the native metadata manifest.</summary>
    public AssemblyIdentity Identity { get; }
    /// <summary>Gets owned exact identities of direct native dependencies in manifest order.</summary>
    /// <remarks>Includes implementation dependencies omitted from primitive-only PE projections. No dependencies are resolved or loaded.</remarks>
    public IReadOnlyList<AssemblyIdentity> References { get; }

    /// <summary>Copies the supported declarations from native UTF-8 JSON without executing code.</summary>
    /// <param name="image">At most 4 MiB of API-produced format-5 JSON.</param>
    /// <returns>An owned snapshot independent of the input buffer and JSON document.</returns>
    /// <exception cref="InvalidDataException">Malformed/ambiguous JSON, unsupported declarations, inconsistent names/identity, or exceeded bounds.</exception>
    /// <remarks>Rejects unknown declaration fields, duplicate JSON properties and unsupported signatures. Instruction bodies are not interpreted or translated.</remarks>
    public static NativeAssemblyDefinition ReadAssembly(ReadOnlySpan<byte> image)
    {
        if (image.Length > MetadataArtifactReader.MaxImageSize) throw new InvalidDataException("native image exceeds limit");
        try
        {
            using var document = JsonDocument.Parse(image.ToArray(), new JsonDocumentOptions { MaxDepth = 64 });
            var root = document.RootElement;
            CheckDuplicates(root);
            Shape(root, "format", "name", "revision", "references", "entry", "assemblies", "types", "functions");
            Require(root.GetProperty("format").GetInt32() == 5, "unsupported native format");
            var manifests = Array(root, "assemblies", 1); Require(manifests.Length == 1, "one assembly manifest required");
            var manifest = manifests[0]; Shape(manifest, "name", "full_name", "modules", "references");
            var identityText = Text(manifest, "full_name");
            var identity = ReadIdentity(identityText);
            CheckName(identity.Name);
            Require(identity.PublicKeyToken.Length == 0 && identity.Flags == 0, "unsupported native assembly identity");
            var moduleName = ModuleName(identityText);
            Require(Text(root, "name") == moduleName && Text(root, "revision") == identity.Version.ToString() && Text(manifest, "name") == identity.Name, "native identity mismatch");
            var modules = Array(manifest, "modules", 1);
            Require(modules.Length == 1 && modules[0].GetString() == identity.Name + ".dll", "unsupported native module");
            var referenceNames = Array(manifest, "references", 256).Select(e => e.GetString() ?? throw new InvalidDataException("null reference identity")).ToArray();
            var references = Array(root, "references", 256);
            Require(referenceNames.Length == references.Length, "native reference count mismatch");
            var seenReferences = new HashSet<AssemblyIdentity>();
            var referenceIdentities = new List<AssemblyIdentity>();
            for (int i = 0; i < references.Length; i++)
            {
                var reference = ReadIdentity(referenceNames[i]); Shape(references[i], "name", "revision");
                Require(!reference.Equals(identity) && seenReferences.Add(reference), "duplicate or self native reference");
                Require(Text(references[i], "name") == ModuleName(referenceNames[i]) && Text(references[i], "revision") == reference.Version.ToString(), "native reference identity mismatch");
                referenceIdentities.Add(reference);
            }
            var typeElements = Array(root, "types", 256);
            var types = new List<TypeRow>();
            int nextPropertyToken = 0x17000001;
            int nextFieldToken = 0x04000001;
            foreach (var type in typeElements)
            {
                var typeFields = new List<string> { "name", "fields", "is_reference_type", "is_abstract", "is_sealed", "origin" };
                if (type.TryGetProperty("properties", out _)) typeFields.Add("properties");
                var propertyElements = type.TryGetProperty("properties", out _) ? Array(type, "properties", 256) : [];
                var visibility = TypeVisibility.Public;
                if (type.TryGetProperty("visibility", out var access))
                {
                    typeFields.Add("visibility");
                    visibility = access.GetString() switch {
                        "public" => TypeVisibility.Public,
                        "internal" => TypeVisibility.Internal,
                        _ => throw new InvalidDataException("unsupported native type visibility")
                    };
                }
                Shape(type, typeFields.ToArray());
                var isStatic = type.GetProperty("is_abstract").GetBoolean();
                Require(type.GetProperty("is_reference_type").GetBoolean() && type.GetProperty("is_sealed").GetBoolean() == isStatic, "unsupported native type shape");
                var fieldRows = new List<FieldRow>();
                foreach (var field in Array(type, "fields", 256))
                {
                    Shape(field, "name", "ty", "visibility");
                    var fieldName = Text(field, "name"); CheckName(fieldName);
                    Require(fieldName.Length <= 1024 && fieldRows.All(f => f.Name != fieldName), "invalid or duplicate field");
                    var fieldVisibility = Text(field, "visibility") switch {
                        "public" => FieldVisibility.Public, "internal" => FieldVisibility.Internal, "private" => FieldVisibility.Private,
                        _ => throw new InvalidDataException("unsupported field visibility") };
                    fieldRows.Add(new(fieldName, field.GetProperty("ty").Clone(), fieldVisibility));
                }
                Require(!isStatic || fieldRows.Count == 0, "static type cannot have instance fields");
                Require(nextFieldToken + fieldRows.Count <= 0x04001001, "too many fields");
                var nativeName = Text(type, "name"); var prefix = moduleName + ".T_";
                Require(nativeName.StartsWith(prefix, StringComparison.Ordinal), "native type scope mismatch");
                var parts = nativeName[prefix.Length..].Split('_'); Require(parts.Length == 2, "invalid native type name");
                var ns = Decode(parts[0]); var name = Decode(parts[1]);
                Require(name.Length > 0 && name != "<Module>" && ns.Length + name.Length <= 1024 && types.All(t => t.NativeName != nativeName), "invalid or duplicate native type");
                CheckName(ns.Length == 0 ? name : ns + "." + name);
                var origin = type.GetProperty("origin");
                var originFields = new List<string> { "assembly", "module", "name", "token", "publicly_visible" };
                if (origin.TryGetProperty("property_tokens", out _))
                {
                    originFields.Add("property_tokens");
                    var tokens = Array(origin, "property_tokens", 256);
                    Require(tokens.Length == propertyElements.Length && tokens.Select((t, i) => t.GetInt32() == nextPropertyToken + i).All(v => v), "property origin mismatch");
                }
                else Require(propertyElements.Length == 0, "missing property origins");
                nextPropertyToken += propertyElements.Length;
                Require(nextPropertyToken <= 0x17001001, "too many properties");
                if (fieldRows.Count == 0 && propertyElements.Length == 0) Shape(origin, originFields.ToArray());
                else
                {
                    originFields.AddRange(["field_tokens", "field_access", "field_readonly"]);
                    Shape(origin, originFields.ToArray());
                    var tokens = Array(origin, "field_tokens", 256); var fieldAccess = Array(origin, "field_access", 256); var readOnly = Array(origin, "field_readonly", 256);
                    Require(tokens.Length == fieldRows.Count && fieldAccess.Length == fieldRows.Count && readOnly.Length == fieldRows.Count, "field origin count mismatch");
                    for (int f = 0; f < fieldRows.Count; f++)
                    {
                        Require(tokens[f].GetInt32() == nextFieldToken + f && fieldAccess[f].GetString() ==
                            (fieldRows[f].Visibility == FieldVisibility.Internal ? "Assembly" : fieldRows[f].Visibility.ToString()), "field origin mismatch");
                        fieldRows[f] = fieldRows[f] with { IsReadOnly = readOnly[f].GetBoolean() };
                    }
                }
                nextFieldToken += fieldRows.Count;
                Origin(origin, identityText, identity, ns.Length == 0 ? name : ns + "." + name, 0x02000002 + types.Count);
                Require(origin.GetProperty("publicly_visible").GetBoolean() == (visibility == TypeVisibility.Public), "native type visibility mismatch");
                types.Add(new(ns, name, nativeName, visibility, isStatic, fieldRows.ToArray()));
            }
            // Private identity graph for immutable declaration signatures, remapped into each projection.
            var signatureGraph = new AssemblyBuilder(identity, identity);
            var signatureOwners = types.Select(t => t.IsStatic ? signatureGraph.AddType(t.Namespace, t.Name) : signatureGraph.AddClass(t.Namespace, t.Name)).ToArray();
            int genericArity = 0;
            SignatureType ReadType(JsonElement element, bool allowVoid, bool allowArray = true)
            {
                if (element.ValueKind == JsonValueKind.String) return ReadPrimitive(element.GetString(), allowVoid);
                if (element.TryGetProperty("MethodTypeParameter", out var parameter))
                {
                    Shape(element, "MethodTypeParameter");
                    int ordinal = parameter.GetInt32(); Require(ordinal >= 0 && ordinal < genericArity, "method parameter outside scope");
                    return SignatureType.MethodParameter(ordinal);
                }
                if (element.TryGetProperty("ArrayRef", out var arrayElement))
                {
                    Require(allowArray, "nested arrays unsupported"); Shape(element, "ArrayRef");
                    return SignatureType.ArrayOf(ReadType(arrayElement, false, false));
                }
                Shape(element, "Named");
                var index = types.FindIndex(t => t.NativeName == Text(element, "Named") && !t.IsStatic);
                Require(index >= 0, "signature class must be an owned root");
                return signatureOwners[index];
            }
            foreach (var type in types)
                foreach (var field in type.Fields) _ = ReadType(field.Type, false);
            string TypeKey(SignatureType type) => type.MethodParameterIndex is { } index ? "method:" + index : type.ArrayElement is { } element ? "array:" + TypeKey(element)
                : type.ClassType is { } c ? "class:" + System.Array.IndexOf(signatureOwners, c) : "primitive:" + type.Primitive;
            var methods = new List<MethodRow>();
            var methodNames = new List<string>();
            var counts = new Dictionary<int, int>();
            var seenMethods = new HashSet<(int Owner, string Namespace, string Name, string Parameters)>();
            foreach (var method in Array(root, "functions", 4096))
            {
                var fields = new List<string> { "name", "owner", "parameters", "returns", "no_result", "origin", "body" };
                var genericNames = method.TryGetProperty("generic_parameters", out _) ? Array(method, "generic_parameters", 32).Select(p => p.GetString() ?? throw new InvalidDataException("null generic name")).ToArray() : [];
                genericArity = genericNames.Length;
                if (method.TryGetProperty("generic_parameters", out _)) fields.Add("generic_parameters");
                if (method.TryGetProperty("locals", out _)) fields.Add("locals");
                var ns = "";
                if (method.TryGetProperty("namespace", out var scope))
                {
                    fields.Add("namespace"); ns = scope.GetString() ?? throw new InvalidDataException("null function namespace");
                    FunctionNamespaceEncoding.Validate(ns);
                }
                var instance = false;
                if (method.TryGetProperty("instance", out var instanceValue)) { fields.Add("instance"); instance = instanceValue.GetBoolean(); }
                var visibility = MethodVisibility.Public;
                if (method.TryGetProperty("visibility", out var access))
                {
                    fields.Add("visibility");
                    visibility = access.GetString() switch {
                        "public" => MethodVisibility.Public,
                        "internal" => MethodVisibility.Internal,
                        "private" => MethodVisibility.Private,
                        _ => throw new InvalidDataException("unsupported native method visibility")
                    };
                }
                Shape(method, fields.ToArray());
                var origin = method.GetProperty("origin"); Shape(origin, "assembly", "module", "name", "token", "member_access", "parameter_tokens");
                var name = Text(origin, "name"); Require(name.Length is > 0 and <= 1024, "invalid native method name"); CheckName(name);
                var owner = method.GetProperty("owner"); int ownerIndex = -1;
                if (owner.ValueKind != JsonValueKind.Null)
                {
                    Shape(owner, "Named"); var ownerName = Text(owner, "Named");
                    ownerIndex = types.FindIndex(t => t.NativeName == ownerName); Require(ownerIndex >= 0, "missing native method owner");
                }
                else Require(methods.All(m => m.Owner < 0), "global functions must precede type methods");
                Require(methods.Count == 0 || methods[^1].Owner <= ownerIndex, "native owner declaration order mismatch");
                if (method.TryGetProperty("locals", out _))
                    foreach (var local in Array(method, "locals", 256)) _ = ReadType(local, false);
                var parameters = Array(method, "parameters", 256);
                var parameterTypes = parameters.Select(p => ReadType(p, false)).ToArray();
                var noResult = method.GetProperty("no_result").GetBoolean();
                var resultType = ReadType(method.GetProperty("returns"), true);
                Require(noResult == (resultType == PrimitiveType.Void), "inconsistent native result");
                Require(ownerIndex < 0 || ns.Length == 0, "type method cannot declare a function namespace");
                Require(!instance || ownerIndex >= 0 && !types[ownerIndex].IsStatic, "instance method requires a root class");
                var constructor = instance && name == ".ctor";
                Require(!constructor || resultType == PrimitiveType.Void && genericArity == 0, "constructor must be nongeneric with no result");
                var expectedName = constructor ? types[ownerIndex].NativeName + "..ctor" : (ownerIndex < 0 ? moduleName + ".F_" : types[ownerIndex].NativeName + ".M_") + Convert.ToHexString(Encoding.UTF8.GetBytes(ownerIndex < 0 ? FunctionNamespaceEncoding.Encode(ns, name) : name));
                Require(Text(method, "name") == expectedName, "native callable name mismatch");
                Origin(origin, identityText, identity, name, 0x06000001 + methods.Count);
                Require(Text(origin, "member_access") == (visibility == MethodVisibility.Internal ? "Assembly" : visibility.ToString()), "native method visibility mismatch");
                Require(ownerIndex >= 0 || visibility != MethodVisibility.Private, "private native global function unsupported");
                var tokens = Array(origin, "parameter_tokens", 256);
                Require(tokens.Length == parameters.Length && tokens.All(t => t.GetInt32() == 0), "unsupported native parameter metadata");
                Require(method.GetProperty("body").ValueKind == JsonValueKind.Array, "native body array required");
                Require(seenMethods.Add((ownerIndex, ns, name, genericArity + ":" + string.Join(",", parameterTypes.Select(TypeKey)))), "duplicate native signature");
                counts.TryGetValue(ownerIndex, out int count); Require(count < 256, "too many methods per owner"); counts[ownerIndex] = count + 1;
                methods.Add(new(ns, name, ownerIndex, new(resultType, parameterTypes, genericNames), visibility, instance)); methodNames.Add(expectedName);
            }
            genericArity = 0;
            var properties = new List<PropertyRow>();
            var usedAccessors = new HashSet<int>();
            for (int owner = 0; owner < typeElements.Length; owner++)
            {
                var names = new HashSet<string>();
                foreach (var property in typeElements[owner].TryGetProperty("properties", out _) ? Array(typeElements[owner], "properties", 256) : [])
                {
                    Shape(property, "name", "instance", "parameters", "ty", "getter", "setter");
                    var name = Text(property, "name"); CheckName(name);
                    var indices = Array(property, "parameters", 256).Select(p => ReadType(p, false)).ToArray();
                    Require(name.Length <= 1024 && names.Add(name + "(" + string.Join(",", indices.Select(TypeKey)) + ")"), "invalid or duplicate property");
                    var valueType = ReadType(property.GetProperty("ty"), false);
                    var instance = property.GetProperty("instance").GetBoolean();
                    int Accessor(string key, bool setter)
                    {
                        var reference = property.GetProperty(key);
                        if (reference.ValueKind == JsonValueKind.Null) return -1;
                        Shape(reference, "name", "owner", "instance", "parameters");
                        var referenceOwner = reference.GetProperty("owner"); Shape(referenceOwner, "Named");
                        Require(Text(referenceOwner, "Named") == types[owner].NativeName && reference.GetProperty("instance").GetBoolean() == instance, "property accessor owner/instance mismatch");
                        var parameters = Array(reference, "parameters", 256).Select(p => ReadType(p, false)).ToArray();
                        Require(parameters.SequenceEqual(setter ? indices.Append(valueType) : indices), "property accessor parameters mismatch");
                        var candidates = methods.Select((m, i) => (m, i)).Where(p => p.m.Owner == owner && methodNames[p.i] == Text(reference, "name") && p.m.Signature.ParameterTypes.SequenceEqual(parameters)).ToArray();
                        Require(candidates.Length == 1, "missing or ambiguous property accessor");
                        var (method, index) = candidates[0];
                        Require(method.Instance == instance && method.Signature.GenericParameterNames.Count == 0 && method.Name != ".ctor" && method.Signature.ReturnType == (setter ? PrimitiveType.Void : valueType) && usedAccessors.Add(index), "incompatible or reused property accessor");
                        return index;
                    }
                    var getter = Accessor("getter", false); var setter = Accessor("setter", true);
                    Require(getter >= 0 || setter >= 0, "property needs an accessor");
                    properties.Add(new(owner, name, valueType, getter, setter));
                }
            }
            var entry = Text(root, "entry");
            if (entry.Length != 0)
            {
                var candidates = methodNames.Select((name, index) => (name, index)).Where(p => p.name == entry && !methods[p.index].Instance && methods[p.index].Signature.GenericParameterNames.Count == 0 && methods[p.index].Signature.ParameterTypes.Count == 0 && methods[p.index].Signature.ReturnType.Primitive is PrimitiveType.Int32 or PrimitiveType.Void).ToArray();
                Require(candidates.Length == 1, "invalid native entry point");
            }
            return new(identity, types.ToArray(), methods.ToArray(), properties.ToArray(), referenceIdentities.ToArray());
        }
        catch (Exception error) when (error is JsonException or InvalidOperationException or KeyNotFoundException or FormatException or ArgumentException or OverflowException)
        { throw new InvalidDataException("invalid native metadata", error); }
    }

    /// <summary>Creates a reference-only PE snapshot for the temporary .NET semantic-loader bridge.</summary>
    /// <param name="coreLibrary">Explicit core identity supplying System.Object and ReferenceAssemblyAttribute.</param>
    /// <returns>Owned PE bytes containing declarations, a ReferenceAssemblyAttribute and throwing placeholder bodies.</returns>
    /// <exception cref="ArgumentNullException">Core identity is null.</exception>
    /// <exception cref="InvalidDataException">Projection exceeds writer limits.</exception>
    /// <remarks>No native body is translated. Entry points and native dependency references are not projected: supported signatures contain primitives and owned root-class references.
    /// This is compiler reference metadata, never an executable replacement for the native artifact. Per-call MVIDs may differ.</remarks>
    public byte[] CreateReferenceAssembly(AssemblyIdentity coreLibrary)
    {
        ArgumentNullException.ThrowIfNull(coreLibrary);
        var graph = new AssemblyBuilder(Identity, coreLibrary);
        var owners = types.Select(t => t.IsStatic ? graph.AddType(t.Namespace, t.Name, t.Visibility) : graph.AddClass(t.Namespace, t.Name, t.Visibility)).ToArray();
        SignatureType ProjectType(JsonElement type) => type.ValueKind == JsonValueKind.String ? (SignatureType)ReadPrimitive(type.GetString(), false)
            : type.TryGetProperty("ArrayRef", out var element) ? SignatureType.ArrayOf(ProjectType(element))
            : owners[System.Array.FindIndex(types, row => row.NativeName == Text(type, "Named"))];
        for (int t = 0; t < types.Length; t++)
            foreach (var field in types[t].Fields) owners[t].AddField(field.Name, ProjectType(field.Type), field.Visibility, field.IsReadOnly);
        SignatureType Remap(SignatureType type) => type.ArrayElement is { } element ? SignatureType.ArrayOf(Remap(element))
            : type.ClassType is { } c ? owners[System.Array.FindIndex(types, t => t.Namespace == c.Namespace && t.Name == c.Name)] : type;
        var projectedMethods = new List<MethodBuilder>();
        foreach (var method in methods)
        {
            var signature = new MethodSignature(Remap(method.Signature.ReturnType), method.Signature.ParameterTypes.Select(Remap), method.Signature.GenericParameterNames);
            var output = method.Owner < 0 ? graph.AddFunction(method.Namespace, method.Name, signature, method.Visibility)
                : !method.Instance ? owners[method.Owner].AddMethod(method.Name, signature, method.Visibility)
                : method.Name == ".ctor" ? owners[method.Owner].AddConstructor(signature, method.Visibility)
                : owners[method.Owner].AddInstanceMethod(method.Name, signature, method.Visibility);
            // Reference emission supplies throwing bodies; do not invent executable native behavior.
            projectedMethods.Add(output);
        }
        foreach (var property in properties)
            owners[property.Owner].AddProperty(property.Name, Remap(property.Type), property.Getter < 0 ? null : projectedMethods[property.Getter], property.Setter < 0 ? null : projectedMethods[property.Setter]);
        return graph.WriteReferenceImage();
    }
    private static PrimitiveType ReadPrimitive(string? name, bool allowVoid) => name switch
    {
        "String" => PrimitiveType.String,
        "Int64" => PrimitiveType.Int64,
        "Int32" => PrimitiveType.Int32,
        "Boolean" => PrimitiveType.Boolean,
        "Void" when allowVoid => PrimitiveType.Void,
        _ => throw new InvalidDataException("unsupported native signature type")
    };
    private static void CheckName(string name)
    {
        Require(!string.IsNullOrWhiteSpace(name) && !name.Any(char.IsControl), "invalid native descriptive name");
        _ = new UTF8Encoding(false, true).GetByteCount(name);
    }
    private static void Require(bool condition, string message) { if (!condition) throw new InvalidDataException(message); }
    private static JsonElement[] Array(JsonElement element, string name, int limit)
    {
        var value = element.GetProperty(name); Require(value.ValueKind == JsonValueKind.Array && value.GetArrayLength() <= limit, "invalid or excessive " + name);
        return value.EnumerateArray().ToArray();
    }
    private static string Text(JsonElement element, string name) => element.GetProperty(name).GetString() ?? throw new InvalidDataException("null " + name);
    private static void Shape(JsonElement element, params string[] names)
    {
        Require(element.ValueKind == JsonValueKind.Object && element.EnumerateObject().Count() == names.Length && names.All(n => element.TryGetProperty(n, out _)), "unsupported native metadata fields");
    }
    private static void CheckDuplicates(JsonElement value)
    {
        if (value.ValueKind == JsonValueKind.Object)
        {
            var seen = new HashSet<string>();
            foreach (var property in value.EnumerateObject()) { Require(seen.Add(property.Name), "duplicate JSON property"); CheckDuplicates(property.Value); }
        }
        else if (value.ValueKind == JsonValueKind.Array) foreach (var item in value.EnumerateArray()) CheckDuplicates(item);
    }
    private static string Decode(string hex)
    {
        var bytes = Convert.FromHexString(hex); Require(Convert.ToHexString(bytes) == hex, "noncanonical native name");
        return new UTF8Encoding(false, true).GetString(bytes);
    }
    private static AssemblyIdentity ReadIdentity(string text)
    {
        var values = JsonSerializer.Deserialize<string[]>(text) ?? throw new InvalidDataException("missing native identity");
        Require(values.Length == 5 && values.All(v => v is not null), "invalid native identity tuple");
        var identity = new AssemblyIdentity(values[0], Version.Parse(values[1]), values[2], values[3], uint.Parse(values[4], CultureInfo.InvariantCulture));
        var canonical = JsonSerializer.Serialize(new[] { identity.Name, identity.Version.ToString(), identity.Culture, identity.PublicKeyToken, identity.Flags.ToString(CultureInfo.InvariantCulture) });
        Require(text == canonical, "noncanonical native identity"); return identity;
    }
    private static string ModuleName(string identity) => "NeoMetadata_" + Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(identity)));
    private static void Origin(JsonElement origin, string identityText, AssemblyIdentity identity, string name, int token)
    {
        Require(Text(origin, "assembly") == identityText && Text(origin, "module") == identity.Name + ".dll" && Text(origin, "name") == name && origin.GetProperty("token").GetInt32() == token, "native origin mismatch");
    }
}
