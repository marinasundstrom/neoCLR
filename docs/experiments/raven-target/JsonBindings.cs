using Mono.Cecil;

// Bounded JSON DOM and codec; helpers never become application capabilities.
static class JsonBindings {
    public const string Root = "System.Data.Json.JsonValue";
    public const string Prefix = "System.Data.Json.";
    public static readonly string[] Leaves = new[] { "JsonObject", "JsonArray", "JsonString", "JsonNumber", "JsonBoolean", "JsonNull" }.Select(n => Prefix + n).ToArray();
    public static readonly string[] Names = new[] { Root, Prefix + "JsonSerializer", Prefix + "ObjectMapper", Prefix + "DocumentReader", Prefix + "DocumentWriter", Prefix + "MessageReader", Prefix + "JsonSyntax" }.Concat(Leaves).ToArray();
    const string Marker = "System.Runtime.CompilerServices.ClosedHierarchyAttribute";
    public static bool Assignable(string source, string target) => target == Root && Leaves.Contains(source);
    public static bool IsName(string name) => Names.Contains(name);
    public static bool IsProvider(TypeDefinition type) => IsName(type.FullName) && type.Name is "ObjectMapper" or "DocumentReader" or "DocumentWriter" or "MessageReader" or "JsonSyntax";
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope) && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public const string Declarations = """
namespace Data.Json {
public struct JsonError { public struct Syntax { } public struct LimitExceeded { } public struct TypeMismatch { } public struct MissingField { } public struct DuplicateField { } public struct InvalidIndex { } public struct InvalidNumber { } public struct NumberOutOfRange { } public struct Read { } public struct Write { } public struct Reflection { } public struct UnsupportedMapping { } }
public abstract class JsonValue {
    protected JsonValue() { }
}
public sealed class JsonObject : JsonValue {
    public JsonObject() { }
    public int Count => default;
    public Result<JsonValue, JsonError> Field(string name) => default;
    public Result<JsonValue, JsonError> Item(int index) => default;
    public Result<string, JsonError> Name(int index) => default;
    public Result<PropagationUnit, JsonError> Add(string name, JsonValue value) => default;
}
public sealed class JsonArray : JsonValue {
    public JsonArray() { }
    public int Count => default;
    public Result<JsonValue, JsonError> Item(int index) => default;
    public Result<PropagationUnit, JsonError> Add(JsonValue value) => default;
}
public sealed class JsonString : JsonValue {
    public JsonString(string value) { }
    public string Value => default;
}
public sealed class JsonNumber : JsonValue {
    private JsonNumber(string text) { }
    public string Text => default;
    public static Result<JsonNumber, JsonError> Parse(string text) => default;
    public Result<int, JsonError> AsInt32() => default;
}
public sealed class JsonBoolean : JsonValue {
    public JsonBoolean(bool value) { }
    public bool Value => default;
}
public sealed class JsonNull : JsonValue {
    public JsonNull() { }
}
public sealed class JsonSerializer {
    private JsonSerializer() { }
    public static Result<object, JsonError> Deserialize(string text, Introspection.TypeInfo type) => default;
    public static Result<object, JsonError> Deserialize(IO.InputStream input, Introspection.TypeInfo type) => default;
    public static Result<string, JsonError> Serialize(object value) => default;
    public static Result<PropagationUnit, JsonError> Serialize(IO.OutputStream output, object value) => default;
    public static Result<JsonValue, JsonError> Deserialize(string text) => default;
    public static Result<JsonValue, JsonError> Deserialize(IO.InputStream input) => default;
    public static Result<string, JsonError> Serialize(JsonValue value) => default;
    public static Result<PropagationUnit, JsonError> Serialize(IO.OutputStream output, JsonValue value) => default;
}
internal sealed class ObjectMapper {
    public ObjectMapper() { }
    public static Result<JsonObject, JsonError> Write(object value) => default;
    public static Result<object, JsonError> Read(JsonValue value, Introspection.TypeInfo type) => default;
}
internal sealed class DocumentReader { public DocumentReader(string text) { } public Result<JsonValue, JsonError> Read() => default; }
internal sealed class DocumentWriter { public DocumentWriter() { } public Result<string, JsonError> Write(JsonValue value) => default; }
internal sealed class MessageReader { public MessageReader(string text) { } public Result<string, JsonError> Read() => default; }
internal sealed class JsonSyntax {
    public JsonSyntax() { }
    public static bool Digit(byte value) => default;
    public static bool ValidNumber(string text) => default;
    public static int HexDigit(int value) => default;
    public static bool Whitespace(byte value) => default;
    public static Result<string, JsonError> WriteMessage(string text) => default;
}
}
""";
    public static void Project(ModuleDefinition module) {
        var constructor = module.GetType(Marker).Methods.Single(m => m.IsConstructor);
        var attribute = new CustomAttribute(constructor);
        attribute.ConstructorArguments.Add(new CustomAttributeArgument(constructor.Parameters[0].ParameterType,
            Leaves.Select(name => new CustomAttributeArgument(module.GetType("System.Type"), module.GetType(name))).ToArray()));
        module.GetType(Root).CustomAttributes.Add(attribute);
    }
    public static void Validate(TypeDefinition type)
    {
        if (type.FullName != Root && !Leaves.Contains(type.FullName)) return;
        var markers = type.CustomAttributes.Where(a => a.AttributeType.FullName == Marker).ToArray();
        if (type.FullName != Root) {
            if (!type.IsSealed || type.IsAbstract || type.BaseType?.Resolve() != type.Module.GetType(Root) || markers.Length != 0)
                throw new InvalidDataException("Invalid JSON leaf contract.");
            return;
        }
        if (!type.IsAbstract || type.IsSealed || type.BaseType?.FullName != "System.Object"
            || markers.Length != 1 || markers[0].ConstructorArguments.Count != 1
            || markers[0].ConstructorArguments[0].Value is not CustomAttributeArgument[] branches
            || branches.Length != Leaves.Length
            || branches.Any(b => b.Value is not TypeReference r || type.Module.GetType(r.FullName) is null
                || (r.Scope is AssemblyNameReference assembly ? assembly.Name != type.Module.Assembly.Name.Name : r.Scope != type.Module))
            || !branches.Select(b => ((TypeReference)b.Value).FullName).Order().SequenceEqual(Leaves.Order()))
            throw new InvalidDataException("JSON must be closed to the JSON value family.");
    }
    public static void RejectExternalBranches(IEnumerable<ModuleDefinition> modules)
    {
        foreach (var type in modules.SelectMany(m => m.GetTypes()))
            if (type.BaseType is { } parent && RuntimeSignatures.IsCore(parent.Scope) && (parent.FullName == Root || Leaves.Contains(parent.FullName)))
                throw new InvalidDataException("External JSON branches are not permitted: " + type.FullName);
    }

    static readonly Dictionary<string, (string Result, bool Static, bool Construct)> Members = new() {
        ["JsonObject::.ctor()"] = ("noresult", false, true),
        ["JsonObject::get_Count()"] = ("Int32", false, false),
        ["JsonObject::Field(String)"] = ("System.Result<System.Data.Json.JsonValue,System.Data.Json.JsonError>", false, false),
        ["JsonObject::Item(Int32)"] = ("System.Result<System.Data.Json.JsonValue,System.Data.Json.JsonError>", false, false),
        ["JsonObject::Name(Int32)"] = ("System.Result<String,System.Data.Json.JsonError>", false, false),
        ["JsonObject::Add(String,System.Data.Json.JsonValue)"] = ("System.Result<Void,System.Data.Json.JsonError>", false, false),
        ["JsonArray::.ctor()"] = ("noresult", false, true),
        ["JsonArray::get_Count()"] = ("Int32", false, false),
        ["JsonArray::Item(Int32)"] = ("System.Result<System.Data.Json.JsonValue,System.Data.Json.JsonError>", false, false),
        ["JsonArray::Add(System.Data.Json.JsonValue)"] = ("System.Result<Void,System.Data.Json.JsonError>", false, false),
        ["JsonString::.ctor(String)"] = ("noresult", false, true),
        ["JsonString::get_Value()"] = ("String", false, false),
        ["JsonNumber::get_Text()"] = ("String", false, false),
        ["JsonNumber::Parse(String)"] = ("System.Result<System.Data.Json.JsonNumber,System.Data.Json.JsonError>", true, false),
        ["JsonNumber::AsInt32()"] = ("System.Result<Int32,System.Data.Json.JsonError>", false, false),
        ["JsonBoolean::.ctor(Boolean)"] = ("noresult", false, true),
        ["JsonBoolean::get_Value()"] = ("Boolean", false, false),
        ["JsonNull::.ctor()"] = ("noresult", false, true),
        ["JsonSerializer::Deserialize(String)"] = ("System.Result<System.Data.Json.JsonValue,System.Data.Json.JsonError>", true, false),
        ["JsonSerializer::Deserialize(System.IO.InputStream)"] = ("System.Result<System.Data.Json.JsonValue,System.Data.Json.JsonError>", true, false),
        ["JsonSerializer::Serialize(System.Data.Json.JsonValue)"] = ("System.Result<String,System.Data.Json.JsonError>", true, false),
        ["JsonSerializer::Serialize(System.IO.OutputStream,System.Data.Json.JsonValue)"] = ("System.Result<Void,System.Data.Json.JsonError>", true, false),
        ["JsonSerializer::Deserialize(String,System.Introspection.TypeInfo)"] = ("System.Result<System.Object,System.Data.Json.JsonError>", true, false),
        ["JsonSerializer::Deserialize(System.IO.InputStream,System.Introspection.TypeInfo)"] = ("System.Result<System.Object,System.Data.Json.JsonError>", true, false),
        ["JsonSerializer::Serialize(System.Object)"] = ("System.Result<String,System.Data.Json.JsonError>", true, false),
        ["JsonSerializer::Serialize(System.IO.OutputStream,System.Object)"] = ("System.Result<Void,System.Data.Json.JsonError>", true, false),
        ["ObjectMapper::.ctor()"] = ("noresult", false, true),
        ["ObjectMapper::Write(System.Object)"] = ("System.Result<System.Data.Json.JsonObject,System.Data.Json.JsonError>", true, false),
        ["ObjectMapper::Read(System.Data.Json.JsonValue,System.Introspection.TypeInfo)"] = ("System.Result<System.Object,System.Data.Json.JsonError>", true, false),
        ["DocumentReader::.ctor(String)"] = ("noresult", false, true),
        ["DocumentReader::Read()"] = ("System.Result<System.Data.Json.JsonValue,System.Data.Json.JsonError>", false, false),
        ["DocumentWriter::.ctor()"] = ("noresult", false, true),
        ["DocumentWriter::Write(System.Data.Json.JsonValue)"] = ("System.Result<String,System.Data.Json.JsonError>", false, false),
        ["MessageReader::.ctor(String)"] = ("noresult", false, true),
        ["MessageReader::Read()"] = ("System.Result<String,System.Data.Json.JsonError>", false, false),
        ["JsonSyntax::.ctor()"] = ("noresult", false, true),
        ["JsonSyntax::Digit(Byte)"] = ("Boolean", true, false),
        ["JsonSyntax::ValidNumber(String)"] = ("Boolean", true, false),
        ["JsonSyntax::HexDigit(Int32)"] = ("Int32", true, false),
        ["JsonSyntax::Whitespace(Byte)"] = ("Boolean", true, false),
        ["JsonSyntax::WriteMessage(String)"] = ("System.Result<String,System.Data.Json.JsonError>", true, false),
    };
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct, bool library) {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        Validate(definition.DeclaringType);
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var key = definition.DeclaringType.Name + "::" + reference.Name + "(" + string.Join(',', args) + ")";
        if (definition.DeclaringType.FullName != owner || definition.DeclaringType.HasGenericParameters
            || (owner != Root && !definition.DeclaringType.IsSealed)
            || !Members.TryGetValue(key, out var expected) || !definition.IsPublic || (!library && IsProvider(definition.DeclaringType))
            || definition.IsConstructor != construct || construct != expected.Construct || definition.IsStatic != expected.Static
            || reference.HasThis == expected.Static || definition.HasGenericParameters || definition.IsVirtual || definition.IsAbstract
            || result != expected.Result)
            throw new InvalidDataException("Unsupported JSON member: " + reference.FullName);
        return construct ? new(args, owner, $"newobj instance {owner}::.ctor({string.Join(',', args)})")
            : new(definition.IsStatic ? args : new[] { owner }.Concat(args).ToArray(), result,
                $"call {(definition.IsStatic ? "" : "instance ")}{owner}::{reference.Name}({string.Join(',', args)})");
    }
}
