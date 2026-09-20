# NeoCLR Collections API — Consolidated Proposal

The NeoCLR collections API should make a clean distinction between **collection contracts** and **concrete data structures**.

The goal isn't abstraction for abstraction's sake. It is to make the type system communicate what a data structure actually is, what operations are available through a particular contract, and what an API intends its consumers to rely upon.

The central principles are:

> **Interfaces describe shape and capabilities. Concrete types describe data structures and behavioral guarantees.**

> **Choose data structures explicitly. Expose the contract that consumers are intended to rely upon.**

This gives NeoCLR the explicit data-structure vocabulary associated with Java while going further in separating traversal, shape, mutation, representation, and immutability—and doing so without producing a complicated API.

---

## 1. Why change the .NET model?

.NET has accumulated several overlapping concepts:

```text
IEnumerable<T>
ICollection<T>
IReadOnlyCollection<T>

IList<T>
IReadOnlyList<T>
List<T>

ImmutableList<T>
FrozenSet<T>
...
```

There are historical reasons for this, but the resulting model obscures several distinctions.

Most notably, `.NET List<T>` sounds like the abstract notion of a list but is actually a **resizable array-backed implementation**.

The abstract contract is `IList<T>`, which also implies mutation. `IReadOnlyList<T>` was later introduced to provide a restricted view, but “read-only” does not mean immutable.

NeoCLR can start with these distinctions designed into the platform.

---

# 2. Contracts versus data structures

The first major separation is:

```text
CONTRACT                         DATA STRUCTURE

List<T>                          ArrayList<T>
Set<T>                           HashSet<T>
Map<K, V>                        HashMap<K, V>

"What can I rely on?"            "What did I choose?"
```

For example:

```raven
let users = ArrayList<User>()
```

is deliberately explicit.

The programmer has chosen a **resizable array-backed list**.

But an API consuming those users might declare:

```raven
func Process(users: List<User>)
```

because it doesn't care how the list is stored.

Or perhaps:

```raven
func Process(users: Sequence<User>)
```

because it only needs ordered traversal.

Or:

```raven
func Process(users: Iterable<User>)
```

because it merely enumerates them.

The implementation is explicit where it matters. The implementation disappears behind a contract where it doesn't.

---

# 3. `Iterable<T>`

The most fundamental capability is iteration.

```raven
interface Iterable<T>
{
    func GetIterator() -> Iterator<T>
}
```

with:

```raven
interface Iterator<T>
{
    Current: T

    func MoveNext() -> bool
}
```

The exact iterator protocol can still evolve, but the conceptual distinction is important.

`Iterable<T>` promises only that values can be traversed.

It does not promise:

- known size;
- materialization;
- ordering semantics;
- indexed access;
- uniqueness;
- mutability;
- repeated enumeration;
- particular storage.

An algorithm should accept `Iterable<T>` when iteration is genuinely all it needs.

```raven
func Sum(values: Iterable<int>) -> int
```

---

# 4. `Sequence<T>`

`Sequence<T>` represents values with meaningful sequence semantics.

```raven
interface Sequence<T> : Iterable<T>
{
}
```

It may initially be largely a semantic interface.

A sequence is ordered, but it need not be:

- materialized;
- indexed;
- mutable;
- countable without enumeration.

This makes it appropriate for lazy transformations and generators.

```raven
func StartsWith<T>(
    values: Sequence<T>,
    prefix: Sequence<T>
) -> bool
```

The algorithm cares about order but not storage.

---

# 5. `Collection<T>`

`Collection<T>` represents a finite materialized collection with a known number of elements.

```raven
interface Collection<T> : Iterable<T>
{
    Count: int
}
```

It does not necessarily imply order.

For example, both:

```text
List<T>
Set<T>
```

can be collections.

An API requiring only enumeration and a known count should therefore say:

```raven
func Validate(items: Collection<Item>)
```

rather than unnecessarily requiring a list.

---

# 6. `List<T>`

`List<T>` describes the **list abstraction**, not a particular implementation.

```raven
interface List<T> :
    Sequence<T>,
    Collection<T>
{
    this[int index]: T
}
```

It describes a collection that is:

- finite;
- ordered;
- indexed.

Crucially:

> **`List<T>` does not imply mutability.**

