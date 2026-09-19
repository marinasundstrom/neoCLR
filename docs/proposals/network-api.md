# NeoCLR Networking and HTTP Platform Proposal

## Status

Initial design proposal.

## Summary

NeoCLR should provide networking and HTTP as first-class capabilities of the standard platform.

The goal is not to reproduce the .NET networking APIs. Instead, NeoCLR should retain familiar networking concepts while redesigning their APIs around the principles of the new platform:

* asynchronous I/O as the normal execution model;
* runtime-supported task suspension;
* `Result` for recoverable failures;
* portable abstractions independent of operating-system networking APIs;
* common stream abstractions;
* explicit low-level escape hatches;
* modern server-oriented APIs;
* semantic familiarity for developers coming from .NET and other platforms.

The standard platform should include socket and TCP/UDP primitives, DNS and addressing, TLS integration, an HTTP client, and an HTTP server.

Higher-level concepts such as routing, middleware, controllers, endpoint mapping, and `WebApplication` should be built above these primitives rather than being intrinsic parts of the standard networking API.

This gives NeoCLR enough networking infrastructure to build real applications without making a particular web framework part of the runtime.

---

# 1. Goals

The networking platform should make common networking operations straightforward while retaining access to lower-level functionality when necessary.

A developer should be able to:

* resolve a hostname;
* establish TCP connections;
* listen for incoming TCP connections;
* send and receive UDP datagrams;
* perform asynchronous network I/O;
* establish TLS connections;
* make HTTP requests;
* host an HTTP server;
* stream request and response bodies;
* implement protocols directly on top of sockets or streams.

These operations should work naturally with NeoCLR's task and suspension model.

Application code should not normally need to understand whether asynchronous operations are implemented through IOCP, `epoll`, `kqueue`, `io_uring`, or another operating-system mechanism.

---

# 2. Design Philosophy

## 2.1 Semantic familiarity rather than API compatibility

NeoCLR should deliberately use established networking terminology where that terminology remains useful.

Concepts such as:

```text
Socket
IPAddress
IPEndpoint
TcpListener
TcpConnection
Dns
HttpClient
HttpServer
HttpRequest
HttpResponse
```

are already widely understood.

There is little value in renaming these concepts simply because NeoCLR is a new platform.

However, familiarity should not require preserving historical .NET API structures.

NeoCLR should therefore favor:

> Familiar concepts and terminology, redesigned around modern runtime semantics.

A .NET developer encountering `TcpListener` should immediately understand its purpose without NeoCLR having to reproduce every member or supporting abstraction of `System.Net.Sockets.TcpListener`.

---

# 3. Layered Architecture

Networking should consist of several progressively higher-level layers.

```text
                 Application frameworks
                         │
              routing / middleware / RPC
                         │
────────────────────────────────────────────────────
                         │
                       HTTP
                 HttpClient
                 HttpServer
                 HttpRequest
                 HttpResponse
                         │
────────────────────────────────────────────────────
                         │
                  TLS / protocols
                         │
────────────────────────────────────────────────────
                         │
                  TCP / UDP APIs
                 TcpListener
                 TcpConnection
                  UdpSocket
                         │
────────────────────────────────────────────────────
                         │
                      Socket
                         │
────────────────────────────────────────────────────
                         │
               NeoCLR networking runtime
                         │
        ┌────────────────┼────────────────┐
        │                │                │
       IOCP           epoll/io_uring    kqueue
     Windows             Linux          macOS
```

The public API should describe portable networking semantics.

Operating-system-specific mechanisms belong beneath that boundary.

---

# 4. Addressing and DNS

The platform should provide standard representations for network addresses and endpoints.

An initial model could include:

```text
System.Net

IPAddress
IPEndpoint
Dns
AddressFamily
```

For example:

```raven
let addresses = await Dns.Resolve("example.com")?;

for address in addresses {
    Console.WriteLine(address);
}
```

DNS resolution is potentially blocking I/O from the application's perspective and should therefore naturally participate in the asynchronous model.

