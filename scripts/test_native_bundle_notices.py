"""A replaced backend must not inherit stale source-bundle notices."""
import hashlib
from pathlib import Path
import tempfile
import unittest
from native_bundle_notices import refresh_notices

ROOT = Path(__file__).resolve().parents[1]


class BundleNoticesTests(unittest.TestCase):
    def test_refreshes_stale_inventory_preserves_payload_and_records_sources(self):
        with tempfile.TemporaryDirectory() as folder:
            bundle = Path(folder)
            (bundle / 'THIRD_PARTY_NOTICES.md').write_text('old bundle')
            (bundle / 'program.dll').write_bytes(b'keep caller payload')
            hashes = refresh_notices(ROOT, bundle)
            self.assertEqual((bundle / 'program.dll').read_bytes(), b'keep caller payload')
            for relative in ('THIRD_PARTY_NOTICES.md', 'third-party/manifest.json',
                             'tools/aot-poc/CRANELIFT-LICENSE', 'tools/aot-poc/Cargo.lock',
                             'tools/native-tls/THIRD_PARTY_NOTICES.md'):
                source = ROOT / relative
                self.assertEqual((bundle / relative).read_bytes(), source.read_bytes())
                self.assertEqual(hashes[str(source)], hashlib.sha256(source.read_bytes()).hexdigest())
            for source, digest in hashes.items():
                self.assertEqual(hashlib.sha256((bundle / Path(source).relative_to(ROOT)).read_bytes()).hexdigest(), digest)


if __name__ == '__main__':
    unittest.main()
