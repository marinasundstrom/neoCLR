using System.Runtime.CompilerServices;

var retained = CheckInterning();
GC.Collect();
GC.WaitForPendingFinalizers();
GC.Collect();
if (!retained.IsAlive)
    throw new Exception("Interned string was not retained by the CLR pool");
Console.WriteLine(".NET: equal dynamic strings share the returned interned reference; existing references stay distinct.");
Console.WriteLine(".NET: the intern pool retains text after local references are released and GC runs.");

[MethodImpl(MethodImplOptions.NoInlining)]
static WeakReference CheckInterning()
{
    string first = "neoclr-intern-probe-" + Guid.NewGuid().ToString("N");
    string second = new string(first.ToCharArray());
    if (ReferenceEquals(first, second) || String.IsInterned(first) is not null)
        throw new Exception("Probe input unexpectedly shared or interned");
    string canonical = String.Intern(first);
    string repeated = String.Intern(second);
    if (!ReferenceEquals(canonical, repeated) || ReferenceEquals(first, second)
        || first != repeated || !ReferenceEquals(String.IsInterned(second), canonical)
        || RuntimeHelpers.GetHashCode(canonical) != RuntimeHelpers.GetHashCode(repeated))
        throw new Exception("Interning changed the identity/content contract");
    return new WeakReference(canonical);
}
