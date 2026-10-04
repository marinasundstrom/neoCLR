using System.Globalization;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>Read-only declaration snapshot of the bounded metadata writer's native format-5 output.</summary>
/// <remarks>Reads metadata only. Native bodies are opaque and must still be verified by neoCLR. General format-5 assemblies and structural types are unsupported.</remarks>
public sealed partial class NativeAssemblyDefinition
{
    private sealed record TypeRow(string Namespace, string Name, string NativeName, TypeVisibility Visibility, bool IsStatic, bool IsInterface, bool IsValueType, JsonElement[] BaseInterfaces, FieldRow[] Fields, string[] GenericNames, (int Parameter, string Bound)[] Constraints, Dictionary<int, TypeParameterConstraints> SpecialConstraints, int DeclaringType) { internal List<SignatureType> InterfaceSignatures { get; } = []; internal JsonElement[] RawAttributes { get; init; } = []; internal List<AttributeRow> Attributes { get; } = []; }
    private sealed record AttributeRow(SignatureType Owner, CustomAttributeArgument[] Arguments);
    private sealed record FieldRow(string Name, JsonElement Type, FieldVisibility Visibility, bool IsReadOnly = false, SignatureType? Signature = null);
    private sealed record MethodRow(string Namespace, string Name, int Owner, MethodSignature Signature, MethodVisibility Visibility, bool Instance, bool Override) { internal Dictionary<int, string> ParameterNames { get; init; } = []; }
    private sealed record PropertyRow(int Owner, string Name, SignatureType Type, int Getter, int Setter, SignatureType[] Parameters);
    private sealed record NativeTypeAlias(string NativeName, AssemblyIdentity Assembly, string Namespace, string Name, int Arity, bool ValueType, string? Declaring);
    private readonly Dictionary<(string Name, int Arity), NativeTypeAlias> nativeTypeAliases;
    private readonly HashSet<string> valueTypeReferences;
    private readonly PropertyRow[] properties;
    private readonly TypeRow[] types;
    private readonly MethodRow[] methods;
    private readonly uint entryPointToken;
    private NativeAssemblyDefinition(AssemblyIdentity identity, TypeRow[] types, MethodRow[] methods, PropertyRow[] properties, AssemblyIdentity[] references, HashSet<string> valueTypeReferences, Dictionary<(string Name, int Arity), NativeTypeAlias> nativeTypeAliases, uint entryPointToken)
    { this.entryPointToken = entryPointToken; this.nativeTypeAliases = nativeTypeAliases; this.valueTypeReferences = valueTypeReferences; Identity = identity; this.types = types; this.methods = methods; this.properties = properties; References = System.Array.AsReadOnly(references); }
    /// <summary>Gets the exact unsigned assembly identity retained from the native metadata manifest.</summary>
    public AssemblyIdentity Identity { get; }
    /// <summary>Gets owned exact identities of direct native dependencies in manifest order.</summary>
    /// <remarks>Includes implementation dependencies; PE projections retain dependencies used in signatures. No dependencies are resolved or loaded.</remarks>
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
            var manifest = manifests[0];
            var manifestFields = new List<string> { "name", "full_name", "modules", "references" };
            var valueTypeReferences = new HashSet<string>(StringComparer.Ordinal);
            if (manifest.TryGetProperty("value_type_references", out _))
            {
                manifestFields.Add("value_type_references");
                foreach (var valueReference in Array(manifest, "value_type_references", 4096))
                    Require(valueReference.ValueKind == JsonValueKind.String && valueTypeReferences.Add(valueReference.GetString()!), "invalid or duplicate value type reference");
            }
            var nativeTypeAliases = new Dictionary<(string Name, int Arity), NativeTypeAlias>();
            var nativeModuleAliases = new Dictionary<AssemblyIdentity, (string Module, string? Revision)>();
            if (manifest.TryGetProperty("native_module_bindings", out _))
            {
                manifestFields.Add("native_module_bindings");
                foreach (var binding in Array(manifest, "native_module_bindings", 256))
                {
                    Shape(binding, "assembly", "module", "revision");
                    var assemblyIdentity = ReadIdentity(Text(binding, "assembly"));
                    var module = Text(binding, "module"); CheckName(module);
                    Require(nativeModuleAliases.TryAdd(assemblyIdentity, (module, binding.GetProperty("revision").GetString())), "duplicate native module binding");
                }
            }
            if (manifest.TryGetProperty("native_type_bindings", out _))
            {
                manifestFields.Add("native_type_bindings");
                foreach (var binding in Array(manifest, "native_type_bindings", 4096))
                {
                    Shape(binding, "native_name", "assembly", "namespace", "name", "arity", "value_type", "declaring");
                    var alias = new NativeTypeAlias(Text(binding, "native_name"), ReadIdentity(Text(binding, "assembly")), binding.GetProperty("namespace").GetString()!, Text(binding, "name"), binding.GetProperty("arity").GetInt32(), binding.GetProperty("value_type").GetBoolean(), binding.GetProperty("declaring").GetString());
                    CheckName(alias.NativeName);
                    Require(alias.Arity is >= 0 and <= 32 && alias.Namespace is not null && nativeTypeAliases.TryAdd((alias.NativeName, alias.Arity), alias), "invalid native type binding");
                }
            }
            if (manifest.TryGetProperty("array_backing", out _)) manifestFields.Add("array_backing");
            Shape(manifest, manifestFields.ToArray());
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
                var reference = ReadIdentity(referenceNames[i]);
                if (references[i].ValueKind != JsonValueKind.String) Shape(references[i], "name", "revision");
                Require(!reference.Equals(identity) && seenReferences.Add(reference), "duplicate or self native reference");
                var expected = nativeModuleAliases.TryGetValue(reference, out var nativeModule) ? nativeModule : (ModuleName(referenceNames[i]), reference.Version.ToString());
                Require(references[i].ValueKind == JsonValueKind.String ? references[i].GetString() == expected.Item1 && expected.Item2 is null : Text(references[i], "name") == expected.Item1 && references[i].GetProperty("revision").GetString() == expected.Item2, "native reference identity mismatch");
                referenceIdentities.Add(reference);
            }
            Require(nativeModuleAliases.Keys.All(seenReferences.Contains) && nativeTypeAliases.Values.All(a => nativeModuleAliases.ContainsKey(a.Assembly)), "unscoped native binding");
            var typeElements = Array(root, "types", 256);
            if (manifest.TryGetProperty("array_backing", out var arrayBacking))
            {
                Shape(arrayBacking, "module", "revision", "index");
                Require(Text(arrayBacking, "module") == moduleName && Text(arrayBacking, "revision") == identity.Version.ToString() &&
                    arrayBacking.GetProperty("index").GetInt32() is var backingIndex && backingIndex >= 0 && backingIndex < typeElements.Length,
                    "invalid native array backing identity");
                var backing = typeElements[arrayBacking.GetProperty("index").GetInt32()];
                var backingFields = Array(backing, "fields", 256);
                Require(backing.TryGetProperty("is_reference_type", out var referenceBacking) && referenceBacking.GetBoolean() &&
                    (!backing.TryGetProperty("is_abstract", out var abstractBacking) || !abstractBacking.GetBoolean()) &&
                    !backing.TryGetProperty("base", out _) && !backing.TryGetProperty("declaring_type", out _) &&
                    Array(backing, "generic_parameters", 32).Length == 1 &&
                    (!backing.TryGetProperty("generic_constraints", out var bounds) || bounds.GetArrayLength() == 0) &&
                    backingFields.Length == 1 && Text(backingFields[0], "visibility") == "private" &&
                    backingFields[0].GetProperty("ty").TryGetProperty("ArrayRef", out var vector) &&
                    vector.ValueKind == JsonValueKind.Object && vector.TryGetProperty("TypeParameter", out var element) && element.GetInt32() == 0,
                    "invalid native array backing storage contract");
            }
            var types = new List<TypeRow>();
            int nextPropertyToken = 0x17000001;
            int nextFieldToken = 0x04000001;
            foreach (var type in typeElements)
            {
                var typeFields = new List<string> { "name", "fields", "is_reference_type", "is_abstract", "is_sealed", "origin" };
                var typeNames = type.TryGetProperty("generic_parameters", out _) ? Array(type, "generic_parameters", 32).Select(p => p.GetString() ?? throw new InvalidDataException("null type parameter name")).ToArray() : [];
                if (type.TryGetProperty("generic_parameters", out _)) typeFields.Add("generic_parameters");
                var constraints = new List<(int Parameter, string Bound)>();
                var specialConstraints = new Dictionary<int, TypeParameterConstraints>();
                if (type.TryGetProperty("generic_constraints", out _))
                {
                    typeFields.Add("generic_constraints");
                    foreach (var constraint in Array(type, "generic_constraints", 128))
                    {
                        Shape(constraint, "parameter", "kind");
                        int parameter = constraint.GetProperty("parameter").GetInt32();
                        Require(parameter >= 0 && parameter < typeNames.Length, "invalid type bound parameter");
                        var kind = constraint.GetProperty("kind");
                        if (kind.ValueKind == JsonValueKind.String)
                        {
                            var flag = kind.GetString() switch { "ReferenceType" => TypeParameterConstraints.ReferenceType, "ValueType" => TypeParameterConstraints.ValueType, "DefaultConstructor" => TypeParameterConstraints.DefaultConstructor, _ => throw new InvalidDataException("unsupported special constraint") };
                            var existing = specialConstraints.GetValueOrDefault(parameter);
                            Require(!existing.HasFlag(flag), "duplicate special constraint"); specialConstraints[parameter] = existing | flag;
                            continue;
                        }
                        Require(constraints.All(c => c.Parameter != parameter), "duplicate type bound parameter");
                        Shape(kind, "TypeBound");
                        var bound = kind.GetProperty("TypeBound"); Shape(bound, "Named");
                        constraints.Add((parameter, Text(bound, "Named")));
                    }
                }
                if (type.TryGetProperty("custom_attributes", out _)) typeFields.Add("custom_attributes");
                if (type.TryGetProperty("properties", out _)) typeFields.Add("properties");
                var propertyElements = type.TryGetProperty("properties", out _) ? Array(type, "properties", 256) : [];
                var visibility = TypeVisibility.Public;
                if (type.TryGetProperty("visibility", out var access))
                {
                    typeFields.Add("visibility");
                    visibility = access.GetString() switch
                    {
                        "public" => TypeVisibility.Public,
                        "internal" => TypeVisibility.Internal,
                        _ => throw new InvalidDataException("unsupported native type visibility")
                    };
                }
                bool isInterface = type.TryGetProperty("representation", out var representation);
                if (isInterface) { typeFields.Add("representation"); Require(representation.GetString() == "Interface", "unsupported native representation"); }
                var baseInterfaces = type.TryGetProperty("implements", out _) ? Array(type, "implements", 256).Select(b => b.Clone()).ToArray() : [];
                if (type.TryGetProperty("implements", out _)) typeFields.Add("implements");

                int declaringType = -1;
                if (type.TryGetProperty("declaring_type", out var declaring))
                {
                    typeFields.Add("declaring_type"); Shape(declaring, "module", "revision", "index");
                    declaringType = declaring.GetProperty("index").GetInt32();
                    Require(Text(declaring, "module") == moduleName && Text(declaring, "revision") == identity.Version.ToString() &&
                        declaringType >= 0 && declaringType < types.Count && types[declaringType].GenericNames.Length == 0,
                        "invalid or unsupported nested owner");
                }
                Shape(type, typeFields.ToArray());
                var isStatic = !isInterface && type.GetProperty("is_abstract").GetBoolean();
                var isValueType = !isInterface && !type.GetProperty("is_reference_type").GetBoolean();
                Require((!isValueType || !isStatic) &&
                    type.GetProperty("is_reference_type").GetBoolean() == (!isInterface && !isValueType) && type.GetProperty("is_sealed").GetBoolean() == (isStatic || isValueType) &&
                    (!isInterface || !type.GetProperty("is_abstract").GetBoolean() && Array(type, "fields", 256).Length == 0), "unsupported native type shape");
                var fieldRows = new List<FieldRow>();
                foreach (var field in Array(type, "fields", 256))
                {
                    Shape(field, "name", "ty", "visibility");
                    var fieldName = Text(field, "name"); CheckName(fieldName);
                    Require(fieldName.Length <= 1024 && fieldRows.All(f => f.Name != fieldName), "invalid or duplicate field");
                    var fieldVisibility = Text(field, "visibility") switch
                    {
                        "public" => FieldVisibility.Public,
                        "internal" => FieldVisibility.Internal,
                        "private" => FieldVisibility.Private,
                        _ => throw new InvalidDataException("unsupported field visibility")
                    };
                    fieldRows.Add(new(fieldName, field.GetProperty("ty").Clone(), fieldVisibility));
                }
                Require(!isStatic || fieldRows.Count == 0, "static type cannot have instance fields");
                Require(nextFieldToken + fieldRows.Count <= 0x04001001, "too many fields");
                var nativeName = Text(type, "name"); var prefix = moduleName + ".T_";
                Require(nativeName.StartsWith(prefix, StringComparison.Ordinal), "native type scope mismatch");
                string ns, name;
                if (declaringType >= 0)
                {
                    var nestedPrefix = types[declaringType].NativeName + ".N_";
                    Require(nativeName.StartsWith(nestedPrefix, StringComparison.Ordinal) && !isStatic && !isInterface, "invalid nested native type identity");
                    ns = ""; name = Decode(nativeName[nestedPrefix.Length..]);
                }
                else
                {
                    var parts = nativeName[prefix.Length..].Split('_'); Require(parts.Length == 2, "invalid native type name");
                    ns = Decode(parts[0]); name = Decode(parts[1]);
                }
                Require(name.Length > 0 && name != "<Module>" && ns.Length + name.Length <= 1024 && types.All(t => t.NativeName != nativeName), "invalid or duplicate native type");
                Require(typeNames.Length == 0 || name.EndsWith("`" + typeNames.Length, StringComparison.Ordinal), "generic owner arity name mismatch");
                CheckName(ns.Length == 0 ? name : ns + "." + name);
                var origin = type.GetProperty("origin");
                var originFields = new List<string> { "assembly", "module", "name", "token", "publicly_visible" };
                if (declaringType >= 0)
                {
                    originFields.Add("declaring_type_token");
                    Require(origin.GetProperty("declaring_type_token").GetInt32() == 0x02000002 + declaringType, "nested origin mismatch");
                }
                if (origin.TryGetProperty("property_tokens", out _))
                {
                    originFields.Add("property_tokens");
                    var tokens = Array(origin, "property_tokens", 256);
                    Require(tokens.Length == propertyElements.Length && tokens.Select((t, i) => t.GetInt32() == nextPropertyToken + i).All(v => v), "property origin mismatch");
                }
                else Require(propertyElements.Length == 0, "missing property origins");
                nextPropertyToken += propertyElements.Length;
                Require(nextPropertyToken <= 0x17001001, "too many properties");
                if (fieldRows.Count == 0 && !origin.TryGetProperty("field_tokens", out _)) Shape(origin, originFields.ToArray());
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
                types.Add(new(ns, name, nativeName, visibility, isStatic, isInterface, isValueType, baseInterfaces, fieldRows.ToArray(), typeNames, constraints.ToArray(), specialConstraints, declaringType) { RawAttributes = type.TryGetProperty("custom_attributes", out _) ? Array(type, "custom_attributes", 256).Select(a => a.Clone()).ToArray() : [] });
            }
            // Private identity graph for immutable declaration signatures, remapped into each projection.
            var signatureGraph = new AssemblyBuilder(identity, identity);
            foreach (var reference in valueTypeReferences)
            {
                var nativeMatches = nativeTypeAliases.Values.Where(a => a.NativeName == reference && a.ValueType).ToArray();
                Require(nativeMatches.Length <= 1, "ambiguous native value binding");
                var simpleName = nativeMatches.Length == 1 ? nativeMatches[0].Name : Decode(reference[(reference.LastIndexOf('_') + 1)..]);
                int marker = simpleName.LastIndexOf('`');
                int importedArity = 0;
                if (marker >= 0) Require(int.TryParse(simpleName[(marker + 1)..], out importedArity) && importedArity > 0, "invalid imported value arity");
                _ = ImportExternalType(signatureGraph, reference, importedArity, referenceIdentities, valueTypeReferences, nativeTypeAliases);
            }
            foreach (var alias in nativeTypeAliases.Values)
                _ = ImportExternalType(signatureGraph, alias.NativeName, alias.Arity, referenceIdentities, valueTypeReferences, nativeTypeAliases);
            int genericArity = 0; int typeArity = 0;
            var signatureOwners = DefineTypes(signatureGraph, types);
            for (int i = 0; i < types.Count; i++)
                foreach (var inherited in types[i].BaseInterfaces)
                {
                    bool constructed = inherited.TryGetProperty("Constructed", out var instance);
                    Shape(inherited, constructed ? "Constructed" : "Named");
                    if (constructed) Shape(instance, "definition", "arguments");
                    int parent = types.FindIndex(t => t.NativeName == (constructed ? Text(instance, "definition") : Text(inherited, "Named")));
                    if (parent < 0)
                    {
                        typeArity = types[i].GenericNames.Length;
                        var external = ReadType(inherited, false);
                        Require(external.ImportedType is { IsValueType: false }, "invalid external interface relationship");
                        types[i].InterfaceSignatures.Add(external);
                        typeArity = 0;
                        continue;
                    }
                    if (constructed)
                    {
                        typeArity = types[i].GenericNames.Length;
                        var inheritedType = signatureOwners[parent].MakeGenericInstance(Array(instance, "arguments", 32).Select(a => ReadType(a, false)).ToArray());
                        types[i].InterfaceSignatures.Add(inheritedType);
                        if (types[i].IsInterface) signatureOwners[i].AddBaseInterface(inheritedType);
                        else signatureOwners[i].AddInterfaceImplementation(inheritedType);
                        typeArity = 0;
                    }
                    else
                    {
                        types[i].InterfaceSignatures.Add(signatureOwners[parent]);
                        if (types[i].IsInterface) signatureOwners[i].AddBaseInterface(signatureOwners[parent]);
                        else signatureOwners[i].AddInterfaceImplementation(signatureOwners[parent]);
                    }
                }
            for (int i = 0; i < types.Count; i++)
                foreach (var constraint in types[i].Constraints)
                {
                    int bound = types.FindIndex(t => t.NativeName == constraint.Bound && !t.IsStatic && !t.IsInterface && !t.IsValueType && t.GenericNames.Length == 0);
                    Require(bound >= 0, "type bound requires owned nongeneric class");
                    signatureOwners[i].AddBaseTypeConstraint(constraint.Parameter, signatureOwners[bound]);
                }
            for (int i = 0; i < types.Count; i++)
                foreach (var (parameter, flags) in types[i].SpecialConstraints) signatureOwners[i].SetSpecialConstraints(parameter, flags);
            SignatureType ReadType(JsonElement element, bool allowVoid, bool allowArray = true, bool allowByReference = false, bool allowSelf = false)
            {
                if (element.ValueKind == JsonValueKind.String)
                {
                    if (!allowVoid && element.GetString() == "Void" && nativeTypeAliases.TryGetValue(("System.Void", 0), out var unit) && unit.ValueType && unit.Namespace == "System" && unit.Name == "Void")
                        return ImportExternalType(signatureGraph, "System.Void", 0, referenceIdentities, valueTypeReferences, nativeTypeAliases);
                    if (element.GetString() == "Char")
                    {
                        Require(nativeTypeAliases.TryGetValue(("System.Char", 0), out var character) && character.ValueType && character.Namespace == "System" && character.Name == "Char" && character.Declaring is null, "Char requires an explicit core value alias");
                        return ImportExternalType(signatureGraph, "System.Char", 0, referenceIdentities, valueTypeReferences, nativeTypeAliases);
                    }
                    if (element.GetString() == "SelfType")
                    {
                        Require(allowSelf, "Self requires an interface contract signature");
                        return SignatureType.Self;
                    }
                    return ReadPrimitive(element.GetString(), allowVoid);
                }
                if (element.TryGetProperty("Function", out var function))
                {
                    Shape(element, "Function"); Shape(function, "parameters", "returns", "no_result");
                    bool noResult = function.GetProperty("no_result").GetBoolean();
                    var returns = ReadType(function.GetProperty("returns"), noResult);
                    Require(noResult == (returns.Primitive == PrimitiveType.Void), "function result convention mismatch");
                    return SignatureType.Function(new MethodSignature(returns, Array(function, "parameters", 16).Select(p => ReadType(p, false))));
                }
                if (element.TryGetProperty("ByRef", out var target))
                {
                    Require(allowByReference, "byref only supported in method parameters");
                    Shape(element, "ByRef");
                    return SignatureType.ByReference(ReadType(target, false, allowSelf: allowSelf));
                }
                if (element.TryGetProperty("TypeParameter", out var typeParameter))
                {
                    Shape(element, "TypeParameter");
                    int ordinal = typeParameter.GetInt32(); Require(ordinal >= 0 && ordinal < typeArity, "type parameter outside scope");
                    return SignatureType.TypeParameter(ordinal);
                }
                if (element.TryGetProperty("MethodTypeParameter", out var parameter))
                {
                    Shape(element, "MethodTypeParameter");
                    int ordinal = parameter.GetInt32(); Require(ordinal >= 0 && ordinal < genericArity, "method parameter outside scope");
                    return SignatureType.MethodParameter(ordinal);
                }
                if (element.TryGetProperty("ArrayRef", out var arrayElement))
                {
                    Require(allowArray, "array signature unsupported here"); Shape(element, "ArrayRef");
                    return SignatureType.ArrayOf(ReadType(arrayElement, false, allowSelf: allowSelf));
                }
                if (element.TryGetProperty("Constructed", out var construction))
                {
                    Shape(element, "Constructed"); Shape(construction, "definition", "arguments");
                    var definition = types.FindIndex(t => t.NativeName == Text(construction, "definition") && !t.IsStatic && t.GenericNames.Length > 0);
                    var arguments = Array(construction, "arguments", 32).Select(a => ReadType(a, false, allowSelf: allowSelf)).ToArray();
                    return definition >= 0 ? signatureOwners[definition].MakeGenericInstance(arguments)
                        : ImportExternalType(signatureGraph, Text(construction, "definition"), arguments.Length, referenceIdentities, valueTypeReferences, nativeTypeAliases).MakeGenericInstance(arguments);
                }
                Shape(element, "Named");
                var index = types.FindIndex(t => t.NativeName == Text(element, "Named") && !t.IsStatic);
                return index >= 0 ? (SignatureType)signatureOwners[index]
                    : ImportExternalType(signatureGraph, Text(element, "Named"), 0, referenceIdentities, valueTypeReferences, nativeTypeAliases);
            }
            foreach (var type in types)
                foreach (var attribute in type.RawAttributes)
                {
                    Shape(attribute, "constructor", "arguments");
                    var constructor = attribute.GetProperty("constructor"); Shape(constructor, "name", "owner", "instance", "parameters");
                    var ownerValue = constructor.GetProperty("owner"); Shape(ownerValue, "Named");
                    Require(constructor.GetProperty("instance").GetBoolean() && Text(constructor, "name") == Text(ownerValue, "Named") + "..ctor", "invalid attribute constructor");
                    var owner = ReadType(ownerValue, false);
                    Require(owner.ClassType is { IsValueType: false, IsStatic: false, GenericParameterNames.Count: 0 } || owner.ImportedType is { IsValueType: false, GenericArity: 0 }, "invalid attribute type");
                    var parameters = Array(constructor, "parameters", 256);
                    var arguments = Array(attribute, "arguments", 256);
                    Require(parameters.Length == arguments.Length, "attribute argument count mismatch");
                    var decoded = new List<CustomAttributeArgument>();
                    for (int i = 0; i < arguments.Length; i++)
                    {
                        var parameter = parameters[i].GetString();
                        Require(parameter is "String" or "Int32" or "Boolean", "unsupported attribute parameter");
                        Shape(arguments[i], parameter!);
                        var value = arguments[i].GetProperty(parameter!);
                        decoded.Add(parameter switch
                        {
                            "String" => new(PrimitiveType.String, value.ValueKind == JsonValueKind.Null ? null : value.GetString()),
                            "Int32" => new(PrimitiveType.Int32, value.GetInt32()),
                            _ => new(PrimitiveType.Boolean, value.GetBoolean())
                        });
                    }
                    type.Attributes.Add(new(owner, decoded.ToArray()));
                }
            for (int typeIndex = 0; typeIndex < types.Count; typeIndex++)
            {
                var type = types[typeIndex];
                typeArity = type.GenericNames.Length;
                for (int fieldIndex = 0; fieldIndex < type.Fields.Length; fieldIndex++)
                {
                    var field = type.Fields[fieldIndex];
                    var storage = ReadType(field.Type, false);
                    type.Fields[fieldIndex] = field with { Signature = storage };
                    signatureOwners[typeIndex].AddField(field.Name, storage, field.Visibility, field.IsReadOnly);
                }
            }
            signatureGraph.ValidateValueLayouts();
            typeArity = 0;
            string TypeKey(SignatureType type) => type.IsSelf ? "self" : type.FunctionSignature is { } function ? "function:" + TypeKey(function.ReturnType) + "(" + string.Join(",", function.ParameterTypes.Select(TypeKey)) + ")" : type.ByReferenceElement is { } target ? "byref:" + TypeKey(target) : type.ImportedType is { } imported ? "external:" + JsonSerializer.Serialize(new { imported.AssemblyIdentity, imported.Namespace, imported.Name, Arguments = imported.TypeArguments.Select(TypeKey).ToArray() }) : type.GenericInstance is { } instance ? "constructed:" + System.Array.IndexOf(signatureOwners, instance.Definition) + "<" + string.Join(",", instance.TypeArguments.Select(TypeKey)) + ">" : type.TypeParameterIndex is { } ordinal ? "type:" + ordinal : type.MethodParameterIndex is { } index ? "method:" + index : type.ArrayElement is { } element ? "array:" + TypeKey(element)
                : type.ClassType is { } c ? "class:" + System.Array.IndexOf(signatureOwners, c) : "primitive:" + type.Primitive;
            var methods = new List<MethodRow>();
            var methodNames = new List<string>();
            var counts = new Dictionary<int, int>();
            var seenMethods = new HashSet<(int Owner, string Namespace, string Name, string Parameters)>();
            foreach (var method in Array(root, "functions", 4096))
            {
                var fields = new List<string> { "name", "owner", "parameters", "returns", "no_result", "origin", "body" };
                if (method.TryGetProperty("parameter_names", out _)) fields.Add("parameter_names");
                if (method.TryGetProperty("out_parameters", out _)) fields.Add("out_parameters");
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
                bool receiverByRef = false;
                if (method.TryGetProperty("receiver_byref", out var byRefReceiver)) { fields.Add("receiver_byref"); receiverByRef = byRefReceiver.GetBoolean(); }
                var visibility = MethodVisibility.Public;
                if (method.TryGetProperty("visibility", out var access))
                {
                    fields.Add("visibility");
                    visibility = access.GetString() switch
                    {
                        "public" => MethodVisibility.Public,
                        "internal" => MethodVisibility.Internal,
                        "private" => MethodVisibility.Private,
                        _ => throw new InvalidDataException("unsupported native method visibility")
                    };
                }
                bool isAbstract = method.TryGetProperty("is_abstract", out var abstractFlag);
                bool isVirtual = method.TryGetProperty("is_virtual", out var virtualFlag);
                bool isOverride = method.TryGetProperty("is_override", out var overrideFlag);
                if (isOverride) { fields.Add("is_override"); Require(overrideFlag.GetBoolean(), "override flag must be true"); }
                if (isAbstract) { fields.Add("is_abstract"); Require(abstractFlag.GetBoolean(), "abstract flag must be true"); }
                if (isVirtual) { fields.Add("is_virtual"); Require(virtualFlag.GetBoolean(), "virtual flag must be true"); }
                Shape(method, fields.ToArray());
                var origin = method.GetProperty("origin"); Shape(origin, "assembly", "module", "name", "token", "member_access", "parameter_tokens");
                var name = Text(origin, "name"); Require(name.Length is > 0 and <= 1024, "invalid native method name"); CheckName(name);
                var owner = method.GetProperty("owner"); int ownerIndex = -1;
                if (owner.ValueKind != JsonValueKind.Null)
                {
                    var constructed = owner.TryGetProperty("Constructed", out var construction);
                    Shape(owner, constructed ? "Constructed" : "Named");
                    if (constructed) Shape(construction, "definition", "arguments");
                    var ownerName = constructed ? Text(construction, "definition") : Text(owner, "Named");
                    ownerIndex = types.FindIndex(t => t.NativeName == ownerName); Require(ownerIndex >= 0, "missing native method owner");
                    var arity = types[ownerIndex].GenericNames.Length;
                    Require(constructed == (arity > 0), "open owner construction required");
                    if (constructed)
                    {
                        var arguments = Array(construction, "arguments", 32);
                        Require(arguments.Length == arity, "owner argument count mismatch");
                        for (int i = 0; i < arguments.Length; i++) { Shape(arguments[i], "TypeParameter"); Require(arguments[i].GetProperty("TypeParameter").GetInt32() == i, "method owner must be open definition"); }
                    }
                }
                else Require(methods.All(m => m.Owner < 0), "global functions must precede type methods");
                Require(methods.Count == 0 || methods[^1].Owner <= ownerIndex, "native owner declaration order mismatch");
                typeArity = ownerIndex < 0 ? 0 : types[ownerIndex].GenericNames.Length;
                if (method.TryGetProperty("locals", out _))
                    foreach (var local in Array(method, "locals", 256)) _ = ReadType(local, false);
                var parameters = Array(method, "parameters", 256);
                var parameterTypes = parameters.Select(p => ReadType(p, false, allowByReference: true, allowSelf: ownerIndex >= 0 && types[ownerIndex].IsInterface)).ToArray();
                var noResult = method.GetProperty("no_result").GetBoolean();
                var resultType = ReadType(method.GetProperty("returns"), noResult, allowSelf: ownerIndex >= 0 && types[ownerIndex].IsInterface);
                Require(noResult == (resultType == PrimitiveType.Void), "inconsistent native result");
                Require(ownerIndex < 0 || ns.Length == 0, "type method cannot declare a function namespace");
                Require(!instance || ownerIndex >= 0 && !types[ownerIndex].IsStatic, "instance method requires a nonstatic owner");
                Require(receiverByRef == (instance && types[ownerIndex].IsValueType), "value instance receiver must be byref");
                var interfaceOwner = ownerIndex >= 0 && types[ownerIndex].IsInterface;
                Require(isAbstract == interfaceOwner && isVirtual == (interfaceOwner || isOverride), "interface method flags mismatch");
                Require(!interfaceOwner || instance && visibility == MethodVisibility.Public && name != ".ctor" && genericArity == 0 &&
                    method.GetProperty("body").GetArrayLength() == 0 && (!method.TryGetProperty("locals", out var interfaceLocals) || interfaceLocals.GetArrayLength() == 0), "invalid abstract interface method");
                Require(!isOverride || !interfaceOwner && instance && types[ownerIndex].IsValueType &&
                    visibility == MethodVisibility.Public && name == "ToString" && resultType == PrimitiveType.String &&
                    parameterTypes.Length == 0 && genericArity == 0 && nativeModuleAliases.Values.Count(m => m.Module == "System") == 1,
                    "unsupported or unbound native Object override");
                var constructor = instance && name == ".ctor";
                Require(!constructor || resultType == PrimitiveType.Void && genericArity == 0, "constructor must be nongeneric with no result");
                var expectedName = isOverride ? types[ownerIndex].NativeName + "." + name : constructor ? types[ownerIndex].NativeName + "..ctor" : (ownerIndex < 0 ? moduleName + ".F_" : types[ownerIndex].NativeName + ".M_") + Convert.ToHexString(Encoding.UTF8.GetBytes(ownerIndex < 0 ? FunctionNamespaceEncoding.Encode(ns, name) : name));
                Require(Text(method, "name") == expectedName, "native callable name mismatch");
                Origin(origin, identityText, identity, name, 0x06000001 + methods.Count);
                Require(Text(origin, "member_access") == (visibility == MethodVisibility.Internal ? "Assembly" : visibility.ToString()), "native method visibility mismatch");
                Require(ownerIndex >= 0 || visibility != MethodVisibility.Private, "private native global function unsupported");
                var tokens = Array(origin, "parameter_tokens", 256);
                Require(tokens.Length == parameters.Length && tokens.All(t => t.GetInt32() == 0), "unsupported native parameter metadata");
                Require(method.GetProperty("body").ValueKind == JsonValueKind.Array, "native body array required");
                Require(seenMethods.Add((ownerIndex, ns, name, genericArity + ":" + string.Join(",", parameterTypes.Select(TypeKey)))), "duplicate native signature");
                counts.TryGetValue(ownerIndex, out int count); Require(count < 256, "too many methods per owner"); counts[ownerIndex] = count + 1;
                var parameterNames = new Dictionary<int, string>();
                if (method.TryGetProperty("parameter_names", out _))
                {
                    var names = Array(method, "parameter_names", 256);
                    Require(names.Length == parameterTypes.Length, "parameter names must align with signature");
                    for (int i = 0; i < names.Length; i++)
                    {
                        if (names[i].ValueKind == JsonValueKind.Null) continue;
                        var parameterName = names[i].GetString();
                        Require(parameterName is { Length: > 0 and <= 1024 } && !parameterName.Any(char.IsControl), "invalid parameter name");
                        parameterNames.Add(i, parameterName!);
                    }
                }
                methods.Add(new(ns, name, ownerIndex, new(resultType, parameterTypes, genericNames, method.TryGetProperty("out_parameters", out _) ? Array(method, "out_parameters", 256).Select(p => p.GetInt32()) : []), visibility, instance, isOverride) { ParameterNames = parameterNames }); methodNames.Add(expectedName);
            }
            genericArity = 0; typeArity = 0;
            var properties = new List<PropertyRow>();
            var usedAccessors = new HashSet<int>();
            for (int owner = 0; owner < typeElements.Length; owner++)
            {
                typeArity = types[owner].GenericNames.Length;
                var names = new HashSet<string>();
                foreach (var property in typeElements[owner].TryGetProperty("properties", out _) ? Array(typeElements[owner], "properties", 256) : [])
                {
                    Shape(property, "name", "instance", "parameters", "ty", "getter", "setter");
                    var name = Text(property, "name"); CheckName(name);
                    var indices = Array(property, "parameters", 256).Select(p => ReadType(p, false, allowSelf: types[owner].IsInterface)).ToArray();
                    Require(name.Length <= 1024 && names.Add(name + "(" + string.Join(",", indices.Select(TypeKey)) + ")"), "invalid or duplicate property");
                    var valueType = ReadType(property.GetProperty("ty"), false, allowSelf: types[owner].IsInterface);
                    var instance = property.GetProperty("instance").GetBoolean();
                    int Accessor(string key, bool setter)
                    {
                        var reference = property.GetProperty(key);
                        if (reference.ValueKind == JsonValueKind.Null) return -1;
                        Shape(reference, "name", "owner", "instance", "parameters");
                        var referenceOwner = reference.GetProperty("owner");
                        var constructed = referenceOwner.TryGetProperty("Constructed", out var construction);
                        Shape(referenceOwner, constructed ? "Constructed" : "Named");
                        Require(constructed == (typeArity > 0), "property accessor requires open owner construction");
                        if (constructed)
                        {
                            Shape(construction, "definition", "arguments");
                            var arguments = Array(construction, "arguments", 32);
                            Require(arguments.Length == typeArity, "property accessor owner arity mismatch");
                            for (int i = 0; i < arguments.Length; i++)
                            {
                                Shape(arguments[i], "TypeParameter");
                                Require(arguments[i].GetProperty("TypeParameter").GetInt32() == i, "property accessor requires canonical open owner");
                            }
                        }
                        var ownerName = constructed ? Text(construction, "definition") : Text(referenceOwner, "Named");
                        Require(ownerName == types[owner].NativeName && reference.GetProperty("instance").GetBoolean() == instance, "property accessor owner/instance mismatch");
                        var parameters = Array(reference, "parameters", 256).Select(p => ReadType(p, false, allowByReference: true, allowSelf: types[owner].IsInterface)).ToArray();
                        Require(parameters.SequenceEqual(setter ? indices.Append(valueType) : indices), "property accessor parameters mismatch");
                        var candidates = methods.Select((m, i) => (m, i)).Where(p => p.m.Owner == owner && methodNames[p.i] == Text(reference, "name") && p.m.Signature.ParameterTypes.SequenceEqual(parameters)).ToArray();
                        Require(candidates.Length == 1, "missing or ambiguous property accessor");
                        var (method, index) = candidates[0];
                        Require(method.Instance == instance && method.Signature.GenericParameterNames.Count == 0 && method.Name != ".ctor" && method.Signature.ReturnType == (setter ? PrimitiveType.Void : valueType) && usedAccessors.Add(index), "incompatible or reused property accessor");
                        return index;
                    }
                    var getter = Accessor("getter", false); var setter = Accessor("setter", true);
                    Require(getter >= 0 || setter >= 0, "property needs an accessor");
                    properties.Add(new(owner, name, valueType, getter, setter, indices));
                }
            }
            var entry = Text(root, "entry");
            uint entryPointToken = 0;
            if (entry.Length != 0)
            {
                var candidates = methodNames.Select((name, index) => (name, index)).Where(p => p.name == entry && !methods[p.index].Instance && (methods[p.index].Owner < 0 || types[methods[p.index].Owner].GenericNames.Length == 0) && methods[p.index].Signature.GenericParameterNames.Count == 0 && methods[p.index].Signature.ParameterTypes.Count == 0 && methods[p.index].Signature.ReturnType.Primitive is PrimitiveType.Int32 or PrimitiveType.Void).ToArray();
                Require(candidates.Length == 1, "invalid native entry point");
                entryPointToken = 0x06000001u + (uint)candidates[0].index;
            }
            return new(identity, types.ToArray(), methods.ToArray(), properties.ToArray(), referenceIdentities.ToArray(), valueTypeReferences, nativeTypeAliases, entryPointToken);
        }
        catch (Exception error) when (error is JsonException or InvalidOperationException or KeyNotFoundException or FormatException or ArgumentException or OverflowException)
        { throw new InvalidDataException("invalid native metadata", error); }
    }

    /// <summary>Creates a reference-only PE snapshot for the temporary .NET semantic-loader bridge.</summary>
    /// <param name="coreLibrary">Explicit core identity supplying System.Object and ReferenceAssemblyAttribute.</param>
    /// <returns>Owned PE bytes containing declarations, a ReferenceAssemblyAttribute and throwing placeholder bodies for concrete methods; abstract interface methods remain bodyless.</returns>
    /// <exception cref="ArgumentNullException">Core identity is null.</exception>
    /// <exception cref="InvalidDataException">Projection exceeds writer limits.</exception>
    /// <remarks>No native body is translated. Entry points and body-only dependencies are not projected. External signature references retain their exact assembly scope.
    /// This is compiler reference metadata, never an executable replacement for the native artifact. Per-call MVIDs may differ.</remarks>
    public byte[] CreateReferenceAssembly(AssemblyIdentity coreLibrary)
    {
        ArgumentNullException.ThrowIfNull(coreLibrary);
        if (nativeTypeAliases.TryGetValue(("System.Void", 0), out var unitAlias) && !unitAlias.Assembly.Equals(coreLibrary))
            throw new InvalidDataException("inhabited Void projection requires the explicit core scope");
        if (nativeTypeAliases.TryGetValue(("System.Char", 0), out var charAlias) && !charAlias.Assembly.Equals(coreLibrary))
            throw new InvalidDataException("Char projection requires the explicit core scope");
        var graph = new AssemblyBuilder(Identity, coreLibrary);
        var owners = DefineTypes(graph, types);
        for (int i = 0; i < types.Length; i++)
            foreach (var constraint in types[i].Constraints)
                owners[i].AddBaseTypeConstraint(constraint.Parameter, owners[System.Array.FindIndex(types, t => t.NativeName == constraint.Bound)]);
        for (int i = 0; i < types.Length; i++)
            foreach (var (parameter, flags) in types[i].SpecialConstraints) owners[i].SetSpecialConstraints(parameter, flags);
        SignatureType ProjectType(JsonElement type) => type.ValueKind == JsonValueKind.String ? type.GetString() == "Char" ? ProjectNamed("System.Char") : type.GetString() == "Void" && nativeTypeAliases.ContainsKey(("System.Void", 0)) ? ProjectNamed("System.Void") : (SignatureType)ReadPrimitive(type.GetString(), false)
            : type.TryGetProperty("Function", out var function) ? SignatureType.Function(new MethodSignature(function.GetProperty("no_result").GetBoolean() ? PrimitiveType.Void : ProjectType(function.GetProperty("returns")), Array(function, "parameters", 16).Select(ProjectType)))
            : type.TryGetProperty("TypeParameter", out var parameter) ? SignatureType.TypeParameter(parameter.GetInt32())
            : type.TryGetProperty("Constructed", out var instance) ? ProjectConstruction(Text(instance, "definition"), Array(instance, "arguments", 32).Select(ProjectType).ToArray())
            : type.TryGetProperty("ArrayRef", out var element) ? SignatureType.ArrayOf(ProjectType(element))
            : ProjectNamed(Text(type, "Named"));
        SignatureType ProjectNamed(string name)
        {
            int index = System.Array.FindIndex(types, row => row.NativeName == name);
            return index >= 0 ? (SignatureType)owners[index] : ImportExternalType(graph, name, 0, References, valueTypeReferences, nativeTypeAliases);
        }
        SignatureType ProjectConstruction(string name, SignatureType[] arguments)
        {
            int index = System.Array.FindIndex(types, row => row.NativeName == name);
            return index >= 0 ? owners[index].MakeGenericInstance(arguments) : ImportExternalType(graph, name, arguments.Length, References, valueTypeReferences, nativeTypeAliases).MakeGenericInstance(arguments);
        }
        for (int i = 0; i < types.Length; i++)
            foreach (var inherited in types[i].BaseInterfaces)
            {
                var contract = ProjectType(inherited);
                if (contract.ImportedType is not null) throw new NotSupportedException("CLI projection of external interface declarations requires resolved contracts; use native metadata import");
                if (contract.GenericInstance is { } constructed) { if (types[i].IsInterface) owners[i].AddBaseInterface(constructed); else owners[i].AddInterfaceImplementation(constructed); }
                else if (types[i].IsInterface) owners[i].AddBaseInterface(contract.ClassType!);
                else owners[i].AddInterfaceImplementation(contract.ClassType!);
            }
        for (int t = 0; t < types.Length; t++)
            foreach (var field in types[t].Fields) owners[t].AddField(field.Name, ProjectType(field.Type), field.Visibility, field.IsReadOnly);
        var mappedOwners = new Dictionary<TypeBuilder, TypeBuilder>();
        TypeBuilder RemapOwner(TypeBuilder original)
        {
            if (mappedOwners.Count == 0)
                foreach (var (owner, index) in original.Assembly.Types.Select((owner, index) => (owner, index))) mappedOwners.Add(owner, owners[index]);
            return mappedOwners[original];
        }
        SignatureType Remap(SignatureType type) => type.FunctionSignature is { } function ? function.Substitute(Remap) : type.ByReferenceElement is { } target ? SignatureType.ByReference(Remap(target)) : type.ImportedType is { } imported ? RemapImported(imported) : type.GenericInstance is { } instance ? RemapOwner(instance.Definition).MakeGenericInstance(instance.TypeArguments.Select(Remap).ToArray()) : type.ArrayElement is { } element ? SignatureType.ArrayOf(Remap(element))
            : type.ClassType is { } c ? RemapOwner(c) : type;
        SignatureType RemapImported(ImportedTypeReference type)
        {
            var definition = graph.ImportTypeIdentity(type.AssemblyIdentity, type.Namespace, type.Name, type.GenericArity, type.IsValueType, type.DeclaringType is null ? null : RemapImported(type.DeclaringType).ImportedType);
            return type.TypeArguments.Count == 0 ? definition : definition.MakeGenericInstance(type.TypeArguments.Select(Remap).ToArray());
        }
        var projectedMethods = new List<MethodBuilder>();
        foreach (var method in methods)
        {
            var signature = new MethodSignature(Remap(method.Signature.ReturnType), method.Signature.ParameterTypes.Select(Remap), method.Signature.GenericParameterNames, method.Signature.OutParameters);
            var output = method.Owner < 0 ? graph.AddFunction(method.Namespace, method.Name, signature, method.Visibility)
                : owners[method.Owner].IsInterface ? owners[method.Owner].AddInterfaceMethod(method.Name, signature)
                : !method.Instance ? owners[method.Owner].AddMethod(method.Name, signature, method.Visibility)
                : method.Override ? owners[method.Owner].AddOverride(method.Name, signature)
                : method.Name == ".ctor" ? owners[method.Owner].AddConstructor(signature, method.Visibility)
                : owners[method.Owner].AddInstanceMethod(method.Name, signature, method.Visibility);
            foreach (var pair in method.ParameterNames) output.SetParameterName(pair.Key, pair.Value);
            // Reference emission supplies throwing bodies; do not invent executable native behavior.
            projectedMethods.Add(output);
        }
        foreach (var property in properties)
            owners[property.Owner].AddProperty(property.Name, Remap(property.Type), property.Getter < 0 ? null : projectedMethods[property.Getter], property.Setter < 0 ? null : projectedMethods[property.Setter]);
        for (int i = 0; i < types.Length; i++)
            foreach (var attribute in types[i].Attributes)
            {
                var owner = Remap(attribute.Owner);
                var reference = owner.ClassType is { } local ? local.Definition.ToReference()
                    : graph.Definition.MainModule.ImportReference(owner.ImportedType!.AssemblyIdentity, owner.ImportedType.Namespace, owner.ImportedType.Name);
                owners[i].AddCustomAttribute(new(reference, attribute.Arguments));
            }
        return graph.WriteReferenceImage();
    }
    private static TypeBuilder[] DefineTypes(AssemblyBuilder graph, IReadOnlyList<TypeRow> rows)
    {
        var result = new TypeBuilder[rows.Count];
        for (int i = 0; i < rows.Count; i++)
        {
            var type = rows[i];
            result[i] = type.DeclaringType < 0 ? DefineType(graph, type) : type.IsValueType
                ? type.GenericNames.Length == 0 ? result[type.DeclaringType].AddNestedValueType(type.Name, type.Visibility) : result[type.DeclaringType].AddNestedGenericValueType(type.Name[..type.Name.LastIndexOf('`')], type.GenericNames, type.Visibility)
                : result[type.DeclaringType].AddNestedClass(type.Name, type.Visibility);
        }
        return result;
    }
    private static TypeBuilder DefineType(AssemblyBuilder graph, TypeRow type)
    {
        if (type.GenericNames.Length == 0)
            return type.IsValueType ? graph.AddValueType(type.Namespace, type.Name, type.Visibility)
                : type.IsInterface ? graph.AddInterface(type.Namespace, type.Name, type.Visibility)
                : type.IsStatic ? graph.AddType(type.Namespace, type.Name, type.Visibility)
                : graph.AddClass(type.Namespace, type.Name, type.Visibility);
        var name = type.Name[..type.Name.LastIndexOf('`')];
        return type.IsValueType ? graph.AddGenericValueType(type.Namespace, name, type.GenericNames, type.Visibility)
            : type.IsInterface ? graph.AddGenericInterface(type.Namespace, name, type.GenericNames, type.Visibility)
            : type.IsStatic ? graph.AddGenericType(type.Namespace, name, type.GenericNames, type.Visibility)
            : graph.AddGenericClass(type.Namespace, name, type.GenericNames, type.Visibility);
    }

    private static ImportedTypeReference ImportExternalType(AssemblyBuilder graph, string name, int arity, IEnumerable<AssemblyIdentity> references, HashSet<string> valueTypeReferences, Dictionary<(string Name, int Arity), NativeTypeAlias> nativeTypeAliases)
    {
        if (nativeTypeAliases.TryGetValue((name, arity), out var alias)) return ImportAlias(alias, 0);
        ImportedTypeReference ImportAlias(NativeTypeAlias item, int depth)
        {
            Require(depth < 16 && references.Contains(item.Assembly) && (!item.ValueType || valueTypeReferences.Contains(item.NativeName)), "invalid native type alias scope/category");
            ImportedTypeReference? parent = null;
            if (item.Declaring is { } declaring)
            {
                Require(nativeTypeAliases.TryGetValue((declaring, 0), out var enclosing) && enclosing.Assembly.Equals(item.Assembly), "invalid native declaring binding");
                parent = ImportAlias(enclosing!, depth + 1);
            }
            return graph.ImportTypeIdentity(item.Assembly, item.Namespace, item.Name, item.Arity, item.ValueType, parent);
        }
        foreach (var identity in references)
        {
            var text = JsonSerializer.Serialize(new[] { identity.Name, identity.Version.ToString(), identity.Culture, identity.PublicKeyToken, identity.Flags.ToString(CultureInfo.InvariantCulture) });
            var prefix = ModuleName(text) + ".T_";
            if (!name.StartsWith(prefix, StringComparison.Ordinal)) continue;
            var path = name[prefix.Length..].Split(".N_", StringSplitOptions.None);
            Require(path.Length <= 16, "nested imported depth exceeded");
            var parts = path[0].Split('_'); Require(parts.Length == 2, "invalid imported native type name");
            var top = prefix + path[0];
            var reference = graph.ImportTypeIdentity(identity, Decode(parts[0]), Decode(parts[1]), path.Length == 1 ? arity : 0, valueTypeReferences.Contains(top));
            for (int i = 1; i < path.Length; i++)
            {
                top += ".N_" + path[i];
                reference = graph.ImportTypeIdentity(identity, "", Decode(path[i]), i == path.Length - 1 ? arity : 0, valueTypeReferences.Contains(top), reference);
            }
            return reference;
        }
        throw new InvalidDataException("signature type must be owned or scoped to a declared dependency");
    }
    private static PrimitiveType ReadPrimitive(string? name, bool allowVoid) => name switch
    {
        "String" => PrimitiveType.String,
        "Byte" => PrimitiveType.Byte,
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
