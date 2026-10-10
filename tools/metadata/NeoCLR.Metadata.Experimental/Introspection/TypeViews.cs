using NeoCLR.Metadata.Experimental.Model;

namespace NeoCLR.Metadata.Experimental.Introspection;

/// <summary>A primitive signature category; it does not invent a core-library definition.</summary>
public sealed class PrimitiveTypeInfo : TypeInfo
{
    internal PrimitiveTypeInfo(MetadataLoadContext context, PrimitiveType kind) : base(context) { Kind = kind; }
    /// <summary>Gets the metadata primitive category, including Void for a no-result signature.</summary>
    public PrimitiveType Kind { get; }
    /// <inheritdoc/>
    public override string DisplayName => Kind.ToString();
    /// <inheritdoc/>
    public override bool IsNominalType => false;
}

/// <summary>A single-dimensional metadata vector view.</summary>
public sealed class ArrayTypeInfo : TypeInfo
{
    internal ArrayTypeInfo(MetadataLoadContext context, TypeInfo element) : base(context) { ElementType = element; }
    /// <summary>Gets the canonical element view.</summary>
    public TypeInfo ElementType { get; }
    internal override int Depth => 1 + ElementType.Depth;
    /// <inheritdoc/>
    public override string DisplayName => ElementType.DisplayName + "[]";
    /// <inheritdoc/>
    public override bool IsNominalType => false;
}

/// <summary>A type parameter whose identity includes its declaring definition and context.</summary>
public sealed class GenericParameterTypeInfo : TypeInfo
{
    internal GenericParameterTypeInfo(MetadataLoadContext context, NominalTypeInfo owner, int position) : base(context)
    { DeclaringType = owner; Position = position; }
    /// <summary>Gets the canonical declaring definition.</summary>
    public NominalTypeInfo DeclaringType { get; }
    /// <summary>Gets the zero-based declaration ordinal.</summary>
    public int Position { get; }
    /// <summary>Gets the declared generic parameter name from supported native metadata.</summary>
    /// <exception cref="NotSupportedException">The reader has not materialized parameter names for this snapshot format.</exception>
    /// <remarks>The name is descriptive; identity remains the declaring type and ordinal.</remarks>
    public string Name => DeclaringType.Definition.GenericParameterNames is { } names
        ? names[Position] : throw new NotSupportedException("generic parameter names are not materialized for this snapshot");
    /// <inheritdoc/>
    public override string DisplayName => "!" + Position;
    /// <inheritdoc/>
    public override bool IsNominalType => false;
}

/// <summary>The implementing type of a particular interface view, retained without a runtime type or generic ordinal.</summary>
public sealed class SelfTypeInfo : TypeInfo
{
    internal SelfTypeInfo(MetadataLoadContext context, TypeInfo contract) : base(context) { DeclaringType = contract; }
    /// <summary>Gets the canonical open or constructed interface view defining this Self scope.</summary>
    public TypeInfo DeclaringType { get; }
    /// <inheritdoc/>
    public override string DisplayName => DeclaringType.DisplayName + ".Self";
    /// <inheritdoc/>
    public override bool IsNominalType => false;
}

