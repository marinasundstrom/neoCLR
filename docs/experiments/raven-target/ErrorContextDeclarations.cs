// CLI documentation projections. Bodies never execute; native Raven owns implementation.
static class ErrorContextDeclarations
{
    internal const string Source = """
        public interface Error {
            string Message { get; }
            Optional<Error> Cause { get; }
            Optional<Diagnostics.StackTrace> StackTrace { get; }
        }
        public class ContextError<E> : Error where E : Error {
            public ContextError(string message, E cause) { }
            public ContextError(string message, E cause, bool captureStackTrace) { }
            public string Message => default;
            public E Cause => default;
            Optional<Error> Error.Cause => default;
            public Optional<Diagnostics.StackTrace> StackTrace => default;
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
        namespace Diagnostics.CodeAnalysis {
            [System.Flags]
            public enum DynamicallyAccessedMemberTypes {
                None = 0, PublicParameterlessConstructor = 1, PublicConstructors = 3,
                NonPublicConstructors = 4, PublicMethods = 8, NonPublicMethods = 16,
                PublicFields = 32, NonPublicFields = 64, PublicNestedTypes = 128,
                NonPublicNestedTypes = 256, PublicProperties = 512, NonPublicProperties = 1024,
                PublicEvents = 2048, NonPublicEvents = 4096, Interfaces = 8192, All = -1
            }
            [System.AttributeUsage(System.AttributeTargets.Parameter | System.AttributeTargets.ReturnValue | System.AttributeTargets.Field | System.AttributeTargets.Property)]
            public class DynamicallyAccessedMembersAttribute : System.Attribute {
                public DynamicallyAccessedMembersAttribute(DynamicallyAccessedMemberTypes memberTypes) { }
                public DynamicallyAccessedMemberTypes MemberTypes => default;
            }
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
                "public System.Optional<System.Error> Cause => default; " +
                "public System.Optional<System.Diagnostics.StackTrace> StackTrace => default; ");
        return source;
    }
}
