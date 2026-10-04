using System.Text.Json;

namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<AssemblyIdentity, NativeImportBinding> nativeBindings = [];
    internal NativeImportBinding? NativeBinding { get; private set; }
    internal NativeImportBinding? NativeBindingFor(AssemblyIdentity identity) => nativeBindings.GetValueOrDefault(identity);

    /// <summary>Binds an explicit CLI declaration snapshot to a translated native implementation before importing its members.</summary>
    /// <param name="reference">The exact immutable declaration snapshot used by the host compiler.</param>
    /// <param name="implementation">Native implementation inventory with matching public names, arities and signatures.</param>
    /// <param name="coreLibrary">Explicit matching core contract.</param>
    /// <exception cref="ArgumentNullException">A required argument is null.</exception>
    /// <exception cref="InvalidDataException">Core mismatch, conflicting identity, prior imports or reference limit.</exception>
    /// <remarks>Imports validate selected declarations lazily. Native names follow the translated CLI convention (nested names use dots, generic arity is separate).
    /// This bridge does not replace metadata resolution or infer implementation identity from the host. CLI output retains the original reference identity. Native value ToString overrides require exactly one System binding; writing validates its CLI and native Object.ToString declarations and records the dependency even without a body call.</remarks>
    public void BindNativeLibrary(AssemblyDefinition reference, NativeLibraryDefinition implementation, AssemblyIdentity coreLibrary)
    {
        ArgumentNullException.ThrowIfNull(reference); ArgumentNullException.ThrowIfNull(implementation); ArgumentNullException.ThrowIfNull(coreLibrary);
        if (!CoreLibrary.Equals(coreLibrary) || reference.Identity.Equals(Identity) || importedGraphs.ContainsKey(reference.Identity) ||
            importedNominalTypes.Values.Any(t => t.AssemblyIdentity.Equals(reference.Identity)) || nativeBindings.Count >= 256)
            throw new InvalidDataException("native binding requires a matching core, distinct dependency and no prior imports");
        var binding = new NativeImportBinding(reference, implementation, coreLibrary);
        nativeBindings.Add(reference.Identity, binding);
        importedGraphs.Add(reference.Identity, (reference.ImportSnapshotIdentity, new AssemblyBuilder(reference.Identity, coreLibrary) { NativeBinding = binding }));
    }
}

public sealed partial class MethodBuilder
{
    internal bool IsCoreObjectToString { get; set; }
    internal bool IsCoreObjectHash { get; set; }
    internal bool NativeValueOverride { get; set; }
    internal string? NativeImportName { get; set; }
    internal bool NativeImportCharOwner { get; set; }
    internal PrimitiveType? NativeImportPrimitiveOwner { get; set; }
    internal bool NativeImportIsNamespaceFunction { get; set; }
    internal bool DiscardNativeImportResult { get; set; }
}