Recoverable lookup failures should be represented using `Result`, rather than exceptions.

---

# 5. Socket API

NeoCLR should expose sockets as its low-level networking escape hatch.

The socket API exists for applications and libraries that require direct control over:

* protocol selection;
* socket types;
* binding;
* connection establishment;
* socket options;
* sending and receiving;
* shutdown behavior;
* multicast;
* protocol-specific functionality.

Conceptually:

```raven
let socket = Socket(
    AddressFamily.IPv6,
    SocketType.Stream,
    Protocol.Tcp
);

await socket.Connect(endpoint)?;
```

The precise API requires further design.

The important principle is that `Socket` represents the portable socket capability rather than directly exposing one operating system's socket implementation.

Platform-specific functionality can be made available separately where necessary.

---

# 6. TCP

Most applications should not have to manipulate raw sockets.

NeoCLR should therefore provide higher-level TCP abstractions.

An initial model could be:

```text
TcpListener
TcpConnection
TcpClient
```

A server might resemble:

```raven
let listener = TcpListener(IPAddress.Any, 8080);

await listener.Start()?;

while true {
    let connection = await listener.Accept()?;
    spawn HandleConnection(connection);
}
```

Connection establishment might similarly be:

```raven
let connection =
    await TcpClient.Connect("example.com", 443)?;
```

The exact need for a separate `TcpClient` type should be evaluated. Connection establishment could potentially be expressed through `TcpConnection.Connect`.

The API should be chosen according to usability rather than adherence to the .NET type hierarchy.

---

# 7. Runtime Suspension

Networking is one of the primary use cases for NeoCLR's suspension model.

An operation such as:

```raven
let connection = await listener.Accept()?;
```

should conceptually mean:

1. initiate or register the accept operation;
2. suspend the current task;
3. allow the runtime to perform other work;
4. resume the task when the networking backend reports completion.

The public API should not expose implementation mechanisms such as completion ports or readiness notification.

Initially, Raven may still generate async state machines.

The API must nevertheless be designed according to the intended NeoCLR suspension semantics so that moving suspension into the runtime does not require redesigning networking APIs.

---

# 8. Streams

Networking should integrate with the common NeoCLR stream model.

A TCP connection should expose readable and writable byte streams rather than requiring every consumer to depend upon socket-specific operations.

Conceptually:

```raven
interface TcpConnection
{
    LocalEndpoint: IPEndpoint;
    RemoteEndpoint: IPEndpoint;

    Input: ReadStream;
    Output: WriteStream;
}
```

Application code could consequently use:

```raven
let count = await connection.Input.Read(buffer)?;

await connection.Output.Write(data)?;
```

The same fundamental stream abstractions could then be used by:

* files;
* TCP;
* TLS;
* compression;
* process pipes;
* HTTP bodies;
* serialization.

This makes networking an important validation case for the separate NeoCLR Streams API.

The design must account for backpressure, buffering, ownership, cancellation, partial reads and writes, and efficient access to memory.

---

# 9. UDP

UDP should have an abstraction appropriate to datagrams rather than forcing it into stream semantics.

For example:

```raven
let socket = UdpSocket.Bind(endpoint)?;

let datagram = await socket.Receive()?;

Console.WriteLine(datagram.RemoteEndpoint);
```

Sending could resemble:

```raven
await socket.Send(data, endpoint)?;
```

TCP and UDP therefore share addressing and lower-level networking infrastructure without pretending that they have the same communication model.

---

# 10. TLS

TLS should compose with the networking and stream model.

Conceptually:

```text
TcpConnection
      │
      ▼
TlsConnection
      │
      ▼
ReadStream / WriteStream
```

For example:

```raven
let tcp = await TcpClient.Connect(host, 443)?;

let connection =
    await Tls.Connect(tcp, host)?;
```

Higher-level facilities such as `HttpClient` should normally configure TLS automatically.

Direct TLS APIs remain useful for custom protocols.

