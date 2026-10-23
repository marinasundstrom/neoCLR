// Metadata declarations only: these bodies never execute. The bounded importer binds
// selected signatures to real library methods; TryGetValue projects TryGet.
static class UnionDeclarations
{
    public const string Source = """
        public interface Propagatable<TSelf, TOutput, TResidual> {
            bool TryGetOutput(out TOutput output);
            bool TryGetResidual(out TResidual residual);
        }
        public static class Optional {
            public struct None { public None() { } }
            public struct Some<T> {
                public Some(T value) { Value = value; }
                public T Value { get; set; }
                public void Deconstruct(out T value) { value = Value; }
            }
        }
        public struct PropagationUnit { }
        // Replaced by the source union projection before documentation consumption.
        public struct Union<T1, T2> { }
        [System.Runtime.CompilerServices.Union]
        public struct Optional<T> : Propagatable<Optional<T>, T, PropagationUnit> {
            public bool IsSome => false;
            public bool IsNone => false;
            public Optional.Some<T> GetSomeCase() => default;
            public Optional.None GetNoneCase() => default;
            public bool TryGet(out Optional.Some<T> value) { value = default; return false; }
            public bool TryGet(out Optional.None value) { value = default; return false; }
            public bool TryGetOutput(out T output) { output = default; return false; }
            public bool TryGetResidual(out PropagationUnit residual) { residual = default; return false; }
            public static Optional<T> FromResidual(PropagationUnit residual) => default;
            public object Value => default;
            public Optional(Optional.Some<T> value) { }
            public Optional(Optional.None value) { }
            public bool TryGetValue(out Optional.Some<T> value) { value = default; return false; }
            public bool TryGetValue(out Optional.None value) { value = default; return false; }
        }
        public static class Result {
            public struct Ok<T> {
                public Ok(T value) { Value = value; }
                public T Value { get; set; }
                public void Deconstruct(out T value) { value = Value; }
            }
            public struct Failure<E> {
                public Failure(E value) { Value = value; }
                public E Value { get; set; }
                public void Deconstruct(out E value) { value = Value; }
            }
        }
        [System.Runtime.CompilerServices.Union]
        public struct Result<T, E> : Propagatable<Result<T, E>, T, E> {
            public static Result<T, E> Ok(T value) => default;
            public static Result<T, E> Failure(E error) => default;
            public bool IsOk => false;
            public bool IsOkCase => false;
            public bool IsErr => false;
            public bool IsFailureCase => false;
            public Result.Ok<T> GetOkCase() => default;
            public Result.Failure<E> GetFailureCase() => default;
            public bool TryGet(out Result.Ok<T> value) { value = default; return false; }
            public bool TryGet(out Result.Failure<E> value) { value = default; return false; }
            public bool TryGetOutput(out T output) { output = default; return false; }
            public bool TryGetResidual(out E residual) { residual = default; return false; }
            public static Result<T, E> FromResidual(E residual) => default;
            // Required by Raven's union recognition protocol; not an admitted runtime API.
            public object Value => default;
            public Result(Result.Ok<T> value) { }
            public Result(Result.Failure<E> value) { }
            public bool TryGetValue(out Result.Ok<T> value) { value = default; return false; }
            public bool TryGetValue(out Result.Failure<E> value) { value = default; return false; }
        }
        """;
}
