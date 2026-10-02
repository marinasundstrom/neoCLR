namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private readonly List<ImportedFieldReference> authoredFields = [];
    /// <summary>Authors a native instance-field contract from explicit semantic and layout values.</summary>
    /// <param name="declaringType">Owned top-level root reference-class definition, not a construction.</param>
    /// <param name="name">Nonempty metadata field name.</param>
    /// <param name="fieldType">Primitive, scoped owner parameter, external nominal construction or single-vector storage type.</param>
    /// <param name="instanceStorageOrdinal">Zero-based native field slot in the selected declaring artifact, including private fields.</param>
    /// <param name="isReadOnly">Whether external stores are forbidden.</param>
    /// <returns>An interned output-owned field reference.</returns>
    /// <exception cref="ArgumentNullException">An argument is null.</exception>
    /// <exception cref="ArgumentException">Invalid owner/name/ordinal/type or foreign signature.</exception>
    /// <exception cref="InvalidDataException">Conflicting name/slot contract or resource limit.</exception>
    /// <remarks>The caller asserts public instance access and correct layout for the owner's registered artifact.
    /// No image is loaded or checked. Native output uses the slot; CLI output uses name/type MemberRef.
    /// Static, inherited, byref and value-type field profiles are unsupported.</remarks>
    public ImportedFieldReference CreateFieldReference(ImportedTypeReference declaringType, string name,
        SignatureType fieldType, int instanceStorageOrdinal, bool isReadOnly = false)
    {
        ArgumentNullException.ThrowIfNull(declaringType); ArgumentNullException.ThrowIfNull(name); ArgumentNullException.ThrowIfNull(fieldType);
        if (!ReferenceEquals(declaringType.Owner, this) || declaringType.TypeArguments.Count != 0 || declaringType.DeclaringType is not null ||
            declaringType.IsValueType || !importedGraphs.TryGetValue(declaringType.AssemblyIdentity, out var graph) || !graph.Snapshot.StartsWith("native:", StringComparison.Ordinal) ||
            name.Length == 0 || name.Length > 1024 || name.Any(char.IsControl) || instanceStorageOrdinal < 0 || !Supported(fieldType))
            throw new ArgumentException("unsupported native field contract");
        fieldType.ValidateOwner(this, typeArity: declaringType.GenericArity);
        foreach (var existing in authoredFields.Concat(importedFields.Values))
        {
            if (!existing.DeclaringType.Equals(declaringType) || existing.Name != name && existing.NativeIndex != instanceStorageOrdinal) continue;
            if (existing.Name != name || existing.NativeIndex != instanceStorageOrdinal || existing.FieldType != fieldType || existing.IsReadOnly != isReadOnly)
                throw new InvalidDataException("conflicting native field contract");
            return existing;
        }
        if (authoredFields.Count + importedFields.Count >= 4096) throw new InvalidDataException("too many imported fields");
        var reference = new ImportedFieldReference(this, declaringType, name, fieldType, instanceStorageOrdinal, isReadOnly);
        authoredFields.Add(reference); return reference;

        static bool Supported(SignatureType type) => type.Primitive is { } primitive ? primitive != PrimitiveType.Void :
            type.TypeParameterIndex is not null ||
            type.ImportedType is { IsValueType: false, DeclaringType: null } imported &&
            imported.TypeArguments.Count == imported.GenericArity && imported.TypeArguments.All(Supported) ||
            type.ArrayElement is { ArrayElement: null } element && Supported(element);
    }
}