The following can therefore all be lists:

```text
Array<T>
ArrayList<T>
ImmutableList<T>
FrozenList<T>
```

They share list semantics while differing substantially in representation and behavior.

---

# 7. Mutation is an additional capability

Mutation should not be bundled into `List<T>` merely because mutable lists are common.

NeoCLR makes it explicit.

At the collection level:

```raven
interface MutableCollection<T> : Collection<T>
{
    func Add(value: T)
    func Remove(value: T) -> bool
    func Clear()
}
```

For structurally mutable lists:

```raven
interface MutableList<T> :
    List<T>,
    MutableCollection<T>
{
    this[int index]: T { get; set }

    func Insert(index: int, value: T)
    func RemoveAt(index: int)
}
```

This creates intentionally different API contracts:

```raven
func Process(items: Iterable<Item>)
```

means:

> I need to enumerate these values.

```raven
func Process(items: Collection<Item>)
```

means:

> I need a finite materialized collection and its size.

```raven
func Process(items: List<Item>)
```

means:

> I need ordered indexed access.

```raven
func Process(items: MutableList<Item>)
```

means:

> I need list semantics and permission to modify its structure.

`MutableList` being more cumbersome to write is acceptable—and arguably desirable. The API is asking for considerably more capability.

---

# 8. Element mutation and structural mutation

One important distinction emerges from arrays.

An array can usually do this:

```raven
values[3] = replacement
```

but not:

```raven
values.Add(replacement)
values.RemoveAt(3)
```

Therefore **writable elements and mutable structure are separate capabilities**.

A contract such as this is likely useful:

```raven
interface MutableElements<T> : List<T>
{
    this[int index]: T { get; set }
}
```

Then:

```text
Array<T>
    List<T>
    MutableElements<T>

ArrayList<T>
    List<T>
    MutableElements<T>
    MutableList<T>

ImmutableList<T>
    List<T>

FrozenList<T>
    List<T>
```

This isn't merely theoretical.

An in-place sorting algorithm needs writable elements:

```raven
func Sort<T>(values: MutableElements<T>)
```

It doesn't require the ability to resize the collection.

Consequently both `Array<T>` and `ArrayList<T>` can participate.

An append operation genuinely needs structural mutation:

```raven
func Append<T>(
    values: MutableList<T>,
    value: T
)
```

An array cannot satisfy that contract.

This is exactly the kind of distinction the capability model should express.

---

# 9. `Array<T>` is a real generic type

NeoCLR has already corrected an important historical CLR design issue.

Arrays are represented by a generic type:

```raven
Array<T>
```

rather than having the peculiar non-generic runtime hierarchy inherited by .NET.

`Array<T>` is a first-class generic collection type.

Its characteristics are explicit:

```text
Array<T>

generic
invariant
fixed-size
contiguous
ordered
indexed
elements replaceable
structure not resizable
```

Conceptually:

```raven
class Array<T> :
    List<T>,
    MutableElements<T>
{
    Count: int

    this[int index]: T { get; set }
}
```

---

# 10. Arrays are invariant

NeoCLR also removes array covariance.

Given:

```raven
class Animal {}
class Dog : Animal {}
class Cat : Animal {}
```

this is invalid:

```raven
let dogs: Array<Dog> = ...

let animals: Array<Animal> = dogs // error
```

even though `Dog` derives from `Animal`.

Otherwise this could become possible:

```raven
animals[0] = Cat()
```

while the underlying storage is actually `Array<Dog>`.

.NET permits this relationship and consequently needs runtime array-store checks.

NeoCLR doesn't.

Likewise, the opposite conversion isn't allowed:

```raven
Array<Animal> -> Array<Dog>
```

`Array<T>` is invariant.

The relationship between element types does not manufacture a relationship between mutable storage types.

---

# 11. `ArrayList<T>`

`ArrayList<T>` is the standard general-purpose resizable array-backed list.

```raven
class ArrayList<T> :
    MutableList<T>,
    MutableElements<T>
{
    ...
}
```

It provides:

- contiguous backing storage;
- indexed access;
- resizing;
- append;
- insertion;
- removal;
- element replacement.

This corresponds roughly to the actual data structure represented by `.NET List<T>`.

