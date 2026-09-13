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
                public ArrayList() { }
                public ArrayList(int capacity) { }
                public int FindIndex(Func<T, bool> match) => default;
                public bool Exists(Func<T, bool> match) => default;
                public Option<T> Find(Func<T, bool> match) => default;
                public ArrayList<T> Copy() => default;
                public int Capacity => default;
                public int Count => default;
                public T this[int index] { get => default; set { } }
                public void Add(T value) { }
                public Iterator<T> GetIterator() => default;
            }
        }
        """;
}
