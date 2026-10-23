using System.Net;
using System.Net.Sockets;
using Microsoft.AspNetCore.Hosting.Server;
using Microsoft.AspNetCore.Hosting.Server.Features;

var builder = WebApplication.CreateSlimBuilder(args);
builder.Logging.ClearProviders();
builder.WebHost.ConfigureKestrel(options => options.Listen(IPAddress.Loopback, 0,
    listen => listen.Protocols = Microsoft.AspNetCore.Server.Kestrel.Core.HttpProtocols.Http1));
var app = builder.Build();
byte[] body = "xxxxxxxxxx"u8.ToArray();
app.Run(async context =>
{
    context.Response.ContentType = "text/plain; charset=utf-8";
    context.Response.ContentLength = body.Length;
    await context.Response.Body.WriteAsync(body);
});
using var control = new TcpListener(IPAddress.Loopback, 0);
control.Start();
await app.StartAsync();
var addresses = app.Services.GetRequiredService<IServer>().Features.Get<IServerAddressesFeature>()!;
var port = new Uri(addresses.Addresses.Single()).Port;
Console.WriteLine($"READY {port} {((IPEndPoint)control.LocalEndpoint).Port}");
using var signal = await control.AcceptSocketAsync();
await app.StopAsync();
