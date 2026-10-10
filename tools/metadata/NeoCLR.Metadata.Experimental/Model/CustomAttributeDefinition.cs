using System.Reflection.Metadata;
using System.Text;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A typed immutable attribute value: String, Int32, Boolean or a nominal Int32-backed enum.</summary>
public sealed class CustomAttributeArgument
{
    /// <summary>Creates a supported primitive or nominal enum argument; only String permits null.</summary>
    /// <remarks>External enum identities are validated against the explicit catalog when linking.</remarks>
    /// <exception cref="ArgumentException">The type/value combination is unsupported.</exception>
    public CustomAttributeArgument(SignatureType type, object? value)
    {
        ArgumentNullException.ThrowIfNull(type);
        bool nominal = type.ClassType is { IsEnum: true } ||
            type.ImportedType is { IsValueType: true, GenericArity: 0 } || type.ReferencedType is not null;
        if (!(type.Primitive == PrimitiveType.String && value is null or string ||
              type.Primitive == PrimitiveType.Int32 && value is int ||
              type.Primitive == PrimitiveType.Boolean && value is bool || nominal && value is int))
            throw new ArgumentException("unsupported custom attribute argument");
        if (value is string text)
        {
            if (text.Length > 65536) throw new ArgumentException("attribute string exceeds limit");
            _ = new UTF8Encoding(false, true).GetByteCount(text);
        }
        Type = type; Value = value;
    }
    /// <summary>Gets the declared argument type, preserving nominal enum identity.</summary>
    public SignatureType Type { get; }
    /// <summary>Gets the immutable value. Enum values carry their Int32 storage value.</summary>
    public object? Value { get; }
}

/// <summary>A named field or property assignment retained as metadata, never executed during inspection.</summary>
public sealed class CustomAttributeNamedArgument
{
    /// <summary>Creates a named primitive field/property value.</summary>
    /// <exception cref="ArgumentException">The name is invalid or the value has an unsupported named-argument type.</exception>
    public CustomAttributeNamedArgument(string name, bool isField, CustomAttributeArgument value)
    {
        ArgumentNullException.ThrowIfNull(value);
        if (string.IsNullOrWhiteSpace(name) || name.Length > 1024 || name.Any(char.IsControl) || value.Type.Primitive is null)
            throw new ArgumentException("unsupported named attribute argument");
        MemberName = name; IsField = isField; TypedValue = value;
    }
    /// <summary>Gets the exact metadata member name.</summary>
    public string MemberName { get; }
    /// <summary>Gets true for a field assignment, false for a property assignment.</summary>
    public bool IsField { get; }
    /// <summary>Gets the immutable typed value.</summary>
    public CustomAttributeArgument TypedValue { get; }
}

/// <summary>Metadata-only custom attribute data. Reading never invokes constructors or named assignments.</summary>
public sealed class CustomAttributeDefinition
{
    private readonly byte[] signature;
    private readonly byte[] value;
    private readonly TypeReference? authoredType;
    private readonly ModuleDefinition? module;
    private readonly uint constructor;
    private readonly IReadOnlyList<CustomAttributeArgument>? authoredArguments;
    private readonly IReadOnlyList<CustomAttributeNamedArgument>? authoredNamedArguments;

