// Metadata declarations only: these bodies never execute. The bounded importer binds
// selected signatures to real library methods; TryGetValue projects TryGet.
static class UnionDeclarations
{
    public const string Source = """
        public interface Propagatable<TSelf, TOutput, TResidual> {
            bool TryGetOutput(out TOutput output);
            bool TryGetResidual(out TResidual residual);
        }
        public static class Option {
            public struct None { public None() { } }
            public struct Some<T> {
                public Some(T value) { Value = value; }
                public T Value { get; }
            }
        }
        public struct PropagationUnit { }
        [System.Runtime.CompilerServices.Union]
        public struct Option<T> : Propagatable<Option<T>, T, PropagationUnit> {
            public bool TryGetOutput(out T output) { output = default; return false; }
            public bool TryGetResidual(out PropagationUnit residual) { residual = default; return false; }
            public static Option<T> FromResidual(PropagationUnit residual) => default;
            public object Value => default;
            public Option(Option.Some<T> value) { }
            public Option(Option.None value) { }
            public bool TryGetValue(out Option.Some<T> value) { value = default; return false; }
            public bool TryGetValue(out Option.None value) { value = default; return false; }
        }
        public struct OverflowError { }
        public static class Result {
            public struct Ok<T> {
                public Ok(T value) { Value = value; }
                public T Value { get; }
            }
            public struct Error<E> {
                public Error(E value) { Value = value; }
                public E Value { get; }
            }
        }
        [System.Runtime.CompilerServices.Union]
        public struct Result<T, E> : Propagatable<Result<T, E>, T, E> {
            public bool TryGetOutput(out T output) { output = default; return false; }
            public bool TryGetResidual(out E residual) { residual = default; return false; }
            public static Result<T, E> FromResidual(E residual) => default;
            // Required by Raven's union recognition protocol; not an admitted runtime API.
            public object Value => default;
            public Result(Result.Ok<T> value) { }
            public Result(Result.Error<E> value) { }
            public bool TryGetValue(out Result.Ok<T> value) { value = default; return false; }
            public bool TryGetValue(out Result.Error<E> value) { value = default; return false; }
        }
        """;
}
