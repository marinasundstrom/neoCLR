using System.Net;
using System.Net.Http;
using System.Text;
using var client = new HttpClient(new SocketsHttpHandler { UseProxy = false });
byte[][] bodies = [Encoding.UTF8.GetBytes("Café 🌍"), [0, 255, 13, 10], []];
foreach (var body in bodies) {
    using var response = await client.PostAsync($"http://127.0.0.1:{args[0]}/echo", new ByteArrayContent(body));
    if (response.StatusCode != HttpStatusCode.Created || !(await response.Content.ReadAsByteArrayAsync()).SequenceEqual(body))
        throw new Exception("POST echo mismatch");
}
Console.WriteLine(".NET POST interoperability passed");
