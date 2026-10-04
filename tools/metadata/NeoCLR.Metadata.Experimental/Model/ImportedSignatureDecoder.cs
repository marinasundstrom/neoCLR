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
            case 0x03: return consumer.ImportCharacterSignature(core);
            case 0x08: return PrimitiveType.Int32;
            case 0x05: return PrimitiveType.Byte;
            case 0x04: return PrimitiveType.SByte;
            case 0x06: return PrimitiveType.Int16;
            case 0x07: return PrimitiveType.UInt16;
            case 0x09: return PrimitiveType.UInt32;
            case 0x0b: return PrimitiveType.UInt64;

            case 0x0c: return PrimitiveType.Single;
            case 0x0d: return PrimitiveType.Double;
            case 0x0a: return PrimitiveType.Int64;
            case 0x0e: return PrimitiveType.String;
            case 0x1c: return consumer.CoreObjectType;
            case 0x13:
                var ownerOrdinal = Number();
                if (ownerOrdinal >= ownerArity) throw new InvalidDataException("unscoped imported owner parameter");
                return SignatureType.TypeParameter(ownerOrdinal);
            case 0x1e:
                var ordinal = Number();
                if (ordinal >= arity) throw new InvalidDataException("unscoped imported method parameter");
                return SignatureType.MethodParameter(ordinal);
            case 0x1d: return SignatureType.ArrayOf(Type(false, depth + 1));
            case 0x12:
                var classToken = Number();
                if (FunctionCarrier(classToken, out var actionArity, out var action) && action && actionArity == 0)
                    return SignatureType.Function(new MethodSignature(PrimitiveType.Void, []));
                return Nominal(false, classToken);
            case 0x11:
                var valueToken = Number();
                if (IsRuntimeTypeHandle(valueToken)) return PrimitiveType.RuntimeTypeHandle;
                return Nominal(true, valueToken);
            case 0x15:
                var category = Byte();
                if (category is not (0x11 or 0x12)) throw new InvalidDataException("invalid nominal signature category");
                var definitionToken = Number();
                if (category == 0x12 && FunctionCarrier(definitionToken, out var carrierArity, out var noResult))
                {
                    var argumentCount = Number();
                    if (argumentCount == 0 || argumentCount != carrierArity) throw new InvalidDataException("function carrier arity mismatch");
                    var carrierArguments = new SignatureType[argumentCount];
                    for (int i = 0; i < argumentCount; i++) carrierArguments[i] = Type(false, depth + 1);
                    return SignatureType.Function(new MethodSignature(noResult ? PrimitiveType.Void : carrierArguments[^1], noResult ? carrierArguments : carrierArguments[..^1]));
                }
                var definition = Nominal(category == 0x11, definitionToken);
                var count = Number();
                if (count == 0 || count != definition.GenericArity) throw new InvalidDataException("imported construction arity mismatch");
                var arguments = new SignatureType[count];
                for (var i = 0; i < count; i++) arguments[i] = Type(false, depth + 1);
                return definition.MakeGenericInstance(arguments);
            default: throw new InvalidDataException("unsupported imported signature type");
        }
    }

    private bool IsRuntimeTypeHandle(int token)
    {
        if ((token & 3) != 1 || token >> 2 == 0) return false;
        var reference = module.TypeReferences.SingleOrDefault(t => t.MetadataToken == (0x01000000u | (uint)(token >> 2)));
        var coreIdentity = core;
        return reference is { Namespace: "System", Name: "RuntimeTypeHandle" } &&
            module.AssemblyReferences.Any(a => a.MetadataToken == reference.ResolutionScopeToken && a.Identity.Equals(coreIdentity));
    }

    private ImportedTypeReference Nominal(bool valueType, int token)
    {
        // Module-scoped TypeRefs are another encoding of a dependency-local definition.
        // Assembly-scoped references still require an explicit resolver and are rejected.
        var definition = (token & 3) switch
        {
            0 when token >> 2 != 0 => module.GetTypeDefinition(0x02000000u | (uint)(token >> 2)),
            1 when token >> 2 != 0 => LocalReference(0x01000000u | (uint)(token >> 2), 0),
            _ => null
        } ?? throw new InvalidDataException("missing dependency-local nominal signature definition");
        if (definition.IsValueType != valueType) throw new InvalidDataException("nominal signature category disagrees with declaration");
        return consumer.ImportReference(definition, core);
    }

    private TypeDefinition LocalReference(uint token, int depth)
    {
        if (depth >= 16) throw new InvalidDataException("imported declaring reference nesting limit");
        var row = module.TypeReferences.SingleOrDefault(r => r.MetadataToken == token)
            ?? throw new InvalidDataException("missing imported TypeRef");
        var parent = row.ResolutionScopeToken == 1 ? null : row.ResolutionScopeToken >> 24 == 1
            ? LocalReference(row.ResolutionScopeToken, depth + 1)
            : throw new InvalidDataException("imported nominal signature requires a dependency-local scope");
        var candidates = module.Types.Where(t => t.Namespace == row.Namespace && t.Name == row.Name && ReferenceEquals(t.DeclaringType, parent)).Take(2).ToArray();
        return candidates.Length == 1 ? candidates[0] : throw new InvalidDataException("missing or ambiguous imported local TypeRef");
    }

    private bool FunctionCarrier(int token, out int count, out bool action)
    {
        count = 0; action = false;
        string name;
        if ((token & 3) == 1 && token >> 2 != 0)
        {
            var coreIdentity = core;
            var reference = module.TypeReferences.SingleOrDefault(t => t.MetadataToken == (0x01000000u | (uint)(token >> 2)));
            if (reference is null || reference.Namespace != "System" ||
                !(reference.ResolutionScopeToken == 1 && module.Assembly.Identity.Equals(coreIdentity)) &&
                !module.AssemblyReferences.Any(a => a.MetadataToken == reference.ResolutionScopeToken && a.Identity.Equals(coreIdentity))) return false;
            name = reference.Name;
        }
        else if ((token & 3) == 0 && token >> 2 != 0 && module.Assembly.Identity.Equals(core))
        {
            var definition = module.GetTypeDefinition(0x02000000u | (uint)(token >> 2));
            if (definition is null || definition.Namespace != "System" || definition.DeclaringType is not null) return false;
            name = definition.Name;
        }
        else return false;
        action = name == "Action" || name.StartsWith("Action`", StringComparison.Ordinal);
        if (name == "Action") return true;
        var prefix = action ? "Action`" : "Func`";
        return name.StartsWith(prefix, StringComparison.Ordinal) && int.TryParse(name[prefix.Length..], out count) &&
            count >= 1 && count <= (action ? 16 : 17) && name == prefix + count;
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
