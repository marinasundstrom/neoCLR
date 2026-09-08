using System.Reflection;
interface Readable { int Read(); }
interface Identified { int Read(); }
class Base : Readable, Identified
{
    int Readable.Read() => 21;
    int Identified.Read() => 1;
    public virtual int Read() => 0;
}
class Child : Base { public override int Read() => 42; }
class Remapped : Child, Readable { }
class Direct : Base, Readable { public override int Read() => 42; }
class Explicit : Child, Readable { int Readable.Read() => 43; }
class Program
{
    static void Main()
    {
        Base value = new Child();
        if (((Readable)value).Read() != 21 || ((Identified)value).Read() != 1 || value.Read() != 42)
            throw new Exception("Mapping and class virtual slots were conflated");
        Console.WriteLine($"Direct override remapping: {((Readable)new Direct()).Read()}; inherited override remapping: {((Readable)new Remapped()).Read()}");
        if (((Readable)new Remapped()).Read() != 21 || ((Readable)new Explicit()).Read() != 43)
            throw new Exception($"Reimplementation failed: {((Readable)new Remapped()).Read()}, {((Readable)new Explicit()).Read()}");
        var body = typeof(Base).GetInterfaceMap(typeof(Readable)).TargetMethods.Single();
        if (!body.IsPrivate || !body.IsVirtual || !body.IsFinal || body.Name != "Readable.Read")
            throw new Exception("Unexpected explicit body metadata");
        if (typeof(Base).GetMethods(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly).Length != 1)
            throw new Exception("Explicit bodies leaked into public surface");
        Console.WriteLine("Distinct mappings, inheritance, reimplementation and private/virtual/final metadata passed.");
    }
}
