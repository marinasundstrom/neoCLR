# Node.js HTTP comparison fixture

A retained dependency-free Node.js application for the [HTTP comparison matrix](../platforms.md).
It uses `http.createServer` directly, without Express, routing or middleware.
Each request creates the plain text or serializes a fresh object with JSON.stringify,
then encodes a Buffer as UTF-8. Response bytes are not cached. Native Date-header
behavior remains, as with Go and Kestrel.

```sh
python3 benchmarks/http-server/node/prepare.py --output target/node-comparison-build
```

Preparation checks syntax and retains exact Node/V8 versions and runtime/source
hashes. No npm install is needed. `node server.mjs plain|json` prints its ephemeral
port, serves 32 connections, then prints DONE after closing the server. The common
controller validates each response. This entry uses default V8 execution, not
`--jitless`, snapshots or an AOT variant. The recorded pilot uses Node 22.21.0;
future runtime upgrades create new versioned results.

Primary contract: [Node HTTP server](https://nodejs.org/docs/latest-v22.x/api/http.html#httpcreateserveroptions-requestlistener).
