import asyncio
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('lazy_load', Path(__file__).with_name('lazy-load.py'))
harness = importlib.util.module_from_spec(spec)
spec.loader.exec_module(harness)


class Contracts(unittest.IsolatedAsyncioTestCase):
    async def test_dependency_really_suspends_and_counts_calls(self):
        dependency = harness.Dependency(.02)
        try:
            began = asyncio.get_running_loop().time()
            reader, writer = await asyncio.open_connection('127.0.0.1', dependency.port)
            writer.write(bytes([42]))
            await writer.drain()
            self.assertEqual(await reader.readexactly(1), bytes([43]))
            self.assertGreaterEqual(asyncio.get_running_loop().time()-began, .02)
            writer.close()
            await writer.wait_closed()
            self.assertEqual(dependency.requests, 1)
        finally:
            dependency.close()

    async def test_fragmented_request_and_checked_body(self):
        seen = []

        async def peer(reader, writer):
            seen.append(await reader.readuntil(b'\r\n'))
            seen.append(await reader.readuntil(b'\r\n\r\n'))
            body = json.dumps({'id': 42, 'name': 'Café team', 'active': True}).encode()
            writer.write(b'HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: '
                         + str(len(body)).encode() + b'\r\n\r\n' + body)
            await writer.drain()
            writer.close()
            await writer.wait_closed()

        server = await asyncio.start_server(peer, '127.0.0.1', 0)
        async with server:
            for gap in [0, .001]:
                with self.subTest(header_gap=gap):
                    seen.clear()
                    result = await harness.exchange(server.sockets[0].getsockname()[1], '/json', 256, 1,
                                                    header_gap=gap)
                    self.assertEqual(result, 'ok')
                    self.assertEqual(seen[0], b'GET /json HTTP/1.1\r\n')
                    self.assertIn(b'Connection: close', seen[1])

    async def test_custom_exchange_preserves_missed_arrivals(self):
        async def slow(*args):
            await asyncio.sleep(.04)
            return 'ok'
        result = await harness.load.window(1, '/json', 256, 100, .15, 1, 1, request_fn=slow)
        self.assertEqual(result['offered'], 15)
        self.assertEqual(sum(result['outcomes'].values()), 15)
        self.assertGreater(result['outcomes']['generator_inflight_limit'], 0)
        self.assertLess(result['successfulOfferedFraction'], 1)


if __name__ == '__main__':
    unittest.main()
