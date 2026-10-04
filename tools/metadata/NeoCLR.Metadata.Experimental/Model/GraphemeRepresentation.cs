namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class TypeDefinition
{
    /// <summary>Whether this declaration owns the native Unicode grapheme Char representation.</summary>
    public bool NativeGrapheme { get; private set; }

    /// <summary>Designates canonical System.Char as runtime-owned grapheme storage, not a CLI code unit.</summary>
    /// <exception cref="InvalidOperationException">The definition is a loaded snapshot.</exception>
    /// <exception cref="ArgumentException">The definition has the wrong identity, category or storage.</exception>
    public void SetNativeGrapheme()
    {
        if (authoredFields is null) throw new InvalidOperationException("loaded grapheme declarations are immutable");
        if (NativePrimitive is not null || Namespace != "System" || Name != "Char" || !IsValueType ||
            GenericArity != 0 || DeclaringType is not null || (Attributes & 0x1b8) != 0x108 ||
            Fields.Count != 0 || Methods.Any(m => m.Name is ".ctor" or ".cctor"))
            throw new ArgumentException("native grapheme requires canonical System.Char value storage without fields or constructors");
        NativeGrapheme = true;
    }
}

public sealed partial class TypeBuilder
{
    /// <summary>Whether this type owns native grapheme storage.</summary>
    public bool NativeGrapheme => Definition.NativeGrapheme;
    /// <summary>Designates runtime grapheme storage through the definition validation path.</summary>
    /// <exception cref="ArgumentException">The canonical identity, category or storage is incompatible.</exception>
    /// <exception cref="InvalidOperationException">The definition is a loaded snapshot.</exception>
    public void SetNativeGrapheme() => Definition.SetNativeGrapheme();
}

public sealed partial class AssemblyBuilder
{
    private ImportedTypeReference? externalGrapheme;
    /// <summary>Designates an output-owned external System.Char reference as native grapheme storage.</summary>
    /// <param name="type">An explicitly supplied nongeneric top-level System.Char value reference.</param>
    /// <exception cref="ArgumentNullException">The reference is null.</exception>
    /// <exception cref="ArgumentException">The reference is foreign or has an incompatible identity/category.</exception>
    /// <exception cref="InvalidDataException">Another dependency owns Char, or ordinary methods were already authored.</exception>
    public void SetNativeGrapheme(ImportedTypeReference type)
    {
        ArgumentNullException.ThrowIfNull(type);
        if (!ReferenceEquals(type.Owner, this) || type.Namespace != "System" || type.Name != "Char" ||
            !type.IsValueType || type.GenericArity != 0 || type.DeclaringType is not null)
            throw new ArgumentException("native grapheme requires an output-owned canonical System.Char value reference", nameof(type));
        if (types.Any(t => t.NativeGrapheme) ||
            externalGrapheme is not null && !Equals(externalGrapheme, type) ||
            authoredCallableReferences.Concat(importedReferences.Values).Any(m => Equals(m.DeclaringReference, type) && m.Target.DeclaringType?.NativeGrapheme != true))
            throw new InvalidDataException("conflicting native grapheme owner or prior ordinary method contract");
        externalGrapheme = type;
    }
    // Only the explicitly bound bootstrap's CLI Char signature is a native transport alias.
    internal SignatureType ImportCharacterSignature(AssemblyIdentity core)
    {
        if (NativeBindingFor(core)?.Library.ModuleName == "System")
        {
            if (externalGrapheme is not null) return externalGrapheme;
            if (types.SingleOrDefault(t => t.NativeGrapheme) is { } local) return local;
        }
        return ImportTypeIdentity(core, "System", "Char", 0, isValueType: true);
    }
    internal bool IsNativeGrapheme(ImportedTypeReference type) => Equals(externalGrapheme, type);
}
