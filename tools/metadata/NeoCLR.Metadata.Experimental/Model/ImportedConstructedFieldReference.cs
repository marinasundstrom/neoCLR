namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable imported instance field on a constructed generic owner.</summary>
public sealed class ImportedConstructedFieldReference
{
    internal ImportedConstructedFieldReference(ImportedFieldReference definition, ImportedTypeReference owner)
    {
        Definition = definition;
        DeclaringType = owner;
        SignatureType Substitute(SignatureType type) => type.TypeParameterIndex is { } index ? owner.TypeArguments[index]
            : type.ArrayElement is { } element ? SignatureType.ArrayOf(Substitute(element))
            : type.ImportedType is { } imported ? imported.Substitute(Substitute) : type;
        FieldType = Substitute(definition.FieldType);
    }
    /// <summary>Gets the open, output-owned field contract.</summary>
    public ImportedFieldReference Definition { get; }
    /// <summary>Gets the constructed output-owned declaring type.</summary>
    public ImportedTypeReference DeclaringType { get; }
    /// <summary>Gets the simultaneously substituted storage signature.</summary>
    public SignatureType FieldType { get; }
}

public sealed partial class ImportedFieldReference
{
    /// <summary>Binds this generic owner's parameters, copying arguments and preserving caller scopes.</summary>
    /// <param name="typeArguments">One non-Void output-owned argument per owner parameter.</param>
    /// <returns>An immutable constructed field reference.</returns>
    /// <exception cref="ArgumentNullException">Arguments are null.</exception>
    /// <exception cref="ArgumentException">Wrong arity, nongeneric owner or invalid/foreign argument.</exception>
    public ImportedConstructedFieldReference MakeConstructedReference(params SignatureType[] typeArguments)
        => new(this, DeclaringType.MakeGenericInstance(typeArguments));
}

public sealed partial class MethodBuilder
{
    /// <summary>Loads an imported field from an exactly matching constructed receiver.</summary>
    public void LoadField(ImportedConstructedFieldReference field) => GetILGenerator().LoadField(field);
    /// <summary>Stores an imported field on an exactly matching constructed receiver.</summary>
    public void StoreField(ImportedConstructedFieldReference field) => GetILGenerator().StoreField(field);
    /// <summary>Appends Ldfld or Stfld for an output-owned constructed field; scope is checked now and stack/readonly rules on write.</summary>
    public void Emit(OpCode opCode, ImportedConstructedFieldReference operand) => GetILGenerator().Emit(opCode, operand);
}
