using Mono.Cecil;

// File API declarations/bindings; carrier mechanics are shared with other APIs.
static class FileBindings
{
    public const string ReadError = ResultBindings.ReadError, WriteError = ResultBindings.WriteError;
    public const string Declarations = """
        namespace Storage { public enum EntryKind { File = 1, Directory = 2 }
        public static class Metadata {
            public static Result<EntryKind, StorageLookupError> GetKind(string path) => default;
        } public static class FileText {
            public static Result<string, FileReadError> ReadAllText(string path, int maxBytes) => default;
            public static Result<PropagationUnit, FileWriteError> WriteAllText(string path, string text, int maxBytes) => default;
        } }
        """;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName == "System.Storage.Metadata") {
            var signature = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
            if (reference.HasThis || !definition.IsPublic || !definition.IsStatic || definition.HasGenericParameters
                || reference.Name != "GetKind" || !signature.Args.SequenceEqual(new[] { "String" })
                || signature.Result != "System.Result<System.Storage.EntryKind,System.Storage.StorageLookupError>")
                throw new InvalidDataException("Unsupported storage metadata signature.");
            return new("System.Storage.Metadata::GetKind", signature.Args, signature.Result);
        }
        if (reference.DeclaringType.FullName != "System.Storage.FileText") return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, ResultBindings.Type);
        if (!reference.HasThis && ((reference.Name == "ReadAllText" && args.SequenceEqual(new[] { "String", "Int32" })
                && result == "System.Result<String,System.Storage.FileReadError>")
            || (reference.Name == "WriteAllText" && args.SequenceEqual(new[] { "String", "String", "Int32" })
                && result == "System.Result<Void,System.Storage.FileWriteError>")))
            return new("System.Storage.FileText::" + reference.Name, args, result);
        throw new InvalidDataException("Unsupported file member: " + reference.FullName);
    }
}
