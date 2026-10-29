# Initial generic function-method investigation

These are pre-implementation parsing probes using an existing local Raven driver.
The driver was not rebuilt to match inspected source. See [results.json](results.json)
for the exact commands, SHA-256, exit codes and outputs. The explicit-signature
class-library control analyzed successfully; projected syntax did not. Native
probes did not select a matching core and are not evidence of missing func constraints.

Later implementation tests are separate evidence; these historical failures are
retained unchanged.
