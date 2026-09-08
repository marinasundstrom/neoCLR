static void Check(bool ok) { if (!ok) throw new Exception("date/time mismatch"); }
Check(default(DateOnly).DayNumber == 0 && default(DateOnly).Year == 1);
Check(default(TimeOnly).Ticks == 0);
Check(DateOnly.MaxValue.DayNumber == 3652058);
Check(TimeOnly.MaxValue.Ticks == 863999999999L);
Check(new DateOnly(2000, 2, 29).DayOfYear == 60);
try { _ = new DateOnly(1900, 2, 29); throw new Exception("expected invalid date"); }
catch (ArgumentOutOfRangeException) { }
try { _ = new TimeOnly(24, 0); throw new Exception("expected invalid time"); }
catch (ArgumentOutOfRangeException) { }
Check(new TimeOnly(863999999999L).Hour == 23);
var cases = new List<int[]>();
foreach (var year in new[] { 1, 4, 100, 400, 1900, 2000, 2024, 9999 })
for (int month = 1; month <= 12; month++)
foreach (var day in new[] { 1, DateTime.DaysInMonth(year, month) }) {
    var date = new DateOnly(year, month, day);
    cases.Add(new[] { year, month, day, date.DayNumber, date.DayOfYear });
    Check(DateOnly.FromDayNumber(date.DayNumber) == date);
}
if (args.Length == 1) File.WriteAllText(args[0], System.Text.Json.JsonSerializer.Serialize(cases));
Console.WriteLine($"Date/time boundaries and {cases.Count} Gregorian cases passed.");
