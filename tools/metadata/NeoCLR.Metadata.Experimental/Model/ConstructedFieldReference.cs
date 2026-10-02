namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An immutable field reference on an owned constructed generic class.</summary>
public sealed class ConstructedFieldReference
{
    internal ConstructedFieldReference(FieldBuilder definition, GenericTypeInstance owner)
    {
        Definition = definition; DeclaringType = owner;
        SignatureType Substitute(SignatureType type) => type.FunctionSignature is { } function ? function.Substitute(Substitute) : type.ImportedType is { } imported ? imported.Substitute(Substitute) : type.TypeParameterIndex is { } index ? owner.TypeArguments[index]
            : type.ArrayElement is { } element ? SignatureType.ArrayOf(Substitute(element))
            : type.GenericInstance is { } instance ? instance.Definition.MakeGenericInstance(instance.TypeArguments.Select(Substitute).ToArray()) : type;
        FieldType = Substitute(definition.FieldType);
    }
    /// <summary>Gets the open field definition.</summary>
    public FieldBuilder Definition { get; }
    /// <summary>Gets the exact constructed owner.</summary>
    public GenericTypeInstance DeclaringType { get; }
    /// <summary>Gets the simultaneously substituted field signature.</summary>
    public SignatureType FieldType { get; }
}

public sealed partial class FieldBuilder
{
    /// <summary>Binds the declaring class parameters, copying arguments and preserving caller parameters.</summary>
    /// <param name="typeArguments">One non-Void supported type per owner parameter.</param>
    /// <returns>An immutable field reference.</returns>
    /// <exception cref="ArgumentNullException">Null arguments.</exception>
    /// <exception cref="ArgumentException">Nongeneric owner, wrong arity, invalid/foreign argument or unsupported nested substitution.</exception>
    public ConstructedFieldReference MakeConstructedReference(params SignatureType[] typeArguments)
        => new(this, DeclaringType.MakeGenericInstance(typeArguments));
}

public sealed partial class MethodBuilder
{
    /// <summary>Loads a field from an exactly matching constructed receiver.</summary>
    public void LoadField(ConstructedFieldReference field) => GetILGenerator().LoadField(field);
    /// <summary>Stores a field on an exactly matching constructed receiver.</summary>
    public void StoreField(ConstructedFieldReference field) => GetILGenerator().StoreField(field);
    /// <summary>Appends Ldfld or Stfld with a constructed field reference.</summary>
    /// <param name="opCode">Ldfld or Stfld.</param>
    /// <param name="operand">Owned reference, valid in the caller's parameter scope.</param>
    /// <exception cref="ArgumentNullException">Null reference.</exception>
    /// <exception cref="ArgumentException">Wrong opcode, foreign definition or out-of-scope argument.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded; receiver, value and readonly checks run on write.</exception>
    public void Emit(OpCode opCode, ConstructedFieldReference operand) => GetILGenerator().Emit(opCode, operand);
}
