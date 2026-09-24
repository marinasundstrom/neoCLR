using Mono.Cecil;
using Mono.Cecil.Cil;

static class ManagedArrayBindings
{
    public static bool IsType(string type) => type.StartsWith("arrayref<", StringComparison.Ordinal) && type.EndsWith('>');
    public static bool IsReference(string type) => AsyncBindings.IsType(type) || ReaderBindings.IsName(type) || type == FileSystemBindings.Name || StorageItemBindings.IsName(type) || ApplicationTypes.IsReference(type) || type == "String" || type == "System.Object" || IsType(type)
        || ReflectionBindings.IsReference(type) || CollectionBindings.IsReference(type) || InterfaceBindings.IsInterface(type);
    public static bool Defaultable(string type) => IsReference(type) || type is "Int32" or "Double" or "Boolean" or "Void"
        || PrimitiveBindings.Types.Contains(type) || CalendarBindings.Types.Contains(type) || ErrorBindings.IsEmpty(type);
    public static void CheckElement(Code code, string element, string? token)
    {
        var valid = code switch {
            Code.Ldelem_Any or Code.Stelem_Any => token == element,
            Code.Ldelem_I1 => element is "SByte" or "Boolean",
            Code.Ldelem_U1 => element is "Byte" or "Boolean",
            Code.Stelem_I1 => element is "SByte" or "Byte" or "Boolean",
            Code.Ldelem_I2 => element == "Int16",
            Code.Ldelem_U2 => element == "UInt16",
            Code.Stelem_I2 => element is "Int16" or "UInt16",
            Code.Ldelem_I4 => element == "Int32",
            Code.Ldelem_U4 => element is "UInt32",
            Code.Stelem_I4 => element is "Int32" or "UInt32",
            Code.Ldelem_I8 or Code.Stelem_I8 => element is "Int64" or "UInt64",
            Code.Ldelem_I or Code.Stelem_I => element is "IntPtr" or "UIntPtr",
            Code.Ldelem_R4 or Code.Stelem_R4 => element == "Single",
            Code.Ldelem_R8 or Code.Stelem_R8 => element == "Double",
            _ => false
        };
        if (!valid) throw new InvalidDataException($"Array opcode {code} and element {element} (token {token}) mismatch.");
    }
    public static string? Type(TypeReference type)
    {
        if (type is not ArrayType { IsVector: true } array) return null;
        var element = GenericUnionBindings.Type(array.ElementType);
        return element is not null && Defaultable(element) ? "arrayref<" + element + ">" : null;
    }
}
