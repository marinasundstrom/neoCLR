interface Readable { int Read(); }
interface Counter : Readable { void Add(int amount); }
abstract class CounterBase(int value) : Counter
{
    protected int Value = value;
    public abstract int Read();
    public void Add(int amount) => Value += amount;
}
class OffsetCounter(int value, int offset) : CounterBase(value)
{
    public override int Read() => Value + offset;
}
class PublicBody { public int Read() => 42; }
class NewMapping : PublicBody, Readable { }
class RepeatedMapping(int value, int offset) : OffsetCounter(value, offset), Counter { }
class Program
{
    static void Main()
    {
        CounterBase original = new OffsetCounter(19, 1);
        Counter counter = original;
        counter.Add(1);
        Readable read = original;
        if (counter.Read() + read.Read() != 42 || !ReferenceEquals(counter, read))
            throw new Exception("Inherited virtual mapping/identity failed");
        if (typeof(OffsetCounter).GetInterfaces().Length != 2)
            throw new Exception("Inherited interface enumeration failed");
        if (((Readable)new NewMapping()).Read() != 42 ||
            ((Counter)new RepeatedMapping(40, 2)).Read() != 42)
            throw new Exception("Inherited public mapping failed");
        Console.WriteLine("Inherited mappings, override dispatch, identity and enumeration passed.");
    }
}
