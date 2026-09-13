using Mono.Cecil;

static class CalendarBindings
{
    public static readonly string[] Types = ["System.Date", "System.Time", "System.LocalDateTime"];
    public const string DateResult = "System.Result<System.Date,System.InvalidDateError>";
    public const string TimeResult = "System.Result<System.Time,System.InvalidTimeError>";
    sealed record Member(string Owner, string Name, string[] Args, string Result, bool Instance = false);
    static readonly Member[] Members = [
        new("Date", "Create", ["Int32", "Int32", "Int32"], DateResult),
        new("Date", "FromDayNumber", ["Int32"], DateResult),
        new("Date", "get_DayNumber", [], "Int32", true),
        new("Date", "get_Year", [], "Int32", true),
        new("Date", "get_DayOfYear", [], "Int32", true),
        new("Date", "get_Month", [], "Int32", true),
        new("Date", "get_Day", [], "Int32", true),
        new("Date", "Equals", ["System.Date"], "Boolean", true),
        new("Date", "CompareTo", ["System.Date"], "Int32", true),
        new("Time", "Create", ["Int32", "Int32", "Int32"], TimeResult),
        new("Time", "Create", ["Int32", "Int32", "Int32", "Int32"], TimeResult),
        new("Time", "FromTicks", ["Int64"], TimeResult),
        new("Time", "get_Ticks", [], "Int64", true),
        new("Time", "get_Hour", [], "Int32", true),
        new("Time", "get_Minute", [], "Int32", true),
        new("Time", "get_Second", [], "Int32", true),
        new("Time", "get_Millisecond", [], "Int32", true),
        new("Time", "get_FractionTicks", [], "Int32", true),
        new("Time", "Equals", ["System.Time"], "Boolean", true),
        new("Time", "CompareTo", ["System.Time"], "Int32", true),
        new("LocalDateTime", "get_Date", [], "System.Date", true),
        new("LocalDateTime", "get_Time", [], "System.Time", true),
        new("LocalDateTime", "get_UtcOffsetSeconds", [], "Int32", true),
        new("Clock", "GetLocalNow", [], "System.LocalDateTime")
    ];
    static string CSharp(string t) => t switch { "Int32" => "int", "Int64" => "long", "Boolean" => "bool", _ => t };
    public static string Declarations => string.Join("\n", Members.GroupBy(m => m.Owner).Select(group =>
        $"public {(group.Key == "Clock" ? "static class" : "struct")} {group.Key} {{ " + string.Join(" ", group.Select(m =>
            m.Name.StartsWith("get_") ? $"public {CSharp(m.Result)} {m.Name[4..]} => default;"
            : $"public {(m.Instance ? "" : "static ")}{CSharp(m.Result)} {m.Name}({string.Join(',', m.Args.Select((a, i) => CSharp(a) + " arg" + i))}) => default;")) + " }"));
    public static string? Type(TypeReference type) => type.IsValueType && RuntimeSignatures.IsCore(type.Scope) && Types.Contains(type.FullName) ? type.FullName : null;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || !Members.Any(m => "System." + m.Owner == reference.DeclaringType.FullName)) return null;
        var signature = RuntimeSignatures.Match(reference, definition, t => Type(t) ?? ResultBindings.Type(t));
        var member = Members.SingleOrDefault(m => "System." + m.Owner == reference.DeclaringType.FullName && m.Name == reference.Name
            && m.Instance == reference.HasThis && m.Args.SequenceEqual(signature.Args) && m.Result == signature.Result);
        if (member is null || (definition.IsVirtual && !definition.IsFinal)) throw new InvalidDataException("Unsupported calendar member: " + reference.FullName);
        return new(member.Instance ? "Runtime" + member.Owner + member.Name : "System." + member.Owner + "::" + member.Name,
            member.Instance ? new[] { "System." + member.Owner + "&" }.Concat(member.Args).ToArray() : member.Args, member.Result);
    }
    public static string Adapters => string.Join("\n", Members.Where(m => m.Instance).Select(m =>
        $".function Runtime{m.Owner}{m.Name}(System.{m.Owner}& source{string.Concat(m.Args.Select((a, i) => "," + a + " arg" + i))}) -> {m.Result}\nldarg source\n"
        + string.Concat(m.Args.Select((_, i) => $"ldarg arg{i}\n")) + $"call instance System.{m.Owner}::{m.Name}({string.Join(',', m.Args)})\nret\n.end\n"));
}
