namespace NeoCLR.Metadata.Experimental.Model;

// Decode in the consuming scope: TypeDef tokens belong to the dependency snapshot,
// but the resulting references and constructed arguments belong to the output.
internal ref struct ImportedSignatureDecoder(ReadOnlySpan<byte> bytes, ModuleDefinition module,
    AssemblyBuilder consumer, AssemblyIdentity core, int arity, int ownerArity, bool instance)
{
    private readonly ReadOnlySpan<byte> signature = bytes;
    private int position;

    internal MethodSignature Read()
    {
        if (Byte() != ((arity == 0 ? 0 : 0x10) | (instance ? 0x20 : 0)) || arity > 0 && Number() != arity)
            throw new InvalidDataException("unsupported imported calling convention");
        var count = Number();
        if (count > 256) throw new InvalidDataException("imported parameter limit");
        var result = Type(true, 0);
        var parameters = new SignatureType[count];
        for (var i = 0; i < count; i++) parameters[i] = Type(false, 0, true);
        if (position != signature.Length) throw new InvalidDataException("trailing imported signature bytes");
        return new(result, parameters, Enumerable.Range(0, arity).Select(i => "T" + i));
    }

    private SignatureType Type(bool allowVoid, int depth, bool allowByReference = false)
    {
        if (depth >= 16) throw new InvalidDataException("imported signature nesting limit");
        switch (Byte())
        {
            case 0x10 when allowByReference: return SignatureType.ByReference(Type(false, depth + 1));
            case 0x01 when allowVoid: return PrimitiveType.Void;
            case 0x02: return PrimitiveType.Boolean;
            case 0x08: return PrimitiveType.Int32;
            case 0x0a: return PrimitiveType.Int64;
            case 0x0e: return PrimitiveType.String;
            case 0x13:
                var ownerOrdinal = Number();
                if (ownerOrdinal >= ownerArity) throw new InvalidDataException("unscoped imported owner parameter");
                return SignatureType.TypeParameter(ownerOrdinal);
            case 0x1e:
                var ordinal = Number();
                if (ordinal >= arity) throw new InvalidDataException("unscoped imported method parameter");
                return SignatureType.MethodParameter(ordinal);
            case 0x1d: return SignatureType.ArrayOf(Type(false, depth + 1));
            case 0x12: return Nominal(false);
            case 0x11: return Nominal(true);
            case 0x15:
                var category = Byte();
                if (category is not (0x11 or 0x12)) throw new InvalidDataException("invalid nominal signature category");
                var definition = Nominal(category == 0x11);
                var count = Number();
                if (count == 0 || count != definition.GenericArity) throw new InvalidDataException("imported construction arity mismatch");
                var arguments = new SignatureType[count];
                for (var i = 0; i < count; i++) arguments[i] = Type(false, depth + 1);
                return definition.MakeGenericInstance(arguments);
            default: throw new InvalidDataException("unsupported imported signature type");
        }
    }

    private ImportedTypeReference Nominal(bool valueType)
    {
        var token = Number();
        // Cross-dependency TypeRefs need an explicit resolver contract; never guess scope.
        if ((token & 3) != 0 || token >> 2 == 0)
            throw new InvalidDataException("imported nominal signature requires a dependency-local TypeDef");
        var definition = module.GetTypeDefinition(0x02000000u | (uint)(token >> 2))
            ?? throw new InvalidDataException("missing imported signature TypeDef");
        if (definition.IsValueType != valueType) throw new InvalidDataException("nominal signature category disagrees with declaration");
        return consumer.ImportReference(definition, core);
    }

    private byte Byte()
        => position < signature.Length ? signature[position++] : throw new InvalidDataException("truncated imported signature");

    private int Number()
    {
        var first = Byte();
        if (first < 0x80) return first;
        if ((first & 0xc0) == 0x80)
        {
            var value = ((first & 0x3f) << 8) | Byte();
            return value >= 0x80 ? value : throw new InvalidDataException("noncanonical signature integer");
        }
        if ((first & 0xe0) == 0xc0)
        {
            var value = ((first & 0x1f) << 24) | (Byte() << 16) | (Byte() << 8) | Byte();
            return value >= 0x4000 ? value : throw new InvalidDataException("noncanonical signature integer");
        }
        throw new InvalidDataException("invalid signature integer");
    }
}
