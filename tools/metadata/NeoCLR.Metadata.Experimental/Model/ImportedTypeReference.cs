namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable external nominal reference or construction owned by one output assembly.</summary>
public sealed class ImportedTypeReference : IEquatable<ImportedTypeReference>
{
    internal ImportedTypeReference(AssemblyBuilder owner, AssemblyIdentity identity, string ns, string name, int arity, SignatureType[] arguments, bool isValueType = false, ImportedTypeReference? declaringType = null)
    { DeclaringType = declaringType; IsValueType = isValueType; Owner = owner; AssemblyIdentity = identity; Namespace = ns; Name = name; GenericArity = arity; TypeArguments = Array.AsReadOnly(arguments); }
    /// <summary>Gets the enclosing definition reference, or null for a top-level identity.</summary>
    public ImportedTypeReference? DeclaringType { get; }
    /// <summary>Gets whether signatures encode this declaration as a value type.</summary>
    public bool IsValueType { get; }
    /// <summary>Gets the consuming assembly.</summary>
    public AssemblyBuilder Owner { get; }
    /// <summary>Gets the exact external assembly identity.</summary>
    public AssemblyIdentity AssemblyIdentity { get; }
    /// <summary>Gets the namespace.</summary>
    public string Namespace { get; }
    /// <summary>Gets the metadata name, including generic arity suffix.</summary>
    public string Name { get; }
    /// <summary>Gets the definition's generic arity.</summary>
    public int GenericArity { get; }
    /// <summary>Gets copied construction arguments, or an empty list for a definition reference.</summary>
    public IReadOnlyList<SignatureType> TypeArguments { get; }
    /// <summary>Constructs this generic definition with types belonging to the consumer.</summary>
    /// <param name="arguments">Exactly one non-Void type per parameter; copied.</param>
    /// <returns>An immutable constructed reference.</returns>
    /// <exception cref="ArgumentNullException">Arguments are null.</exception>
    /// <exception cref="ArgumentException">Wrong arity, already constructed, unsupported scope or excessive depth.</exception>
    public ImportedTypeReference MakeGenericInstance(params SignatureType[] arguments)
    {
        ArgumentNullException.ThrowIfNull(arguments);
        if (GenericArity == 0 || TypeArguments.Count != 0 || arguments.Length != GenericArity ||
            arguments.Any(t => t is null || t.Primitive == PrimitiveType.Void || t.NestingDepth >= 16)) throw new ArgumentException("invalid imported type construction", nameof(arguments));
        foreach (var argument in arguments) argument.ValidateOwner(Owner, 32, 32, allowSelf: true);
        return new(Owner, AssemblyIdentity, Namespace, Name, GenericArity, (SignatureType[])arguments.Clone(), IsValueType, DeclaringType);
    }
    internal ImportedTypeReference Substitute(Func<SignatureType, SignatureType> substitute) => TypeArguments.Count == 0 ? this
        : Owner.ImportTypeIdentity(AssemblyIdentity, Namespace, Name, GenericArity, IsValueType, DeclaringType).MakeGenericInstance(TypeArguments.Select(substitute).ToArray());
    /// <summary>Compares consumer, external identity and ordered type arguments.</summary>
    public bool Equals(ImportedTypeReference? other) => other is not null && ReferenceEquals(Owner, other.Owner) && AssemblyIdentity.Equals(other.AssemblyIdentity) &&
        Equals(DeclaringType, other.DeclaringType) && Namespace == other.Namespace && Name == other.Name && GenericArity == other.GenericArity && IsValueType == other.IsValueType && TypeArguments.SequenceEqual(other.TypeArguments);
    /// <summary>Compares reference identity and construction.</summary>
    public override bool Equals(object? obj) => obj is ImportedTypeReference other && Equals(other);
    /// <summary>Gets a hash consistent with equality.</summary>
    public override int GetHashCode()
    { var hash = new HashCode(); hash.Add(DeclaringType); hash.Add(Owner); hash.Add(AssemblyIdentity); hash.Add(Namespace); hash.Add(Name); hash.Add(GenericArity); hash.Add(IsValueType); foreach (var argument in TypeArguments) hash.Add(argument); return hash.ToHashCode(); }
    /// <summary>Returns a diagnostic name, not a serialized identity.</summary>
    public override string ToString() => (DeclaringType is null ? Namespace + "." : DeclaringType + "+") + Name + (TypeArguments.Count == 0 ? "" : "<" + string.Join(",", TypeArguments) + ">");
}

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<(AssemblyIdentity, string, string, ImportedTypeReference?), ImportedTypeReference> importedNominalTypes = [];
    private readonly Dictionary<ImportedTypeReference, TypeDefinition> nativeImportedDefinitions = [];
    private readonly Dictionary<(ImportedTypeReference, ImportedTypeReference), bool> nativeInterfaceConversions = [];
    internal bool HasNativeInterfaceConversion(ImportedTypeReference actual, ImportedTypeReference target)
    {
        if (IsAuthoredInterface(target) && HasAuthoredInterfaceConversion(actual, target)) return true;
        if (!nativeImportedDefinitions.ContainsKey(InterfaceDefinition(actual)) || !nativeImportedDefinitions.TryGetValue(InterfaceDefinition(target), out var destination) ||
            (destination.Attributes & 0x20) == 0) return false;
        if (nativeInterfaceConversions.TryGetValue((actual, target), out var known)) return known;
        var seen = new HashSet<ImportedTypeReference>();
        bool Visit(ImportedTypeReference current)
        {
            if (!seen.Add(current)) return false;
            if (current.Equals(target)) return true;
            if (seen.Count > 4096) throw new InvalidDataException("interface conversion graph exceeds limit");
            if (!nativeImportedDefinitions.TryGetValue(InterfaceDefinition(current), out var definition)) return false;
            SignatureType Substitute(SignatureType type) => type.TypeParameterIndex is { } index && current.TypeArguments.Count != 0
                ? current.TypeArguments[index] : type.ArrayElement is { } element ? SignatureType.ArrayOf(Substitute(element))
                : type.ImportedType is { } imported ? imported.Substitute(Substitute) : type;
            foreach (var relationship in definition.Interfaces)
            {
                var parent = ImportReference(relationship.InterfaceType.Resolve(), CoreLibrary);
                if (relationship.TypeArguments.Count != 0)
                    parent = parent.MakeGenericInstance(relationship.TypeArguments.Select(t => Substitute(ImportNativeSignatureType(t, CoreLibrary, null))).ToArray());
                if (Visit(parent)) return true;
            }
            return false;
        }
        return nativeInterfaceConversions[(actual, target)] = Visit(actual);
    }
    internal SignatureType ImportNativeSignatureType(SignatureType type, AssemblyIdentity core, IAssemblyResolver? resolver)
        => type.ByReferenceElement is { } target ? SignatureType.ByReference(ImportNativeSignatureType(target, core, resolver))
            : type.ReferencedGenericInstance is { } constructed
            ? ImportReference(constructed.Definition.Resolve(resolver), core).MakeGenericInstance(constructed.TypeArguments.Select(t => ImportNativeSignatureType(t, core, resolver)).ToArray())
            : type.TypeParameterIndex is { } typeParameter ? SignatureType.TypeParameter(typeParameter)
            : type.MethodParameterIndex is { } parameter ? SignatureType.MethodParameter(parameter)
            : type.ArrayElement is { } element ? SignatureType.ArrayOf(ImportNativeSignatureType(element, core, resolver))
            : type.Primitive is { } primitive ? primitive
            : type.ReferencedType is { } reference ? ImportNativeSignatureReference(reference, core, resolver)
            : throw new InvalidDataException("unsupported native signature type");
    internal ImportedTypeReference ImportNativeSignatureReference(TypeReference reference, AssemblyIdentity core, IAssemblyResolver? resolver)
    {
        var definition = reference.Resolve(resolver);
        if (definition.GenericArity != 0)
            throw new InvalidDataException("native nominal signature requires a nongeneric class, value or interface");
        return ImportReference(definition, core);
    }
    /// <summary>Imports a public class, interface or value-type definition for use in signatures.</summary>
    /// <param name="definition">External public type; generic definitions must be unconstrained and invariant. Values must extend the explicit core System.ValueType; enums are not admitted.</param>
    /// <param name="dependencyCoreLibrary">Explicit matching core contract.</param>
    /// <returns>An interned immutable reference; generic definitions require construction before signature use.</returns>
    /// <exception cref="ArgumentNullException">A required argument is null.</exception>
    /// <exception cref="InvalidDataException">Unsupported declaration, identity/core mismatch, conflicting snapshot or limit.</exception>
    public ImportedTypeReference ImportReference(TypeDefinition definition, AssemblyIdentity dependencyCoreLibrary)
    {
        ArgumentNullException.ThrowIfNull(definition); ArgumentNullException.ThrowIfNull(dependencyCoreLibrary);
        if (!CoreLibrary.Equals(dependencyCoreLibrary) || !definition.CanImportReference || definition.IsValueType && !definition.Module.Assembly.IsNative && !Equals(definition.ValueTypeCore, dependencyCoreLibrary) || (definition.Attributes & 7) != (definition.DeclaringType is null ? 1u : 2u))
            throw new InvalidDataException("unsupported imported type or core contract: " + definition.Namespace + "." + definition.Name + " (value core " + definition.ValueTypeCore?.Name + ", expected " + dependencyCoreLibrary.Name + ")");
        var identity = definition.Module.Assembly.Identity;
        NativeBindingFor(identity)?.ValidateType(definition);
        if (importedGraphs.TryGetValue(identity, out var prior) && prior.Snapshot != definition.Module.Assembly.ImportSnapshotIdentity) throw new InvalidDataException("conflicting dependency module snapshots");
        var result = ImportTypeIdentity(identity, definition.Namespace, definition.Name, definition.GenericArity, definition.IsValueType, ImportDeclaringScope(definition.DeclaringType, dependencyCoreLibrary));
        if (!importedGraphs.ContainsKey(identity)) importedGraphs.Add(identity, (definition.Module.Assembly.ImportSnapshotIdentity, new AssemblyBuilder(identity, dependencyCoreLibrary)));
        RegisterNominalKind(result, (definition.Attributes & 0x20) != 0);
        if (definition.Module.Assembly.IsNative) nativeImportedDefinitions[result] = definition;
        return result;
    }
    private ImportedTypeReference? ImportDeclaringScope(TypeDefinition? type, AssemblyIdentity core, int depth = 0)
    {
        if (type is null) return null;
        if (depth >= 16 || type.GenericArity != 0 || (type.Attributes & 7) != (type.DeclaringType is null ? 1u : 2u) ||
            type.IsValueType && !type.Module.Assembly.IsNative && !Equals(type.ValueTypeCore, core)) throw new InvalidDataException("unsupported enclosing imported type");
        return ImportTypeIdentity(type.Module.Assembly.Identity, type.Namespace, type.Name, 0, type.IsValueType, ImportDeclaringScope(type.DeclaringType, core, depth + 1));
    }
    internal ImportedTypeReference ImportTypeIdentity(AssemblyIdentity identity, string ns, string name, int arity, bool isValueType = false, ImportedTypeReference? declaringType = null)
    {
        if (identity.Equals(Identity) || identity.PublicKeyToken.Length != 0 || identity.Flags != 0 || arity is < 0 or > 32 ||
            name.Length == 0 || name == "<Module>" || ns.Length + name.Length > 1024 || (ns + name).Any(char.IsControl) ||
            (arity == 0 ? name.Contains('`') : !name.EndsWith("`" + arity, StringComparison.Ordinal))) throw new InvalidDataException("unsupported imported type identity");
        if (declaringType is not null && (!ReferenceEquals(declaringType.Owner, this) || !declaringType.AssemblyIdentity.Equals(identity) ||
            declaringType.GenericArity != 0 || declaringType.TypeArguments.Count != 0 || ns.Length != 0))
            throw new InvalidDataException("unsupported nested imported scope");
        var key = (identity, ns, name, declaringType);
        if (importedNominalTypes.TryGetValue(key, out var existing))
        { if (existing.GenericArity != arity || existing.IsValueType != isValueType) throw new InvalidDataException("conflicting imported type arity"); return existing; }
        if (importedNominalTypes.Count >= 4096 || importedNominalTypes.Values.Select(t => t.AssemblyIdentity).Concat(importedGraphs.Keys).Append(identity).Distinct().Count() > 256)
            throw new InvalidDataException("imported type/reference limit exceeded");
        var reference = new ImportedTypeReference(this, identity, ns, name, arity, [], isValueType, declaringType);
        importedNominalTypes.Add(key, reference); return reference;
    }
}
