namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable external nominal reference or construction owned by one output assembly.</summary>
public sealed class ImportedTypeReference : IEquatable<ImportedTypeReference>
{
    internal ImportedTypeReference(AssemblyBuilder owner, AssemblyIdentity identity, string ns, string name, int arity, SignatureType[] arguments)
    { Owner = owner; AssemblyIdentity = identity; Namespace = ns; Name = name; GenericArity = arity; TypeArguments = Array.AsReadOnly(arguments); }
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
        foreach (var argument in arguments) argument.ValidateOwner(Owner, 32, 32);
        return new(Owner, AssemblyIdentity, Namespace, Name, GenericArity, (SignatureType[])arguments.Clone());
    }
    internal ImportedTypeReference Substitute(Func<SignatureType, SignatureType> substitute) => TypeArguments.Count == 0 ? this
        : Owner.ImportTypeIdentity(AssemblyIdentity, Namespace, Name, GenericArity).MakeGenericInstance(TypeArguments.Select(substitute).ToArray());
    /// <summary>Compares consumer, external identity and ordered type arguments.</summary>
    public bool Equals(ImportedTypeReference? other) => other is not null && ReferenceEquals(Owner, other.Owner) && AssemblyIdentity.Equals(other.AssemblyIdentity) &&
        Namespace == other.Namespace && Name == other.Name && GenericArity == other.GenericArity && TypeArguments.SequenceEqual(other.TypeArguments);
    /// <summary>Compares reference identity and construction.</summary>
    public override bool Equals(object? obj) => obj is ImportedTypeReference other && Equals(other);
    /// <summary>Gets a hash consistent with equality.</summary>
    public override int GetHashCode()
    { var hash = new HashCode(); hash.Add(Owner); hash.Add(AssemblyIdentity); hash.Add(Namespace); hash.Add(Name); hash.Add(GenericArity); foreach (var argument in TypeArguments) hash.Add(argument); return hash.ToHashCode(); }
    /// <summary>Returns a diagnostic name, not a serialized identity.</summary>
    public override string ToString() => Namespace + "." + Name + (TypeArguments.Count == 0 ? "" : "<" + string.Join(",", TypeArguments) + ">");
}

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<(AssemblyIdentity, string, string), ImportedTypeReference> importedNominalTypes = [];
    /// <summary>Imports a top-level reference class/interface definition for use in signatures.</summary>
    /// <param name="definition">External public reference type; generic definitions must be unconstrained and invariant.</param>
    /// <param name="dependencyCoreLibrary">Explicit matching core contract.</param>
    /// <returns>An interned immutable reference; generic definitions require construction before signature use.</returns>
    /// <exception cref="ArgumentNullException">A required argument is null.</exception>
    /// <exception cref="InvalidDataException">Unsupported declaration, identity/core mismatch, conflicting snapshot or limit.</exception>
    public ImportedTypeReference ImportReference(TypeDefinition definition, AssemblyIdentity dependencyCoreLibrary)
    {
        ArgumentNullException.ThrowIfNull(definition); ArgumentNullException.ThrowIfNull(dependencyCoreLibrary);
        if (!CoreLibrary.Equals(dependencyCoreLibrary) || !definition.CanImportReference || definition.DeclaringType is not null || (definition.Attributes & 7) != 1)
            throw new InvalidDataException("unsupported imported type or core contract");
        var identity = definition.Module.Assembly.Identity;
        if (importedGraphs.TryGetValue(identity, out var prior) && prior.Mvid != definition.Module.Mvid) throw new InvalidDataException("conflicting dependency module snapshots");
        var result = ImportTypeIdentity(identity, definition.Namespace, definition.Name, definition.GenericArity);
        if (!importedGraphs.ContainsKey(identity)) importedGraphs.Add(identity, (definition.Module.Mvid, new AssemblyBuilder(identity, dependencyCoreLibrary)));
        return result;
    }
    internal ImportedTypeReference ImportTypeIdentity(AssemblyIdentity identity, string ns, string name, int arity)
    {
        if (identity.Equals(Identity) || identity.PublicKeyToken.Length != 0 || identity.Flags != 0 || arity is < 0 or > 32 ||
            name.Length == 0 || name == "<Module>" || ns.Length + name.Length > 1024 || (ns + name).Any(char.IsControl) ||
            (arity == 0 ? name.Contains('`') : !name.EndsWith("`" + arity, StringComparison.Ordinal))) throw new InvalidDataException("unsupported imported type identity");
        var key = (identity, ns, name);
        if (importedNominalTypes.TryGetValue(key, out var existing))
        { if (existing.GenericArity != arity) throw new InvalidDataException("conflicting imported type arity"); return existing; }
        if (importedNominalTypes.Count >= 4096 || importedNominalTypes.Values.Select(t => t.AssemblyIdentity).Concat(importedGraphs.Keys).Append(identity).Distinct().Count() > 256)
            throw new InvalidDataException("imported type/reference limit exceeded");
        var reference = new ImportedTypeReference(this, identity, ns, name, arity, []);
        importedNominalTypes.Add(key, reference); return reference;
    }
}
