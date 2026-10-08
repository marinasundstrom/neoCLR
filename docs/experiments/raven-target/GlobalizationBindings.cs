using Mono.Cecil;

// Bounded immutable Gregorian/invariant policies; ordinary library classes.
static class GlobalizationBindings
{
    public const string Zoned = "System.Time.ZonedDateTime";
    public const string Zone = "System.Time.TimeZone";
    public const string Calendar = "System.Time.Calendar";
    public const string Culture = "System.Globalization.Culture";
    public const string Format = "System.Globalization.DateTimeFormat";
    public const string Language = "System.Globalization.Language";
    public const string Provider = "System.Globalization.CultureProvider";
    public const string FixedProvider = "System.Globalization.FixedCultureProvider";
    public const string SystemProvider = "System.Globalization.SystemCultureProvider";
    public static bool IsProvider(string name) => name is Provider or FixedProvider or SystemProvider;
    public static bool Converts(string source, string target) => source is FixedProvider or SystemProvider && target == Provider;
    public static bool IsName(string name) => name is Zoned or Zone or Calendar or Culture or Format or Language or Provider or FixedProvider or SystemProvider;
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope) && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public const string Declarations = """
        namespace Time {
        public struct DateTime { }
        public struct LocalTimeMapping { public struct Unique { } public struct Ambiguous { } public struct Skipped { } }
        public sealed class ZonedDateTime {
            private ZonedDateTime() { }
            public static Result<ZonedDateTime,TimeZoneError> Create(Instant instant, TimeZone zone) => default;
            public Instant Instant => default;
            public TimeZone Zone => default;
            public LocalDateTime LocalDateTime => default;
            public TimeOffset Offset => default;
        }
        public sealed class TimeZone {
            private TimeZone() { }
            public static TimeZone Utc => default;
            public static string DatabaseVersion => default;
            public string Id => default;
            public int MinYear => default;
            public int MaxYear => default;
            public static Result<TimeZone,TimeZoneError> Find(string id) => default;
            public static Result<TimeZone,TimeZoneError> GetSystem() => default;
            public Result<TimeOffset,TimeZoneError> GetUtcOffset(Instant instant) => default;
            public Result<ZonedDateTime,TimeZoneError> AtInstant(Instant instant) => default;
            public Result<LocalTimeMapping,TimeZoneError> MapLocal(LocalDateTime local) => default;
        }
        public sealed class Calendar {
            private Calendar() { }
            public static Calendar Gregorian => default;
            public static Calendar Hebrew => default;
            public int MinYear => default;
            public int MaxYear => default;
            public Result<int,InvalidDateError> GetYear(Date date) => default;
            public Result<int,InvalidDateError> GetMonth(Date date) => default;
            public Result<int,InvalidDateError> GetDay(Date date) => default;
            public Result<int,InvalidDateError> GetMonthsInYear(int year) => default;
            public string Id => default;
            public Result<Date,InvalidDateError> CreateDate(int year, int month, int day) => default;
            public Result<bool,InvalidDateError> IsLeapYear(int year) => default;
            public Result<int,InvalidDateError> GetDaysInMonth(int year, int month) => default;
            public Result<Date,InvalidDateError> AddDays(Date date, int days) => default;
            public Result<Date,InvalidDateError> AddMonths(Date date, int months) => default;
            public Result<Date,InvalidDateError> AddYears(Date date, int years) => default;
        }
        }
        namespace Globalization {
            using System.Time;
            public sealed class Language {
                private Language() { }
                public static Language Undetermined => default;
                public static Language Swedish => default;
                public static Language Hebrew => default;
                public string Code => default;
            }
            public interface CultureProvider { Culture Current { get; } }
            public sealed class FixedCultureProvider : CultureProvider {
                public FixedCultureProvider(Culture culture) { }
                public Culture Current => default;
            }
            public sealed class SystemCultureProvider : CultureProvider {
                public SystemCultureProvider() { }
                public static SystemCultureProvider FromPreference(string preference) => default;
                public string PreferredCultureName => default;
                public bool IsFallback => default;
                public Culture Current => default;
            }
            public sealed class Culture {
                private Culture() { }
                public static Culture Invariant => default;
                public static Culture SwedishSweden => default;
                public static Culture HebrewIsrael => default;
                public string Name => default;
                public Language Language => default;
                public static Culture Current => default;
                public System.Time.Calendar DefaultCalendar => default;
                public DateTimeFormat DateTimeFormat => default;
            }
            public sealed class DateTimeFormat {
                private DateTimeFormat() { }
                public static DateTimeFormat Invariant => default;
                public System.Time.Calendar Calendar => default;
                public Culture Culture => default;
                public static DateTimeFormat Create(Culture culture, System.Time.Calendar calendar) => default;
                public DateTimeFormat WithCalendar(System.Time.Calendar calendar) => default;
                public Result<string,InvalidDateError> FormatDate(Date date) => default;
                public string FormatTime(TimeOfDay time) => default;
                public Result<string,InvalidDateError> FormatLocalDateTime(LocalDateTime value) => default;
            }
        }
        """;
    public static CollectionBindings.Binding? Construct(MethodReference reference, MethodDefinition definition)
    {
        var call = Bind(reference, definition, true);
        return call is null ? null : new(call.Arguments, call.Result, call.Instruction!);
    }

    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct = false)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        (string Args, string Result, bool Static) expected = (owner, reference.Name) switch {
            (Language, "get_Undetermined" or "get_Swedish" or "get_Hebrew") => ("", Language, true),
            (Language, "get_Code") => ("", "String", false),
            (Culture, "get_Language") => ("", Language, false),
            (Culture, "get_Current") => ("", Culture, true),
            (Provider or FixedProvider or SystemProvider, "get_Current") => ("", Culture, false),
            (FixedProvider, ".ctor") => (Culture, "noresult", false),
            (SystemProvider, ".ctor") => ("", "noresult", false),
            (SystemProvider, "FromPreference") => ("String", SystemProvider, true),
            (SystemProvider, "get_PreferredCultureName") => ("", "String", false),
            (SystemProvider, "get_IsFallback") => ("", "Boolean", false),
            (Zoned, "Create") => ("System.Time.Instant,System.Time.TimeZone", "System.Result<System.Time.ZonedDateTime,System.Time.TimeZoneError>", true),
            (Zoned, "get_Instant") => ("", "System.Time.Instant", false),
            (Zoned, "get_Zone") => ("", Zone, false),
            (Zoned, "get_LocalDateTime") => ("", "System.Time.LocalDateTime", false),
            (Zoned, "get_Offset") => ("", "System.Time.TimeOffset", false),
            (Zone, "get_Utc") => ("", Zone, true),
            (Zone, "get_DatabaseVersion") => ("", "String", true),
            (Zone, "get_Id") => ("", "String", false),
            (Zone, "get_MinYear" or "get_MaxYear") => ("", "Int32", false),
            (Zone, "Find") => ("String", "System.Result<System.Time.TimeZone,System.Time.TimeZoneError>", true),
            (Zone, "GetSystem") => ("", "System.Result<System.Time.TimeZone,System.Time.TimeZoneError>", true),
            (Zone, "GetUtcOffset") => ("System.Time.Instant", "System.Result<System.Time.TimeOffset,System.Time.TimeZoneError>", false),
            (Zone, "AtInstant") => ("System.Time.Instant", "System.Result<System.Time.ZonedDateTime,System.Time.TimeZoneError>", false),
            (Zone, "MapLocal") => ("System.Time.LocalDateTime", "System.Result<System.Time.LocalTimeMapping,System.Time.TimeZoneError>", false),
            (Calendar, "get_Gregorian" or "get_Hebrew") => ("", Calendar, true),
            (Calendar, "get_Id") or (Culture, "get_Name") => ("", "String", false),
            (Calendar, "get_MinYear" or "get_MaxYear") => ("", "Int32", false),
            (Calendar, "GetYear" or "GetMonth" or "GetDay") => ("System.Time.Date", "System.Result<Int32,System.Time.InvalidDateError>", false),
            (Calendar, "GetMonthsInYear") => ("Int32", "System.Result<Int32,System.Time.InvalidDateError>", false),
            (Calendar, "CreateDate") => ("Int32,Int32,Int32", CalendarBindings.DateResult, false),
            (Calendar, "IsLeapYear") => ("Int32", "System.Result<Boolean,System.Time.InvalidDateError>", false),
            (Calendar, "GetDaysInMonth") => ("Int32,Int32", "System.Result<Int32,System.Time.InvalidDateError>", false),
            (Calendar, "AddDays" or "AddMonths" or "AddYears") => ("System.Time.Date,Int32", CalendarBindings.DateResult, false),
            (Culture, "get_Invariant" or "get_SwedishSweden" or "get_HebrewIsrael") => ("", Culture, true),
            (Culture, "get_DefaultCalendar") or (Format, "get_Calendar") => ("", Calendar, false),
            (Culture, "get_DateTimeFormat") => ("", Format, false),
            (Format, "get_Invariant") => ("", Format, true),
            (Format, "get_Culture") => ("", Culture, false),
            (Format, "Create") => ($"{Culture},{Calendar}", Format, true),
            (Format, "WithCalendar") => (Calendar, Format, false),
            (Format, "FormatDate") => ("System.Time.Date", "System.Result<String,System.Time.InvalidDateError>", false),
            (Format, "FormatTime") => ("System.Time.TimeOfDay", "String", false),
            (Format, "FormatLocalDateTime") => ("System.Time.LocalDateTime", "System.Result<String,System.Time.InvalidDateError>", false),
            _ => throw new InvalidDataException("Unsupported globalization member: " + reference.FullName)
        };
        var contract = owner == Provider;
        var dispatch = IsProvider(owner) && reference.Name == "get_Current";
        if (!definition.IsPublic || definition.HasGenericParameters || definition.IsStatic != expected.Static
            || reference.HasThis == expected.Static || construct != definition.IsConstructor
            || definition.IsVirtual != dispatch || definition.IsAbstract != contract
            || definition.IsNewSlot != dispatch || definition.IsFinal != (dispatch && !contract)
            || definition.DeclaringType.IsInterface != contract || definition.DeclaringType.IsSealed == contract
            || string.Join(',', args) != expected.Args || result != expected.Result)
            throw new InvalidDataException("Unsupported globalization signature: " + reference.FullName);
        return new("", construct || expected.Static ? args : new[] { owner }.Concat(args).ToArray(), construct ? owner : result,
            Instruction: $"{(construct ? "newobj instance " : expected.Static ? "call " : dispatch ? "callvirt instance " : "call instance ")}{owner}::{reference.Name}({string.Join(',', args)})");
    }
}
