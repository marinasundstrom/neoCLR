using System.Collections.Immutable;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;

// Validate the encoded distinction before Cecil's primitive-type cache can blur it.
static class VoidStorageValidation
{
    public static void Check(string path)
    {
        using var pe = new PEReader(File.OpenRead(path));
        var metadata = pe.GetMetadataReader();
        var provider = new Provider();
        foreach (var handle in metadata.MethodDefinitions)
            CheckParameters(metadata.GetMethodDefinition(handle).DecodeSignature(provider, (object?)null));
        foreach (var handle in metadata.MemberReferences)
        {
            var member = metadata.GetMemberReference(handle);
            if (member.GetKind() == MemberReferenceKind.Method)
                CheckParameters(member.DecodeMethodSignature(provider, (object?)null));
            else RequireValue(member.DecodeFieldSignature(provider, (object?)null));
        }
        foreach (var handle in metadata.FieldDefinitions)
            RequireValue(metadata.GetFieldDefinition(handle).DecodeSignature(provider, (object?)null));
        foreach (var handle in metadata.MethodDefinitions)
        {
            var method = metadata.GetMethodDefinition(handle);
            if (method.RelativeVirtualAddress == 0) continue;
            var body = pe.GetMethodBody(method.RelativeVirtualAddress);
            if (!body.LocalSignature.IsNil)
                foreach (var type in metadata.GetStandaloneSignature(body.LocalSignature).DecodeLocalSignature(provider, (object?)null)) RequireValue(type);
        }
    }
    static void RequireValue(bool primitiveVoid)
    {
        if (primitiveVoid) throw new InvalidDataException("Unsupported Result profile type: CLI VOID marker is not value storage.");
    }
    static void CheckParameters(MethodSignature<bool> signature)
    {
        foreach (var type in signature.ParameterTypes) RequireValue(type);
    }
    sealed class Provider : ISignatureTypeProvider<bool, object?>
    {
        public bool GetPrimitiveType(PrimitiveTypeCode code) => code == PrimitiveTypeCode.Void;
        public bool GetTypeFromDefinition(MetadataReader reader, TypeDefinitionHandle handle, byte rawTypeKind) => false;
        public bool GetTypeFromReference(MetadataReader reader, TypeReferenceHandle handle, byte rawTypeKind) => false;
        public bool GetTypeFromSpecification(MetadataReader reader, object? context, TypeSpecificationHandle handle, byte rawTypeKind)
            => reader.GetTypeSpecification(handle).DecodeSignature(this, context);
        public bool GetGenericInstantiation(bool genericType, ImmutableArray<bool> arguments)
        { foreach (var type in arguments) RequireValue(type); return false; }
        public bool GetByReferenceType(bool element) { RequireValue(element); return false; }
        public bool GetArrayType(bool element, ArrayShape shape) { RequireValue(element); return false; }
        public bool GetSZArrayType(bool element) { RequireValue(element); return false; }
        public bool GetPointerType(bool element) => false;
        public bool GetPinnedType(bool element) => element;
        public bool GetModifiedType(bool modifier, bool element, bool required) => element;
        public bool GetGenericMethodParameter(object? context, int index) => false;
        public bool GetGenericTypeParameter(object? context, int index) => false;
        public bool GetFunctionPointerType(MethodSignature<bool> signature) { CheckParameters(signature); return false; }
    }
}