/// <summary>A nominal construction retaining its original definition and substituted declared fields.</summary>
public sealed class ConstructedTypeInfo : TypeInfo
{
    private readonly Lazy<IReadOnlyList<FieldInfo>> fields;
    private readonly Lazy<IReadOnlyList<PropertyInfo>> properties;
    private readonly Lazy<IReadOnlyList<TypeInfo>> interfaces;
    private readonly Lazy<IReadOnlyList<TypeInfo>> allInterfaces;
    internal ConstructedTypeInfo(MetadataLoadContext context, NominalTypeInfo definition, TypeInfo[] arguments) : base(context)
    {
        Definition = definition; TypeArguments = Array.AsReadOnly(arguments);
        fields = new(() => context.ProjectFields(definition.Definition, this, TypeArguments));
        properties = new(() => context.ProjectProperties(Definition.Definition, this, TypeArguments));
        interfaces = new(() => context.ProjectInterfaces(Definition.Definition, TypeArguments, this));
        allInterfaces = new(() => context.ProjectInterfaceClosure(this));
    }
    /// <summary>Gets the canonical open declaration.</summary>
    public NominalTypeInfo Definition { get; }
    /// <summary>Gets copied arguments, which may retain caller-scoped parameters.</summary>
    public IReadOnlyList<TypeInfo> TypeArguments { get; }
    /// <summary>Gets declared field views with simultaneous owner-argument substitution.</summary>
    /// <exception cref="InvalidDataException">A field signature or dependency is unsupported/unavailable.</exception>
    public IReadOnlyList<FieldInfo> GetFields() => fields.Value;
    /// <summary>Gets declared properties in metadata order, including non-public and indexed properties.</summary>
    /// <exception cref="InvalidDataException">A signature or dependency is unsupported or missing.</exception>
    public IReadOnlyList<PropertyInfo> GetProperties() => properties.Value;
    /// <summary>Gets directly declared interface relationships with owner arguments substituted; no transitive traversal.</summary>
    /// <exception cref="NotSupportedException">The reader cannot materialize interface relationships for this format.</exception>
    /// <exception cref="InvalidDataException">A relationship or dependency cannot be resolved.</exception>
    public IReadOnlyList<TypeInfo> GetDeclaredInterfaces() => interfaces.Value;
    /// <summary>Gets distinct direct and inherited interface views in depth-first metadata order, excluding this type.</summary>
    /// <exception cref="InvalidDataException">An invalid/cyclic relationship, missing dependency or traversal bound is encountered.</exception>
    /// <exception cref="NotSupportedException">The reader cannot materialize interface relationships.</exception>
    public IReadOnlyList<TypeInfo> GetInterfaces() => allInterfaces.Value;
    /// <summary>Gets declared non-constructor methods through this constructed owner.</summary>
    public IReadOnlyList<MethodInfo> GetMethods() => Context.GetMethods(Definition.Definition, this);
    /// <summary>Gets constructor declarations with this owner's arguments substituted.</summary>
    public IReadOnlyList<MethodInfo> GetConstructors() => Context.GetConstructors(Definition.Definition, this);
    internal override int Depth => 1 + TypeArguments.Max(t => t.Depth);
    /// <inheritdoc/>
    public override string DisplayName => Definition.DisplayName + "<" + string.Join(", ", TypeArguments.Select(t => t.DisplayName)) + ">";
    /// <inheritdoc/>
    public override bool IsNominalType => true;
}

/// <summary>A declared field viewed through an open or constructed metadata owner; no object access is available.</summary>
public sealed class FieldInfo
{
    private readonly Lazy<IReadOnlyList<CustomAttributeInfo>> attributes;
    /// <summary>Gets declared attributes without invoking constructors or accessors.</summary>
    public IReadOnlyList<CustomAttributeInfo> GetCustomAttributes() => attributes.Value;

    internal FieldInfo(FieldDefinition definition, TypeInfo declaringType, TypeInfo fieldType)
    {
        attributes = new(() => Array.AsReadOnly(definition.CustomAttributes.Select(a => new CustomAttributeInfo(declaringType.Context, a)).ToArray()));
        IsLiteral = definition.IsLiteral; Constant = definition.Constant; Accessibility = MetadataAccess.Member(definition.Attributes);
        Name = definition.Name; MetadataToken = definition.MetadataToken; DeclaringType = declaringType; FieldType = fieldType;
        IsStatic = (definition.Attributes & 0x10) != 0; IsReadOnly = (definition.Attributes & 0x20) != 0;
    }
    /// <summary>Gets the metadata name.</summary>
    public string Name { get; }
    /// <summary>Gets the original declaration's module-local field token.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the view through which this field was selected.</summary>
    public TypeInfo DeclaringType { get; }
    /// <summary>Gets the resolved and simultaneously substituted storage type.</summary>
    public TypeInfo FieldType { get; }
    /// <summary>Gets the metadata static flag.</summary>
    public bool IsStatic { get; }
    /// <summary>Gets declared metadata accessibility.</summary>
    public MetadataAccessibility Accessibility { get; }
    /// <summary>Gets the metadata init-only flag; this facade cannot write values.</summary>
    public bool IsReadOnly { get; }
    /// <summary>Gets whether the field is a static literal.</summary>
    public bool IsLiteral { get; }
    /// <summary>Gets its supported Int32 constant, or null when absent or unsupported.</summary>
    public int? Constant { get; }
}

