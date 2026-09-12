using Mono.Cecil;
using System.Text;

// A closed projection of the existing bounded UTF-8 APIs, not host System.IO.
static class FileBindings
{
    public const string ReadError = "System.IO.FileReadError", WriteError = "System.IO.FileWriteError";
    const string Read = "System.Result<String,System.IO.FileReadError>", Write = "System.Result<Void,System.IO.FileWriteError>";
    const string StringOk = "System.Result.Ok<String>", VoidOk = "System.Result.Ok<Void>";
    static readonly string[] Errors = [ReadError, WriteError];
    static readonly string[] CommonCases = ["InvalidLimit", "InvalidPath", "NotFound", "AccessDenied", "NotRegularFile", "TooLarge"];
    public static string Declarations => "namespace IO { public static class File { "
        + "public static Result<string, FileReadError> ReadAllText(string path, int maxBytes) => default; "
        + "public static Result<PropagationUnit, FileWriteError> WriteAllText(string path, string text, int maxBytes) => default; } "
        + string.Join(" ", Errors.Select(e => "public struct " + e.Split('.').Last() + " { "
            + string.Join(" ", Cases(e).Select(c => "public bool Is" + c + " => false;")) + " }")) + " }";
    static IEnumerable<string> Cases(string error) => CommonCases.Concat(error == ReadError ? ["ReadFailed", "InvalidUtf8"] : new[] { "WriteFailed" });
    public static bool IsType(string type) => type is Read or Write or StringOk or ReadError or WriteError
        || Errors.Any(e => type == $"System.Result.Error<{e}>");
    static bool IsCore(IMetadataScope scope) => scope.Name == CoreDeclarations.Identity || scope is ModuleDefinition module && module.Assembly.Name.Name == CoreDeclarations.Identity;
    public static string? Type(TypeReference type)
    {
        if (!type.IsValueType || !IsCore(type.Scope)) return null;
        if (type.FullName is ReadError or WriteError) return type.FullName;
        if (type is not GenericInstanceType g) return null;
        string? Argument(TypeReference t) => t.MetadataType == MetadataType.String ? "String"
            : t.FullName == "System.Void" && t.IsValueType && IsCore(t.Scope) ? "Void"
            : t.FullName is ReadError or WriteError && t.IsValueType && IsCore(t.Scope) ? t.FullName : null;
        var args = g.GenericArguments.Select(Argument).ToArray();
        var name = g.ElementType.FullName.Split('`')[0].Replace('/', '.') + "<" + string.Join(',', args) + ">";
        return args.All(a => a is not null) && IsType(name) ? name : null;
    }
    public sealed record Binding(string Name, string[] Arguments, string Result, int OutArgument = -1, string? Instruction = null);
    static string Closed(TypeReference t, TypeReference owner, bool returns = false)
    {
        if (t is ByReferenceType b) return Closed(b.ElementType, owner) + "&";
        if (t is GenericParameter p && p.Type == GenericParameterType.Type && owner is GenericInstanceType g)
            return Closed(g.GenericArguments[p.Position], owner);
        if (t is GenericInstanceType nested)
        {
            var copy = new GenericInstanceType(nested.ElementType);
            foreach (var arg in nested.GenericArguments)
                copy.GenericArguments.Add(arg is GenericParameter p2 && owner is GenericInstanceType g2 ? g2.GenericArguments[p2.Position] : arg);
            return Type(copy) ?? (copy.FullName == "System.Result/Ok`1<System.Void>" ? VoidOk : throw new InvalidDataException("Unsupported file generic signature: " + copy.FullName));
        }
        return t.MetadataType switch { MetadataType.Boolean => "Boolean", MetadataType.Int32 => "Int32", MetadataType.String => "String",
            MetadataType.Void when returns => "noresult", _ => t.FullName == "System.Void" && t.IsValueType ? "Void"
            : Type(t) ?? throw new InvalidDataException("Unsupported file signature: " + t.FullName) };
    }
    static (string[] Args, string Result) Signature(MethodReference reference, MethodDefinition definition)
    {
        var args = reference.Parameters.Select(p => Closed(p.ParameterType, reference.DeclaringType)).ToArray();
        var result = Closed(reference.ReturnType, reference.DeclaringType, true);
        if (definition.HasGenericParameters || !args.SequenceEqual(definition.Parameters.Select(p => Closed(p.ParameterType, reference.DeclaringType)))
            || result != Closed(definition.ReturnType, reference.DeclaringType, true))
            throw new InvalidDataException("File reference/definition signature mismatch.");
        return (args, result);
    }
    public static Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null && reference.DeclaringType.FullName != "System.IO.File") return null;
        var (args, result) = Signature(reference, definition);
        var name = reference.Name;
        if (owner is null && !reference.HasThis && ((name == "ReadAllText" && args.SequenceEqual(new[] { "String", "Int32" }) && result == Read)
            || (name == "WriteAllText" && args.SequenceEqual(new[] { "String", "String", "Int32" }) && result == Write)))
            return new("System.IO.File::" + name, args, result);
        if (owner is Read or Write)
        {
            var output = owner == Read ? "String" : "Void";
            var error = owner == Read ? ReadError : WriteError;
            if (name == "FromResidual" && !reference.HasThis && args.SequenceEqual(new[] { error }) && result == owner)
                return new(owner + "::FromResidual", args, result);
            var expected = name switch { "TryGetOutput" => output, "TryGetResidual" => error,
                "TryGetValue" when args.Length == 1 && args[0] == $"System.Result.Ok<{output}>&" => $"System.Result.Ok<{output}>",
                "TryGetValue" => $"System.Result.Error<{error}>", _ => "" };
            if (reference.HasThis && result == "Boolean" && args.SequenceEqual(new[] { expected + "&" }) && definition.Parameters[0].IsOut)
                return new("", [owner + "&", expected + "&"], "Boolean", 1,
                    name == "TryGetValue" ? $"call {Helper(owner, expected)}({owner}&,{expected}&)" : $"call instance {owner}::{name}({expected}&)");
        }
        var value = owner == StringOk ? "String" : Errors.FirstOrDefault(e => owner == $"System.Result.Error<{e}>");
        if (reference.HasThis && args.Length == 0 && name == "get_Value" && result == value)
            return new(Helper(owner!, name), [owner + "&"], result);
        if (owner is ReadError or WriteError && reference.HasThis && args.Length == 0 && result == "Boolean"
            && Cases(owner).Any(c => name == "get_Is" + c))
            return new(Helper(owner, name), [owner + "&"], result);
        throw new InvalidDataException("Unsupported file member: " + reference.FullName);
    }
    public static Binding? Construct(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = Signature(reference, definition);
        var expected = owner switch { StringOk => "String", Read => StringOk, Write => VoidOk, _ => "" };
        if (result == "noresult" && args.SequenceEqual(new[] { expected }))
            return new(Helper(owner, "New"), args, owner);
        throw new InvalidDataException("Unsupported file constructor: " + reference.FullName);
    }
    static string Helper(string owner, string name) => "File_" + new string((owner + "_" + name).Select(c => char.IsLetterOrDigit(c) ? c : '_').ToArray());
    public static string Adapters()
    {
        var text = new StringBuilder();
        foreach (var (owner, arg) in new[] { (StringOk, "String"), (Read, StringOk), (Write, VoidOk) })
            text.AppendLine($".function {Helper(owner, "New")}({arg} value) -> {owner}\nldarg value\nnewobj instance {owner}::.ctor({arg})\nret\n.end");
        foreach (var (owner, result, name) in Errors.SelectMany(e => Cases(e).Select(c => (e, "Boolean", "get_Is" + c)))
            .Concat(Errors.Select(e => ($"System.Result.Error<{e}>", e, "get_Value")))
            .Append((StringOk, "String", "get_Value")))
            text.AppendLine($".function {Helper(owner, name)}({owner}& value) -> {result}\nldarg value\nldobj {owner}\ncall instance {owner}::{name}()\nret\n.end");
        foreach (var (owner, output, error) in new[] { (Read, "String", ReadError), (Write, "Void", WriteError) })
            foreach (var variant in new[] { $"System.Result.Ok<{output}>", $"System.Result.Error<{error}>" })
                text.AppendLine($".function {Helper(owner, variant)}({owner}& source,out(true) {variant}& destination) -> Boolean\nldarg source\nldobj {owner}\nldarg destination\ncall instance {owner}::TryGet({variant}&)\nret\n.end");
        return text.ToString();
    }
}
