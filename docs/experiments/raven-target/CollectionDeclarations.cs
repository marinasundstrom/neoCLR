// Reference surface for the bounded collection profile. These declarations are not executable library replacements.
static class CollectionDeclarations
{
    public const string Source = """
        public interface Disposable { void Dispose(); }
        namespace Collections {
            public interface Iterable<T> { Iterator<T> GetIterator(); }
            public interface Iterator<T> : Disposable {
                bool MoveNext();
                T Current { get; }
            }
            public interface List<T> : Iterable<T> {
                int Count { get; }
                T this[int index] { get; set; }
                void Add(T value);
            }
            public class ArrayList<T> : List<T> {
                public static ArrayList<T> Allocate(int capacity) => default;
                public int Count => default;
                public T this[int index] { get => default; set { } }
                public void Add(T value) { }
                public Iterator<T> GetIterator() => default;
            }
        }
        """;
}