/// <summary>An unmanaged pointer signature view. It conveys no ownership or dereference capability.</summary>
public sealed class PointerTypeInfo : TypeInfo
{
    internal PointerTypeInfo(MetadataLoadContext context, TypeInfo element) : base(context) { ElementType = element; }
    /// <summary>Gets the canonical pointed-to scalar or pointer view.</summary>
    public TypeInfo ElementType { get; }
    internal override int Depth => 1 + ElementType.Depth;
    /// <inheritdoc/>
    public override string DisplayName => ElementType.DisplayName + "*";
    /// <inheritdoc/>
    public override bool IsNominalType => false;
}

public sealed partial class MetadataLoadContext
{
    private readonly Dictionary<TypeInfo, PointerTypeInfo> pointers = [];
    private readonly Dictionary<FunctionKey, FunctionTypeInfo> functions = [];
    private readonly Dictionary<TypeInfo, SelfTypeInfo> selfTypes = [];
    private readonly Dictionary<PrimitiveType, PrimitiveTypeInfo> primitives = [];
    private readonly Dictionary<TypeInfo, ArrayTypeInfo> arrays = [];
    private readonly Dictionary<ConstructionKey, ConstructedTypeInfo> constructions = [];

    /// <summary>Projects a loaded primitive/nominal/vector/construction/function signature using explicit owner arguments.</summary>
    /// <param name="signature">Reader signature; output-builder operands are not accepted.</param>
    /// <param name="typeArguments">Owner arguments or scoped parameter views; copied before traversal.</param>
    /// <returns>A canonical context-owned type view.</returns>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">Arguments are foreign, null or excessive.</exception>
    /// <exception cref="InvalidDataException">Missing dependency, invalid parameter scope or unsupported signature (including method parameters).</exception>
    public TypeInfo ResolveSignature(SignatureType signature, IReadOnlyList<TypeInfo>? typeArguments = null)
    {
        return ResolveSignature(signature, typeArguments, null);
    }

    /// <summary>Projects a signature with separate owner and method parameter scopes.</summary>
    /// <exception cref="ArgumentNullException">Signature is null.</exception>
    /// <exception cref="ArgumentException">A supplied scope is excessive or contains null, foreign or Void arguments.</exception>
    /// <exception cref="InvalidDataException">A referenced parameter or signature is unsupported or a dependency is missing.</exception>
    public TypeInfo ResolveSignature(SignatureType signature, IReadOnlyList<TypeInfo>? typeArguments, IReadOnlyList<TypeInfo>? methodArguments)
    {
        ArgumentNullException.ThrowIfNull(signature);
        if (typeArguments?.Count > 32 || methodArguments?.Count > 32) throw new ArgumentException("too many type arguments", nameof(typeArguments));
        var arguments = typeArguments?.ToArray() ?? [];
        var methods = methodArguments?.ToArray() ?? [];
        if (arguments.Concat(methods).Any(t => t is null || !ReferenceEquals(t.Context, this) || t is PrimitiveTypeInfo { Kind: PrimitiveType.Void })) throw new ArgumentException("foreign type argument", nameof(typeArguments));
        lock (gate) return Project(signature, arguments, methods);
    }

    internal TypeInfo ResolveMemberSignature(SignatureType signature, IReadOnlyList<TypeInfo> arguments, IReadOnlyList<TypeInfo> methods, TypeInfo? owner)
    {
        lock (gate) return Project(signature, arguments, methods, owner);
    }

