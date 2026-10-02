namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable primitive or local nominal instance-field reference owned by one output assembly.</summary>
public sealed class ImportedFieldReference
{
    internal ImportedFieldReference(AssemblyBuilder owner, ImportedTypeReference declaringType, FieldDefinition definition, SignatureType type, int index)
    { Owner = owner; DeclaringType = declaringType; Name = definition.Name; FieldType = type; IsReadOnly = (definition.Attributes & 0x20) != 0; NativeIndex = definition.Module.Assembly.IsNative ? index : null; }
    /// <summary>Gets the consuming builder.</summary>
    public AssemblyBuilder Owner { get; }
    /// <summary>Gets the exact external declaring class.</summary>
    public ImportedTypeReference DeclaringType { get; }
    /// <summary>Gets the metadata field name.</summary>
    public string Name { get; }
    /// <summary>Gets the output-owned primitive or imported nominal storage signature.</summary>
    public SignatureType FieldType { get; }
    /// <summary>Gets whether stores from this external consumer are forbidden.</summary>
    public bool IsReadOnly { get; }
    internal int? NativeIndex { get; }
}

public sealed partial class AssemblyBuilder
{
    private readonly Dictionary<(AssemblyIdentity, uint), ImportedFieldReference> importedFields = [];
    /// <summary>Imports a public primitive or native local-class instance field on a public nongeneric top-level reference class.</summary>
    /// <param name="definition">Immutable CLI or native field definition.</param>
    /// <param name="dependencyCoreLibrary">Explicit core identity, matching this output.</param>
    /// <returns>An interned reference owned by this builder.</returns>
    /// <exception cref="ArgumentNullException">A required argument is null.</exception>
    /// <exception cref="InvalidDataException">Unsupported owner/signature/access/layout, incompatible core/snapshot, translated binding or limit.</exception>
    /// <remarks>CLI output uses a MemberRef. Native output requires a native definition snapshot and uses its validated field ordinal. CLI input fields support CLI output only. Static fields, constructed owners and translated layouts are not admitted. Readonly loads are supported; stores fail body validation.</remarks>
    public ImportedFieldReference ImportReference(FieldDefinition definition, AssemblyIdentity dependencyCoreLibrary)
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
        SignatureType storage = signature!.Primitive is { } primitive ? primitive
            : signature.ReferencedType is { } nominal ? ImportReference(nominal.Resolve(), dependencyCoreLibrary)
            : throw new InvalidDataException("unsupported imported field signature");
        var key = (type.Module.Assembly.Identity, definition.MetadataToken);
        if (importedFields.TryGetValue(key, out var existing))
        {
            if (existing.Name != definition.Name || existing.FieldType != storage || existing.IsReadOnly != ((definition.Attributes & 0x20) != 0) || !ReferenceEquals(existing.DeclaringType, owner))
                throw new InvalidDataException("conflicting imported field contract");
            return existing;
        }
        if (importedFields.Count >= 4096) throw new InvalidDataException("too many imported fields");
        var index = type.Fields.IndexOf(definition);
        if (index < 0) throw new InvalidDataException("field is not owned by its declaring snapshot");
        var reference = new ImportedFieldReference(this, owner, definition, storage, index);
        importedFields.Add(key, reference);
        return reference;
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Loads a primitive or nominal field from its exact external receiver type.</summary>
    public void LoadField(ImportedFieldReference field) => Emit(OpCode.Ldfld, field);
    /// <summary>Stores a primitive or nominal field on its exact external receiver type; readonly stores fail validation.</summary>
    public void StoreField(ImportedFieldReference field) => Emit(OpCode.Stfld, field);
    /// <summary>Appends Ldfld or Stfld with an imported field operand.</summary>
    /// <param name="opCode">Ldfld or Stfld.</param>
    /// <param name="operand">Reference owned by this output builder.</param>
    /// <exception cref="ArgumentNullException">Operand is null.</exception>
    /// <exception cref="ArgumentException">Wrong opcode or foreign reference.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; stack and readonly checks run on write.</exception>
    public void Emit(OpCode opCode, ImportedFieldReference operand)
    {
        ArgumentNullException.ThrowIfNull(operand);
        if (!ReferenceEquals(operand.Owner, Assembly)) throw new ArgumentException("foreign imported field", nameof(operand));
        Append(new(opCode switch { OpCode.Ldfld => "field.import.load", OpCode.Stfld => "field.import.store", _ => throw OperandError(opCode) }, ImportedField: operand));
    }
}
