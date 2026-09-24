using Mono.Cecil;
using System.Text;

// Existing error-value APIs. Empty cases have valid defaults; carriers do not.
static class ErrorBindings
{
    public static readonly Dictionary<string, string[]> Cases = new() {
        ["System.Networking.DnsError"] = ["InvalidName", "LimitExceeded", "NoAddress", "TimedOut", "Cancelled", "LookupFailed"],
        ["System.Networking.Sockets.SocketError"] = ["Closed", "Busy", "InvalidRange", "LimitExceeded", "Cancelled", "InvalidAddress", "ConnectionRefused", "ConnectionReset", "AccessDenied", "TimedOut", "IoFailure"],
        ["System.Storage.InvalidPathError"] = [],
        ["System.Storage.EntryKind"] = ["File", "Directory"],
        ["System.Storage.StorageLookupError"] = ["InvalidPath", "NotFound", "AccessDenied", "WrongKind", "IoFailure", "InvalidRange", "LimitExceeded"],
        ["System.IO.TextReadError"] = ["InvalidPath", "NotFound", "AccessDenied", "WrongKind", "AlreadyExists", "Closed", "InvalidRange", "LimitExceeded", "WrongAccess", "IoFailure", "InvalidUtf8"],
        ["System.IO.StreamError"] = ["InvalidPath", "NotFound", "AccessDenied", "WrongKind", "AlreadyExists", "Closed", "InvalidRange", "LimitExceeded", "WrongAccess", "IoFailure"],
        ["System.Storage.FileReadError"] = ["InvalidLimit", "InvalidPath", "NotFound", "AccessDenied", "NotRegularFile", "TooLarge", "ReadFailed", "InvalidUtf8"],
        ["System.Storage.FileWriteError"] = ["InvalidLimit", "InvalidPath", "NotFound", "AccessDenied", "NotRegularFile", "TooLarge", "WriteFailed"],
        ["System.ConsoleReadError"] = ["Unavailable", "ReadFailed"],
        ["System.Text.InvalidUtf8Error"] = [],
        ["System.Text.Utf8SliceError"] = ["OutOfRange", "InvalidBoundary"],
        ["System.Int32ParseError"] = ["InvalidFormat", "Overflow"],
        ["System.Linq.SingleError"] = ["Empty", "Multiple"],
        ["System.IntegerDivisionError"] = ["DivisionByZero", "Overflow"],
        ["System.InvalidRangeError"] = [], ["System.InvalidDateError"] = [],
        ["System.InvalidTimeError"] = [], ["System.OverflowError"] = [], ["System.EnvironmentError"] = []
    };
    public static IEnumerable<string> Errors => Cases.Keys;
    static IEnumerable<string> CaseTypes => Cases.SelectMany(e => e.Value.Select(c => e.Key + "." + c));
    public static bool IsType(string type) => Errors.Contains(type) || CaseTypes.Contains(type);
    public static bool IsEmpty(string type) => CaseTypes.Contains(type) || Cases.TryGetValue(type, out var cases) && cases.Length == 0;
    public static string? Type(TypeReference type) => type.IsValueType && RuntimeSignatures.IsCore(type.Scope)
        && IsType(type.FullName.Replace('/', '.')) ? type.FullName.Replace('/', '.') : null;
    public static string Declarations => string.Join("\n", Cases.Select(entry => {
        var error = entry.Key; var name = error.Split('.').Last();
        var declaration = (entry.Value.Length > 0 ? "[System.Runtime.CompilerServices.Union] " : "")
            + "public struct " + name + " { public string ToString() => default; "
            + (entry.Value.Length == 0 && error != "System.EnvironmentError" ? $"public {name}() {{ }} " : "")
            + string.Join(" ", entry.Value.Select(c => $"public struct {c} {{ public {c}() {{ }} }} public {name}({c} value) {{ }} public bool Is{c} => false; public {c} Get{c}() => default;")) + " }";
        var separator = error.LastIndexOf('.');
        return separator == 6 ? declaration : "namespace " + error[7..separator] + " { " + declaration + " }";
    }));
    static string Helper(string owner, string member) => "RuntimeError" + new string((owner + member).Where(char.IsLetterOrDigit).ToArray());
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var signature = RuntimeSignatures.Match(reference, definition, Type);
        if (Errors.Contains(owner) && reference.HasThis && !definition.IsVirtual && signature.Args.Length == 0
            && (reference.Name == "ToString" && signature.Result == "String"
                || Cases[owner].Any(c => reference.Name == "get_Is" + c && signature.Result == "Boolean"
                    || reference.Name == "Get" + c && signature.Result == owner + "." + c)))
            return new(Helper(owner, reference.Name), [owner + "&"], signature.Result);
        throw new InvalidDataException("Unsupported error member: " + reference.FullName);
    }
    public static ResultBindings.Binding? Construct(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var signature = RuntimeSignatures.Match(reference, definition, Type);
        if (signature.Result == "noresult" && (IsEmpty(owner) && owner != "System.EnvironmentError" && signature.Args.Length == 0
            || Cases.TryGetValue(owner, out var cases) && cases.Any(c => signature.Args.SequenceEqual(new[] { owner + "." + c }))))
            return new(Helper(owner, "New" + string.Concat(signature.Args)), signature.Args, owner);
        throw new InvalidDataException("Unsupported error constructor.");
    }
    public static string Adapters()
    {
        var text = new StringBuilder();
        foreach (var owner in Errors)
        {
            var methods = Cases[owner].SelectMany(c => new[] { ("get_Is" + c, "Boolean"), ("Get" + c, owner + "." + c) }).Append(("ToString", "String"));
            foreach (var (name, result) in methods)
                text.AppendLine($".function {Helper(owner, name)}({owner}& source) -> {result}\nldarg source\nldobj {owner}\ncall instance {owner}::{name}()\nret\n.end");
            foreach (var arg in Cases[owner].Select(c => owner + "." + c))
                text.AppendLine($".function {Helper(owner, "New" + arg)}({arg} value) -> {owner}\nldarg value\nnewobj instance {owner}::.ctor({arg})\nret\n.end");
        }
        foreach (var owner in Errors.Concat(CaseTypes).Where(t => IsEmpty(t) && t != "System.EnvironmentError"))
            text.AppendLine($".function {Helper(owner, "New")}() -> {owner}\nnewobj instance {owner}::.ctor()\nret\n.end");
        return text.ToString();
    }
}
