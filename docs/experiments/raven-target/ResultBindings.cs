using Mono.Cecil;
using System.Text;

// Shared bounded Result/error catalog for existing runtime-library operations.
static class ResultBindings
{
    public const string ReadError = "System.IO.FileReadError", WriteError = "System.IO.FileWriteError";
    const string Int32Ok = "System.Result.Ok<Int32>";
    public const string DivisionError = "System.IntegerDivisionError";
    public const string ParseError = "System.Int32ParseError";
    const string StringOk = "System.Result.Ok<String>", VoidOk = "System.Result.Ok<Void>";
    public const string SliceError = "System.Text.Utf8SliceError";
    public const string Slice = "System.Result<String,System.Text.Utf8SliceError>";
    sealed record Carrier(string Output, string Error)
    {
        public string Type => $"System.Result<{Output},{Error}>";
    }
    static readonly Dictionary<string, string[]> ErrorCases = new() {
        [ReadError] = ["InvalidLimit", "InvalidPath", "NotFound", "AccessDenied", "NotRegularFile", "TooLarge", "ReadFailed", "InvalidUtf8"],
        [WriteError] = ["InvalidLimit", "InvalidPath", "NotFound", "AccessDenied", "NotRegularFile", "TooLarge", "WriteFailed"],
        [SliceError] = ["OutOfRange", "InvalidBoundary"],
        [ParseError] = ["InvalidFormat", "Overflow"],
        [DivisionError] = ["DivisionByZero", "Overflow"]
    };
    static readonly Carrier[] Carriers = [new("String", ReadError), new("Void", WriteError), new("String", SliceError), new("Int32", ParseError), new("Int32", DivisionError)];
    static IEnumerable<string> Errors => ErrorCases.Keys;
    static IEnumerable<string> Cases(string error) => ErrorCases[error];
    public static string Declarations => string.Join(" ", Errors.Select(error => {
        var declaration = "public struct " + error.Split('.').Last() + " { "
            + string.Join(" ", Cases(error).Select(name => "public bool Is" + name + " => false;")) + " }";
        var separator = error.LastIndexOf('.');
        return separator == 6 ? declaration : "namespace " + error[7..separator] + " { " + declaration + " }";
    }));
    public static bool IsType(string type) => type is StringOk || Carriers.Any(c => type == c.Type || type == c.Error)
        || Errors.Any(e => type == $"System.Result.Error<{e}>");
    public static string? Type(TypeReference type)
    {
        if (!type.IsValueType || !RuntimeSignatures.IsCore(type.Scope)) return null;
        if (Errors.Contains(type.FullName)) return type.FullName;
        if (type is not GenericInstanceType g) return null;
        string? Argument(TypeReference t) => t.MetadataType == MetadataType.Int32 ? "Int32" : t.MetadataType == MetadataType.String ? "String"
            : t.FullName == "System.Void" && t.IsValueType && RuntimeSignatures.IsCore(t.Scope) ? "Void"
            : Errors.Contains(t.FullName) && t.IsValueType && RuntimeSignatures.IsCore(t.Scope) ? t.FullName : null;
        var args = g.GenericArguments.Select(Argument).ToArray();
        var name = g.ElementType.FullName.Split('`')[0].Replace('/', '.') + "<" + string.Join(',', args) + ">";
        return args.All(a => a is not null) && IsType(name) ? name : null;
    }
    public sealed record Binding(string Name, string[] Arguments, string Result, int OutArgument = -1, string? Instruction = null);
    static (string[] Args, string Result) Signature(MethodReference reference, MethodDefinition definition)
        => RuntimeSignatures.Match(reference, definition, type => Type(type)
            ?? (type is GenericInstanceType integerOk && integerOk.ElementType.FullName == "System.Result/Ok`1"
                && integerOk.GenericArguments.Count == 1 && integerOk.GenericArguments[0].MetadataType == MetadataType.Int32
                && type.IsValueType && RuntimeSignatures.IsCore(type.Scope) ? Int32Ok : null)
            ?? (type.FullName == "System.Result/Ok`1<System.Void>" && type.IsValueType && RuntimeSignatures.IsCore(type.Scope)
                && type is GenericInstanceType g && g.GenericArguments[0].IsValueType
                && RuntimeSignatures.IsCore(g.GenericArguments[0].Scope) ? VoidOk : null));
    public static Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = Signature(reference, definition);
        var name = reference.Name;
        var carrier = Carriers.SingleOrDefault(c => c.Type == owner);
        if (carrier is not null)
        {
            var output = carrier.Output;
            var error = carrier.Error;
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
        if (Errors.Contains(owner) && reference.HasThis && args.Length == 0 && result == "Boolean"
            && Cases(owner).Any(c => name == "get_Is" + c))
            return new(Helper(owner, name), [owner + "&"], result);
        throw new InvalidDataException("Unsupported Result member: " + reference.FullName);
    }
    public static Binding? Construct(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = Signature(reference, definition);
        var expected = owner == StringOk ? "String" : Carriers.Where(c => c.Type == owner).Select(c => $"System.Result.Ok<{c.Output}>").SingleOrDefault() ?? "";
        if (result == "noresult" && args.SequenceEqual(new[] { expected }))
            return new(Helper(owner, "New"), args, owner);
        throw new InvalidDataException("Unsupported Result constructor: " + reference.FullName);
    }
    static string Helper(string owner, string name) => "Result_" + new string((owner + "_" + name).Select(c => char.IsLetterOrDigit(c) ? c : '_').ToArray());
    public static string Adapters()
    {
        var text = new StringBuilder();
        foreach (var (owner, arg) in Carriers.Select(c => (c.Type, $"System.Result.Ok<{c.Output}>" )).Prepend((StringOk, "String")))
            text.AppendLine($".function {Helper(owner, "New")}({arg} value) -> {owner}\nldarg value\nnewobj instance {owner}::.ctor({arg})\nret\n.end");
        foreach (var (owner, result, name) in Errors.SelectMany(e => Cases(e).Select(c => (e, "Boolean", "get_Is" + c)))
            .Concat(Errors.Select(e => ($"System.Result.Error<{e}>", e, "get_Value")))
            .Append((StringOk, "String", "get_Value")))
            text.AppendLine($".function {Helper(owner, name)}({owner}& value) -> {result}\nldarg value\nldobj {owner}\ncall instance {owner}::{name}()\nret\n.end");
        foreach (var (owner, output, error) in Carriers.Select(c => (c.Type, c.Output, c.Error)))
            foreach (var variant in new[] { $"System.Result.Ok<{output}>", $"System.Result.Error<{error}>" })
                text.AppendLine($".function {Helper(owner, variant)}({owner}& source,out(true) {variant}& destination) -> Boolean\nldarg source\nldobj {owner}\nldarg destination\ncall instance {owner}::TryGet({variant}&)\nret\n.end");
        return text.ToString();
    }
}
