using NeoCLR.Metadata.Experimental.Model;

namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>A metadata callable view with separate declaring-type and method-parameter scopes.</summary>
/// <remarks>No invocation is available. Constructor definitions may be resolved directly; GetMethods excludes them.</remarks>
public sealed class MethodInfo
{
    private readonly Lazy<TypeInfo> returns;
    private readonly Lazy<IReadOnlyList<ParameterInfo>> parameters;
    private readonly IReadOnlyList<TypeInfo> genericArguments;
    internal MetadataLoadContext Context { get; }
    internal MethodDefinition Definition { get; }
    private readonly MethodInfo? genericDefinition;
    internal MethodInfo(MetadataLoadContext context, MethodDefinition definition, TypeInfo? owner,
        MethodInfo? genericDefinition = null, TypeInfo[]? arguments = null)
    {
        if (!definition.TryGetSignature(out var signature)) throw new InvalidDataException("unsupported method metadata signature: " + definition.Name);
        Context = context; Definition = definition; this.genericDefinition = genericDefinition;
        Name = definition.Name; Namespace = definition.Namespace ?? ""; MetadataToken = definition.MetadataToken;
        DeclaringType = owner; Module = context.RequireSnapshot(definition.Module.Assembly).GetModules()[0];
        Accessibility = MetadataAccess.Member(definition.Attributes);
        IsStatic = definition.IsStatic; IsAbstract = (definition.Attributes & 0x400) != 0; IsVirtual = (definition.Attributes & 0x40) != 0; IsNewSlot = (definition.Attributes & 0x100) != 0;
        GenericParameterNames = Array.AsReadOnly(signature!.GenericParameterNames.ToArray());
        genericArguments = Array.AsReadOnly(arguments ?? GenericParameterNames.Select((_, i) => (TypeInfo)new MethodGenericParameterTypeInfo(context, this, i)).ToArray());
        var typeArguments = owner switch { NominalTypeInfo nominal => nominal.GetGenericArguments(), ConstructedTypeInfo constructed => constructed.TypeArguments, _ => Array.Empty<TypeInfo>() };
        returns = new(() => context.ResolveMemberSignature(signature.ReturnType, typeArguments, genericArguments, owner));
        parameters = new(() => Array.AsReadOnly(signature.ParameterTypes.Select((type, i) => new ParameterInfo(this, i,
            context.ResolveMemberSignature(type.ByReferenceElement ?? type, typeArguments, genericArguments, owner),
            type.ByReferenceElement is null ? ParameterPassingMode.Value : signature.OutParameters.Contains(i) ? ParameterPassingMode.Out : ParameterPassingMode.Ref,
            definition.ParameterNames.GetValueOrDefault(i))).ToArray()));
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
    /// <summary>Gets stable method-scoped parameters on a definition, or supplied arguments on a construction.</summary>
    public IReadOnlyList<TypeInfo> GetGenericArguments() => genericArguments;
    /// <summary>Gets whether this view is generic and has no supplied method arguments.</summary>
    public bool IsGenericMethodDefinition => GenericParameterNames.Count != 0 && genericDefinition is null;
    /// <summary>Gets the generic definition on the same declaring owner.</summary>
    /// <exception cref="InvalidOperationException">This method is not generic.</exception>
    public MethodInfo GetGenericMethodDefinition() => GenericParameterNames.Count != 0
        ? genericDefinition ?? this : throw new InvalidOperationException("method is not generic");
    /// <summary>Projects this generic definition with copied same-context method arguments; no invocation is available.</summary>
    /// <exception cref="InvalidOperationException">This view is not a generic method definition.</exception>
    /// <exception cref="ArgumentNullException">The argument array is null.</exception>
    /// <exception cref="ArgumentException">Wrong arity, null/foreign/Void/bare-generic arguments or excessive nesting.</exception>
    public MethodInfo MakeGenericMethod(params TypeInfo[] arguments) => Context.ConstructMethod(this, arguments);
    /// <summary>Gets declared metadata accessibility, without applying language access rules.</summary>
    public MetadataAccessibility Accessibility { get; }
    /// <summary>Gets whether this is an instance constructor declaration.</summary>
    public bool IsConstructor => Name == ".ctor" && !IsStatic;
    /// <summary>Gets whether this is a type initializer declaration.</summary>
    public bool IsStaticConstructor => Name == ".cctor" && IsStatic;
    /// <summary>Gets the metadata static flag.</summary>
    public bool IsStatic { get; }
    /// <summary>Gets the metadata abstract flag.</summary>
    public bool IsAbstract { get; }
    /// <summary>Gets the metadata virtual flag.</summary>
    public bool IsVirtual { get; }
    /// <summary>Gets the CLI NewSlot flag; a virtual method without it reuses an inherited slot.</summary>
    public bool IsNewSlot { get; }
}

/// <summary>The supported metadata parameter passing conventions.</summary>
public enum ParameterPassingMode
{
    /// <summary>A value is copied into the parameter.</summary>
    Value,
    /// <summary>A writable managed reference whose target is initialized by the caller.</summary>
    Ref,
    /// <summary>A writable managed reference assigned by the callee before normal return.</summary>
    Out
}

/// <summary>A metadata parameter with position and type; absent native parameter names are not invented.</summary>
public sealed class ParameterInfo
{
    internal ParameterInfo(MethodInfo method, int position, TypeInfo type, ParameterPassingMode passingMode, string? name) { DeclaringMethod = method; Position = position; ParameterType = type; PassingMode = passingMode; Name = name; }
    /// <summary>Gets the declared parameter name, or null when metadata omits it.</summary>
    public string? Name { get; }
    /// <summary>Gets the canonical owning method view.</summary>
    public MethodInfo DeclaringMethod { get; }
    /// <summary>Gets the zero-based parameter index.</summary>
    public int Position { get; }
    /// <summary>Gets the declared value, ref or out passing convention.</summary>
    public ParameterPassingMode PassingMode { get; }
    /// <summary>Gets the projected value type, or the referenced element type for ref/out parameters.</summary>
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
    /// <summary>Resolves the owned nongeneric interface bounds for this method parameter.</summary>
    /// <remarks>Inference, language conversions and generic argument admission belong to the caller.</remarks>
    public IReadOnlyList<NominalTypeInfo> GetInterfaceConstraints() => Array.AsReadOnly(DeclaringMethod.Definition.InterfaceConstraints
        .Where(c => c.ParameterIndex == Position).Select(c => Context.GetType(c.InterfaceType)).ToArray());
    /// <inheritdoc/>
    public override string DisplayName => "!!" + Position;
    /// <inheritdoc/>
    public override bool IsNominalType => false;
}

public sealed partial class MetadataLoadContext
{
    private readonly Dictionary<(MethodDefinition, TypeInfo?), MethodInfo> methods = [];
    private readonly Dictionary<MethodConstructionKey, MethodInfo> methodConstructions = [];
    internal MethodInfo ConstructMethod(MethodInfo definition, TypeInfo[] arguments)
    {
        ArgumentNullException.ThrowIfNull(arguments);
        if (!definition.IsGenericMethodDefinition) throw new InvalidOperationException("method is not a generic definition");
        var copied = (TypeInfo[])arguments.Clone();
        if (copied.Length != definition.GenericParameterNames.Count || copied.Any(t => t is null ||
            !ReferenceEquals(t.Context, this) || t is PrimitiveTypeInfo { Kind: PrimitiveType.Void } ||
            t is NominalTypeInfo { GenericArity: > 0 } || t.Depth >= 16))
            throw new ArgumentException("invalid metadata method construction", nameof(arguments));
        var key = new MethodConstructionKey(definition, copied);
        lock (gate)
        {
            if (!methodConstructions.TryGetValue(key, out var view))
                methodConstructions.Add(key, view = new MethodInfo(this, definition.Definition, definition.DeclaringType, definition, copied));
            return view;
        }
    }
    private sealed class MethodConstructionKey(MethodInfo definition, TypeInfo[] arguments) : IEquatable<MethodConstructionKey>
    {
        private MethodInfo Definition { get; } = definition;
        private TypeInfo[] Arguments { get; } = arguments;
        public bool Equals(MethodConstructionKey? other) => other is not null && ReferenceEquals(Definition, other.Definition) && Arguments.SequenceEqual(other.Arguments);
        public override bool Equals(object? obj) => obj is MethodConstructionKey other && Equals(other);
        public override int GetHashCode() { var hash = new HashCode(); hash.Add(Definition); foreach (var argument in Arguments) hash.Add(argument); return hash.ToHashCode(); }
    }
    /// <summary>Gets the canonical callable view on its open declaring type or namespace.</summary>
    /// <exception cref="ArgumentNullException">Definition is null.</exception>
    /// <exception cref="InvalidDataException">The snapshot is unregistered or its signature cannot be decoded.</exception>
    public MethodInfo Resolve(MethodDefinition definition)
    {
        ArgumentNullException.ThrowIfNull(definition);
        RequireSnapshot(definition.Module.Assembly);
        return GetMethod(definition, definition.DeclaringType is { } owner ? GetType(owner) : null);
    }
    internal MethodInfo GetMethod(MethodDefinition definition, TypeInfo? owner)
    {
        lock (gate)
        {
            if (!methods.TryGetValue((definition, owner), out var view)) methods.Add((definition, owner), view = new MethodInfo(this, definition, owner));
            return view;
        }
    }
    internal IReadOnlyList<MethodInfo> GetConstructors(TypeDefinition definition, TypeInfo owner)
        => Array.AsReadOnly(definition.Methods.Where(m => m.Name is ".ctor" or ".cctor").Select(m => GetMethod(m, owner)).ToArray());
    internal IReadOnlyList<MethodInfo> GetMethods(TypeDefinition definition, TypeInfo owner)
        => Array.AsReadOnly(definition.Methods.Where(m => m.Name is not (".ctor" or ".cctor")).Select(m => GetMethod(m, owner)).ToArray());
}
