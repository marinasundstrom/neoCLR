import os
import sys
import unittest
from process_memory import parse_snapshot, snapshot


class ProcessMemoryTests(unittest.TestCase):
    def test_units_and_identity(self):
        self.assertEqual(parse_snapshot(' 42 100 200\n', 42),
                         dict(pid=42, rssBytes=102400, virtualBytes=204800))

    def test_bad_rows_rejected(self):
        for text in ['', '41 100 200', '42 -1 200', '42 100 -1',
                     '42 100', '42 100 200 300', '42 1 2\n42 1 2\n', '42 x 2']:
            with self.subTest(text=text), self.assertRaises(ValueError):
                parse_snapshot(text, 42)

    @unittest.skipUnless(sys.platform == 'darwin', 'macOS process snapshot')
    def test_live_owned_process(self):
        result = snapshot(os.getpid())
        self.assertGreater(result['rssBytes'], 0)
        self.assertGreater(result['virtualBytes'], 0)
        self.assertEqual(result['exitCode'], 0)


if __name__ == '__main__':
    unittest.main()