internal sealed class NativeImportBinding(AssemblyDefinition reference, NativeLibraryDefinition library, AssemblyIdentity core)
{
    private readonly Dictionary<uint, bool> namespaceContainers = [];
    internal NativeLibraryDefinition Library { get; } = library;
    internal string? Revision => Library.Declarations.TryGetProperty("revision", out var value) ? value.GetString() : null;
    internal static string SimpleName(string name) => name.Split('`')[0];
    internal string TypeName(TypeDefinition type) => type.DeclaringType is { } parent ? TypeName(parent) + "." + SimpleName(type.Name) :
        (type.Namespace.Length == 0 ? "" : type.Namespace + ".") + SimpleName(type.Name);
    internal string TypeName(ImportedTypeReference type) => type.DeclaringType is { } parent ? TypeName(parent) + "." + SimpleName(type.Name) :
        (type.Namespace.Length == 0 ? "" : type.Namespace + ".") + SimpleName(type.Name);
    internal void ValidateBoxingCore()
    {
        var owners = reference.MainModule.Types.Where(t => t.Namespace == "System" && t.Name == "Object" && t.DeclaringType is null).ToArray();
        if (Library.ModuleName != "System" || owners.Length != 1 || owners[0].IsValueType || (owners[0].Attributes & 0x27) != 1)
            throw new InvalidDataException("native boxing requires a public System.Object class in the explicit core binding");
        ValidateType(owners[0]);
    }
    internal void ValidateObjectToStringSlot() => ValidateObjectSlot("ToString", "String", 0x0e);
    internal void ValidateObjectHashSlot() => ValidateObjectSlot("GetHashCode", "Int32", 0x08);
    private void ValidateObjectSlot(string name, string result, byte signatureResult)
    {
        if (reference.IsNative) throw new InvalidDataException("Object bootstrap binding requires an explicit CLI declaration snapshot");
        var owners = reference.MainModule.Types.Where(t => t.Namespace == "System" && t.Name == "Object" && t.DeclaringType is null).Take(2).ToArray();
        var owner = owners.Length == 1 ? owners[0] : null;
        // Each bounded slot has its exact canonical CLI instance result signature.
        var slots = owner?.Methods.Where(m => m.Name == name && !m.IsStatic && m.GenericArity == 0 && m.AuthoredSignature is null &&
            m.GetSignature().AsSpan().SequenceEqual(new byte[] { 0x20, 0, signatureResult })).Take(2).ToArray() ?? [];
        var slot = slots.Length == 1 ? slots[0] : null;
        if (Library.ModuleName != "System" || owner is null || owner.IsValueType || slot is null || (slot.Attributes & 0x447) != 0x46)
            throw new InvalidDataException("native Object override requires a public virtual System.Object." + name + " declaration");
        ValidateType(owner);
        var matches = Library.Declarations.GetProperty("functions").EnumerateArray().Where(f =>
            f.GetProperty("name").GetString() == "System.Object." + name &&
            f.TryGetProperty("owner", out var target) && TypeKey(target) == "Named(System.Object)" &&
            Flag(f, "instance") && Count(f, "parameters") == 0).ToArray();
        if (matches.Length != 1) throw new InvalidDataException("missing or ambiguous native Object." + name + " slot");
        var method = matches[0];
        if (!NativeLibraryDefinition.IsPublic(method) || !Flag(method, "is_virtual") || Flag(method, "is_abstract") ||
            Flag(method, "receiver_byref") || Flag(method, "no_result") || Count(method, "generic_parameters") != 0 ||
            TypeKey(method.GetProperty("returns")) != result || Indices(method, "out_parameters").Any() || Indices(method, "out_when_true").Any() || Indices(method, "readonly_parameters").Any())
            throw new InvalidDataException("incompatible native Object." + name + " slot");
    }

