abstract class Counter
{
    public int Value;
    public abstract int Read();
    public virtual int Fallback() => 1;
    public void Add(int amount) => Value += amount;
}
sealed class OffsetCounter : Counter
{
    public int Offset;
    public override int Read() => Value + Offset;
    public override int Fallback() => 42;
    public int BaseFallback() => base.Fallback();
}
static class Program
{
    static void Main()
    {
        var concrete = new OffsetCounter { Value = 39, Offset = 2 };
        Counter view = concrete;
        view.Add(1);
        Console.WriteLine(view.Read());
        Console.WriteLine(concrete.BaseFallback());
        Console.WriteLine(view.Fallback());
        Console.WriteLine(typeof(Counter).IsAbstract);
        Console.WriteLine(typeof(OffsetCounter).GetMethod("Read")!.GetBaseDefinition().DeclaringType == typeof(Counter));
    }
}
