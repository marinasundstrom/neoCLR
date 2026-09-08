using System.Reflection;
class Base<T> { public T Value = default!; }
class Derived : Base<int> { public int Extra = 2; }
static class Program
{
    static void Main()
    {
        var value = new Derived { Value = 40 };
        Console.WriteLine(value.Value + value.Extra);
        Console.WriteLine(typeof(Derived).BaseType == typeof(Base<int>));
        Console.WriteLine(typeof(Derived).GetFields(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly).Length);
        Console.WriteLine(typeof(Base<int>).BaseType == typeof(object));
        Base<int> view = value;
        Console.WriteLine(ReferenceEquals(view, value));
        Console.WriteLine(view.GetType() == typeof(Derived));
    }
}
