using Mono.Cecil;

static class CalendarBindings
{
    public static readonly string[] Types = ["System.TimeOffset", "System.Date", "System.Time", "System.LocalDateTime", "System.Instant", "System.Duration"];
    public static bool IsReference(string type) => type is "System.Clock" or "System.SystemClock";
    public const string DateResult = "System.Result<System.Date,System.InvalidDateError>";
    public const string TimeResult = "System.Result<System.Time,System.InvalidTimeError>";
    sealed record Member(string Owner, string Name, string[] Args, string Result, bool Instance = false);
    static readonly Member[] Members = [
        new("TimeOffset", "get_Zero", [], "System.TimeOffset"),
        new("TimeOffset", "get_Seconds", [], "Int32", true),
        new("TimeOffset", "FromSeconds", ["Int32"], "System.Result<System.TimeOffset,System.InvalidTimeError>"),
        new("TimeOffset", "AtInstant", ["System.Instant"], "System.Result<System.LocalDateTime,System.InvalidDateError>", true),
        new("TimeOffset", "ToInstant", ["System.LocalDateTime"], "System.Instant", true),
        new("TimeOffset", "Equals", ["System.TimeOffset"], "Boolean", true),
        new("TimeOffset", "CompareTo", ["System.TimeOffset"], "Int32", true),
        new("Time", "Add", ["System.Duration"], "System.Time", true),
        new("Time", "ToString", [], "String", true),
        new("Time", "ToString", ["System.Globalization.Culture"], "String", true),
        new("LocalDateTime", "Add", ["System.Duration"], "System.Result<System.LocalDateTime,System.InvalidDateError>", true),
        new("Instant", "Add", ["System.Duration"], "System.Result<System.Instant,System.OverflowError>", true),
        new("Instant", "FromUnixTimeTicks", ["Int64"], "System.Instant"),
        new("Instant", "get_UnixTimeTicks", [], "Int64", true),
        new("Instant", "Equals", ["System.Instant"], "Boolean", true),
        new("Instant", "CompareTo", ["System.Instant"], "Int32", true),
        new("Instant", "ToLocalDateTime", [], "System.LocalDateTime", true),
        new("Duration", "FromTicks", ["Int64"], "System.Duration"),
        new("Duration", "get_Ticks", [], "Int64", true),
        new("Duration", "Equals", ["System.Duration"], "Boolean", true),
        new("Duration", "CompareTo", ["System.Duration"], "Int32", true),
        new("Date", "Create", ["Int32", "Int32", "Int32"], DateResult),
        new("Date", "ToString", [], "String", true),
        new("Date", "ToString", ["System.Globalization.Culture"], "String", true),
        new("Date", "AddDays", ["Int32"], DateResult, true),
        new("Date", "AddMonths", ["Int32"], DateResult, true),
        new("Date", "AddYears", ["Int32"], DateResult, true),
        new("LocalDateTime", "Create", ["System.Date", "System.Time"], "System.LocalDateTime"),
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
        new("LocalDateTime", "get_Time", [], "System.Time", true)
    ];
    static string CSharp(string t) => t switch { "Int32" => "int", "Int64" => "long", "Boolean" => "bool", _ => t };
    static string ParameterName(Member member, int index) => member.Name switch
    {
        "Create" when member.Owner == "Date" => new[] { "year", "month", "day" }[index],
        "Create" when member.Owner == "Time" => new[] { "hour", "minute", "second", "fractionTicks" }[index],
        "Create" when member.Owner == "LocalDateTime" => new[] { "date", "time" }[index],
        "ToString" => "culture",
        "AddDays" => "days",
        "AddMonths" => "months",
        "AddYears" => "years",
        "FromDayNumber" => "dayNumber",
        "FromSeconds" => "seconds",
        "AtInstant" => "instant",
        "ToInstant" => "local",
        "Add" => "duration",
        "FromTicks" or "FromUnixTimeTicks" => "ticks",
        "Equals" or "CompareTo" => "other",
        _ => throw new InvalidDataException("Missing calendar parameter name: " + member.Name)
    };
    public static string Declarations => "public interface Clock { Instant Now { get; } } public class SystemClock : Clock { public SystemClock() { } public Instant Now => default; }\n" + string.Join("\n", Members.GroupBy(m => m.Owner).Select(group =>
        $"public struct {group.Key} {{ " + string.Join(" ", group.Select(m =>
            m.Name.StartsWith("get_") ? $"public {(m.Instance ? "" : "static ")}{CSharp(m.Result)} {m.Name[4..]} => default;"
            : $"public {(m.Name == "ToString" && m.Args.Length == 0 ? "override " : m.Instance ? "" : "static ")}{CSharp(m.Result)} {m.Name}({string.Join(',', m.Args.Select((a, i) => CSharp(a) + " " + ParameterName(m, i)))}) => default;")) + " }"));
    public static void ProjectLayout(ModuleDefinition module)
    {
        var attribute = new TypeDefinition("System.Runtime.CompilerServices", "IsReadOnlyAttribute",
            TypeAttributes.Public | TypeAttributes.Sealed, module.GetType("System.Attribute"));
        module.Types.Add(attribute);
        var constructor = new MethodDefinition(".ctor", MethodAttributes.Public | MethodAttributes.SpecialName | MethodAttributes.RTSpecialName,
            module.Types.SelectMany(t => t.Methods).Select(m => m.ReturnType)
                .First(t => t.MetadataType == MetadataType.Void));
        attribute.Methods.Add(constructor);
        constructor.Body.Instructions.Add(Mono.Cecil.Cil.Instruction.Create(Mono.Cecil.Cil.OpCodes.Ret));
        var local = module.GetType("System.LocalDateTime");
        local.PackingSize = -1;
        local.ClassSize = -1;
        local.Fields.Add(new FieldDefinition("StoredDate", FieldAttributes.Private, module.GetType("System.Date")));
        local.Fields.Add(new FieldDefinition("StoredTime", FieldAttributes.Private, module.GetType("System.Time")));
        foreach (var method in local.Methods.Where(m => m.HasThis && !m.IsConstructor))
            method.CustomAttributes.Add(new CustomAttribute(constructor));
        foreach (var (name, field, scalar) in new[] { ("TimeOffset", "StoredSeconds", "Int32"), ("Date", "StoredDayNumber", "Int32"), ("Time", "StoredTicks", "Int64"), ("Instant", "StoredTicks", "Int64"), ("Duration", "StoredTicks", "Int64") })
        {
            var type = module.GetType("System." + name);
            type.PackingSize = -1;
            type.ClassSize = -1;
            type.Fields.Add(new FieldDefinition(field, FieldAttributes.Private,
                module.Types.SelectMany(t => t.Methods)
                    .SelectMany(m => m.Parameters.Select(p => p.ParameterType).Append(m.ReturnType))
                    .First(t => t.FullName == "System." + scalar)));
            foreach (var method in type.Methods.Where(m => m.HasThis && !m.IsConstructor))
                method.CustomAttributes.Add(new CustomAttribute(constructor));
        }
    }
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope) && (type.IsValueType ? Types.Contains(type.FullName) : IsReference(type.FullName)) ? type.FullName : null;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (RuntimeSignatures.IsCore(reference.DeclaringType.Scope) && IsReference(reference.DeclaringType.FullName) && reference.Name == "get_Now")
        {
            var shape = RuntimeSignatures.Match(reference, definition, Type);
            if (!reference.HasThis || shape.Args.Length != 0 || shape.Result != "System.Instant")
                throw new InvalidDataException("Unsupported clock property signature.");
            var owner = reference.DeclaringType.FullName;
            return new(owner + "::get_Now", [owner], shape.Result, Instruction: $"callvirt instance {owner}::get_Now()");
        }
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || !Members.Any(m => "System." + m.Owner == reference.DeclaringType.FullName)) return null;
        var signature = RuntimeSignatures.Match(reference, definition, t => GlobalizationBindings.Type(t) ?? Type(t) ?? GenericUnionBindings.Type(t) ?? ResultBindings.Type(t));
        var member = Members.SingleOrDefault(m => "System." + m.Owner == reference.DeclaringType.FullName && m.Name == reference.Name
            && m.Instance == reference.HasThis && m.Args.SequenceEqual(signature.Args) && m.Result == signature.Result);
        if (member is null || (definition.IsVirtual && !definition.IsFinal && !(member.Owner is "Date" or "Time" && member.Name == "ToString" && member.Args.Length == 0 && !definition.IsNewSlot))) throw new InvalidDataException("Unsupported calendar member: " + reference.FullName);
        return new(member.Instance ? AdapterName(member) : "System." + member.Owner + "::" + member.Name,
            member.Instance ? new[] { "System." + member.Owner + "&" }.Concat(member.Args).ToArray() : member.Args, member.Result);
    }
    static string AdapterName(Member member) => "Runtime" + member.Owner + member.Name + (member.Name == "ToString" && member.Args.Length == 1 ? "Culture" : "");
    public static string Adapters => ".function RuntimeNewSystemClock() -> System.SystemClock\nnewobj instance System.SystemClock::.ctor()\nret\n.end\n" + string.Join("\n", Members.Where(m => m.Instance).Select(m =>
        $".function {AdapterName(m)}(System.{m.Owner}& source{string.Concat(m.Args.Select((a, i) => "," + a + " arg" + i))}) -> {m.Result}\nldarg source\n"
        + string.Concat(m.Args.Select((_, i) => $"ldarg arg{i}\n")) + $"call instance System.{m.Owner}::{m.Name}({string.Join(',', m.Args)})\nret\n.end\n"));
}
