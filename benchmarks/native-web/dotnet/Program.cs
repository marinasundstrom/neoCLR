using System.Text;

// Comparison candidate for docs/experiments/http-server/Server.rvn.
// Valid GET /greeting only. Invalid-request semantics are not yet matched.
var builder = WebApplication.CreateSlimBuilder(args);
var app = builder.Build();
var greeting = Encoding.UTF8.GetBytes("Café 🌍");
app.MapGet("/greeting", async context =>
{
    context.Response.StatusCode = StatusCodes.Status200OK;
    context.Response.ContentType = "text/plain; charset=utf-8";
    context.Response.ContentLength = greeting.Length;
    context.Response.Headers.Connection = "close";
    await context.Response.Body.WriteAsync(greeting);
});
app.Run();