Certificate validation, trust stores, identities, client certificates, and platform-specific certificate facilities will require a separate security design.

---

# 11. HTTP as a Standard Platform Capability

HTTP should be part of the NeoCLR standard platform.

This should include both client and server functionality.

An initial namespace could contain:

```text
System.Net.Http

HttpClient
HttpServer

HttpRequest
HttpResponse

HttpHeaders
HttpMethod
HttpStatus

HttpRequestBody
HttpResponseBody
```

The exact naming and namespace structure remain open.

HTTP/1.1, HTTP/2, and eventually HTTP/3 should be protocol implementations beneath a common HTTP semantic model where practical.

---

# 12. HTTP Client

A normal HTTP request should require very little infrastructure.

For example:

```raven
let client = HttpClient();

let response =
    await client.Get("https://example.com/data")?;

let text =
    await response.Content.ReadText()?;
```

More explicit requests could use:

```raven
let request = HttpRequest.Post(url)
    .Header("Content-Type", "application/json")
    .Body(content);

let response = await client.Send(request)?;
```

The API should support both convenient buffered operations and streaming without making either model awkward.

Connection pooling, protocol negotiation, DNS, TLS, redirects, compression, and other transport concerns should normally be managed by the platform implementation.

---

# 13. HTTP Server

NeoCLR should provide a portable HTTP server API.

This is deliberately different from making a web application framework part of the platform.

The HTTP server is responsible for HTTP.

It should understand concepts such as:

* listening endpoints;
* connections;
* HTTP requests;
* HTTP responses;
* headers;
* request and response bodies;
* protocol versions;
* connection lifetime;
* TLS;
* timeouts;
* streaming;
* backpressure.

It should **not** inherently understand:

* application routes;
* controllers;
* MVC;
* dependency injection;
* endpoint parameter binding;
* application authorization policies;
* application middleware;
* Razor-like rendering;
* application structure.

Those belong to frameworks built on top of the server.

An API might eventually resemble:

```raven
let server = HttpServer(options);

await server.Run(async request => {
    return HttpResponse.Ok("Hello, world");
});
```

Another possible model is asynchronous request iteration:

```raven
for await context in server.Requests() {
    spawn HandleRequest(context);
}
```

The choice between these models should be made together with the design of async sequences, structured concurrency, cancellation, and resource ownership.

---

# 14. Avoiding the HttpListener Problem

NeoCLR should not tie its public HTTP server abstraction to a particular operating-system HTTP implementation.

Historically, .NET's server APIs evolved under different constraints, and ASP.NET Core ultimately introduced Kestrel as its portable application server.

NeoCLR can establish the abstraction boundary before that ecosystem develops.

```text
             NeoCLR HttpServer
                    │
         standard HTTP semantics
                    │
        ┌───────────┼────────────┐
        │           │            │
    NeoCLR HTTP   native OS    specialized
     backend       backend       backend
```

The standard NeoCLR distribution should provide a high-quality default implementation.

Alternative implementations should remain possible.

The public `HttpServer` contract should therefore not expose implementation details belonging specifically to the default server.

---

# 15. No Required "Kestrel" Layer

NeoCLR will need an efficient HTTP server implementation, but that implementation does not necessarily need to become a separate conceptual layer applications must understand.

From an application's perspective:

```raven
let server = HttpServer(options);
```

should be sufficient.

The standard platform can select and provide its default implementation.

Specialized environments may substitute another implementation where appropriate.

Thus NeoCLR can provide the functionality represented by Kestrel without requiring a historical distinction between "the HTTP APIs" and "the cross-platform web server."

---

# 16. Web Frameworks

A web application framework can then be implemented entirely above the standard HTTP capability.

For example:

```text
Neo.Web

WebApplication
Router
Route
Middleware
Endpoint
Authentication
Authorization
ModelBinding
StaticFiles
...
```

giving Raven applications an ergonomic surface such as:

