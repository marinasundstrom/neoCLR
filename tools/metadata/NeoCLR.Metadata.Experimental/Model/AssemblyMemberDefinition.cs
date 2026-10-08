namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>The kind of a declaration directly owned by an assembly.</summary>
public enum AssemblyMemberKind
{
    /// <summary>A top-level type; nested types remain owned by their declaring type.</summary>
    Type,
    /// <summary>A function with no declaring type.</summary>
    Function,
    /// <summary>A compile-time constant with no declaring type.</summary>
    Constant
}

/// <summary>An assembly-level member view shared by authored graphs and read-only metadata snapshots.</summary>
/// <remarks>Exactly one typed declaration is present. Namespace is part of FullName; assembly identity and
/// function signatures remain necessary for unambiguous resolution. This view does not create new identities.</remarks>
public sealed class AssemblyMemberDefinition
{
    internal AssemblyMemberDefinition(TypeDefinition type)
    {
        Type = type;
        Kind = AssemblyMemberKind.Type;
        Module = type.Module;
        Namespace = type.Namespace;
        Name = type.Name;
    }
    internal AssemblyMemberDefinition(MethodDefinition function)
    {
        Function = function;
        Kind = AssemblyMemberKind.Function;
        Module = function.Module;
        Namespace = function.Namespace ?? "";
        Name = function.Name;
    }
    internal AssemblyMemberDefinition(AssemblyConstantDefinition constant)
    {
        Constant = constant;
        Kind = AssemblyMemberKind.Constant;
        Module = constant.Module!;
        Namespace = constant.Namespace;
        Name = constant.Name;
    }
    /// <summary>Gets the member kind.</summary>
    public AssemblyMemberKind Kind { get; }
    /// <summary>Gets the owning metadata module.</summary>
    public ModuleDefinition Module { get; }
    /// <summary>Gets the declaring assembly and its exact identity.</summary>
    public AssemblyDefinition Assembly => Module.Assembly;
    /// <summary>Gets the logical module directly owning this declaration.</summary>
    public DeclarationModuleDefinition DeclaringModule => Assembly.GetDeclarationModule(Namespace);
    /// <summary>Gets the namespace portion of the qualified name.</summary>
    public string Namespace { get; }
    /// <summary>Gets the simple metadata name.</summary>
    public string Name { get; }
    /// <summary>Gets the namespace-qualified name, without an assembly or overload signature suffix.</summary>
    public string FullName => Namespace.Length == 0 ? Name : Namespace + "." + Name;
    /// <summary>Gets the type declaration for Type members; otherwise null.</summary>
    public TypeDefinition? Type { get; }
    /// <summary>Gets the callable declaration for Function members; otherwise null.</summary>
    public MethodDefinition? Function { get; }
    /// <summary>Gets the constant declaration for Constant members; otherwise null.</summary>
    public AssemblyConstantDefinition? Constant { get; }
}

public sealed partial class AssemblyDefinition
{
    /// <summary>Gets assembly-level types, functions and constants from the manifest module.</summary>
    /// <remarks>Returns a fresh read-only list, ordered by types, functions and constants, preserving declaration
    /// order within each kind. Excludes the CLI Module pseudo-type, nested types and type-owned methods/fields.
    /// Authored and loaded declarations use the same view; general multi-module assemblies remain unsupported.</remarks>
    public IReadOnlyList<AssemblyMemberDefinition> GetMembers() => Array.AsReadOnly(
        MainModule.Types.Where(t => t.DeclaringType is null && t.Name != "<Module>").Select(t => new AssemblyMemberDefinition(t))
        .Concat(MainModule.Functions.Select(f => new AssemblyMemberDefinition(f)))
        .Concat(MainModule.Constants.Select(c => new AssemblyMemberDefinition(c))).ToArray());
}
