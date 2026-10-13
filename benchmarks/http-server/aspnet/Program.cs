using System.Net;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.AspNetCore.Hosting.Server;
using Microsoft.AspNetCore.Hosting.Server.Features;
using Microsoft.AspNetCore.Server.Kestrel.Core;

if (args.Length != 1 || args[0] is not ("plain" or "json"))
    throw new ArgumentException("Choose plain or json");
var workload = args[0];
var builder = WebApplication.CreateSlimBuilder();
builder.Logging.ClearProviders();
builder.WebHost.ConfigureKestrel(options =>
{
    options.AddServerHeader = false;
    options.Listen(IPAddress.Loopback, 0, listen => listen.Protocols = HttpProtocols.Http1);
});
var app = builder.Build();
var completed = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
var requests = 0;
// The same generated serializer is used in JIT and AOT. Encode UTF-8 per request,
// including plain text, matching the work performed by the Raven handlers.
var json = new PersonJson(new JsonSerializerOptions { Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping });
// Terminal delegate: no endpoint routing, binding, filters or controllers.
app.Run(async context =>
{
    var text = workload == "plain" ? "Café 🌍" :
        JsonSerializer.Serialize(new Person { id = 42, name = "Café team", active = true }, json.Person);
    var bytes = Encoding.UTF8.GetBytes(text);
    context.Response.ContentType = workload == "plain" ? "text/plain; charset=utf-8" : "application/json; charset=utf-8";
    context.Response.ContentLength = bytes.Length;
    context.Response.Headers.Connection = "close";
    // Observe completed writes before ending this fixed-count benchmark process.
    context.Response.OnCompleted(() =>
    {
        if (Interlocked.Increment(ref requests) == 32)
            completed.TrySetResult();
        return Task.CompletedTask;
    });
    await context.Response.Body.WriteAsync(bytes);
});
await app.StartAsync();
var addresses = app.Services.GetRequiredService<IServer>().Features.Get<IServerAddressesFeature>()!;
Console.WriteLine(new Uri(addresses.Addresses.Single()).Port);
await completed.Task.WaitAsync(TimeSpan.FromSeconds(120));
await app.StopAsync();
await app.DisposeAsync();
Console.WriteLine("DONE");

public sealed class Person
{
    public int id { get; set; }
    public string name { get; set; } = "";
    public bool active { get; set; }
}

[JsonSerializable(typeof(Person))]
internal partial class PersonJson : JsonSerializerContext { }