NeoCLR simply names it for what it is:

```raven
let users = ArrayList<User>()
```

There is educational value in this explicitness.

The developer understands that choosing `ArrayList<T>` means choosing an array-backed list with its corresponding algorithmic characteristics.

---

# 12. Concrete collection types are not forbidden in APIs

NeoCLR should **not** turn “program against interfaces” into a rule that developers mechanically follow.

This is perfectly valid:

```raven
func Process(items: ArrayList<Item>)
```

It simply communicates a stronger contract:

> The consumer may rely specifically on this being an `ArrayList<Item>`.

Sometimes that's exactly what an API wants.

The difference is that NeoCLR makes the choice explicit.

Compare:

```raven
ArrayList<Item>
MutableList<Item>
List<Item>
Sequence<Item>
Iterable<Item>
```

Each communicates a different boundary.

The question isn't:

> Interface or concrete type?

The question is:

> **What do I intend the consumer to know and rely upon?**

---

# 13. API boundaries therefore become deliberate

Parameter types describe requirements.

For example:

```raven
func Calculate(items: Iterable<Item>)
```

asks very little from callers.

Whereas:

```raven
func Calculate(items: List<Item>)
```

requires considerably more.

Return types work in the opposite direction: they determine what the API promises to consumers.

Consider:

```raven
func GetUsers() -> ArrayList<User>
```

The API exposes the concrete representation and its capabilities.

```raven
func GetUsers() -> MutableList<User>
```

The representation is hidden, but structural mutation is explicitly permitted.

```raven
func GetUsers() -> List<User>
```

The consumer receives list semantics but no mutation capability through that contract.

```raven
func GetUsers() -> Sequence<User>
```

The API promises only sequence semantics.

This makes the signature an intentional visibility boundary.

---

# 14. No parallel `ReadOnly` hierarchy

NeoCLR should not recreate:

```text
ReadOnlyCollection<T>
ReadOnlyList<T>
```

as fundamental interfaces.

There is an important difference between **not exposing mutation** and **being immutable**.

For example:

```raven
let storage = ArrayList<User>()

let users: List<User> = storage
```

`users` does not provide mutation operations.

But this remains possible:

```raven
storage.Add(newUser)
```

and `users` can observe the resulting change.

Therefore `List<T>` does not mean:

> This object cannot change.

It means:

> Mutation isn't part of this contract.

That's a much cleaner statement.

---

# 15. `ImmutableList<T>`

Actual immutability is a property of a concrete collection implementation.

```raven
let users = ImmutableList<User>(...)
```

The collection itself cannot be mutated.

However, immutable collections **do support modification operations**. Those operations produce new values.

```raven
let a = ImmutableList(1, 2, 3)

let b = a.Add(4)
let c = b.Remove(2)
let d = c.Set(0, 10)
```

Semantically:

```text
a = [1, 2, 3]
b = [1, 2, 3, 4]
c = [1, 3, 4]
d = [10, 3, 4]
```

`a`, `b`, and `c` remain unchanged when later versions are produced.

The concrete API could resemble:

```raven
class ImmutableList<T> : List<T>
{
    func Add(value: T) -> ImmutableList<T>

    func Insert(
        index: int,
        value: T
    ) -> ImmutableList<T>

    func Remove(value: T) -> ImmutableList<T>

    func RemoveAt(
        index: int
    ) -> ImmutableList<T>

    func Set(
        index: int,
        value: T
    ) -> ImmutableList<T>

    func Clear() -> ImmutableList<T>
}
```

---

# 16. Persistent collections use structural sharing

The previous operations behave *semantically* like copying the collection and applying a change.

That does not mean implementations must physically copy everything.

For example:

```raven
let updated = original.Add(value)
```

may structurally share most of its internal representation with `original`.

The observable guarantee is:

> `original` never changes.

The internal implementation is free to use persistent data structures to achieve efficient updates.

---

# 17. Mutable and immutable `Add` are different concepts

NeoCLR should not artificially unify these operations.

For a mutable collection:

```raven
items.Add(value)
```

means:

> Modify this collection.

For an immutable collection:

```raven
let updated = items.Add(value)
```

means:

> Return another immutable collection containing the change.

They can use the same natural verb while remaining separate contracts.

