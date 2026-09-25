using System.Net.Http;
using var client = new HttpClient(new SocketsHttpHandler { UseProxy = false, AllowAutoRedirect = false });
foreach (var code in new[] {201, 204, 205, 304, 400, 404, 500, 599}) {
    var url = $"http://127.0.0.1:{args[0]}/{code}";
    using var response = await client.GetAsync(url);
    if ((int)response.StatusCode != code || response.IsSuccessStatusCode != (code is >= 200 and <= 299)) throw new Exception("Status mismatch");
    var expected = code is 204 or 205 or 304 ? "" : "body";
    if (await response.Content.ReadAsStringAsync() != expected) throw new Exception("Body mismatch");
    try {
        var text = await client.GetStringAsync(url);
        if (code >= 300 || text != expected) throw new Exception("Text success policy mismatch");
    } catch (HttpRequestException error) when (code >= 300 && (int?)error.StatusCode == code) { }
}
Console.WriteLine(".NET status and GetString baseline passed");
