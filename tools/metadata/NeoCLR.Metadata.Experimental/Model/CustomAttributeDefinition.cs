using System.Reflection.Metadata;
using System.Text;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>A typed fixed constructor argument. The initial profile supports String, Int32 and Boolean.</summary>
public sealed class CustomAttributeArgument
{
    /// <summary>Creates an immutable argument; only String permits a null value.</summary>
    /// <exception cref="ArgumentException">The type/value combination is unsupported.</exception>
    public CustomAttributeArgument(PrimitiveType type, object? value)
    {
        if (!(type == PrimitiveType.String && value is null or string || type == PrimitiveType.Int32 && value is int || type == PrimitiveType.Boolean && value is bool))
            throw new ArgumentException("unsupported custom attribute argument");
        if (value is string text)
        {
            if (text.Length > 65536) throw new ArgumentException("attribute string exceeds limit");
            _ = new UTF8Encoding(false, true).GetByteCount(text);
        }
        Type = type; Value = value;
    }
    /// <summary>Gets the constructor parameter's primitive type.</summary>
    public PrimitiveType Type { get; }
    /// <summary>Gets the immutable String, Int32, Boolean or null String value.</summary>
    public object? Value { get; }
}

/// <summary>Metadata-only custom attribute data. Reading never invokes a constructor.</summary>
public sealed class CustomAttributeDefinition
{
    private readonly byte[] signature;
    private readonly byte[] value;
    private readonly TypeReference? authoredType;
    private readonly ModuleDefinition? module;
    private readonly uint constructor;
    /// <summary>Authors a .ctor reference with an explicit nominal owner and fixed argument types.</summary>
    /// <remarks>Only top-level nongeneric attribute owners and String/Int32/Boolean fixed arguments are supported.
    /// No dependency is loaded; the host must supply the actual constructor when linking/executing.
    /// Named arguments, arrays, enum arguments and System.Type arguments remain unsupported for authoring.</remarks>
    /// <exception cref="ArgumentException">Unsupported owner, arguments or count.</exception>
    public CustomAttributeDefinition(TypeReference attributeType, IEnumerable<CustomAttributeArgument> arguments)
    {
        ArgumentNullException.ThrowIfNull(attributeType); ArgumentNullException.ThrowIfNull(arguments);
        var copied = arguments.Take(257).ToArray();
        if (copied.Length > 256 || copied.Any(a => a is null) || attributeType.Name.Contains('`') || attributeType.ResolutionScopeToken >> 24 == 1)
            throw new ArgumentException("unsupported custom attribute contract");
        if (copied.Sum(a => a.Value is string text ? (long)Encoding.UTF8.GetByteCount(text) + 4 : 4) > 1048576)
            throw new ArgumentException("attribute data exceeds limit");
        authoredType = attributeType;
        var sig = new BlobBuilder(); sig.WriteByte(0x20); sig.WriteCompressedInteger(copied.Length); sig.WriteByte(1);
        var blob = new BlobBuilder(); blob.WriteUInt16(1);
        foreach (var argument in copied)
        {
            sig.WriteByte(argument.Type == PrimitiveType.String ? (byte)0x0e : argument.Type == PrimitiveType.Int32 ? (byte)8 : (byte)2);
            if (argument.Type == PrimitiveType.String) blob.WriteSerializedString((string?)argument.Value);
            else if (argument.Type == PrimitiveType.Int32) blob.WriteInt32((int)argument.Value!);
            else blob.WriteByte((bool)argument.Value! ? (byte)1 : (byte)0);
        }
        blob.WriteUInt16(0); signature = sig.ToArray(); value = blob.ToArray();
    }
    /// <summary>Authors an attribute from an owned constructor definition and matching fixed arguments.</summary>
    /// <exception cref="ArgumentException">The method is not an owned supported constructor or arguments do not match.</exception>
    public CustomAttributeDefinition(MethodDefinition constructor, IEnumerable<CustomAttributeArgument> arguments)
        : this(OwnedConstructorType(constructor), arguments)
    {
        ValidateSignature(constructor.Producer!.Signature);
    }
    /// <summary>Authors an attribute from an output-owned imported constructor and matching fixed arguments.</summary>
    /// <exception cref="ArgumentException">The method is not a supported constructor or arguments do not match.</exception>
    public CustomAttributeDefinition(ImportedMethodReference constructor, IEnumerable<CustomAttributeArgument> arguments)
        : this(ImportedConstructorType(constructor), arguments)
    {
        ValidateSignature(constructor.Signature);
    }
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
        if (!constructor.Matches(new MethodSignature(PrimitiveType.Void, GetArguments().Select(a => (SignatureType)a.Type))))
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
    /// <summary>Copies the CLI instance constructor signature, preserving unsupported signatures on loaded data.</summary>
    public byte[] GetConstructorSignature() => (byte[])signature.Clone();
    /// <summary>Copies the CLI custom attribute blob, preserving unsupported named/fixed arguments on loaded data.</summary>
    public byte[] GetValue() => (byte[])value.Clone();
    /// <summary>Decodes the supported fixed arguments without reflection or dependency loading.</summary>
    /// <exception cref="NotSupportedException">The constructor or argument category is outside the bounded profile.</exception>
    /// <exception cref="InvalidDataException">Malformed supported data.</exception>
    public IReadOnlyList<CustomAttributeArgument> GetArguments()
    {
        var s = new AttributeBlobReader(signature); var v = new AttributeBlobReader(value);
        if (s.Byte() != 0x20) throw new NotSupportedException("attribute constructor calling convention");
        int count = s.Compressed();
        if (count > 256 || s.Byte() != 1) throw new NotSupportedException("attribute constructor signature");
        if (v.UInt16() != 1) throw new InvalidDataException("attribute prolog");
        var result = new List<CustomAttributeArgument>();
        for (int i = 0; i < count; i++)
        {
            switch (s.Byte())
            {
                case 0x0e: result.Add(new(PrimitiveType.String, v.String())); break;
                case 8: result.Add(new(PrimitiveType.Int32, v.Int32())); break;
                case 2:
                    int boolean = v.Byte(); if (boolean > 1) throw new InvalidDataException("attribute Boolean");
                    result.Add(new(PrimitiveType.Boolean, boolean == 1)); break;
                default: throw new NotSupportedException("attribute fixed argument category");
            }
        }
        if (!s.End) throw new InvalidDataException("trailing constructor signature data");
        if (v.UInt16() != 0) throw new NotSupportedException("named attribute arguments");
        if (!v.End) throw new InvalidDataException("trailing custom attribute data");
        return result.AsReadOnly();
    }
    internal void ValidateOwner(TypeDefinition owner)
    {
        if (authoredType is null || !ReferenceEquals(authoredType.Module, owner.Module)) throw new ArgumentException("attribute reference must be authored in the owning module");
        if (authoredType.ExplicitScope is null && authoredType.Resolve().DeclaringType is not null) throw new ArgumentException("nested attribute owner unsupported");
    }
    internal void ValidateContract(TypeDefinition owner)
    {
        ValidateOwner(owner);
        if (AttributeType.ExplicitScope is not null) return; // Linked against the explicit external catalog/runtime.
        var type = AttributeType.Resolve();
        var expected = new MethodSignature(PrimitiveType.Void, GetArguments().Select(a => (SignatureType)a.Type));
        if (type.Producer is not { IsStatic: false, IsValueType: false } producer || type.GenericArity != 0 ||
            !producer.Methods.Any(m => m.IsConstructor && m.Visibility == MethodVisibility.Public && m.Signature.Matches(expected)))
            throw new InvalidDataException("missing or incompatible local attribute constructor");
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
