namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable primitive, nominal or vector instance-field reference owned by one output assembly.</summary>
public sealed partial class ImportedFieldReference
{
    internal ImportedFieldReference(AssemblyBuilder owner, ImportedTypeReference declaringType, FieldDefinition definition, SignatureType type, int index)
    { Owner = owner; DeclaringType = declaringType; Name = definition.Name; FieldType = type; IsReadOnly = (definition.Attributes & 0x20) != 0; NativeIndex = definition.Module.Assembly.IsNative ? index : null; }
    internal ImportedFieldReference(AssemblyBuilder owner, ImportedTypeReference declaringType, string name, SignatureType type, int index, bool readOnly)
    { Owner = owner; DeclaringType = declaringType; Name = name; FieldType = type; NativeIndex = index; IsReadOnly = readOnly; }
    /// <summary>Gets the consuming builder.</summary>
    public AssemblyBuilder Owner { get; }
    /// <summary>Gets the exact external declaring class.</summary>
    public ImportedTypeReference DeclaringType { get; }
    /// <summary>Gets the metadata field name.</summary>
    public string Name { get; }
    /// <summary>Gets the output-owned primitive, imported nominal or vector storage signature.</summary>
    public SignatureType FieldType { get; }
    /// <summary>Gets whether stores from this external consumer are forbidden.</summary>
    public bool IsReadOnly { get; }
    internal int? NativeIndex { get; }
}

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<(AssemblyIdentity, uint), ImportedFieldReference> importedFields = [];
    /// <summary>Imports a public primitive or native nominal-class/vector instance field on a public nongeneric top-level reference class.</summary>
    /// <param name="definition">Immutable CLI or native field definition.</param>
    /// <param name="dependencyCoreLibrary">Explicit core identity, matching this output.</param>
    /// <returns>An interned reference owned by this builder.</returns>
    /// <exception cref="ArgumentNullException">A required argument is null.</exception>
    /// <exception cref="InvalidDataException">Unsupported owner/signature/access/layout, incompatible core/snapshot, translated binding or limit.</exception>
    /// <remarks>CLI output uses a MemberRef. Native output requires a native definition snapshot and uses its validated field ordinal. CLI input fields support CLI output only. Static fields, constructed owners and translated layouts are not admitted. Readonly loads are supported; stores fail body validation.</remarks>
    public ImportedFieldReference ImportReference(FieldDefinition definition, AssemblyIdentity dependencyCoreLibrary)
        => ImportReference(definition, dependencyCoreLibrary, null);

    /// <summary>Imports a native contract with explicitly resolved cross-assembly nominal signature types.</summary>
    /// <param name="definition">Immutable method or field definition.</param>
    /// <param name="dependencyCoreLibrary">Matching explicit core contract for all resolved dependencies.</param>
    /// <param name="resolver">Exact identity resolver; null permits local signature types only.</param>
    /// <returns>An output-owned immutable reference.</returns>
    /// <exception cref="InvalidDataException">A dependency/type is missing, mismatched, unsupported or conflicts with an imported snapshot.</exception>
    public ImportedFieldReference ImportReference(FieldDefinition definition, AssemblyIdentity dependencyCoreLibrary, IAssemblyResolver? resolver)
    {
        ArgumentNullException.ThrowIfNull(definition); ArgumentNullException.ThrowIfNull(dependencyCoreLibrary);
        if (definition.Module is null || definition.Module.Assembly.Producer is not null)
            throw new InvalidDataException("field import requires a loaded immutable snapshot");
        var type = definition.DeclaringType;
        if ((definition.Attributes & 7) != 6 || (definition.Attributes & ~0x27) != 0 ||
            !definition.TryGetSignature(out var signature) || type.IsValueType || type.GenericArity != 0 || type.DeclaringType is not null ||
            (type.Attributes & 0x20) != 0 || type.Fields.Any(field => (field.Attributes & 0x10) != 0) || NativeBindingFor(type.Module.Assembly.Identity) is not null)
            throw new InvalidDataException("unsupported imported field contract");
        var owner = ImportReference(type, dependencyCoreLibrary);
        var storage = ImportNativeSignatureType(signature!, dependencyCoreLibrary, resolver);
        var key = (type.Module.Assembly.Identity, definition.MetadataToken);
        if (importedFields.TryGetValue(key, out var existing))
        {
            if (existing.Name != definition.Name || existing.FieldType != storage || existing.IsReadOnly != ((definition.Attributes & 0x20) != 0) || !ReferenceEquals(existing.DeclaringType, owner))
                throw new InvalidDataException("conflicting imported field contract");
            return existing;
        }
        if (importedFields.Count + authoredFields.Count >= 4096) throw new InvalidDataException("too many imported fields");
        var index = type.Fields.IndexOf(definition);
        if (index < 0) throw new InvalidDataException("field is not owned by its declaring snapshot");
        var reference = new ImportedFieldReference(this, owner, definition, storage, index);
        importedFields.Add(key, reference);
        return reference;
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Loads a primitive, nominal or vector field from its exact external receiver type.</summary>
    public void LoadField(ImportedFieldReference field) => GetILGenerator().LoadField(field);
    /// <summary>Stores a primitive, nominal or vector field on its exact external receiver type; readonly stores fail validation.</summary>
    public void StoreField(ImportedFieldReference field) => GetILGenerator().StoreField(field);
    /// <summary>Appends Ldfld or Stfld with an imported field operand.</summary>
    /// <param name="opCode">Ldfld or Stfld.</param>
    /// <param name="operand">Reference owned by this output builder.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode or foreign reference.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack and readonly checks run on write.</exception>
    public void Emit(OpCode opCode, ImportedFieldReference operand) => GetILGenerator().Emit(opCode, operand);
}
