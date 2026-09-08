using System.Runtime.CompilerServices;

Probe.Run();
static class Probe
{
    public static void Run()
    {
        Func<int, int> identity = Identity<int>;
        Check(identity(42) == 42 && identity.Target is null, "static and closed generic target");
        var owner = new Counter { Value = 1 };
        Func<int> shared = owner.Next;
        owner.Value = 40;
        Check(shared() == 41 && owner.Value == 41 && ReferenceEquals(shared.Target, owner), "shared class receiver");
        var value = new ValueCounter { Value = 1 };
        Func<int> copied = value.Next;
        value.Value = 40;
        Check(copied() == 2 && value.Value == 40 && copied.Target is ValueCounter, "boxed value receiver copy");
        Base view = new Derived();
        Func<int> dispatched = view.Read;
        Check(dispatched() == 42 && dispatched.Method.DeclaringType == typeof(Derived), "virtual implementation selected at binding");
        Func<int> same = owner.Next;
        Check(shared.Equals(same), "same nominal delegate target/method equality");
        OtherReader other = owner.Next;
        Check(!shared.Equals(other), "distinct nominal delegate types differ");
        Check(owner.PrivateCallback()() == 41, "binding transfers private invocation capability");
        OutSetter setter = Set;
        setter(out var output);
        ReadOnlyReader reader = Read;
        Check(output == 42 && reader(in output) == 42, "ref contracts");
        var order = new List<int>();
        Func<int> first = () => { order.Add(1); return 1; };
        Func<int> last = () => { order.Add(2); return 42; };
        var combined = first + last;
        Check(combined() == 42 && order.SequenceEqual(new[] { 1, 2 }), "multicast order and last result");
        order.Clear();
        Func<int> throws = () => throw new InvalidOperationException("stop");
        try { (first + throws + last)(); throw new Exception("expected exception"); }
        catch (InvalidOperationException) { }
        Check(order.SequenceEqual(new[] { 1 }), "multicast stops on exception");
        var weak = ExerciseRetainedTarget();
        Collect();
        Check(!weak.IsAlive, "released delegate releases target root");
        Console.WriteLine("Delegate binding, identity, receiver copy, ref contracts, multicast and GC probes passed.");
    }
    static T Identity<T>(T value) => value;
    static void Set(out int value) { value = 42; }
    static int Read(in int value) => value;
    static void Check(bool result, string scenario) { if (!result) throw new Exception(scenario); }
    static void Collect() { GC.Collect(); GC.WaitForPendingFinalizers(); GC.Collect(); }
    [MethodImpl(MethodImplOptions.NoInlining)]
    static (Func<int>, WeakReference) MakeCallback()
    {
        var owner = new Counter { Value = 41 };
        return (owner.Next, new WeakReference(owner));
    }
    [MethodImpl(MethodImplOptions.NoInlining)]
    static WeakReference ExerciseRetainedTarget()
    {
        var (callback, weak) = MakeCallback();
        Collect();
        Check(weak.IsAlive && callback() == 42, "delegate retains receiver through GC");
        GC.KeepAlive(callback);
        return weak;
    }
#if BAD_REF_CONTRACT
    static Action<int> InvalidContract = Set;
#endif
#if BAD_REF_CAPTURE
    static Func<int> Capture(ref int value) => () => value;
#endif
}
delegate int OtherReader();
delegate void OutSetter(out int value);
delegate int ReadOnlyReader(in int value);
class Counter
{
    public int Value;
    public int Next() => ++Value;
    private int ReadPrivate() => Value;
    public Func<int> PrivateCallback() => ReadPrivate;
}
struct ValueCounter { public int Value; public int Next() => ++Value; }
class Base { public virtual int Read() => 1; }
class Derived : Base { public override int Read() => 42; }