    /// <summary>Authors an explicit nominal constructor and fixed/named data.</summary>
    /// <remarks>Fixed values support String, Int32, Boolean and Int32-backed enums. Named values currently support
    /// String, Int32 and Boolean. Owners must be top-level nongeneric nominal types. Arrays and Type values remain unsupported.</remarks>
    /// <exception cref="ArgumentException">Unsupported owner, data or bounds.</exception>
    public CustomAttributeDefinition(TypeReference attributeType, IEnumerable<CustomAttributeArgument> arguments,
        IEnumerable<CustomAttributeNamedArgument>? namedArguments = null)
    {
        ArgumentNullException.ThrowIfNull(attributeType); ArgumentNullException.ThrowIfNull(arguments);
        var copied = arguments.Take(257).ToArray();
        var named = (namedArguments ?? []).Take(257).ToArray();
        if (copied.Length > 256 || copied.Any(a => a is null) || named.Length > 256 || named.Any(a => a is null) ||
            named.Select(a => a.MemberName).Distinct(StringComparer.Ordinal).Count() != named.Length ||
            attributeType.Name.Contains('`') || attributeType.ResolutionScopeToken >> 24 == 1)
            throw new ArgumentException("unsupported custom attribute contract");
        if (copied.Concat(named.Select(n => n.TypedValue)).Sum(a => a.Value is string text ? (long)Encoding.UTF8.GetByteCount(text) + 4 : 4) > 1048576)
            throw new ArgumentException("attribute data exceeds limit");
        authoredType = attributeType;
        authoredArguments = Array.AsReadOnly(copied); authoredNamedArguments = Array.AsReadOnly(named);
        var sig = new BlobBuilder(); sig.WriteByte(0x20); sig.WriteCompressedInteger(copied.Length); sig.WriteByte(1);
        var blob = new BlobBuilder(); blob.WriteUInt16(1);
        foreach (var argument in copied)
        {
            if (argument.Type.Primitive is { } primitive) sig.WriteByte(TypeCode(primitive));
            WriteValue(blob, argument);
        }
        blob.WriteUInt16((ushort)named.Length);
        foreach (var argument in named)
        {
            blob.WriteByte(argument.IsField ? (byte)0x53 : (byte)0x54);
            blob.WriteByte(TypeCode(argument.TypedValue.Type.Primitive!.Value));
            blob.WriteSerializedString(argument.MemberName);
            WriteValue(blob, argument.TypedValue);
        }
        signature = copied.All(a => a.Type.Primitive is not null) ? sig.ToArray() : [];
        value = blob.ToArray();
    }
    /// <summary>Authors data for an output-owned constructor, checking its full signature.</summary>
    public CustomAttributeDefinition(MethodDefinition constructor, IEnumerable<CustomAttributeArgument> arguments,
        IEnumerable<CustomAttributeNamedArgument>? namedArguments = null) : this(OwnedConstructorType(constructor), arguments, namedArguments)
    { ValidateSignature(constructor.Producer!.Signature); }
    /// <summary>Authors data for an imported constructor, checking its full signature.</summary>
    public CustomAttributeDefinition(ImportedMethodReference constructor, IEnumerable<CustomAttributeArgument> arguments,
        IEnumerable<CustomAttributeNamedArgument>? namedArguments = null) : this(ImportedConstructorType(constructor), arguments, namedArguments)
    { ValidateSignature(constructor.Signature); }
    private static TypeReference OwnedConstructorType(MethodDefinition constructor)
    {
        ArgumentNullException.ThrowIfNull(constructor);
        if (constructor.Producer is not { IsConstructor: true } method || method.DeclaringType!.GenericParameterNames.Count != 0 || method.DeclaringType.IsValueType || method.DeclaringType.Definition.DeclaringType is not null || method.Visibility != MethodVisibility.Public)
            throw new ArgumentException("expected an owned nongeneric constructor");
        return constructor.DeclaringType!.ToReference();
    }
    private static TypeReference ImportedConstructorType(ImportedMethodReference constructor)
    {
        ArgumentNullException.ThrowIfNull(constructor);
        if (!constructor.IsConstructor || constructor.Target.DeclaringType!.IsValueType || constructor.Target.DeclaringType.GenericParameterNames.Count != 0 || constructor.Target.DeclaringType.Definition.DeclaringType is not null)
            throw new ArgumentException("expected an imported top-level nongeneric constructor");
        var owner = constructor.Target.DeclaringType;
        return constructor.Owner.Definition.MainModule.ImportReference(owner.Assembly.Identity, owner.Namespace, owner.Name);
    }

