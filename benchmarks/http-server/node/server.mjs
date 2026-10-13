import http from 'node:http';

const workload = process.argv[2];
if (!['plain', 'json'].includes(workload)) throw new Error('Choose plain or json');
let closed = 0;
const server = http.createServer((request, response) => {
  // No router or framework. Generate and encode the payload on every request.
  const text = workload === 'plain'
    ? 'Café 🌍'
    : JSON.stringify({ id: 42, name: 'Café team', active: true });
  const body = Buffer.from(text, 'utf8');
  response.writeHead(200, {
    'Content-Type': workload === 'plain' ? 'text/plain; charset=utf-8' : 'application/json; charset=utf-8',
    'Content-Length': body.length,
    'Connection': 'close',
  });
  response.end(body);
});
// Socket closure observes completion after all response bytes have been written.
server.on('connection', socket => socket.once('close', () => {
  if (++closed === 32) server.close(() => console.log('DONE'));
}));
server.listen(0, '127.0.0.1', () => console.log(server.address().port));
setTimeout(() => { throw new Error('Benchmark requests did not complete'); }, 120_000).unref();
