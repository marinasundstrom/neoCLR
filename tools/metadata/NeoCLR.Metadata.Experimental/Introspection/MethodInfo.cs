using NeoCLR.Metadata.Experimental.Model;

namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>A metadata callable view with separate declaring-type and method-parameter scopes.</summary>
/// <remarks>No invocation is available. Constructor definitions may be resolved directly; GetMethods excludes them.</remarks>
public sealed class MethodInfo
{
    private readonly Lazy<TypeInfo> returns;
    private readonly Lazy<IReadOnlyList<ParameterInfo>> parameters;
    private readonly IReadOnlyList<TypeInfo> genericArguments;
    internal MethodInfo(MetadataLoadContext context, MethodDefinition definition, TypeInfo? owner)
    {
        if (!definition.TryGetSignature(out var signature)) throw new InvalidDataException("unsupported method metadata signature: " + definition.Name);
        Name = definition.Name; Namespace = definition.Namespace ?? ""; MetadataToken = definition.MetadataToken;
        DeclaringType = owner; Module = context.RequireSnapshot(definition.Module.Assembly).GetModules()[0];
        IsStatic = definition.IsStatic; IsAbstract = (definition.Attributes & 0x400) != 0; IsVirtual = (definition.Attributes & 0x40) != 0;
        GenericParameterNames = Array.AsReadOnly(signature!.GenericParameterNames.ToArray());
        genericArguments = Array.AsReadOnly(GenericParameterNames.Select((_, i) => (TypeInfo)new MethodGenericParameterTypeInfo(context, this, i)).ToArray());
        var typeArguments = owner switch { NominalTypeInfo nominal => nominal.GetGenericArguments(), ConstructedTypeInfo constructed => constructed.TypeArguments, _ => Array.Empty<TypeInfo>() };
        returns = new(() => context.ResolveSignature(signature.ReturnType, typeArguments, genericArguments));
        parameters = new(() => Array.AsReadOnly(signature.ParameterTypes.Select((type, i) => new ParameterInfo(this, i,
            context.ResolveSignature(type, typeArguments, genericArguments))).ToArray()));
    }
    /// <summary>Gets the declaration name.</summary>
    public string Name { get; }
    /// <summary>Gets the namespace recorded for a namespace function; empty when absent.</summary>
    public string Namespace { get; }
    /// <summary>Gets the original module-local method token.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the declaring module view.</summary>
    public ModuleInfo Module { get; }
    /// <summary>Gets the open/constructed declaring type; null for namespace functions.</summary>
    public TypeInfo? DeclaringType { get; }
    /// <summary>Gets the return signature projected in both generic scopes.</summary>
    public TypeInfo ReturnType => returns.Value;
    /// <summary>Gets declared parameters in order. Unsupported signatures/dependencies throw InvalidDataException on access.</summary>
    public IReadOnlyList<ParameterInfo> GetParameters() => parameters.Value;
    /// <summary>Gets copied metadata generic parameter names.</summary>
    public IReadOnlyList<string> GenericParameterNames { get; }
    /// <summary>Gets stable method-scoped parameter views.</summary>
    public IReadOnlyList<TypeInfo> GetGenericArguments() => genericArguments;
    /// <summary>Gets the metadata static flag.</summary>
    public bool IsStatic { get; }
    /// <summary>Gets the metadata abstract flag.</summary>
    public bool IsAbstract { get; }
    /// <summary>Gets the metadata virtual flag.</summary>
    public bool IsVirtual { get; }
}

/// <summary>A metadata parameter with position and type; absent native parameter names are not invented.</summary>
public sealed class ParameterInfo
{
    internal ParameterInfo(MethodInfo method, int position, TypeInfo type) { DeclaringMethod = method; Position = position; ParameterType = type; }
    /// <summary>Gets the canonical owning method view.</summary>
    public MethodInfo DeclaringMethod { get; }
    /// <summary>Gets the zero-based parameter index.</summary>
    public int Position { get; }
    /// <summary>Gets the projected parameter type.</summary>
    public TypeInfo ParameterType { get; }
}

/// <summary>A method parameter identity distinct from a declaring-type parameter at the same ordinal.</summary>
public sealed class MethodGenericParameterTypeInfo : TypeInfo
{
    internal MethodGenericParameterTypeInfo(MetadataLoadContext context, MethodInfo method, int position) : base(context)
    { DeclaringMethod = method; Position = position; }
    /// <summary>Gets the canonical method view that declares this parameter.</summary>
    public MethodInfo DeclaringMethod { get; }
    /// <summary>Gets its zero-based method scope ordinal.</summary>
    public int Position { get; }
    /// <inheritdoc/>
    public override string DisplayName => "!!" + Position;
    /// <inheritdoc/>
    public override bool IsNominalType => false;
}

public sealed partial class MetadataLoadContext
{
    private readonly Dictionary<(MethodDefinition, TypeInfo?), MethodInfo> methods = [];
    /// <summary>Gets the canonical callable view on its open declaring type or namespace.</summary>
    /// <exception cref="ArgumentNullException">Definition is null.</exception>
    /// <exception cref="InvalidDataException">The snapshot is unregistered or its signature cannot be decoded.</exception>
    public MethodInfo Resolve(MethodDefinition definition)
    {
        ArgumentNullException.ThrowIfNull(definition);
        RequireSnapshot(definition.Module.Assembly);
        return GetMethod(definition, definition.DeclaringType is { } owner ? GetType(owner) : null);
    }
    private MethodInfo GetMethod(MethodDefinition definition, TypeInfo? owner)
    {
        lock (gate)
        {
            if (!methods.TryGetValue((definition, owner), out var view)) methods.Add((definition, owner), view = new MethodInfo(this, definition, owner));
            return view;
        }
    }
    internal IReadOnlyList<MethodInfo> GetMethods(TypeDefinition definition, TypeInfo owner)
        => Array.AsReadOnly(definition.Methods.Where(m => m.Name is not (".ctor" or ".cctor")).Select(m => GetMethod(m, owner)).ToArray());
}
