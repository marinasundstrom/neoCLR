using System.Reflection;

namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class MethodDefinition
{
    internal MethodBuilder? Producer { get; }
    internal MethodDefinition(MethodBuilder producer, string name, string ns, MethodSignature methodSignature, MethodVisibility visibility, bool isStatic)
    {
        Producer = producer;
        Module = producer.Assembly.Definition.MainModule;
        Name = name;
        Namespace = ns;
        AuthoredSignature = methodSignature;
        GenericArity = methodSignature.GenericParameterNames.Count;
        declarationAttributes = (ushort)((visibility switch { MethodVisibility.Public => 6, MethodVisibility.Internal => 3, _ => 1 }) | (isStatic ? 0x10 : 0));
        signature = [];
    }
    /// <summary>Gets the authored source namespace; null for a loaded physical declaration.</summary>
    public string? Namespace { get; }
    /// <summary>Gets the authored signature shared by the builder; null for a loaded physical signature.</summary>
    /// <remarks>Raw signature recognizers apply to loaded declarations. Encode and reread for physical tokens and signature blobs.</remarks>
    public MethodSignature? AuthoredSignature { get; }
}

public sealed partial class MethodBuilder
{
    /// <summary>Gets the canonical declaration shared with the authored assembly's method views.</summary>
    public MethodDefinition Definition { get; }
    // Keep access flags and context-derived CLI flags in one place for inspection and writing.
    internal ushort GetAttributes(bool? accessor = null)
    {
        bool special = accessor ?? DeclaringType?.Properties.Any(p => ReferenceEquals(p.GetMethod, this) || ReferenceEquals(p.SetMethod, this)) == true;
        var flags = (MethodAttributes)Definition.DeclarationAttributes | MethodAttributes.HideBySig;
        if (IsAbstract) flags |= MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot;
        else if (DeclaringType?.Implements(this) == true) flags |= MethodAttributes.Virtual | MethodAttributes.Final | MethodAttributes.NewSlot;
        if (IsConstructor) flags |= MethodAttributes.SpecialName | MethodAttributes.RTSpecialName;
        else if (special) flags |= MethodAttributes.SpecialName;
        return (ushort)flags;
    }
}
