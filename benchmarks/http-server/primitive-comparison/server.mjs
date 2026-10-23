import http from 'node:http';
import net from 'node:net';
const body = Buffer.from('xxxxxxxxxx');
const server = http.createServer((request, response) => {
  response.writeHead(200, {'Content-Length': body.length, 'Content-Type': 'text/plain; charset=utf-8'});
  response.end(body);
});
const control = net.createServer(socket => {
  socket.end();
  control.close();
  server.close();
});
await new Promise(resolve => control.listen(0, '127.0.0.1', resolve));
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
console.log(`READY ${server.address().port} ${control.address().port}`);
