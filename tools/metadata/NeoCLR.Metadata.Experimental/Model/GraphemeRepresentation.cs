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
