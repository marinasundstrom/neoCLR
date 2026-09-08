"""Run the pinned .NET delegate comparison, including compiler rejection probes."""
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parent

def run(*args):
    return subprocess.run(["dotnet", *args], cwd=root, text=True, capture_output=True)

positive = run("run", "--project", "Probe.csproj", "-c", "Release")
if positive.returncode:
    raise SystemExit(positive.stdout + positive.stderr)
print(positive.stdout.strip())
for symbol, diagnostic in [("BAD_REF_CONTRACT", "CS0123"), ("BAD_REF_CAPTURE", "CS1628")]:
    result = run("build", "Probe.csproj", "-c", "Release", "--no-restore", f"-p:DefineConstants={symbol}")
    if result.returncode == 0 or diagnostic not in result.stdout + result.stderr:
        raise SystemExit(f"Unexpected result for {symbol}:\n{result.stdout}{result.stderr}")
    print(f"{symbol}: rejected with {diagnostic}")
