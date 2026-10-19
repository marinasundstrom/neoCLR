# HTTP message metadata and ownership

Development after Preview 16. HttpRequest and HttpResponse expose a compact
HttpVersion (currently HTTP/1.1 only). Both implement Disposable; the context
disposes incoming and outgoing content. Header-derived requests share content
and disposal; leaveOpen streams retain external ownership.

The header/value suite passes nine tests in interpreter/JIT/AOT (27 executions),
and the client lifetime suite passes eleven in each mode (33 executions). The
initial combined report records the 33 successes followed by a test-only wrong
factory arity during discovery of the value suite. The corrected value report
passes all modes. No failed case is treated as a success.

All nine context lifecycle cases pass, including cancellation, close, inline
shutdown and late content disposal. The headers-first probe also verifies that
request Version is available before the peer sends its body: all 24 context/Accept cases pass across three modes.

Four API snapshot checks and the full 4532-page local website build pass.
Publication and a new release remain separate operations.

## Client response reason phrases

`ReasonPhrase.rvn` and the independent Python peer verify all four client response
metadata properties in AOT, JIT and interpreter (21 response cases). Coverage
includes exact tabs/trailing spaces, empty and cached OK phrases, informational
responses followed by final status, error statuses, access after disposal and a
withheld body released only after metadata is observed. Local responses use OK
for 200 and an empty phrase otherwise, matching the current writer.

Custom incoming phrases decode once on first access, releasing their retained
bounded header buffer. Until access or collection that buffer remains alive;
this is an allocation/retention tradeoff, not a measured throughput improvement.
The current parser accepts ASCII reason text and tabs, not extended bytes.
Native documentation audit, reference fingerprints and all four native snapshot
checks pass. See `evidence/reason-phrase.json` for inputs and three-mode results.
