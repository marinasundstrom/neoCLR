using System.Text.Json;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

// Owned by this framework. No dependency on the other framework or its contracts.
internal static class DataAdapters
{
    internal record Registration(string Suffix, string Callback, string? Provider = null, string? ProviderReference = null);
    internal static List<Registration> Read(MethodDefinition definition, MethodInfo method,
        AssemblyDefinition snapshot, MetadataLoadContext context, string scope, string sourceAttribute)
    {
        var rows = new List<Registration>();
        var parameters = method.GetParameters();
        foreach (var data in definition.CustomAttributes.Where(a => a.AttributeType.Namespace == scope &&
                     a.AttributeType.Name is "ArgumentsAttribute" || a.AttributeType.Namespace == scope && a.AttributeType.Name == sourceAttribute))
        {
            var type = context.Resolve(data.AttributeType);
            var own = context.Resolve(snapshot.Identity).GetTypes().Single(t => t.FullName == scope + "." + data.AttributeType.Name);
            if (!ReferenceEquals(type, own)) continue;
            if (data.GetNamedArguments().Count != 0) throw new InvalidDataException("Data attributes do not support named arguments yet");
            var values = data.GetArguments();
            if (data.AttributeType.Name == "ArgumentsAttribute")
            {
                if (values.Count != parameters.Count) throw new InvalidDataException(method.Name + ": argument row has wrong arity");
                for (var i = 0; i < values.Count; i++)
                {
                    var kind = values[i].Value switch { int => PrimitiveType.Int32, string => PrimitiveType.String, bool => PrimitiveType.Boolean, _ => throw new InvalidDataException("Unsupported inline value") };
                    if (parameters[i].ParameterType is not PrimitiveTypeInfo primitive || primitive.Kind != kind)
                        throw new InvalidDataException(method.Name + ": argument row has wrong parameter type");
                }
                var literals = string.Join(", ", values.Select(v => v.Value is string text ? RavenNames.StringLiteral(text) : JsonSerializer.Serialize(v.Value)));
                var identity = string.Join(", ", values.Select(v => JsonSerializer.Serialize(v.Value)));
                rows.Add(new("(" + identity + ")", literals));
            }
            else
            {
                if (values.Count != 1 || values[0].Value is not string provider || parameters.Count != 1)
                    throw new InvalidDataException(method.Name + ": a method source requires a name and one input parameter");
                // Resolve metadata names before generating source. A dot inside a method
                // name belongs to that name, not necessarily to a namespace separator.
                var candidates = snapshot.MainModule.Methods.Where(d => d.Name == provider || provider.EndsWith("." + d.Name, StringComparison.Ordinal)).Select(context.Resolve).Where(d =>
                    (d.Namespace == method.Namespace && d.Name == provider) ||
                    (d.Namespace.Length == 0 ? d.Name : d.Namespace + "." + d.Name) == provider).ToArray();
                if (candidates.Length != 1) throw new InvalidDataException("Missing or ambiguous data source: " + provider);
                var source = candidates[0];
                if (source.DeclaringType is not null || !source.IsStatic || source.IsAbstract || source.IsGenericMethodDefinition ||
                    source.Accessibility is not (MetadataAccessibility.Public or MetadataAccessibility.Assembly) || source.GetParameters().Count != 0 ||
                    source.ReturnType is not ArrayTypeInfo array || !ReferenceEquals(array.ElementType, parameters[0].ParameterType))
                    throw new InvalidDataException(provider + ": source must be an accessible parameterless module function returning an array of the input type");
                provider = source.Namespace.Length == 0 ? source.Name : source.Namespace + "." + source.Name;
                rows.Add(new("/" + provider, "value", provider, RavenNames.Reference(source.Namespace, source.Name)));
            }
        }
        if (rows.Count == 0)
        {
            if (parameters.Count != 0) throw new InvalidDataException(method.Name + ": parameterized function needs a data attribute");
            rows.Add(new("", ""));
        }
        if (rows.Select(r => r.Suffix).Distinct(StringComparer.Ordinal).Count() != rows.Count)
            throw new InvalidDataException(method.Name + ": duplicate data row or source");
        return rows;
    }
}
