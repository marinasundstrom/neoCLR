using Mono.Cecil;
using System.Text.Json;

// Retain user-defined attributes as data. Constructors are imported but never run
// by metadata discovery. Existing framework/compiler annotations have separate
// exact bridge contracts and are not implicitly promoted to runtime attributes.
static class AttributeMetadata
{
    static IEnumerable<CustomAttribute> Retained(ICustomAttributeProvider provider) =>
        provider.CustomAttributes.Where(a => a.AttributeType.FullName is not (
            "System.Runtime.CompilerServices.NullableAttribute" or
            "System.Runtime.CompilerServices.NullableContextAttribute" or
            RavenUnionMetadata.CaseAttribute or RavenUnionMetadata.CompanionAttribute)
            && ApplicationTypes.IsModule(a.AttributeType.Resolve()?.Module));

    public static void Discover(TypeDefinition type, Queue<MethodDefinition> pending)
    {
        if (ApplicationTypes.IsLibrary(type)) return;
        var providers = new List<ICustomAttributeProvider> { type };
        providers.AddRange(type.Fields);
        providers.AddRange(type.Properties);
        providers.AddRange(type.Methods);
        providers.AddRange(type.Methods.SelectMany(m => m.Parameters));
        foreach (var attribute in providers.SelectMany(Retained))
        {
            Validate(attribute);
            _ = ApplicationTypes.Type(attribute.AttributeType);
            pending.Enqueue(attribute.Constructor.Resolve());
        }
    }

    static object Argument(CustomAttributeArgument argument) => argument.Type.MetadataType switch {
        MetadataType.String => new Dictionary<string, object?> { ["String"] = argument.Value },
        MetadataType.Int32 => new Dictionary<string, object?> { ["Int32"] = argument.Value },
        MetadataType.Boolean => new Dictionary<string, object?> { ["Boolean"] = argument.Value },
        _ => throw new InvalidDataException("Unsupported attribute argument type: " + argument.Type.FullName)
    };

    static void Validate(CustomAttribute attribute)
    {
        if (attribute.HasFields || attribute.HasProperties || attribute.AttributeType.HasGenericParameters
            || attribute.Constructor.HasGenericParameters || !attribute.Constructor.HasThis
            || attribute.ConstructorArguments.Count != attribute.Constructor.Parameters.Count)
            throw new InvalidDataException("Unsupported attribute metadata: " + attribute.AttributeType.FullName);
        foreach (var argument in attribute.ConstructorArguments) _ = Argument(argument);
    }

    static IEnumerable<string> Emit(ICustomAttributeProvider provider, Func<TypeReference, bool, string> map, uint? token = null)
    {
        foreach (var attribute in Retained(provider))
        {
            Validate(attribute);
            var prefix = token is { } value ? $"token {value} " : "";
            yield return $".custom {prefix}instance {map(attribute.AttributeType, false)}::.ctor({string.Join(',', attribute.Constructor.Parameters.Select(p => map(p.ParameterType, false)))}) = {JsonSerializer.Serialize(attribute.ConstructorArguments.Select(Argument).ToArray())}";
        }
    }
    public static IEnumerable<string> Type(TypeDefinition type, Func<TypeReference, bool, string> map) =>
        Emit(type, map).Concat(type.Fields.SelectMany(f => Emit(f, map, f.MetadataToken.ToUInt32())))
            .Concat(ApplicationTypes.RuntimeProperties(type).SelectMany(p => Emit(p, map, p.MetadataToken.ToUInt32())));
    public static IEnumerable<string> Method(MethodDefinition method, Func<TypeReference, bool, string> map) =>
        Emit(method, map).Concat(method.Parameters.SelectMany(p => Emit(p, map, p.MetadataToken.ToUInt32())));
}