    private void ValidateSignature(MethodSignature constructor)
    {
        if (!constructor.Matches(new MethodSignature(PrimitiveType.Void, GetArguments().Select(a => a.Type))))
            throw new ArgumentException("attribute arguments do not match constructor signature");
    }
    internal CustomAttributeDefinition(ModuleDefinition module, uint constructor, byte[] signature, byte[] value)
    { this.module = module; this.constructor = constructor; this.signature = (byte[])signature.Clone(); this.value = (byte[])value.Clone(); }
    /// <summary>Gets the constructor's nominal owner without resolving or loading its assembly.</summary>
    /// <exception cref="InvalidDataException">The snapshot uses an unsupported constructor parent.</exception>
    public TypeReference AttributeType
    {
        get
        {
            if (authoredType is not null) return authoredType;
            if (constructor >> 24 == 6)
                return (module!.GetMethodDefinition(constructor)?.DeclaringType ?? throw new InvalidDataException("attribute constructor has no declaring type")).ToReference();
            var member = module!.MemberReferences.SingleOrDefault(m => m.MetadataToken == constructor)
                ?? throw new InvalidDataException("attribute constructor reference missing");
            if (member.ParentToken >> 24 == 2) return (module.GetTypeDefinition(member.ParentToken) ?? throw new InvalidDataException("attribute owner missing")).ToReference();
            return module.TypeReferences.SingleOrDefault(t => t.MetadataToken == member.ParentToken)
                ?? throw new InvalidDataException("unsupported attribute owner");
        }
    }

    /// <summary>Copies the stored CLI signature for loaded or primitive-only authored attributes.</summary>
    /// <exception cref="NotSupportedException">Authored nominal parameters need output token assignment; inspect GetArguments instead.</exception>
    public byte[] GetConstructorSignature() => signature.Length != 0 ? (byte[])signature.Clone()
        : throw new NotSupportedException("nominal attribute signatures require output token assignment");
    /// <summary>Copies the CLI attribute payload, including named data.</summary>
    public byte[] GetValue() => (byte[])value.Clone();
    /// <summary>Gets typed fixed data without invoking the constructor.</summary>
    public IReadOnlyList<CustomAttributeArgument> GetArguments() => authoredArguments ?? Decode().Arguments;
    /// <summary>Gets typed named data without invoking setters or writing fields.</summary>
    public IReadOnlyList<CustomAttributeNamedArgument> GetNamedArguments() => authoredNamedArguments ?? Decode().Named;

