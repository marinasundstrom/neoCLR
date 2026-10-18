#!/usr/bin/env python3
"""Independent trivial peer for load-generator headroom; not a server comparison."""
import asyncio


async def main():
    completed = 0
    stopping = asyncio.Event()

    async def respond(reader, writer):
        nonlocal completed
        try:
            head = await reader.readuntil(b'\r\n\r\n')
            body = b'' if head.startswith(b'GET /stop ') else b'x' * 10
            writer.write(b'HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: ' +
                         str(len(body)).encode() + b'\r\n\r\n' + body)
            await writer.drain()
            completed += bool(body)
        finally:
            writer.close()
            await writer.wait_closed()

    async def stop(reader, writer):
        writer.close()
        await writer.wait_closed()
        stopping.set()

    server = await asyncio.start_server(respond, '127.0.0.1', 0)
    control = await asyncio.start_server(stop, '127.0.0.1', 0)
    print(server.sockets[0].getsockname()[1], flush=True)
    print(control.sockets[0].getsockname()[1], flush=True)
    async with server, control:
        await stopping.wait()
    print('DONE', flush=True)
    print(completed, flush=True)
    print(0, flush=True)
    print(0, flush=True)


if __name__ == '__main__':
    asyncio.run(main())
