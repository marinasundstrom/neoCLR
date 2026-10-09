# Deferred compiler candidate: pattern local across await

Discovered while validating retained HTTP on 2026-10-09 with Raven bundle compiler
`71cafd353`. This is a general async emission candidate, not an experimental native
Function rule. No compiler fix is included in this slice.

The first interpreter consumer used a synchronous factory returning
`Result<HttpSession, HttpError>` and this Main:

```raven
async func Main() -> Task<()> {
    if let Ok(session) = CreateSession() {
        await session.Exchange()
        await session.Exchange()
        await session.Exchange()
        session.Close()
    } else {
        System.Fail("Retained HTTP listen failed")
    }
}
```

The emitted Main state machine extracts the case payload into local 3 at instruction
60, then loads field 2 of the state-machine receiver at instructions 61–62 before
calling Exchange at 63. No write initializes that hoisted field. Exchange receives
null and faults reading its captured receiver's server field at instruction 21.
Both the pinned bundle interpreter and the available development interpreter produce
that NullReference. The native consumer starts from Bootstrap and does not execute
Main, explaining why the initial native run passed.

Local investigation artifacts: `target/retained-http-validation/native output/app.dll`
and the failed report in `target/retained-http-validation/report.json` (ephemeral,
not published qualification evidence). The inspected native metadata supports an
emission defect; this has not yet been reduced and verified independently on Raven's
shared compiler line or against ordinary .NET execution.

The final consumer has a synchronous factory returning HttpSession (failing terminally
if listening fails), followed by `let session = CreateSession()` in async Main. This
is also appropriate for the sample's terminal-startup policy. It does not fix pattern
local hoisting. Follow-up: reduce the case on Raven's shared line, add an execution
regression, fix the binding-to-hoisted-field assignment and validate .NET and native
emission independently before updating the compiler bundle. Keep this candidate
explicit until that work is complete.
