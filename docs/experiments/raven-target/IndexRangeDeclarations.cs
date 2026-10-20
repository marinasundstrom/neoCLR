// Documentation reference for native library value contracts. Runtime execution
// imports the native System.Runtime artifact rather than this aggregate facade.
static class IndexRangeDeclarations
{
    public const string Source = """
        public struct Index : EquatableTo<Index> {
            public Index(int value) { }
            public Index(int value, bool fromEnd) { }
            public int Value => default;
            public bool IsFromEnd => default;
            public static Index Start => default;
            public static Index End => default;
            public static Index FromStart(int value) => default;
            public static Index FromEnd(int value) => default;
            public static implicit operator Index(int value) => default;
            public int GetOffset(int length) => default;
            public bool Equals(Index other) => default;
            public override bool Equals(object other) => default;
            public override int GetHashCode() => default;
            public override string ToString() => default;
        }
        public struct Range : EquatableTo<Range> {
            public Range(Index start, Index end) { }
            public Index Start => default;
            public Index End => default;
            public static Range All => default;
            public static Range StartAt(Index start) => default;
            public static Range EndAt(Index end) => default;
            public Tuple<int, int> GetOffsetAndLength(int length) => default;
            public Result<Tuple<int, int>, InvalidRangeError> TryGetOffsetAndLength(int length) => default;
            public bool Equals(Range other) => default;
            public override bool Equals(object other) => default;
            public override int GetHashCode() => default;
            public override string ToString() => default;
        }
        namespace Runtime.CompilerServices {
            public static class RuntimeHelpers {
                public static void DrainEntryTasks() { }
                public static bool IsAssignable<TSource, TTarget>() => default;
                public static T[] GetSubArray<T>(T[] array, Range range) => default;
            }
        }
        """;
}
