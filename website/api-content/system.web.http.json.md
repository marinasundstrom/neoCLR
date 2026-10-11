---
uid: N:System.Web.Http.Json
---
## JSON over HTTP

JsonContent and HttpClientJsonExtensions connect HTTP content and client operations
to System.Data.Json. HttpJsonError retains the distinction between HTTP failures,
JSON failures and unsuccessful response status.

See [Web](/features/web/) and [JSON](/docs/json.html). These helpers retain the
underlying HTTP and JSON bounds and supported mapping shapes; they do not add a
separate transport or unrestricted object serializer.

Development content reads are available directly through HttpContentJsonExtensions:
`ReadFromJson<T>`, `ReadFromJson(TypeInfo)` and `ReadJsonNode`, each with a cancellation
overload. They return tasks containing Result values, preserve JsonError and do not
consume buffered content. Pre-cancellation skips conversion. Conversion remains
synchronous, strict UTF-8 and bounded by the serializer; streams are unsupported.

HttpContextJsonExtensions.RespondJson configures status 200 or an explicit status
with JSON content. Serialization failure leaves the response unchanged. It does not
send; Complete retains responsibility for status/body validation and sending.
