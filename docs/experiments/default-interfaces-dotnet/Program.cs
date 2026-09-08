using System.Reflection;
interface Root { int Read() => 1; }
interface Left : Root { int Root.Read() => 2; }
interface Right : Root { int Root.Read() => 3; }
interface Resolved : Left, Right { int Root.Read() => 42; }
interface Required : Root { abstract int Root.Read(); }
class DefaultOnly : Root { }
class MostSpecific : Left { }
class DiamondResolved : Resolved { }
class ClassWins : Left, Right { public int Read() => 42; }
class ExplicitWins : Left, Right { int Root.Read() => 42; }
class ReabstractImplemented : Required { public int Read() => 42; }
class InheritsDefault : DefaultOnly { public int Read() => 42; }
class RemapsDefault : DefaultOnly, Root { public int Read() => 42; }
interface Counter
{
    ref int Address();
    int Increment()
    {
        ref int value = ref Address();
        return ++value;
    }
    Counter Same() => this;
}
class MutableCounter : Counter
{
    public int Value = 41;
    ref int Counter.Address() => ref Value;
}
#if AMBIGUOUS
class Ambiguous : Left, Right { }
#endif
#if REABSTRACT
class MissingRequired : Required { }
#endif
class Program
{
    static void Main()
    {
#if CLASSACCESS
        _ = new DefaultOnly().Read();
#endif
        Check(((Root)new DefaultOnly()).Read(), 1, "fallback");
        Check(((Root)new MostSpecific()).Read(), 2, "most-specific");
        Check(((Root)new DiamondResolved()).Read(), 42, "resolved diamond");
        Check(((Root)new ClassWins()).Read(), 42, "public class precedence");
        Check(((Root)new ExplicitWins()).Read(), 42, "explicit class precedence");
        Check(((Root)new ReabstractImplemented()).Read(), 42, "reabstraction satisfied");
        Check(((Root)new InheritsDefault()).Read(), 1, "inherited mapping stays anchored");
        Check(((Root)new RemapsDefault()).Read(), 42, "redeclared conformance");
        var value = new MutableCounter();
        Counter view = value;
        Check(view.Increment(), 42, "default calls explicit contract");
        Check(value.Value, 42, "original storage mutation");
        if (!ReferenceEquals(value, view.Same())) throw new Exception("Receiver identity changed");
        var mapping = typeof(DefaultOnly).GetInterfaceMap(typeof(Root)).TargetMethods.Single();
        if (mapping.DeclaringType != typeof(Root) || mapping.IsAbstract || !mapping.IsVirtual)
            throw new Exception("Unexpected default body metadata");
        if (typeof(DefaultOnly).GetMethods(BindingFlags.DeclaredOnly | BindingFlags.Instance | BindingFlags.Public).Length != 0)
            throw new Exception("Default became a class declaration");
        Console.WriteLine($"Runtime {Environment.Version}: default selection, precedence, mutation, identity and metadata passed.");
    }
    static void Check(int actual, int expected, string scenario)
    {
        if (actual != expected) throw new Exception($"{scenario}: expected {expected}, got {actual}");
    }
}
