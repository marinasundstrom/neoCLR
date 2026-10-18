import asyncio
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch
spec = importlib.util.spec_from_file_location('http_load', Path(__file__).with_name('run.py'))
load = importlib.util.module_from_spec(spec)
spec.loader.exec_module(load)


class LoadContract(unittest.IsolatedAsyncioTestCase):
    async def test_slow_service_does_not_reduce_offered_denominator(self):
        async def slow(*args):
            await asyncio.sleep(.03)
            return 'ok'
        with patch.object(load, 'request', slow):
            report = await load.window(1, '/', 1, 100, .15, 1, 1)
        self.assertEqual(report['offered'], 15)
        self.assertEqual(len(report['requests']), 15)
        self.assertEqual(sum(report['outcomes'].values()), 15)
        self.assertGreater(report['outcomes']['generator_inflight_limit'], 0)
        self.assertEqual(report['admitted'], report['outcomes']['ok'])
        self.assertLess(report['successfulOfferedFraction'], 1)
        self.assertEqual([r['intendedSeconds'] for r in report['requests']],
                         sorted(r['intendedSeconds'] for r in report['requests']))
        for r in report['requests']:
            if r['result'] == 'ok':
                self.assertGreaterEqual(r['offeredLatencySeconds'], r['serviceLatencySeconds'])

    async def test_invalid_body_is_failure_even_with_correct_status_and_length(self):
        async def peer(reader, writer):
            await reader.readuntil(b'\r\n\r\n')
            writer.write(b'HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 1\r\n\r\ny')
            await writer.drain()
            writer.close()
            await writer.wait_closed()
        server = await asyncio.start_server(peer, '127.0.0.1', 0)
        async with server:
            outcome = await load.request(server.sockets[0].getsockname()[1], '/', 1, 1)
        self.assertEqual(outcome, 'invalid_body')

    async def test_diagnostics_separate_connection_and_response_wait(self):
        async def peer(reader, writer):
            await reader.readuntil(b'\r\n\r\n')
            await asyncio.sleep(.03)
            writer.write(b'HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 1\r\n\r\nx')
            await writer.drain()
            writer.close()
            await writer.wait_closed()
        server = await asyncio.start_server(peer, '127.0.0.1', 0)
        async with server:
            report = await load.window(server.sockets[0].getsockname()[1], '/', 1,
                                       10, .1, 2, 1, diagnostics=True)
        record = report['requests'][0]
        self.assertEqual(record['result'], 'ok')
        phases = record['phases']
        ordered = [phases[key] for key in ['connectedSeconds', 'sentSeconds',
                   'firstByteSeconds', 'eofSeconds', 'closedSeconds']]
        self.assertEqual(ordered, sorted(ordered))
        self.assertGreaterEqual(phases['firstByteSeconds'] - phases['sentSeconds'], .025)
        self.assertEqual(record['inflightAtAdmission'], 1)
        self.assertGreaterEqual(record['serviceLatencySeconds'], phases['closedSeconds'])

    async def test_transport_failures_are_counted(self):
        async def fail(*args):
            return 'timeout'
        with patch.object(load, 'request', fail):
            report = await load.window(1, '/', 1, 50, .1, 2, 1)
        self.assertEqual(report['outcomes'], {'timeout': 5})
        self.assertEqual(report['successfulOfferedFraction'], 0)
        self.assertEqual(report['completedWithinWindow'], 0)
        self.assertIsNone(report['offeredLatencySeconds']['p99'])


if __name__ == '__main__':
    unittest.main()
