interface Readable<T> { T Read(); }
interface Left<T> : Readable<T> { }
interface Right<T> : Readable<T> { }
interface Counter<T> : Left<T>, Right<T> { }
sealed class Cell : Counter<int> { public int Read() => 42; }
static class Program
{
    static void Main()
    {
        Counter<int> derived = new Cell();
        Readable<int> basis = derived;
        Console.WriteLine(basis.Read());
        Console.WriteLine(ReferenceEquals(derived, basis));
        Console.WriteLine(typeof(Cell).GetInterfaces().Length);
        Console.WriteLine(typeof(Counter<int>).GetInterfaces().Length);
    }
}
