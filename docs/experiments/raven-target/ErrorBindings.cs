using Mono.Cecil;
using System.Text;

// Existing error-value APIs. Empty cases have valid defaults; carriers do not.
static class ErrorBindings
{
    public static readonly Dictionary<string, string[]> Cases = new() {
        ["System.Networking.DnsError"] = ["InvalidName", "LimitExceeded", "NoAddress", "TimedOut", "Cancelled", "LookupFailed"],
        ["System.Networking.Sockets.SocketError"] = ["Closed", "Busy", "InvalidRange", "LimitExceeded", "Cancelled", "InvalidAddress", "ConnectionRefused", "ConnectionReset", "AccessDenied", "TimedOut", "IoFailure", "InvalidOperation", "AddressInUse"],
        ["System.UriError"] = ["InvalidFormat", "UnsupportedAuthority", "TooLong", "BaseNotAbsolute"],
        ["System.Storage.InvalidPathError"] = [],
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
    static readonly HashSet<string> Standard = new();
    public static bool IsStandard(string type) => Standard.Contains(type) || PayloadUnionBindings.IsType(type);
    public static void Reset(ModuleDefinition core)
    {
        Standard.Clear();
        PayloadUnionBindings.Reset(core);
        foreach (var (name, cases) in Cases)
            if (core.GetType(name) is { } type && ApplicationTypes.IsEmptyCaseUnion(type))
            {
                var metadata = RavenUnionMetadata.ValidateNestedCases(type);
                if (!metadata.Select(c => c.Name).SequenceEqual(cases))
                    throw new InvalidDataException("Unexpected standard error case catalog: " + name);
                Standard.Add(name);
            }
    }
    public static IEnumerable<string> Errors => Cases.Keys;
    static IEnumerable<string> CaseTypes => Cases.SelectMany(e => e.Value.Select(c => e.Key + "." + c));
    public static bool IsType(string type) => PayloadUnionBindings.IsType(type) || Errors.Contains(type) || CaseTypes.Contains(type);
    public static bool IsEmpty(string type) => CaseTypes.Contains(type) || Cases.TryGetValue(type, out var cases) && cases.Length == 0;
    public static string? Type(TypeReference type) => PayloadUnionBindings.Type(type) ?? (RuntimeSignatures.IsCore(type.Scope)
        && (Errors.Contains(type.FullName.Replace('/', '.')) || CaseTypes.Contains(type.FullName.Replace('/', '.')))
        // isinst operands can omit the CLI valuetype signature flag. Resolve the
        // supplied-core definition rather than rejecting a valid case type token.
        && (type.IsValueType || type.Resolve()?.IsValueType == true) ? type.FullName.Replace('/', '.') : null);
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
        if (PayloadUnionBindings.IsType(reference.DeclaringType.FullName.Replace('/', '.')))
            return PayloadUnionBindings.Bind(reference, definition);
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var signature = RuntimeSignatures.Match(reference, definition,
            type => type.FullName == "System.Object" && (type.MetadataType == MetadataType.Object || RuntimeSignatures.IsCore(type.Scope)) ? "System.Object" : Type(type));
        if (IsStandard(owner))
        {
            var output = ApplicationTypes.IsConditionalUnionOutput(definition);
            var caseNames = Cases[owner].Select(c => owner + "." + c).ToArray();
            var allowed = reference.HasThis
                ? output || signature.Args.Length == 0 && (reference.Name == "get_HasValue" && signature.Result == "Boolean"
                    || reference.Name == "get_Value" && signature.Result == "System.Object"
                    || reference.Name == "ToString" && signature.Result == "String"
                    || Cases[owner].Any(c => reference.Name == "get_Is" + c && signature.Result == "Boolean"
                        || reference.Name == "Get" + c && signature.Result == owner + "." + c))
                : reference.Name == "op_Implicit" && signature.Args.Length == 1 && caseNames.Contains(signature.Args[0]) && signature.Result == owner
                    || reference.Name == "op_Explicit" && signature.Args.SequenceEqual(new[] { owner }) && caseNames.Contains(signature.Result)
                    || signature.Args.Length == 0 && Cases[owner].Any(c => reference.Name == "get_" + c && signature.Result == owner + "." + c);
            if (!allowed) throw new InvalidDataException("Unsupported standard error member: " + reference.FullName);
            var name = ApplicationTypes.MethodName(definition);
            return new(owner + "::" + name,
                (reference.HasThis ? new[] { owner + "&" } : []).Concat(signature.Args).ToArray(), signature.Result,
                output ? 1 : -1, Instruction: $"call {(reference.HasThis ? "instance " : "")}{owner}::{name}({string.Join(',', signature.Args)})");
        }
        if (Errors.Contains(owner) && reference.HasThis && !definition.IsVirtual && signature.Args.Length == 0
            && (reference.Name == "ToString" && signature.Result == "String"
                || Cases[owner].Any(c => reference.Name == "get_Is" + c && signature.Result == "Boolean"
                    || reference.Name == "Get" + c && signature.Result == owner + "." + c)))
            return new(Helper(owner, reference.Name), [owner + "&"], signature.Result);
        throw new InvalidDataException("Unsupported error member: " + reference.FullName);
    }
    public static ResultBindings.Binding? Construct(MethodReference reference, MethodDefinition definition)
    {
        if (PayloadUnionBindings.IsType(reference.DeclaringType.FullName.Replace('/', '.')))
            return PayloadUnionBindings.Bind(reference, definition, construct: true);
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
            var methods = IsStandard(owner) ? Enumerable.Empty<(string, string)>() : Cases[owner].SelectMany(c => new[] { ("get_Is" + c, "Boolean"), ("Get" + c, owner + "." + c) }).Append(("ToString", "String"));
            foreach (var (name, result) in methods)
                text.AppendLine($".function {Helper(owner, name)}({owner}& source) -> {result}\nldarg source\n{(IsStandard(owner) ? "" : "ldobj " + owner + "\n")}call instance {owner}::{name}()\nret\n.end");
            foreach (var arg in Cases[owner].Select(c => owner + "." + c))
                text.AppendLine($".function {Helper(owner, "New" + arg)}({arg} value) -> {owner}\nldarg value\nnewobj instance {owner}::.ctor({arg})\nret\n.end");
        }
        foreach (var owner in Errors.Concat(CaseTypes).Where(t => IsEmpty(t) && t != "System.EnvironmentError"))
            text.AppendLine($".function {Helper(owner, "New")}() -> {owner}\nnewobj instance {owner}::.ctor()\nret\n.end");
        return text.ToString();
    }
}
