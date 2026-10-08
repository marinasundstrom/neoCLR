using Mono.Cecil;

static class CalendarBindings
{
    public static readonly string[] Types = ["System.Time.TimeOffset", "System.Time.Date", "System.Time.TimeOfDay", "System.Time.LocalDateTime", "System.Time.Instant", "System.Time.Duration"];
    public static bool IsReference(string type) => type is "System.Time.Clock" or "System.Time.SystemClock";
    public const string DateResult = "System.Result<System.Time.Date,System.Time.InvalidDateError>";
    public const string TimeResult = "System.Result<System.Time.TimeOfDay,System.Time.InvalidTimeError>";
    sealed record Member(string Owner, string Name, string[] Args, string Result, bool Instance = false);
    static readonly Member[] Members = [
        new("TimeOffset", "get_Zero", [], "System.Time.TimeOffset"),
        new("TimeOffset", "get_Seconds", [], "Int32", true),
        new("TimeOffset", "FromSeconds", ["Int32"], "System.Result<System.Time.TimeOffset,System.Time.InvalidTimeError>"),
        new("TimeOffset", "AtInstant", ["System.Time.Instant"], "System.Result<System.Time.LocalDateTime,System.Time.InvalidDateError>", true),
        new("TimeOffset", "ToInstant", ["System.Time.LocalDateTime"], "System.Time.Instant", true),
        new("TimeOffset", "Equals", ["System.Time.TimeOffset"], "Boolean", true),
        new("TimeOffset", "CompareTo", ["System.Time.TimeOffset"], "Int32", true),
        new("TimeOfDay", "Add", ["System.Time.Duration"], "System.Time.TimeOfDay", true),
        new("TimeOfDay", "ToString", [], "String", true),
        new("TimeOfDay", "ToString", ["System.Globalization.Culture"], "String", true),
        new("LocalDateTime", "Add", ["System.Time.Duration"], "System.Result<System.Time.LocalDateTime,System.Time.InvalidDateError>", true),
        new("Instant", "Add", ["System.Time.Duration"], "System.Result<System.Time.Instant,System.OverflowError>", true),
        new("Instant", "FromUnixTimeTicks", ["Int64"], "System.Time.Instant"),
        new("Instant", "get_UnixTimeTicks", [], "Int64", true),
        new("Instant", "Equals", ["System.Time.Instant"], "Boolean", true),
        new("Instant", "CompareTo", ["System.Time.Instant"], "Int32", true),
        new("Instant", "ToLocalDateTime", [], "System.Time.LocalDateTime", true),
        new("Duration", "FromTicks", ["Int64"], "System.Time.Duration"),
        new("Duration", "get_Ticks", [], "Int64", true),
        new("Duration", "Equals", ["System.Time.Duration"], "Boolean", true),
        new("Duration", "CompareTo", ["System.Time.Duration"], "Int32", true),
        new("Date", "Create", ["Int32", "Int32", "Int32"], DateResult),
        new("Date", "ToString", [], "String", true),
        new("Date", "ToString", ["System.Globalization.Culture"], "String", true),
        new("Date", "AddDays", ["Int32"], DateResult, true),
        new("Date", "AddMonths", ["Int32"], DateResult, true),
        new("Date", "AddYears", ["Int32"], DateResult, true),
        new("LocalDateTime", "Create", ["System.Time.Date", "System.Time.TimeOfDay"], "System.Time.LocalDateTime"),
        new("Date", "FromDayNumber", ["Int32"], DateResult),
        new("Date", "get_DayNumber", [], "Int32", true),
        new("Date", "get_Year", [], "Int32", true),
        new("Date", "get_DayOfYear", [], "Int32", true),
        new("Date", "get_Month", [], "Int32", true),
        new("Date", "get_Day", [], "Int32", true),
        new("Date", "Equals", ["System.Time.Date"], "Boolean", true),
        new("Date", "CompareTo", ["System.Time.Date"], "Int32", true),
        new("TimeOfDay", "Create", ["Int32", "Int32", "Int32"], TimeResult),
        new("TimeOfDay", "Create", ["Int32", "Int32", "Int32", "Int32"], TimeResult),
        new("TimeOfDay", "FromTicks", ["Int64"], TimeResult),
        new("TimeOfDay", "get_Ticks", [], "Int64", true),
        new("TimeOfDay", "get_Hour", [], "Int32", true),
        new("TimeOfDay", "get_Minute", [], "Int32", true),
        new("TimeOfDay", "get_Second", [], "Int32", true),
        new("TimeOfDay", "get_Millisecond", [], "Int32", true),
        new("TimeOfDay", "get_FractionTicks", [], "Int32", true),
        new("TimeOfDay", "Equals", ["System.Time.TimeOfDay"], "Boolean", true),
        new("TimeOfDay", "CompareTo", ["System.Time.TimeOfDay"], "Int32", true),
        new("LocalDateTime", "get_Date", [], "System.Time.Date", true),
        new("LocalDateTime", "get_Time", [], "System.Time.TimeOfDay", true)
    ];
    static string CSharp(string t) => t switch { "Int32" => "int", "Int64" => "long", "Boolean" => "bool", _ => t };
    static string ParameterName(Member member, int index) => member.Name switch
    {
        "Create" when member.Owner == "Date" => new[] { "year", "month", "day" }[index],
        "Create" when member.Owner == "TimeOfDay" => new[] { "hour", "minute", "second", "fractionTicks" }[index],
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
    public static string Declarations => "namespace Time { public interface Clock { Instant Now { get; } } public class SystemClock : Clock { public SystemClock() { } public Instant Now => default; }\n" + string.Join("\n", Members.GroupBy(m => m.Owner).Select(group =>
        $"public struct {group.Key} {{ " + string.Join(" ", group.Select(m =>
            m.Name.StartsWith("get_") ? $"public {(m.Instance ? "" : "static ")}{CSharp(m.Result)} {m.Name[4..]} => default;"
            : $"public {(m.Name == "ToString" && m.Args.Length == 0 ? "override " : m.Instance ? "" : "static ")}{CSharp(m.Result)} {m.Name}({string.Join(',', m.Args.Select((a, i) => CSharp(a) + " " + ParameterName(m, i)))}) => default;")) + " }")) + " }";
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
        var local = module.GetType("System.Time.LocalDateTime");
        local.PackingSize = -1;
        local.ClassSize = -1;
        local.Fields.Add(new FieldDefinition("StoredDate", FieldAttributes.Private, module.GetType("System.Time.Date")));
        local.Fields.Add(new FieldDefinition("StoredTime", FieldAttributes.Private, module.GetType("System.Time.TimeOfDay")));
        foreach (var method in local.Methods.Where(m => m.HasThis && !m.IsConstructor))
            method.CustomAttributes.Add(new CustomAttribute(constructor));
        foreach (var (name, field, scalar) in new[] { ("TimeOffset", "StoredSeconds", "Int32"), ("Date", "StoredDayNumber", "Int32"), ("TimeOfDay", "StoredTicks", "Int64"), ("Instant", "StoredTicks", "Int64"), ("Duration", "StoredTicks", "Int64") })
        {
            var type = module.GetType("System.Time." + name);
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
    public static ResultBindings.Binding? BindInternal(MethodReference reference, MethodDefinition definition, string? libraryOwner)
    {
        if (libraryOwner != "System.Time.Instant" || !RuntimeSignatures.IsCore(reference.DeclaringType.Scope)
            || reference.DeclaringType.FullName != "System.Time.LocalDateTime" || reference.Name != "FromUnixTimeTicks") return null;
        var signature = RuntimeSignatures.Match(reference, definition, Type, allowInternal: true);
        if (!definition.IsAssembly || !definition.IsStatic || reference.HasThis || reference.HasGenericParameters
            || !signature.Args.SequenceEqual(new[] { "Int64" }) || signature.Result != "System.Time.LocalDateTime")
            throw new InvalidDataException("Unsupported internal local-time factory.");
        return new("System.Time.LocalDateTime::FromUnixTimeTicks", signature.Args, signature.Result);
    }
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (RuntimeSignatures.IsCore(reference.DeclaringType.Scope) && IsReference(reference.DeclaringType.FullName) && reference.Name == "get_Now")
        {
            var shape = RuntimeSignatures.Match(reference, definition, Type);
            if (!reference.HasThis || shape.Args.Length != 0 || shape.Result != "System.Time.Instant")
                throw new InvalidDataException("Unsupported clock property signature.");
            var owner = reference.DeclaringType.FullName;
            return new(owner + "::get_Now", [owner], shape.Result, Instruction: $"callvirt instance {owner}::get_Now()");
        }
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || !Members.Any(m => "System.Time." + m.Owner == reference.DeclaringType.FullName)) return null;
        var signature = RuntimeSignatures.Match(reference, definition, t => GlobalizationBindings.Type(t) ?? Type(t) ?? GenericUnionBindings.Type(t) ?? ResultBindings.Type(t));
        var member = Members.SingleOrDefault(m => "System.Time." + m.Owner == reference.DeclaringType.FullName && m.Name == reference.Name
            && m.Instance == reference.HasThis && m.Args.SequenceEqual(signature.Args) && m.Result == signature.Result);
        if (member is null || (definition.IsVirtual && !definition.IsFinal && !(member.Owner is "Date" or "TimeOfDay" && member.Name == "ToString" && member.Args.Length == 0 && !definition.IsNewSlot))) throw new InvalidDataException("Unsupported calendar member: " + reference.FullName);
        return new(member.Instance ? AdapterName(member) : "System.Time." + member.Owner + "::" + member.Name,
            member.Instance ? new[] { "System.Time." + member.Owner + "&" }.Concat(member.Args).ToArray() : member.Args, member.Result);
    }
    static string AdapterName(Member member) => "Runtime" + member.Owner + member.Name + (member.Name == "ToString" && member.Args.Length == 1 ? "Culture" : "");
    public static string Adapters => ".function RuntimeNewSystemClock() -> System.Time.SystemClock\nnewobj instance System.Time.SystemClock::.ctor()\nret\n.end\n" + string.Join("\n", Members.Where(m => m.Instance).Select(m =>
        $".function {AdapterName(m)}(System.Time.{m.Owner}& source{string.Concat(m.Args.Select((a, i) => "," + a + " arg" + i))}) -> {m.Result}\nldarg source\n"
        + string.Concat(m.Args.Select((_, i) => $"ldarg arg{i}\n")) + $"call instance System.Time.{m.Owner}::{m.Name}({string.Join(',', m.Args)})\nret\n.end\n"));
}
