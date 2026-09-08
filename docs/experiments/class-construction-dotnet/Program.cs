_ = new Derived();
var expected = new[] { "derived field", "base field", "base body", "derived body" };
if (!Log.Events.SequenceEqual(expected)) throw new Exception("Unexpected C# construction order");
if (new ValueCounter().Value != 42 || default(ValueCounter).Value != 0 || default(Derived) != null)
    throw new Exception("Unexpected default semantics");
Console.WriteLine(string.Join(", ", Log.Events));
Console.WriteLine("Constructed value: 42; default value: 0; reference default: null.");
static class Log {
    public static List<string> Events = new();
    public static int Trace(string name) { Events.Add(name); return 1; }
}
class Base {
    public int Value = Log.Trace("base field");
    public Base() { Log.Trace("base body"); }
}
class Derived : Base {
    public int Extra = Log.Trace("derived field");
    public Derived() { Log.Trace("derived body"); }
}
struct ValueCounter {
    public int Value;
    public ValueCounter() { Value = 42; }
}