There is no need to invent an abstraction simply because the operation happens to be called `Add`.

---

# 18. `FrozenList<T>`

Frozen collections have another purpose.

```raven
FrozenList<T>
```

is immutable, but unlike `ImmutableList<T>`, it isn't primarily designed for producing modified versions.

Its lifecycle is:

```text
build
  ↓
freeze
  ↓
read
read
read
read
```

That allows the implementation to optimize its final representation for reads.

The distinction is:

| Type | In-place mutation | Persistent changes | Primary goal |
|---|---|---|---|
| `ArrayList<T>` | Yes | No | General mutable list |
| `ImmutableList<T>` | No | Yes | Persistent immutable values |
| `FrozenList<T>` | No | No | Finalized read-optimized value |

Both `ImmutableList<T>` and `FrozenList<T>` can satisfy:

```raven
List<T>
```

Their stronger guarantees come from their concrete types.

---

# 19. Sets follow the same model

The abstract set contract is:

```raven
interface Set<T> : Collection<T>
{
    func Contains(value: T) -> bool
}
```

Mutation is separate:

```raven
interface MutableSet<T> :
    Set<T>,
    MutableCollection<T>
{
}
```

Concrete data structures can include:

```text
HashSet<T>
TreeSet<T>
```

and specialized semantic implementations:

```text
ImmutableSet<T>
FrozenSet<T>
```

The names reveal what was selected.

```raven
let ids = HashSet<UserId>()
```

says something useful about the chosen data structure.

But:

```raven
func Authorize(ids: Set<UserId>)
```

says that the implementation isn't relevant to the consumer.

---

# 20. Maps follow the same model

The abstract key/value contract should be called `Map`.

```raven
interface Map<TKey, TValue> :
    Collection<KeyValuePair<TKey, TValue>>
{
    func Get(key: TKey) -> Option<TValue>

    func ContainsKey(key: TKey) -> bool
}
```

This works naturally with NeoCLR's `Option` model:

```raven
let user = users.Get(id)?
```

Mutation is explicit:

```raven
interface MutableMap<TKey, TValue> :
    Map<TKey, TValue>
{
    func Set(
        key: TKey,
        value: TValue
    )

    func Remove(key: TKey) -> bool

    func Clear()
}
```

Concrete implementations can then include:

```text
HashMap<K, V>
TreeMap<K, V>

ImmutableMap<K, V>
FrozenMap<K, V>
```

Again:

```raven
let users = HashMap<UserId, User>()
```

makes a data-structure choice explicit.

Whereas:

```raven
func FindUser(
    users: Map<UserId, User>,
    id: UserId
) -> Option<User>
```

depends only upon map semantics.

---

# 21. This is a capability graph, not one giant hierarchy

The collection model should not be forced into:

```text
Iterable
   ↓
Sequence
   ↓
Collection
   ↓
List
```

The concepts overlap.

Conceptually:

```text
                    Iterable<T>
                    /         \
                   /           \
          Sequence<T>       Collection<T>
                   \           /
                    \         /
                      List<T>
```

A lazy ordered pipeline can be:

```text
Iterable<T>
Sequence<T>
```

without being a `Collection<T>`.

A set can be:

```text
Iterable<T>
Collection<T>
Set<T>
```

without being a `Sequence<T>`.

A list combines both collection and sequence semantics.

Additional capabilities then intersect with those shapes:

```text
                         List<T>
                       /         \
                      /           \
          MutableElements<T>   MutableList<T>
```

Concrete implementations select whichever contracts accurately describe them.

---

# 22. Algorithms depend on minimum capabilities

This model should influence the standard collection algorithms.

For example:

```raven
func First<T>(
    values: Iterable<T>
) -> Option<T>
```

requires only iteration.

```raven
func StartsWith<T>(
    values: Sequence<T>,
    prefix: Sequence<T>
) -> bool
```

requires ordering.

```raven
func BinarySearch<T>(
    values: List<T>,
    value: T
) -> Option<int>
```

requires positional access.

```raven
func Sort<T>(
    values: MutableElements<T>
)
```

requires indexed replacement but not structural resizing.

```raven
func Append<T>(
    values: MutableList<T>,
    value: T
)
```

requires structural mutation.

The type signature therefore explains something