using System.Net;
using System.Net.Http;
using System.Text;

using var handler = new ProbeHandler();
using var client = new HttpClient(handler);
using var source = new CancellationTokenSource();
var pending = client.GetStringAsync("http://example.test/", source.Token);
source.Cancel();
try {
    await pending;
    throw new Exception("Cancellation was lost");
} catch (OperationCanceledException) {
    if (!handler.Cleaned || !handler.SawCancelable) throw new Exception("Handler acknowledgement was lost");
}
handler.Pending = false;
if (await client.GetStringAsync("http://example.test/") != "Café 🌍") throw new Exception("Text mismatch");
handler.Status = HttpStatusCode.NotFound;
try {
    await client.GetStringAsync("http://example.test/");
    throw new Exception("Non-success status accepted");
} catch (HttpRequestException error) when (error.StatusCode == HttpStatusCode.NotFound) { }
Console.WriteLine(".NET 10: GetStringAsync forwards cancellation, reads UTF-8 and rejects non-success status");

sealed class ProbeHandler : HttpMessageHandler {
    public bool Pending = true;
    public bool Cleaned;
    public bool SawCancelable;
    public HttpStatusCode Status = HttpStatusCode.OK;
    protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken token) {
        SawCancelable = token.CanBeCanceled;
        if (Pending) {
            var completion = new TaskCompletionSource<HttpResponseMessage>();
            using var registration = token.Register(() => {
                Cleaned = true;
                completion.TrySetCanceled(token);
            });
            return await completion.Task;
        }
        return new HttpResponseMessage(Status) { Content = new StringContent("Café 🌍", Encoding.UTF8) };
    }
}
