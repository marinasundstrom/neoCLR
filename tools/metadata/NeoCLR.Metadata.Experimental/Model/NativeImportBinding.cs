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
    /// This bridge does not replace metadata resolution or infer implementation identity from the host. CLI output retains the original reference identity.</remarks>
    public void BindNativeLibrary(AssemblyDefinition reference, NativeLibraryDefinition implementation, AssemblyIdentity coreLibrary)
    {
        ArgumentNullException.ThrowIfNull(reference); ArgumentNullException.ThrowIfNull(implementation); ArgumentNullException.ThrowIfNull(coreLibrary);
        if (!CoreLibrary.Equals(coreLibrary) || reference.Identity.Equals(Identity) || importedGraphs.ContainsKey(reference.Identity) ||
            importedNominalTypes.Values.Any(t => t.AssemblyIdentity.Equals(reference.Identity)) || nativeBindings.Count >= 256)
            throw new InvalidDataException("native binding requires a matching core, distinct dependency and no prior imports");
        var binding = new NativeImportBinding(reference, implementation);
        nativeBindings.Add(reference.Identity, binding);
        importedGraphs.Add(reference.Identity, (reference.MainModule.Mvid, new AssemblyBuilder(reference.Identity, coreLibrary) { NativeBinding = binding }));
    }
}

public sealed partial class MethodBuilder
{
    internal string? NativeImportName { get; set; }
    internal bool DiscardNativeImportResult { get; set; }
}

internal sealed class NativeImportBinding(AssemblyDefinition reference, NativeLibraryDefinition library)
{
    internal NativeLibraryDefinition Library { get; } = library;
    internal string? Revision => Library.Declarations.TryGetProperty("revision", out var value) ? value.GetString() : null;
    internal static string SimpleName(string name) => name.Split('`')[0];
    internal string TypeName(TypeDefinition type) => type.DeclaringType is { } parent ? TypeName(parent) + "." + SimpleName(type.Name) :
        (type.Namespace.Length == 0 ? "" : type.Namespace + ".") + SimpleName(type.Name);
    internal string TypeName(ImportedTypeReference type) => type.DeclaringType is { } parent ? TypeName(parent) + "." + SimpleName(type.Name) :
        (type.Namespace.Length == 0 ? "" : type.Namespace + ".") + SimpleName(type.Name);
    internal void ValidateType(TypeDefinition type)
    {
        if (type.Module.Mvid != reference.MainModule.Mvid) throw new InvalidDataException("native binding snapshot mismatch");
        var name = TypeName(type);
        var matches = Library.Declarations.GetProperty("types").EnumerateArray().Where(t =>
            t.GetProperty("name").GetString() == name && Count(t, "generic_parameters") == type.GenericArity).Take(2).ToArray();
        if (matches.Length != 1) throw new InvalidDataException("native type missing or ambiguous: " + name);
        var native = matches[0];
        if (!NativeLibraryDefinition.IsPublic(native)) throw new InvalidDataException("native type is not public: " + name);
        var contract = (type.Attributes & 0x20) != 0;
        var nativeInterface = native.TryGetProperty("representation", out var kind) && kind.GetString() == "Interface";
        if (contract != nativeInterface || !contract && (type.Attributes & 0x180) != 0x180 && type.IsValueType == Flag(native, "is_reference_type"))
            throw new InvalidDataException("native type category mismatch: " + name);
    }
    internal void ValidateMethod(MethodDefinition definition, MethodBuilder target)
    {
        if (definition.DeclaringType is not { } owner) throw new InvalidDataException("translated free-function binding is not yet supported");
        ValidateType(owner);
        var name = TypeName(owner) + "." + definition.Name;
        var parameters = target.Signature.ParameterTypes.Select(TypeKey).ToArray();
        var ownerName = TypeName(owner);
        var ownerKey = owner.GenericArity == 0 ? "Named(" + ownerName + ")" : "Constructed(" + ownerName + ";" + string.Join(",", Enumerable.Range(0, owner.GenericArity).Select(i => "TypeParameter(" + i + ")")) + ")";
        var matches = Library.Declarations.GetProperty("functions").EnumerateArray().Where(f =>
            f.GetProperty("name").GetString() == name && f.TryGetProperty("owner", out var nativeOwner) && TypeKey(nativeOwner) == ownerKey && Flag(f, "instance") == !definition.IsStatic &&
            Count(f, "generic_parameters") == definition.GenericArity &&
            f.GetProperty("parameters").EnumerateArray().Select(TypeKey).SequenceEqual(parameters)).Take(2).ToArray();
        if (matches.Length != 1) throw new InvalidDataException("native method missing or ambiguous: " + name);
        var function = matches[0];
        if (!NativeLibraryDefinition.IsPublic(function) || (Flag(function, "is_abstract") || (owner.Attributes & 0x20) != 0) != ((definition.Attributes & 0x400) != 0) || (owner.Attributes & 0x20) != 0 && Count(function, "body") != 0)
            throw new InvalidDataException("native method visibility/body contract mismatch: " + name);
        if (TypeKey(function.GetProperty("returns")) != TypeKey(target.Signature.ReturnType) ||
            Flag(function, "receiver_byref") != (!definition.IsStatic && owner.IsValueType) ||
            Flag(function, "no_result") && target.Signature.ReturnType.Primitive != PrimitiveType.Void)
            throw new InvalidDataException("native method result/receiver mismatch: " + name);
        var actualOutputs = Indices(function, "out_parameters").Concat(Indices(function, "out_when_true")).Distinct().Order();
        if (!actualOutputs.SequenceEqual(target.Signature.OutParameters.Order())) throw new InvalidDataException("native output contract mismatch: " + name);
        target.NativeImportName = name;
        target.DiscardNativeImportResult = target.Signature.ReturnType.Primitive == PrimitiveType.Void && !Flag(function, "no_result");
    }
    internal static bool IsInhabitedVoid(ImportedTypeReference type) =>
        type.AssemblyIdentity.Equals(type.Owner.CoreLibrary) && type.Owner.NativeBindingFor(type.AssemblyIdentity) is not null &&
        type.DeclaringType is null && type.Namespace == "System" && type.Name == "Void" && type.IsValueType && type.GenericArity == 0;
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
