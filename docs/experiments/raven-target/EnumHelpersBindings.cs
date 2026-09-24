using Mono.Cecil;
using System.Text;

// Development bridge adapters. Names and values come from the same runtime
// metadata query; generic values are copied into a typed managed snapshot.
static class EnumHelpersBindings
{
    public const string Declaration = """
        public abstract class Enum : ValueType {
            public static System.Collections.Sequence<string> GetNames(System.Introspection.TypeInfo enumType) => default;
            public static System.Collections.Sequence<object> GetValues(System.Introspection.TypeInfo enumType) => default;
            public static System.Collections.Sequence<string> GetNames<TEnum>() where TEnum : struct, System.Enum => default;
            public static System.Collections.Sequence<TEnum> GetValues<TEnum>() where TEnum : struct, System.Enum => default;
        }
        """;
    static readonly Dictionary<string, string> Helpers = new();
    public static void Reset() => Helpers.Clear();
    public static string Adapters => string.Join("\n", Helpers.Values);

    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != "System.Enum") return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope)
            || !RuntimeSignatures.IsCore(definition.DeclaringType.Scope)
            || reference.HasThis || !definition.IsStatic || definition.IsVirtual
            || reference.Name is not ("GetNames" or "GetValues"))
            throw new InvalidDataException("Unsupported Enum helper.");
        var generic = reference as GenericInstanceMethod;
        string? element = null;
        if (generic is not null)
        {
            if (generic.GenericArguments.Count != 1 || definition.GenericParameters.Count != 1)
                throw new InvalidDataException("Enum helpers require one enum type parameter.");
            var parameter = definition.GenericParameters[0];
            if (parameter.Attributes != (GenericParameterAttributes.NotNullableValueTypeConstraint | GenericParameterAttributes.DefaultConstructorConstraint)
                || parameter.Constraints.Count != 2
                || !parameter.Constraints.Select(c => c.ConstraintType.FullName).Order().SequenceEqual(new[] { "System.Enum", "System.ValueType" })
                || parameter.Constraints.Any(c => !RuntimeSignatures.IsCore(c.ConstraintType.Scope)))
                throw new InvalidDataException("Enum helper requires the struct and Enum constraints.");
            element = EnumBindings.Type(generic.GenericArguments[0])
                ?? throw new InvalidDataException("Enum helper requires an admitted enum type.");
            EnumBindings.Validate(generic.GenericArguments[0].Resolve().Module, element);
        }
        if (definition.ReturnType is not GenericInstanceType returned
            || returned.ElementType.FullName != "System.Collections.Sequence`1"
            || !RuntimeSignatures.IsCore(returned.ElementType.Scope) || returned.GenericArguments.Count != 1
            || (generic is not null && reference.Name == "GetValues"
                ? returned.GenericArguments[0] is not GenericParameter resultParameter || resultParameter.Owner != definition || resultParameter.Position != 0
                : returned.GenericArguments[0].MetadataType != (reference.Name == "GetNames" ? MetadataType.String : MetadataType.Object)))
            throw new InvalidDataException("Enum helper result must preserve its declared element type: " + definition.FullName + " / " + definition.ReturnType.Scope);
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = "System.Collections.Sequence<" + (reference.Name == "GetNames" ? "String" : element ?? "System.Object") + ">";
        if (result != expected || !args.SequenceEqual(generic is null ? new[] { "System.Introspection.TypeInfo" } : Array.Empty<string>()))
            throw new InvalidDataException("Unsupported Enum helper signature.");
        var key = reference.FullName;
        var name = "RuntimeEnum" + Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(Encoding.UTF8.GetBytes(key)))[..16];
        if (!Helpers.ContainsKey(key))
        {
            var body = new StringBuilder($".function {name}({string.Join(',', args.Select(t => t + " enumType"))}) -> {result}\n");
            if (element is not null && reference.Name == "GetValues")
                body.AppendLine($".local System.Collections.Sequence<System.Object> source\n.local arrayref<{element}> destination\n.local Int32 index");
            body.AppendLine(element is null ? "ldarg enumType" : $"call System.Runtime.RuntimeContext::get_Current()\nldtoken {element}\ncall instance System.Runtime.RuntimeContext::GetTypeInfoFromHandle(System.RuntimeTypeHandle)");
            body.AppendLine($"callvirt instance System.Introspection.TypeInfo::GetEnum{(reference.Name == "GetNames" ? "Names" : "Values")}()");
            if (element is not null && reference.Name == "GetValues")
                body.AppendLine($"stloc source\nldloc source\ncallvirt instance System.Collections.Collection<System.Object>::get_Count()\nnewarr {element}\nstloc destination\nldc.i4 0\nstloc index\nbr Test\nCopy:\nldloc destination\nldloc index\nldloc source\nldloc index\ncallvirt instance System.Collections.Sequence<System.Object>::get_Item(Int32)\nunbox.any {element}\nstelem {element}\nldloc index\nldc.i4 1\nadd\nstloc index\nTest:\nldloc index\nldloc destination\nldlen\nconv.i4\nblt Copy\nldloc destination");
            body.AppendLine("ret\n.end");
            Helpers.Add(key, body.ToString());
        }
        return new(name, args, result);
    }
}
