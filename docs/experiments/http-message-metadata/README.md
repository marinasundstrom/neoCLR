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
