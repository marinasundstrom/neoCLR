using System.Reflection;
if (Helpers.Identity(42) != 42 || Box<string>.First<int>("owner", 42) != "owner")
    throw new Exception("Generic method result mismatch");
var method = typeof(Box<>).GetMethod(nameof(Box<string>.First))!;
if (!method.IsGenericMethodDefinition || method.GetGenericArguments().Length != 1 ||
    method.GetParameters()[0].ParameterType.DeclaringMethod != null ||
    method.GetParameters()[1].ParameterType.DeclaringMethod == null)
    throw new Exception("Owner and method parameters must be independent");
Console.WriteLine("Independent owner/method parameters; explicit and inferred calls passed.");
static class Helpers { public static T Identity<T>(T value) => value; }
class Box<T> { public static T First<U>(T first, U second) => first; }
