using NeoCLR.Metadata.Experimental.Model;

namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>A context-owned view of a Raven declaration module, distinct from a physical metadata module.</summary>
/// <remarks>Reference identity is canonical within one metadata context. Equal names in different
/// assemblies or contexts remain distinct. No loading, attribute construction or invocation occurs.</remarks>
public sealed class DeclarationModuleInfo
{
    private readonly Lazy<IReadOnlyList<AssemblyMemberInfo>> members;

    internal DeclarationModuleInfo(MetadataLoadContext context, AssemblyInfo assembly, DeclarationModuleDefinition definition)
    {
        Assembly = assembly;
        Name = definition.Name;
        IsProjection = definition.IsProjection;
        members = new(() => Array.AsReadOnly(definition.GetMembers()
            .Select(member => new AssemblyMemberInfo(context, assembly, assembly.GetModules()[0], member)).ToArray()));
    }

    /// <summary>Gets the complete ordinal module name; empty denotes the global module.</summary>
    public string Name { get; }
    /// <summary>Gets the canonical assembly that owns this declaration module.</summary>
    public AssemblyInfo Assembly { get; }
    /// <summary>Gets whether this module was inferred from older namespace metadata rather than declared explicitly.</summary>
    public bool IsProjection { get; }
    /// <summary>Gets directly owned types, functions and constants in assembly-member order.</summary>
    /// <remarks>The read-only list excludes child modules, nested types and type-owned members.
    /// Typed member signatures are resolved only when their Type or Function properties are accessed.</remarks>
    public IReadOnlyList<AssemblyMemberInfo> GetMembers() => members.Value;
}