    private (IReadOnlyList<CustomAttributeArgument> Arguments, IReadOnlyList<CustomAttributeNamedArgument> Named) Decode()
    {
        var s = new AttributeBlobReader(signature); var v = new AttributeBlobReader(value);
        if (s.Byte() != 0x20) throw new NotSupportedException("attribute constructor calling convention");
        int count = s.Compressed();
        if (count > 256 || s.Byte() != 1) throw new NotSupportedException("attribute constructor signature");
        if (v.UInt16() != 1) throw new InvalidDataException("attribute prolog");
        var result = new List<CustomAttributeArgument>();
        for (int i = 0; i < count; i++)
        {
            int code = s.Byte();
            SignatureType type;
            if (code == 0x11)
            {
                uint coded = (uint)s.Compressed();
                var reference = (coded & 3) switch
                {
                    0 => module!.GetTypeDefinition(0x02000000u | coded >> 2)?.ToReference(),
                    1 => module!.TypeReferences.SingleOrDefault(t => t.MetadataToken == (0x01000000u | coded >> 2)),
                    _ => null
                } ?? throw new InvalidDataException("invalid enum attribute parameter");
                type = SignatureType.FromReference(reference);
            }
            else type = Primitive(code);
            result.Add(new(type, ReadValue(v, type.Primitive)));
        }
        if (!s.End) throw new InvalidDataException("trailing constructor signature data");
        int namedCount = v.UInt16();
        if (namedCount > 256) throw new InvalidDataException("too many named attribute arguments");
        var named = new List<CustomAttributeNamedArgument>();
        var names = new HashSet<string>(StringComparer.Ordinal);
        for (int i = 0; i < namedCount; i++)
        {
            int kind = v.Byte();
            if (kind is not (0x53 or 0x54)) throw new InvalidDataException("invalid named attribute kind");
            var type = Primitive(v.Byte());
            string name = v.String() ?? throw new InvalidDataException("null named attribute name");
            if (!names.Add(name)) throw new InvalidDataException("duplicate named attribute argument");
            named.Add(new(name, kind == 0x53, new(type, ReadValue(v, type))));
        }
        if (!v.End) throw new InvalidDataException("trailing custom attribute data");
        return (result.AsReadOnly(), named.AsReadOnly());
    }
    private static PrimitiveType Primitive(int code) => code switch
    { 0x0e => PrimitiveType.String, 8 => PrimitiveType.Int32, 2 => PrimitiveType.Boolean, _ => throw new NotSupportedException("attribute argument category") };
    private static byte TypeCode(PrimitiveType type) => type switch
    { PrimitiveType.String => 0x0e, PrimitiveType.Int32 => 8, PrimitiveType.Boolean => 2, _ => throw new NotSupportedException("attribute argument category") };
    private static object? ReadValue(AttributeBlobReader reader, PrimitiveType? type) => type switch
    {
        PrimitiveType.String => reader.String(),
        PrimitiveType.Boolean => reader.Byte() switch { 0 => false, 1 => true, _ => throw new InvalidDataException("attribute Boolean") },
        _ => reader.Int32()
    };
    private static void WriteValue(BlobBuilder blob, CustomAttributeArgument argument)
    {
        if (argument.Type.Primitive == PrimitiveType.String) blob.WriteSerializedString((string?)argument.Value);
        else if (argument.Type.Primitive == PrimitiveType.Boolean) blob.WriteByte((bool)argument.Value! ? (byte)1 : (byte)0);
        else blob.WriteInt32((int)argument.Value!);
    }
    internal void ValidateOwner(TypeDefinition owner) => ValidateOwner(owner.Module);
    internal void ValidateOwner(ModuleDefinition owner)
    {
        if (authoredType is null || !ReferenceEquals(authoredType.Module, owner)) throw new ArgumentException("attribute reference must be authored in the owning module");
        if (authoredType.ExplicitScope is null && authoredType.Resolve().DeclaringType is not null) throw new ArgumentException("nested attribute owner unsupported");
        foreach (var argument in GetArguments()) argument.Type.ValidateOwner(owner.Assembly.Producer ?? throw new InvalidOperationException("loaded attribute owner"));
    }
    internal void ValidateContract(TypeDefinition owner) => ValidateContract(owner.Module);
    internal void ValidateContract(ModuleDefinition owner)
    {
        ValidateOwner(owner);
        if (AttributeType.ExplicitScope is not null) return;
        var type = AttributeType.Resolve();
        var expected = new MethodSignature(PrimitiveType.Void, GetArguments().Select(a => a.Type));
        if (type.Producer is not { IsStatic: false, IsValueType: false } producer || type.GenericArity != 0 ||
            !producer.Methods.Any(m => m.IsConstructor && m.Visibility == MethodVisibility.Public && m.Signature.Matches(expected)))
            throw new InvalidDataException("missing or incompatible local attribute constructor");
        foreach (var argument in GetNamedArguments())
        {
            bool valid = argument.IsField
                ? producer.MetadataFields.Any(f => f.Name == argument.MemberName && f.Visibility == FieldVisibility.Public && !f.Definition.IsLiteral && (f.Definition.Attributes & 0x30) == 0 && f.FieldType.Equals(argument.TypedValue.Type))
                : producer.Properties.Any(p => p.Name == argument.MemberName && p.SetMethod is { IsStatic: false, Visibility: MethodVisibility.Public } setter && p.GetMethod is { IsStatic: false, Visibility: MethodVisibility.Public } && p.ParameterTypes.Count == 0 && p.PropertyType.Equals(argument.TypedValue.Type));
            if (!valid) throw new InvalidDataException("invalid named attribute member: " + argument.MemberName);
        }
    }
    private sealed class AttributeBlobReader(byte[] data)
    {
        private int position;
        internal bool End => position == data.Length;
        internal byte Byte() => position < data.Length ? data[position++] : throw new InvalidDataException("truncated attribute data");
        internal int UInt16() => Byte() | Byte() << 8;
        internal int Int32() => UInt16() | UInt16() << 16;
        internal int Compressed()
        {
            int b = Byte();
            if (b < 0x80) return b;
            if (b < 0xc0) return (b & 0x3f) << 8 | Byte();
            if (b < 0xe0) return (b & 0x1f) << 24 | Byte() << 16 | Byte() << 8 | Byte();
            throw new InvalidDataException("invalid compressed attribute integer");
        }
        internal string? String()
        {
            if (position < data.Length && data[position] == 255) { position++; return null; }
            int count = Compressed();
            if (count > data.Length - position || count > 262144) throw new InvalidDataException("attribute string exceeds bounds");
            try { var text = new UTF8Encoding(false, true).GetString(data, position, count); if (text.Length > 65536) throw new InvalidDataException("attribute string exceeds limit"); position += count; return text; }
            catch (DecoderFallbackException error) { throw new InvalidDataException("attribute string is not UTF-8", error); }
        }
    }
}

