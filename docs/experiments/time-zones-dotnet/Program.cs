// Independent .NET 10 comparison for the executed modern transition cases.
var cases = new[] {
    ("Europe/Stockholm", new DateTime(2024,3,31,2,30,0), 0),
    ("Europe/Stockholm", new DateTime(2024,10,27,2,30,0), 2),
    ("America/New_York", new DateTime(2024,11,3,1,30,0), 2),
    ("Australia/Lord_Howe", new DateTime(2024,4,7,1,45,0), 2),
    ("Asia/Jerusalem", new DateTime(2024,1,1,12,0,0), 1)
};
foreach (var (id, local, count) in cases) {
    var zone = TimeZoneInfo.FindSystemTimeZoneById(id);
    var actual = zone.IsInvalidTime(local) ? 0 : zone.IsAmbiguousTime(local) ? 2 : 1;
    if (actual != count) throw new Exception(id);
    var offsets = actual == 0 ? [] : actual == 2 ? zone.GetAmbiguousTimeOffsets(local) : new[] {zone.GetUtcOffset(local)};
    var ticks = offsets.Select(offset => new DateTimeOffset(local, offset).UtcTicks - DateTime.UnixEpoch.Ticks).Order().ToArray();
    Console.WriteLine($"{id} {local:yyyy-MM-ddTHH:mm:ss} count={actual} ticks={string.Join(',',ticks)}");
}
var stockholm = TimeZoneInfo.FindSystemTimeZoneById("Europe/Stockholm");
var overlap = new DateTime(2024,10,27,2,30,0);
if (TimeZoneInfo.ConvertTimeToUtc(overlap, stockholm) != new DateTime(2024,10,27,1,30,0,DateTimeKind.Utc)) throw new Exception("Unexpected default resolution");
Console.WriteLine("Timezone comparisons passed");
