using Mono.Cecil;
using System.Text;

static class NativeArrayBindings
{
    public const string Declaration = """
        public unsafe struct Array<T> {
            public T* Data;
            public int Length;
            public static Array<T> Allocate(int length, T initialValue) => default;
            public static Array<T> View(T* data, int length) => default;
            public T* GetElementAddress(int index) => default;
            public T Get(int index) => default;
            public void Set(int index, T value) { }
            public T this[int index] { get => default; set { } }
            public void Free() { }
        }
        """;
    static readonly Dictionary<string, string> Shapes = new();
    static readonly Dictionary<string, string> Helpers = new();
    public static void Reset() { Shapes.Clear(); Helpers.Clear(); }
    public static bool IsType(string type) => Shapes.ContainsKey(type);
    public static bool IsPointer(string type) => type.EndsWith('*') && IsElement(type[..^1]);
    static bool IsElement(string type) => type is "Int32" or "Double" or "Boolean" || PrimitiveBindings.Types.Contains(type);
    public static string? Type(TypeReference type)
    {
        if (type is PointerType pointer)
        {
            var element = GenericUnionBindings.Type(pointer.ElementType);
            return element is not null && IsElement(element) ? element + "*" : null;
        }
        if (type is not GenericInstanceType g || !type.IsValueType || !RuntimeSignatures.IsCore(g.Scope)
            || g.ElementType.FullName != "System.Array`1" || g.GenericArguments.Count != 1) return null;
        var payload = GenericUnionBindings.Type(g.GenericArguments[0]);
        if (payload is null || !IsElement(payload)) return null;
        var name = $"System.Array<{payload}>";
        Shapes[name] = payload;
        return name;
    }
    public static string Adapters => string.Join("\n", Helpers.Values);
    public static void Validate(ModuleDefinition module)
    {
        var type = module.GetType("System.Array`1");
        if (type is null || !type.IsValueType || type.IsEnum || type.GenericParameters.Count != 1
            || type.GenericParameters[0].HasConstraints || type.GenericParameters[0].Attributes != GenericParameterAttributes.NonVariant
            || type.Fields.Count != 2 || type.Interfaces.Count != 0)
            throw new InvalidDataException("Unsupported native buffer descriptor metadata.");
        var owner = new GenericInstanceType(type); owner.GenericArguments.Add(module.TypeSystem.Int32);
        foreach (var field in type.Fields)
            Field(new FieldReference(field.Name, field.FieldType, owner));
    }
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null || !Shapes.TryGetValue(owner, out var element)) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, t => Type(t) ?? GenericUnionBindings.Type(t), pointers: true);
        var expected = reference.Name switch {
            "Allocate" => ("Int32," + element, owner, false),
            "View" => (element + "*,Int32", owner, false),
            "GetElementAddress" => ("Int32", element + "*", true),
            "Get" or "get_Item" => ("Int32", element, true),
            "Set" or "set_Item" => ("Int32," + element, "noresult", true),
            "Free" => ("", "noresult", true),
            _ => throw new InvalidDataException("Unsupported native buffer member.")
        };
        if (string.Join(',', args) != expected.Item1 || result != expected.Item2 || reference.HasThis != expected.Item3)
            throw new InvalidDataException("Unsupported native buffer signature.");
        var inputs = reference.HasThis ? new[] { owner + "&" }.Concat(args).ToArray() : args;
        var key = reference.FullName;
        var name = "RuntimeNativeArray" + Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(Encoding.UTF8.GetBytes(key)))[..16];
        if (!Helpers.ContainsKey(key))
        {
            var body = new StringBuilder($".function {name}({string.Join(',', inputs.Select((t,i) => t + " arg" + i))}) -> {result}\n");
            for (var i=0; i<inputs.Length; i++) {
                body.AppendLine("ldarg arg" + i);
                if (i == 0 && reference.HasThis) body.AppendLine("ldobj " + owner);
            }
            body.AppendLine($"call {(reference.HasThis ? "instance " : "")}{owner}::{reference.Name}({string.Join(',',args)})");
            if (result == "noresult") body.AppendLine("pop");
            body.AppendLine("ret\n.end"); Helpers.Add(key,body.ToString());
        }
        return new(name,inputs,result);
    }
    public static (string Owner, string Type) Field(FieldReference reference)
    {
        var owner = Type(reference.DeclaringType) ?? throw new InvalidDataException("Unsupported native buffer field owner.");
        if (!Shapes.TryGetValue(owner, out var element)) throw new InvalidDataException("Unsupported native buffer field owner.");
        var field = reference.Resolve();
        var target = reference.Name switch { "Data" => element + "*", "Length" => "Int32", _ => throw new InvalidDataException("Unsupported native buffer field.") };
        var mapped = RuntimeSignatures.Map(RuntimeSignatures.Close(reference.FieldType, reference.DeclaringType, pointers: true), t => Type(t) ?? GenericUnionBindings.Type(t));
        if (field is null || !field.IsPublic || field.IsStatic || field.IsInitOnly || mapped != target
            || RuntimeSignatures.Map(RuntimeSignatures.Close(field.FieldType, reference.DeclaringType, pointers: true), t => Type(t) ?? GenericUnionBindings.Type(t)) != target)
            throw new InvalidDataException("Unsupported native buffer field signature.");
        return (owner, target);
    }
}
