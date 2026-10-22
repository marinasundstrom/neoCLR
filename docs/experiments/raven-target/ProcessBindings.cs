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
        ("System.Console", "PeekChar", [], "System.Result<System.Option<Char>,System.IO.TextReadError>"),
        ("System.Console", "ReadChar", [], "System.Result<System.Option<Char>,System.IO.TextReadError>"),
        ("System.Console", "Read", ["Int32"], "System.Result<String,System.IO.TextReadError>"),
        ("System.Console", "Read", ["Int32", "Int32"], "System.Result<String,System.IO.TextReadError>"),
        ("System.Console", "ReadLine", [], "System.Result<System.Option<String>,System.IO.TextReadError>"),
        ("System.Console", "ReadLine", ["Int32"], "System.Result<System.Option<String>,System.IO.TextReadError>"),
        ("System.Console", "Write", ["String"], "void"),
        ("System.Console", "Write", ["Int32"], "void"),
        ("System.Console", "WriteLine", [], "void"),
        ("System.Console", "WriteLine", ["String"], "void"),
        ("System.Console", "WriteLine", ["Int32"], "void"),
        ("System.Console", "WriteLine", ["System.Object"], "void"),
        ("System.Console", "WriteLine", ["Boolean"], "void"),
        ("System.Console", "WriteLine", ["Char"], "void"),
        ("System.Console", "WriteLine", ["SByte"], "void"),
        ("System.Console", "WriteLine", ["Byte"], "void"),
        ("System.Console", "WriteLine", ["Int16"], "void"),
        ("System.Console", "WriteLine", ["UInt16"], "void"),
        ("System.Console", "WriteLine", ["UInt32"], "void"),
        ("System.Console", "WriteLine", ["Int64"], "void"),
        ("System.Console", "WriteLine", ["UInt64"], "void"),
        ("System.Console", "WriteLine", ["IntPtr"], "void"),
        ("System.Console", "WriteLine", ["UIntPtr"], "void"),
        ("System.Console", "ReadByte", [], "System.Result<System.Option<Byte>,System.ConsoleReadError>"),
        ("System.Environment", "GetCommandLineArgs", [], "arrayref<String>"),
        ("System.Environment", "GetCurrentDirectory", [], "System.Result<String,System.EnvironmentError>"),
        ("System.Environment", "GetEnvironmentVariable", ["String"], "System.Result<System.Option<String>,System.EnvironmentError>")
    ];
    public const string ConsoleDeclaration = """
        public static Result<ConsoleKeyReader, ConsoleKeyError> OpenKeyInput() => default;
        public static Result<ConsoleKeyReader, ConsoleKeyError> OpenKeyInput(bool captureInterrupts) => default;
        public static void Write(string format, object? arg0) { }
        public static void Write(string format, object? arg0, object? arg1) { }
        public static void Write(string format, object? arg0, object? arg1, object? arg2) { }
        public static void Write(string format, params object?[] arguments) { }
        public static void WriteLine(string format, object? arg0) { }
        public static void WriteLine(string format, object? arg0, object? arg1) { }
        public static void WriteLine(string format, object? arg0, object? arg1, object? arg2) { }
        public static void WriteLine(string format, params object?[] arguments) { }
        public static Result<Option<byte>, ConsoleReadError> ReadByte() => default;
        public static Result<Option<char>, IO.TextReadError> PeekChar() => default;
        public static Result<Option<char>, IO.TextReadError> ReadChar() => default;
        public static Result<string, IO.TextReadError> Read(int count) => default;
        public static Result<string, IO.TextReadError> Read(int count, int maxUtf8Bytes) => default;
        public static IO.SyncTextReader In => default;
        public static IO.SyncTextWriter Out => default;
        public static IO.SyncTextWriter Error => default;
        public static IO.SyncInputStream OpenStandardInput() => default;
        public static IO.SyncOutputStream OpenStandardOutput() => default;
        public static IO.SyncOutputStream OpenStandardError() => default;
        public static Result<Option<string>, IO.TextReadError> ReadLine() => default;
        public static Result<Option<string>, IO.TextReadError> ReadLine(int maxUtf8Bytes) => default;
        public static void Write(string value0) { }
        public static void Write(int value0) { }
        public static void WriteLine() { }
        public static void WriteLine(object? value) { }
        public static void WriteLine(bool value) { }
        public static void WriteLine(char value) { }
        public static void WriteLine(sbyte value) { }
        public static void WriteLine(byte value) { }
        public static void WriteLine(short value) { }
        public static void WriteLine(ushort value) { }
        public static void WriteLine(uint value) { }
        public static void WriteLine(long value) { }
        public static void WriteLine(ulong value) { }
        public static void WriteLine(System.IntPtr value) { }
        public static void WriteLine(System.UIntPtr value) { }
        """;
    public const string Declarations = """
        public enum ConsoleKey {
            None = 0,
            Backspace = 8,
            Tab = 9,
            Enter = 13,
            Escape = 27,
            Space = 32,
            D0 = 48,
            D1 = 49,
            D2 = 50,
            D3 = 51,
            D4 = 52,
            D5 = 53,
            D6 = 54,
            D7 = 55,
            D8 = 56,
            D9 = 57,
            A = 65,
            B = 66,
            C = 67,
            D = 68,
            E = 69,
            F = 70,
            G = 71,
            H = 72,
            I = 73,
            J = 74,
            K = 75,
            L = 76,
            M = 77,
            N = 78,
            O = 79,
            P = 80,
            Q = 81,
            R = 82,
            S = 83,
            T = 84,
            U = 85,
            V = 86,
            W = 87,
            X = 88,
            Y = 89,
            Z = 90,
            UpArrow = 256,
            DownArrow = 257,
            RightArrow = 258,
            LeftArrow = 259,
            Home = 260,
            End = 261,
            Insert = 262,
            Delete = 263,
            PageUp = 264,
            PageDown = 265,
            F1 = 272,
            F2 = 273,
            F3 = 274,
            F4 = 275,
            F5 = 276,
            F6 = 277,
            F7 = 278,
            F8 = 279,
            F9 = 280,
            F10 = 281,
            F11 = 282,
            F12 = 283
        }
        public struct ConsoleKeyInfo : EquatableTo<ConsoleKeyInfo> {
            public ConsoleKeyInfo(ConsoleKey key, bool shift, bool alt, bool control, Option<int> textScalar) { }
            public ConsoleKey Key => default;
            public bool Shift => default;
            public bool Alt => default;
            public bool Control => default;
            public Option<int> TextScalar => default;
            public bool Equals(ConsoleKeyInfo other) => default;
            public override bool Equals(object? obj) => default;
            public override int GetHashCode() => default;
            public static bool operator ==(ConsoleKeyInfo left, ConsoleKeyInfo right) => default;
            public static bool operator !=(ConsoleKeyInfo left, ConsoleKeyInfo right) => default;
        }
        public class ConsoleKeyReader : Disposable {
            internal ConsoleKeyReader() { }
            public Result<Option<ConsoleKeyInfo>, ConsoleKeyError> ReadKey() => default;
            public Result<PropagationUnit, ConsoleKeyError> Close() => default;
            public void Dispose() { }
        }
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
        if (result == "void")
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
