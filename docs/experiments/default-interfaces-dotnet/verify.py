"""Run the pinned positive probe and verify three expected compiler rejections."""
from pathlib import Path
import subprocess

project = Path(__file__).resolve().parent

def run(*args):
    return subprocess.run(["dotnet", *args], cwd=project, text=True,
                          stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

sdk = run("--version")
assert sdk.returncode == 0 and sdk.stdout.strip() == "10.0.100", sdk.stdout
positive = run("run")
assert positive.returncode == 0, positive.stdout
print(positive.stdout.strip())
for symbol, diagnostic in [("AMBIGUOUS", "CS8705"), ("REABSTRACT", "CS0535"),
                           ("CLASSACCESS", "CS1061")]:
    negative = run("build", "--no-restore", f"-p:DefineConstants={symbol}")
    assert negative.returncode != 0 and diagnostic in negative.stdout, negative.stdout
    print(f"{symbol}: rejected with {diagnostic}")
