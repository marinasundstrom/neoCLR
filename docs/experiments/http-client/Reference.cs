using System.Net.Http;

using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(5));
using var client = new HttpClient(new SocketsHttpHandler { UseProxy = false });
using var request = new HttpRequestMessage(HttpMethod.Get, $"http://localhost:{args[0]}/greeting");
request.Headers.ConnectionClose = true;
using var response = await client.SendAsync(request, HttpCompletionOption.ResponseHeadersRead, deadline.Token);
var text = await response.Content.ReadAsStringAsync(deadline.Token);
if ((int)response.StatusCode != 200 || text != "Café 🌍")
    throw new Exception("Reference exchange failed");
Console.WriteLine("HTTP 200");
Console.WriteLine(text);
