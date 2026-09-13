using Mono.Cecil;
using System.Text;

// Per-import closed shapes. Generic metadata is shared; only used adapters are emitted.
static class GenericUnionBindings
{
    sealed record Shape(string Kind, string[] Args, string Name);
    static readonly Dictionary<string, Shape> Shapes = new();
    static readonly Dictionary<string, (string Name, string Body)> Helpers = new();
    public static void Reset() { Shapes.Clear(); Helpers.Clear(); }
    static string Register(string kind, params string[] args)
    {
        var name = "System." + kind + (args.Length == 0 ? "" : "<" + string.Join(',', args) + ">");
        Shapes.TryAdd(name, new(kind, args, name));
        if (kind == "Result") { Register("Result.Ok", args[0]); Register("Result.Error", args[1]); }
        if (kind == "Option") { Register("Option.Some", args[0]); Register("Option.None"); }
        return name;
    }
    static int MappingDepth;
    public static string? Type(TypeReference type)
    {
        if (++MappingDepth > 32) { MappingDepth--; throw new InvalidDataException("API type nesting limit exceeded."); }
        try { return Map(type, 0); } finally { MappingDepth--; }
    }
    static string? Map(TypeReference type, int depth)
    {
        if (depth > 24) throw new InvalidDataException("Union payload nesting limit exceeded.");
        if (type is GenericInstanceType g && RuntimeSignatures.IsCore(type.Scope) && type.IsValueType)
        {
            var kind = g.ElementType.FullName switch {
                "System.Result`2" => "Result", "System.Option`1" => "Option",
                "System.Result/Ok`1" => "Result.Ok", "System.Result/Error`1" => "Result.Error",
                "System.Option/Some`1" => "Option.Some", _ => null
            };
            if (kind is null || g.GenericArguments.Count != (kind == "Result" ? 2 : 1)) return null;
            var args = g.GenericArguments.Select(t => Map(t, depth + 1)).ToArray();
            return args.All(t => t is not null) ? Register(kind, args.Select(t => t!).ToArray()) : null;
        }
        if (type.FullName == "System.Option/None" && type.IsValueType && RuntimeSignatures.IsCore(type.Scope)) return Register("Option.None");
        if (type.FullName == "System.Value" && type.IsValueType && RuntimeSignatures.IsCore(type.Scope)) return "Value";
        if (type.FullName == "System.Void" && type.IsValueType && RuntimeSignatures.IsCore(type.Scope)) return "Void";
        return ReflectionBindings.Type(type) ?? CollectionBindings.Type(type) ?? InterfaceBindings.Type(type) ?? DelegateBindings.Type(type) ?? NativeArrayBindings.Type(type) ?? ManagedArrayBindings.Type(type) ?? CalendarBindings.Type(type) ?? ErrorBindings.Type(type) ?? PrimitiveBindings.Type(type) ?? type.MetadataType switch {
            MetadataType.Int32 => "Int32", MetadataType.Double => "Double", MetadataType.Boolean => "Boolean", MetadataType.String => "String", _ => null
        };
    }
    public static bool IsType(string type) => Shapes.ContainsKey(type);
    static bool DefaultPayload(string type) => type is "Void" or "Int32" or "Double" or "Boolean"
        || PrimitiveBindings.Types.Contains(type) || CalendarBindings.Types.Contains(type) || ErrorBindings.IsEmpty(type)
        || Shapes.TryGetValue(type, out var nested) && CanDefault(nested);
    static bool CanDefault(Shape shape) => shape.Kind == "Option.None"
        || shape.Kind is "Result.Ok" or "Result.Error" or "Option.Some" && DefaultPayload(shape.Args[0]);
    public static bool RequiresInitialization(string type) => Shapes.TryGetValue(type, out var shape) && !CanDefault(shape);
    static ResultBindings.Binding Helper(Shape shape, string method, string[] args, string result, bool instance, bool byref, bool construct = false, bool output = false)
    {
        var inputs = instance ? new[] { shape.Name + "&" }.Concat(args).ToArray() : args;
        var key = shape.Name + ":" + method + "(" + string.Join(',', args) + ")" + (construct ? "new" : "call");
        if (!Helpers.TryGetValue(key, out var helper))
        {
            var name = "RuntimeUnion" + Helpers.Count;
            var declaration = inputs.Select((t, i) => (output && i == 1 ? "out(true) " : "") + t + " arg" + i);
            var body = new StringBuilder($".function {name}({string.Join(',', declaration)}) -> {result}\n");
            if (instance) { body.AppendLine("ldarg arg0"); if (!byref) body.AppendLine("ldobj " + shape.Name); }
            for (var i = instance ? 1 : 0; i < inputs.Length; i++) body.AppendLine("ldarg arg" + i);
            body.AppendLine((construct ? "newobj instance " : instance ? "call instance " : "call ") + shape.Name + "::" + method + "(" + string.Join(',', args) + ")");
            body.AppendLine("ret\n.end");
            helper = (name, body.ToString()); Helpers.Add(key, helper);
        }
        return new(helper.Name, inputs, result, output ? 1 : -1);
    }
    public static ResultBindings.Binding? Construct(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null || !Shapes.TryGetValue(owner, out var shape)) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, Type);
        var allowed = shape.Kind switch {
            "Result" => new[] { Register("Result.Ok", shape.Args[0]), Register("Result.Error", shape.Args[1]) },
            "Option" => new[] { Register("Option.Some", shape.Args[0]), Register("Option.None") },
            "Option.None" => Array.Empty<string>(), _ => new[] { shape.Args[0] }
        };
        if (result != "noresult" || (shape.Kind == "Option.None" ? args.Length != 0 : args.Length != 1 || !allowed.Contains(args[0])))
            throw new InvalidDataException("Unsupported union constructor.");
        return Helper(shape, ".ctor", args, owner, false, false, construct: true);
    }
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null || !Shapes.TryGetValue(owner, out var shape)) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, Type);
        var name = reference.Name;
        if (definition.IsVirtual && !definition.IsFinal) throw new InvalidDataException("Unexpected union virtual method.");
        if (shape.Kind is "Result.Ok" or "Result.Error" or "Option.Some")
        {
            if (reference.HasThis && name == "get_Value" && args.Length == 0 && result == shape.Args[0])
                return Helper(shape, name, args, result, true, false);
        }
        if (shape.Kind is "Result" or "Option")
        {
            var payload = shape.Args[0]; var residual = shape.Kind == "Result" ? shape.Args[1] : "Void";
            var first = Register(shape.Kind == "Result" ? "Result.Ok" : "Option.Some", payload);
            var second = shape.Kind == "Result" ? Register("Result.Error", residual) : Register("Option.None");
            if (!reference.HasThis && result == owner && args.Length == 1
                && (name == "FromResidual" && args[0] == residual
                    || shape.Kind == "Result" && (name == "Ok" && args[0] == payload || name == "Error" && args[0] == residual)))
                return Helper(shape, name, args, result, false, false);
            if (reference.HasThis && args.Length == 0)
            {
                var flags = shape.Kind == "Result" ? new[] { "get_IsOk", "get_IsOkCase", "get_IsErr", "get_IsErrorCase" } : new[] { "get_IsSome", "get_IsNone" };
                if (result == "Boolean" && flags.Contains(name)
                    || name == (shape.Kind == "Result" ? "GetOkCase" : "GetSomeCase") && result == first
                    || name == (shape.Kind == "Result" ? "GetErrorCase" : "GetNoneCase") && result == second)
                    return Helper(shape, name, args, result, true, false);
            }
            if (reference.HasThis && result == "Boolean" && args.Length == 1 && definition.Parameters[0].IsOut)
            {
                if (name is "TryGet" or "TryGetValue" && (args[0] == first + "&" || args[0] == second + "&"))
                    return Helper(shape, "TryGet", args, result, true, false, output: true);
                if (name == "TryGetOutput" && args[0] == payload + "&" || name == "TryGetResidual" && args[0] == residual + "&")
                    return Helper(shape, name, args, result, true, true, output: true);
            }
        }
        throw new InvalidDataException("Unsupported union member: " + reference.FullName);
    }
    public static string Adapters => string.Concat(Helpers.Values.Select(h => h.Body));
}
