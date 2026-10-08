namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A logical declaration container, distinct from the physical metadata image.</summary>
/// <remarks>Identity is the owning assembly identity plus the complete ordinal module name.
/// Dotted names do not grant private access or imply separately loadable artifacts.</remarks>
public sealed class DeclarationModuleDefinition
{
    internal DeclarationModuleDefinition(AssemblyDefinition assembly, string name)
    { Assembly = assembly; Name = name; }
    /// <summary>Gets the owning assembly; its name need not match this module.</summary>
    public AssemblyDefinition Assembly { get; }
    /// <summary>Gets the complete qualified module name. Empty denotes the global module.</summary>
    public string Name { get; }
    /// <summary>Whether this is an inferred view over older native or CLI namespace metadata.</summary>
    public bool IsProjection => Assembly.Producer is null && Assembly.DeclaredModuleNames is null;
    private AssemblyBuilder Builder => Assembly.Producer ?? throw new InvalidOperationException("loaded modules are read-only");
    /// <summary>Adds a class to this authored module; loaded modules reject mutation.</summary>
    public TypeBuilder AddClass(string name) => Builder.AddClass(Name, name);
    /// <summary>Adds an ownerless function to this authored module; loaded modules reject mutation.</summary>
    public MethodBuilder AddFunction(string name, MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public)
        => Builder.AddFunction(Name, name, signature, visibility);
    /// <summary>Adds a finite Double constant to this authored module; loaded modules reject mutation.</summary>
    public void AddConstant(string name, double value, MethodVisibility visibility = MethodVisibility.Public)
        => Builder.AddConstant(new(Name, name, value, visibility));
    /// <summary>Gets directly owned types, functions and constants; excludes child modules and nested types.</summary>
    public IReadOnlyList<AssemblyMemberDefinition> GetMembers() => Array.AsReadOnly(
        Assembly.GetMembers().Where(member => member.Namespace == Name).ToArray());
}

public sealed partial class AssemblyDefinition
{
    internal string[]? DeclaredModuleNames { get; set; }
    private readonly System.Collections.Concurrent.ConcurrentDictionary<string, DeclarationModuleDefinition> declarationModules = new(StringComparer.Ordinal);

    /// <summary>Gets logical modules in ordinal name order, independently of the physical MainModule.</summary>
    /// <remarks>Native v1 module manifests preserve empty modules. Older input exposes explicitly marked
    /// namespace projections. Parent names are not synthesized. No dependencies are loaded.</remarks>
    public IReadOnlyList<DeclarationModuleDefinition> GetModules()
    {
        var names = (DeclaredModuleNames ?? []).Concat(GetMembers().Select(member => member.Namespace))
            .Distinct(StringComparer.Ordinal).Order(StringComparer.Ordinal).ToArray();
        if (names.Length > 4096) throw new InvalidDataException("too many declaration modules");
        return Array.AsReadOnly(names.Select(GetDeclarationModule).ToArray());
    }

    internal DeclarationModuleDefinition GetDeclarationModule(string name)
    {
        return declarationModules.GetOrAdd(name, key => new(this, key));
    }
}

public sealed partial class AssemblyBuilder
{
    /// <summary>Declares or retrieves a logical module, including an empty one, for native emission.</summary>
    /// <param name="name">Qualified module name; empty denotes the global module.</param>
    /// <returns>The canonical module owned by this assembly.</returns>
    /// <exception cref="ArgumentException">Invalid name or more than 4096 modules.</exception>
    /// <remarks>Existing type/function/constant APIs select this owner through their namespace argument.
    /// CLI output retains populated namespace names but cannot preserve empty module declarations.</remarks>
    public DeclarationModuleDefinition DefineModule(string name)
    {
        FunctionNamespaceEncoding.Validate(name);
        var names = Definition.GetModules().Select(module => module.Name).Append(name).Distinct(StringComparer.Ordinal).ToArray();
        if (names.Length > 4096) throw new ArgumentException("too many declaration modules", nameof(name));
        Definition.DeclaredModuleNames = names;
        return Definition.GetDeclarationModule(name);
    }
}
