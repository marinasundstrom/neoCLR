using System.Globalization;
using System.Text.Json;

var calendar = new HebrewCalendar();
var rows = new List<int[]>();
for (var year = 5344; year <= 5999; year++)
{
    var months = calendar.GetMonthsInYear(year);
    var start = DateOnly.FromDateTime(calendar.ToDateTime(year, 1, 1, 0, 0, 0, 0)).DayNumber;
    var row = new List<int> { year, start };
    for (var month = 1; month <= months; month++) row.Add(calendar.GetDaysInMonth(year, month));
    rows.Add(row.ToArray());
}
if (args.Length == 1) File.WriteAllText(args[0], JsonSerializer.Serialize(rows) + "\n");
void Check(bool ok) { if (!ok) throw new Exception("Calendar comparison failed"); }
Check(new DateOnly(2024,1,31).AddMonths(1) == new DateOnly(2024,2,29));
Check(new DateOnly(2024,2,29).AddYears(1) == new DateOnly(2025,2,28));
Check(calendar.GetMonth(calendar.AddYears(calendar.ToDateTime(5784,13,29,0,0,0,0),1)) == 12);
Check(calendar.GetDayOfMonth(calendar.AddMonths(calendar.ToDateTime(5784,6,30,0,0,0,0),1)) == 29);
foreach (var name in new[] { "sv-SE", "he-IL" }) {
    var culture = CultureInfo.GetCultureInfo(name);
    Console.WriteLine($"{name}: {culture.Calendar.GetType().Name}; short date {culture.DateTimeFormat.ShortDatePattern}; decimal {culture.NumberFormat.NumberDecimalSeparator}");
}
Console.WriteLine($"Compared {rows.Count} complete Hebrew years; last month count {rows[^1].Length - 2}.");
