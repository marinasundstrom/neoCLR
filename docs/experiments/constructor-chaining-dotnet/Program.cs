abstract class Counter
{
    protected int Value;
    protected Counter(int value)
    {
        Console.WriteLine("base");
        Value = value;
    }
    public abstract int Read();
}
sealed class OffsetCounter : Counter
{
    readonly int offset;
    public OffsetCounter(int value, int offset) : base(value)
    {
        Console.WriteLine("derived");
        this.offset = offset;
    }
    public OffsetCounter(int value) : this(value, 2)
    {
        Console.WriteLine("delegating");
    }
    public override int Read() => Value + offset;
}
static class Program
{
    static void Main()
    {
        Counter counter = new OffsetCounter(40);
        Console.WriteLine(counter.Read());
    }
}
