"""Keep development JIT bundle notices aligned with the source used to build it."""
import hashlib
from pathlib import Path
import shutil


def refresh_notices(root, bundle):
    root, bundle = Path(root), Path(bundle)
    paths = [root / 'LICENSE', root / 'THIRD_PARTY_NOTICES.md', root / 'Cargo.lock']
    paths += sorted(path for path in (root / 'third-party').rglob('*') if path.is_file())
    for directory, names in (
        ('tools/aot-poc', ('THIRD_PARTY_NOTICES.md', 'CRANELIFT-LICENSE')),
        ('tools/native-tls', ('THIRD_PARTY_NOTICES.md',)),
    ):
        paths += [root / directory / name for name in names]
    hashes = {}
    for source in paths:
        destination = bundle / source.relative_to(root)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
        hashes[str(source.resolve())] = hashlib.sha256(source.read_bytes()).hexdigest()
    return hashes