public sealed partial class TypeDefinition
{
    private IList<CustomAttributeDefinition>? customAttributes;
    /// <summary>Gets type custom attributes. Authored definitions append attributes; loaded snapshots are read-only.</summary>
    public IList<CustomAttributeDefinition> CustomAttributes => customAttributes ??= authoredFields is null
        ? Array.AsReadOnly(Array.Empty<CustomAttributeDefinition>())
        : new DefinitionCollection<CustomAttributeDefinition>([], a => { if (customAttributes!.Count >= 256) throw new InvalidDataException("too many type attributes"); a.ValidateOwner(this); });
    internal void SetLoadedAttributes(IEnumerable<CustomAttributeDefinition> attributes) => customAttributes = Array.AsReadOnly(attributes.ToArray());
}

public sealed partial class TypeBuilder
{
    /// <summary>Appends an attribute through the definition's ownership validation.</summary>
    public void AddCustomAttribute(CustomAttributeDefinition attribute) => Definition.CustomAttributes.Add(attribute);
}

public sealed partial class MethodDefinition
{
    private IList<CustomAttributeDefinition>? customAttributes;
    /// <summary>Gets declared method or assembly-function attributes, excluding parameter attributes.</summary>
    /// <remarks>Authored definitions accept output-owned attributes. Loaded snapshots are read-only.
    /// Reading data never invokes an attribute constructor.</remarks>
    public IList<CustomAttributeDefinition> CustomAttributes => customAttributes ??= AuthoredSignature is null
        ? Array.AsReadOnly(Array.Empty<CustomAttributeDefinition>())
        : new DefinitionCollection<CustomAttributeDefinition>([], attribute =>
        {
            if (customAttributes!.Count >= 256) throw new InvalidDataException("too many method attributes");
            attribute.ValidateOwner(Module);
        });
    internal void SetLoadedAttributes(IEnumerable<CustomAttributeDefinition> attributes) => customAttributes = Array.AsReadOnly(attributes.ToArray());
}

public sealed partial class MethodBuilder
{
    /// <summary>Adds a declared attribute using the callable's output-module ownership.</summary>
    /// <exception cref="ArgumentException">The attribute belongs to another module.</exception>
    public void AddCustomAttribute(CustomAttributeDefinition attribute) => Definition.CustomAttributes.Add(attribute);
}

