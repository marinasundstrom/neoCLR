using System.Reflection;

class Parent
{
    public int Inherited => 1;
}
class Model : Parent
{
    public int Age = 42;
    public int Read() => Age;
    public int Count => 1;
}
static class Program
{
    static void Main()
    {
        MemberInfo[] members = [
            typeof(Model).GetField(nameof(Model.Age))!,
            typeof(Model).GetMethod(nameof(Model.Read))!,
            typeof(Model).GetProperty(nameof(Model.Count))!
        ];
        foreach (MemberInfo member in members)
            Console.WriteLine($"{member.Name}:{member.DeclaringType!.Name}");
        Console.WriteLine(typeof(MethodInfo).BaseType == typeof(MethodBase));
        Console.WriteLine(typeof(MethodBase).BaseType == typeof(MemberInfo));
        Console.WriteLine(typeof(Model).GetProperties().Length);
        Console.WriteLine(typeof(Model).GetProperties(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly).Length);
    }
}
