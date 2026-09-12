// Emission-only contract probe. These bodies must never execute or enter the admitted
// runtime catalog. TryGetValue projects the current neoCLR Result.TryGet operation.
static class UnionDeclarations
{
    public const string Source = """
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
        public struct Result<T, E> {
            public Result(Result.Ok<T> value) { }
            public Result(Result.Error<E> value) { }
            public bool TryGetValue(out Result.Ok<T> value) { value = default; return false; }
            public bool TryGetValue(out Result.Error<E> value) { value = default; return false; }
        }
        """;
}
