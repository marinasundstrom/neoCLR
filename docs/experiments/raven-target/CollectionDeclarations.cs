// Reference surface for the bounded collection profile. These declarations are not executable library replacements.
static class CollectionDeclarations
{
    public const string Source = """
        // Interface metadata for CLI vectors; allocation still uses newarr.
        public sealed class Array<T> : Collections.MutableSequence<T> {
            public T[] this[Range range] => default;
            public Optional<T> Get(int index) => default;
            public Optional<T> Get(Index index) => default;
            public Result<T[], InvalidRangeError> Slice(Range range) => default;
            private Array() { }
            public static T[] Empty => default;
            public void ForEach(Func<T, PropagationUnit> action) { }
            public int Count => default;
            public T this[int index] { get => default; set { } }
            public Collections.Iterator<T> GetIterator() => default;
        }
        public interface Disposable { void Dispose(); }
        public interface AsyncDisposable { Concurrency.Tasks.Task<PropagationUnit> DisposeAsync(); }
        namespace Collections {
            public interface Queue<T> : Collection<T> {
                void Enqueue(T value);
                Optional<T> Dequeue();
                Optional<T> Peek();
                void Clear();
            }
            public class ArrayQueue<T> : Queue<T> {
                public ArrayQueue() { }
                public ArrayQueue(int capacity) { }
                public ArrayQueue(Iterable<T> items) { }
                public int Count => default;
                public int Capacity => default;
                public void Enqueue(T value) { }
                public Optional<T> Dequeue() => default;
                public Optional<T> Peek() => default;
                public void Clear() { }
                public Iterator<T> GetIterator() => default;
            }
            public class PriorityArrayQueue<TElement, TPriority> {
                public PriorityArrayQueue(Comparer<TPriority> comparer) { }
                public PriorityArrayQueue(int capacity, Comparer<TPriority> comparer) { }
                public int Count => default;
                public int Capacity => default;
                public Comparer<TPriority> Comparer => default;
                public int EnsureCapacity(int capacity) => default;
                public void Enqueue(TElement element, TPriority priority) { }
                public Optional<TElement> Dequeue() => default;
                public Optional<TElement> Peek() => default;
                public Optional<TPriority> PeekPriority() => default;
                public void Clear() { }
            }
            public interface Stack<T> : Collection<T> {
                void Push(T value);
                Optional<T> Pop();
                Optional<T> Peek();
                void Clear();
            }
            public class ArrayStack<T> : Stack<T> {
                public ArrayStack() { }
                public ArrayStack(int capacity) { }
                public ArrayStack(Iterable<T> items) { }
                public int Count => default;
                public int Capacity => default;
                public void Push(T value) { }
                public Optional<T> Pop() => default;
                public Optional<T> Peek() => default;
                public void Clear() { }
                public Iterator<T> GetIterator() => default;
            }
            public interface Set<T> : Collection<T> {
                bool Contains(T value);
            }
            public interface MutableSet<T> : Set<T> {
                Optional<T> Find(T value);
                bool Add(T value);
                bool Remove(T value);
                void Clear();
            }
            public class HashSet<T> : MutableSet<T> {
                public Optional<T> Find(T value) => default;
                public HashSet(EqualityComparer<T> comparer) { }
                public HashSet(Iterable<T> items, EqualityComparer<T> comparer) { }
                public int Count => default;
                public bool Contains(T value) => default;
                public bool Add(T value) => default;
                public bool Remove(T value) => default;
                public void Clear() { }
                public Iterator<T> GetIterator() => default;
            }
            public readonly struct KeyValuePair<K, V> : EquatableTo<KeyValuePair<K, V>> {
                public KeyValuePair(K Key, V Value) { this.Key = Key; this.Value = Value; }
                public K Key { get; init; }
                public V Value { get; init; }
                public void Deconstruct(out K Key, out V Value) { Key = this.Key; Value = this.Value; }
                public bool Equals(KeyValuePair<K, V> other) => default;
                public override bool Equals(object other) => default;
                public override int GetHashCode() => default;
                public static bool operator ==(KeyValuePair<K, V> left, KeyValuePair<K, V> right) => default;
                public static bool operator !=(KeyValuePair<K, V> left, KeyValuePair<K, V> right) => default;
            }
            public interface Map<K, V> : Iterable<KeyValuePair<K, V>> {
                V this[K key] { get; }
                int Count { get; }
                Sequence<K> Keys { get; }
                Optional<V> Find(K key);
                bool ContainsKey(K key);
            }
            public interface MutableMap<K, V> : Map<K, V> {
                new V this[K key] { get; set; }
                bool TryAdd(K key, V value);
                void Set(K key, V value);
            }
            public sealed class MapEntry<K, V> {
                public MapEntry(MutableMap<K, V> map, K key) { }
                public K Key => default;
                public V Value => default;
                public Optional<V> Find() => default;
                public MapEntry<K, V> Set(V value) => default;
                public V OrInsert(V value) => default;
                public V OrInsertWith(Func<V> factory) => default;
                public MapEntry<K, V> AndModify(Func<V, V> transform) => default;
            }
            public static class MutableMapExtensions {
                public static MapEntry<K, V> Entry<K, V>(this MutableMap<K, V> self, K key) => default;
            }
            public static class MutableSetExtensions {
                public static T GetOrInsert<T>(this MutableSet<T> self, T value) => default;
            }
            public sealed class HashMap<K, V> : MutableMap<K, V> {
                public V this[K key] { get => default; set { } }
                public HashMap(Func<K, K, bool> equal, Func<K, int> hash) { }
                public HashMap(EqualityComparer<K> comparer) { }
                public HashMap(Iterable<KeyValuePair<K, V>> items, EqualityComparer<K> comparer) { }
                public HashMap(Iterable<KeyValuePair<K, V>> items, Func<K, K, bool> equal, Func<K, int> hash) { }
                public int Count => default;
                public Sequence<K> Keys => default;
                public Optional<V> Find(K key) => default;
                public bool ContainsKey(K key) => default;
                public bool TryAdd(K key, V value) => default;
                public void Set(K key, V value) { }
                public Iterator<KeyValuePair<K, V>> GetIterator() => default;
            }
            public interface AsyncIterable<out T> { AsyncIterator<T> GetAsyncIterator(); }
            public interface AsyncIterator<out T> : AsyncDisposable {
                Concurrency.Tasks.Task<bool> MoveNextAsync();
                T Current { get; }
            }
            public interface Iterable<out T> { Iterator<T> GetIterator(); }
            public interface Iterator<out T> : Disposable {
                bool MoveNext();
                T Current { get; }
            }
            public interface Collection<out T> : Iterable<T> {
                int Count { get; }
            }
            public interface Sequence<out T> : Collection<T> {
                T this[int index] { get; }
            }
            public interface MutableSequence<T> : Sequence<T> {
                new T this[int index] { get; set; }
            }
            public interface List<T> : MutableSequence<T> {
                T this[Index index] { get; set; }
                T[] this[Range range] { get; }
                void Add(T value);
            }
            public static class SequenceAccess {
                [System.Runtime.CompilerServices.SpecialName]
                public static T get_Item<T>(this Sequence<T> source, Index index) => default;
                [System.Runtime.CompilerServices.SpecialName]
                public static T[] get_Item<T>(this Sequence<T> source, Range range) => default;
                public static Optional<T> Get<T>(this Sequence<T> source, int index) => default;
                public static Optional<T> Get<T>(this Sequence<T> source, Index index) => default;
                public static Result<T[], InvalidRangeError> Slice<T>(this Sequence<T> source, Range range) => default;
            }
            public class ArrayList<T> : List<T> {
                public Result<T[], InvalidRangeError> Slice(Range range) => default;
                public T this[Index index] { get => default; set { } }
                public T[] this[Range range] => default;
                public ArrayList() { }
                public ArrayList(int capacity) { }
                public ArrayList(Iterable<T> items) { }
                public Optional<int> FindIndex(Func<T, bool> match) => default;
                public Optional<int> FindLastIndex(Func<T, bool> match) => default;
                public Optional<T> FindLast(Func<T, bool> match) => default;
                public ArrayList<T> FindAll(Func<T, bool> match) => default;
                public bool TrueForAll(Func<T, bool> match) => default;
                public bool Exists(Func<T, bool> match) => default;
                public Optional<T> Find(Func<T, bool> match) => default;
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