    private TypeInfo Project(SignatureType signature, IReadOnlyList<TypeInfo> arguments, IReadOnlyList<TypeInfo> methods, TypeInfo? selfOwner = null)
    {
        if (signature.FunctionSignature is { } function)
        {
            var result = Project(function.ReturnType, arguments, methods, selfOwner);
            var parameters = function.ParameterTypes.Select(t => Project(t, arguments, methods, selfOwner)).ToArray();
            if (parameters.Append(result).Any(t => t.Depth >= 16)) throw new InvalidDataException("metadata type nesting exceeds limit");
            var key = new FunctionKey(result, parameters, function.NoResult);
            if (!functions.TryGetValue(key, out var view)) functions.Add(key, view = new FunctionTypeInfo(this, result, parameters, function.NoResult));
            return view;
        }
        if (signature.IsSelf)
        {
            if (selfOwner is not NominalTypeInfo { IsInterface: true } && selfOwner is not ConstructedTypeInfo { Definition.IsInterface: true })
                throw new InvalidDataException("Self requires an interface view scope");
            if (!selfTypes.TryGetValue(selfOwner, out var view)) selfTypes.Add(selfOwner, view = new SelfTypeInfo(this, selfOwner));
            return view;
        }
        if (signature.MethodParameterIndex is { } methodPosition)
            return methodPosition < methods.Count ? methods[methodPosition] : throw new InvalidDataException("metadata method parameter outside supplied scope");
        if (signature.TypeParameterIndex is { } position)
            return position < arguments.Count ? arguments[position] : throw new InvalidDataException("metadata type parameter outside supplied scope");
        if (signature.Primitive is { } primitive)
        {
            if (!primitives.TryGetValue(primitive, out var view)) primitives.Add(primitive, view = new PrimitiveTypeInfo(this, primitive));
            return view;
        }
        if (signature.PointerElement is { } pointer)
        {
            var type = Project(pointer, arguments, methods, selfOwner);
            if (type.Depth >= 16) throw new InvalidDataException("metadata type nesting exceeds limit");
            if (!pointers.TryGetValue(type, out var view)) pointers.Add(type, view = new PointerTypeInfo(this, type));
            return view;
        }
        if (signature.ArrayElement is { } element)
        {
            var type = Project(element, arguments, methods, selfOwner);
            if (type.Depth >= 16) throw new InvalidDataException("metadata type nesting exceeds limit");
            if (!arrays.TryGetValue(type, out var view)) arrays.Add(type, view = new ArrayTypeInfo(this, type));
            return view;
        }
        if (signature.ReferencedGenericInstance is { } constructed)
            return Construct(Resolve(constructed.Definition), constructed.TypeArguments.Select(t => Project(t, arguments, methods, selfOwner)).ToArray());
        if (signature.ReferencedType is { } reference) return Resolve(reference);
        throw new InvalidDataException("unsupported metadata facade signature");
    }

    internal ConstructedTypeInfo Construct(NominalTypeInfo definition, TypeInfo[] arguments)
    {
        ArgumentNullException.ThrowIfNull(arguments);
        if (definition.GenericArity == 0 || arguments.Length != definition.GenericArity ||
            arguments.Any(t => t is null || !ReferenceEquals(t.Context, this) || t is PrimitiveTypeInfo { Kind: PrimitiveType.Void } || t is NominalTypeInfo { GenericArity: > 0 } || t.Depth >= 16))
            throw new ArgumentException("invalid metadata type construction", nameof(arguments));
        var key = new ConstructionKey(definition, (TypeInfo[])arguments.Clone());
        lock (gate)
        {
            if (!constructions.TryGetValue(key, out var view)) constructions.Add(key, view = new ConstructedTypeInfo(this, definition, key.Arguments));
            return view;
        }
    }

    internal IReadOnlyList<FieldInfo> ProjectFields(TypeDefinition definition, TypeInfo owner, IReadOnlyList<TypeInfo> arguments)
        => Array.AsReadOnly(definition.Fields.Select(field =>
        {
            if (!field.TryGetSignature(out var signature)) throw new InvalidDataException("unsupported field metadata signature: " + field.Name);
            return new FieldInfo(field, owner, ResolveSignature(signature!, arguments));
        }).ToArray());

    private sealed record FunctionKey(TypeInfo Result, TypeInfo[] Parameters, bool NoResult)
    {
        public bool Equals(FunctionKey? other) => other is not null && ReferenceEquals(Result, other.Result) && NoResult == other.NoResult && Parameters.SequenceEqual(other.Parameters);
        public override int GetHashCode() { var hash = new HashCode(); hash.Add(Result); hash.Add(NoResult); foreach (var p in Parameters) hash.Add(p); return hash.ToHashCode(); }
    }

    private sealed class ConstructionKey(NominalTypeInfo definition, TypeInfo[] arguments) : IEquatable<ConstructionKey>
    {
        internal TypeInfo[] Arguments { get; } = arguments;
        private NominalTypeInfo Definition { get; } = definition;
        public bool Equals(ConstructionKey? other) => other is not null && ReferenceEquals(Definition, other.Definition) && Arguments.SequenceEqual(other.Arguments);
        public override bool Equals(object? obj) => obj is ConstructionKey other && Equals(other);
        public override int GetHashCode() { var hash = new HashCode(); hash.Add(Definition); foreach (var argument in Arguments) hash.Add(argument); return hash.ToHashCode(); }
    }
}