    internal void ValidateType(TypeDefinition type, bool intrinsicStringOwner = false)
    {
        if (type.Module.Assembly.ImportSnapshotIdentity != reference.ImportSnapshotIdentity) throw new InvalidDataException("native binding snapshot mismatch");
        var name = TypeName(type);
        var matches = Library.Declarations.GetProperty("types").EnumerateArray().Where(t =>
            t.GetProperty("name").GetString() == name && Count(t, "generic_parameters") == type.GenericArity).Take(2).ToArray();
        if (matches.Length != 1) throw new InvalidDataException("native type missing or ambiguous: " + name);
        var native = matches[0];
        if (!NativeLibraryDefinition.IsPublic(native)) throw new InvalidDataException("native type is not public: " + name);
        var contract = (type.Attributes & 0x20) != 0;
        var nativeInterface = native.TryGetProperty("representation", out var kind) && kind.GetString() == "Interface";
        if (contract != nativeInterface || !contract && !intrinsicStringOwner && (type.Attributes & 0x180) != 0x180 && type.IsValueType == Flag(native, "is_reference_type"))
            throw new InvalidDataException("native type category mismatch: " + name);
    }
    internal void ValidateMethod(MethodDefinition definition, MethodBuilder target)
    {
        if (definition.DeclaringType is not { } owner) throw new InvalidDataException("translated free-function binding is not yet supported");
        if (owner.Module.Assembly.ImportSnapshotIdentity != reference.ImportSnapshotIdentity) throw new InvalidDataException("native binding snapshot mismatch");
        if (!namespaceContainers.TryGetValue(owner.MetadataToken, out var namespaceContainer))
        {
            namespaceContainer = owner.DeclaringType is null && owner.GenericArity == 0 && (owner.Attributes & 0x1a7) == 0x181 &&
                reference.HasCoreTopLevelMarker(owner.MetadataToken, core);
            namespaceContainers.Add(owner.MetadataToken, namespaceContainer);
        }
        if (namespaceContainer && (!definition.IsStatic || (definition.Attributes & 7) != 6 || (definition.Attributes & 0x440) != 0))
            throw new InvalidDataException("namespace function must be public static and concrete");
        // Explicit primitive bootstrap members use intrinsic receiver storage, not
        // nominal layouts. Validate every signature and receiver mode against the seed;
        // this does not admit primitive owners as ordinary imported nominal types.
        var intrinsicPrimitiveOwner = reference.Identity.Equals(core) &&
            Library.ModuleName == "System" && owner is { Namespace: "System", Name: "String" or "Int32" or "Int64", GenericArity: 0, DeclaringType: null } &&
            owner.IsValueType == (owner.Name is "Int32" or "Int64");
        if (!namespaceContainer) ValidateType(owner, intrinsicPrimitiveOwner && owner.Name == "String");
        var name = (namespaceContainer ? owner.Namespace : TypeName(owner));
        name = (name.Length == 0 ? "" : name + ".") + definition.Name;
        var parameters = target.Signature.ParameterTypes.Select(TypeKey).ToArray();
        var ownerName = TypeName(owner);
        var intrinsicCharOwner = reference.Identity.Equals(core) && Library.ModuleName == "System" &&
            owner is { Namespace: "System", Name: "Char", IsValueType: true, GenericArity: 0, DeclaringType: null };
        var ownerKey = intrinsicCharOwner ? "Char" : intrinsicPrimitiveOwner ? owner.Name : owner.GenericArity == 0 ? "Named(" + ownerName + ")" : "Constructed(" + ownerName + ";" + string.Join(",", Enumerable.Range(0, owner.GenericArity).Select(i => "TypeParameter(" + i + ")")) + ")";
        var matches = Library.Declarations.GetProperty("functions").EnumerateArray().Where(f =>
            f.GetProperty("name").GetString() == name && f.TryGetProperty("owner", out var nativeOwner) && (namespaceContainer ? nativeOwner.ValueKind == JsonValueKind.Null : TypeKey(nativeOwner) == ownerKey) && Flag(f, "instance") == !definition.IsStatic &&
            Count(f, "generic_parameters") == definition.GenericArity &&
            f.GetProperty("parameters").EnumerateArray().Select(TypeKey).SequenceEqual(parameters)).Take(2).ToArray();
        if (matches.Length != 1) throw new InvalidDataException("native method missing or ambiguous: " + name);
        var function = matches[0];
        if ((intrinsicPrimitiveOwner || intrinsicCharOwner) && (Flag(function, "is_virtual") || Flag(function, "is_abstract")))
            throw new InvalidDataException("primitive bootstrap member must be concrete and nonvirtual: " + name);
        if (!NativeLibraryDefinition.IsPublic(function) || (Flag(function, "is_abstract") || (owner.Attributes & 0x20) != 0) != ((definition.Attributes & 0x400) != 0) || (owner.Attributes & 0x20) != 0 && Count(function, "body") != 0)
            throw new InvalidDataException("native method visibility/body contract mismatch: " + name);
        if (TypeKey(function.GetProperty("returns")) != TypeKey(target.Signature.ReturnType) ||
            Flag(function, "receiver_byref") != (!definition.IsStatic && owner.IsValueType) ||
            Flag(function, "no_result") && target.Signature.ReturnType.Primitive != PrimitiveType.Void)
            throw new InvalidDataException("native method result/receiver mismatch: " + name);
        var actualOutputs = Indices(function, "out_parameters").Concat(Indices(function, "out_when_true")).Distinct().Order();
        if (!actualOutputs.SequenceEqual(target.Signature.OutParameters.Order())) throw new InvalidDataException("native output contract mismatch: " + name);
        target.NativeImportName = name;
        target.NativeImportCharOwner = intrinsicCharOwner;
        target.NativeImportPrimitiveOwner = intrinsicPrimitiveOwner ? owner.Name switch { "String" => PrimitiveType.String, "Int64" => PrimitiveType.Int64, _ => PrimitiveType.Int32 } : null;
        target.NativeImportIsNamespaceFunction = namespaceContainer;
        target.DiscardNativeImportResult = target.Signature.ReturnType.Primitive == PrimitiveType.Void && !Flag(function, "no_result");
    }
    internal static bool IsInhabitedVoid(ImportedTypeReference type) =>
        type.AssemblyIdentity.Equals(type.Owner.CoreLibrary) && type.Owner.NativeBindingFor(type.AssemblyIdentity) is not null &&
        type.DeclaringType is null && type.Namespace == "System" && type.Name == "Void" && type.IsValueType && type.GenericArity == 0;
    internal static bool IsIntrinsicChar(ImportedTypeReference type) => type.Owner.IsNativeGrapheme(type) ||
        type.AssemblyIdentity.Equals(type.Owner.CoreLibrary) && type.Owner.NativeBindingFor(type.AssemblyIdentity)?.Library.ModuleName == "System" &&
        type.DeclaringType is null && type.Namespace == "System" && type.Name == "Char" && type.IsValueType && type.GenericArity == 0;
    internal static bool IsErasedValue(ImportedTypeReference type) =>
        type.AssemblyIdentity.Equals(type.Owner.CoreLibrary) && type.Owner.NativeBindingFor(type.AssemblyIdentity)?.Library.ModuleName == "System" &&
        type.DeclaringType is null && type.Namespace == "System" && type.Name == "Value" && type.IsValueType && type.GenericArity == 0;
    private string TypeKey(SignatureType type)
    {
        if (type.Primitive is { } primitive) return primitive.ToString();
        if (type.ByReferenceElement is { } byref) return "ByRef(" + TypeKey(byref) + ")";
        if (type.ArrayElement is { } element) return "ArrayRef(" + TypeKey(element) + ")";
        if (type.MethodParameterIndex is { } method) return "MethodTypeParameter(" + method + ")";
        if (type.TypeParameterIndex is { } owner) return "TypeParameter(" + owner + ")";
        if (type.FunctionSignature is { } function) return "Function(" + string.Join(",", function.ParameterTypes.Select(TypeKey)) + ")->" + TypeKey(function.ReturnType) + ":" + function.NoResult;
        if (type.ImportedType is { } imported)
        {
            if (IsInhabitedVoid(imported)) return "Void";
            if (IsIntrinsicChar(imported)) return "Char";
            if (IsErasedValue(imported)) return "Value";
            var binding = imported.Owner.NativeBindingFor(imported.AssemblyIdentity) ?? throw new InvalidDataException("native signature requires an explicit dependency binding");
            var name = binding.TypeName(imported);
            return imported.TypeArguments.Count == 0 ? "Named(" + name + ")" : "Constructed(" + name + ";" + string.Join(",", imported.TypeArguments.Select(TypeKey)) + ")";
        }
        throw new InvalidDataException("unsupported native binding signature");
    }
    private static string TypeKey(JsonElement type)
    {
        if (type.ValueKind == JsonValueKind.String) return type.GetString()!;
        if (type.TryGetProperty("Function", out var function))
        {
            if (Count(function, "out_parameters") != 0 || Count(function, "out_when_true") != 0) throw new InvalidDataException("function out contracts not supported by binding");
            return "Function(" + string.Join(",", function.GetProperty("parameters").EnumerateArray().Select(TypeKey)) + ")->" + TypeKey(function.GetProperty("returns")) + ":" + Flag(function, "no_result");
        }
        if (type.TryGetProperty("Constructed", out var constructed)) return "Constructed(" + constructed.GetProperty("definition").GetString() + ";" + string.Join(",", constructed.GetProperty("arguments").EnumerateArray().Select(TypeKey)) + ")";
        foreach (var name in new[] { "ByRef", "ArrayRef" }) if (type.TryGetProperty(name, out var nested)) return name + "(" + TypeKey(nested) + ")";
        foreach (var name in new[] { "Named", "TypeParameter", "MethodTypeParameter" }) if (type.TryGetProperty(name, out var atom)) return name + "(" + atom.ToString() + ")";
        throw new InvalidDataException("unsupported native binding signature kind");
    }
    private static bool Flag(JsonElement value, string name) => value.TryGetProperty(name, out var flag) && flag.ValueKind == JsonValueKind.True;
    private static int Count(JsonElement value, string name) => value.TryGetProperty(name, out var list) ? list.GetArrayLength() : 0;
    private static IEnumerable<int> Indices(JsonElement value, string name) => value.TryGetProperty(name, out var list) ? list.EnumerateArray().Select(i => i.GetInt32()) : [];
}
