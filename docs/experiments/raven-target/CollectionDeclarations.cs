// Reference surface for the bounded collection profile. These declarations are not executable library replacements.
static class CollectionDeclarations
{
    public const string Source = """
        // Interface metadata for CLI vectors; allocation still uses newarr.
        public sealed class Array<T> : Collections.MutableSequence<T> {
            private Array() { }
            public static T[] Empty => default;
            public void ForEach(Func<T, PropagationUnit> action) { }
            public int Count => default;
            public T this[int index] { get => default; set { } }
            public Collections.Iterator<T> GetIterator() => default;
        }
        public interface Disposable { void Dispose(); }
        namespace Collections {
            public interface Queue<T> : Collection<T> {
                void Enqueue(T value);
                Option<T> Dequeue();
                Option<T> Peek();
                void Clear();
            }
            public class ArrayQueue<T> : Queue<T> {
                public ArrayQueue() { }
                public ArrayQueue(int capacity) { }
                public int Count => default;
                public int Capacity => default;
                public void Enqueue(T value) { }
                public Option<T> Dequeue() => default;
                public Option<T> Peek() => default;
                public void Clear() { }
                public Iterator<T> GetIterator() => default;
            }
            public interface Stack<T> : Collection<T> {
                void Push(T value);
                Option<T> Pop();
                Option<T> Peek();
                void Clear();
            }
            public class ArrayStack<T> : Stack<T> {
                public ArrayStack() { }
                public ArrayStack(int capacity) { }
                public int Count => default;
                public int Capacity => default;
                public void Push(T value) { }
                public Option<T> Pop() => default;
                public Option<T> Peek() => default;
                public void Clear() { }
                public Iterator<T> GetIterator() => default;
            }
            public interface Set<T> : Collection<T> {
                bool Contains(T value);
            }
            public interface MutableSet<T> : Set<T> {
                bool Add(T value);
                bool Remove(T value);
                void Clear();
            }
            public class HashSet<T> : MutableSet<T> {
                public HashSet(EqualityComparer<T> comparer) { }
                public int Count => default;
                public bool Contains(T value) => default;
                public bool Add(T value) => default;
                public bool Remove(T value) => default;
                public void Clear() { }
                public Iterator<T> GetIterator() => default;
            }
            public interface Map<K, V> {
                int Count { get; }
                Sequence<K> Keys { get; }
                Option<V> Find(K key);
                bool ContainsKey(K key);
            }
            public interface MutableMap<K, V> : Map<K, V> {
                bool TryAdd(K key, V value);
                void Set(K key, V value);
            }
            public sealed class HashMap<K, V> : MutableMap<K, V> {
                public HashMap(Func<K, K, bool> equal, Func<K, int> hash) { }
                public HashMap(EqualityComparer<K> comparer) { }
                public int Count => default;
                public Sequence<K> Keys => default;
                public Option<V> Find(K key) => default;
                public bool ContainsKey(K key) => default;
                public bool TryAdd(K key, V value) => default;
                public void Set(K key, V value) { }
            }
            public interface Iterable<T> { Iterator<T> GetIterator(); }
            public interface Iterator<T> : Disposable {
                bool MoveNext();
                T Current { get; }
            }
            public interface Collection<T> : Iterable<T> {
                int Count { get; }
            }
            public interface Sequence<T> : Collection<T> {
                T this[int index] { get; }
            }
            public interface MutableSequence<T> : Sequence<T> {
                new T this[int index] { get; set; }
            }
            public interface List<T> : MutableSequence<T> {
                void Add(T value);
            }
            public class ArrayList<T> : List<T> {
                public ArrayList() { }
                public ArrayList(int capacity) { }
                public Option<int> FindIndex(Func<T, bool> match) => default;
                public Option<int> FindLastIndex(Func<T, bool> match) => default;
                public Option<T> FindLast(Func<T, bool> match) => default;
                public ArrayList<T> FindAll(Func<T, bool> match) => default;
                public bool TrueForAll(Func<T, bool> match) => default;
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
