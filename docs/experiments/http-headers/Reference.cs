using System;
using System.Linq;
using System.Net.Http;

using var response = new HttpResponseMessage();
response.Headers.TryAddWithoutValidation("Set-Cookie", new[] { "a=1; Expires=Wed, 21 Oct 2030 07:28:00 GMT", "b=2" });
response.Headers.TryAddWithoutValidation("X-Empty", "");
response.Headers.TryAddWithoutValidation("X-List", new[] { "one, two", "three" });
if (!response.Headers.TryGetValues("sEt-CoOkIe", out var cookies) || cookies.Count() != 2 || cookies.Last() != "b=2")
    throw new Exception("Cookie lookup changed");
if (!response.Headers.TryGetValues("x-empty", out var empty) || !empty.SequenceEqual(new[] { "" }))
    throw new Exception("Empty value lost");
if (!response.Headers.TryGetValues("X-LIST", out var lists) || !lists.SequenceEqual(new[] { "one, two", "three" }))
    throw new Exception("Extension field values changed");
if (response.Headers.TryGetValues("missing", out _) || response.Headers.TryGetValues(" X-List", out _))
    throw new Exception("Missing or invalid name matched");
Console.WriteLine(".NET header lookup comparison passed");
