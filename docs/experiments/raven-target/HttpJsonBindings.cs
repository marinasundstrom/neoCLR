using Mono.Cecil;

// Bounded generic HTTP JSON conveniences; no general generic class-method admission.
static class HttpJsonBindings
{
    public const string Extensions = "System.Web.Http.Json.HttpClientJsonExtensions";
    public const string Operations = "System.Web.Http.Json.JsonClientOperations";
    public const string Reader = "System.Web.Http.Json.JsonClientRead";
    public const string Error = "System.Web.Http.Json.HttpJsonError";
    public static readonly string[] Names = [Extensions, Operations, Reader];
    public static bool IsProvider(string name) => name is Operations or Reader;
    const string Client = "System.Web.Http.HttpClient";
    const string Response = "System.Web.Http.HttpResponse";
    const string Token = "System.Concurrency.CancellationToken";
    static string Result(string value) => $"System.Result<{value},{Error}>";
    static string Task(string value) => $"System.Tasks.Task<{Result(value)}>";
    public static bool IsGeneric(MethodDefinition method)
    {
        if (method.DeclaringType.FullName is not (Extensions or Operations)
            || !method.IsPublic || !method.IsStatic || method.IsVirtual || method.IsAbstract || method.ExplicitThis
            || method.CallingConvention != MethodCallingConvention.Generic || method.GenericParameters.Count != 1
            || method.GenericParameters[0].Attributes != GenericParameterAttributes.NonVariant || method.GenericParameters[0].HasConstraints)
            return false;
        try {
            string Shape(TypeReference type) {
                if (type is GenericParameter parameter && parameter.Owner == method && parameter.Position == 0) return "T0";
                if (type is GenericInstanceType generic && RuntimeSignatures.IsCore(generic.ElementType.Scope))
                    return generic.ElementType.FullName.Split('`')[0] + "<" + string.Join(',', generic.GenericArguments.Select(Shape)) + ">";
                return RuntimeSignatures.Map(type, GenericUnionBindings.Type);
            }
            var args = method.Parameters.Select(p => Shape(p.ParameterType)).ToArray();
            var returned = Shape(method.ReturnType);
            if (method.DeclaringType.FullName == Operations)
                return method.Name == "Convert" && args.SequenceEqual([Result("System.Object")]) && returned == Result("T0");
            var post = method.Name == "PostAsJson";
            if ((!post && method.Name != "GetFromJson") || args.Length < 2 || args[0] != Client || args[1] is not ("String" or "System.Uri")) return false;
            var expected = post ? new[] { Client, args[1], "T0" } : new[] { Client, args[1] };
            return (args.SequenceEqual(expected) || args.SequenceEqual(expected.Append(Token))) && returned == Task(post ? Response : "T0");
        } catch (InvalidDataException) { return false; }
    }
    public static string ImplementationName(MethodDefinition method)
    {
        if (method.DeclaringType.FullName == Operations) return Operations + ".Convert";
        var address = method.Parameters[1].ParameterType.MetadataType == MetadataType.String ? "Text" : "Uri";
        var token = method.Parameters.Last().ParameterType.FullName == Token ? "WithToken" : "";
        return Extensions + "." + method.Name + address + token;
    }
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct, bool library)
    {
        if (!Names.Contains(definition.DeclaringType.FullName)) return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || construct || !IsGeneric(definition)
            || reference is not GenericInstanceMethod generic || generic.GenericArguments.Count != 1
            || !library && IsProvider(definition.DeclaringType.FullName))
            throw new InvalidDataException("Unsupported HTTP JSON generic member: " + reference.FullName);
        if (definition.DeclaringType.FullName == Extensions) LibraryImplementation.CheckExtensionContract(definition, definition);
        var signature = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type, allowOpenMethodParameters: library);
        var argument = RuntimeSignatures.Map(generic.GenericArguments[0], GenericUnionBindings.Type);
        return new(signature.Args, signature.Result,
            $"call {ImplementationName(definition)}<{argument}>({string.Join(',', signature.Args)})");
    }
}
