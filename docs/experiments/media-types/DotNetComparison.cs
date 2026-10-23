using System;
using System.Linq;
using System.Net.Http.Headers;
using System.Text.Json;

var parseInputs = new[] {
    "application/json", "TEXT/HTML; Charset=\"UTF-8\"", "text/plain; x =a",
    "text/plain; x= a", "text/plain; x=a; X=b", "text/plain; ;",
    "text/plain; q=0.8", "text/*", "application/*+json", "text/plain\r\nX: bad"
};
var pairs = new[] {
    ("text/html; charset=utf-8; x=one", "TEXT/HTML; x=one; charset=UTF-8"),
    ("multipart/form-data; boundary=ABC", "multipart/form-data; boundary=abc"),
    ("text/plain; x=one", "text/plain; x=\"one\""),
    ("text/plain; x=ONE", "text/plain; x=one"),
    ("application/json", "application/problem+json")
};
Console.WriteLine(JsonSerializer.Serialize(new {
    runtime = System.Runtime.InteropServices.RuntimeInformation.FrameworkDescription,
    parsing = parseInputs.Select(input => new { input, accepted = MediaTypeHeaderValue.TryParse(input, out _) }),
    equality = pairs.Select(pair => {
        var left = MediaTypeHeaderValue.Parse(pair.Item1);
        var right = MediaTypeHeaderValue.Parse(pair.Item2);
        return new { left = pair.Item1, right = pair.Item2, equal = left.Equals(right), equalHash = left.GetHashCode() == right.GetHashCode() };
    })
}, new JsonSerializerOptions { WriteIndented = true }));
