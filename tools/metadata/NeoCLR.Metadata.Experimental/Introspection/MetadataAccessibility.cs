namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>Declared CLI-shaped accessibility; consumers apply their own language access rules.</summary>
public enum MetadataAccessibility
{
    /// <summary>Compiler-controlled member access (PrivateScope).</summary>
    CompilerControlled,
    /// <summary>Accessible only within the declaring type.</summary>
    Private,
    /// <summary>Accessible to derived types within the assembly.</summary>
    FamilyAndAssembly,
    /// <summary>Accessible within the assembly.</summary>
    Assembly,
    /// <summary>Accessible to derived types.</summary>
    Family,
    /// <summary>Accessible within the assembly or to derived types.</summary>
    FamilyOrAssembly,
    /// <summary>Publicly accessible.</summary>
    Public
}

internal static class MetadataAccess
{
    internal static MetadataAccessibility Member(ushort attributes) => (attributes & 7) switch
    {
        0 => MetadataAccessibility.CompilerControlled,
        1 => MetadataAccessibility.Private,
        2 => MetadataAccessibility.FamilyAndAssembly,
        3 => MetadataAccessibility.Assembly,
        4 => MetadataAccessibility.Family,
        5 => MetadataAccessibility.FamilyOrAssembly,
        6 => MetadataAccessibility.Public,
        _ => throw new InvalidDataException("invalid metadata member accessibility")
    };
    internal static MetadataAccessibility Type(uint attributes) => (attributes & 7) switch
    {
        0 or 5 => MetadataAccessibility.Assembly,
        1 or 2 => MetadataAccessibility.Public,
        3 => MetadataAccessibility.Private,
        4 => MetadataAccessibility.Family,
        6 => MetadataAccessibility.FamilyAndAssembly,
        7 => MetadataAccessibility.FamilyOrAssembly,
        _ => throw new InvalidDataException("invalid metadata type accessibility")
    };
}
