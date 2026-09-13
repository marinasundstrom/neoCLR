using Mono.Cecil;
using System.Text;
using System.Globalization;
using System.Text.RegularExpressions;

// Versioned, reversible spelling for the current text importer, not a new CLI ABI.
static class MetadataIdentity
{
    static readonly UTF8Encoding Utf8 = new(false, true);
    static string Parts(params string[] parts) => string.Concat(parts.Select(p => p.Length.ToString(CultureInfo.InvariantCulture) + ":" + p));
    static string Encode(string value) => Convert.ToHexString(Utf8.GetBytes(value));
    static string Scope(TypeReference type) => type.Scope switch
    {
        AssemblyNameReference assembly => assembly.FullName,
        ModuleDefinition module when module.Assembly is not null => module.Assembly.Name.FullName,
        _ => throw new InvalidDataException("Unsupported metadata identity scope: " + type.FullName)
    };
    public static string TypeKey(TypeReference type) => type switch
    {
        ByReferenceType byref => Parts("byref", TypeKey(byref.ElementType)),
        PointerType pointer => Parts("pointer", TypeKey(pointer.ElementType)),
        ArrayType array when array.IsVector => Parts("vector", TypeKey(array.ElementType)),
        GenericInstanceType generic => Parts("generic", TypeKey(generic.ElementType), Parts(generic.GenericArguments.Select(TypeKey).ToArray())),
        TypeSpecification or GenericParameter => throw new InvalidDataException("Unsupported metadata identity signature: " + type.FullName),
        _ when type.DeclaringType is not null => Parts("nested", TypeKey(type.DeclaringType), type.Name),
        _ => Parts("type", Scope(type), type.Namespace, type.Name)
    };
    public static string TypeName(TypeReference type) => "Application.V1_T_" + Encode(TypeKey(type));
    public static string MethodKey(MethodReference method) => Parts("method", TypeKey(method.DeclaringType),
        method.Name, method.HasThis.ToString(), method.ExplicitThis.ToString(), ((int)method.CallingConvention).ToString(CultureInfo.InvariantCulture),
        method.GenericParameters.Count.ToString(CultureInfo.InvariantCulture), TypeKey(method.ReturnType),
        Parts(method.Parameters.Select(p => TypeKey(p.ParameterType)).ToArray()));
    public static string FunctionName(MethodReference method) => "Application_V1_M_" + Encode(MethodKey(method));
    // Preserve ordinary virtual slot names across declaring types. Escape the prefix
    // too, so a literal name cannot impersonate an encoded name.
    public static string MemberName(string name) => Regex.IsMatch(name, @"^[A-Za-z_][A-Za-z0-9_]*$") && !name.StartsWith("Metadata_V1_", StringComparison.Ordinal)
        ? name : "Metadata_V1_" + Encode(name);
}
