---
uid: N:System.Tasks
---
## Completion and asynchronous composition

Task, Promise and TaskOutcome describe completion, while TaskQueue provides the
current dispatch mechanism. Composition and propagation helpers connect operations;
expected application failures remain values such as Result.

See [Tasks](/features/tasks/) and [callbacks](/docs/callbacks.html) for supported
execution and ownership contracts. A task is not inherently a worker thread, and
future scheduler or green-thread designs are not part of this preview's API promise.
