using Mono.Cecil;

// Existing process APIs; host services remain behind their library contracts.
static class ProcessBindings
{
    static readonly (string Owner, string Name, string[] Args, string Result)[] Members = [
        ("System.Console", "get_In", [], "System.IO.TextReader"),
        ("System.Console", "get_Out", [], "System.IO.TextWriter"),
        ("System.Console", "get_Error", [], "System.IO.TextWriter"),
        ("System.Console", "OpenStandardInput", [], "System.IO.InputStream"),
        ("System.Console", "OpenStandardOutput", [], "System.IO.OutputStream"),
        ("System.Console", "OpenStandardError", [], "System.IO.OutputStream"),
        ("System.Console", "ReadLine", [], "System.Result<System.Option<String>,System.IO.TextReadError>"),
        ("System.Console", "ReadLine", ["Int32"], "System.Result<System.Option<String>,System.IO.TextReadError>"),
        ("System.Console", "Write", ["String"], "noresult"),
        ("System.Console", "Write", ["Int32"], "noresult"),
        ("System.Console", "WriteLine", [], "noresult"),
        ("System.Console", "WriteLine", ["String"], "noresult"),
        ("System.Console", "WriteLine", ["Int32"], "noresult"),
        ("System.Console", "ReadByte", [], "System.Result<System.Option<Byte>,System.ConsoleReadError>"),
        ("System.Environment", "GetCommandLineArgs", [], "arrayref<String>"),
        ("System.Environment", "GetCurrentDirectory", [], "System.Result<String,System.EnvironmentError>"),
        ("System.Environment", "GetEnvironmentVariable", ["String"], "System.Result<System.Option<String>,System.EnvironmentError>")
    ];
    public const string ConsoleDeclaration = """
        public static Result<Option<byte>, ConsoleReadError> ReadByte() => default;
        public static IO.TextReader In => default;
        public static IO.TextWriter Out => default;
        public static IO.TextWriter Error => default;
        public static IO.InputStream OpenStandardInput() => default;
        public static IO.OutputStream OpenStandardOutput() => default;
        public static IO.OutputStream OpenStandardError() => default;
        public static Result<Option<string>, IO.TextReadError> ReadLine() => default;
        public static Result<Option<string>, IO.TextReadError> ReadLine(int maxUtf8Bytes) => default;
        public static void Write(string value0) { }
        public static void Write(int value0) { }
        public static void WriteLine() { }
        """;
    public const string Declarations = """
        public static class Environment {
            public static string[] GetCommandLineArgs() => default;
            public static Result<string, EnvironmentError> GetCurrentDirectory() => default;
            public static Result<Option<string>, EnvironmentError> GetEnvironmentVariable(string name) => default;
        }
        """;
    public static string? ArrayType(TypeReference type) => type is ArrayType { IsVector: true } array
        && array.ElementType.MetadataType == MetadataType.String ? "arrayref<String>" : null;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (!Members.Any(m => m.Owner == reference.DeclaringType.FullName && m.Name == reference.Name)) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, t => ArrayType(t) ?? GenericUnionBindings.Type(t));
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis || definition.IsVirtual
            || !Members.Any(m => m.Owner == reference.DeclaringType.FullName && m.Name == reference.Name
                && m.Args.SequenceEqual(args) && m.Result == result))
            throw new InvalidDataException("Unsupported process member: " + reference.FullName);
        if (result == "noresult")
            return new("", args, result, Instruction: $"call System.Console::{reference.Name}({string.Join(',', args)})\npop");
        return new(reference.Name == "GetCommandLineArgs" ? "RuntimeArguments" : reference.DeclaringType.FullName + "::" + reference.Name, args, result);
    }
    // The native service currently produces an owned vector. Project a fresh managed
    // array into CLI code, preserving the existing per-call independent snapshot.
    public static string Adapters(bool managedArguments) => managedArguments
        ? ".function RuntimeArguments() -> arrayref<String>\ncall System.Environment::GetCommandLineArgs()\nret\n.end\n"
        : LegacyAdapters;
    const string LegacyAdapters = """
        .function RuntimeArguments() -> arrayref<String>
            .local String[] source
            .local arrayref<String> destination
            .local Int32 index
            call System.Environment::GetCommandLineArgs()
            stloc source
            ldloc source
            ldlen
            conv.i4
            newarr String
            stloc destination
            ldc.i4 0
            stloc index
            br Test
        Copy:
            ldloc destination
            ldloc index
            ldloc source
            ldloc index
            ldelem String
            stelem String
            ldloc index
            ldc.i4 1
            add
            stloc index
        Test:
            ldloc index
            ldloc source
            ldlen
            conv.i4
            blt Copy
            ldloc destination
            ret
        .end
        """;
}
