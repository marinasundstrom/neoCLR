using Mono.Cecil;

// One bounded contract drives compiler declarations and runtime binding.
static class TargetSurface
{
    public sealed record Method(string Owner, string Name, string[] Parameters, string Returns)
    {
        public string Signature => $"System.{Owner}::{Name}({string.Join(',', Parameters)})";
        public string ImportTarget => Returns == "Void" ? $"Runtime{Owner}{Name}{string.Join("", Parameters)}" : $"System.{Owner}::{Name}";
    }
    public static readonly Method[] Methods = [
        new("Console", "WriteLine", ["String"], "Void"),
        new("Console", "WriteLine", ["Int32"], "Void"),
        new("Math", "Min", ["Int32", "Int32"], "Int32"),
        new("Math", "Max", ["Int32", "Int32"], "Int32"),
        new("Math", "Sign", ["Int32"], "Int32")
    ];
    public static Method? Bind(MethodDefinition method) => Methods.SingleOrDefault(m =>
        method.DeclaringType.FullName == "System." + m.Owner && method.Name == m.Name
        && method.ReturnType.FullName == "System." + m.Returns
        && method.Parameters.Select(p => p.ParameterType.FullName).SequenceEqual(m.Parameters.Select(p => "System." + p)));
    public static string Declarations(bool includeConsole, bool stringParameter) => string.Join("\n",
        Methods.Where(m => m.Owner != "Console" || includeConsole && (stringParameter || m.Parameters[0] != "String"))
            .GroupBy(m => m.Owner).Select(group => $"public static class {group.Key} {{ " + string.Join(" ", group.Select(m =>
                $"public static {CSharp(m.Returns)} {m.Name}({string.Join(',', m.Parameters.Select((p,i) => CSharp(p) + " value" + i))}) " +
                (m.Returns == "Void" ? "{ }" : "=> 0;"))) + " }"));
    static string CSharp(string type) => type switch { "Int32" => "int", "String" => "string", "Void" => "void", _ => throw new InvalidDataException(type) };
}
