namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyBuilder
{
    private TypeBuilder? arrayBacking;

    /// <summary>Selects the output-owned nominal class backing native vector storage.</summary>
    /// <param name="type">Unconstrained generic class with one private element-vector field. Its local base may be absent or the native Object root.</param>
    /// <exception cref="ArgumentException">Foreign or incompatible descriptor.</exception>
    /// <exception cref="InvalidOperationException">A different descriptor was already selected.</exception>
    /// <remarks>Native execution metadata only; CLI vectors retain ordinary CLI semantics. Revalidated on native write.</remarks>
    public void SetArrayBacking(TypeBuilder type)
    {
        ArgumentNullException.ThrowIfNull(type);
        ValidateArrayBacking(type);
        if (arrayBacking is not null && !ReferenceEquals(arrayBacking, type))
            throw new InvalidOperationException("array backing already selected");
        arrayBacking = type;
    }

    private void ValidateArrayBacking(TypeBuilder type)
    {
        if (!ReferenceEquals(type.Assembly, this) || type.IsStatic || type.IsValueType || type.IsInterface ||
            (type.Definition.Attributes & 0x80) != 0 || type.Definition.DeclaringType is not null ||
            type.LocalBase is { IsNativeObjectRoot: false } ||
            type.GenericConstraints.Count != 0 || type.SpecialConstraints.Values.Any(v => v != 0) ||
            type.GenericParameterNames.Count != 1 || type.Fields.Count != 1 ||
            type.Fields[0].Visibility != FieldVisibility.Private ||
            type.Fields[0].FieldType != SignatureType.ArrayOf(SignatureType.TypeParameter(0)))
            throw new ArgumentException("array backing requires an owned generic class over no base or the native Object root with one private element-vector field");
    }
}
