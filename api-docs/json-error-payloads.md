# JSON error payloads

The [JsonError union](xref:System.Data.Json.JsonError) exposes these case payloads.
The pinned RavenDoc build renders case signatures but does not yet generate
individual payload-property pages. This reference preserves those members until
that renderer support is available. Prefer case patterns to extract payloads.

## Syntax.Message

<a id="syntax-message"></a>

**Property value:** `string`

Diagnostic message for malformed JSON.

## LimitExceeded.Message

<a id="limitexceeded-message"></a>

**Property value:** `string`

Description of the exceeded resource limit.

## TypeMismatch.Expected

<a id="typemismatch-expected"></a>

**Property value:** `string`

Description of the expected JSON value type.

## MissingField.Name

<a id="missingfield-name"></a>

**Property value:** `string`

Name of the required field.

## DuplicateField.Name

<a id="duplicatefield-name"></a>

**Property value:** `string`

Name of the duplicate field.

## InvalidIndex.Index

<a id="invalidindex-index"></a>

**Property value:** `int`

Requested array index.

## Read.Reason

<a id="read-reason"></a>

**Property value:** [TextReadError](xref:System.IO.TextReadError)

Underlying text-input error.

## Write.Reason

<a id="write-reason"></a>

**Property value:** [StreamError](xref:System.IO.StreamError)

Underlying stream-output error.

## Reflection.Reason

<a id="reflection-reason"></a>

**Property value:** [ReflectionError](xref:System.Runtime.Reflection.ReflectionError)

Underlying reflection error.

## UnsupportedMapping.Message

<a id="unsupportedmapping-message"></a>

**Property value:** `string`

Description of the unsupported object mapping.
