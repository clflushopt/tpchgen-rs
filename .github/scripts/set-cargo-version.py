from pathlib import Path
import os


package = os.environ["PACKAGE"]
version = os.environ["VERSION"].removeprefix("v")

if package == "all":
    raise SystemExit("Version override requires selecting a single package.")

manifest = Path("Cargo.toml") if package == "tpchgen" else Path(package) / "Cargo.toml"
lines = manifest.read_text(encoding="utf-8").splitlines(keepends=True)

for index, line in enumerate(lines):
    if line.startswith("version = "):
        newline = "\n" if line.endswith("\n") else ""
        lines[index] = f'version = "{version}"{newline}'
        break
else:
    raise SystemExit(f"Could not find a version field in {manifest}")

manifest.write_text("".join(lines), encoding="utf-8")
print(lines[index].strip())