public sealed partial class FieldDefinition
{
    private IList<CustomAttributeDefinition>? customAttributes;
    /// <summary>Gets declared field attributes. Loaded snapshots are read-only; inspection never invokes constructors.</summary>
    public IList<CustomAttributeDefinition> CustomAttributes => customAttributes ??= FieldType is null
        ? Array.AsReadOnly(Array.Empty<CustomAttributeDefinition>())
        : new DefinitionCollection<CustomAttributeDefinition>([], attribute =>
        {
            if (customAttributes!.Count >= 256) throw new InvalidDataException("too many field attributes");
            attribute.ValidateOwner(Module);
        });
    internal void SetLoadedAttributes(IEnumerable<CustomAttributeDefinition> attributes) => customAttributes = Array.AsReadOnly(attributes.ToArray());
}

public sealed partial class PropertyDefinition
{
    private IList<CustomAttributeDefinition>? customAttributes;
    /// <summary>Gets declared property attributes. Loaded snapshots are read-only; inspection never invokes constructors.</summary>
    public IList<CustomAttributeDefinition> CustomAttributes => customAttributes ??= PropertyType is null
        ? Array.AsReadOnly(Array.Empty<CustomAttributeDefinition>())
        : new DefinitionCollection<CustomAttributeDefinition>([], attribute =>
        {
            if (customAttributes!.Count >= 256) throw new InvalidDataException("too many property attributes");
            attribute.ValidateOwner(Module);
        });
    internal void SetLoadedAttributes(IEnumerable<CustomAttributeDefinition> attributes) => customAttributes = Array.AsReadOnly(attributes.ToArray());
}

public sealed partial class MethodDefinition
{
    private readonly Dictionary<int, IList<CustomAttributeDefinition>> parameterAttributes = [];
    internal IEnumerable<KeyValuePair<int, IList<CustomAttributeDefinition>>> ParameterAttributes => parameterAttributes;
    /// <summary>Gets attributes declared on a zero-based parameter, excluding a receiver. Loaded lists are read-only.</summary>
    /// <exception cref="ArgumentOutOfRangeException">The position is outside the supported method signature.</exception>
    /// <exception cref="NotSupportedException">The method signature cannot be decoded.</exception>
    public IList<CustomAttributeDefinition> GetParameterCustomAttributes(int position)
    {
        if (!TryGetSignature(out var method)) throw new NotSupportedException("unsupported parameter signature");
        if (position < 0 || position >= method!.ParameterTypes.Count) throw new ArgumentOutOfRangeException(nameof(position));
        if (!parameterAttributes.TryGetValue(position, out var attributes))
        {
            attributes = AuthoredSignature is null ? Array.AsReadOnly(Array.Empty<CustomAttributeDefinition>())
                : new DefinitionCollection<CustomAttributeDefinition>([], attribute =>
                {
                    if (parameterAttributes[position].Count >= 256) throw new InvalidDataException("too many parameter attributes");
                    attribute.ValidateOwner(Module);
                });
            parameterAttributes.Add(position, attributes);
        }
        return attributes;
    }
    internal void SetLoadedParameterAttributes(int position, IEnumerable<CustomAttributeDefinition> attributes)
        => parameterAttributes[position] = Array.AsReadOnly(attributes.ToArray());
}

public sealed partial class AssemblyBuilder
{
    private bool HasParameterArrayAttribute(MethodBuilder method, int position) => method.Definition.GetParameterCustomAttributes(position)
        .Any(attribute => attribute.AttributeType.Namespace == "System" && attribute.AttributeType.Name == "ParamArrayAttribute" &&
            (attribute.AttributeType.ExplicitScope ?? attribute.AttributeType.Module.Assembly.Identity).Equals(CoreLibrary));
}
