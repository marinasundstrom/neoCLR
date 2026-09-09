interface Counter { int Read(); void Add(int n); }
struct Cell(int n) : Counter {
    public int Value = n;
    public readonly int Read() => Value;
    public void Add(int n) => Value += n;
}
abstract class Base { public abstract int Read(); }
class Derived : Base { public override int Read() => 42; }
class Program {
    static void Add<T>(ref T x, int n) where T : Counter => x.Add(n);
    static int Read<T>(ref T x) where T : Counter => x.Read();
    static int ReadBase<T>(T x) where T : Base => x.Read();
    static Counter ToCounter<T>(T value) where T : Counter => value;
    static Base ToBase<T>(T value) where T : Base => value;
    static void Main() {
        var x = new Cell(40);
        Add(ref x, 2);
        Console.WriteLine(Read(ref x));
        Console.WriteLine(ReadBase(new Derived()));
        try {
            typeof(Program).GetMethod("Add", System.Reflection.BindingFlags.Static | System.Reflection.BindingFlags.NonPublic)!.MakeGenericMethod(typeof(int));
            throw new Exception("constraint not enforced");
        } catch (ArgumentException) { Console.WriteLine("invalid bound rejected"); }
        var boxed = ToCounter(x);
        boxed.Add(1);
        Console.WriteLine($"value={x.Read()}, interface={boxed.Read()}");
        var derived = new Derived();
        Console.WriteLine(ReferenceEquals(derived, ToBase(derived)));
        Console.WriteLine(System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription);
    }
}
