using System.Collections.Generic;

var dogs = new[] { new Dog(7) };
IReadOnlyList<Animal> view = Array.AsReadOnly(dogs);
ReadFirst(view);
dogs[0] = new Dog(42);
ReadFirst(view);
view[0].Age = 99;
Console.WriteLine(dogs[0].Age);
Console.WriteLine(view is Dog[]);
// A bare read-only interface over the array does not prevent an exact back-cast.
IReadOnlyList<Animal> bareView = dogs;
Console.WriteLine(ReferenceEquals(dogs, (Dog[])bareView));
Console.WriteLine(typeof(IReadOnlyList<>).GetGenericArguments()[0].GenericParameterAttributes);

static void ReadFirst(IReadOnlyList<Animal> items)
{
    Console.WriteLine(items.Count);
    Console.WriteLine(items[0].Age);
}

class Animal(int age)
{
    public int Age { get; set; } = age;
}
class Dog(int age) : Animal(age);
