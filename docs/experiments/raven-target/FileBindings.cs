using Mono.Cecil;

// File API declarations/bindings; carrier mechanics are shared with other APIs.
static class FileBindings
{
    public const string ReadError = ResultBindings.ReadError, WriteError = ResultBindings.WriteError;
    public const string Declarations = """
        namespace Storage { public static class File {
            public static Result<string, FileReadError> ReadAllText(string path, int maxBytes) => default;
            public static Result<PropagationUnit, FileWriteError> WriteAllText(string path, string text, int maxBytes) => default;
        } }
        """;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != "System.Storage.File") return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, ResultBindings.Type);
        if (!reference.HasThis && ((reference.Name == "ReadAllText" && args.SequenceEqual(new[] { "String", "Int32" })
                && result == "System.Result<String,System.Storage.FileReadError>")
            || (reference.Name == "WriteAllText" && args.SequenceEqual(new[] { "String", "String", "Int32" })
                && result == "System.Result<Void,System.Storage.FileWriteError>")))
            return new("System.Storage.File::" + reference.Name, args, result);
        throw new InvalidDataException("Unsupported file member: " + reference.FullName);
    }
}
