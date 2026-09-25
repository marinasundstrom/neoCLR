using System;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text;
using var client = new HttpClient();
var url = "http://127.0.0.1:" + args[0] + "/item";
var bodies = new[] { Encoding.UTF8.GetBytes("Café"), new byte[] { 0, 255 }, Array.Empty<byte>(), Array.Empty<byte>() };
var methods = new[] { HttpMethod.Put, HttpMethod.Patch, HttpMethod.Delete, HttpMethod.Head };
for (var i = 0; i < methods.Length; i++) {
    using var request = new HttpRequestMessage(methods[i], url);
    if (i < 2) request.Content = new ByteArrayContent(bodies[i]);
    using var response = await client.SendAsync(request);
    if (i == 3 && response.Content.Headers.ContentLength != 5) throw new Exception("HEAD representation length mismatch");
    if (response.StatusCode != (i == 2 ? HttpStatusCode.NoContent : HttpStatusCode.OK) ||
        response.Headers.GetValues("X-Method").Single() != methods[i].Method ||
        !(await response.Content.ReadAsByteArrayAsync()).SequenceEqual(bodies[i]))
        throw new Exception("Verb response mismatch");
}
Console.WriteLine(".NET verb interoperability passed");
