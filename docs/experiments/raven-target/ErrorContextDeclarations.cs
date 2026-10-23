// CLI documentation projections. Bodies never execute; native Raven owns implementation.
static class ErrorContextDeclarations
{
    internal const string Source = """
        public interface Error {
            string Message { get; }
            Option<Error> Cause { get; }
            Option<Diagnostics.StackTrace> StackTrace { get; }
        }
        public class ContextError<E> : Error where E : Error {
            public ContextError(string message, E cause) { }
            public ContextError(string message, E cause, bool captureStackTrace) { }
            public string Message => default;
            public E Cause => default;
            Option<Error> Error.Cause => default;
            public Option<Diagnostics.StackTrace> StackTrace => default;
        }
        public static class ErrorDiagnostics {
            public static bool CaptureStackTraces => default;
            public static void ConfigureStackTraceCapture(bool enabled) { }
        }
        public static class ErrorExtensions {
            public static ContextError<E> WithContext<E>(this E self, string message) where E : Error => default;
            public static ContextError<E> WithContext<E>(this E self, string message, bool captureStackTrace) where E : Error => default;
        }
        public static class ResultErrorContextExtensions {
            public static Result<T,ContextError<E>> WithContext<T,E>(this Result<T,E> self, string message) where E : Error => default;
            public static Result<T,ContextError<E>> WithContext<T,E>(this Result<T,E> self, string message, bool captureStackTrace) where E : Error => default;
            public static Result<T,ContextError<E>> WithContext<T,E>(this Result<T,E> self, Func<string> message) where E : Error => default;
            public static Result<T,ContextError<E>> WithContext<T,E>(this Result<T,E> self, Func<string> message, bool captureStackTrace) where E : Error => default;
        }
        public static class ErrorFormattingExtensions {
            public static string FormatError(this Error self) => default;
        }
        namespace Diagnostics {
            [System.AttributeUsage(System.AttributeTargets.Method | System.AttributeTargets.Class, AllowMultiple = true)]
            public class ConditionalAttribute : System.Attribute {
                public ConditionalAttribute(string conditionString) { }
                public string ConditionString => default;
            }
        }
        """;

    internal static string ProjectLeaves(string source)
    {
        foreach (var name in new[] { "FileReadError", "NumberParseError", "StreamError" })
            source = source.Replace("public struct " + name + " {",
                "public struct " + name + " : System.Error { public string Message => default; " +
                "public System.Option<System.Error> Cause => default; " +
                "public System.Option<System.Diagnostics.StackTrace> StackTrace => default; ");
        return source;
    }
}