```raven
let app = WebApplication();

app.Get("/", () => "Hello");

app.Get("/users/{id}", async (id: UserId) => {
    return await users.Get(id)?;
});

await app.Run();
```

Internally:

```text
WebApplication
      │
    Router
      │
  Middleware
      │
  HttpServer
      │
 TCP / TLS
      │
NeoCLR runtime
```

NeoCLR itself therefore does not dictate how web applications must be structured.

Different frameworks can coexist while sharing the same networking and HTTP foundation.

---

# 17. Development Server

The standard platform should make it straightforward to host HTTP during development.

Whether a development-server experience belongs directly to the HTTP package, development tooling, or a higher-level web package can be decided later.

The important requirement is that it should not require an entirely separate networking architecture.

Development and production hosting should ultimately consume the same underlying HTTP server capability.

---

# 18. Error Handling

Expected networking failures should use `Result`.

Examples include:

```raven
await Dns.Resolve(host)?
await TcpClient.Connect(host, port)?
await stream.Read(buffer)?
await client.Send(request)?
await server.Start()?
```

Potential errors include:

```text
DnsError
ConnectionError
SocketError
TlsError
HttpError
```

The exact hierarchy should be designed carefully.

NeoCLR should avoid collapsing every failure into a generic networking exception while also avoiding an excessively complicated taxonomy.

Faults and exceptions should remain reserved for conditions that cannot reasonably be handled through the normal API contract.

---

# 19. Cancellation, Timeouts, and Structured Lifetime

Networking APIs must integrate with NeoCLR's eventual model for cancellation and structured concurrency.

This affects:

```text
DNS lookup
Connect
Accept
Read
Write
TLS handshake
HTTP request
HTTP server shutdown
```

Timeouts should not be independently reinvented by every networking type.

Similarly, server shutdown should have defined semantics for:

* stopping new connections;
* stopping new requests;
* allowing active work to complete;
* cancelling remaining work;
* closing underlying resources.

These concerns should be coordinated with the broader Task and application-hosting design.

---

# 20. Implementation Strategy

The networking stack can initially be implemented while Raven and NeoCLR are still being developed.

A useful progression is:

```text
Raven
  │
  ▼
.NET backend
  │
  ▼
NeoCLR networking API
implemented using .NET
  │
  ▼
real Raven applications
```

This allows the API to be exercised before the native NeoCLR implementation is complete.

Later:

```text
Raven
  │
  ▼
NeoCLR backend
  │
  ▼
NeoCLR networking API
  │
  ▼
native NeoCLR networking runtime
```

The implementation can progressively move away from the .NET substrate without changing the application-facing model.

---

# 21. First Vertical Slice

Networking provides an excellent vertical slice for proving NeoCLR as an application platform.

A useful milestone would be:

```text
Raven source
     │
     ▼
NeoCLR compiler backend
     │
     ▼
NeoCLR IL
     │
     ▼
Task + suspension
     │
     ▼
Socket
     │
     ▼
TCP
     │
     ▼
Streams
     │
     ▼
HTTP server
     │
     ▼
GET /
     │
     ▼
"Hello from NeoCLR"
```

This exercises a significant portion of the platform simultaneously:

* compiler;
* IL;
* runtime;
* tasks;
* suspension;
* memory;
* networking;
* streams;
* text;
* errors;
* resource lifetime;
* HTTP.

Once this works, NeoCLR is capable of hosting a real network application rather than merely executing isolated programs.

---

# 22. Proposed Principle

The networking design can be summarized by one broader NeoCLR library principle:

> **Preserve semantic familiarity while redesigning structural and execution semantics for the platform we are building.**

NeoCLR does not need to discard concepts simply because they also exist in .NET.

Nor should it reproduce .NET's APIs simply because developers already know them.

For networking in particular, NeoCLR should combine familiar concepts from .NET with lessons from modern platforms such as Node.js, Go, Rust and contemporary asynchronous systems, while taking advantage of capabilities those platforms could not necessarily assume when their original APIs were designed.

The result should feel recognizable without feeling inherited.
